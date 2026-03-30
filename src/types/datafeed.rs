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
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataFeed {
    pub general: GeneralInfo,
    pub pilots: Vec<Pilot>,
    pub controllers: Vec<Controller>,
    pub atis: Vec<Atis>,
    pub servers: Vec<Server>,
    pub prefiles: Vec<Prefile>,
    pub facilities: Vec<FacilityInfo>,
    pub ratings: Vec<RatingInfo>,
    pub pilot_ratings: Vec<PilotRatingInfo>,
    pub military_ratings: Vec<MilitaryRatingInfo>,
}

/// General metadata about the current data feed snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GeneralInfo {
    pub version: u32,
    /// When the data feed was last generated
    #[cfg(feature = "chrono")]
    pub update_timestamp: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub update_timestamp: String,
    /// Total clients (pilots + controllers + ATIS) connected
    pub connected_clients: u32,
    /// Total unique users connected
    pub unique_users: u32,
}

/// A pilot currently connected to the VATSIM network.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Pilot {
    pub cid: CertificateId,
    pub name: String,
    pub callsign: String,
    pub server: String,
    pub pilot_rating: PilotRating,
    pub military_rating: MilitaryRating,
    pub latitude: f64,
    pub longitude: f64,
    /// Altitude in feet MSL
    pub altitude: i32,
    /// Ground speed in knots
    pub groundspeed: u32,
    pub transponder: String,
    /// Heading in degrees magnetic
    pub heading: u16,
    /// QNH in inches of mercury
    pub qnh_i_hg: f32,
    /// QNH in millibars
    pub qnh_mb: u32,
    /// Filed flight plan, if present
    pub flight_plan: Option<FlightPlan>,
    #[cfg(feature = "chrono")]
    pub logon_time: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub logon_time: String,
    #[cfg(feature = "chrono")]
    pub last_updated: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub last_updated: String,
}

/// An ATC controller currently connected to the VATSIM network.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Controller {
    pub cid: CertificateId,
    pub name: String,
    pub callsign: String,
    /// Frequency in MHz, e.g. "121.500"
    pub frequency: String,
    pub facility: Facility,
    pub rating: ControllerRating,
    pub server: String,
    /// Visual range in nautical miles
    pub visual_range: u32,
    /// ATIS/controller info lines, if any
    pub text_atis: Option<Vec<String>>,
    #[cfg(feature = "chrono")]
    pub last_updated: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub last_updated: String,
    #[cfg(feature = "chrono")]
    pub logon_time: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub logon_time: String,
}

/// An ATIS station currently broadcasting on the VATSIM network.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Atis {
    pub cid: CertificateId,
    pub name: String,
    pub callsign: String,
    /// Frequency in MHz, e.g. "121.500"
    pub frequency: String,
    pub facility: Facility,
    pub rating: ControllerRating,
    pub server: String,
    pub visual_range: u32,
    /// Current ATIS phonetic letter, e.g. "A"
    pub atis_code: Option<String>,
    pub text_atis: Option<Vec<String>>,
    #[cfg(feature = "chrono")]
    pub last_updated: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub last_updated: String,
    #[cfg(feature = "chrono")]
    pub logon_time: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub logon_time: String,
}

/// A VATSIM FSD server.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Server {
    pub ident: String,
    pub hostname_or_ip: String,
    pub location: String,
    pub name: String,
    /// Whether this server is currently accepting connections
    pub client_connections_allowed: bool,
    /// Whether this server is a sweatbox (training) server
    pub is_sweatbox: bool,
}

/// A prefiled flight plan not yet associated with an active connection
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Prefile {
    pub cid: CertificateId,
    pub name: String,
    pub callsign: String,
    pub flight_plan: FlightPlan,
    #[cfg(feature = "chrono")]
    pub last_updated: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub last_updated: String,
}

/// A filed flight plan.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FlightPlan {
    pub flight_rules: FlightRules,
    /// ICAO aircraft type and equipment suffix, e.g. "B764/H-SDE3FGHIM3RWXY/LB1"
    pub aircraft: String,
    /// FAA aircraft type, e.g. "B764/L"
    pub aircraft_faa: String,
    /// ICAO aircraft type designator only, e.g. "B764"
    pub aircraft_short: String,
    pub departure: String,
    pub arrival: String,
    pub alternate: String,
    /// Estimated off-block time, e.g. "1430"
    pub deptime: String,
    /// Estimated time enroute, e.g. "0615"
    pub enroute_time: String,
    /// Fuel endurance, e.g. "0745"
    pub fuel_time: String,
    pub remarks: String,
    pub route: String,
    pub revision_id: u32,
    pub assigned_transponder: String,
}

/// ICAO flight rules for a filed flight plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FlightRules {
    #[cfg_attr(feature = "serde", serde(rename = "I"))]
    /// Instrument flight rules.
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
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FacilityInfo {
    pub id: i8,
    pub short: String,
    pub long_name: String,
}

/// Human-readable metadata for a controller rating, as returned in the datafeed
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RatingInfo {
    pub id: i8,
    pub short_name: String,
    pub long_name: String,
}

/// Human-readable metadata for a pilot rating, as returned in the datafeed
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PilotRatingInfo {
    pub id: i8,
    pub short_name: String,
    pub long_name: String,
}

/// Human-readable metadata for a military rating, as returned in the datafeed
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MilitaryRatingInfo {
    pub id: i8,
    pub short_name: String,
    pub long_name: String,
}
