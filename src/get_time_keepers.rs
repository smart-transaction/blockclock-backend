use std::sync::Arc;

use axum::{http::StatusCode, Json};
use log::error;
use mysql::PooledConn;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::db::get_time_keepers_count;

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeKeepersStats {
    count: u64,
}

pub async fn handle_get_time_keepers(
    db_conn: Arc<Mutex<PooledConn>>,
) -> Result<Json<TimeKeepersStats>, StatusCode> {
    let mut conn = db_conn.lock().await;
    match get_time_keepers_count(conn.as_mut()).await {
        Ok(tk_count) => Ok(Json(TimeKeepersStats { count: tk_count })),
        Err(err) => {
            error!("Error getting time keepers: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
