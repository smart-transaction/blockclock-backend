use std::error::Error;

use ethers::types::Address;
use mysql::{prelude::Queryable, Conn};

pub async fn store_user_data(
    conn: &mut Conn,
    addr: &Address,
    avatar: &String,
) -> Result<(), Box<dyn Error>> {
    let res: Option<String> = conn.exec_first(
        "SELECT address FROM whitelisted_addresses WHERE address = ?",
        (addr.to_string(),),
    )?;
    if res == None {
        conn.exec_drop(
            "INSERT INTO whitelisted_addresses (address, avatar) VALUES (?, ?)",
            (addr.to_string(), avatar),
        )?;
    }
    Ok(())
}

pub async fn update_user_data(
    conn: &mut Conn,
    addr: &Address,
    avatar: &String,
) -> Result<(), Box<dyn Error>> {
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
    let res: Option<String> = conn.exec_first(
        "SELECT address FROM whitelisted_addresses WHERE address != ? AND avatar = ?",
        (addr.to_string(), avatar),
    )?;
    if let Some(_) = res {
        return Ok(false);
    }
    Ok(true)
}
