use serde::{Deserialize, Serialize};

pub type NauticalMiles = f64;
pub type Degrees = f64;
pub type Radians = f64;
pub type Feet = f64;
pub type Knots = f64;
pub type Minutes = f64;
pub type KiloHertz = f64;
pub type MegaHertz = f64;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Coordinates {
    pub lat: Degrees,
    pub long: Degrees,
}

const EARTH_RADIUS: NauticalMiles = 3443.92;
const MIN_LAT: Degrees = -90.0;
const MAX_LAT: Degrees = 90.0;
const MIN_LONG: Degrees = -180.0;
const MAX_LONG: Degrees = 180.0;

/// These
impl Coordinates {
    /// Returns the Southwest and Northeast corner of a box around coordinates with a minimum `distance`
    pub fn distance_bounds(&self, distance: NauticalMiles) -> (Coordinates, Coordinates) {
        let radial_distance: Radians = distance / EARTH_RADIUS;

        let mut low_lat = self.lat - radial_distance.to_degrees();
        let mut high_lat = self.lat + radial_distance.to_degrees();

        let mut low_long;
        let mut high_long;

        if low_lat > MIN_LAT && high_lat < MAX_LAT {
            let delta_long = (radial_distance.sin() / self.lat.to_radians().cos())
                .asin()
                .to_degrees();
            low_long = self.long - delta_long;

            if low_long < MIN_LONG {
                low_long += 360.0;
            }

            high_long = self.long + delta_long;

            if high_long > MAX_LONG {
                high_long -= 360.0;
            }
        } else {
            low_lat = low_lat.max(MIN_LAT);
            high_lat = high_lat.max(MAX_LAT);

            low_long = MIN_LONG;
            high_long = MIN_LONG;
        }

        (
            Coordinates {
                lat: low_lat,
                long: low_long,
            },
            Coordinates {
                lat: high_lat,
                long: high_long,
            },
        )
    }

    pub fn distance_to(&self, other: &Coordinates) -> NauticalMiles {
        let delta_lat: Radians = (other.lat - self.lat).to_radians();
        let delta_long: Degrees = (other.long - self.long).to_radians();

        let a =
            (delta_lat / 2.0).sin().powi(2) + self.lat.to_radians().cos().powi(2) * (delta_long / 2.0).sin().powi(2);

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS * c
    }
}
