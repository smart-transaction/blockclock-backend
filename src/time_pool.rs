use std::{str::FromStr, sync::Arc};

use axum::{http::StatusCode, Json};
use ethers::types::{Address, Bytes, U256};
use log::error;
use mysql::PooledConn;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    db::{fix_address, is_address_whitelisted},
    time_signature::Chronicle,
};

pub type TimeSigPool = Vec<Chronicle>;

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeSigInput {
    epoch: String,
    time_keeper: String,
    signature: String,
}

pub async fn handle_add_time_sig(
    Json(input): Json<TimeSigInput>,
    pool: Arc<Mutex<TimeSigPool>>,
    db_conn: Arc<Mutex<PooledConn>>,
) -> Result<(), StatusCode> {
    let epoch = U256::from_str_radix(&input.epoch, 10);
    let time_keeper = Address::from_str(&input.time_keeper);
    let signature = Bytes::from_str(&input.signature);
    if let Err(err) = epoch {
        error!("Error extracting epoch: {}", err);
        return Err(StatusCode::BAD_REQUEST);
    }
    if let Err(err) = time_keeper {
        error!("Error extracting time keeper: {}", err);
        return Err(StatusCode::BAD_REQUEST);
    }
    if let Err(err) = signature {
        error!("Error extracting signature: {}", err);
        return Err(StatusCode::BAD_REQUEST);
    }
    {
        let mut db_conn = db_conn.lock().await;
        match is_address_whitelisted(db_conn.as_mut(), &time_keeper.unwrap()).await {
            Ok(res) => {
                if !res {
                    error!("The address {} isn't whitelisted", time_keeper.unwrap());
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
            Err(err) => {
                error!("Error checking time keepers whitelist: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
    let time_signature = Chronicle::new(epoch.unwrap(), time_keeper.unwrap(), signature.unwrap());
    if time_signature.verify() {
        // Update the address in the database, fix the display address error.
        let mut db_conn = db_conn.lock().await;
        if let Err(err) = fix_address(db_conn.as_mut(), &time_signature.time_keeper).await {
            error!("Error fixing address: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        let mut time_sig_pool = pool.lock().await;
        time_sig_pool.push(time_signature);
        return Ok(());
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }
}

pub async fn handle_list_time_sigs(pool: Arc<Mutex<TimeSigPool>>) -> Json<TimeSigPool> {
    let pool = pool.lock().await;
    Json(pool.to_vec())
}
