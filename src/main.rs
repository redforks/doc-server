use axum::Router;
use tokio::process::Command;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

async fn update_doc() {
    match Command::new("cargo")
        .args(["doc", "--workspace", "--keep-going"])
        .spawn()
    {
        Err(e) => {
            error!("Error start cargo: {}", e);
        }
        Ok(mut child) => {
            match child.wait().await {
                Ok(exit_status) => info!("cargo doc exit with status: {exit_status}"),
                Err(e) => {
                    error!("Error execute cargo: {}", e)
                }
            };
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // tracing_subscriber by default captures log crate emitted messages
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    update_doc().await;

    let app = Router::new()
        .nest_service("/", ServeDir::new("target/doc"))
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
