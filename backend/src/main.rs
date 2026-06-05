use std::path::Path;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
    routing::{any, get},
};

use futures_util::{
    SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};

use serde::Serialize;

use tokio::net::TcpListener;
use tokio::select;

use tuptacz::{
    algo::{EventClient, InteractiveAlgo},
    graphs::{Edge, Graph, repr::AdjList},
    gtfs::{Gtfs, Route, Shape, Stop, load_gtfs},
    pathfinding::{Num, dijkstra::Dijkstra},
    presentation::{self, GraphEvent},
    roads::{Intersection, Road, load},
};

const SERVER_ADDRESS: &str = "0.0.0.0:3000";

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
    graph: AdjList<Intersection, Road>,
    gtfs: Gtfs,
}

type SharedAppState = Arc<AppState>;

async fn health_check_handler() -> &'static str {
    "Backend up."
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<SharedAppState>) -> Response {
    ws.on_upgrade(async move |socket| handle_socket(socket, state).await)
}

async fn handle_socket(socket: WebSocket, state: SharedAppState) {
    let (sender, receiver) = socket.split();
    socket_loop(sender, receiver, state).await;
}

fn init_graph_event<V, E>(graph: &AdjList<V, E>) -> GraphEvent<V, E>
where
    V: Clone,
    E: Clone,
{
    let vertices = graph.iter_vertices().map(|v| v.clone()).collect();
    let edges = graph
        .iter_edges()
        .map(|(source, target, properties)| presentation::Edge {
            source,
            target,
            properties: properties.clone(),
        })
        .collect();

    return GraphEvent {
        action: presentation::ServerAction::InitGraph {
            vertices: vertices,
            edges: edges,
        },
        comment: "Initialized graph".to_owned(),
    };
}

async fn socket_loop(
    mut sender: SplitSink<WebSocket, Message>,
    mut receiver: SplitStream<WebSocket>,
    state: SharedAppState,
) {
    let mut client = SimpleEventClient::<Intersection, Road>::new();
    // let mut dijkstra = Dijkstra::init((state.graph.clone(), 0), &mut client);
    client.flush(&mut sender).await;

    let event = init_graph_event(&state.graph);
    let serialized = serde_json::to_string(&event).unwrap();
    let message = Message::Text(serialized.into());
    sender.send(message).await.unwrap();

    loop {
        select! {
            Some(Ok(message)) = receiver.next() => {
                match message {
                    Message::Text(_utf8_bytes) => {
                        // dijkstra.step(&mut client);
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

async fn get_stops(State(state): State<Arc<AppState>>) -> Json<Vec<Stop>> {
    Json((&state.gtfs).stops.values().cloned().collect::<Vec<Stop>>())
}

async fn get_shapes(State(state): State<Arc<AppState>>) -> Json<Vec<Shape>> {
    Json(
        (&state.gtfs)
            .shapes
            .values()
            .cloned()
            .collect::<Vec<Shape>>(),
    )
}

fn gtfs_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/stops", get(get_stops))
        .route("/shapes", get(get_shapes))
}

#[tokio::main]
async fn main() {
    let graph = load("maps/krakow.osm.pbf").unwrap();

    let gtfs = load_gtfs(Path::new("gtfs/KRK/T"));

    let app_state = AppState {
        graph: graph,
        gtfs: gtfs,
    };

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .route("/api/health-check", get(health_check_handler))
        .nest("/api/gtfs", gtfs_router())
        .with_state(Arc::new(app_state));

    let listener = TcpListener::bind(SERVER_ADDRESS).await.unwrap();
    println!("Server running on {}", SERVER_ADDRESS);

    axum::serve(listener, app).await.unwrap();
}
