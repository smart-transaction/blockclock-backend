use std::{str::FromStr, sync::Arc};

use axum::{http::StatusCode, Json};
use ethers::types::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::time_signature::TimeSignature;

pub type TimeSigPool = Vec<TimeSignature>;

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeSigInput {
    epoch: String,
    time_keeper: String,
    signature: String,
}

pub async fn handle_add_time_sig(
    Json(input): Json<TimeSigInput>,
    pool: Arc<Mutex<TimeSigPool>>,
) -> Result<(), StatusCode> {
    let epoch = U256::from_str_radix(&input.epoch, 10);
    let time_keeper = Address::from_str(&input.time_keeper);
    let signature = Bytes::from_str(&input.signature);
    if let Err(err) = epoch {
        println!("Error extracting epoch: {}", err);
        return Err(StatusCode::BAD_REQUEST);
    }
    if let Err(err) = time_keeper {
        println!("Error extracting time keeper: {}", err);
        return Err(StatusCode::BAD_REQUEST);
    }
    if let Err(err) = signature {
        println!("Error extracting signature: {}", err);
        return Err(StatusCode::BAD_REQUEST);
    }
    let time_signature = TimeSignature::new(
        epoch.unwrap(),
        time_keeper.unwrap(),
        signature.unwrap(),
    );
    if time_signature.verify() {
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
