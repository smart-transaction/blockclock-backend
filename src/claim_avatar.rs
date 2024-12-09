use axum::{http::StatusCode, Json};
use mysql::PooledConn;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    db::{is_avatar_available, update_user_data},
    user_data::UserData,
};

pub async fn handle_claim_avatar(
    input_json: Json<UserData>,
    db_conn: Arc<Mutex<PooledConn>>,
) -> Result<(), StatusCode> {
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
                println!("The avatar {} is already in use", input_json.avatar);
                return Err(StatusCode::CONFLICT);
            }
        }
        Err(err) => {
            println!("Error checking the avatar: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    match update_user_data(
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
            println!("Error updating the avatar: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}
