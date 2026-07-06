use std::sync::Arc;

use axum::{Router, routing::get};

use tokio::net::TcpListener;

use tuptacz::app;
use tuptacz::loading;
use tuptacz::routing_ws::handle_socket;
use tuptacz::transit_router::transit_router;

const REST_ADDRESS: &str = "0.0.0.0:3000";
const WS_ADDRESS: &str = "0.0.0.0:3001";

async fn health_check_handler() -> &'static str {
    "Backend up."
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let routing_info = loading::load_routing_info().unwrap();
    let transit_info = loading::load_transit_info().unwrap();
    let state = Arc::new(app::State {
        routing_info,
        transit_info,
    });

    // REST server via axum
    let rest_app = Router::new()
        .route("/api/health-check", get(health_check_handler))
        .nest("/api/transit", transit_router())
        .with_state(state.clone());

    // WebSocket server via tungstenite
    let ws_listener = TcpListener::bind(WS_ADDRESS).await.unwrap();
    let rest_listener = TcpListener::bind(REST_ADDRESS).await.unwrap();

    eprintln!("REST running on {}", REST_ADDRESS);
    eprintln!("WS running on {}", WS_ADDRESS);

    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            tokio::task::spawn_local(async move {
                axum::serve(rest_listener, rest_app).await.unwrap();
            });

            loop {
                let (stream, _) = ws_listener.accept().await.unwrap();
                let state = state.clone();
                tokio::task::spawn_local(async move {
                    let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
                    handle_socket(ws, state).await;
                });
            }
        })
        .await;
}
