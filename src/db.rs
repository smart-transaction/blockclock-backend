use std::error::Error;

use ethers::types::Address;
use mysql::{prelude::Queryable, Conn};

pub async fn store_whitelisted_address(
    conn: &mut Conn,
    addr: Address,
) -> Result<(), Box<dyn Error>> {
    let res: Option<String> = conn.exec_first(
        "SELECT address FROM whitelisted_addresses WHERE address = ?",
        (addr.to_string(),),
    )?;
    if res == None {
        conn.exec_drop(
            "INSERT INTO whitelisted_addresses (address) VALUES (?)",
            (addr.to_string(),),
        )?;
    }
    Ok(())
}

pub async fn is_address_whitelisted(
    conn: &mut Conn,
    addr: Address,
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
