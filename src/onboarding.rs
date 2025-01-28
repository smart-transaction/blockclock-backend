use std::sync::Arc;

use axum::{http::StatusCode, Json};
use mysql::PooledConn;
use tokio::sync::Mutex;

use crate::{db::store_user_data, user_data::UserData};

pub async fn handle_onboard(
    input_json: Json<UserData>,
    db_conn: Arc<Mutex<PooledConn>>,
) -> Result<(), StatusCode> {
    let mut db_conn = db_conn.lock().await;
    match store_user_data(
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
            println!("Error storing whilelisted address: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}
