use std::{collections::HashMap, sync::Arc};

use axum::{extract::Query, http::StatusCode, Json};
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::db::{read_referral, write_referral};

#[derive(Debug, Deserialize, Serialize)]
pub struct ReferralData {
    pub refkey: String,   // Format: <IPAddress>:<screenwidth>:<screenheight>
    pub refvalue: String, // Referral code
}

pub async fn handle_read_referral(
    params: Query<HashMap<String, String>>,
    db_conn: Arc<Mutex<mysql::PooledConn>>,
) -> Result<Json<ReferralData>, StatusCode> {
    if let Some(ref_key) = params.get("ref_key") {
        let mut db_conn = db_conn.lock().await;
        match read_referral(db_conn.as_mut(), ref_key) {
            Ok(referral_code) => {
                return Ok(Json(ReferralData {
                    refkey: ref_key.to_string(),
                    refvalue: referral_code,
                }));
            }
            Err(err) => {
                error!("Error reading the referral: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
    Err(StatusCode::BAD_REQUEST)
}

pub async fn handle_write_referral(
    input_json: Json<ReferralData>,
    db_conn: Arc<Mutex<mysql::PooledConn>>,
) -> Result<(), StatusCode> {
    let mut db_conn = db_conn.lock().await;
    match write_referral(db_conn.as_mut(), &input_json.0) {
        Ok(_) => {
            return Ok(());
        }
        Err(err) => {
            error!("Error storing the referral: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}
