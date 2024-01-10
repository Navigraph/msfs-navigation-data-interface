use serde::Serialize;

use super::fix::{map_fix, Fix};
use crate::query::{
    enums::{AirwayDirection, AirwayLevel, AirwayRouteType},
    sql_structs,
};

#[derive(Serialize)]
pub struct Airway {
    pub ident: String,
    pub fixes: Vec<Fix>,
    pub route_type: AirwayRouteType,
    pub level: AirwayLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<AirwayDirection>,
}

/// Takes a vector of EnrouteAirway rows from the database and collects them into Airway structs
///
/// This function requires complete airway data, so it is expected that the provided data comes from a query by
/// route_identifier, to ensure that full airways will be present.
///
/// When querying airways by location always be sure to query all airways with route_identifiers which appear within the
/// query area. This is icao_code can change along one airway so it should not be used to group airways. There is no way
/// to way to identify distinct airways other than iterating through them to find an end of airway flag
pub fn map_airways(data: Vec<sql_structs::EnrouteAirways>) -> Vec<Airway> {
    let mut airway_complete = false;
    data.into_iter().fold(Vec::new(), |mut airways, airway_row| {
        if airways.len() == 0 || airway_complete {
            airways.push(Airway {
                ident: airway_row.route_identifier,
                fixes: Vec::new(),
                route_type: airway_row.route_type,
                level: airway_row.flightlevel,
                direction: airway_row.direction_restriction,
            });

            airway_complete = false;
        }

        let target_airway = airways.last_mut().unwrap();

        target_airway.fixes.push(map_fix(
            airway_row.waypoint_latitude,
            airway_row.waypoint_longitude,
            airway_row.id,
        ));

        if airway_row.waypoint_description_code.chars().nth(1) == Some('E') {
            airway_complete = true;
        }

        airways
    })
}
