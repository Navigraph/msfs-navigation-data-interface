use serde::Serialize;

use super::fix::Fix;
use crate::{
    enums::{AltitudeDescriptor, LegType, SpeedDescriptor, TurnDirection},
    math::{Degrees, Feet, Knots, Minutes, NauticalMiles},
    sql_structs, v2,
};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone)]
pub struct AltitudeContstraint {
    altitude1: Feet,
    altitude2: Option<Feet>,
    descriptor: AltitudeDescriptor,
}

#[derive(Serialize, Clone)]
pub struct SpeedConstraint {
    value: Knots,
    descriptor: SpeedDescriptor,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone)]
/// Represents a leg as part of a `Departure`, `Arrival`, or `Approach`.
pub struct ProcedureLeg {
    /// Whether or not this termination of this leg should be flown directly over
    overfly: bool,

    /// The type of leg
    leg_type: LegType,

    /// The altitude constraint of this leg.
    ///
    /// This is a required field for any `XA` or `PI` leg
    altitude: Option<AltitudeContstraint>,

    /// The speed constraint of this leg
    speed: Option<SpeedConstraint>,

    /// The vertical angle constraint of this leg
    vertical_angle: Option<Degrees>,

    /// The rnp (required navigational performance) of this leg in nautical miles
    rnp: Option<NauticalMiles>,

    /// The fix that this leg terminates at
    ///
    /// This is a required field for any `XF`, `FX`, `HX` or `PI` leg.
    fix: Option<Fix>,

    /// The fix that is used as the associated radio navigational aid for this leg.
    ///
    /// This is a required field for any `AF`, `CD`, `CF`, `CR`, `FX`, `PI`, `VD`, or `VR` leg
    recommended_navaid: Option<Fix>,

    /// The magnetic bearing from the `recommended_navaid` to the `fix`, or the magnetic radial from the
    /// `recommended_navaid` to intersect with in a `XR` leg
    theta: Option<Degrees>,

    /// The distance in nautical miles from the `recommended_navaid` to the `fix`
    rho: Option<NauticalMiles>,

    /// The magnetic course to be flown for legs which are defined by a course or heading to a termination, or the
    /// radial from the `recomended_navaid` to the expected start location on an `AF` leg
    magnetic_course: Option<Degrees>,

    /// The length of the leg in nautical miles
    length: Option<NauticalMiles>,

    /// The time to be used when flying a hold leg, if any
    length_time: Option<Minutes>,

    /// The constraint on the direction of turn to be used when flying this leg
    turn_direction: Option<TurnDirection>,

    /// The center of the arc to be flown for an `RF` leg
    arc_center_fix: Option<Fix>,

    /// The radius of the arc to be flown for an `RF` leg
    arc_radius: Option<NauticalMiles>,
}

impl From<sql_structs::Procedures> for ProcedureLeg {
    fn from(leg: sql_structs::Procedures) -> Self {
        ProcedureLeg {
            overfly: leg
                .waypoint_description_code
                .map_or(false, |x| x.chars().nth(1) == Some('Y')),
            altitude: leg.altitude1.map(|altitude1| AltitudeContstraint {
                altitude1,
                altitude2: leg.altitude2,
                descriptor: leg
                    .altitude_description
                    .unwrap_or(AltitudeDescriptor::AtAlt1),
            }),
            speed: leg.speed_limit.map(|speed| SpeedConstraint {
                value: speed,
                descriptor: leg
                    .speed_limit_description
                    .unwrap_or(SpeedDescriptor::Mandatory),
            }),
            vertical_angle: leg.vertical_angle,
            rnp: leg.rnp,
            fix: if !leg.id.is_empty() {
                Some(Fix::from_row_data(
                    leg.waypoint_latitude.unwrap(),
                    leg.waypoint_longitude.unwrap(),
                    leg.id,
                ))
            } else {
                None
            },
            recommended_navaid: if !leg.recommanded_id.is_empty() {
                Some(Fix::from_row_data(
                    leg.recommanded_navaid_latitude.unwrap(),
                    leg.recommanded_navaid_longitude.unwrap(),
                    leg.recommanded_id,
                ))
            } else {
                None
            },
            theta: leg.theta,
            rho: leg.rho,
            magnetic_course: leg.magnetic_course,
            length: if leg.distance_time == Some("D".to_string()) {
                leg.route_distance_holding_distance_time
            } else {
                None
            },
            length_time: if leg.distance_time == Some("T".to_string()) {
                leg.route_distance_holding_distance_time
            } else {
                None
            },
            turn_direction: leg.turn_direction,
            arc_center_fix: if !leg.center_id.is_empty() {
                Some(Fix::from_row_data(
                    leg.center_waypoint_latitude.unwrap(),
                    leg.center_waypoint_longitude.unwrap(),
                    leg.center_id,
                ))
            } else {
                None
            },
            arc_radius: leg.arc_radius,
            leg_type: leg.path_termination,
        }
    }
}

impl From<v2::sql_structs::Procedures> for ProcedureLeg {
    fn from(leg: v2::sql_structs::Procedures) -> Self {
        ProcedureLeg {
            overfly: leg
                .waypoint_description_code
                .map_or(false, |x| x.chars().nth(1) == Some('Y')),
            altitude: leg.altitude1.map(|altitude1| AltitudeContstraint {
                altitude1,
                altitude2: leg.altitude2,
                descriptor: leg
                    .altitude_description
                    .unwrap_or(AltitudeDescriptor::AtAlt1),
            }),
            speed: leg.speed_limit.map(|speed| SpeedConstraint {
                value: speed,
                descriptor: leg
                    .speed_limit_description
                    .unwrap_or(SpeedDescriptor::Mandatory),
            }),
            vertical_angle: leg.vertical_angle,
            rnp: leg.rnp,
            fix: if leg.waypoint_identifier.is_some() {
                Some(Fix::from_row_data_v2(
                    leg.waypoint_latitude.unwrap(),
                    leg.waypoint_longitude.unwrap(),
                    leg.waypoint_identifier.unwrap(),
                    leg.waypoint_icao_code.unwrap(),
                    Some(leg.airport_identifier.clone()),
                    leg.waypoint_ref_table,
                ))
            } else {
                None
            },
            recommended_navaid: if leg.recommended_navaid.is_some() {
                Some(Fix::from_row_data_v2(
                    leg.recommended_navaid_latitude.unwrap(),
                    leg.recommended_navaid_longitude.unwrap(),
                    leg.recommended_navaid.unwrap(),
                    leg.recommended_navaid_icao_code.unwrap(),
                    Some(leg.airport_identifier.clone()),
                    leg.recommended_navaid_ref_table,
                ))
            } else {
                None
            },
            theta: leg.theta,
            rho: leg.rho,
            magnetic_course: None,
            length: if leg.route_distance_holding_distance_time == Some("D".to_string()) {
                leg.distance_time
            } else {
                None
            },
            length_time: if leg.route_distance_holding_distance_time == Some("T".to_string()) {
                leg.distance_time
            } else {
                None
            },
            turn_direction: leg.turn_direction,
            arc_center_fix: if leg.center_waypoint.is_some() {
                Some(Fix::from_row_data_v2(
                    leg.center_waypoint_latitude.unwrap(),
                    leg.center_waypoint_longitude.unwrap(),
                    leg.center_waypoint.unwrap(),
                    leg.center_waypoint_icao_code.unwrap(),
                    Some(leg.airport_identifier),
                    leg.center_waypoint_ref_table,
                ))
            } else {
                None
            },
            arc_radius: leg.arc_radius,
            leg_type: leg.path_termination,
        }
    }
}
