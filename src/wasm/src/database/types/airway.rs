use sentry::capture_message;
use serde::Serialize;

use super::{
    enums::{AirwayDirection, AirwayLevel, AirwayRouteType},
    fix::Fix,
    sql,
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
pub fn map_airways(data: Vec<sql::EnrouteAirways>) -> Vec<Airway> {
    let mut airway_complete = false;

    let mut error_in_row = false;

    let new_data = data
        .into_iter()
        .fold(Vec::new(), |mut airways, airway_row| {
            if airways.is_empty() || airway_complete {
                airways.push(Airway {
                    ident: airway_row.route_identifier.unwrap_or_else(|| {
                        error_in_row = true;
                        "ERROR".to_string()
                    }),
                    fixes: Vec::new(),
                    route_type: airway_row.route_type.unwrap_or(AirwayRouteType::Unknown),
                    level: airway_row.flightlevel.unwrap_or(AirwayLevel::Unknown),
                    direction: airway_row.direction_restriction,
                });

                airway_complete = false;
            }

            let target_airway = airways.last_mut().unwrap();

            target_airway.fixes.push(Fix::from_row_data(
                airway_row.waypoint_latitude.unwrap_or_else(|| {
                    error_in_row = true;
                    0.
                }),
                airway_row.waypoint_longitude.unwrap_or_else(|| {
                    error_in_row = true;
                    0.
                }),
                airway_row.waypoint_identifier.unwrap_or_else(|| {
                    error_in_row = true;
                    "UNKN".to_string()
                }),
                airway_row.icao_code.unwrap_or_else(|| {
                    error_in_row = true;
                    "UNKN".to_string()
                }),
                None,
                airway_row.waypoint_ref_table,
                airway_row.waypoint_description_code.clone(),
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
        });

    if error_in_row {
        let error_text = format!(
            "Error found in ControlledAirspace: {}",
            serde_json::to_string(&new_data).unwrap_or_else(|_| {
                let row = &new_data.first();

                match row {
                    Some(row) => {
                        format!("Error serializing output, airway {}", row.ident)
                    }
                    None => "ControlledAirspace is unknown".to_string(),
                }
            })
        );

        capture_message(&error_text, sentry::Level::Warning);
    }

    new_data
}
