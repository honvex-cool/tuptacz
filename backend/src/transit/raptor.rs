use std::collections::HashSet;
use std::hash::Hash;

// Core RAPTOR algorithm implementation
use chrono::{DateTime, FixedOffset, NaiveTime, Timelike};
use serde::Serialize;

use crate::transit::gtfs::{ServiceDate, ServiceTime};
use crate::transit::model::{Float, MetaStopId, StopId, TransitInfo, TripId};

use crate::transit::transit_view::TransitView;

const MAX_CHANGES: u32 = 5;

#[derive(Copy, Clone)]
struct TravelTrip {
    trip_id: TripId,
    start_stop_idx: usize,
    arrival_time: ServiceTime,
    departure_time: ServiceTime,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub struct Leg {
    pub trip_id: TripId,
    pub start_stop_idx: usize,
    pub end_stop_idx: usize,
    pub walked_distance: u32,
}

struct Search {
    earliest_arrival: Vec<Option<ServiceTime>>,
    parent: Vec<Vec<Option<Leg>>>,
}

// On top of slow walk speed we add two minutes of buffer for delays
const TRANSFER_DURATION_SECONDS: u32 = 120;

fn init_earliest_trip(transit_view: &TransitView, search: &Search) -> Vec<Vec<Option<TravelTrip>>> {
    let mut earliest_trip: Vec<Vec<Option<TravelTrip>>> = transit_view
        .trip_patterns()
        .map(|p| p.stops().map(|_| Option::None).collect())
        .collect();

    for (pattern_id, pattern) in transit_view.trip_patterns().enumerate() {
        for (stop_idx, stop_id) in pattern.stops().enumerate() {
            if let Some(earliest_arrival_time) = search.earliest_arrival[stop_id.0] {
                for trip_id in pattern.trips() {
                    let trip_stop_time = &transit_view.trip_by_id(*trip_id).stop_times[stop_idx];
                    if trip_stop_time.departure_time > earliest_arrival_time.plus_seconds(TRANSFER_DURATION_SECONDS) {
                        earliest_trip[pattern_id][stop_idx] = Some(TravelTrip {
                            trip_id: *trip_id,
                            start_stop_idx: stop_idx,
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

const WALKING_SPEED_METERS_PER_SECOND: Float = 0.5;
const SECONDS_IN_DAY: u32 = 3600 * 24;

fn round(transit_view: &TransitView, search: &mut Search) {
    let mut earliest_arrival = search.earliest_arrival.clone();
    let earliest_trip = init_earliest_trip(transit_view, search);
    let mut parent: Vec<Option<Leg>> = transit_view.stops().map(|_| Option::None).collect();

    for (pattern_idx, trip_pattern) in transit_view.trip_patterns().enumerate() {
        let mut active_trip: Option<TravelTrip> = None;

        for (stop_idx, stop_id) in trip_pattern.stops().enumerate() {
            // Check if we can catch an earlier trip of the same pattern at currently processed stop
            if let Some(trip) = &earliest_trip[pattern_idx][stop_idx] {
                if let Some(active_trip_stop) = active_trip {
                    if trip.departure_time
                        < transit_view.depature_from(active_trip_stop.trip_id, stop_idx)
                    {
                        active_trip = Some(*trip);
                    }
                } else {
                    active_trip = Some(*trip);
                }
            }

            if let Some(trip) = active_trip {
                let arrival_time = transit_view.arrival_to(trip.trip_id, stop_idx);

                if let Some(current_best_time) = earliest_arrival[stop_id.0] {
                    if arrival_time < current_best_time {
                        earliest_arrival[stop_id.0] = Some(arrival_time);
                        parent[stop_id.0] = Some(Leg {
                            trip_id: trip.trip_id,
                            start_stop_idx: trip.start_stop_idx,
                            end_stop_idx: stop_idx,
                            walked_distance: 0,
                        });
                    }
                } else {
                    earliest_arrival[stop_id.0] = Some(arrival_time);
                    parent[stop_id.0] = Some(Leg {
                        trip_id: trip.trip_id,
                        start_stop_idx: trip.start_stop_idx,
                        end_stop_idx: stop_idx,
                        walked_distance: 0,
                    });
                }
            }
        }
    }


    for (stop_id, foot_paths) in transit_view.foot_paths().enumerate() {
        if let Some(arrival_time) = earliest_arrival[stop_id] {
            for path in foot_paths {
                let duration =
                    (path.distance_meters / WALKING_SPEED_METERS_PER_SECOND).round() as u32;
                if let Some(end_arrival_time) = earliest_arrival[path.to.0] {
                    if arrival_time.0 + duration < end_arrival_time.0 {
                        earliest_arrival[path.to.0] =
                            Some(ServiceTime(arrival_time.0 + duration));
                        parent[path.to.0] = parent[stop_id].map(|mut p| {
                            p.walked_distance = path.distance_meters.round() as u32;
                            p
                        });
                    }
                } else {
                    earliest_arrival[path.to.0] = Some(ServiceTime(arrival_time.0 + duration));
                    parent[path.to.0] = parent[stop_id].map(|mut p| {
                        p.walked_distance = path.distance_meters.round() as u32;
                        p
                    });
                }
            }
        }
    }

    search.earliest_arrival = earliest_arrival;
    search.parent.push(parent);
}

#[derive(Debug, Serialize, Hash, PartialEq, Eq, Clone)]
pub struct Journey {
    pub arrival_time: ServiceTime,
    pub legs: Vec<Leg>,
}

fn print_journey(transit_info: &TransitInfo, journey: &Journey) {
    println!(
        "Arrival time at {:?} {:?}",
        journey.arrival_time,
        chrono::NaiveTime::from_num_seconds_from_midnight_opt(
            journey.arrival_time.0 % SECONDS_IN_DAY,
            0
        )
    );
    println!("Steps: ");
    for step in &journey.legs {
        let trip = &transit_info.get_trip(step.trip_id);
        let route = &transit_info.routes[trip.route_id.0];

        let start_stop_time = &trip.stop_times[step.start_stop_idx];
        let end_stop_time = &trip.stop_times[step.end_stop_idx];

        let start_stop = &transit_info.stops[start_stop_time.stop_id.0];
        let end_stop = &transit_info.stops[end_stop_time.stop_id.0];
        println!(
            "{:?} : {:?} ({:?}) [{:?} | {:?}] -> {:?} ({:?}) [{:?} | {:?}] + walk {:?}m",
            route.short_name,
            start_stop.name,
            start_stop.code,
            start_stop_time.stop_id,
            NaiveTime::from_num_seconds_from_midnight_opt(
                start_stop_time.departure_time.0 % SECONDS_IN_DAY,
                0
            ),
            end_stop.name,
            end_stop.code,
            end_stop_time.stop_id,
            NaiveTime::from_num_seconds_from_midnight_opt(
                end_stop_time.arrival_time.0 % SECONDS_IN_DAY,
                0
            ),
            step.walked_distance
        );
    }
    println!();
}

fn reconstruct_journey(
    transit_info: &TransitInfo,
    search: &Search,
    end_stop: StopId,
    rounds: usize,
) -> Option<Journey> {
    let mut legs = Vec::new();

    let mut stop = end_stop;

    let arrival_time = search.earliest_arrival[stop.0]?;

    for round in (0..=rounds).rev() {
        if let Some(step) = &search.parent[round][stop.0] {
            legs.push(*step);
            let trip = &transit_info.get_trip(step.trip_id);
            stop = trip.stop_times[step.start_stop_idx].stop_id;
        }
    }
    legs.reverse();

    Some(Journey {
        legs,
        arrival_time: arrival_time,
    })
}


fn dominates(j1: &Journey, j2: &Journey) -> bool {
    let is_not_worse = j1.arrival_time <= j2.arrival_time && j1.legs.len() <= j2.legs.len();
    let earlier = j1.arrival_time < j2.arrival_time;
    let less_legs = j1.legs.len() < j2.legs.len();

    is_not_worse && (earlier || less_legs)
}

fn deduplicate(journeys: Vec<Journey>) -> Vec<Journey> {
    let mut result = HashSet::new();

    for journey in journeys.iter () {
        let mut dominated = false;
        for j in journeys.iter() {
            if dominates(j, &journey) {
                dominated = true;
                break;
            }
        }

        if !dominated {
            result.insert(journey.clone());
        }
    }

    let mut result : Vec<Journey> = result.into_iter().collect();
    result.sort_by(|j1, j2| j1.arrival_time.cmp(&j2.arrival_time));
    result
}

pub fn search_journeys(
    transit_info: &TransitInfo,
    start: MetaStopId,
    end: MetaStopId,
    departure_time: DateTime<FixedOffset>,
) -> Vec<Journey> {
    if start == end {
        return vec![];
    }

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

    let transit_view = TransitView::new(transit_info, date);

    let mut search = Search {
        earliest_arrival,
        parent: parent,
    };

    let mut journeys = Vec::new();
    for i in 0..MAX_CHANGES as usize {
        round(&transit_view, &mut search);

        for end_stop in &transit_info.meta_stops[end.0].stops {
            if let Some(journey) = reconstruct_journey(&transit_info, &search, *end_stop, i + 1) {
                journeys.push(journey);
            }
        }
    }

    let journeys = deduplicate(journeys);
    journeys.iter().for_each(|j| print_journey(transit_info, j));

    journeys
}
