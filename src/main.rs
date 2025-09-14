use std::{error::Error, sync::Arc};

use axum::{
    http::{
        header::{ACCEPT, ACCEPT_LANGUAGE, CONTENT_LANGUAGE, CONTENT_TYPE},
        Method,
    },
    routing::{get, post},
    serve, Json, Router,
};
use call_breaker::CallBreakerData;
use claim_avatar::handle_claim_avatar;
use clap::{ArgAction, Parser};
use ethers::{
    middleware::MiddlewareBuilder,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, Bytes},
};
use get_time_keepers::handle_get_time_keepers;
use log::{info, Level};
use meantime::MeanTime;
use mysql::Pool;
use onboarding::handle_onboard;
use referral::{handle_read_referral, handle_write_referral};
use referral_code::{handle_update_referral_code, handle_update_referred_from};
use serde_json::json;
use stderrlog::Timestamp;
use time_pool::{handle_add_time_sig, handle_list_time_sigs, TimeSigPool};
use timer::TimeTick;
use tokio::{net::TcpListener, sync::Mutex, task::JoinSet};
use tower_http::cors::{Any, CorsLayer};

mod address_str;
mod call_breaker;
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
    pub validator_private_key: LocalWallet,

    #[arg(long)]
    pub primary_chain_id: u64,

    #[arg(long)]
    pub primary_http_chain_url: String,

    #[arg(long)]
    pub secondary_chain_id: u64,

    #[arg(long)]
    pub secondary_http_chain_url: String,

    #[arg(long)]
    pub primary_block_time_address: Address,

    #[arg(long)]
    pub primary_call_breaker_address: Address,

    #[arg(long)]
    pub secondary_block_time_address: Address,

    #[arg(long)]
    pub secondary_call_breaker_address: Address,

    #[arg(long)]
    pub app_id: Bytes,

    #[arg(long)]
    pub tick_period: String,

    // Added for suspending rewards during airdrop.
    #[arg(long, default_value="false", default_missing_value="false", num_args(0..=1), action=ArgAction::Set)]
    pub dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let time_sig_pool = Arc::new(Mutex::new(TimeSigPool::new()));
    let time_window = parse_duration::parse(&args.time_window)?;
    let tick_period = parse_duration::parse(&args.tick_period)?;

    stderrlog::new()
        .verbosity(Level::Info)
        .timestamp(Timestamp::Millisecond)
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

    let primary_wallet = args
        .solver_private_key
        .clone()
        .with_chain_id(args.primary_chain_id);

    let secondary_wallet = args
        .solver_private_key
        .clone()
        .with_chain_id(args.secondary_chain_id);

    let validator_wallet = args.validator_private_key.clone();

    let app_id = args.app_id.clone();

    info!(
        "Using primary wallet {}",
        format!("{:#x}", primary_wallet.address())
    );

    info!(
        "Using secondary wallet {}",
        format!("{:#x}", secondary_wallet.address())
    );

    info!(
        "Connecting to the primary chain with URL {} ...",
        args.primary_http_chain_url.as_str()
    );
    let primary_provider = Provider::<Http>::try_from(args.primary_http_chain_url.as_str())?;
    info!(
        "Successfully connected to the primary chain {}.",
        args.primary_chain_id
    );
    info!(
        "Connecting to the secondary chain with URL {} ...",
        args.secondary_http_chain_url.as_str()
    );
    let secondary_provider = Provider::<Http>::try_from(args.secondary_http_chain_url.as_str())?;
    info!(
        "Successfully connected to the secondary chain {}.",
        args.secondary_chain_id
    );

    let primary_client = Arc::new(primary_provider.with_signer(primary_wallet.clone()));
    let primary_call_breaker_comp = Arc::new(CallBreakerData::new(
        args.primary_call_breaker_address,
        args.primary_block_time_address,
        primary_client,
        primary_wallet,
        validator_wallet.clone(),
        app_id.clone(),
    ));

    let secondary_client = Arc::new(secondary_provider.with_signer(secondary_wallet.clone()));
    let secondary_call_breaker_comp = Arc::new(CallBreakerData::new(
        args.secondary_call_breaker_address,
        args.secondary_block_time_address,
        secondary_client,
        secondary_wallet,
        validator_wallet.clone(),
        app_id.clone(),
    ));

    let meantime_comp = Arc::new(Mutex::new(MeanTime::new(
        time_sig_pool.clone(),
        primary_call_breaker_comp,
        secondary_call_breaker_comp,
        time_window,
        args.dry_run,
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
