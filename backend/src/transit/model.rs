use serde::Serialize;
use std::{collections::HashMap, hash::Hash, num::NonZeroU128};

use crate::transit::gtfs::{Gtfs, GtfsShapeEntry};

macro_rules! id_type {
    ($name:ident, $type:ty) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize)]
        pub struct $name(pub $type);
    };
}

id_type!(StopId, usize);
id_type!(MetaStopId, usize);
id_type!(ShapeId, usize);
id_type!(TripId, usize);
id_type!(RouteId, usize);

#[derive(Debug, Serialize, Clone, Copy)]
pub struct LatLng {
    pub latitude: f32,
    pub longitude: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct Stop {
    pub code: String,
    pub name: String,
    pub position: LatLng,
}

#[derive(Debug, Serialize, Clone)]
pub struct Shape {
    pub points: Vec<LatLng>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Trip {
    pub route_id: RouteId,
    pub shape_id: ShapeId,
    pub stops: Vec<StopId>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Route {
    pub short_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct StopTime {
    pub trip_id: TripId,
    pub stop_id: StopId,
}

// A bunch of stops named the same way
#[derive(Debug, Serialize, Clone)]
pub struct MetaStop {
    pub id: MetaStopId,
    pub name: String,
    pub stops: Vec<StopId>,
    pub position: LatLng,
}

fn avg(old: f32, new_point: f32, points: usize) -> f32 {
    return (old * points as f32 + new_point) / (points + 1) as f32;
}

impl MetaStop {
    fn add_stop(&mut self, stop_id: StopId, stop: &Stop) {
        self.position.latitude = avg(
            self.position.latitude,
            stop.position.latitude,
            self.stops.len(),
        );
        self.position.longitude = avg(
            self.position.longitude,
            stop.position.longitude,
            self.stops.len(),
        );
        self.stops.push(stop_id);
    }
}

pub struct TransitInfo {
    pub meta_stops: Vec<MetaStop>,
    pub meta_stops_by_name: HashMap<String, MetaStopId>,

    pub stops: Vec<Stop>,
    pub routes: Vec<Route>,
    pub shapes: Vec<Shape>,
    pub trips: Vec<Trip>,
}

impl TransitInfo {
    pub fn new() -> Self {
        Self {
            meta_stops: Vec::new(),
            meta_stops_by_name: HashMap::new(),
            stops: Vec::new(),
            routes: Vec::new(),
            shapes: Vec::new(),
            trips: Vec::new(),
        }
    }

    fn add_stops(&mut self, gtfs: &Gtfs) -> HashMap<String, StopId> {
        let mut stop_id_map = HashMap::new();

        for stop in gtfs.stops.iter() {
            let id = StopId(self.stops.len());
            let new_stop = Stop {
                code: stop.stop_code.clone(),
                name: stop.stop_name.clone(),
                position: LatLng {
                    latitude: stop.stop_lat,
                    longitude: stop.stop_lon,
                },
            };
            stop_id_map.insert(stop.stop_id.to_owned(), id);

            let MetaStopId(meta_stop_id) = if self.meta_stops_by_name.contains_key(&stop.stop_name)
            {
                *self.meta_stops_by_name.get(&stop.stop_name).unwrap()
            } else {
                let meta_stop_id = MetaStopId(self.meta_stops.len());
                self.meta_stops.push(MetaStop {
                    id: meta_stop_id,
                    name: stop.stop_name.clone(),
                    stops: Vec::new(),
                    position: LatLng {
                        latitude: 0.0,
                        longitude: 0.0,
                    },
                });
                self.meta_stops_by_name
                    .insert(stop.stop_name.clone(), meta_stop_id);
                meta_stop_id
            };

            self.meta_stops[meta_stop_id].add_stop(id, &new_stop);
            self.stops.push(new_stop);
        }

        stop_id_map
    }

    fn add_routes(&mut self, gtfs: &Gtfs) -> HashMap<String, RouteId> {
        let mut route_id_map = HashMap::new();

        for route in gtfs.routes.iter() {
            let id = RouteId(self.routes.len());
            route_id_map.insert(route.route_id.to_owned(), id);
            self.routes.push(Route {
                short_name: route.route_short_name.clone(),
            });
        }

        route_id_map
    }

    fn add_shapes(&mut self, gtfs: &Gtfs) -> HashMap<String, ShapeId> {
        let mut shape_entries: HashMap<String, Vec<&GtfsShapeEntry>> = HashMap::new();
        for shape_entry in &gtfs.shapes {
            if !shape_entries.contains_key(&shape_entry.shape_id) {
                shape_entries.insert(shape_entry.shape_id.clone(), Vec::new());
            }
            shape_entries
                .get_mut(&shape_entry.shape_id)
                .unwrap()
                .push(shape_entry);
        }

        let mut shape_id_map = HashMap::new();
        for (shape_id, mut entries) in shape_entries.into_iter() {
            entries.sort_by(|e1, e2| e1.shape_pt_sequence.cmp(&e2.shape_pt_sequence));
            let points = entries
                .into_iter()
                .map(|e| LatLng {
                    latitude: e.shape_pt_lat,
                    longitude: e.shape_pt_lon,
                })
                .collect();
            let id = ShapeId(self.shapes.len());
            shape_id_map.insert(shape_id, id);
            self.shapes.push(Shape { points })
        }

        shape_id_map
    }

    fn add_trips(
        &mut self,
        gtfs: &Gtfs,
        stop_id_map: &HashMap<String, StopId>,
        route_id_map: &HashMap<String, RouteId>,
        shape_id_map: &HashMap<String, ShapeId>,
    ) -> HashMap<String, TripId> {
        let mut trip_id_map = HashMap::new();

        for trip in &gtfs.trips {
            let id = TripId(self.trips.len());
            trip_id_map.insert(trip.trip_id.clone(), id);
            self.trips.push(Trip {
                route_id: *route_id_map.get(&trip.route_id).unwrap(),
                shape_id: *shape_id_map.get(&trip.shape_id).unwrap(),
                stops: Vec::new(),
            })
        }

        trip_id_map
    }

    pub fn add_gtfs(&mut self, gtfs: &Gtfs) {
        let stop_id_map = self.add_stops(&gtfs);
        let route_id_map = self.add_routes(&gtfs);
        let shape_id_map = self.add_shapes(&gtfs);
        self.add_trips(gtfs, &stop_id_map, &route_id_map, &shape_id_map);
    }
}
