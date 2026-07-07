/// Conversion from simple RAPTOR output to a rich structure that can be presented to a user.
/// Basically RAPTOR returns a sequence of trip ids and change stop ids but we need to present a bit more info:
/// - line numbers
/// - intermediate stops
/// - locations of the stops
/// - times of the changes and footpaths
/// - geometry of the travel
use serde::Serialize;

use crate::transit::{
    gtfs::ServiceTime,
    model::{LatLng, RouteType, StopId, TransitInfo, TripId},
    raptor::{Journey, Leg},
};

#[derive(Debug, Serialize)]
pub struct RichJourneyStop {
    stop_id: StopId,
    stop_name: String,
    arrival_time: ServiceTime,
    position: LatLng,
}

#[derive(Debug, Serialize)]
pub struct RichJourneyLeg {
    trip_id: TripId,
    route_name: String,
    route_type: RouteType,
    stops: Vec<RichJourneyStop>,
    walked_distance: u32,
}

#[derive(Debug, Serialize)]
pub struct RichJourney {
    legs: Vec<RichJourneyLeg>,
}

fn enrich_leg(transit_info: &TransitInfo, leg: &Leg) -> RichJourneyLeg {
    let trip = transit_info.get_trip(leg.trip_id);
    let trip_stops = transit_info.get_trip_stops(leg.trip_id);
    let route = transit_info.get_route(trip.route_id);

    let mut stops = vec![];
    for i in leg.start_stop_idx..=leg.end_stop_idx {
        let stop_time = &trip.stop_times[i];
        let stop_id = trip_stops[i];
        let stop = transit_info.get_stop(stop_id);
        stops.push(RichJourneyStop {
            stop_id,
            stop_name: stop.name.clone(),
            position: stop.position,
            arrival_time: stop_time.arrival_time,
        });
    }

    RichJourneyLeg {
        trip_id: leg.trip_id,
        route_name: route.short_name.clone(),
        route_type: route.route_type,
        stops,
        walked_distance: leg.walked_distance
    }
}

pub fn enrich_journey(transit_info: &TransitInfo, journey: &Journey) -> RichJourney {
    let legs = journey
        .legs
        .iter()
        .map(|leg| enrich_leg(transit_info, leg))
        .collect();

    RichJourney { legs }
}
