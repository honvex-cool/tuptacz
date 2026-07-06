use serde::{Deserialize, Serialize};

use crate::graphs::{Edge, Vertex};

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
}
