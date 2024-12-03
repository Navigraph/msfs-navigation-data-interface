use std::collections::{hash_map::Entry, HashMap};

use serde::Serialize;

use super::{
    apply_common_leg, apply_common_leg_v2, apply_enroute_transition_leg,
    apply_runway_transition_leg, apply_runway_transition_leg_v2, Transition,
};
use crate::{output::procedure_leg::ProcedureLeg, sql_structs, v2};

#[derive(Serialize)]
pub struct Departure {
    /// The `ident` uniquely identifies this arrival within the airport which it serves.
    ///
    /// While departure identifiers may seem unique everywhere, it is possible for two airports to share a departure or
    /// have a departure of the same name like Approaches
    ident: String,
    /// A list of runway transitions which are part of this departure.
    ///
    /// This field can be used to determine which runways this departure serves, and is garunteed to always have at
    /// least one value.
    runway_transitions: Vec<Transition>,
    /// A list of legs which apply to all runways which this departure serves.
    ///
    /// Keep in mind it is not common for this field to have any values as most departure consist only serve a single
    /// runway, and will hence have a single runway transition and no `common_legs`
    common_legs: Vec<ProcedureLeg>,
    /// A list of the transitions which are available for this arrival.
    enroute_transitions: Vec<Transition>,
    engine_out_legs: Vec<ProcedureLeg>,
}

/// Maps a list of departure rows from the sqlite database into `Departure` structs, by condensing them using
/// `procedure_identifier` and `transition_identifier`
///
/// This function requires complete data for a single airport and the same ordering as the database provides by default.
///
/// The recommended SQL query to load the neccesary data for this function is:
/// ```sql
/// SELECT * FROM tbl_sids WHERE airport_identifier = (?1)
/// ```
pub(crate) fn map_departures(
    data: Vec<sql_structs::Procedures>,
    runways: Vec<sql_structs::Runways>,
) -> Vec<Departure> {
    data.into_iter()
        .fold(HashMap::new(), |mut departures, row| {
            let departure = match departures.entry(row.procedure_identifier.clone()) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(Departure {
                    ident: row.procedure_identifier.clone(),
                    runway_transitions: Vec::new(),
                    common_legs: Vec::new(),
                    enroute_transitions: Vec::new(),
                    engine_out_legs: Vec::new(),
                }),
            };

            let route_type = row.route_type.clone();
            let transition_identifier = row.transition_identifier.clone();

            let leg = ProcedureLeg::from(row);

            // We want to ensure there is a runway transition for every single runway which this procedure serves, even
            // if the procedure does not differ between runways This makes it very easy to implement in an FMS as there
            // need not be special logic for determining which runways are compatible
            match route_type.as_str() {
                "0" => departure.engine_out_legs.push(leg),
                // These route types are for runway transitions
                "1" | "4" | "F" | "T" => apply_runway_transition_leg(
                    leg,
                    transition_identifier
                        .expect("Runway transition leg was found without a transition identifier"),
                    &mut departure.runway_transitions,
                    &runways,
                ),
                // These route types are for common legs
                "2" | "5" | "M" => apply_common_leg(
                    leg,
                    transition_identifier,
                    &mut departure.runway_transitions,
                    &mut departure.common_legs,
                    &runways,
                ),
                // These route types are for enroute transitions
                "3" | "6" | "S" | "V" => apply_enroute_transition_leg(
                    leg,
                    transition_identifier
                        .expect("Enroute transition leg was found without a transition identifier"),
                    &mut departure.enroute_transitions,
                ),
                _ => unreachable!(),
            }

            departures
        })
        .into_values()
        .collect()
}

pub(crate) fn map_departures_v2(
    data: Vec<v2::sql_structs::Procedures>,
    runways: Vec<v2::sql_structs::Runways>,
) -> Vec<Departure> {
    data.into_iter()
        .fold(HashMap::new(), |mut departures, row| {
            let departure = match departures.entry(row.procedure_identifier.clone()) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(Departure {
                    ident: row.procedure_identifier.clone(),
                    runway_transitions: Vec::new(),
                    common_legs: Vec::new(),
                    enroute_transitions: Vec::new(),
                    engine_out_legs: Vec::new(),
                }),
            };

            let route_type = row.route_type.clone();
            let transition_identifier = row.transition_identifier.clone();

            let leg = ProcedureLeg::from(row);

            // We want to ensure there is a runway transition for every single runway which this procedure serves, even
            // if the procedure does not differ between runways This makes it very easy to implement in an FMS as there
            // need not be special logic for determining which runways are compatible
            match route_type.as_str() {
                "0" => departure.engine_out_legs.push(leg),
                // These route types are for runway transitions
                "1" | "4" | "F" | "T" => apply_runway_transition_leg_v2(
                    leg,
                    transition_identifier
                        .expect("Runway transition leg was found without a transition identifier"),
                    &mut departure.runway_transitions,
                    &runways,
                ),
                // These route types are for common legs
                "2" | "5" | "M" => apply_common_leg_v2(
                    leg,
                    transition_identifier,
                    &mut departure.runway_transitions,
                    &mut departure.common_legs,
                    &runways,
                ),
                // These route types are for enroute transitions
                "3" | "6" | "S" | "V" => apply_enroute_transition_leg(
                    leg,
                    transition_identifier
                        .expect("Enroute transition leg was found without a transition identifier"),
                    &mut departure.enroute_transitions,
                ),
                _ => unreachable!(),
            }

            departures
        })
        .into_values()
        .collect()
}
