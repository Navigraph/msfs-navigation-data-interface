use sentry::capture_message;
use serde::Serialize;

use crate::database::utils::{Coordinates, Degrees, NauticalMiles};

use super::{
    enums::{ControlledAirspaceType, RestrictiveAirspaceType, TurnDirection},
    sql,
};

#[derive(Serialize, Debug)]
pub struct Arc {
    pub origin: Coordinates,
    pub distance: NauticalMiles,
    pub bearing: Degrees,
    pub direction: TurnDirection,
}

#[derive(Serialize, Debug, Copy, Clone)]
pub enum PathType {
    #[serde(rename = "C")]
    Circle,
    #[serde(rename = "G")]
    GreatCircle,
    #[serde(rename = "R")]
    RhumbLine,
    #[serde(rename = "A")]
    Arc,
    #[serde(rename = "U")]
    Unknown,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Debug)]
pub struct Path {
    pub location: Coordinates,
    pub arc: Option<Arc>,
    pub path_type: PathType,
}

impl Path {
    fn from_data(
        latitude: Option<f64>,
        longitude: Option<f64>,
        arc_latitude: Option<f64>,
        arc_longitude: Option<f64>,
        arc_distance: Option<f64>,
        arc_bearing: Option<f64>,
        boundary_via: String,
    ) -> (Self, bool) {
        let boundary_char = boundary_via.chars().nth(0).unwrap();

        let mut error_in_row = false;

        let path = match boundary_char {
            'C' => Self {
                location: Coordinates {
                    lat: arc_latitude.unwrap_or_else(|| {
                        error_in_row = true;
                        0.
                    }),
                    long: arc_longitude.unwrap_or_else(|| {
                        error_in_row = true;
                        0.
                    }),
                },
                arc: None,
                path_type: PathType::Circle,
            },
            'G' | 'H' => Self {
                location: Coordinates {
                    lat: latitude.unwrap_or_else(|| {
                        error_in_row = true;
                        0.
                    }),
                    long: longitude.unwrap_or_else(|| {
                        error_in_row = true;
                        0.
                    }),
                },
                arc: None,
                path_type: match boundary_char {
                    'G' => PathType::GreatCircle,
                    _ => PathType::RhumbLine,
                },
            },
            'L' | 'R' => Self {
                location: Coordinates {
                    lat: latitude.unwrap_or_else(|| {
                        error_in_row = true;
                        0.
                    }),
                    long: longitude.unwrap_or_else(|| {
                        error_in_row = true;
                        0.
                    }),
                },
                arc: Some(Arc {
                    origin: Coordinates {
                        lat: arc_latitude.unwrap_or_else(|| {
                            error_in_row = true;
                            0.
                        }),
                        long: arc_longitude.unwrap_or_else(|| {
                            error_in_row = true;
                            0.
                        }),
                    },
                    distance: arc_distance.unwrap_or_else(|| {
                        error_in_row = true;
                        0.
                    }),
                    bearing: arc_bearing.unwrap_or_else(|| {
                        error_in_row = true;
                        0.
                    }),
                    direction: match boundary_char {
                        'R' => TurnDirection::Right,
                        _ => TurnDirection::Left,
                    },
                }),
                path_type: PathType::Arc,
            },
            _ => {
                error_in_row = true;
                Self {
                    location: Coordinates {
                        lat: 0.0,
                        long: 0.0,
                    },
                    arc: None,
                    path_type: PathType::Unknown,
                }
            }
        };

        (path, error_in_row)
    }
}

#[derive(Serialize, Debug)]
pub struct ControlledAirspace {
    pub area_code: String,
    pub icao_code: String,
    pub airspace_center: String,
    pub name: String,
    pub airspace_type: ControlledAirspaceType,
    pub boundary_paths: Vec<Path>,
}

#[derive(Serialize, Debug)]
pub struct RestrictiveAirspace {
    pub area_code: String,
    pub icao_code: String,
    pub designation: String,
    pub name: String,
    pub airspace_type: RestrictiveAirspaceType,
    pub boundary_paths: Vec<Path>,
}

pub fn map_controlled_airspaces(data: Vec<sql::ControlledAirspace>) -> Vec<ControlledAirspace> {
    let mut airspace_complete = false;

    let mut error_in_row = false;

    let new_data = data.into_iter().fold(Vec::new(), |mut airspaces, row| {
        if airspaces.is_empty() || airspace_complete {
            let name = row.controlled_airspace_name.clone();

            // Skip areas that go 'outside' of range
            if name.is_none() {
                airspace_complete = false;
                return airspaces;
            }

            airspaces.push(ControlledAirspace {
                area_code: row.area_code.clone(),
                icao_code: row.icao_code.clone(),
                airspace_center: row.airspace_center.clone(),
                name: name.unwrap(),
                airspace_type: row.airspace_type,
                boundary_paths: Vec::new(),
            });

            airspace_complete = false;
        }

        if row.boundary_via.chars().nth(1) == Some('E') {
            airspace_complete = true;
        }

        let target_airspace = airspaces.last_mut().unwrap();

        let (path, error) = Path::from_data(
            row.latitude,
            row.longitude,
            row.arc_origin_latitude,
            row.arc_origin_longitude,
            row.arc_distance,
            row.arc_bearing,
            row.boundary_via,
        );

        error_in_row = error;

        target_airspace.boundary_paths.push(path);

        airspaces
    });

    if error_in_row {
        let error_text = format!(
            "Error found in ControlledAirspace: {}",
            serde_json::to_string(&new_data).unwrap_or_else(|_| {
                let row = &new_data.first();

                match row {
                    Some(row) => {
                        format!(
                            "Error serializing output, {} proedure {}",
                            row.icao_code, row.name
                        )
                    }
                    None => "ControlledAirspace is unknown".to_string(),
                }
            })
        );

        capture_message(&error_text, sentry::Level::Warning);
    }

    new_data
}

pub fn map_restrictive_airspaces(data: Vec<sql::RestrictiveAirspace>) -> Vec<RestrictiveAirspace> {
    let mut airspace_complete = false;

    let mut error_in_row = false;

    let new_data = data.into_iter().fold(Vec::new(), |mut airspaces, row| {
        if airspaces.is_empty() || airspace_complete {
            let name = row.restrictive_airspace_name.clone();

            // Skip areas that go 'outside' of range
            if name.is_none() {
                airspace_complete = false;
                return airspaces;
            }

            airspaces.push(RestrictiveAirspace {
                area_code: row.area_code.clone(),
                icao_code: row.icao_code.clone(),
                designation: row.restrictive_airspace_designation.clone(),
                name: name.unwrap(),
                airspace_type: row.restrictive_type,
                boundary_paths: Vec::new(),
            });

            airspace_complete = false;
        }

        if row.boundary_via.chars().nth(1) == Some('E') {
            airspace_complete = true;
        }

        let target_airspace = airspaces.last_mut().unwrap();

        let (path, error) = Path::from_data(
            row.latitude,
            row.longitude,
            row.arc_origin_latitude,
            row.arc_origin_longitude,
            row.arc_distance,
            row.arc_bearing,
            row.boundary_via,
        );

        error_in_row = error;

        target_airspace.boundary_paths.push(path);

        airspaces
    });

    if error_in_row {
        let error_text = format!(
            "Error found in ControlledAirspace: {}",
            serde_json::to_string(&new_data).unwrap_or_else(|_| {
                let row = &new_data.first();

                match row {
                    Some(row) => {
                        format!(
                            "Error serializing output, {} proedure {}",
                            row.icao_code, row.name
                        )
                    }
                    None => "ControlledAirspace is unknown".to_string(),
                }
            })
        );

        capture_message(&error_text, sentry::Level::Warning);
    }

    new_data
}
