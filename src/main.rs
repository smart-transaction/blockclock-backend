use axum::{routing::get, serve, Router};
use clap::Parser;
use tokio::net::TcpListener;

mod time_signature;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long, default_value_t = 8000)]
    pub port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let app = Router::new().route("/", get(|| async { "Blockclock Backend" }));

    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", args.port))
        .await
        .unwrap();

    println!("Starting server at port {}", args.port);
    serve(tcp_listener, app).await.unwrap();
}
