use std::{
    error::Error,
    sync::Arc,
    time::{Duration, SystemTime},
};

use axum::{
    routing::{get, post},
    serve, Router,
};
use claim_avatar::handle_claim_avatar;
use clap::Parser;
use mysql::Pool;
use onboarding::handle_onboard;
use time_pool::{handle_add_time_sig, handle_list_time_sigs, TimeSigPool};
use timer::TimeTick;
use tokio::{net::TcpListener, sync::Mutex, task::JoinSet};

mod claim_avatar;
mod db;
mod meantime;
mod onboarding;
mod time_pool;
mod time_signature;
mod timer;
mod user_data;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long, default_value_t = 8000)]
    pub port: u16,

    #[arg(long)]
    pub mysql_url: String,

    #[arg(long)]
    pub time_window: String,
}

fn time_handler(timestamp: SystemTime) {
    // TODO: Add time handling with checking signatures and sending to the contract
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let time_sig_pool = Arc::new(Mutex::new(TimeSigPool::new()));
    let time_window = parse_duration::parse(args.time_window.as_str());

    let db_conn = Pool::new(args.mysql_url.as_str())?.get_conn()?;
    let db_conn: Arc<Mutex<mysql::PooledConn>> = Arc::new(Mutex::new(db_conn));

    let mut exec_set: JoinSet<()> = JoinSet::new();

    let mut time_tick = TimeTick::new(Duration::new(0, 100000000));
    exec_set.spawn(async move {
        time_tick.ticker(time_handler).await;
    });

    let app = Router::new()
        .route("/", get(|| async { "Blockclock Backend" }))
        .route(
            "/list_time_sigs",
            get({
                let time_sig_pool = Arc::clone(&time_sig_pool);
                move || handle_list_time_sigs(time_sig_pool)
            }),
        )
        .route(
            "/add_time_sig",
            post({
                let time_sig_pool = Arc::clone(&time_sig_pool);
                let db_conn = Arc::clone(&db_conn);
                move |input| handle_add_time_sig(input, time_sig_pool, db_conn)
            }),
        )
        .route(
            "/claim_avatar",
            post({
                let db_conn = Arc::clone(&db_conn);
                move |input| handle_claim_avatar(input, db_conn)
            }),
        )
        .route(
            "/onboard",
            post({
                let db_conn = Arc::clone(&db_conn);
                move |input| handle_onboard(input, db_conn)
            }),
        );

    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", args.port))
        .await
        .unwrap();

    println!("Starting server at port {}", args.port);
    serve(tcp_listener, app).await.unwrap();
    Ok(())
}
