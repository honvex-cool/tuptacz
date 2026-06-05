use csv::Reader;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
struct GtfsStop {
    stop_id: String,
    stop_code: String,
    stop_name: String,
    stop_lat: f32,
    stop_lon: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct LatLng {
    pub latitude: f32,
    pub longitude: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct Stop {
    pub id: String,
    pub code: String,
    pub name: String,
    pub point: LatLng,
}

#[derive(Debug, Deserialize, Clone)]
struct GtfsShapeEntry {
    shape_id: String,
    shape_pt_lat: f32,
    shape_pt_lon: f32,
    shape_pt_sequence: u32
}

#[derive(Debug, Serialize, Clone)]
pub struct Shape {
    pub id: String,
    pub points: Vec<LatLng>,
}

pub struct Trip {
    pub id: String,
    pub route_id: String,
    pub shape_id: String,
}

pub struct Route {
    pub id: String,
    pub short_name: String,
}

pub struct Gtfs {
    pub stops: HashMap<String, Stop>,
    pub shapes: HashMap<String, Shape>,
    pub routes: HashMap<String, Route>,
    pub trips: HashMap<String, Trip>,
}

impl From<GtfsStop> for Stop {
    fn from(s: GtfsStop) -> Self {
        Self {
            id: s.stop_id,
            code: s.stop_code,
            name: s.stop_name,
            point: LatLng {
                latitude: s.stop_lat,
                longitude: s.stop_lon,
            },
        }
    }
}

pub fn load_stops(path: &Path) -> HashMap<String, Stop> {
    let mut reader = Reader::from_path(path).unwrap();

    let mut stops: HashMap<String, Stop> = HashMap::new();

    for result in reader.deserialize::<GtfsStop>() {
        let stop = result.unwrap();
        stops.insert(stop.stop_id.clone(), stop.into());
    }

    stops
}

pub fn load_shapes(path: &Path) -> HashMap<String, Shape> {
    let mut reader = Reader::from_path(path).unwrap();

    let mut shape_entries: HashMap<String, Vec<GtfsShapeEntry>> = HashMap::new();

    for result in reader.deserialize::<GtfsShapeEntry>() {
        let shape_entry = result.unwrap();
        if !shape_entries.contains_key(&shape_entry.shape_id) {
           shape_entries.insert(shape_entry.shape_id.clone(), vec![]);
        }

        shape_entries
            .get_mut(&shape_entry.shape_id)
            .unwrap()
            .push(shape_entry.clone());
    }

    shape_entries
        .into_iter()
        .map(|(k, mut v)| {
            v.sort_by(|e1, e2| e1.shape_pt_sequence.cmp(&e2.shape_pt_sequence));
            (k.clone(), Shape {
                id: k,
                points: v.into_iter()
                .map(|p| LatLng { latitude: p.shape_pt_lat, longitude: p.shape_pt_lon})
                .collect()
            })
        })
        .collect()
}

pub fn load_gtfs(path: &Path) -> Gtfs {
    Gtfs {
        stops: load_stops(&path.join("stops.txt")),
        shapes: load_shapes(&path.join("shapes.txt")),
        routes: HashMap::new(),
        trips: HashMap::new(),
    }
}
