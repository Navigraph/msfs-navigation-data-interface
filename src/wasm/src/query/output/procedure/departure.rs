use std::collections::{hash_map::Entry, HashMap};

use serde::Serialize;

use super::{mut_find_or_insert, Transition};
use crate::query::{output::procedure_leg::ProcedureLeg, sql_structs};

#[derive(Serialize)]
pub struct Departure {
    ident: String,
    runway_transitions: Vec<Transition>,
    common_legs: Vec<ProcedureLeg>,
    enroute_transitions: Vec<Transition>,
    engine_out_legs: Vec<ProcedureLeg>,

    /// Indicates whether all the runway_transitions on this are encoded the same way, so runway selection does not
    /// need to occur. This would usually be encoded in common legs however it must be in runway transitions to
    /// indicate which runways it is compatible with
    identical_runway_transitions: bool,
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

                    identical_runway_transitions: false,
                }),
            };

            let route_type = row.route_type.clone();
            let transition_identifier = row.transition_identifier.clone();

            let leg = ProcedureLeg::from(row);

            // We want to ensure there is a runway transition for every single runway which this procedure serves, even
            // if the procedure does not differ between runways This makes it very easy to implement in an FMS as there
            // need not be special logic for determining which runways are compatible
            //
            // One consideration to make is whether we indicate that all the runway transitions are the same, so that
            // the procedure can be used without selecting a specific runway
            match route_type.as_str() {
                "0" => departure.engine_out_legs.push(leg),
                // These route types are for runway transitions
                "1" | "4" | "F" | "T" => {
                    let transition_identifier =
                        transition_identifier.expect("Runway transition leg was found without a transition identifier");

                    // If transition identifier ends in B, it means this transition serves all runways with the same
                    // number. To make this easier to use in an FMS, we duplicate the transitions for all runways which
                    // it serves
                    if transition_identifier.chars().nth(4) == Some('B') {
                        let target_runways = runways
                            .iter()
                            .filter(|runway| runway.runway_identifier[0..4] == transition_identifier[0..4]);

                        for runway in target_runways {
                            let transition = mut_find_or_insert(
                                &mut departure.runway_transitions,
                                |transition| transition.ident == runway.runway_identifier,
                                Transition {
                                    ident: runway.runway_identifier.clone(),
                                    legs: Vec::new(),
                                },
                            );

                            transition.legs.push(leg.clone());
                        }
                    } else {
                        let transition = mut_find_or_insert(
                            &mut departure.runway_transitions,
                            |transition| transition.ident == transition_identifier,
                            Transition {
                                ident: transition_identifier.to_string(),
                                legs: Vec::new(),
                            },
                        );

                        transition.legs.push(leg.clone());
                    }
                },
                // These route types are for common legs
                "2" | "5" | "M" => {
                    // Common legs can still have a transition identifier, meaning that this procedure is only for
                    // specific runways, but with the same legs for each runway.
                    //
                    // If it is not present, it means there are seperate runway transitions for each runway after these
                    // common legs
                    if let Some(transition_identifier) = transition_identifier {
                        // If the transition identifier is `ALL`, this means that this procedure is for all runways and
                        // has exactly the same legs for each runway, so we insert a runway transition for every runway
                        // at the airport
                        if transition_identifier == "ALL" {
                            departure.identical_runway_transitions = true;

                            for runway in runways.iter() {
                                let transition = mut_find_or_insert(
                                    &mut departure.runway_transitions,
                                    |transition| transition.ident == runway.runway_identifier,
                                    Transition {
                                        ident: runway.runway_identifier.clone(),
                                        legs: Vec::new(),
                                    },
                                );

                                transition.legs.push(leg.clone());
                            }
                        // When the identifier ends with B, this procedure is for all runways with that number
                        } else if transition_identifier.chars().nth(4) == Some('B') {
                            departure.identical_runway_transitions = true;

                            let target_runways = runways
                                .iter()
                                .filter(|runway| runway.runway_identifier[0..4] == transition_identifier[0..4]);

                            for runway in target_runways {
                                let transition = mut_find_or_insert(
                                    &mut departure.runway_transitions,
                                    |transition| transition.ident == runway.runway_identifier,
                                    Transition {
                                        ident: runway.runway_identifier.clone(),
                                        legs: Vec::new(),
                                    },
                                );

                                transition.legs.push(leg.clone());
                            }
                        // In this case, the transition identifier is for a specific runway, so we insert it as a runway
                        // transition to indicate which runway this procedure is specifically for
                        } else {
                            let transition = mut_find_or_insert(
                                &mut departure.runway_transitions,
                                |transition| transition.ident == transition_identifier,
                                Transition {
                                    ident: transition_identifier.to_string(),
                                    legs: Vec::new(),
                                },
                            );

                            transition.legs.push(leg);
                        }
                    // When there is no transiton identifier, that means there are seperate runway transitions, so these
                    // legs should actually be inserted as common legs
                    } else {
                        departure.common_legs.push(leg);
                    }
                },
                // These route types are for enroute transitions
                "3" | "6" | "S" | "V" => {
                    let transition_identifier = transition_identifier
                        .expect("Enroute transition leg was found without a transition identifier");

                    let transition = mut_find_or_insert(
                        &mut departure.enroute_transitions,
                        |transition| transition.ident == transition_identifier,
                        Transition {
                            ident: transition_identifier.to_string(),
                            legs: Vec::new(),
                        },
                    );

                    transition.legs.push(leg);
                },
                _ => unreachable!(),
            }

            departures
        })
        .into_values()
        .collect()
}
