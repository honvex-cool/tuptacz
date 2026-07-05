use serde::{Deserialize, Serialize};

use crate::graphs::{EdgeId, VertexId};

#[derive(Serialize, Deserialize)]
pub struct GraphEvent<V, E> {
    pub action: GraphAction<V, E>,
    pub comment: String,
}

#[derive(Serialize, Deserialize)]
pub enum HighlightMode {
    Visited,
    Awaiting,
    Source,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GraphAction<V, E> {
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
        properties: V,
    },
    AddEdge {
        id: EdgeId,
        start_id: VertexId,
        end_id: VertexId,
        properties: E,
    },
    AddShortcut {
        source: VertexId,
        target: VertexId,
    },
}
