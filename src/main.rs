use axum::Router;
use std::{
    io::{self, Write},
    process::Command,
};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::error;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn update_doc() {
    match Command::new("cargo")
        .args(["doc", "--workspace", "--keep-going"])
        .output()
    {
        Err(e) => {
            error!("Error execute cargo: {}", e);
        }
        Ok(output) => {
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
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

    update_doc();

    let app = Router::new()
        .nest_service("/", ServeDir::new("target/doc"))
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
