/// Representation of raw GTFS data as rust structures
use chrono::{Datelike, NaiveDate, Weekday};
use csv::Reader;
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use std::path::Path;

use crate::transit::model::Float;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Hash)]
pub struct ServiceTime(pub u32);

impl ServiceTime {
    pub fn plus_seconds(&self, seconds: u32) -> ServiceTime {
        ServiceTime(self.0 + seconds)
    }
}

impl<'de> Deserialize<'de> for ServiceTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let mut parts = s.split(':');

        let h: u32 = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing hour"))?
            .parse()
            .map_err(serde::de::Error::custom)?;

        let m: u32 = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing minute"))?
            .parse()
            .map_err(serde::de::Error::custom)?;

        let sec: u32 = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing second"))?
            .parse()
            .map_err(serde::de::Error::custom)?;

        Ok(ServiceTime(h * 3600 + m * 60 + sec))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Hash)]
pub struct ServiceDate(pub NaiveDate);

impl ServiceDate {
    pub fn weekday(&self) -> Weekday {
        self.0.weekday()
    }

    pub fn next_day(&self) -> ServiceDate {
        ServiceDate(self.0.checked_add_days(chrono::Days::new(1)).unwrap())
    }
}

impl<'de> Deserialize<'de> for ServiceDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let date = NaiveDate::parse_from_str(&s, "%Y%m%d").map_err(serde::de::Error::custom)?;
        Ok(ServiceDate(date))
    }
}

#[derive(Debug)]
pub enum GtfsServiceAvailability {
    Available,
    Unavailable,
}

#[derive(Debug)]
pub enum GtfsDateExceptionType {
    ServiceAdded,
    ServiceRemoved,
}

impl<'de> Deserialize<'de> for GtfsServiceAvailability {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s == "0" {
            Ok(Self::Unavailable)
        } else if s == "1" {
            Ok(Self::Available)
        } else {
            Err(serde::de::Error::custom("Invalid service availability"))
        }
    }
}

impl<'de> Deserialize<'de> for GtfsDateExceptionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s == "1" {
            Ok(Self::ServiceAdded)
        } else if s == "2" {
            Ok(Self::ServiceAdded)
        } else {
            Err(serde::de::Error::custom("Invalid exception type"))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GtfsStop {
    pub stop_id: String,
    pub stop_code: String,
    pub stop_name: String,
    pub stop_lat: Float,
    pub stop_lon: Float,
}

#[derive(Debug, Deserialize)]
pub struct GtfsShapeEntry {
    pub shape_id: String,
    pub shape_pt_lat: Float,
    pub shape_pt_lon: Float,
    pub shape_pt_sequence: u32,
}

#[derive(Debug, Deserialize)]
pub struct GtfsRoute {
    pub route_id: String,
    pub route_short_name: String,
    pub route_type: u32,
}

#[derive(Debug, Deserialize)]
pub struct GtfsTrip {
    pub trip_id: String,
    pub route_id: String,
    pub shape_id: String,
    pub service_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GtfsStopTime {
    pub trip_id: String,
    pub stop_id: String,
    pub arrival_time: ServiceTime,
    pub departure_time: ServiceTime,
    pub stop_sequence: u32,
}

#[derive(Debug, Deserialize)]
pub struct GtfsCalendarEntry {
    pub service_id: String,
    pub monday: GtfsServiceAvailability,
    pub tuesday: GtfsServiceAvailability,
    pub wednesday: GtfsServiceAvailability,
    pub thursday: GtfsServiceAvailability,
    pub friday: GtfsServiceAvailability,
    pub saturday: GtfsServiceAvailability,
    pub sunday: GtfsServiceAvailability,
    pub start_date: ServiceDate,
    pub end_date: ServiceDate,
}

#[derive(Debug, Deserialize)]
pub struct GtfsCalendarDateEntry {
    pub service_id: String,
    pub date: ServiceDate,
    pub exception_type: GtfsDateExceptionType,
}

pub struct Gtfs {
    pub stops: Vec<GtfsStop>,
    pub shapes: Vec<GtfsShapeEntry>,
    pub routes: Vec<GtfsRoute>,
    pub trips: Vec<GtfsTrip>,
    pub stop_times: Vec<GtfsStopTime>,
    pub calendar: Vec<GtfsCalendarEntry>,
    pub calendar_dates: Vec<GtfsCalendarDateEntry>,
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
    eprintln!("Loading GTFS from {}", path.display());

    let gtfs = Gtfs {
        stops: load(&path.join("stops.txt")),
        shapes: load(&path.join("shapes.txt")),
        routes: load(&path.join("routes.txt")),
        trips: load(&path.join("trips.txt")),
        stop_times: load(&path.join("stop_times.txt")),
        calendar: load(&path.join("calendar.txt")),
        calendar_dates: load(&path.join("calendar_dates.txt")),
    };

    eprintln!("GTFS loaded");

    gtfs
}
