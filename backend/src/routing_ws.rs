use futures_util::{SinkExt, StreamExt};
use ouroboros::self_referencing;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

use crate::{
    app::SharedState,
    graphs::{Path, VertexId},
    routing::{
        a_star::algos::a_star,
        dijkstra::algos::dijkstra,
        model::{LatLng, Road, RoutingInfo, RoutingNetwork},
        presentation::GraphEvent,
    },
    utils::algo::{self, EventClient, InteractiveAlgo},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ControlEvent {
    AvailableRoutingNetworks {
        routing_network_names: Vec<String>,
    },
    RoutingNetworkReady,
    PreprocessingReady,
    PreprocessingDone,
    QueryReady,
    QueryDone {
        path: Option<Path<LatLng, Road>>,
    },
    StepDone,
    ClosestVertexResponse {
        name: String,
        lat_lng: LatLng,
        id: VertexId,
    },
}

type AlgoEvent = GraphEvent<LatLng, Road>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerEvent {
    Control { event: ControlEvent },
    Algo { event: AlgoEvent },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AlgorithmSelection {
    Dijkstra { is_bidirectional: bool },
    AStar { is_bidirectional: bool },
    ContractionHierarchies,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FrontendEvent {
    SelectRoutingNetwork {
        routing_network_name: String,
    },
    SelectAlgorithm {
        algorithm_selection: AlgorithmSelection,
    },
    StepPreprocessing,
    RunPreprocessingToCompletion,
    SelectQuery {
        source_id: VertexId,
        target_id: VertexId,
    },
    StepQuery,
    RunQueryToCompletion,
    ClosestVertexRequest {
        name: String,
        lat_lng: LatLng,
    },
}

struct SimpleEventClient<V, E> {
    events: Vec<GraphEvent<V, E>>,
    is_enabled: bool,
}

impl SimpleEventClient<LatLng, Road> {
    fn new(is_enabled: bool) -> Self {
        Self {
            events: Vec::new(),
            is_enabled,
        }
    }

    async fn flush(&mut self, sender: &mut Sender) {
        for event in std::mem::take(&mut self.events) {
            algo_event(event, sender).await;
        }
    }
}

impl<V, E> EventClient<GraphEvent<V, E>> for SimpleEventClient<V, E> {
    fn consume(&mut self, event: GraphEvent<V, E>) {
        if self.is_enabled {
            self.events.push(event);
        }
    }
}

type Event = GraphEvent<LatLng, Road>;
type Client = SimpleEventClient<LatLng, Road>;

type QueryAlgo<'q> = dyn InteractiveAlgo<Client, Event, Result = Option<Path<LatLng, Road>>> + 'q;

type RoutingAlgo = crate::routing::RoutingAlgo<'static, LatLng, Road, Client>;
type Pathfinder = crate::routing::Pathfinder<'static, LatLng, Road, Client>;

#[self_referencing]
struct RunningQuery {
    engine: Box<Pathfinder>,
    #[borrows(mut engine)]
    #[not_covariant]
    query: Option<Box<QueryAlgo<'this>>>,
}

enum AlgoState {
    None,
    Preprocessing(Box<RoutingAlgo>),
    Queryable(Box<Pathfinder>),
    Query(RunningQuery),
}

struct LocalState<'r> {
    routing_info: &'r RoutingInfo,
    routing_network: Option<&'r RoutingNetwork<Road>>,
    algo_state: AlgoState,
}

impl<'r> LocalState<'r> {
    fn new(routing_info: &'r RoutingInfo) -> Self {
        Self {
            routing_info,
            routing_network: None,
            algo_state: AlgoState::None,
        }
    }

    async fn handle_frontend_event(&mut self, frontend_event: FrontendEvent, sender: &mut Sender) {
        // eprintln!("Received: {:?}", frontend_event);

        let mut is_change_preprocessing_to_queryable = false;
        let mut is_query_done = false;

        match frontend_event {
            FrontendEvent::SelectRoutingNetwork {
                routing_network_name,
            } => {
                self.routing_network = self.routing_info.get(&routing_network_name);
                self.algo_state = AlgoState::None;
                control_event(ControlEvent::RoutingNetworkReady, sender).await;
            }
            FrontendEvent::SelectAlgorithm {
                algorithm_selection,
            } => {
                let algo = self.get_algo(algorithm_selection);
                self.algo_state = AlgoState::Preprocessing(algo);
                control_event(ControlEvent::PreprocessingReady, sender).await;
            }
            FrontendEvent::StepPreprocessing => {
                if let AlgoState::Preprocessing(preprocessing) = &mut self.algo_state {
                    let mut client = Client::new(true);
                    is_change_preprocessing_to_queryable = !preprocessing.step(&mut client);
                    client.flush(sender).await;
                    control_event(ControlEvent::StepDone, sender).await;
                }
            }
            FrontendEvent::RunPreprocessingToCompletion => {
                if let AlgoState::Preprocessing(preprocessing) = &mut self.algo_state {
                    let mut client = Client::new(false);
                    algo::complete_dyn(preprocessing.as_mut(), &mut client);
                    is_change_preprocessing_to_queryable = true;
                }
            }
            FrontendEvent::SelectQuery {
                source_id,
                target_id,
            } => {
                let engine = std::mem::replace(&mut self.algo_state, AlgoState::None);
                if let AlgoState::Queryable(engine) = engine {
                    let input = (source_id, target_id);
                    let mut client = Client::new(true);
                    let running_query = RunningQueryBuilder {
                        engine,
                        query_builder: |engine| Some(engine.query(input, &mut client)),
                    }
                    .build();
                    self.algo_state = AlgoState::Query(running_query);
                    client.flush(sender).await;
                    control_event(ControlEvent::QueryReady, sender).await;
                }
            }
            FrontendEvent::StepQuery => {
                if let AlgoState::Query(running_query) = &mut self.algo_state {
                    let mut client = Client::new(true);
                    is_query_done = running_query
                        .with_query_mut(|query| !query.as_mut().unwrap().step(&mut client));
                    client.flush(sender).await;
                    control_event(ControlEvent::StepDone, sender).await;
                }
            }
            FrontendEvent::RunQueryToCompletion => {
                if let AlgoState::Query(running_query) = &mut self.algo_state {
                    is_query_done = true;
                    let mut client = Client::new(false);
                    running_query.with_query_mut(|query| {
                        algo::complete_dyn(query.as_mut().unwrap().as_mut(), &mut client)
                    });
                }
            }
            FrontendEvent::ClosestVertexRequest { name, lat_lng } => {
                if let Some(routing_network) = self.routing_network {
                    let id = routing_network.nerest_vertex_id(lat_lng);
                    let lat_lng = routing_network.graph_elements.vertices[id];
                    let response = ControlEvent::ClosestVertexResponse { name, lat_lng, id };
                    control_event(response, sender).await;
                }
            }
        }

        if is_change_preprocessing_to_queryable {
            let preprocessing = std::mem::replace(&mut self.algo_state, AlgoState::None);
            if let AlgoState::Preprocessing(preprocessing) = preprocessing {
                let engine = preprocessing.result_dyn();
                self.algo_state = AlgoState::Queryable(engine);
                control_event(ControlEvent::PreprocessingDone, sender).await;
            }
        }

        if is_query_done {
            let algo_state = std::mem::replace(&mut self.algo_state, AlgoState::None);
            if let AlgoState::Query(mut running_query) = algo_state {
                let path = running_query.with_query_mut(|query| query.take().unwrap().result_dyn());
                let engine = running_query.into_heads().engine;
                self.algo_state = AlgoState::Queryable(engine);
                control_event(ControlEvent::QueryDone { path }, sender).await;
            }
        }
    }

    fn get_algo(&self, algorithm_selection: AlgorithmSelection) -> Box<RoutingAlgo> {
        let graph_elements = self.routing_network.unwrap().graph_elements.clone();
        match algorithm_selection {
            AlgorithmSelection::Dijkstra { is_bidirectional } => {
                dijkstra(graph_elements, is_bidirectional)
            }
            AlgorithmSelection::AStar { is_bidirectional } => {
                a_star(graph_elements, is_bidirectional)
            }
            AlgorithmSelection::ContractionHierarchies => todo!(),
        }
    }
}

type Socket = tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>;
type Sender = futures_util::stream::SplitSink<Socket, Message>;
type Receiver = futures_util::stream::SplitStream<Socket>;

pub async fn handle_socket(socket: Socket, state: SharedState) {
    let (sender, receiver) = socket.split();
    socket_loop(sender, receiver, &state.routing_info).await;
}

async fn socket_loop(mut sender: Sender, mut receiver: Receiver, routing_info: &RoutingInfo) {
    eprintln!("Connected to socket");

    let mut state = LocalState::new(routing_info);

    control_event(
        ControlEvent::AvailableRoutingNetworks {
            routing_network_names: routing_info.keys().cloned().collect(),
        },
        &mut sender,
    )
    .await;

    while let Some(Ok(message)) = receiver.next().await {
        match message {
            Message::Text(utf8_bytes) => {
                let frontend_event = serde_json::from_str(&utf8_bytes).unwrap();
                state
                    .handle_frontend_event(frontend_event, &mut sender)
                    .await;
            }
            Message::Close(_) => eprintln!("Socket connection closed"),
            _ => {}
        }
    }
}

async fn algo_event(event: AlgoEvent, sender: &mut Sender) {
    server_event(ServerEvent::Algo { event }, sender).await;
}

async fn control_event(event: ControlEvent, sender: &mut Sender) {
    server_event(ServerEvent::Control { event }, sender).await;
}

async fn server_event(event: ServerEvent, sender: &mut Sender) {
    // eprintln!("Sent: {:?}", event);
    let serialized = serde_json::to_string(&event).unwrap();
    let message = Message::Text(serialized.into());
    sender.send(message).await.unwrap();
}
