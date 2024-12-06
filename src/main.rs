use std::sync::Arc;

use axum::{
    routing::{get, post},
    serve, Router,
};
use clap::Parser;
use time_pool::{handle_add_time_sig, handle_list_time_sigs, TimeSigPool};
use tokio::{net::TcpListener, sync::Mutex};

mod time_pool;
mod time_signature;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long, default_value_t = 8000)]
    pub port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let time_sig_pool = Arc::new(Mutex::new(TimeSigPool::new()));

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
                move |input| handle_add_time_sig(input, time_sig_pool)
            }),
        )
        .with_state(time_sig_pool);

    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", args.port))
        .await
        .unwrap();

    println!("Starting server at port {}", args.port);
    serve(tcp_listener, app).await.unwrap();
}
