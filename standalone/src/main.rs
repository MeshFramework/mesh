#[cfg(feature = "neo4j")]
mod handlers;
mod types;

use ast::repo::StatusUpdate;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::{routing::get, routing::post, Router};
use broadcast::error::RecvError;
use futures::stream;
use std::convert::Infallible;
use std::time::Duration;

use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeFile;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};
use types::Result;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<StatusUpdate>,
}

#[cfg(feature = "neo4j")]
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(filter)
        .init();

    let (tx, _rx) = broadcast::channel(10000);

    let mut dummy_rx = tx.subscribe();
    tokio::spawn(async move {
        while let Ok(_) = dummy_rx.recv().await {
            // Just consume messages, don't do anything
            // this is required to keep the msgs fast. weird.
        }
    });

    let app_state = Arc::new(AppState { tx: tx });

    let cors_layer = CorsLayer::permissive();
    let app = Router::new()
        .route("/process", post(handlers::process))
        .route("/clear", post(handlers::clear_graph))
        .route("/ingest", post(handlers::ingest))
        .route("/fetch-repo", post(handlers::fetch_repo))
        .route("/events", get(sse_handler))
        .route_service("/", static_file("index.html"))
        .route_service("/styles.css", static_file("styles.css"))
        .route_service("/app.js", static_file("app.js"))
        .route_service("/utils.js", static_file("utils.js"))
        .with_state(app_state)
        .layer(cors_layer);

    let port = std::env::var("PORT").unwrap_or_else(|_| "7777".to_string());
    let bind = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    println!("=> listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

fn static_file(path: &str) -> ServeFile {
    ServeFile::new(format!("standalone/static/{}", path))
}

async fn sse_handler(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let rx = app_state.tx.subscribe();

    let stream = stream::unfold(rx, move |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    let data = msg.as_json_str();
                    let millis = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                    let event = Event::default().data(data).id(format!("{}", millis));
                    return Some((Ok::<Event, Infallible>(event), rx));
                }
                Err(RecvError::Lagged(skipped)) => {
                    println!("SSE receiver lagged, skipped {} messages", skipped);
                    continue;
                }
                Err(RecvError::Closed) => {
                    return None;
                }
            }
        }
    });

    let headers = [
        ("Cache-Control", "no-cache, no-store, must-revalidate"),
        ("Connection", "keep-alive"),
        ("Content-Type", "text/event-stream"),
        ("X-Accel-Buffering", "no"), // nginx
        ("X-Proxy-Buffering", "no"), // other proxies
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Headers", "Cache-Control"),
    ];
    (
        headers,
        Sse::new(stream).keep_alive(
            KeepAlive::new()
                .interval(Duration::from_millis(500))
                .text("ping"),
        ),
    )
}

#[cfg(not(feature = "neo4j"))]
fn main() -> Result<()> {
    println!(
        "The 'neo4j' feature must be enabled to build this binary. Use: cargo run --features neo4j"
    );
    Ok(())
}
