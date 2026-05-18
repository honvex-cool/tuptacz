use std::sync::Arc;

use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
    routing,
};

use futures_util::{
    SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};

use rand::RngExt;

use serde::Serialize;

use tokio::net::TcpListener;
use tokio::select;

use tuptacz::{
    algo::{EventClient, InteractiveAlgo},
    graphs::{AdjListRepr, Edge, Graph, Vertex},
    pathfinding::{Dijkstra, Num},
    presentation::GraphEvent,
};

const SERVER_ADDRESS: &str = "127.0.0.1:3000";

struct SimpleEventClient<V, E> {
    events: Vec<GraphEvent<V, E>>,
}

impl<V, E> SimpleEventClient<V, E>
where
    V: Serialize,
    E: Serialize,
{
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    async fn flush(&mut self, sender: &mut SplitSink<WebSocket, Message>) {
        for event in &self.events {
            let serialized = serde_json::to_string(event).unwrap();
            let message = Message::Text(serialized.into());
            sender.send(message).await.unwrap();
        }
        self.events.clear();
    }
}

impl<V, E> EventClient<GraphEvent<V, E>> for SimpleEventClient<V, E> {
    fn consume(&mut self, event: GraphEvent<V, E>) {
        self.events.push(event);
    }
}

struct AppState {
    graph_blueprint: AdjListRepr<(), Num>,
}

type SharedAppState = Arc<AppState>;

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<SharedAppState>) -> Response {
    ws.on_upgrade(async move |socket| handle_socket(socket, state).await)
}

async fn handle_socket(socket: WebSocket, state: SharedAppState) {
    let (sender, receiver) = socket.split();
    socket_loop(sender, receiver, state).await;
}

async fn socket_loop(
    mut sender: SplitSink<WebSocket, Message>,
    mut receiver: SplitStream<WebSocket>,
    state: SharedAppState,
) {
    let mut client = SimpleEventClient::new();
    let mut dijkstra = Dijkstra::init((state.graph_blueprint.clone(), 0), &mut client);
    client.flush(&mut sender).await;
    loop {
        select! {
            Some(Ok(message)) = receiver.next() => {
                match message {
                    Message::Text(_utf8_bytes) => {
                        dijkstra.step(&mut client);
                        client.flush(&mut sender).await;
                    }
                    Message::Close(_) => break,
                    _ => todo!(),
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
    let vertices = make_random_smol_graph();

    let app_state = AppState {
        graph_blueprint: vertices,
    };

    let app = Router::new()
        .route("/ws", routing::any(ws_handler))
        .with_state(Arc::new(app_state));

    let listener = TcpListener::bind(SERVER_ADDRESS).await.unwrap();
    println!("Server running on {}", SERVER_ADDRESS);

    axum::serve(listener, app).await.unwrap();
}

fn make_random_smol_graph() -> AdjListRepr<(), Num> {
    let mut rng = rand::rng();
    let n = 10;
    let d = 2;
    let v = 8;
    let mut graph = AdjListRepr::with_size(n);
    for vertex_id in 0..n {
        let mut num_edges = 0;
        while num_edges < d {
            let end_id = rng.random_range(0..n);
            if end_id == vertex_id {
                continue;
            }
            let weight = rng.random_range(1..=v);
            graph.add_edge(vertex_id, end_id, weight);
            num_edges += 1;
        }
    }
    graph
}
