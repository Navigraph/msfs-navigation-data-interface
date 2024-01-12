use std::collections::{hash_map::Entry, HashMap};

use serde::Serialize;

use super::{apply_common_leg, apply_enroute_transition_leg, apply_runway_transition_leg, Transition};
use crate::{output::procedure_leg::ProcedureLeg, sql_structs};

#[derive(Serialize)]
pub struct Arrival {
    ident: String,
    enroute_transitions: Vec<Transition>,
    common_legs: Vec<ProcedureLeg>,
    runway_transitions: Vec<Transition>,
}

pub fn map_arrivals(data: Vec<sql_structs::Procedures>, runways: Vec<sql_structs::Runways>) -> Vec<Arrival> {
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
                    transition_identifier.expect("Enroute transition leg was found without a transition identifier"),
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
                    transition_identifier.expect("Runway transition leg was found without a transition identifier"),
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
