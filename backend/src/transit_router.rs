use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};

use chrono::{FixedOffset, Utc};
use serde::{Deserialize, Serialize};

use crate::transit::{
    model::{MetaStop, MetaStopId, Shape, Stop, TransitInfo},
    raptor::search_path,
};

pub trait HasTransitInfo {
    fn transit_info(&self) -> &TransitInfo;
}

async fn get_stops<S>(State(state): State<Arc<S>>) -> Json<Vec<MetaStop>>
where
    S: HasTransitInfo + Send + Sync,
{
    Json(state.transit_info().meta_stops.iter().cloned().collect())
}

async fn get_shapes<S>(State(state): State<Arc<S>>) -> Json<Vec<Shape>>
where
    S: HasTransitInfo + Send + Sync,
{
    Json(
        state
            .transit_info()
            .shapes
            .iter()
            .cloned()
            .collect::<Vec<Shape>>(),
    )
}
#[derive(Debug, Serialize, Deserialize)]
struct SearchRequest {
    start: usize,
    end: usize,
    departure_time: chrono::DateTime<FixedOffset>,
}

async fn search<S>(State(state): State<Arc<S>>, Json(payload): Json<SearchRequest>) -> Json<()>
where
    S: HasTransitInfo + Send + Sync,
{
    println!("{:?}", payload);
    search_path(
        state.transit_info(),
        MetaStopId(payload.start),
        MetaStopId(payload.end),
        payload.departure_time,
    );
    Json(())
}

pub fn transit_router<S>() -> Router<Arc<S>>
where
    S: HasTransitInfo + Send + Sync + 'static,
{
    Router::new()
        .route("/stops", get(get_stops))
        .route("/shapes", get(get_shapes))
        .route("/search", post(search))
}
