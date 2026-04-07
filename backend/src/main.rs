use std::marker::PhantomData;

use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
    routing::{any, get},
};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};

use tokio::net::TcpListener;
use tokio::select;

use serde::{Deserialize, Serialize};
use serde_json::Result;

const SERVER_ADDRESS: &str = "127.0.0.1:3000";

#[derive(Clone)]
struct AppState {
    nodes: u32,
}

#[derive(Serialize, Deserialize)]
struct AddNodeEvent {
    eventType: String,
    nodeId: String,
}

impl AddNodeEvent {
    fn new(id: String) -> Self {
        AddNodeEvent {
            eventType: "ADD_NODE".into(),
            nodeId: id,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AddEdgeEvent {
    eventType: String,
    startNodeId: String,
    endNodeId: String,
}

impl AddEdgeEvent {
    fn new(start: String, end: String) -> Self {
        AddEdgeEvent {
            eventType: "ADD_EDGE".into(),
            startNodeId: start,
            endNodeId: end,
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, mut state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    socket_loop(sender, receiver, state).await;
}

async fn socket_loop(
    mut sender: SplitSink<WebSocket, Message>,
    mut receiver: SplitStream<WebSocket>,
    mut state: AppState,
) {
    loop {
        select! {
            Some(Ok(message)) = receiver.next() => {
            match message {
                axum::extract::ws::Message::Text(utf8_bytes) => {
                    // TODO: Here should be some reasonable logic, lol
                    state.nodes += 1;
                    let event = AddNodeEvent::new(state.nodes.to_string());
                    let s = serde_json::to_string(&event).unwrap();
                    sender.send(Message::Text(s.into())).await;

                    if state.nodes > 2 {
                        let event = AddEdgeEvent::new((state.nodes - 1).to_string(), (state.nodes - 2).to_string());
                        let s = serde_json::to_string(&event).unwrap();
                        sender.send(Message::Text(s.into())).await;
                    }
                }
                axum::extract::ws::Message::Binary(bytes) => todo!(),
                axum::extract::ws::Message::Ping(bytes) => todo!(),
                axum::extract::ws::Message::Pong(bytes) => todo!(),
                axum::extract::ws::Message::Close(_) => break
            }
            }
            else => {
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/ws", any(ws_handler))
        .with_state(AppState { nodes: 0 });

    let listener = TcpListener::bind(SERVER_ADDRESS).await.unwrap();
    println!("Server running on {}", SERVER_ADDRESS);

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
