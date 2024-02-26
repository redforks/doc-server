use axum::Router;
use futures_util::stream::StreamExt;
use inotify::{Inotify, WatchMask};
use std::{future::Future, path::Path, time::Duration};
use tokio::process::Command;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{debug, error, info};
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

    // check Cargo.lock exist in current directory
    if !Path::new("./Cargo.lock").is_file() {
        eprintln!("No Cargo.lock file in current directory.");
        return;
    }

    update_doc().await;
    tokio::spawn(update_doc_on_cargo_chanes());

    let app = Router::new()
        .nest_service("/", ServeDir::new("target/doc"))
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn update_doc_on_cargo_chanes() -> impl Future<Output = ()> {
    let inotify = Inotify::init().expect("Failed init file watch");
    let mut watch = inotify.watches();
    watch
        .add(
            "Cargo.lock",
            WatchMask::CREATE | WatchMask::DELETE | WatchMask::MODIFY,
        )
        .expect("Add watch directory failed");
    let buffer = [0; 1024];
    let stream = inotify
        .into_event_stream(buffer)
        .expect("Failed to open change stream");
    let mut stream = debounced::debounced(stream, Duration::from_secs(1));

    return async move {
        loop {
            let event = stream.next().await.unwrap().expect("get event failed");
            debug!("get file changed event: {:?}", event);
            update_doc().await;
        }
    };
}
