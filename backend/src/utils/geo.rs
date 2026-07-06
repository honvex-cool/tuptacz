use itertools::Itertools;
use num_traits::Float;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LatLng<F> {
    pub latitude: F,
    pub longitude: F,
}

impl<F> LatLng<F>
where
    F: Float,
{
    // https://en.wikipedia.org/wiki/Haversine_formula#Formulation
    pub fn distance_meters(self, other: Self) -> F {
        let earth_radius_meters = F::from(6_371_000.0).unwrap();
        let two = F::from(2.0).unwrap();

        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let lon1 = self.longitude.to_radians();
        let lon2 = other.longitude.to_radians();

        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;

        let lat_sin = (dlat / two).sin();
        let lon_sin = (dlon / two).sin();

        let hav_theta = lat_sin * lat_sin + lon_sin * lon_sin * lat1.cos() * lat2.cos();

        let theta = hav_theta.sqrt().asin() * two;

        theta * earth_radius_meters
    }

    pub fn poly_distance_meters(lat_lngs: &[Self]) -> F {
        lat_lngs
            .iter()
            .tuple_windows()
            .map(|(&first, &second)| Self::distance_meters(first, second))
            .fold(F::zero(), |acc, d| acc + d)
    }
}
