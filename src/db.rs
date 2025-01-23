use std::error::Error;

use ethers::types::Address;
use mysql::{prelude::Queryable, Conn};

use crate::referral::ReferralData;

pub async fn store_user_data(
    conn: &mut Conn,
    addr: &Address,
    avatar: &String,
    referral_code: &String,
) -> Result<(), Box<dyn Error>> {
    check_conn(conn);
    let res: Option<String> = conn.exec_first(
        "SELECT address FROM whitelisted_addresses WHERE address = ?",
        (addr.to_string(),),
    )?;
    if res == None {
        conn.exec_drop(
            "INSERT INTO whitelisted_addresses (address, avatar, referral_code) VALUES (?, ?, ?)",
            (addr.to_string(), avatar, referral_code),
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
    conn.exec_drop(
        "UPDATE whitelisted_addresses SET avatar = ? WHERE address = ?",
        (avatar, addr.to_string()),
    )?;
    Ok(())
}

pub async fn is_address_whitelisted(
    conn: &mut Conn,
    addr: &Address,
) -> Result<bool, Box<dyn Error>> {
    check_conn(conn);
    let res: Option<String> = conn.exec_first(
        "SELECT address FROM whitelisted_addresses WHERE address = ?",
        (addr.to_string(),),
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
    let res: Option<String> = conn.exec_first(
        "SELECT address FROM whitelisted_addresses WHERE address != ? AND avatar = ?",
        (addr.to_string(), avatar),
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

pub fn write_referral(
    conn: &mut Conn,
    ref_data: &ReferralData,
) -> Result<(), Box<dyn Error>> {
    check_conn(conn);
    if read_referral(conn, &ref_data.refkey)?.is_empty() {
        conn.exec_drop(
            "INSERT INTO referrals (refkey, refvalue) VALUES (?, ?)",
            (&ref_data.refkey, &ref_data.refvalue),
        )?;
    }
    Ok(())
}
