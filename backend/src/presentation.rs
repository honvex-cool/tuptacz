use serde::{Deserialize, Serialize};

use crate::graphs::Graph;

#[derive(Serialize, Deserialize)]
pub enum HighlightMode {
    Visited,
    Awaiting
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerAction<V, E> {
    InitGraph {
        graph: Graph<V, E>,
    },
    HighlightVertex {
        id: usize,
        mode: HighlightMode,
    },
    HideVertex {
        id: usize,
    },
    HighlightEdge {
        id: usize,
        mode: HighlightMode,
    },
    AddVertex {
        id: usize,
    },
    AddEdge {
        id: usize,
        start_id: usize,
        end_id: usize,
    },
}

#[derive(Serialize, Deserialize)]
pub struct GraphEvent<V, E> {
    pub action: ServerAction<V, E>,
    pub comment: String,
}
