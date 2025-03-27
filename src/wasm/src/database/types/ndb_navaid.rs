use sentry::capture_message;
use serde::Serialize;

use crate::database::utils::{Coordinates, KiloHertz, NauticalMiles};

use super::sql;

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
pub struct NdbNavaid {
    /// Represents the geographic region in which this NdbNavaid is located
    pub area_code: String,
    /// Continent of the waypoint (v2 only)
    pub continent: Option<String>,
    /// Country of the waypoint (v2 only)
    pub country: Option<String>,
    /// 3 Letter identifier describing the local horizontal identifier (v2 only)
    pub datum_code: Option<String>,
    /// The identifier of the airport that this NdbNavaid is associated with, if any
    pub airport_ident: Option<String>,
    /// The icao prefix of the region that this NdbNavaid is in.
    pub icao_code: String,
    /// The identifier of this NdbNavaid (not unique), such as `BI` or `PHH`
    pub ident: String,
    /// The formal name of this NdbNavaid such as `HERBB OLATHE` or `KEDZI CHICAGO`
    pub name: String,
    /// The frequency of this NdbNavaid in kilohertz
    pub frequency: KiloHertz,
    /// The geographic location of thie NdbNavaid
    pub location: Coordinates,
    /// Range of the NDB (v2 only)
    pub range: Option<NauticalMiles>,
}

impl From<sql::NdbNavaids> for NdbNavaid {
    fn from(navaid: sql::NdbNavaids) -> Self {
        let mut error_in_row = false;

        let navaid_new = Self {
            area_code: navaid.area_code.clone(),
            airport_ident: navaid.airport_identifier,
            icao_code: navaid.icao_code.unwrap_or_else(|| {
                error_in_row = true;
                "UNKN".to_string()
            }),
            ident: navaid.navaid_identifier.unwrap_or_else(|| {
                error_in_row = true;
                "UNKN".to_string()
            }),
            name: navaid.navaid_name.clone(),
            frequency: navaid.navaid_frequency,
            location: Coordinates {
                lat: navaid.navaid_latitude.unwrap_or_else(|| {
                    error_in_row = true;
                    0.
                }),
                long: navaid.navaid_longitude.unwrap_or_else(|| {
                    error_in_row = true;
                    0.
                }),
            },
            continent: navaid.continent,
            country: navaid.country,
            datum_code: navaid.datum_code,
            range: navaid.range,
        };

        if error_in_row {
            let error_text = format!(
                "Error found in NdbNavaid: {}",
                serde_json::to_string(&navaid_new).unwrap_or(format!(
                    "Error serializing output, {} navaid {}",
                    navaid.area_code, navaid.navaid_name
                ))
            );

            capture_message(&error_text, sentry::Level::Warning);
        }

        navaid_new
    }
}
