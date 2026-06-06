use csv::Reader;
use serde::{Deserialize, de::DeserializeOwned};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct GtfsStop {
    pub stop_id: String,
    pub stop_code: String,
    pub stop_name: String,
    pub stop_lat: f32,
    pub stop_lon: f32,
}

#[derive(Debug, Deserialize)]
pub struct GtfsShapeEntry {
    pub shape_id: String,
    pub shape_pt_lat: f32,
    pub shape_pt_lon: f32,
    pub shape_pt_sequence: u32,
}

#[derive(Debug, Deserialize)]
pub struct GtfsRoute {
    pub route_id: String,
    pub route_short_name: String,
}

#[derive(Debug, Deserialize)]
pub struct GtfsTrip {
    pub trip_id: String,
    pub route_id: String,
    pub shape_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GtfsStopTime {
    pub trip_id: String,
    pub stop_id: String,
    pub arrival_time: String,
    pub departure_time: String,
}

pub struct Gtfs {
    pub stops: Vec<GtfsStop>,
    pub shapes: Vec<GtfsShapeEntry>,
    pub routes: Vec<GtfsRoute>,
    pub trips: Vec<GtfsTrip>,
    pub stop_times: Vec<GtfsStopTime>,
}

fn load<T>(path: &Path) -> Vec<T>
where
    T: DeserializeOwned,
{
    let mut reader = Reader::from_path(path).unwrap();

    let mut rows = Vec::new();

    for result in reader.deserialize::<T>() {
        let row = result.unwrap();
        rows.push(row);
    }

    rows
}

pub fn load_gtfs(path: &Path) -> Gtfs {
    Gtfs {
        stops: load(&path.join("stops.txt")),
        shapes: load(&path.join("shapes.txt")),
        routes: load(&path.join("routes.txt")),
        trips: load(&path.join("trips.txt")),
        stop_times: load(&path.join("stop_times.txt")),
    }
}
