use std::collections::{hash_map::Entry, HashMap};

use regex::Regex;
use serde::Serialize;

use super::{apply_enroute_transition_leg, Transition};
use crate::{enums::ApproachType, output::procedure_leg::ProcedureLeg, sql_structs};

#[derive(Serialize)]
/// Represents an approach procedure for an airport.
///
/// # Example
/// Basic querying:
/// ```rs
/// let database = Database::new();
/// let approaches: Vec<Approach> = database.get_approaches_at_airport("KJFK");
/// ```
pub struct Approach {
    /// The `ident` uniquely identifies this approach within the airport which it serves
    ///
    /// For approaches which are for a specific runway, it will have a format such as `I08L` or `R12-M`.
    /// - The first character identifies the type of approach, however this will not always match the `approach_type`
    /// field. The next three characters represent the runway identifier, such as `08L` or `12`.
    /// - The 5th character (optional) is the multiple indicator of the approach, it can be any capital letter.
    /// - For approaches with a multiple indicator and no `LCR` on the runway, the 4th character will be a `-`
    ///
    /// If this approach is for no specific runway, it will have a format such as `RNVC` or `GPSM`
    ident: String,
    /// Contains the transitions for the approach. On Airbus aircraft, these are known as `VIAs`.
    transitions: Vec<Transition>,
    /// Contains the legs which make up the main body of this approach.
    legs: Vec<ProcedureLeg>,
    /// Contains the legs which are part of the missed approach portion of this approach.
    missed_legs: Vec<ProcedureLeg>,
    /// Represents the runway which this approach is for, if it is for a specific runway.
    ///
    /// This Field is generated from the `ident` in order to better match the `ident` field of `RunwayThreshold`.
    ///
    /// e.g. `RW27L`
    runway_ident: Option<String>,
    /// Determines the type of approach, such as ILS, GPS, RNAV, etc.
    ///
    /// This is not garunteed to match the type found through the `ident` field.
    approach_type: ApproachType,
}

/// Extracts the following information from a standard runway approach identifier.
/// - The approach type character
/// - The runway identifier
/// - The multiple indicator
///
/// If the approach identifier is not in this format, this function will return `None`.
///
/// # Example
/// ```rs
/// let (approach_type, runway_ident, multiple_indicator) = split_approach_ident("I08L".to_string()).unwrap();
///
/// assert_eq!(approach_type, "I");
/// assert_eq!(runway_ident, "08L");
/// assert_eq!(multiple_indicator, None);
/// ```
pub fn split_approach_ident(ident: String) -> Option<(String, String, Option<String>)> {
    let regex = Regex::new("^([A-Z])([0-9]{2}[LCR]?)-?([A-Z])?$").unwrap();
    let captures = regex.captures_iter(ident.as_str()).next()?;

    Some((
        captures.get(1).unwrap().as_str().to_string(),
        captures.get(2).unwrap().as_str().to_string(),
        captures.get(3).map(|x| x.as_str().to_string()),
    ))
}

/// Maps a list of approach rows from the sqlite database into `Approach` structs, by condensing them by
/// `procedure_identifier` and `transition_identifier`
///
/// This function requires complete data for a single airport and the same ordering as the database provides by default.
///
/// The recommended SQL query to load the neccesary data for this function is:
/// ```sql
/// SELECT * FROM tbl_iaps WHERE airport_identifier = (?1)
/// ```
pub(crate) fn map_approaches(data: Vec<sql_structs::Procedures>) -> Vec<Approach> {
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
                }
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
                    transition_identifier
                        .expect("Transition leg was found without a transition identifier"),
                    &mut approach.transitions,
                ),
                "Z" => approach.missed_legs.push(leg),
                x => {
                    if missed_started || x == "Z" {
                        approach.missed_legs.push(leg);
                    } else {
                        approach.approach_type =
                            serde_json::from_value(serde_json::Value::String(route_type)).unwrap();

                        approach.legs.push(leg)
                    }
                }
            }

            approaches
        })
        .into_values()
        .collect()
}
