use serde::{Deserialize, Serialize};

use crate::graphs::{Edge, EdgeId, VertexId};

#[derive(Serialize, Deserialize)]
pub enum HighlightMode {
    Visited,
    Awaiting,
    Source,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerAction<V, E> {
    InitGraph {
        vertices: Vec<(VertexId, V)>,
        edges: Vec<Edge<E>>,
    },
    HighlightVertex {
        id: VertexId,
        mode: HighlightMode,
    },
    HideVertex {
        id: VertexId,
    },
    HighlightEdge {
        id: EdgeId,
        mode: HighlightMode,
    },
    AddVertex {
        id: EdgeId,
    },
    AddEdge {
        id: EdgeId,
        start_id: VertexId,
        end_id: VertexId,
    },
}

#[derive(Serialize, Deserialize)]
pub struct GraphEvent<V, E> {
    pub action: ServerAction<V, E>,
    pub comment: String,
}
