use std::{error::Error, sync::Arc};

use axum::{
    http::{
        header::{ACCEPT, ACCEPT_LANGUAGE, CONTENT_LANGUAGE, CONTENT_TYPE},
        Method,
    },
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
use get_time_keepers::handle_get_time_keepers;
use log::{info, Level};
use meantime::MeanTime;
use mysql::Pool;
use onboarding::handle_onboard;
use referral::{handle_read_referral, handle_write_referral};
use referral_code::{handle_update_referral_code, handle_update_referred_from};
use serde_json::json;
use time_pool::{handle_add_time_sig, handle_list_time_sigs, TimeSigPool};
use timer::TimeTick;
use tokio::{net::TcpListener, sync::Mutex, task::JoinSet};
use tower_http::cors::{Any, CorsLayer};

mod address_str;
mod claim_avatar;
mod db;
mod get_time_keepers;
mod meantime;
mod onboarding;
mod referral;
mod referral_code;
mod referrers_fetch;
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

    stderrlog::new()
        .verbosity(Level::Info)
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
        .unwrap();

    let mysql_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        args.mysql_user, args.mysql_password, args.mysql_host, args.mysql_port, args.mysql_database
    );
    let mysql_display_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        args.mysql_user, "********", args.mysql_host, args.mysql_port, args.mysql_database
    );
    info!(
        "Connecting to the database with URL {} ...",
        mysql_display_url
    );
    let db_conn = Pool::new(mysql_url.as_str())?.get_conn()?;
    info!("Successfully created DB connection.");
    let db_conn: Arc<Mutex<mysql::PooledConn>> = Arc::new(Mutex::new(db_conn));

    let mut exec_set: JoinSet<()> = JoinSet::new();

    let wallet = args.solver_private_key.with_chain_id(args.chain_id);

    info!(
        "Connecting to the chain with URL {} ...",
        args.ws_chain_url.as_str()
    );
    let provider = Provider::<Ws>::connect(args.ws_chain_url.as_str()).await?;
    info!("Successfully connected to the chain.");
    let middleware = Arc::new(provider.with_signer(wallet));

    let meantime_comp = Arc::new(Mutex::new(MeanTime::new(
        time_sig_pool.clone(),
        args.block_time_address,
        middleware,
        time_window,
    )));

    let mut time_tick = TimeTick::new(tick_period, meantime_comp, db_conn.clone());
    exec_set.spawn(async move {
        time_tick.ticker().await;
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers([ACCEPT, ACCEPT_LANGUAGE, CONTENT_LANGUAGE, CONTENT_TYPE]);

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
        )
        .route(
            "/update_referral_code",
            post({
                let db_conn = Arc::clone(&db_conn);
                move |input| handle_update_referral_code(input, db_conn)
            }),
        )
        .route(
            "/update_referred_from",
            post({
                let db_conn = Arc::clone(&db_conn);
                move |input| handle_update_referred_from(input, db_conn)
            }),
        )
        .route(
            "/get_time_keepers_count",
            get({
                let db_conn = Arc::clone(&db_conn);
                move || handle_get_time_keepers(db_conn)
            }),
        )
        .route(
            "/read_referral",
            get({
                let db_conn = Arc::clone(&db_conn);
                move |params| handle_read_referral(params, db_conn)
            }),
        )
        .route(
            "/write_referral",
            post({
                let db_conn = Arc::clone(&db_conn);
                move |input| handle_write_referral(input, db_conn)
            }),
        )
        .layer(cors);

    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", args.port))
        .await
        .unwrap();

    info!("Starting server at port {}", args.port);
    serve(tcp_listener, app).await.unwrap();
    Ok(())
}
