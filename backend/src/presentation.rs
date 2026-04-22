use std::future::Future;

use serde::{Deserialize, Serialize};

use crate::graphs::Graph;

#[derive(Serialize, Deserialize)]
pub enum HighlightMode {}

#[derive(Serialize, Deserialize)]
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

pub trait EventClient<V, E> {
    fn consume(&mut self, event: &GraphEvent<V, E>) -> impl Future<Output = ()> + Send;
}
