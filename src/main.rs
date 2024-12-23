use std::{error::Error, sync::Arc};

use axum::{
    routing::{get, post},
    serve, Json, Router,
};
use claim_avatar::handle_claim_avatar;
use clap::Parser;
use ethers::{
    middleware::MiddlewareBuilder,
    providers::{Provider, Ws},
    signers::{LocalWallet, Signer},
    types::Address,
};
use meantime::MeanTime;
use mysql::Pool;
use onboarding::handle_onboard;
use serde_json::json;
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
    pub mysql_user: String,

    #[arg(long)]
    pub mysql_password: String,

    #[arg(long)]
    pub mysql_host: String,

    #[arg(long, default_value_t = 3306)]
    pub mysql_port: u16,

    #[arg(long)]
    pub mysql_database: String,

    #[arg(long)]
    pub time_window: String,

    #[arg(long)]
    pub solver_private_key: LocalWallet,

    #[arg(long)]
    pub chain_id: u64,

    #[arg(long)]
    pub ws_chain_url: String,

    #[arg(long)]
    pub block_time_address: Address,

    #[arg(long)]
    pub tick_period: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let time_sig_pool = Arc::new(Mutex::new(TimeSigPool::new()));
    let time_window = parse_duration::parse(&args.time_window)?;
    let tick_period = parse_duration::parse(&args.tick_period)?;

    let mysql_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        args.mysql_user, args.mysql_password, args.mysql_host, args.mysql_port, args.mysql_database
    );
    let mysql_display_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        args.mysql_user, "********", args.mysql_host, args.mysql_port, args.mysql_database
    );
    println!(
        "Connecting to the database with URL {} ...",
        mysql_display_url
    );
    let db_conn = Pool::new(mysql_url.as_str())?.get_conn()?;
    println!("Successfully created DB connection.");
    let db_conn: Arc<Mutex<mysql::PooledConn>> = Arc::new(Mutex::new(db_conn));

    let mut exec_set: JoinSet<()> = JoinSet::new();

    let wallet = args.solver_private_key.with_chain_id(args.chain_id);

    println!(
        "Connecting to the chain with URL {} ...",
        args.ws_chain_url.as_str()
    );
    let provider = Provider::<Ws>::connect(args.ws_chain_url.as_str()).await?;
    println!("Successfully connected to the chain.");
    let middleware = Arc::new(provider.with_signer(wallet));

    let meantime_comp = Arc::new(Mutex::new(MeanTime::new(
        time_sig_pool.clone(),
        args.block_time_address,
        middleware,
        time_window,
    )));

    let mut time_tick = TimeTick::new(tick_period, meantime_comp);
    exec_set.spawn(async move {
        time_tick.ticker().await;
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
            "/get_time_margin",
            get({
                let output = Json(json!({
                    "time_margin": time_window.as_nanos().to_string(),
                }));
                move || async { output }
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
