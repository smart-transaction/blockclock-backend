use std::sync::Arc;

use axum::{http::StatusCode, Json};
use log::error;
use mysql::PooledConn;
use tokio::sync::Mutex;

use crate::{
    db::{update_referral_code, update_referred_from},
    user_data::{ReferralCodeData, ReferredFromData},
};

pub async fn handle_update_referral_code(
    input: Json<ReferralCodeData>,
    db_conn: Arc<Mutex<PooledConn>>,
) -> Result<(), StatusCode> {
    let mut conn = db_conn.lock().await;
    match update_referral_code(conn.as_mut(), &input.time_keeper, &input.referral_code).await {
        Ok(_) => {
            return Ok(());
        }
        Err(err) => {
            error!("Error updating referral code: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

pub async fn handle_update_referred_from(
    input: Json<ReferredFromData>,
    db_conn: Arc<Mutex<PooledConn>>,
) -> Result<(), StatusCode> {
    let mut conn = db_conn.lock().await;
    match update_referred_from(conn.as_mut(), &input.time_keeper, &input.referred_from).await {
        Ok(_) => {
            return Ok(());
        }
        Err(err) => {
            error!("Error updating referred from: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}
