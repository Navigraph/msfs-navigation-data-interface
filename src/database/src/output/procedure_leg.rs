use serde::Serialize;

use super::fix::{map_fix, Fix};
use crate::{
    enums::{AltitudeDescriptor, LegType, SpeedDescriptor, TurnDirection},
    math::{Degrees, Feet, Knots, Minutes, NauticalMiles},
    sql_structs,
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
pub struct ProcedureLeg {
    overfly: bool,

    leg_type: LegType,

    altitude: Option<AltitudeContstraint>,

    speed: Option<SpeedConstraint>,

    vertical_angle: Option<Degrees>,

    rnp: Option<NauticalMiles>,

    fix: Option<Fix>,

    recommended_navaid: Option<Fix>,

    theta: Option<Degrees>,

    rho: Option<NauticalMiles>,

    magnetic_course: Option<Degrees>,

    length: Option<NauticalMiles>,

    length_time: Option<Minutes>,

    turn_direction: Option<TurnDirection>,

    arc_center_fix: Option<Fix>,

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
                descriptor: leg.altitude_description.unwrap_or(AltitudeDescriptor::AtAlt1),
            }),
            speed: leg.speed_limit.map(|speed| SpeedConstraint {
                value: speed,
                descriptor: leg.speed_limit_description.unwrap_or(SpeedDescriptor::Mandatory),
            }),
            vertical_angle: leg.vertical_angle,
            rnp: leg.rnp,
            fix: if !leg.id.is_empty() {
                Some(map_fix(
                    leg.waypoint_latitude.unwrap(),
                    leg.waypoint_longitude.unwrap(),
                    leg.id,
                ))
            } else {
                None
            },
            recommended_navaid: if !leg.recommanded_id.is_empty() {
                Some(map_fix(
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
                Some(map_fix(
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
