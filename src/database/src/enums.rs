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

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum TurnDirection {
    #[serde(rename = "L")]
    Left,
    #[serde(rename = "R")]
    Right,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum AltitudeDescriptor {
    #[serde(rename = "@")]
    AtAlt1,
    #[serde(rename = "+")]
    AtOrAboveAlt1,
    #[serde(rename = "-")]
    AtOrBelowAlt1,
    #[serde(rename = "B")]
    BetweenAlt1Alt2,
    #[serde(rename = "C")]
    AtOrAboveAlt2,
    #[serde(rename = "G")]
    AtAlt1GsMslAlt2,
    #[serde(rename = "H")]
    AtOrAboveAlt1GsMslAlt2,
    #[serde(rename = "I")]
    AtAlt1GsInterceptAlt2,
    #[serde(rename = "J")]
    AtOrAboveAlt1GsInterceptAlt2,
    #[serde(rename = "V")]
    AtOrAboveAlt1AngleAlt2,
    #[serde(rename = "X")]
    AtAlt1AngleAlt2,
    #[serde(rename = "Y")]
    AtOrBelowAlt1AngleAlt2,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum SpeedDescriptor {
    #[serde(rename = "@")]
    Mandatory,
    #[serde(rename = "+")]
    Minimum,
    #[serde(rename = "-")]
    Maximum,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum LegType {
    IF,
    TF,
    CF,
    DF,
    FA,
    FC,
    FD,
    FM,
    CA,
    CD,
    CI,
    CR,
    RF,
    AF,
    VA,
    VD,
    VI,
    VM,
    VR,
    PI,
    HA,
    HF,
    HM,
}
