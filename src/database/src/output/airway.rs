use serde::Serialize;

use super::fix::Fix;
use crate::{
    enums::{AirwayDirection, AirwayLevel, AirwayRouteType},
    sql_structs,
    v2::{self},
};

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
pub struct Airway {
    /// Identifier of the airway (not unique), such as `A1` or `Y175`
    pub ident: String,
    /// A list of fixes which make up the airway
    pub fixes: Vec<Fix>,
    /// The type of airway
    pub route_type: AirwayRouteType,
    /// Represents the altitude band which this aircraft is part of
    ///
    /// Can be:
    /// - High
    /// - Low
    /// - Both
    pub level: AirwayLevel,
    /// Represents a directional restriction on this airway
    ///
    /// If it is `AirwayDirection::Forward`, this airway must only be flown in the order that fixes are listed in the
    /// `fixes` field.
    ///
    /// If it is `AirwayDirection::Backward`, this airway must only be flown in the reverse order
    /// that fixes are listed in the `fixes` field
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
pub(crate) fn map_airways(data: Vec<sql_structs::EnrouteAirways>) -> Vec<Airway> {
    let mut airway_complete = false;
    data.into_iter()
        .fold(Vec::new(), |mut airways, airway_row| {
            if airways.is_empty() || airway_complete {
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

            target_airway.fixes.push(Fix::from_row_data(
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

// TODO: Implement error propigation, need to rewrite logic (maybe out of scope)
pub(crate) fn map_airways_v2(data: Vec<v2::sql_structs::EnrouteAirways>) -> Vec<Airway> {
    let mut airway_complete = false;
    data.into_iter()
        .fold(Vec::new(), |mut airways, airway_row| {
            if airways.is_empty() || airway_complete {
                airways.push(Airway {
                    ident: airway_row.route_identifier.unwrap_or("ERROR".to_string()),
                    fixes: Vec::new(),
                    route_type: airway_row
                        .route_type
                        .unwrap_or(AirwayRouteType::UndesignatedAtsRoute),
                    level: airway_row.flightlevel.unwrap_or(AirwayLevel::Both),
                    direction: airway_row.direction_restriction,
                });

                airway_complete = false;
            }

            let target_airway = airways.last_mut().unwrap();

            target_airway.fixes.push(Fix::from_row_data_v2(
                airway_row.waypoint_latitude.unwrap_or(0.),
                airway_row.waypoint_longitude.unwrap_or(0.),
                airway_row.waypoint_identifier.unwrap_or("NULL".to_string()),
                airway_row.icao_code.unwrap_or("NULL".to_string()),
                None,
                airway_row.waypoint_ref_table,
            ));

            if airway_row
                .waypoint_description_code
                .unwrap_or("   ".to_string())
                .chars()
                .nth(1)
                == Some('E')
            {
                airway_complete = true;
            }

            airways
        })
}
