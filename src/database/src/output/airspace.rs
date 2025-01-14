use serde::Serialize;

use crate::{
    enums::{ControlledAirspaceType, RestrictiveAirspaceType, TurnDirection},
    math::{Coordinates, Degrees, NauticalMiles},
    sql_structs,
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
    ) -> Self {
        let boundary_char = boundary_via.chars().nth(0).unwrap();
        match boundary_char {
            'C' => Self {
                location: Coordinates {
                    lat: arc_latitude.unwrap(),
                    long: arc_longitude.unwrap(),
                },
                arc: None,
                path_type: PathType::Circle,
            },
            'G' | 'H' => Self {
                location: Coordinates {
                    lat: latitude.unwrap(),
                    long: longitude.unwrap(),
                },
                arc: None,
                path_type: match boundary_char {
                    'G' => PathType::GreatCircle,
                    _ => PathType::RhumbLine,
                },
            },
            'L' | 'R' => Self {
                location: Coordinates {
                    lat: latitude.unwrap(),
                    long: longitude.unwrap(),
                },
                arc: Some(Arc {
                    origin: Coordinates {
                        lat: arc_latitude.unwrap(),
                        long: arc_longitude.unwrap(),
                    },
                    distance: arc_distance.unwrap(),
                    bearing: arc_bearing.unwrap(),
                    direction: match boundary_char {
                        'R' => TurnDirection::Right,
                        _ => TurnDirection::Left,
                    },
                }),
                path_type: PathType::Arc,
            },
            _ => panic!("Invalid path type"),
        }
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

pub(crate) fn map_controlled_airspaces(
    data: Vec<sql_structs::ControlledAirspace>,
) -> Vec<ControlledAirspace> {
    let mut airspace_complete = false;

    data.into_iter().fold(Vec::new(), |mut airspaces, row| {
        if airspaces.is_empty() || airspace_complete {
            airspaces.push(ControlledAirspace {
                area_code: row.area_code.clone(),
                icao_code: row.icao_code.clone(),
                airspace_center: row.airspace_center.clone(),
                name: row
                    .controlled_airspace_name
                    .clone()
                    .expect("First row of an airspace data must have a name"),
                airspace_type: row.airspace_type,
                boundary_paths: Vec::new(),
            });

            airspace_complete = false;
        }

        if row.boundary_via.chars().nth(1) == Some('E') {
            airspace_complete = true;
        }

        let target_airspace = airspaces.last_mut().unwrap();

        target_airspace.boundary_paths.push(Path::from_data(
            row.latitude,
            row.longitude,
            row.arc_origin_latitude,
            row.arc_origin_longitude,
            row.arc_distance,
            row.arc_bearing,
            row.boundary_via,
        ));

        airspaces
    })
}

pub(crate) fn map_restrictive_airspaces(
    data: Vec<sql_structs::RestrictiveAirspace>,
) -> Vec<RestrictiveAirspace> {
    let mut airspace_complete = false;

    data.into_iter().fold(Vec::new(), |mut airspaces, row| {
        if airspaces.is_empty() || airspace_complete {
            airspaces.push(RestrictiveAirspace {
                area_code: row.area_code.clone(),
                icao_code: row.icao_code.clone(),
                designation: row.restrictive_airspace_designation.clone(),
                name: row
                    .restrictive_airspace_name
                    .clone()
                    .expect("First row of an airspace data must have a name"),
                airspace_type: row.restrictive_type,
                boundary_paths: Vec::new(),
            });

            airspace_complete = false;
        }

        if row.boundary_via.chars().nth(1) == Some('E') {
            airspace_complete = true;
        }

        let target_airspace = airspaces.last_mut().unwrap();

        target_airspace.boundary_paths.push(Path::from_data(
            row.latitude,
            row.longitude,
            row.arc_origin_latitude,
            row.arc_origin_longitude,
            row.arc_distance,
            row.arc_bearing,
            row.boundary_via,
        ));

        airspaces
    })
}
