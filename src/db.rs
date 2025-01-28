use std::error::Error;

use ethers::types::Address;
use mysql::{prelude::Queryable, Conn};

use crate::{address_str::get_address_strings, referral::ReferralData};

// Fixes an invalid address in the addresses db. Replaces a short display address with a full one.
pub async fn fix_address(conn: &mut Conn, addr: &Address) -> Result<(), Box<dyn Error>> {
    check_conn(conn);
    let (full_addr, short_addr) = get_address_strings(&addr);
    conn.exec_drop(
        "UPDATE whitelisted_addresses SET address = ? WHERE address = ?",
        (full_addr, short_addr),
    )?;
    Ok(())
}

pub async fn store_user_data(
    conn: &mut Conn,
    addr: &Address,
    avatar: &String,
) -> Result<(), Box<dyn Error>> {
    check_conn(conn);
    let (address, trunc_address) = get_address_strings(addr);
    let res: Option<String> = conn.exec_first(
        "SELECT address FROM whitelisted_addresses WHERE address = ? OR address = ?",
        (&address, trunc_address),
    )?;
    if res == None {
        conn.exec_drop(
            "INSERT INTO whitelisted_addresses (address, avatar) VALUES (?, ?)",
            (address, avatar),
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
        "UPDATE whitelisted_addresses SET address = ?, avatar = ? WHERE address = ? OR address = ?",
        (&address, avatar, &address, trunc_address),
    )?;
    Ok(())
}

pub async fn update_referral_code(
    conn: &mut Conn,
    addr: &Address,
    referral_code: &String,
) -> Result<(), Box<dyn Error>> {
    check_conn(conn);
    let (address, trunc_address) = get_address_strings(addr);
    conn.exec_drop(
        "UPDATE whitelisted_addresses SET address = ?, referral_code = ? WHERE address = ? OR address = ?",
        (&address, referral_code, &address, trunc_address),
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
        "UPDATE whitelisted_addresses SET address = ?, referred_from = ? WHERE address = ? OR address = ?",
        (&address, referred_from, &address, trunc_address),
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
        "SELECT address FROM whitelisted_addresses WHERE (address != ? AND address != ?) AND avatar = ?",
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

pub fn check_conn(conn: &mut Conn) {
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
