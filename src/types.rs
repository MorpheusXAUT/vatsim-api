//! Core types shared across VATSIM APIs.
//!
//! This module contains enums and newtypes that appear in multiple VATSIM
//! endpoints: [`CertificateId`], [`Facility`], [`ControllerRating`],
//! [`PilotRating`], and [`MilitaryRating`]. Endpoint-specific structs live in
//! the [`datafeed`] and [`slurper`] submodules.

pub mod connect;
pub mod datafeed;
pub mod slurper;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A VATSIM user's certificate ID (CID), a unique numeric identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct CertificateId(u32);

/// ATC facility type as used in the VATSIM data feed and callsign suffixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Facility {
    #[default]
    Observer,
    Ramp,
    ClearanceDelivery,
    Ground,
    Tower,
    Approach,
    Departure,
    Enroute,
    FlightServiceStation,
    Radio,
    TrafficFlow,
}

/// VATSIM ATC controller rating, from inactive/suspended through administrator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum ControllerRating {
    #[default]
    Inactive,
    Suspended,
    Observer,
    TowerTrainee,
    TowerController,
    SeniorStudent,
    EnrouteController,
    #[deprecated(note = "not actively in use on VATSIM")]
    Controller2,
    SeniorController,
    Instructor,
    #[deprecated(note = "not actively in use on VATSIM")]
    Instructor2,
    SeniorInstructor,
    Supervisor,
    Administrator,
}

/// VATSIM pilot rating, from basic member through flight examiner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum PilotRating {
    #[default]
    BasicMember,
    PrivatePilotLicense,
    InstrumentRating,
    CommercialMultiEngineLicense,
    AirlineTransportPilotLicense,
    FlightInstructor,
    FlightExaminer,
}

/// VATSIM military pilot rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum MilitaryRating {
    #[default]
    NoMilitaryRating,
    MilitaryPilotLicense,
    MilitaryInstrumentRating,
    MilitaryMultiEngineRating,
    MilitaryMissionReadyPilot,
}

impl CertificateId {
    /// Creates a new [`CertificateId`] from a raw numeric ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the underlying numeric ID.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for CertificateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for CertificateId {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl From<u32> for CertificateId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<CertificateId> for u32 {
    fn from(id: CertificateId) -> Self {
        id.0
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for CertificateId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Clone, Copy)]
        struct CertificateIdVisitor;
        impl serde::de::Visitor<'_> for CertificateIdVisitor {
            type Value = CertificateId;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a CID as a string or integer")
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<CertificateId, E> {
                u32::try_from(v)
                    .map(CertificateId::new)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Signed(v), &self))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<CertificateId, E> {
                u32::try_from(v)
                    .map(CertificateId::new)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Unsigned(v), &self))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<CertificateId, E> {
                v.parse().map_err(E::custom)
            }
        }
        deserializer.deserialize_any(CertificateIdVisitor)
    }
}

impl Facility {
    /// All active facility variants (excludes no deprecated variants).
    pub const ALL: &'static [Self] = &[
        Self::Observer,
        Self::Ramp,
        Self::ClearanceDelivery,
        Self::Ground,
        Self::Tower,
        Self::Approach,
        Self::Departure,
        Self::Enroute,
        Self::FlightServiceStation,
        Self::Radio,
        Self::TrafficFlow,
    ];

    /// Returns the short string code for this facility (e.g. `"TWR"`, `"APP"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Facility::Observer => "OBS",
            Facility::Ramp => "RMP",
            Facility::ClearanceDelivery => "DEL",
            Facility::Ground => "GND",
            Facility::Tower => "TWR",
            Facility::Approach => "APP",
            Facility::Departure => "DEP",
            Facility::Enroute => "CTR",
            Facility::FlightServiceStation => "FSS",
            Facility::Radio => "RDO",
            Facility::TrafficFlow => "FMP",
        }
    }

    /// Guesses the [`Facility`] from a callsign suffix (e.g. `"LOWW_TWR"` -> [`Facility::Tower`]).
    ///
    /// Returns [`Facility::Observer`] if the suffix is not recognized.
    #[must_use]
    pub fn from_callsign(callsign: impl AsRef<str>) -> Self {
        let facility_suffix = callsign.as_ref().split('_').next_back().unwrap_or_default();
        Facility::try_from(facility_suffix).unwrap_or_default()
    }
}

impl std::fmt::Display for Facility {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Facility {
    type Err = crate::error::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_uppercase();
        match s.as_str() {
            "OBS" | "OBSERVER" => Ok(Facility::Observer),
            "RMP" | "RAMP" => Ok(Facility::Ramp),
            "DEL" | "DELIVERY" | "CLEARANCE DELIVERY" => Ok(Facility::ClearanceDelivery),
            "GND" | "GROUND" => Ok(Facility::Ground),
            "TWR" | "TOWER" => Ok(Facility::Tower),
            "APP" | "APPROACH" | "APPROACH/DEPARTURE" => Ok(Facility::Approach),
            "DEP" | "DEPARTURE" => Ok(Facility::Departure),
            "CTR" | "CENTER" | "CENTRE" | "ENROUTE" => Ok(Facility::Enroute),
            "FSS" | "FLIGHTSERVICESTATION" | "FLIGHT SERVICE STATION" => {
                Ok(Facility::FlightServiceStation)
            }
            "RDO" | "RADIO" => Ok(Facility::Radio),
            "TMU"
            | "TRAFFICMANAGEMENTUNIT"
            | "TRAFFIC MANAGEMENT UNIT"
            | "FMP"
            | "FLOWMANAGEMENTPOSITION"
            | "FLOW MANAGEMENT POSITION"
            | "TRAFFICFLOW"
            | "TRAFFIC FLOW" => Ok(Facility::TrafficFlow),
            other => Err(Self::Err::UnknownValue {
                kind: "facility",
                value: other.to_string(),
            }),
        }
    }
}

impl TryFrom<&str> for Facility {
    type Error = crate::error::ParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for Facility {
    type Error = crate::error::ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().parse()
    }
}

impl TryFrom<i8> for Facility {
    type Error = crate::error::ParseError;
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Facility::Observer),
            1 => Ok(Facility::FlightServiceStation),
            2 => Ok(Facility::ClearanceDelivery),
            3 => Ok(Facility::Ground),
            4 => Ok(Facility::Tower),
            5 => Ok(Facility::Approach),
            6 => Ok(Facility::Enroute),
            other => Err(Self::Error::UnknownValue {
                kind: "facility",
                value: other.to_string(),
            }),
        }
    }
}

impl TryFrom<datafeed::FacilityInfo> for Facility {
    type Error = crate::error::ParseError;
    fn try_from(value: datafeed::FacilityInfo) -> Result<Self, Self::Error> {
        value.id.try_into()
    }
}

#[cfg(feature = "serde")]
impl Serialize for Facility {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Facility {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Clone, Copy)]
        struct FacilityVisitor;
        impl serde::de::Visitor<'_> for FacilityVisitor {
            type Value = Facility;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a facility name string or integer ID")
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Facility, E> {
                i8::try_from(v)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Signed(v), &self))
                    .and_then(|id| Facility::try_from(id).map_err(|e| E::custom(e)))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Facility, E> {
                self.visit_i64(
                    i64::try_from(v)
                        .map_err(|_| E::invalid_value(serde::de::Unexpected::Unsigned(v), &self))?,
                )
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Facility, E> {
                v.parse().map_err(E::custom)
            }
        }
        deserializer.deserialize_any(FacilityVisitor)
    }
}

impl ControllerRating {
    /// All active controller rating variants (excludes deprecated ones).
    pub const ALL: &'static [Self] = &[
        Self::Inactive,
        Self::Suspended,
        Self::Observer,
        Self::TowerTrainee,
        Self::TowerController,
        Self::SeniorStudent,
        Self::EnrouteController,
        Self::SeniorController,
        Self::Instructor,
        Self::SeniorInstructor,
        Self::Supervisor,
        Self::Administrator,
    ];

    /// Returns the short string code for this rating (e.g. `"S1"`, `"C1"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ControllerRating::Inactive => "INAC",
            ControllerRating::Suspended => "SUS",
            ControllerRating::Observer => "OBS",
            ControllerRating::TowerTrainee => "S1",
            ControllerRating::TowerController => "S2",
            ControllerRating::SeniorStudent => "S3",
            ControllerRating::EnrouteController => "C1",
            #[allow(deprecated)]
            ControllerRating::Controller2 => "C2",
            ControllerRating::SeniorController => "C3",
            ControllerRating::Instructor => "I1",
            #[allow(deprecated)]
            ControllerRating::Instructor2 => "I2",
            ControllerRating::SeniorInstructor => "I3",
            ControllerRating::Supervisor => "SUP",
            ControllerRating::Administrator => "ADM",
        }
    }

    /// Returns the numeric ID used in the VATSIM data feed for this rating.
    #[must_use]
    pub const fn as_id(self) -> i8 {
        match self {
            ControllerRating::Inactive => -1,
            ControllerRating::Suspended => 0,
            ControllerRating::Observer => 1,
            ControllerRating::TowerTrainee => 2,
            ControllerRating::TowerController => 3,
            ControllerRating::SeniorStudent => 4,
            ControllerRating::EnrouteController => 5,
            #[allow(deprecated)]
            ControllerRating::Controller2 => 6,
            ControllerRating::SeniorController => 7,
            ControllerRating::Instructor => 8,
            #[allow(deprecated)]
            ControllerRating::Instructor2 => 9,
            ControllerRating::SeniorInstructor => 10,
            ControllerRating::Supervisor => 11,
            ControllerRating::Administrator => 12,
        }
    }
}

impl std::str::FromStr for ControllerRating {
    type Err = crate::error::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_uppercase();
        match s.as_str() {
            "INAC" | "INACTIVE" => Ok(ControllerRating::Inactive),
            "SUS" | "SUSPENDED" => Ok(ControllerRating::Suspended),
            "OBS" | "OBSERVER" => Ok(ControllerRating::Observer),
            "S1" | "TOWER TRAINEE" => Ok(ControllerRating::TowerTrainee),
            "S2" | "TOWER CONTROLLER" => Ok(ControllerRating::TowerController),
            "S3" | "SENIOR STUDENT" => Ok(ControllerRating::SeniorStudent),
            "C1" | "ENROUTE CONTROLLER" => Ok(ControllerRating::EnrouteController),
            #[allow(deprecated)]
            "C2" | "CONTROLLER 2 (NOT IN USE)" => Ok(ControllerRating::Controller2),
            "C3" | "SENIOR CONTROLLER" => Ok(ControllerRating::SeniorController),
            "I1" | "INSTRUCTOR" => Ok(ControllerRating::Instructor),
            #[allow(deprecated)]
            "I2" | "INSTRUCTOR 2 (NOT IN USE)" => Ok(ControllerRating::Instructor2),
            "I3" | "SENIOR INSTRUCTOR" => Ok(ControllerRating::SeniorInstructor),
            "SUP" | "SUPERVISOR" => Ok(ControllerRating::Supervisor),
            "ADM" | "ADMINISTRATOR" => Ok(ControllerRating::Administrator),
            other => Err(Self::Err::UnknownValue {
                kind: "controller rating",
                value: other.to_string(),
            }),
        }
    }
}

impl TryFrom<&str> for ControllerRating {
    type Error = crate::error::ParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for ControllerRating {
    type Error = crate::error::ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().parse()
    }
}

impl TryFrom<i8> for ControllerRating {
    type Error = crate::error::ParseError;
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(ControllerRating::Inactive),
            0 => Ok(ControllerRating::Suspended),
            1 => Ok(ControllerRating::Observer),
            2 => Ok(ControllerRating::TowerTrainee),
            3 => Ok(ControllerRating::TowerController),
            4 => Ok(ControllerRating::SeniorStudent),
            5 => Ok(ControllerRating::EnrouteController),
            #[allow(deprecated)]
            6 => Ok(ControllerRating::Controller2),
            7 => Ok(ControllerRating::SeniorController),
            8 => Ok(ControllerRating::Instructor),
            #[allow(deprecated)]
            9 => Ok(ControllerRating::Instructor2),
            10 => Ok(ControllerRating::SeniorInstructor),
            11 => Ok(ControllerRating::Supervisor),
            12 => Ok(ControllerRating::Administrator),
            other => Err(Self::Error::UnknownValue {
                kind: "controller rating",
                value: other.to_string(),
            }),
        }
    }
}

impl TryFrom<datafeed::RatingInfo> for ControllerRating {
    type Error = crate::error::ParseError;
    fn try_from(value: datafeed::RatingInfo) -> Result<Self, Self::Error> {
        value.id.try_into()
    }
}

#[cfg(feature = "serde")]
impl Serialize for ControllerRating {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ControllerRating {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Clone, Copy)]
        struct RatingVisitor;
        impl serde::de::Visitor<'_> for RatingVisitor {
            type Value = ControllerRating;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a controller rating string or integer ID")
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<ControllerRating, E> {
                i8::try_from(v)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Signed(v), &self))
                    .and_then(|id| ControllerRating::try_from(id).map_err(|e| E::custom(e)))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<ControllerRating, E> {
                self.visit_i64(
                    i64::try_from(v)
                        .map_err(|_| E::invalid_value(serde::de::Unexpected::Unsigned(v), &self))?,
                )
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<ControllerRating, E> {
                v.parse().map_err(E::custom)
            }
        }
        deserializer.deserialize_any(RatingVisitor)
    }
}

impl PilotRating {
    /// All pilot rating variants.
    pub const ALL: &'static [Self] = &[
        Self::BasicMember,
        Self::PrivatePilotLicense,
        Self::InstrumentRating,
        Self::CommercialMultiEngineLicense,
        Self::AirlineTransportPilotLicense,
        Self::FlightInstructor,
        Self::FlightExaminer,
    ];

    /// Returns the short string code for this rating (e.g. `"PPL"`, `"ATPL"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            PilotRating::BasicMember => "NEW",
            PilotRating::PrivatePilotLicense => "PPL",
            PilotRating::InstrumentRating => "IR",
            PilotRating::CommercialMultiEngineLicense => "CMEL",
            PilotRating::AirlineTransportPilotLicense => "ATPL",
            PilotRating::FlightInstructor => "FI",
            PilotRating::FlightExaminer => "FE",
        }
    }
}

impl std::str::FromStr for PilotRating {
    type Err = crate::error::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_uppercase();
        match s.as_str() {
            "NEW" | "BASIC MEMBER" => Ok(PilotRating::BasicMember),
            "PPL" | "PRIVATE PILOT LICENSE" => Ok(PilotRating::PrivatePilotLicense),
            "IR" | "INSTRUMENT RATING" => Ok(PilotRating::InstrumentRating),
            "CMEL" | "COMMERCIAL MULTI-ENGINE LICENSE" => {
                Ok(PilotRating::CommercialMultiEngineLicense)
            }
            "ATPL" | "AIRLINE TRANSPORT PILOT LICENSE" => {
                Ok(PilotRating::AirlineTransportPilotLicense)
            }
            "FI" | "FLIGHT INSTRUCTOR" => Ok(PilotRating::FlightInstructor),
            "FE" | "FLIGHT EXAMINER" => Ok(PilotRating::FlightExaminer),
            other => Err(Self::Err::UnknownValue {
                kind: "pilot rating",
                value: other.to_string(),
            }),
        }
    }
}

impl TryFrom<&str> for PilotRating {
    type Error = crate::error::ParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for PilotRating {
    type Error = crate::error::ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().parse()
    }
}

impl TryFrom<i8> for PilotRating {
    type Error = crate::error::ParseError;
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PilotRating::BasicMember),
            1 => Ok(PilotRating::PrivatePilotLicense),
            3 => Ok(PilotRating::InstrumentRating),
            7 => Ok(PilotRating::CommercialMultiEngineLicense),
            15 => Ok(PilotRating::AirlineTransportPilotLicense),
            31 => Ok(PilotRating::FlightInstructor),
            63 => Ok(PilotRating::FlightExaminer),
            other => Err(Self::Error::UnknownValue {
                kind: "pilot rating",
                value: other.to_string(),
            }),
        }
    }
}

impl TryFrom<datafeed::PilotRatingInfo> for PilotRating {
    type Error = crate::error::ParseError;
    fn try_from(value: datafeed::PilotRatingInfo) -> Result<Self, Self::Error> {
        value.id.try_into()
    }
}

#[cfg(feature = "serde")]
impl Serialize for PilotRating {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for PilotRating {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Clone, Copy)]
        struct RatingVisitor;
        impl serde::de::Visitor<'_> for RatingVisitor {
            type Value = PilotRating;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a pilot rating string or integer ID")
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<PilotRating, E> {
                i8::try_from(v)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Signed(v), &self))
                    .and_then(|id| PilotRating::try_from(id).map_err(|e| E::custom(e)))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<PilotRating, E> {
                self.visit_i64(
                    i64::try_from(v)
                        .map_err(|_| E::invalid_value(serde::de::Unexpected::Unsigned(v), &self))?,
                )
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<PilotRating, E> {
                v.parse().map_err(E::custom)
            }
        }
        deserializer.deserialize_any(RatingVisitor)
    }
}

impl MilitaryRating {
    /// All military rating variants.
    pub const ALL: &'static [Self] = &[
        Self::NoMilitaryRating,
        Self::MilitaryPilotLicense,
        Self::MilitaryInstrumentRating,
        Self::MilitaryMultiEngineRating,
        Self::MilitaryMissionReadyPilot,
    ];

    /// Returns the short string code for this rating (e.g. `"M0"`, `"M4"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            MilitaryRating::NoMilitaryRating => "M0",
            MilitaryRating::MilitaryPilotLicense => "M1",
            MilitaryRating::MilitaryInstrumentRating => "M2",
            MilitaryRating::MilitaryMultiEngineRating => "M3",
            MilitaryRating::MilitaryMissionReadyPilot => "M4",
        }
    }
}

impl std::str::FromStr for MilitaryRating {
    type Err = crate::error::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_uppercase();
        match s.as_str() {
            "M0" | "NO MILITARY RATING" => Ok(MilitaryRating::NoMilitaryRating),
            "M1" | "MILITARY PILOT LICENSE" => Ok(MilitaryRating::MilitaryPilotLicense),
            "M2" | "MILITARY INSTRUMENT RATING" => Ok(MilitaryRating::MilitaryInstrumentRating),
            "M3" | "MILITARY MULTI-ENGINE RATING" => Ok(MilitaryRating::MilitaryMultiEngineRating),
            "M4" | "MILITARY MISSION READY PILOT" => Ok(MilitaryRating::MilitaryMissionReadyPilot),
            other => Err(Self::Err::UnknownValue {
                kind: "military rating",
                value: other.to_string(),
            }),
        }
    }
}

impl TryFrom<&str> for MilitaryRating {
    type Error = crate::error::ParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for MilitaryRating {
    type Error = crate::error::ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().parse()
    }
}

impl TryFrom<i8> for MilitaryRating {
    type Error = crate::error::ParseError;
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MilitaryRating::NoMilitaryRating),
            1 => Ok(MilitaryRating::MilitaryPilotLicense),
            3 => Ok(MilitaryRating::MilitaryInstrumentRating),
            7 => Ok(MilitaryRating::MilitaryMultiEngineRating),
            15 => Ok(MilitaryRating::MilitaryMissionReadyPilot),
            other => Err(Self::Error::UnknownValue {
                kind: "military rating",
                value: other.to_string(),
            }),
        }
    }
}

impl TryFrom<datafeed::MilitaryRatingInfo> for MilitaryRating {
    type Error = crate::error::ParseError;
    fn try_from(value: datafeed::MilitaryRatingInfo) -> Result<Self, Self::Error> {
        value.id.try_into()
    }
}

#[cfg(feature = "serde")]
impl Serialize for MilitaryRating {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for MilitaryRating {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Clone, Copy)]
        struct RatingVisitor;
        impl serde::de::Visitor<'_> for RatingVisitor {
            type Value = MilitaryRating;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a military rating string or integer ID")
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<MilitaryRating, E> {
                i8::try_from(v)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Signed(v), &self))
                    .and_then(|id| MilitaryRating::try_from(id).map_err(|e| E::custom(e)))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<MilitaryRating, E> {
                self.visit_i64(
                    i64::try_from(v)
                        .map_err(|_| E::invalid_value(serde::de::Unexpected::Unsigned(v), &self))?,
                )
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<MilitaryRating, E> {
                v.parse().map_err(E::custom)
            }
        }
        deserializer.deserialize_any(RatingVisitor)
    }
}
