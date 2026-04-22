use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
    routing::any,
};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};

use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;
use tokio::select;

use serde::Serialize;
use tuptacz::{
    algo::InteractiveAlgo,
    pathfinding::{Dijkstra, Num},
    presentation::{EventClient, GraphEvent},
};

const SERVER_ADDRESS: &str = "127.0.0.1:3000";

struct WebSocketClient {
    sender: SplitSink<WebSocket, Message>,
}

impl<V, E> EventClient<V, E> for WebSocketClient
where
    E: Sync + Serialize,
    V: Sync + Serialize,
{
    async fn consume(&mut self, event: &GraphEvent<V, E>) {
        let serialized = serde_json::to_string(event).unwrap();
        self.sender.send(Message::Text(serialized.into())).await;
    }
}

struct InternalAppState {
    algo: Dijkstra<(), Num, WebSocketClient>,
}

type AppState = Arc<Mutex<InternalAppState>>;

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket<'a>(mut socket: WebSocket, mut state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    socket_loop(sender, receiver, state).await;
}

async fn socket_loop<'a>(
    mut sender: SplitSink<WebSocket, Message>,
    mut receiver: SplitStream<WebSocket>,
    mut state: AppState,
) {
    loop {
        select! {
            Some(Ok(message)) = receiver.next() => {
            match message {
                axum::extract::ws::Message::Text(utf8_bytes) => {
                    let mut data = state.lock().unwrap();
                    data.algo.step();
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
    let n = 42;
    let mut edge_id = 0;
    let vertices: Vec<_> = (0..n)
        .map(|i| Vertex {
            id: i,
            edges: vec![Edge {
                id: {
                    let id = edge_id;
                    edge_id += 1;
                    id
                },
                end_index: (i + 1) % n,
                properties: 1,
            }],
            properties: (),
        })
        .collect();

    let algo = Dijkstra::init(Arc::new(vertices), 0);

    // utwórz stan aplikacji
    let app_state = InternalAppState { algo };

    // zarejestruj router ze stanem
    let app = Router::new()
        .route("/ws", any(ws_handler))
        .with_state(Arc::new(Mutex::new(app_state)));

    let listener = TcpListener::bind(SERVER_ADDRESS).await.unwrap();
    println!("Server running on {}", SERVER_ADDRESS);

    axum::serve(listener, app).await.unwrap();
}
