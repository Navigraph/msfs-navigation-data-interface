use serde::Deserialize;

use crate::enums::{
    AirwayDirection,
    AirwayLevel,
    AirwayRouteType,
    AltitudeDescriptor,
    ApproachTypeIdentifier,
    CommunicationType,
    FrequencyUnits,
    IfrCapability,
    LegType,
    RunwayLights,
    RunwaySurface,
    RunwaySurfaceCode,
    SpeedDescriptor,
    TrafficPattern,
    TurnDirection,
};

#[derive(Deserialize, Debug)]
pub struct AirportCommunication {
    pub airport_identifier: String,
    pub area_code: String,
    pub callsign: Option<String>,
    pub communication_frequency: f64,
    pub communication_type: CommunicationType,
    pub frequency_units: FrequencyUnits,
    pub guard_transmit: Option<String>, // new
    pub icao_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub narritive: Option<String>,                 // new
    pub remote_facility_icao_code: Option<String>, // new
    pub remote_facility: Option<String>,           // new
    pub sector_facility_icao_code: Option<String>, // new
    pub sector_facility: Option<String>,           // new
    pub sectorization: Option<String>,             // new
    pub service_indicator: Option<String>,
    pub time_of_operation_1: Option<String>, // new
    pub time_of_operation_2: Option<String>, // new
    pub time_of_operation_3: Option<String>, // new
    pub time_of_operation_4: Option<String>, // new
    pub time_of_operation_5: Option<String>, // new
    pub time_of_operation_6: Option<String>, // new
    pub time_of_operation_7: Option<String>, // new
}

#[derive(Deserialize, Debug)]
#[allow(unused_variables)]
pub struct AirportMsa {
    pub area_code: Option<String>,
    pub icao_code: Option<String>,
    pub airport_identifier: Option<String>,
    pub msa_center: Option<String>,
    pub msa_center_latitude: Option<f64>,
    pub msa_center_longitude: Option<f64>,
    pub magnetic_true_indicator: Option<String>,
    pub multiple_code: Option<String>,
    pub radius_limit: Option<f64>,
    pub sector_bearing_1: Option<f64>,
    pub sector_altitude_1: Option<f64>,
    pub sector_bearing_2: Option<f64>,
    pub sector_altitude_2: Option<f64>,
    pub sector_bearing_3: Option<f64>,
    pub sector_altitude_3: Option<f64>,
    pub sector_bearing_4: Option<f64>,
    pub sector_altitude_4: Option<f64>,
    pub sector_bearing_5: Option<f64>,
    pub sector_altitude_5: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Airports {
    pub airport_identifier: String,
    pub airport_name: String,
    pub airport_ref_latitude: f64,
    pub airport_ref_longitude: f64,
    pub airport_type: String,
    pub area_code: String,
    pub ata_iata_code: Option<String>,
    pub city: Option<String>,
    pub continent: Option<String>,
    pub country: Option<String>,
    pub country_3letter: Option<String>,
    pub elevation: f64,
    pub icao_code: String,
    pub ifr_capability: Option<IfrCapability>,
    pub longest_runway_surface_code: RunwaySurfaceCode,
    pub magnetic_variation: Option<f64>,
    pub speed_limit: Option<f64>,
    pub speed_limit_altitude: Option<String>,
    pub state: Option<String>,
    pub state_2letter: Option<String>,
    pub transition_altitude: Option<f64>,
    pub transition_level: Option<f64>,
    pub airport_identifier_3letter: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CruisingTables {
    pub cruise_table_identifier: Option<String>,
    pub seqno: Option<f64>,
    pub course_from: Option<f64>,
    pub course_to: Option<f64>,
    pub mag_true: Option<String>,
    pub cruise_level_from1: Option<f64>,
    pub vertical_separation1: Option<f64>,
    pub cruise_level_to1: Option<f64>,
    pub cruise_level_from2: Option<f64>,
    pub vertical_separation2: Option<f64>,
    pub cruise_level_to2: Option<f64>,
    pub cruise_level_from3: Option<f64>,
    pub vertical_separation3: Option<f64>,
    pub cruise_level_to3: Option<f64>,
    pub cruise_level_from4: Option<f64>,
    pub vertical_separation4: Option<f64>,
    pub cruise_level_to4: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct EnrouteAirwayRestriction {
    pub area_code: Option<String>,
    pub route_identifier: Option<String>,
    pub restriction_identifier: Option<f64>,
    pub restriction_type: Option<String>,
    pub start_waypoint_identifier: Option<String>,
    pub start_waypoint_latitude: Option<f64>,
    pub start_waypoint_longitude: Option<f64>,
    pub end_waypoint_identifier: Option<String>,
    pub end_waypoint_latitude: Option<f64>,
    pub end_waypoint_longitude: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub units_of_altitude: Option<String>,
    pub restriction_altitude1: Option<f64>,
    pub block_indicator1: Option<String>,
    pub restriction_altitude2: Option<f64>,
    pub block_indicator2: Option<String>,
    pub restriction_altitude3: Option<f64>,
    pub block_indicator3: Option<String>,
    pub restriction_altitude4: Option<f64>,
    pub block_indicator4: Option<String>,
    pub restriction_altitude5: Option<f64>,
    pub block_indicator5: Option<String>,
    pub restriction_altitude6: Option<f64>,
    pub block_indicator6: Option<String>,
    pub restriction_altitude7: Option<f64>,
    pub block_indicator7: Option<String>,
    pub restriction_notes: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct EnrouteAirways {
    pub area_code: String,
    pub crusing_table_identifier: Option<String>,
    pub direction_restriction: Option<AirwayDirection>,
    pub flightlevel: Option<AirwayLevel>,
    pub icao_code: Option<String>,
    pub inbound_course: Option<f64>,
    pub inbound_distance: Option<f64>,
    pub maximum_altitude: Option<f64>,
    pub minimum_altitude1: Option<f64>,
    pub minimum_altitude2: Option<f64>,
    pub outbound_course: Option<f64>,
    pub route_identifier: Option<String>,
    pub route_identifier_postfix: Option<String>,
    pub route_type: Option<AirwayRouteType>,
    pub seqno: Option<f64>,
    pub waypoint_description_code: Option<String>,
    pub waypoint_identifier: Option<String>,
    pub waypoint_latitude: Option<f64>,
    pub waypoint_longitude: Option<f64>,
    pub waypoint_ref_table: String,
}

#[derive(Deserialize, Debug)]
pub struct EnrouteCommunication {
    pub area_code: String,
    pub callsign: Option<String>,
    pub communication_frequency: f64,
    pub communication_type: CommunicationType,
    pub fir_rdo_ident: String,
    pub fir_uir_indicator: Option<String>,
    pub frequency_units: FrequencyUnits,
    pub latitude: f64,
    pub longitude: f64,
    pub remote_facility_icao_code: Option<String>, // new
    pub remote_facility: Option<String>,           // new
    pub remote_name: Option<String>,
    pub service_indicator: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct FirUir {
    pub area_code: Option<String>,
    pub fir_uir_identifier: Option<String>,
    pub fir_uir_address: Option<String>,
    pub fir_uir_name: Option<String>,
    pub fir_uir_indicator: Option<String>,
    pub seqno: Option<f64>,
    pub boundary_via: Option<String>,
    pub adjacent_fir_identifier: Option<String>,
    pub adjacent_uir_identifier: Option<String>,
    pub reporting_units_speed: Option<f64>,
    pub reporting_units_altitude: Option<f64>,
    pub fir_uir_latitude: Option<f64>,
    pub fir_uir_longitude: Option<f64>,
    pub arc_origin_latitude: Option<f64>,
    pub arc_origin_longitude: Option<f64>,
    pub arc_distance: Option<f64>,
    pub arc_bearing: Option<f64>,
    pub fir_upper_limit: Option<String>,
    pub uir_lower_limit: Option<String>,
    pub uir_upper_limit: Option<String>,
    pub cruise_table_identifier: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Gls {
    pub area_code: String,
    pub airport_identifier: String,
    pub icao_code: String,
    pub gls_ref_path_identifier: String,
    pub gls_category: String,
    pub gls_channel: f64,
    pub runway_identifier: String,
    pub gls_approach_bearing: f64,
    pub station_latitude: f64,
    pub station_longitude: f64,
    pub gls_station_ident: String,
    pub gls_approach_slope: f64,
    /// Yes its spelt wrong in the database
    pub magentic_variation: f64,
    pub station_elevation: f64,
    pub station_type: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GridMora {
    pub starting_latitude: Option<f64>,
    pub starting_longitude: Option<f64>,
    pub mora01: Option<String>,
    pub mora02: Option<String>,
    pub mora03: Option<String>,
    pub mora04: Option<String>,
    pub mora05: Option<String>,
    pub mora06: Option<String>,
    pub mora07: Option<String>,
    pub mora08: Option<String>,
    pub mora09: Option<String>,
    pub mora10: Option<String>,
    pub mora11: Option<String>,
    pub mora12: Option<String>,
    pub mora13: Option<String>,
    pub mora14: Option<String>,
    pub mora15: Option<String>,
    pub mora16: Option<String>,
    pub mora17: Option<String>,
    pub mora18: Option<String>,
    pub mora19: Option<String>,
    pub mora20: Option<String>,
    pub mora21: Option<String>,
    pub mora22: Option<String>,
    pub mora23: Option<String>,
    pub mora24: Option<String>,
    pub mora25: Option<String>,
    pub mora26: Option<String>,
    pub mora27: Option<String>,
    pub mora28: Option<String>,
    pub mora29: Option<String>,
    pub mora30: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Header {
    pub creator: String,
    pub cycle: String,
    pub data_provider: String,
    pub dataset_version: String,
    pub dataset: String,
    pub effective_fromto: String,
    pub parsed_at: String,
    pub revision: String,
}

#[derive(Deserialize, Debug)]
pub struct Holdings {
    pub area_code: Option<String>,
    pub region_code: Option<String>,
    pub icao_code: Option<String>,
    pub waypoint_identifier: Option<String>,
    pub holding_name: Option<String>,
    pub waypoint_latitude: Option<f64>,
    pub waypoint_longitude: Option<f64>,
    pub duplicate_identifier: Option<f64>,
    pub inbound_holding_course: Option<f64>,
    pub turn_direction: Option<String>,
    pub leg_length: Option<f64>,
    pub leg_time: Option<f64>,
    pub minimum_altitude: Option<f64>,
    pub maximum_altitude: Option<f64>,
    pub holding_speed: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Procedures {
    pub airport_identifier: String,
    pub altitude_description: Option<AltitudeDescriptor>,
    pub altitude1: Option<f64>,
    pub altitude2: Option<f64>,
    pub arc_radius: Option<f64>,
    pub area_code: String,
    pub authorization_required: Option<String>,    // new
    pub center_waypoint_icao_code: Option<String>, // new
    pub center_waypoint_latitude: Option<f64>,
    pub center_waypoint_longitude: Option<f64>,
    pub center_waypoint_ref_table: String, // new
    pub center_waypoint: Option<String>,
    pub course_flag: Option<String>, // new
    pub course: Option<f64>,         // new
    pub distance_time: Option<f64>,
    pub path_termination: LegType,
    pub procedure_identifier: String,
    pub recommended_navaid_icao_code: Option<String>, // new
    pub recommended_navaid_latitude: Option<f64>,
    pub recommended_navaid_longitude: Option<f64>,
    pub recommended_navaid_ref_table: String, // new
    pub recommended_navaid: Option<String>,
    pub rho: Option<f64>,
    pub rnp: Option<f64>,
    pub route_distance_holding_distance_time: Option<String>,
    pub route_type: String,
    pub seqno: f64,
    pub speed_limit_description: Option<SpeedDescriptor>,
    pub speed_limit: Option<f64>,
    pub theta: Option<f64>,
    pub transition_altitude: Option<f64>,
    pub transition_identifier: Option<String>,
    pub turn_direction: Option<TurnDirection>,
    pub vertical_angle: Option<f64>,
    pub waypoint_description_code: Option<String>,
    pub waypoint_icao_code: Option<String>,
    pub waypoint_identifier: Option<String>,
    pub waypoint_latitude: Option<f64>,
    pub waypoint_longitude: Option<f64>,
    pub waypoint_ref_table: String, // new
}

#[derive(Deserialize, Debug)]
pub struct LocalizerMarker {
    pub area_code: String,
    pub icao_code: String,
    pub airport_identifier: String,
    pub runway_identifier: String,
    pub llz_identifier: String,
    pub marker_identifier: String,
    pub marker_type: String,
    pub marker_latitude: f64,
    pub marker_longitude: f64,
}

#[derive(Deserialize, Debug)]
pub struct LocalizersGlideslopes {
    pub area_code: Option<String>,
    pub icao_code: Option<String>,
    pub airport_identifier: String,
    pub runway_identifier: Option<String>,
    pub llz_identifier: String,
    pub llz_latitude: Option<f64>,
    pub llz_longitude: Option<f64>,
    pub llz_frequency: Option<f64>,
    pub llz_bearing: Option<f64>,
    pub llz_width: Option<f64>,
    pub ils_mls_gls_category: Option<String>,
    pub gs_latitude: Option<f64>,
    pub gs_longitude: Option<f64>,
    pub gs_angle: Option<f64>,
    pub gs_elevation: Option<f64>,
    pub station_declination: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Pathpoints {
    pub airport_icao_code: String,
    pub airport_identifier: String,
    pub approach_performance_designator: Option<String>, // new
    pub approach_procedure_ident: String,
    pub approach_type_identifier: ApproachTypeIdentifier,
    pub area_code: String,
    pub course_width_at_threshold: f64,
    pub flight_path_alignment_latitude: f64,
    pub flight_path_alignment_longitude: f64,
    pub glide_path_angle: f64,
    pub gnss_channel_number: f64,
    pub hal: f64,
    pub landing_threshold_latitude: f64,
    pub landing_threshold_longitude: f64,
    pub length_offset: Option<f64>,
    pub ltp_ellipsoid_height: f64,
    pub operations_type: Option<f64>, // new
    pub path_point_tch: f64,
    pub reference_path_data_selector: Option<f64>, // new
    pub reference_path_identifier: String,
    pub route_indicator: Option<String>,
    pub runway_identifier: String,
    pub sbas_service_provider_identifier: f64,
    pub tch_units_indicator: String,
    pub val: f64,
}

#[derive(Deserialize, Debug)]
pub struct Runways {
    pub airport_identifier: String,
    pub area_code: Option<String>,
    pub displaced_threshold_distance: Option<f64>,
    pub icao_code: Option<String>,
    pub landing_threshold_elevation: f64,
    pub llz_identifier: Option<String>,
    pub llz_mls_gls_category: Option<String>,
    pub part_time_lights: Option<String>,
    pub runway_gradient: Option<f64>,
    pub runway_identifier: String,
    pub runway_latitude: Option<f64>,
    pub runway_length: f64,
    pub runway_lights: Option<RunwayLights>, // new
    pub runway_longitude: Option<f64>,
    pub runway_magnetic_bearing: Option<f64>,
    pub runway_true_bearing: Option<f64>,
    pub runway_width: f64,
    pub surface_code: Option<RunwaySurface>,
    pub threshold_crossing_height: Option<f64>,
    pub traffic_pattern: Option<TrafficPattern>, // new
}

#[derive(Deserialize, Debug)]
pub struct Sids {
    pub area_code: Option<String>,
    pub airport_identifier: Option<String>,
    pub procedure_identifier: Option<String>,
    pub route_type: Option<String>,
    pub transition_identifier: Option<String>,
    pub seqno: Option<f64>,
    pub waypoint_icao_code: Option<String>,
    pub waypoint_identifier: Option<String>,
    pub waypoint_latitude: Option<f64>,
    pub waypoint_longitude: Option<f64>,
    pub waypoint_description_code: Option<String>,
    pub turn_direction: Option<String>,
    pub rnp: Option<f64>,
    pub path_termination: Option<String>,
    pub recommanded_navaid: Option<String>,
    pub recommanded_navaid_latitude: Option<f64>,
    pub recommanded_navaid_longitude: Option<f64>,
    pub arc_radius: Option<f64>,
    pub theta: Option<f64>,
    pub rho: Option<f64>,
    pub magnetic_course: Option<f64>,
    pub route_distance_holding_distance_time: Option<String>,
    pub distance_time: Option<f64>,
    pub altitude_description: Option<String>,
    pub altitude1: Option<f64>,
    pub altitude2: Option<f64>,
    pub transition_altitude: Option<f64>,
    pub speed_limit_description: Option<String>,
    pub speed_limit: Option<f64>,
    pub vertical_angle: Option<f64>,
    pub center_waypoint: Option<String>,
    pub center_waypoint_latitude: Option<f64>,
    pub center_waypoint_longitude: Option<f64>,
    pub aircraft_category: Option<String>,
    pub id: Option<String>,
    pub recommanded_id: Option<String>,
    pub center_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct NdbNavaids {
    pub area_code: String,
    pub airport_identifier: Option<String>,
    pub icao_code: String,
    pub ndb_identifier: String,
    pub ndb_name: String,
    pub ndb_frequency: f64,
    pub navaid_class: String,
    pub ndb_latitude: f64,
    pub ndb_longitude: f64,
    pub range: f64,
}

#[derive(Deserialize, Debug)]
pub struct Waypoints {
    pub area_code: String,
    pub continent: Option<String>,
    pub country: Option<String>,
    pub icao_code: Option<String>,
    pub magnetic_varation: Option<f64>,
    pub region_code: Option<String>,
    pub waypoint_identifier: String,
    pub waypoint_latitude: f64,
    pub waypoint_longitude: f64,
    pub waypoint_name: String,
    pub waypoint_type: String,
    pub waypoint_usage: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct VhfNavaids {
    pub airport_identifier: Option<String>,
    pub area_code: String,
    pub continent: Option<String>,
    pub country: Option<String>,
    pub dme_elevation: Option<f64>,
    pub dme_ident: Option<String>,
    pub dme_latitude: Option<f64>,
    pub dme_longitude: Option<f64>,
    pub icao_code: Option<String>,
    pub ilsdme_bias: Option<f64>,
    pub magnetic_variation: Option<f64>,
    pub navaid_class: String,
    pub navaid_frequency: f64,
    pub navaid_identifier: String,
    pub navaid_latitude: f64,
    pub navaid_longitude: f64,
    pub navaid_name: String,
    pub range: f64,
    pub station_declination: Option<f64>,
}
