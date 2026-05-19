use serde::{Deserialize, Serialize};

use crate::graphs::{EdgeId, VertexId};

#[derive(Serialize, Deserialize)]
pub enum HighlightMode {
    Visited,
    Awaiting,
    Source,
}


#[derive(Serialize, Deserialize)]
pub struct Edge<E> {
    pub source: VertexId,
    pub target: VertexId,
    pub properties: E
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerAction<V, E> {
    InitGraph {
        vertices: Vec<V>,
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
    AddShortcut {
        source: VertexId,
        target: VertexId
    }
}

#[derive(Serialize, Deserialize)]
pub struct GraphEvent<V, E> {
    pub action: ServerAction<V, E>,
    pub comment: String,
}
