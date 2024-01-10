use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum IfrCapability {
    #[serde(rename = "Y")]
    Yes,
    #[serde(rename = "N")]
    No,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RunwaySurfaceCode {
    #[serde(rename = "H")]
    Hard,
    #[serde(rename = "S")]
    Soft,
    #[serde(rename = "W")]
    Water,
    #[serde(rename = "U")]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AirwayRouteType {
    #[serde(rename = "C")]
    Control,
    #[serde(rename = "D")]
    DirectRoute,
    #[serde(rename = "H")]
    HelicopterRoute,
    #[serde(rename = "O")]
    OfficialDesignatedAirwaysExpectRnavAirways,
    #[serde(rename = "R")]
    RnavAirways,
    #[serde(rename = "S")]
    UndesignatedAtsRoute,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AirwayLevel {
    #[serde(rename = "B")]
    Both,
    #[serde(rename = "H")]
    High,
    #[serde(rename = "L")]
    Low,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AirwayDirection {
    #[serde(rename = "F")]
    Forward,
    #[serde(rename = "B")]
    Backward,
}
