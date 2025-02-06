use axum::{http::StatusCode, Json};
use log::*;
use mysql::PooledConn;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    db::{is_avatar_available, update_avatar},
    user_data::AvatarData,
};

pub async fn handle_claim_avatar(
    input_json: Json<AvatarData>,
    db_conn: Arc<Mutex<PooledConn>>,
) -> Result<(), StatusCode> {
    let avatar = input_json.avatar.clone();
    let mut db_conn = db_conn.lock().await;
    match is_avatar_available(
        db_conn.as_mut(),
        &input_json.time_keeper,
        &input_json.avatar,
    )
    .await
    {
        Ok(is_avail) => {
            if !is_avail {
                error!("The avatar {} is already in use", input_json.avatar);
                return Err(StatusCode::CONFLICT);
            }
        }
        Err(err) => {
            error!("Error checking the avatar: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    match update_avatar(
        db_conn.as_mut(),
        &input_json.time_keeper,
        &input_json.avatar,
    )
    .await
    {
        Ok(_) => {
            return Ok(());
        }
        Err(err) => {
            error!("Error updating the avatar: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}
