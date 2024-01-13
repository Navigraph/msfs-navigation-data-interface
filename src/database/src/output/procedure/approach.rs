use std::collections::{hash_map::Entry, HashMap};

use regex::Regex;
use serde::Serialize;

use super::{apply_enroute_transition_leg, Transition};
use crate::{enums::ApproachType, output::procedure_leg::ProcedureLeg, sql_structs};
#[derive(Serialize)]
pub struct Approach {
    ident: String,
    transitions: Vec<Transition>,
    legs: Vec<ProcedureLeg>,
    missed_legs: Vec<ProcedureLeg>,

    runway_ident: Option<String>,
    approach_type: ApproachType,
}

fn split_approach_ident(ident: String) -> Option<(String, String, Option<String>)> {
    let regex = Regex::new("^([A-Z])([0-9]{2}[LCR]?)-?([A-Z])?$").unwrap();
    let captures = regex.captures_iter(ident.as_str()).next()?;

    Some((
        captures.get(1).unwrap().as_str().to_string(),
        captures.get(2).unwrap().as_str().to_string(),
        captures.get(3).map(|x| x.as_str().to_string()),
    ))
}

pub fn map_approaches(data: Vec<sql_structs::Procedures>) -> Vec<Approach> {
    let mut missed_started = false;

    data.into_iter()
        .fold(HashMap::new(), |mut approaches, row| {
            let approach = match approaches.entry(row.procedure_identifier.clone()) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    missed_started = false;

                    entry.insert(Approach {
                        ident: row.procedure_identifier.clone(),
                        transitions: Vec::new(),
                        legs: Vec::new(),
                        missed_legs: Vec::new(),
                        runway_ident: split_approach_ident(row.procedure_identifier.clone())
                            .map(|(_, runway_ident, _)| format!("RW{}", runway_ident)),
                        approach_type: ApproachType::Fms, /* Set to an arbitrary value, will be overwritten once we
                                                           * find a row with a valid approach type (the first row in
                                                           * an approach will usually be a transition so it can not
                                                           * be used to find the approach type) */
                    })
                },
            };

            let route_type = row.route_type.clone();
            let transition_identifier = row.transition_identifier.clone();

            if let Some(description_code) = &row.waypoint_description_code {
                if description_code.chars().nth(2) == Some('M') {
                    missed_started = true;
                }
            }

            let leg = ProcedureLeg::from(row);

            match route_type.as_str() {
                "A" => apply_enroute_transition_leg(
                    leg,
                    transition_identifier.expect("Transition leg was found without a transition identifier"),
                    &mut approach.transitions,
                ),
                "Z" => approach.missed_legs.push(leg),
                x => {
                    if missed_started || x == "Z" {
                        approach.missed_legs.push(leg);
                    } else {
                        approach.approach_type = serde_json::from_str(format!(r#""{}""#, route_type).as_str()).unwrap();

                        approach.legs.push(leg)
                    }
                },
            }

            approaches
        })
        .into_values()
        .collect()
}
