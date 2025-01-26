use std::sync::Arc;

use axum::{http::StatusCode, Json};
use mysql::PooledConn;
use tokio::sync::Mutex;

use crate::{db::update_referred_from, user_data::UserData};

pub async fn handle_update_referred_from(input: Json<UserData>, db_conn: Arc<Mutex<PooledConn>>) -> Result<(), StatusCode> {
  let mut conn = db_conn.lock().await;
  match update_referred_from(conn.as_mut(), &input.time_keeper, &input.referred_from).await {
    Ok(_) => {
      return Ok(());
    }
    Err(err) => {
      println!("Error updating referred from: {}", err);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  }
}