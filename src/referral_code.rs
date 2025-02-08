use std::sync::Arc;

use axum::{http::StatusCode, Json};
use log::{error, warn};
use mysql::PooledConn;
use tokio::sync::Mutex;

use crate::{
    db::{is_referral_code_available, update_referral_code, update_referred_from},
    user_data::{ReferralCodeData, ReferredFromData},
};

pub async fn handle_update_referral_code(
    input: Json<ReferralCodeData>,
    db_conn: Arc<Mutex<PooledConn>>,
) -> Result<(), StatusCode> {
    let mut conn = db_conn.lock().await;
    match is_referral_code_available(conn.as_mut(), &input.time_keeper, &input.referral_code).await {
        Ok(is_avail) => {
            if !is_avail {
                warn!("The referral code {} is already in use", input.referral_code);
                return Err(StatusCode::CONFLICT);
            }
        }
        Err(err) => {
            error!("Error checking the referral code: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
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
