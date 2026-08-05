//! Types for the [VATSIM data feed](https://vatsim.dev/api/data-api/get-network-data).
//!
//! The top-level [`DataFeed`] struct mirrors the JSON response and contains
//! vectors of [`Pilot`], [`Controller`], [`Atis`], [`Server`], and [`Prefile`]
//! entries, plus metadata lookup tables.

use crate::types::{CertificateId, ControllerRating, Facility, MilitaryRating, PilotRating};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

/// Top-level response from the [VATSIM data feed](https://vatsim.dev/api/data-api/get-network-data).
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataFeed {
    /// Metadata about this snapshot.
    pub general: GeneralInfo,
    /// Pilots currently connected to the network.
    pub pilots: Vec<Pilot>,
    /// Controllers currently connected to the network.
    pub controllers: Vec<Controller>,
    /// ATIS stations currently broadcasting.
    pub atis: Vec<Atis>,
    /// FSD servers available to connect to.
    pub servers: Vec<Server>,
    /// Flight plans filed without an active connection.
    pub prefiles: Vec<Prefile>,
    /// Lookup table mapping facility IDs to names.
    pub facilities: Vec<FacilityInfo>,
    /// Lookup table mapping controller rating IDs to names.
    pub ratings: Vec<RatingInfo>,
    /// Lookup table mapping pilot rating IDs to names.
    pub pilot_ratings: Vec<PilotRatingInfo>,
    /// Lookup table mapping military rating IDs to names.
    pub military_ratings: Vec<MilitaryRatingInfo>,
}

/// General metadata about the current data feed snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GeneralInfo {
    /// Data feed schema version, currently always 3.
    pub version: u32,
    /// When the data feed was last generated
    #[cfg(feature = "chrono")]
    pub update_timestamp: DateTime<Utc>,
    /// When the data feed was last generated
    #[cfg(not(feature = "chrono"))]
    pub update_timestamp: String,
    /// Total clients (pilots + controllers + ATIS) connected
    pub connected_clients: u32,
    /// Total unique users connected
    pub unique_users: u32,
}

/// A pilot currently connected to the VATSIM network.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Pilot {
    /// The pilot's VATSIM certificate ID.
    pub cid: CertificateId,
    /// The pilot's full name, or their CID if they hide it.
    pub name: String,
    /// Callsign the pilot is connected as, e.g. "AUA123".
    pub callsign: String,
    /// Ident of the FSD server the pilot is connected to.
    pub server: String,
    /// The pilot's civilian rating.
    pub pilot_rating: PilotRating,
    /// The pilot's military rating.
    pub military_rating: MilitaryRating,
    /// Latitude in decimal degrees.
    pub latitude: f64,
    /// Longitude in decimal degrees.
    pub longitude: f64,
    /// Altitude in feet MSL
    pub altitude: i32,
    /// Ground speed in knots
    pub groundspeed: u32,
    /// Squawk code as four octal digits, e.g. "2000".
    pub transponder: String,
    /// Heading in degrees magnetic
    pub heading: u16,
    /// QNH in inches of mercury
    pub qnh_i_hg: f32,
    /// QNH in millibars
    pub qnh_mb: u32,
    /// Filed flight plan, if present
    pub flight_plan: Option<FlightPlan>,
    /// When the pilot connected to the network.
    #[cfg(feature = "chrono")]
    pub logon_time: DateTime<Utc>,
    /// When the pilot connected to the network.
    #[cfg(not(feature = "chrono"))]
    pub logon_time: String,
    /// When this entry was last refreshed.
    #[cfg(feature = "chrono")]
    pub last_updated: DateTime<Utc>,
    /// When this entry was last refreshed.
    #[cfg(not(feature = "chrono"))]
    pub last_updated: String,
}

/// An ATC controller currently connected to the VATSIM network.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Controller {
    /// The controller's VATSIM certificate ID.
    pub cid: CertificateId,
    /// The controller's full name, or their CID if they hide it.
    pub name: String,
    /// Callsign the controller is connected as, e.g. "LOWW_TWR".
    pub callsign: String,
    /// Frequency in MHz, e.g. "121.500"
    pub frequency: String,
    /// Facility type, derived from the callsign suffix.
    pub facility: Facility,
    /// The controller's ATC rating.
    pub rating: ControllerRating,
    /// Ident of the FSD server the controller is connected to.
    pub server: String,
    /// Visual range in nautical miles
    pub visual_range: u32,
    /// ATIS/controller info lines, if any
    pub text_atis: Option<Vec<String>>,
    /// When this entry was last refreshed.
    #[cfg(feature = "chrono")]
    pub last_updated: DateTime<Utc>,
    /// When this entry was last refreshed.
    #[cfg(not(feature = "chrono"))]
    pub last_updated: String,
    /// When the controller connected to the network.
    #[cfg(feature = "chrono")]
    pub logon_time: DateTime<Utc>,
    /// When the controller connected to the network.
    #[cfg(not(feature = "chrono"))]
    pub logon_time: String,
}

/// An ATIS station currently broadcasting on the VATSIM network.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Atis {
    /// The controller's VATSIM certificate ID.
    pub cid: CertificateId,
    /// The controller's full name, or their CID if they hide it.
    pub name: String,
    /// Callsign of the ATIS connection, e.g. "LOWW_ATIS".
    pub callsign: String,
    /// Frequency in MHz, e.g. "121.500"
    pub frequency: String,
    /// Facility type, derived from the callsign suffix.
    pub facility: Facility,
    /// The controller's ATC rating.
    pub rating: ControllerRating,
    /// Ident of the FSD server the ATIS is connected to.
    pub server: String,
    /// Visual range in nautical miles.
    pub visual_range: u32,
    /// Current ATIS phonetic letter, e.g. "A"
    pub atis_code: Option<String>,
    /// The ATIS text, split into lines.
    pub text_atis: Option<Vec<String>>,
    /// When this entry was last refreshed.
    #[cfg(feature = "chrono")]
    pub last_updated: DateTime<Utc>,
    /// When this entry was last refreshed.
    #[cfg(not(feature = "chrono"))]
    pub last_updated: String,
    /// When the ATIS connected to the network.
    #[cfg(feature = "chrono")]
    pub logon_time: DateTime<Utc>,
    /// When the ATIS connected to the network.
    #[cfg(not(feature = "chrono"))]
    pub logon_time: String,
}

/// A VATSIM FSD server.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Server {
    /// Short server identifier used in pilot and controller entries, e.g. "GERMANY-1".
    pub ident: String,
    /// Hostname or IP address clients connect to.
    pub hostname_or_ip: String,
    /// Geographic location of the server.
    pub location: String,
    /// Human-readable server name.
    pub name: String,
    /// Whether this server is currently accepting connections
    pub client_connections_allowed: bool,
    /// Whether this server is a sweatbox (training) server
    pub is_sweatbox: bool,
}

/// A prefiled flight plan not yet associated with an active connection
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Prefile {
    /// The filer's VATSIM certificate ID.
    pub cid: CertificateId,
    /// The filer's full name, or their CID if they hide it.
    pub name: String,
    /// Callsign the flight plan was filed for.
    pub callsign: String,
    /// The filed flight plan.
    pub flight_plan: FlightPlan,
    /// When this entry was last refreshed.
    #[cfg(feature = "chrono")]
    pub last_updated: DateTime<Utc>,
    /// When this entry was last refreshed.
    #[cfg(not(feature = "chrono"))]
    pub last_updated: String,
}

/// A filed flight plan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FlightPlan {
    /// Flight rules the flight plan was filed under.
    pub flight_rules: FlightRules,
    /// ICAO aircraft type and equipment suffix, e.g. "B764/H-SDE3FGHIM3RWXY/LB1"
    pub aircraft: String,
    /// FAA aircraft type, e.g. "B764/L"
    pub aircraft_faa: String,
    /// ICAO aircraft type designator only, e.g. "B764"
    pub aircraft_short: String,
    /// ICAO code of the departure aerodrome.
    pub departure: String,
    /// ICAO code of the destination aerodrome.
    pub arrival: String,
    /// ICAO code of the alternate aerodrome, empty if none was filed.
    pub alternate: String,
    /// Estimated off-block time, e.g. "1430"
    pub deptime: String,
    /// Estimated time enroute, e.g. "0615"
    pub enroute_time: String,
    /// Fuel endurance, e.g. "0745"
    pub fuel_time: String,
    /// Free-text remarks, including the ICAO field 18 items.
    pub remarks: String,
    /// Filed route, as a space-separated string of waypoints and airways.
    pub route: String,
    /// Revision number, incremented each time the flight plan is refiled.
    pub revision_id: u32,
    /// Squawk code assigned to this flight plan.
    pub assigned_transponder: String,
}

/// ICAO flight rules for a filed flight plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FlightRules {
    #[cfg_attr(feature = "serde", serde(rename = "I"))]
    /// Instrument flight rules.
    #[default]
    IFR,
    #[cfg_attr(feature = "serde", serde(rename = "V"))]
    /// Visual flight rules.
    VFR,
    /// IFR changing to VFR en route (composite flight plan).
    #[cfg_attr(feature = "serde", serde(rename = "Y"))]
    YIFR,
    /// VFR changing to IFR en route (composite flight plan).
    #[cfg_attr(feature = "serde", serde(rename = "Z"))]
    ZVFR,
}

/// Human-readable metadata for a facility type, as returned in the datafeed
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FacilityInfo {
    /// Numeric facility ID as used on controller and ATIS entries.
    pub id: i8,
    /// Short code, e.g. "TWR".
    pub short: String,
    /// Full name, e.g. "Tower".
    pub long: String,
}

/// Human-readable metadata for a controller rating, as returned in the datafeed
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RatingInfo {
    /// Numeric rating ID as used on controller and ATIS entries.
    pub id: i8,
    /// Short code, e.g. "C1".
    pub short: String,
    /// Full name, e.g. "Enroute".
    pub long: String,
}

/// Human-readable metadata for a pilot rating, as returned in the datafeed
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PilotRatingInfo {
    /// Numeric rating ID as used on pilot entries.
    pub id: i8,
    /// Short code, e.g. "PPL".
    pub short_name: String,
    /// Full name, e.g. "Private Pilot Licence".
    pub long_name: String,
}

/// Human-readable metadata for a military rating, as returned in the datafeed
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MilitaryRatingInfo {
    /// Numeric rating ID as used on pilot entries.
    pub id: i8,
    /// Short code, e.g. "M1".
    pub short_name: String,
    /// Full name, e.g. "Military Pilot License".
    pub long_name: String,
}
