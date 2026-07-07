use serde::{Deserialize, Serialize};

use crate::{
    graphs::{Edge, Vertex},
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
    if current == 0 {
        return true;
    }

    let total = total as f64;
    let current_ratio = ((current as f64) / total * 100.0).floor() as usize;
    let previous_ratio = (((current - 1) as f64) / total * 100.0).floor() as usize;

    previous_ratio / 5 != current_ratio / 5
}
