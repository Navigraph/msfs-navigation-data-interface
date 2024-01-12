use std::collections::{hash_map::Entry, HashMap};

use serde::Serialize;

use super::{apply_common_leg, apply_enroute_transition_leg, apply_runway_transition_leg, Transition};
use crate::{output::procedure_leg::ProcedureLeg, sql_structs};

#[derive(Serialize)]
pub struct Departure {
    ident: String,
    runway_transitions: Vec<Transition>,
    common_legs: Vec<ProcedureLeg>,
    enroute_transitions: Vec<Transition>,
    engine_out_legs: Vec<ProcedureLeg>,
}

pub fn map_departures(data: Vec<sql_structs::Procedures>, runways: Vec<sql_structs::Runways>) -> Vec<Departure> {
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
                    transition_identifier.expect("Runway transition leg was found without a transition identifier"),
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
                    transition_identifier.expect("Enroute transition leg was found without a transition identifier"),
                    &mut departure.enroute_transitions,
                ),
                _ => unreachable!(),
            }

            departures
        })
        .into_values()
        .collect()
}
