use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
};

use mysql::{prelude::Queryable, Conn};

use crate::db::check_conn;

pub async fn read_referrers_list(
    conn: &mut Conn,
    total_accounts: &mut BTreeMap<String, f64>,
) -> Result<(), Box<dyn Error>> {
    check_conn(conn);

    // Detect cyclic references, put referral codes of source accounts into visited;
    let mut visited_refs = HashSet::new();

    let stmt = format!(
        "SELECT referral_code
            FROM whitelisted_addresses
            WHERE address IN({})
            AND NULLIF(referral_code, '') IS NOT NULL",
        vec!["?"; total_accounts.len()].join(",")
    );
    let first_result = conn.exec_iter(stmt, total_accounts.keys().collect::<Vec<_>>())?;
    for row_res in first_result {
        let row_res = row_res?;
        let referral_res: Option<String> = row_res.get(0);
        if let Some(referral_code) = referral_res {
            visited_refs.insert(referral_code);
        }
    }
    let mut level = 1;
    let mut ref_accounts: BTreeMap<String, f64> = total_accounts.clone();
    loop {
        let mut next_ref_accounts: BTreeMap<String, f64> = BTreeMap::new();
        println!("{:?}", ref_accounts);
        let stmt = format!(
            "SELECT a1.address, a2.address, a2.referral_code
                FROM whitelisted_addresses AS a1
                JOIN whitelisted_addresses AS a2
                ON a2.referral_code = a1.referred_from
                WHERE a1.address IN({})
                AND NULLIF(a1.referred_from, '') IS NOT NULL",
            vec!["?"; ref_accounts.len()].join(",")
        );
        println!("{}", stmt);
        let result = conn.exec_iter(stmt, ref_accounts.keys().collect::<Vec<_>>())?;
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
                                let new_amount = get_referral_amount(src_amount, &level);
                                // Insert referral rewards or append it for case of multiple referrals
                                match next_ref_accounts.get(&ref_account) {
                                    Some(amount) => {
                                        next_ref_accounts.insert(ref_account, amount + new_amount);
                                    }
                                    None => {
                                        next_ref_accounts.insert(ref_account, new_amount);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if next_ref_accounts.is_empty() {
            break;
        }
        ref_accounts = next_ref_accounts.clone();
        total_accounts.append(&mut next_ref_accounts);
        level += 1;
    }
    println!("{:#?}", total_accounts);
    Ok(())
}

pub fn get_referral_amount(src_amount: &f64, level: &u32) -> f64 {
    match level {
        0 => 1.0,
        1 => 0.1 * src_amount,
        _ => 0.5 * src_amount,
    }
}

#[cfg(test)]
mod tests {
    use crate::referrers_fetch::get_referral_amount;

    #[tokio::test]
    async fn test_ref_amount() {
        assert_eq!(get_referral_amount(&1.0, &1), 0.1);
        assert_eq!(get_referral_amount(&0.1, &2), 0.05);
        assert_eq!(get_referral_amount(&0.025, &4), 0.0125);
    }
}
