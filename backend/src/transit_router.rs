use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

use chrono::{FixedOffset, Utc};
use serde::{Deserialize, Serialize};

use crate::transit::{
    model::{MetaStop, MetaStopId, Route, Shape, Stop, StopTime, TransitInfo, Trip, TripId},
    raptor::{Journey, search_journeys},
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

#[derive(Serialize, Clone)]
struct TripDto {
    route: Route,
    stop_times: Vec<StopTime>,
    stops: Vec<Stop>,
}
async fn get_trip<S>(State(state): State<Arc<S>>, Path(trip_id): Path<TripId>) -> Json<TripDto>
where
    S: HasTransitInfo + Send + Sync + 'static,
{
    let trip = &state.transit_info().trips[trip_id.0];

    Json(TripDto {
        route: state.transit_info().routes[trip.route_id.0].clone(),
        stop_times: trip.stop_times.clone(),
        stops: state.transit_info().trip_patterns[trip.trip_pattern_id.0]
            .stops
            .iter()
            .map(|stop_id| state.transit_info().stops[stop_id.0].clone())
            .collect()
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchRequest {
    start: usize,
    end: usize,
    departure_time: chrono::DateTime<FixedOffset>,
}

async fn search<S>(
    State(state): State<Arc<S>>,
    Json(payload): Json<SearchRequest>,
) -> Json<Vec<Journey>>
where
    S: HasTransitInfo + Send + Sync,
{
    println!("{:?}", payload);
    let journeys = search_journeys(
        state.transit_info(),
        MetaStopId(payload.start),
        MetaStopId(payload.end),
        payload.departure_time,
    );
    Json(journeys)
}

pub fn transit_router<S>() -> Router<Arc<S>>
where
    S: HasTransitInfo + Send + Sync + 'static,
{
    Router::new()
        .route("/stops", get(get_stops))
        .route("/shapes", get(get_shapes))
        .route("/trip/{trip_id}", get(get_trip))
        .route("/search", post(search))
}
