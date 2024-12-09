use std::sync::Arc;

use axum::{http::StatusCode, Json};
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::db::store_whitelisted_address;

#[derive(Debug, Deserialize, Serialize)]
pub struct OnboardData {
    time_keeper: Address,
}

pub async fn handle_onboard(
    input_json: Json<OnboardData>,
    db_conn: Arc<Mutex<mysql::PooledConn>>,
) -> Result<(), StatusCode> {
    let mut db_conn = db_conn.lock().await;
    match store_whitelisted_address(db_conn.as_mut(), input_json.time_keeper).await {
        Ok(_) => {
            return Ok(());
        }
        Err(err) => {
            println!("Error storing whilelisted address: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}
