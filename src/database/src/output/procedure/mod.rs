use serde::Serialize;

use super::procedure_leg::ProcedureLeg;
use crate::sql_structs::Runways;

pub mod approach;
pub mod arrival;
pub mod departure;

#[derive(Serialize)]
pub struct Transition {
    ident: String,
    legs: Vec<ProcedureLeg>,
}

/// A helper function which returns a mutable reference to an item in a vector if it can be found using the `condition`,
/// or inserts a new item `val` into the vector and returns a mutable reference to it.
fn mut_find_or_insert<T, P: FnMut(&T) -> bool>(vec: &mut Vec<T>, condition: P, val: T) -> &mut T {
    if let Some(index) = vec.iter().position(condition) {
        &mut vec[index]
    } else {
        vec.push(val);

        vec.last_mut().unwrap()
    }
}

/// Applies the neccesary logic for adding a leg with an enroute transition route type into a procedure
pub(self) fn apply_enroute_transition_leg(
    leg: ProcedureLeg, transition_identifier: String, enroute_transitions: &mut Vec<Transition>,
) {
    let transition = mut_find_or_insert(
        enroute_transitions,
        |transition| transition.ident == transition_identifier,
        Transition {
            ident: transition_identifier.to_string(),
            legs: Vec::new(),
        },
    );

    transition.legs.push(leg);
}

/// Applies the neccesary logic for adding a leg with a common leg route type into a procedure
pub(self) fn apply_common_leg(
    leg: ProcedureLeg,
    transition_identifier: Option<String>,
    runway_transitions: &mut Vec<Transition>,
    common_legs: &mut Vec<ProcedureLeg>,
    runways: &Vec<Runways>,
) {
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
            for runway in runways.iter() {
                let transition = mut_find_or_insert(
                    runway_transitions,
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
            let target_runways = runways
                .iter()
                .filter(|runway| runway.runway_identifier[0..4] == transition_identifier[0..4]);

            for runway in target_runways {
                let transition = mut_find_or_insert(
                    runway_transitions,
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
                runway_transitions,
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
        common_legs.push(leg);
    }
}

/// Applies the neccesary logic for adding a leg with a runway transition route type into a procedure
pub(self) fn apply_runway_transition_leg(
    leg: ProcedureLeg,
    transition_identifier: String,
    runway_transitions: &mut Vec<Transition>,
    runways: &Vec<Runways>,
) {
    // If transition identifier ends in B, it means this transition serves all runways with the same
    // number. To make this easier to use in an FMS, we duplicate the transitions for all runways which
    // it serves
    if transition_identifier.chars().nth(4) == Some('B') {
        let target_runways = runways
            .iter()
            .filter(|runway| runway.runway_identifier[0..4] == transition_identifier[0..4]);

        for runway in target_runways {
            let transition = mut_find_or_insert(
                runway_transitions,
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
            runway_transitions,
            |transition| transition.ident == transition_identifier,
            Transition {
                ident: transition_identifier.to_string(),
                legs: Vec::new(),
            },
        );

        transition.legs.push(leg.clone());
    }
}
