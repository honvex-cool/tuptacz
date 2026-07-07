use serde::{Deserialize, Serialize};

use crate::{
    graphs::{Edge, Vertex},
    routing::ch::{Priority, PriorityParts, Stats},
    utils::algo::EventClient,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphEvent<V, E> {
    pub action: GraphAction<V, E>,
    pub comment: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HighlightMode {
    Visited,
    Awaiting,
    Source,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GraphAction<V, E> {
    HighlightVertex {
        vertex: Vertex<V>,
        mode: HighlightMode,
    },
    HighlightEdge {
        edge: Edge<V, E>,
        mode: HighlightMode,
    },
    Contraction {
        vertex: Vertex<V>,
        shortcuts: Vec<(usize, Edge<V, E>, Edge<V, E>)>,
    },
    LazyUpdate {
        vertex: Vertex<V>,
    },
    UpdateInGlobal {
        vertex: Vertex<V>,
        coefficients: PriorityParts,
        terms: PriorityParts,
        priority: Priority,
    },
    GlobalUpdateTriggered,
    QuerySummary {
        num_settled_vertices: usize,
        num_inspected_edges: usize,
    },
    ContractionSummary {
        stats: Stats,
    },
    Interrupt,
    Progress {
        current: usize,
        total: usize,
    },
}

#[inline(always)]
pub fn consider_progress_event<V, E, C>(current: usize, total: usize, client: &mut C)
where
    C: EventClient<GraphEvent<V, E>>,
{
    if is_relevant_ratio(current, total) {
        let event = GraphEvent {
            action: GraphAction::Progress { current, total },
            comment: "Progress made".to_owned(),
        };
        client.consume(event);
    }
}

#[inline(always)]
fn is_relevant_ratio(current: usize, total: usize) -> bool {
    if current == 0 || total == 0 {
        return false;
    }

    let total = total as f64;
    let current_percent = ((current as f64) / total * 100.0).floor() as usize;
    let previous_percent = (((current - 1) as f64) / total * 100.0).floor() as usize;

    previous_percent != current_percent
}
