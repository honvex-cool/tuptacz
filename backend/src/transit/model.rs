// A representation of transit network structure a bit more than raw GTFS
use crate::{
    transit::gtfs::{
        Gtfs,
        GtfsDateExceptionType::{ServiceAdded, ServiceRemoved},
        GtfsServiceAvailability::{Available, Unavailable},
        GtfsShapeEntry, GtfsStopTime, ServiceDate, ServiceTime,
    },
};
use chrono::{Datelike, Weekday};
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, collections::HashSet};


// Macro for definitions of id-like types that use the same underlying represenation, but we want to distinguish them in code.
macro_rules! id_type {
    ($name:ident, $type:ty) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub $type);
    };
}

// GTFS represents calendar with a column for each day, so to avoid duplication 7 times we use this macro.
macro_rules! push_if_available {
    ($field:expr, $weekday:ident, $dest:expr) => {
        if let Available = $field {
            $dest.push(Weekday::$weekday);
        }
    };
}

id_type!(StopId, usize);
id_type!(MetaStopId, usize);
id_type!(ShapeId, usize);
id_type!(TripPatternId, usize);
id_type!(TripId, usize);
id_type!(RouteId, usize);
id_type!(ServiceId, usize);

#[derive(Debug, Serialize, Clone, Copy)]
pub struct LatLng {
    pub latitude: f32,
    pub longitude: f32,
}

impl LatLng {
    const EARTH_RADIUS_METERS: f32 = 6_371_000.0;

    // https://en.wikipedia.org/wiki/Haversine_formula#Formulation
    pub fn distance_meters(&self, other: LatLng) -> f32 {
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let lon1 = self.longitude.to_radians();
        let lon2 = other.longitude.to_radians();

        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;

        let lat_sin = (dlat / 2.0).sin();
        let lon_sin = (dlon / 2.0).sin();

        let hav_theta = lat_sin * lat_sin + lon_sin * lon_sin * lat1.cos() * lat2.cos();

        let theta = hav_theta.sqrt().asin() * 2.0;

        return theta * Self::EARTH_RADIUS_METERS;
    }
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
pub struct TripPattern {
    pub stops: Vec<StopId>,
    pub trips: Vec<TripId>,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub enum RouteType {
    Bus,
    Tram
}

impl RouteType {
    fn from_id(id: u32) -> Self {
        match id {
            3 => Self::Bus,
            900 => Self::Tram,
            _ => panic!("Unknown route type, {}", id)
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Route {
    pub short_name: String,
    pub route_type: RouteType
}

#[derive(Debug, Serialize, Clone)]
pub struct StopTime {
    pub stop_id: StopId,
    pub arrival_time: ServiceTime,
    pub departure_time: ServiceTime,
}

#[derive(Debug, Serialize, Clone)]
pub struct Trip {
    pub route_id: RouteId,
    pub shape_id: ShapeId,
    pub service_id: ServiceId,
    pub trip_pattern_id: TripPatternId,
    pub stop_times: Vec<StopTime>,
}

impl Trip {
    pub fn start_time(&self) -> ServiceTime {
        self.stop_times[0].arrival_time
    }
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

#[derive(Debug)]
pub struct ServiceWeekdaySchedule {
    pub weekday: Weekday,
    pub start_date: ServiceDate,
    pub end_date: ServiceDate,
}

#[derive(Debug)]
pub struct ServiceCalendar {
    pub service_name: String,
    pub active_weekdays: Vec<ServiceWeekdaySchedule>,
    pub active_dates: HashSet<ServiceDate>,
    pub inactive_dates: HashSet<ServiceDate>,
}

#[derive(Debug)]
pub struct FootPath {
    pub to: StopId,
    pub distance_meters: f32,
}

pub struct TransitInfo {
    pub meta_stops: Vec<MetaStop>,
    pub meta_stops_by_name: HashMap<String, MetaStopId>,

    pub stops: Vec<Stop>,
    pub routes: Vec<Route>,
    pub shapes: Vec<Shape>,
    trips: Vec<Trip>,
    pub services: Vec<ServiceCalendar>,
    pub trip_patterns: Vec<TripPattern>,
    pub foot_paths: Vec<Vec<FootPath>>,
}

impl TransitInfo {
    const MAX_FOOTPATH_DISTANCE_METERS: f32 = 50.0;

    pub fn new() -> Self {
        Self {
            meta_stops: Vec::new(),
            meta_stops_by_name: HashMap::new(),
            stops: Vec::new(),
            routes: Vec::new(),
            shapes: Vec::new(),
            trips: Vec::new(),
            services: Vec::new(),
            trip_patterns: Vec::new(),
            foot_paths: Vec::new(),
        }
    }

    pub fn get_trip(&self, trip_id: TripId) -> &Trip {
        &self.trips[trip_id.0]
    }

    pub fn get_trip_stops(&self, trip_id: TripId) -> &[StopId] {
        let trip = self.get_trip(trip_id);
        let pattern = &self.trip_patterns[trip.trip_pattern_id.0];
        &pattern.stops
    }

    pub fn get_route(&self, route_id: RouteId) -> &Route {
        &self.routes[route_id.0]
    }

    pub fn get_stop(&self, stop_id: StopId) -> &Stop {
        &self.stops[stop_id.0]
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
            self.foot_paths.push(Vec::new());
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
                route_type: RouteType::from_id(route.route_type)
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

    fn add_calendar(&mut self, gtfs: &Gtfs) -> HashMap<String, ServiceId> {
        let mut service_id_map = HashMap::new();

        for calendar_entry in &gtfs.calendar {
            let ServiceId(id) = if service_id_map.contains_key(&calendar_entry.service_id) {
                *service_id_map.get(&calendar_entry.service_id).unwrap()
            } else {
                let id = ServiceId(self.services.len());
                service_id_map.insert(calendar_entry.service_id.clone(), id);
                self.services.push(ServiceCalendar {
                    service_name: calendar_entry.service_id.clone(),
                    active_weekdays: Vec::new(),
                    active_dates: HashSet::new(),
                    inactive_dates: HashSet::new(),
                });
                id
            };

            let mut active_weekdays = Vec::new();
            push_if_available!(calendar_entry.monday, Mon, active_weekdays);
            push_if_available!(calendar_entry.tuesday, Tue, active_weekdays);
            push_if_available!(calendar_entry.wednesday, Wed, active_weekdays);
            push_if_available!(calendar_entry.thursday, Thu, active_weekdays);
            push_if_available!(calendar_entry.friday, Fri, active_weekdays);
            push_if_available!(calendar_entry.saturday, Sat, active_weekdays);
            push_if_available!(calendar_entry.sunday, Sun, active_weekdays);

            self.services[id].active_weekdays = active_weekdays
                .into_iter()
                .map(|w| ServiceWeekdaySchedule {
                    weekday: w,
                    start_date: calendar_entry.start_date,
                    end_date: calendar_entry.end_date,
                })
                .collect();
        }

        for calendar_date_entry in &gtfs.calendar_dates {
            let ServiceId(id) = if service_id_map.contains_key(&calendar_date_entry.service_id) {
                *service_id_map.get(&calendar_date_entry.service_id).unwrap()
            } else {
                let id = ServiceId(self.services.len());
                service_id_map.insert(calendar_date_entry.service_id.clone(), id);
                self.services.push(ServiceCalendar {
                    service_name: calendar_date_entry.service_id.clone(),
                    active_weekdays: Vec::new(),
                    active_dates: HashSet::new(),
                    inactive_dates: HashSet::new(),
                });
                id
            };

            match calendar_date_entry.exception_type {
                ServiceAdded => {
                    self.services[id]
                        .active_dates
                        .insert(calendar_date_entry.date);
                }
                ServiceRemoved => {
                    self.services[id]
                        .inactive_dates
                        .insert(calendar_date_entry.date);
                }
            }
        }

        service_id_map
    }

    fn group_stop_times_by_trip(
        &self,
        gtfs: &Gtfs,
        stop_id_map: &HashMap<String, StopId>,
    ) -> HashMap<String, Vec<StopTime>> {
        let mut stop_times: HashMap<String, Vec<&GtfsStopTime>> = HashMap::new();

        for gtfs_stop_time in &gtfs.stop_times {
            if !stop_times.contains_key(&gtfs_stop_time.trip_id) {
                stop_times.insert(gtfs_stop_time.trip_id.clone(), Vec::new());
            }

            stop_times
                .get_mut(&gtfs_stop_time.trip_id)
                .unwrap()
                .push(gtfs_stop_time);
        }

        stop_times
            .iter_mut()
            .map(|(trip_id, gtfs_stop_times)| {
                gtfs_stop_times.sort_by(|s1, s2| s1.stop_sequence.cmp(&s2.stop_sequence));
                let stop_times = gtfs_stop_times
                    .iter()
                    .map(|s| StopTime {
                        stop_id: *stop_id_map.get(&s.stop_id).unwrap(),
                        arrival_time: s.arrival_time,
                        departure_time: s.departure_time,
                    })
                    .collect();
                (trip_id.clone(), stop_times)
            })
            .collect()
    }

    fn create_trip_patterns(
        &mut self,
        stop_times_map: &HashMap<String, Vec<StopTime>>,
    ) -> HashMap<String, TripPatternId> {
        let mut trip_patterns: HashMap<Vec<StopId>, TripPatternId> = HashMap::new();
        let mut trip_pattern_map = HashMap::new();

        for (trip_id, stop_times) in stop_times_map.iter() {
            let stops: Vec<StopId> = stop_times.iter().map(|s| s.stop_id).collect();

            let pattern_id = if !trip_patterns.contains_key(&stops) {
                let id = TripPatternId(self.trip_patterns.len());
                self.trip_patterns.push(TripPattern {
                    stops: stops.clone(),
                    trips: Vec::new(),
                });

                trip_patterns.insert(stops, id);
                id
            } else {
                *trip_patterns.get(&stops).unwrap()
            };

            trip_pattern_map.insert(trip_id.clone(), pattern_id);
        }

        trip_pattern_map
    }

    fn add_trips(
        &mut self,
        gtfs: &Gtfs,
        route_id_map: &HashMap<String, RouteId>,
        shape_id_map: &HashMap<String, ShapeId>,
        service_id_map: &HashMap<String, ServiceId>,
        mut stop_times_map: HashMap<String, Vec<StopTime>>,
        trip_patterns_map: &HashMap<String, TripPatternId>,
    ) -> HashMap<String, TripId> {
        let mut trip_id_map = HashMap::new();

        for trip in &gtfs.trips {
            let id = TripId(self.trips.len());
            trip_id_map.insert(trip.trip_id.clone(), id);
            let trip_pattern_id = *trip_patterns_map.get(&trip.trip_id).unwrap();
            self.trips.push(Trip {
                route_id: *route_id_map.get(&trip.route_id).unwrap(),
                shape_id: *shape_id_map.get(&trip.shape_id).unwrap(),
                service_id: *service_id_map.get(&trip.service_id).unwrap(),
                stop_times: stop_times_map.remove(&trip.trip_id).unwrap(),
                trip_pattern_id: trip_pattern_id,
            });
            self.trip_patterns[trip_pattern_id.0].trips.push(id);
        }

        trip_id_map
    }

    fn sort_trip_patterns(&mut self) {
        self.trip_patterns.iter_mut().for_each(|p| {
            p.trips.sort_by(|t1, t2| {
                self.trips[t1.0]
                    .start_time()
                    .cmp(&self.trips[t2.0].start_time())
            })
        });
    }

    fn update_footpaths(&mut self, new_stops: Vec<StopId>) {
        // Naive implementation that assumes we can always walk in straight line between stops.
        // In Kraków this mostly works.
        for (stop_idx, stop) in self.stops.iter().enumerate() {
            for new_stop_id in &new_stops {
                let new_stop = &self.stops[new_stop_id.0];

                let distance_meters = stop.position.distance_meters(new_stop.position);
                if distance_meters <= Self::MAX_FOOTPATH_DISTANCE_METERS {
                    self.foot_paths[stop_idx].push(FootPath {
                        to: *new_stop_id,
                        distance_meters: distance_meters,
                    });
                    self.foot_paths[new_stop_id.0].push(FootPath {
                        to: StopId(stop_idx),
                        distance_meters: distance_meters,
                    })
                }
            }
        }
    }

    pub fn add_gtfs(&mut self, gtfs: &Gtfs) {
        let stop_id_map = self.add_stops(&gtfs);
        let route_id_map = self.add_routes(&gtfs);
        let shape_id_map = self.add_shapes(&gtfs);
        let service_id_map = self.add_calendar(gtfs);
        let stop_times_map = self.group_stop_times_by_trip(gtfs, &stop_id_map);
        let trip_patterns_map = self.create_trip_patterns(&stop_times_map);
        self.sort_trip_patterns();
        self.add_trips(
            gtfs,
            &route_id_map,
            &shape_id_map,
            &service_id_map,
            stop_times_map,
            &trip_patterns_map,
        );
        self.update_footpaths(stop_id_map.values().copied().collect());
    }

    pub fn trip_active(&self, trip_id: TripId, date: ServiceDate) -> bool {
        let service_id = self.trips[trip_id.0].service_id;
        let calendar = &self.services[service_id.0];

        if calendar.active_dates.contains(&date) {
            true
        } else if calendar.inactive_dates.contains(&date) {
            false
        } else {
            let weekday = date.weekday();
            for active_weekday_schedule in &calendar.active_weekdays {
                if weekday == active_weekday_schedule.weekday
                    && active_weekday_schedule.start_date <= date
                    && date <= active_weekday_schedule.end_date
                {
                    return true;
                }
            }
            false
        }
    }
}
