/// Helper structures to view the transit network at a particular (search) dat/

use crate::transit::model::{
    FootPath, Stop, StopId, TransitInfo, Trip, TripId, TripPattern,
};
use crate::transit::gtfs::{ServiceDate, ServiceTime};

pub struct TransitView<'a> {
    transit_info: &'a TransitInfo,
    date: ServiceDate,
}

pub struct TripPatternView<'a> {
    transit_info: &'a TransitInfo,
    trip_pattern: &'a TripPattern,
    date: ServiceDate,
}

impl<'a> TransitView<'a> {
    pub fn new(transit_info: &'a TransitInfo, date: ServiceDate) -> Self {
        Self { transit_info, date }
    }
    pub fn stops(&self) -> impl Iterator<Item = &Stop> {
        self.transit_info.stops.iter()
    }

    pub fn trip_patterns(&self) -> impl Iterator<Item = TripPatternView<'a>> {
        self.transit_info
            .trip_patterns
            .iter()
            .map(|p| TripPatternView {
                transit_info: self.transit_info,
                trip_pattern: p,
                date: self.date,
            })
    }

    pub fn foot_paths(&self) -> impl Iterator<Item = &Vec<FootPath>> {
        self.transit_info.foot_paths.iter()
    }

    pub fn trip_by_id(&self, trip_id: TripId) -> &Trip {
        &self.transit_info.get_trip(trip_id)
    }

    pub fn depature_from(&self, trip_id: TripId, stop_idx: usize) -> ServiceTime {
        self.transit_info.get_trip(trip_id).stop_times[stop_idx].departure_time
    }

    pub fn arrival_to(&self, trip_id: TripId, stop_idx: usize) -> ServiceTime {
        self.transit_info.get_trip(trip_id).stop_times[stop_idx].arrival_time
    }
}

impl<'a> TripPatternView<'a> {
    pub fn stops(&self) -> impl Iterator<Item = &StopId> {
        self.trip_pattern.stops.iter()
    }

    // This is the actual date-dependent part
    pub fn trips(&self) -> impl Iterator<Item = &TripId> {
        self.trip_pattern
            .trips
            .iter()
            .filter(|trip_id| self.transit_info.trip_active(**trip_id, self.date))
    }

    // Account for trips starting before midnight and ending after midnight
    pub fn next_day_trips(&self) -> impl Iterator<Item = &TripId> {
        self.trip_pattern
            .trips
            .iter()
            .filter(|trip_id| self.transit_info.trip_active(**trip_id, self.date.next_day()))
    }
}