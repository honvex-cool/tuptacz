use serde::Serialize;

#[derive(Default, Clone, Copy, Serialize)]
pub struct Coords {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coords {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }
}

impl From<(f64, f64)> for Coords {
    fn from((latitude, longitude): (f64, f64)) -> Self {
        Self::new(latitude, longitude)
    }
}
