use serde::Serialize;

use crate::database::utils::{feet_to_meters, Coordinates, Degrees, Meters};

use super::{enums::ApproachTypeIdentifier, sql};

#[derive(Serialize)]
pub struct PathPoint {
    pub area_code: String,
    pub airport_ident: String,
    pub icao_code: String,
    /// The identifier of the approach this path point is used in, such as `R36RY` or `R20`
    pub approach_ident: String,
    /// The identifier of the runway this path point is used with, such as `RW02` or `RW36L`
    pub runway_ident: String,
    pub ident: String,
    pub landing_threshold_location: Coordinates,
    pub ltp_ellipsoid_height: Meters,
    pub glidepath_angle: Degrees,
    pub flightpath_alignment_location: Coordinates,
    pub course_width: Meters,
    pub length_offset: Meters,
    pub path_point_tch: Meters,
    pub horizontal_alert_limit: Meters,
    pub vertical_alert_limit: Meters,
    pub gnss_channel_number: f64,
    pub approach_type: ApproachTypeIdentifier,
}

impl From<sql::Pathpoints> for PathPoint {
    fn from(row: sql::Pathpoints) -> Self {
        Self {
            area_code: row.area_code,
            airport_ident: row.airport_identifier,
            icao_code: row.airport_icao_code,
            approach_ident: row.approach_procedure_ident,
            runway_ident: row.runway_identifier,
            ident: row.reference_path_identifier,
            landing_threshold_location: Coordinates {
                lat: row.landing_threshold_point_latitude,
                long: row.landing_threshold_point_longitude,
            },
            ltp_ellipsoid_height: row.ltp_ellipsoid_height,
            glidepath_angle: row.glide_path_angle,
            flightpath_alignment_location: Coordinates {
                lat: row.flight_path_alignment_point_latitude,
                long: row.flight_path_alignment_point_longitude,
            },
            course_width: row.course_width_at_threshold,
            length_offset: row.length_offset.unwrap_or_default(),
            path_point_tch: if row.tch_units_indicator == *"F" {
                feet_to_meters(row.path_point_tch)
            } else {
                row.path_point_tch
            },
            horizontal_alert_limit: row.hal,
            vertical_alert_limit: row.val,
            gnss_channel_number: row.gnss_channel_number,
            approach_type: row.approach_type_identifier,
        }
    }
}
