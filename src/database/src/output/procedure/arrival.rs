use std::collections::{hash_map::Entry, HashMap};

use serde::Serialize;

use super::{
    apply_common_leg, apply_common_leg_v2, apply_enroute_transition_leg,
    apply_runway_transition_leg, apply_runway_transition_leg_v2, Transition,
};
use crate::{output::procedure_leg::ProcedureLeg, sql_structs, v2};

#[derive(Serialize)]
/// Represents an arrival procedure (STAR) for an airport.
///
/// # Example
/// Basic querying:
/// ```rs
/// let database = Database::new();
/// let approaches: Vec<Approach> = database.get_arrivals_at_airport("KJFK");
/// ```
pub struct Arrival {
    /// The `ident` uniquely identifies this arrival within the airport which it serves.
    ///
    /// While arrival identifiers may seem unique everywhere, it is possible for two airports to share a arrival or
    /// have a arrival of the same name like Approaches
    ident: String,
    /// A list of the transitions which are available for this arrival.
    enroute_transitions: Vec<Transition>,
    /// A list of legs which apply to all runways which this arrival serves.
    ///
    /// Keep in mind it is not common for this field to have any values as most arrivals consist only serve a single
    /// runway, and will hence have a single runway transition and no `common_legs`
    common_legs: Vec<ProcedureLeg>,
    /// A list of runway transitions which are part of this Arrival.
    ///
    /// This field can be used to determine which runways this arrival serves, and is garunteed to always have at
    /// least one value.
    runway_transitions: Vec<Transition>,
}

/// Maps a list of arrival rows from the sqlite database into `Arrival` structs, by condensing them using
/// `procedure_identifier` and `transition_identifier`
///
/// This function requires complete data for a single airport and the same ordering as the database provides by default.
///
/// The recommended SQL query to load the neccesary data for this function is:
/// ```sql
/// SELECT * FROM tbl_stars WHERE airport_identifier = (?1)
/// ```
pub(crate) fn map_arrivals(
    data: Vec<sql_structs::Procedures>,
    runways: Vec<sql_structs::Runways>,
) -> Vec<Arrival> {
    data.into_iter()
        .fold(HashMap::new(), |mut arrivals, row| {
            let arrival = match arrivals.entry(row.procedure_identifier.clone()) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(Arrival {
                    ident: row.procedure_identifier.clone(),
                    enroute_transitions: Vec::new(),
                    common_legs: Vec::new(),
                    runway_transitions: Vec::new(),
                }),
            };

            let route_type = row.route_type.clone();
            let transition_identifier = row.transition_identifier.clone();

            let leg = ProcedureLeg::from(row);

            // We want to ensure there is a runway transition for every single runway which this procedure serves, even
            // if the procedure does not differ between runways This makes it very easy to implement in an FMS as there
            // need not be special logic for determining which runways are compatible
            match route_type.as_str() {
                "1" | "4" | "7" | "F" => apply_enroute_transition_leg(
                    leg,
                    transition_identifier
                        .expect("Enroute transition leg was found without a transition identifier"),
                    &mut arrival.enroute_transitions,
                ),
                // These route types are for common legs
                "2" | "5" | "8" | "M" => apply_common_leg(
                    leg,
                    transition_identifier,
                    &mut arrival.runway_transitions,
                    &mut arrival.common_legs,
                    &runways,
                ),
                // These route types are for runway transitions
                "3" | "6" | "9" | "S" => apply_runway_transition_leg(
                    leg,
                    transition_identifier
                        .expect("Runway transition leg was found without a transition identifier"),
                    &mut arrival.runway_transitions,
                    &runways,
                ),
                _ => unreachable!(),
            }

            arrivals
        })
        .into_values()
        .collect()
}

pub(crate) fn map_arrivals_v2(
    data: Vec<v2::sql_structs::Procedures>,
    runways: Vec<v2::sql_structs::Runways>,
) -> Vec<Arrival> {
    data.into_iter()
        .fold(HashMap::new(), |mut arrivals, row| {
            let arrival = match arrivals.entry(row.procedure_identifier.clone()) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(Arrival {
                    ident: row.procedure_identifier.clone(),
                    enroute_transitions: Vec::new(),
                    common_legs: Vec::new(),
                    runway_transitions: Vec::new(),
                }),
            };

            let route_type = row.route_type.clone();
            let transition_identifier = row.transition_identifier.clone();

            let leg = ProcedureLeg::from(row);

            // We want to ensure there is a runway transition for every single runway which this procedure serves, even
            // if the procedure does not differ between runways This makes it very easy to implement in an FMS as there
            // need not be special logic for determining which runways are compatible
            match route_type.as_str() {
                "1" | "4" | "7" | "F" => apply_enroute_transition_leg(
                    leg,
                    transition_identifier
                        .expect("Enroute transition leg was found without a transition identifier"),
                    &mut arrival.enroute_transitions,
                ),
                // These route types are for common legs
                "2" | "5" | "8" | "M" => apply_common_leg_v2(
                    leg,
                    transition_identifier,
                    &mut arrival.runway_transitions,
                    &mut arrival.common_legs,
                    &runways,
                ),
                // These route types are for runway transitions
                "3" | "6" | "9" | "S" => apply_runway_transition_leg_v2(
                    leg,
                    transition_identifier
                        .expect("Runway transition leg was found without a transition identifier"),
                    &mut arrival.runway_transitions,
                    &runways,
                ),
                _ => unreachable!(),
            }

            arrivals
        })
        .into_values()
        .collect()
}
