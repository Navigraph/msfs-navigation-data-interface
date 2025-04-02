use serde::Serialize;

use crate::database::utils::Coordinates;

use super::{
    enums::{CommunicationType, FrequencyUnits},
    sql,
};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Debug, Default)]
/// Represents a communication station at an airport or in an enroute fir
pub struct Communication {
    /// The Geographic region where this communication is
    pub area_code: String,
    /// The type of communication
    pub communication_type: CommunicationType,
    /// The identifier of the airport which this communication is at, if this an airport communication
    pub airport_ident: Option<String>,
    /// The identifier of the FIR which this communication is in, if this is an enroute communication
    pub fir_rdo_ident: Option<String>,
    /// The frequency of this communication
    pub frequency: f64,
    /// The units of the frequency of this communication
    pub frequency_units: FrequencyUnits,
    /// The callsign of this communication
    pub callsign: Option<String>,
    /// The name of this communication (only defined for enroute communications)
    pub name: Option<String>,
    /// The location of this communication
    pub location: Coordinates,
    /// Facility in which an RCO will be transmitting through
    pub remote_facility: Option<String>, // new
    pub remote_facility_icao_code: Option<String>, // new
    /// Sector associated with the communication
    pub sector_facility: Option<String>, // new
    pub sector_facility_icao_code: Option<String>, // new
    /// Bearings from the sector facility is applicable to the communication
    pub sectorization: Option<String>, // new
}

impl From<sql::AirportCommunication> for Communication {
    fn from(row: sql::AirportCommunication) -> Self {
        Self {
            area_code: row.area_code,
            communication_type: row.communication_type,
            airport_ident: Some(row.airport_identifier),
            fir_rdo_ident: None,
            frequency: row.communication_frequency,
            frequency_units: row.frequency_units,
            callsign: row.callsign,
            name: None,
            location: Coordinates {
                lat: row.latitude,
                long: row.longitude,
            },
            remote_facility: row.remote_facility,
            remote_facility_icao_code: row.remote_facility_icao_code,
            sector_facility: row.sector_facility,
            sector_facility_icao_code: row.sector_facility_icao_code,
            sectorization: row.sectorization,
        }
    }
}

impl From<sql::EnrouteCommunication> for Communication {
    fn from(row: sql::EnrouteCommunication) -> Self {
        Self {
            area_code: row.area_code,
            communication_type: row.communication_type,
            airport_ident: None,
            fir_rdo_ident: Some(row.fir_rdo_ident),
            frequency: row.communication_frequency,
            frequency_units: row.frequency_units,
            callsign: row.callsign,
            name: None,
            location: Coordinates {
                lat: row.latitude,
                long: row.longitude,
            },
            remote_facility: row.remote_facility,
            remote_facility_icao_code: row.remote_facility_icao_code,
            ..Default::default()
        }
    }
}
