use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum IfrCapability {
    #[serde(rename = "Y")]
    Yes,
    // Never used, for linting
    #[serde(rename = "N")]
    No,
    #[default]
    #[serde(rename = "U")]
    Unknown,
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
    #[serde(rename = "U")]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AirwayLevel {
    #[serde(rename = "B")]
    Both,
    #[serde(rename = "H")]
    High,
    #[serde(rename = "L")]
    Low,
    #[serde(rename = "U")]
    Unknown,
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
    #[serde(rename = "E")]
    Either,
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
    #[serde(rename = "U")]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum SpeedDescriptor {
    #[serde(rename = "@")]
    Mandatory,
    #[serde(rename = "+")]
    Minimum,
    #[serde(rename = "-")]
    Maximum,
    #[serde(rename = "U")]
    Unknown,
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ApproachType {
    #[serde(rename = "B")]
    LocBackcourse,
    #[serde(rename = "D")]
    VorDme,
    #[serde(rename = "F")]
    Fms,
    #[serde(rename = "G")]
    Igs,
    #[serde(rename = "I")]
    Ils,
    #[serde(rename = "J")]
    Gls,
    #[serde(rename = "L")]
    Loc,
    #[serde(rename = "M")]
    Mls,
    #[serde(rename = "N")]
    Ndb,
    #[serde(rename = "P")]
    Gps,
    #[serde(rename = "Q")]
    NdbDme,
    #[serde(rename = "R")]
    Rnav,
    #[serde(rename = "S")]
    Vortac,
    #[serde(rename = "T")]
    Tacan,
    #[serde(rename = "U")]
    Sdf,
    #[serde(rename = "V")]
    Vor,
    #[serde(rename = "W")]
    MlsTypeA,
    #[serde(rename = "X")]
    Lda,
    #[serde(rename = "Y")]
    MlsTypeBC,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ControlledAirspaceType {
    #[serde(rename = "A")]
    ClassC,
    #[serde(rename = "C")]
    ControlArea,
    #[serde(rename = "K")]
    TmaOrTca,
    #[serde(rename = "M")]
    IcaoTerminalControlArea,
    #[serde(rename = "Q")]
    MilitaryControlZone,
    #[serde(rename = "R")]
    RadarZone,
    #[serde(rename = "T")]
    ClassB,
    #[serde(rename = "W")]
    TerminalControlArea,
    #[serde(rename = "X")]
    TerminalArea,
    #[serde(rename = "Y")]
    TerminalRadarServiceArea,
    #[serde(rename = "Z")]
    ClassD,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum RestrictiveAirspaceType {
    #[serde(rename = "S")]
    AdvisoryArea,
    #[serde(rename = "A")]
    Alert,
    #[serde(rename = "B")]
    BufferZone,
    #[serde(rename = "C")]
    Caution,
    #[serde(rename = "F")]
    CrossBorderArea,
    #[serde(rename = "D")]
    Danger,
    #[serde(rename = "M")]
    Military,
    #[serde(rename = "P")]
    Prohibited,
    #[serde(rename = "R")]
    Restricted,
    #[serde(rename = "G")]
    TemporaryReserveArea,
    #[serde(rename = "T")]
    Training,
    #[serde(rename = "K")]
    TemporarySegregatedArea,
    #[serde(rename = "W")]
    Warning,
    #[serde(rename = "U")]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum CommunicationType {
    #[serde(rename = "ACC")]
    AreaControlCenter,
    #[serde(rename = "ACP")]
    AirliftCommandPost,
    #[serde(rename = "AIR")]
    AirToAir,
    #[serde(rename = "APP")]
    ApproachControl,
    #[serde(rename = "ARR")]
    ArrivalControl,
    #[serde(rename = "ASO")]
    AutomaticSurfaceObservingSystem,
    #[serde(rename = "ATI")]
    AutomaticTerminalInformationServices,
    #[serde(rename = "AWI")]
    AirportWeatherInformationBroadcast,
    #[serde(rename = "AWO")]
    AutomaticWeatherObservingBroadcast,
    #[serde(rename = "AWS")]
    AerodromeWeatherInformationService,
    #[serde(rename = "CLD")]
    ClearanceDelivery,
    #[serde(rename = "CPT")]
    ClearancePreTaxi,
    #[serde(rename = "CTA")]
    ControlArea,
    #[serde(rename = "CTL")]
    Control,
    #[serde(rename = "DEP")]
    DepartureControl,
    #[serde(rename = "DIR")]
    Director,
    #[serde(rename = "EFS")]
    EnrouteFlightAdvisoryService,
    #[serde(rename = "EMR")]
    Emergency,
    #[serde(rename = "FSS")]
    FlightServiceStation,
    #[serde(rename = "GCO")]
    GroundCommOutlet,
    #[serde(rename = "GND")]
    GroundControl,
    #[serde(rename = "GET")]
    GateControl,
    #[serde(rename = "HEL")]
    HelicopterFrequency,
    #[serde(rename = "INF")]
    Information,
    #[serde(rename = "MIL")]
    MilitaryFrequency,
    #[serde(rename = "MUL")]
    Multicom,
    #[serde(rename = "OPS")]
    Operations,
    #[serde(rename = "PAL")]
    PilotActivatedLighting,
    #[serde(rename = "RDO")]
    Radio,
    #[serde(rename = "RDR")]
    Radar,
    #[serde(rename = "RFS")]
    RemoteFlightServiceStation,
    #[serde(rename = "RMP")]
    RampTaxiControl,
    #[serde(rename = "RSA")]
    AirportRadarServiceArea,
    #[serde(rename = "TCA")]
    /// Terminal Control Area (TCA)
    Tca,
    #[serde(rename = "TMA")]
    /// Terminal Control Area (TMA)
    Tma,
    #[serde(rename = "TML")]
    Terminal,
    #[serde(rename = "TRS")]
    TerminalRadarServiceArea,
    #[serde(rename = "TWE")]
    TranscriberWeatherBroadcast,
    #[serde(rename = "TWR")]
    Tower,
    #[serde(rename = "UAC")]
    UpperAreaControl,
    // Never used, for linting
    #[default]
    #[serde(rename = "UNI")]
    Unicom,
    #[serde(rename = "VOL")]
    Volmet,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum FrequencyUnits {
    #[serde(rename = "H")]
    High,
    #[serde(rename = "V")]
    VeryHigh,
    #[serde(rename = "U")]
    UltraHigh,
    // Never used, for linting
    #[default]
    #[serde(rename = "C")]
    /// Communication channel for 8.33 kHz spacing
    CommChannel,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum ApproachTypeIdentifier {
    #[serde(rename = "LPV")]
    LocalizerPerformanceVerticalGuidance,
    // Never used, for linting
    #[default]
    #[serde(rename = "LP")]
    LocalizerPerformance,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum TrafficPattern {
    #[serde(rename = "L")]
    Left,
    #[serde(rename = "R")]
    Right,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum RunwayLights {
    #[serde(rename = "Y")]
    Yes,
    #[serde(rename = "N")]
    No,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum RunwaySurface {
    #[serde(rename = "GRVL")]
    Gravel,
    #[serde(rename = "UNPV")]
    Unpaved,
    #[serde(rename = "ASPH")]
    Asphalt,
    #[serde(rename = "TURF")]
    Turf,
    #[serde(rename = "DIRT")]
    Dirt,
    #[serde(rename = "CONC")]
    Concrete,
    #[serde(rename = "WATE")]
    Water,
    #[serde(rename = "SAND")]
    Sand,
    #[serde(rename = "CORL")]
    Coral,
    #[serde(rename = "PAVD")]
    Paved,
    #[serde(rename = "GRAS")]
    Grass,
    #[serde(rename = "BITU")]
    Bitumen,
    #[serde(rename = "PLNG")]
    Planking,
    #[serde(rename = "CLAY")]
    Clay,
    #[serde(rename = "ICE")]
    Ice,
    #[serde(rename = "SILT")]
    Silt,
    #[serde(rename = "LATE")]
    Laterite,
    #[serde(rename = "TARM")]
    Tarmac,
    #[serde(rename = "MACA")]
    Macadam,
    #[serde(rename = "SELD")]
    Sealed,
    #[serde(rename = "SOIL")]
    Soil,
    #[serde(rename = "BRCK")]
    Brick,
    #[serde(rename = "UNKN")]
    Unknown,
    #[serde(rename = "MATS")]
    Mats,
    #[serde(rename = "SNOW")]
    Snow,
    #[serde(rename = "TRTD")]
    Treated,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ProcedureTypeApproved {
    #[serde(rename = "A")]
    Yes,
    #[serde(rename = "N")]
    No,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum AuthorizationRequired {
    #[serde(rename = "Y")]
    Yes,
    #[serde(rename = "N")]
    No,
}
