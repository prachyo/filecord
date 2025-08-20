use axum::{routing::get, Router};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let app = Router::new().route("/health", get(|| async { "ok" }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    tracing::info!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
