use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
};

use ethers::types::Address;
use mysql::{prelude::Queryable, Conn};

use crate::{address_str::get_address_strings, referral::ReferralData};

pub async fn store_user_data(
    conn: &mut Conn,
    addr: &Address,
    avatar: &String,
    referral_code: &String,
) -> Result<(), Box<dyn Error>> {
    check_conn(conn);
    let (address, trunc_address) = get_address_strings(addr);
    println!("{}, {}", address, trunc_address);
    let res: Option<String> = conn.exec_first(
        "SELECT address FROM whitelisted_addresses WHERE address = ? OR address = ?",
        (&address, trunc_address),
    )?;
    if res == None {
        conn.exec_drop(
            "INSERT INTO whitelisted_addresses (address, avatar, referral_code) VALUES (?, ?, ?)",
            (address, avatar, referral_code),
        )?;
    }
    Ok(())
}

pub async fn update_avatar(
    conn: &mut Conn,
    addr: &Address,
    avatar: &String,
) -> Result<(), Box<dyn Error>> {
    check_conn(conn);
    let (address, trunc_address) = get_address_strings(addr);
    conn.exec_drop(
        "UPDATE whitelisted_addresses SET avatar = ? WHERE address = ? OR address = ?",
        (avatar, address, trunc_address),
    )?;
    Ok(())
}

pub async fn update_referred_from(
    conn: &mut Conn,
    addr: &Address,
    referred_from: &String,
) -> Result<(), Box<dyn Error>> {
    check_conn(conn);
    let (address, trunc_address) = get_address_strings(addr);
    conn.exec_drop(
        "UPDATE whitelisted_addresses SET referred_from = ? WHERE address = ? OR address = ?",
        (referred_from, address, trunc_address),
    )?;
    Ok(())
}

pub async fn is_address_whitelisted(
    conn: &mut Conn,
    addr: &Address,
) -> Result<bool, Box<dyn Error>> {
    check_conn(conn);
    let (address, trunc_address) = get_address_strings(addr);
    let res: Option<String> = conn.exec_first(
        "SELECT address FROM whitelisted_addresses WHERE address = ? OR address = ?",
        (address, trunc_address),
    )?;
    if let Some(_) = res {
        return Ok(true);
    }

    Ok(false)
}

pub async fn is_avatar_available(
    conn: &mut Conn,
    addr: &Address,
    avatar: &String,
) -> Result<bool, Box<dyn Error>> {
    check_conn(conn);
    let (address, trunc_address) = get_address_strings(addr);
    let res: Option<String> = conn.exec_first(
        "SELECT address FROM whitelisted_addresses WHERE address != ? AND address != ? AND avatar = ?",
        (address, trunc_address, avatar),
    )?;
    if let Some(_) = res {
        return Ok(false);
    }
    Ok(true)
}

pub async fn get_time_keepers_count(conn: &mut Conn) -> Result<u64, Box<dyn Error>> {
    check_conn(conn);
    let res: Option<u64> =
        conn.exec_first("SELECT count(address) FROM whitelisted_addresses", ())?;
    if let Some(tk_count) = res {
        return Ok(tk_count);
    }
    Ok(0)
}

fn check_conn(conn: &mut Conn) {
    if let Err(_) = conn.ping() {
        let _ = conn.reset();
    }
}

pub fn read_referral(conn: &mut Conn, ref_key: &String) -> Result<String, Box<dyn Error>> {
    check_conn(conn);
    let res: Option<String> = conn.exec_first(
        "SELECT refvalue FROM referrals WHERE refkey = ?",
        (ref_key,),
    )?;
    if let Some(ref_val) = res {
        return Ok(ref_val);
    }
    Ok(String::new())
}

pub fn write_referral(conn: &mut Conn, ref_data: &ReferralData) -> Result<(), Box<dyn Error>> {
    check_conn(conn);
    if read_referral(conn, &ref_data.refkey)?.is_empty() {
        conn.exec_drop(
            "INSERT INTO referrals (refkey, refvalue) VALUES (?, ?)",
            (&ref_data.refkey, &ref_data.refvalue),
        )?;
    }
    Ok(())
}

pub async fn read_referrers_list(
    conn: &mut Conn,
    total_accounts: &mut BTreeMap<String, f64>,
) -> Result<(), Box<dyn Error>> {
    check_conn(conn);

    // Detect cyclic references, put referral codes of source accounts into visited;
    let mut visited_refs = HashSet::new();
    let first_result = conn.exec_iter(
        format!(
            "SELECT referral_code
                FROM whitelisted_addresses
                WHERE address IN({:?})",
            vec!["?"; total_accounts.len()]
        ),
        total_accounts.keys().collect::<Vec<_>>(),
    )?;
    for row_res in first_result {
        let row_res = row_res?;
        let referral_res: Option<String> = row_res.get(0);
        if let Some(referral_code) = referral_res {
            visited_refs.insert(referral_code);
        }
    }
    let mut level = 1;
    loop {
        let mut ref_accounts: BTreeMap<String, f64> = BTreeMap::new();
        let result = conn.exec_iter(
            format!(
                "SELECT a1.address, a2.address, a2.referral_code
                    FROM whitelisted_addresses AS a1
                    JOIN whitelisted_addresses AS a2
                    ON a2.referral_code = a1.referred_from
                    WHERE a1.address IN({:?})
                    AND NULLIF(a1.referred_from, '') IS NOT NULL",
                vec!["?"; total_accounts.len()]
            ),
            total_accounts.keys().collect::<Vec<_>>(),
        )?;
        for row_res in result {
            let row_res = row_res?;
            let src_account_res: Option<String> = row_res.get(0);
            let ref_account_res: Option<String> = row_res.get(1);
            // Finding visited referral codes.
            let referral_code_res: Option<String> = row_res.get(2);
            if let Some(referral_code) = referral_code_res {
                if visited_refs.insert(referral_code) {
                    //Inserting the found account if no cyclic referral detected.
                    if let Some(src_account) = src_account_res {
                        if let Some(ref_account) = ref_account_res {
                            // Compute rewards amount from the source one.
                            if let Some(src_amount) = total_accounts.get(&src_account) {
                                ref_accounts.insert(
                                    ref_account,
                                    src_amount * get_ref_coeff_for_level(level),
                                );
                            }
                        }
                    }
                }
            }
        }
        if ref_accounts.is_empty() {
            break;
        }
        level += 1;
    }
    Ok(())
}

pub fn get_ref_coeff_for_level(level: u32) -> f64 {
    0.1 / level as f64
}

#[cfg(test)]
mod tests {
    use crate::db::get_ref_coeff_for_level;

    #[tokio::test]
    async fn test_ref_coeff() {
        assert_eq!(get_ref_coeff_for_level(1), 0.1);
        assert_eq!(get_ref_coeff_for_level(2), 0.05);
        assert_eq!(get_ref_coeff_for_level(4), 0.025);
    }
}
