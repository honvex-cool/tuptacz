use std::f32::consts::E;

use chrono::{DateTime, FixedOffset, NaiveTime, Timelike};
use serde::Serialize;

use crate::transit;
use crate::transit::gtfs::{ServiceDate, ServiceTime};
use crate::transit::model::{
    FootPath, MetaStopId, Stop, StopId, TransitInfo, Trip, TripId, TripPattern, TripPatternId,
};

const ROUNDS: u32 = 5;

#[derive(Copy, Clone)]
struct TravelTrip {
    trip_id: TripId,
    start_stop_idx: usize,
    arrival_time: ServiceTime,
    departure_time: ServiceTime,
}

struct TransitView<'a> {
    transit_info: &'a TransitInfo,
    date: ServiceDate,
}

struct TripPatternView<'a> {
    transit_info: &'a TransitInfo,
    trip_pattern: &'a TripPattern,
    date: ServiceDate,
}

impl<'a> TransitView<'a> {
    fn stops(&self) -> impl Iterator<Item = &Stop> {
        self.transit_info.stops.iter()
    }

    fn trip_patterns(&self) -> impl Iterator<Item = TripPatternView<'a>> {
        self.transit_info
            .trip_patterns
            .iter()
            .map(|p| TripPatternView {
                transit_info: self.transit_info,
                trip_pattern: p,
                date: self.date,
            })
    }

    fn foot_paths(&self) -> impl Iterator<Item = &Vec<FootPath>> {
        self.transit_info.foot_paths.iter()
    }

    fn trip_by_id(&self, trip_id: &TripId) -> &Trip {
        &self.transit_info.trips[trip_id.0]
    }

    fn depature_from(&self, trip_id: &TripId, stop_idx: usize) -> ServiceTime {
        self.transit_info.trips[trip_id.0].stop_times[stop_idx].departure_time
    }

    fn arrival_to(&self, trip_id: &TripId, stop_idx: usize) -> ServiceTime {
        self.transit_info.trips[trip_id.0].stop_times[stop_idx].arrival_time
    }
}

impl<'a> TripPatternView<'a> {
    fn stops(&self) -> impl Iterator<Item = &StopId> {
        self.trip_pattern.stops.iter()
    }

    fn trips(&self) -> impl Iterator<Item = &TripId> {
        self.trip_pattern
            .trips
            .iter()
            .filter(|trip_id| self.transit_info.trip_active(**trip_id, self.date))
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct Step {
    trip_id: TripId,
    start_stop_idx: usize,
    end_stop_idx: usize,
    walked_distance: f32,
}

struct Search {
    earliest_arrival: Vec<Option<ServiceTime>>,
    parent: Vec<Vec<Option<Step>>>,
}

fn init_earliest_trip(transit_view: &TransitView, search: &Search) -> Vec<Vec<Option<TravelTrip>>> {
    let mut earliest_trip: Vec<Vec<Option<TravelTrip>>> = transit_view
        .trip_patterns()
        .map(|p| p.stops().map(|_| Option::None).collect())
        .collect();

    for (pattern_id, pattern) in transit_view.trip_patterns().enumerate() {
        for (i, stop_id) in pattern.stops().enumerate() {
            if let Some(earliest_arrival_time) = search.earliest_arrival[stop_id.0] {
                for trip_id in pattern.trips() {
                    let trip_stop_time = &transit_view.trip_by_id(trip_id).stop_times[i];
                    if trip_stop_time.departure_time > earliest_arrival_time {
                        earliest_trip[pattern_id][i] = Some(TravelTrip {
                            trip_id: *trip_id,
                            start_stop_idx: i,
                            arrival_time: trip_stop_time.arrival_time,
                            departure_time: trip_stop_time.departure_time,
                        });
                        break;
                    }
                }
            }
        }
    }

    earliest_trip
}

const WALKING_SPEED_METERS_PER_SECOND: f32 = 0.5;

fn round(transit_view: &TransitView, search: &mut Search) {
    let mut earliest_arrival = search.earliest_arrival.clone();
    let earliest_trip = init_earliest_trip(transit_view, search);
    let mut parent: Vec<Option<Step>> = transit_view.stops().map(|_| Option::None).collect();

    for (pattern_idx, trip_pattern) in transit_view.trip_patterns().enumerate() {
        let mut active_trip: Option<TravelTrip> = None;

        for (stop_idx, stop_id) in trip_pattern.stops().enumerate() {
            if let Some(trip) = &earliest_trip[pattern_idx][stop_idx] {
                if let Some(active_trip_stop) = active_trip {
                    if trip.departure_time
                        < transit_view.depature_from(&active_trip_stop.trip_id, stop_idx)
                    {
                        active_trip = Some(*trip);
                    }
                } else {
                    active_trip = Some(*trip);
                }
            }

            if let Some(trip) = active_trip {
                let arrival_time = transit_view.arrival_to(&trip.trip_id, stop_idx);

                if let Some(current_best_time) = earliest_arrival[stop_id.0] {
                    if arrival_time < current_best_time {
                        earliest_arrival[stop_id.0] = Some(arrival_time);
                        parent[stop_id.0] = Some(Step {
                            trip_id: trip.trip_id,
                            start_stop_idx: trip.start_stop_idx,
                            end_stop_idx: stop_idx,
                            walked_distance: 0.0,
                        });
                    }
                } else {
                    earliest_arrival[stop_id.0] = Some(arrival_time);
                    parent[stop_id.0] = Some(Step {
                        trip_id: trip.trip_id,
                        start_stop_idx: trip.start_stop_idx,
                        end_stop_idx: stop_idx,
                        walked_distance: 0.0,
                    });
                }
            }
        }
    }

    let mut earliest_arrival_walk = earliest_arrival.clone();

    for (stop_id, foot_paths) in transit_view.foot_paths().enumerate() {
        if let Some(arrival_time) = earliest_arrival[stop_id] {
            for path in foot_paths {
                let duration =
                    (path.distance_meters / WALKING_SPEED_METERS_PER_SECOND).round() as i32;
                if duration < 0 {
                    println!("very sus")
                }
                let duration = duration as u32;
                if let Some(end_arrival_time) = earliest_arrival[path.to.0] {
                    if arrival_time.0 + duration < end_arrival_time.0 {
                        earliest_arrival_walk[path.to.0] =
                            Some(ServiceTime(arrival_time.0 + duration));
                        parent[path.to.0] = parent[stop_id].map(|mut p| {
                            p.walked_distance = path.distance_meters;
                            p
                        });
                    }
                } else {
                    earliest_arrival_walk[path.to.0] = Some(ServiceTime(arrival_time.0 + duration));
                    parent[path.to.0] = parent[stop_id].map(|mut p| {
                        p.walked_distance = path.distance_meters;
                        p
                    });
                }
            }
        }
    }

    search.earliest_arrival = earliest_arrival_walk;
    search.parent.push(parent);
}

#[derive(Debug, Serialize)]
pub struct Journey {
    arrival_time: ServiceTime,
    steps: Vec<Step>,
}

fn reconstruct_journey(
    transit_info: &TransitInfo,
    search: &Search,
    end_stop: StopId,
    rounds: usize,
) -> Option<Journey> {
    let mut steps = Vec::new();

    let mut stop = end_stop;

    let arrival_time = search.earliest_arrival[stop.0]?;

    for round in (0..=rounds).rev() {
        if let Some(step) = &search.parent[round][stop.0] {
            steps.push(*step);
            let trip = &transit_info.trips[step.trip_id.0];
            stop = trip.stop_times[step.start_stop_idx].stop_id;
        }
    }
    steps.reverse();

    println!("Steps: ");
    for step in &steps {
        let trip = &transit_info.trips[step.trip_id.0];
        let route = &transit_info.routes[trip.route_id.0];

        let start_stop_time = &trip.stop_times[step.start_stop_idx];
        let end_stop_time = &trip.stop_times[step.end_stop_idx];

        let start_stop = &transit_info.stops[start_stop_time.stop_id.0];
        let end_stop = &transit_info.stops[end_stop_time.stop_id.0];
        println!(
            "{:?} : {:?} ({:?}) [{:?}] -> {:?} ({:?}) [{:?}] + walk {:?}m",
            route.short_name,
            start_stop.name,
            start_stop.code,
            NaiveTime::from_num_seconds_from_midnight_opt(start_stop_time.departure_time.0, 0),
            end_stop.name,
            end_stop.code,
            NaiveTime::from_num_seconds_from_midnight_opt(end_stop_time.arrival_time.0, 0),
            step.walked_distance
        );
    }

    Some(Journey {
        steps: steps,
        arrival_time: arrival_time,
    })
}

pub fn search_journeys(
    transit_info: &TransitInfo,
    start: MetaStopId,
    end: MetaStopId,
    departure_time: DateTime<FixedOffset>,
) -> Vec<Journey> {
    let local = departure_time.naive_local();
    let time = ServiceTime(local.time().num_seconds_from_midnight());
    let date = ServiceDate(local.date());

    println!("Starting search {:?} {:?}", local.time(), local.date());

    let mut earliest_arrival: Vec<Option<ServiceTime>> =
        transit_info.stops.iter().map(|_| Option::None).collect();

    let parent = vec![transit_info.stops.iter().map(|_| Option::None).collect()];

    for start_stop in &transit_info.meta_stops[start.0].stops {
        earliest_arrival[start_stop.0] = Some(time);
    }

    let transit_view = TransitView { transit_info, date };

    let mut search = Search {
        earliest_arrival,
        parent: parent,
    };

    let mut journeys = Vec::new();
    for i in 0..ROUNDS as usize {
        round(&transit_view, &mut search);

        println!("RESULT AFTER ROUND {:?}", i);

        for end_stop in &transit_info.meta_stops[end.0].stops {
            let ea = search.earliest_arrival[end_stop.0];
            println!(
                "{:?}",
                ea.map(|s| chrono::NaiveTime::from_num_seconds_from_midnight_opt(s.0, 0))
            );
            if let Some(journey ) = reconstruct_journey(&transit_info, &search, *end_stop, i + 1) {
                journeys.push(journey);
            }
        }
    }

    journeys
}
