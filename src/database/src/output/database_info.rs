use std::str::FromStr;

use serde::Serialize;

use crate::sql_structs;

#[derive(Serialize)]
pub struct DatabaseInfo {
    airac_cycle: String,
    effective_from_to: (String, String),
    previous_from_to: (String, String),
}

fn parse_from_to(data: String) -> Result<(String, String), <u32 as FromStr>::Err> {
    let from_day = data[0..2].parse::<u32>()?;
    let from_month = data[2..4].parse::<u32>()?;
    let to_day = data[4..6].parse::<u32>()?;
    let to_month = data[6..8].parse::<u32>()?;
    let to_year = data[8..10].parse::<u32>()?;

    let from_year = if to_month < from_month { to_year - 1 } else { to_year };

    Ok((
        format!("{from_day:0>2}-{from_month:0>2}-20{from_year:0>2}"),
        format!("{to_day:0>2}-{to_month:0>2}-20{to_year:0>2}"),
    ))
}
impl From<sql_structs::Header> for DatabaseInfo {
    fn from(header: sql_structs::Header) -> Self {
        Self {
            airac_cycle: header.current_airac,
            effective_from_to: parse_from_to(header.effective_fromto).unwrap(),
            previous_from_to: parse_from_to(header.previous_fromto).unwrap(),
        }
    }
}
