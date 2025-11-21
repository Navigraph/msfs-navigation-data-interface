use std::str::FromStr;

use serde::Serialize;

use super::sql;

#[derive(Serialize)]
pub struct DatabaseInfo {
    /// The AIRAC cycle that this database is.
    ///
    /// e.g. `2313` or `2107`
    airac_cycle: String,
    /// The effective date range of this AIRAC cycle.
    effective_from_to: (String, String),
    /// The effective date range of the previous AIRAC cycle
    previous_from_to: (String, String),
}

/// Converts a string of the format `DDMMDDMMYY` into a tuple of two strings of the format `DD-MM-YYYY`.
///
/// If the previous month is greater than the current month, the previous year is decremented by 1.
fn parse_from_to(data: String) -> Result<(String, String), <u32 as FromStr>::Err> {
    let from_day = data[0..2].parse::<u32>()?;
    let from_month = data[2..4].parse::<u32>()?;
    let to_day = data[4..6].parse::<u32>()?;
    let to_month = data[6..8].parse::<u32>()?;
    let to_year = data[8..10].parse::<u32>()?;

    let from_year = if to_month < from_month {
        to_year - 1
    } else {
        to_year
    };

    Ok((
        format!("{from_day:0>2}-{from_month:0>2}-20{from_year:0>2}"),
        format!("{to_day:0>2}-{to_month:0>2}-20{to_year:0>2}"),
    ))
}

impl From<sql::Header> for DatabaseInfo {
    fn from(header: sql::Header) -> Self {
        Self {
            airac_cycle: header.cycle,
            effective_from_to: parse_from_to(header.effective_fromto).unwrap(),
            previous_from_to: ("deprecated".to_string(), "deprecated".to_string()),
        }
    }
}
