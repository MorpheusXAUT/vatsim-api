//! Types for the [VATSIM slurper API](https://vatsim.dev/api/slurper-api/get-user-info).
//!
//! The slurper endpoint returns CSV, not JSON. Each line describes one active
//! connection for a user. [`UserConnection`] models a single line and can be
//! parsed via its [`FromStr`](std::str::FromStr) implementation or
//! [`UserConnection::parse_line`] for line-number-aware errors.

use crate::error::ParseError;
use crate::types::CertificateId;

/// A single row from the slurper CSV response.
///
/// Each line of the [slurper API](https://vatsim.dev/api/slurper-api/get-user-info)
/// response describes one active connection for a user. The CSV fields are:
///
/// `CID,Callsign,FacilityType,Frequency,VisualRange,Lat,Lon[,SecLat,SecLon,...],`
///
/// Pilots have empty frequency and visual-range fields; controllers may have
/// zero or more secondary-position lat/lon pairs appended.
#[derive(Debug, Clone, PartialEq)]
pub struct UserConnection {
    pub cid: CertificateId,
    pub callsign: String,
    pub facility_type: SlurperFacilityType,
    /// MHz frequency string (e.g. `"121.900"`) - [`None`] for pilots.
    pub frequency: Option<String>,
    /// Visual range in nautical miles - [`None`] for pilots.
    pub visual_range: Option<u32>,
    pub latitude: f64,
    pub longitude: f64,
    /// Secondary positions as lat/lon pairs, if set by a controller.
    pub secondary_positions: Vec<(f64, f64)>,
}

/// The facility type reported by the slurper API for each connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum SlurperFacilityType {
    #[default]
    Pilot,
    Atc,
}

impl std::fmt::Display for SlurperFacilityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Pilot => "pilot",
            Self::Atc => "atc",
        })
    }
}

impl std::str::FromStr for SlurperFacilityType {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "pilot" => Ok(Self::Pilot),
            "atc" => Ok(Self::Atc),
            _ => Err(ParseError::UnknownValue {
                kind: "slurper facility type",
                value: value.to_owned(),
            }),
        }
    }
}

impl std::str::FromStr for UserConnection {
    type Err = ParseError;

    /// Parses a single CSV line from the slurper response.
    ///
    /// The `line` parameter is expected to be a **zero-indexed line number**
    /// used in error messages; set it via the [`InvalidSlurperCsv`](ParseError::InvalidSlurperCsv)
    /// variant. Because `FromStr` does not carry a line number, errors
    /// report `line: 0`. Use [`UserConnection::parse_line`] for line-aware parsing.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_line(s, 0)
    }
}

impl UserConnection {
    /// Parses a single CSV line from the slurper response, reporting `line`
    /// in any error.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::InvalidSlurperCsv`] when the line has too few
    /// fields or contains unparseable values.
    pub fn parse_line(s: &str, line: usize) -> Result<Self, ParseError> {
        // Strip trailing comma(s) and whitespace the API appends.
        let trimmed = s.trim_end_matches(',').trim();

        let fields: Vec<&str> = trimmed.split(',').collect();
        if fields.len() < 7 {
            return Err(ParseError::InvalidSlurperCsv {
                line,
                reason: format!("expected at least 7 fields, got {}", fields.len()),
            });
        }

        let csv_err = |reason: String| ParseError::InvalidSlurperCsv { line, reason };

        let cid: u32 = fields[0]
            .trim()
            .parse()
            .map_err(|e| csv_err(format!("invalid CID: {e}")))?;

        let callsign = fields[1].trim().to_owned();
        if callsign.is_empty() {
            return Err(csv_err("empty callsign".to_owned()));
        }

        let facility_type: SlurperFacilityType = fields[2]
            .trim()
            .parse()
            .map_err(|_| csv_err(format!("unknown facility type: {}", fields[2].trim())))?;

        let frequency = non_empty(fields[3]);
        let visual_range = non_empty(fields[4])
            .map(|v| {
                v.parse::<u32>()
                    .map_err(|e| csv_err(format!("invalid visual range: {e}")))
            })
            .transpose()?;

        let latitude: f64 = fields[5]
            .trim()
            .parse()
            .map_err(|e| csv_err(format!("invalid latitude: {e}")))?;
        let longitude: f64 = fields[6]
            .trim()
            .parse()
            .map_err(|e| csv_err(format!("invalid longitude: {e}")))?;

        // Remaining fields are secondary position lat/lon pairs.
        let extra = &fields[7..];
        let mut secondary_positions = Vec::new();
        let mut i = 0;
        while i + 1 < extra.len() {
            let lat_str = extra[i].trim();
            let lon_str = extra[i + 1].trim();
            // Skip zero-padding pairs (the API often appends `0,0,...`)
            if lat_str == "0" && lon_str == "0" {
                i += 2;
                continue;
            }
            let lat: f64 = lat_str
                .parse()
                .map_err(|e| csv_err(format!("invalid secondary latitude: {e}")))?;
            let lon: f64 = lon_str
                .parse()
                .map_err(|e| csv_err(format!("invalid secondary longitude: {e}")))?;
            secondary_positions.push((lat, lon));
            i += 2;
        }

        Ok(Self {
            cid: CertificateId::new(cid),
            callsign,
            facility_type,
            frequency,
            visual_range,
            latitude,
            longitude,
            secondary_positions,
        })
    }
}

/// Returns `Some(trimmed)` if the field is non-empty after trimming,
/// `None` otherwise.
fn non_empty(field: &str) -> Option<String> {
    let trimmed = field.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pilot_line() {
        let line = "1234567,BAW123,pilot,,,51.148056,-0.190278,0,0,0,0,";
        let conn: UserConnection = line.parse().unwrap();
        assert_eq!(conn.cid, CertificateId::new(1_234_567));
        assert_eq!(conn.callsign, "BAW123");
        assert_eq!(conn.facility_type, SlurperFacilityType::Pilot);
        assert_eq!(conn.frequency, None);
        assert_eq!(conn.visual_range, None);
        assert!(conn.secondary_positions.is_empty());
    }

    #[test]
    fn parse_atc_line() {
        let line = "1234567,EGLL_TWR,atc,118.500,50,51.4775,-0.461389,";
        let conn: UserConnection = line.parse().unwrap();
        assert_eq!(conn.callsign, "EGLL_TWR");
        assert_eq!(conn.facility_type, SlurperFacilityType::Atc);
        assert_eq!(conn.frequency.as_deref(), Some("118.500"));
        assert_eq!(conn.visual_range, Some(50));
        assert!(conn.secondary_positions.is_empty());
    }

    #[test]
    fn parse_atc_with_secondary_positions() {
        let line = "1234567,EGLL_APP,atc,119.725,100,51.4775,-0.461389,51.15,-0.18,52.0,0.5,";
        let conn: UserConnection = line.parse().unwrap();
        assert_eq!(conn.secondary_positions.len(), 2);
        assert!((conn.secondary_positions[0].0 - 51.15).abs() < f64::EPSILON);
        assert!((conn.secondary_positions[1].1 - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_strips_zero_padding_pairs() {
        let line = "1234567,EGLL_APP,atc,119.725,100,51.4775,-0.461389,51.15,-0.18,0,0,0,0,";
        let conn: UserConnection = line.parse().unwrap();
        assert_eq!(conn.secondary_positions.len(), 1);
    }

    #[test]
    fn parse_too_few_fields() {
        let line = "1234567,BAW123,pilot,,,51.1";
        let err = line.parse::<UserConnection>().unwrap_err();
        assert!(matches!(err, ParseError::InvalidSlurperCsv { .. }));
    }

    #[test]
    fn parse_invalid_cid() {
        let line = "not_a_number,BAW123,pilot,,,51.1,-0.19,";
        let err = line.parse::<UserConnection>().unwrap_err();
        assert!(matches!(err, ParseError::InvalidSlurperCsv { .. }));
    }

    #[test]
    fn parse_empty_callsign() {
        let line = "1234567,,pilot,,,51.1,-0.19,";
        let err = line.parse::<UserConnection>().unwrap_err();
        assert!(matches!(err, ParseError::InvalidSlurperCsv { .. }));
    }

    #[test]
    fn parse_unknown_facility_type() {
        let line = "1234567,BAW123,observer,,,51.1,-0.19,";
        let err = line.parse::<UserConnection>().unwrap_err();
        assert!(matches!(err, ParseError::InvalidSlurperCsv { .. }));
    }

    #[test]
    fn slurper_facility_type_display() {
        assert_eq!(SlurperFacilityType::Pilot.to_string(), "pilot");
        assert_eq!(SlurperFacilityType::Atc.to_string(), "atc");
    }

    #[test]
    fn slurper_facility_type_roundtrip() {
        for ty in [SlurperFacilityType::Pilot, SlurperFacilityType::Atc] {
            let parsed: SlurperFacilityType = ty.to_string().parse().unwrap();
            assert_eq!(parsed, ty);
        }
    }

    #[test]
    fn parse_line_preserves_line_number_in_error() {
        let result = UserConnection::parse_line("bad", 42);
        match result.unwrap_err() {
            ParseError::InvalidSlurperCsv { line, .. } => assert_eq!(line, 42),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn parse_facility_type_case_insensitive() {
        let line = "1234567,EGLL_TWR,ATC,118.500,50,51.4775,-0.461389,";
        let conn: UserConnection = line.parse().unwrap();
        assert_eq!(conn.facility_type, SlurperFacilityType::Atc);

        let line = "1234567,BAW123,Pilot,,,51.1,-0.19,";
        let conn: UserConnection = line.parse().unwrap();
        assert_eq!(conn.facility_type, SlurperFacilityType::Pilot);
    }

    #[test]
    fn parse_realistic_multi_entry_response() {
        let response = "\
            1234567,LOWW_D_ATIS,atc,121.730,0,48.11028,16.56972,0,0,0,0,0,0,0,0,\n\
            1234567,LOVV_CTR,atc,123.450,600,47.66667,14.33333,0,0,0,0,0,0,0,0,\n\
            1234567,LOWW_A_ATIS,atc,122.955,0,48.11028,16.56972,0,0,0,0,0,0,0,0,\n";

        let connections: Vec<UserConnection> = response
            .lines()
            .filter(|l| !l.trim().is_empty())
            .enumerate()
            .map(|(i, l)| UserConnection::parse_line(l, i).unwrap())
            .collect();

        assert_eq!(connections.len(), 3);
        assert_eq!(connections[0].callsign, "LOWW_D_ATIS");
        assert_eq!(connections[0].visual_range, Some(0));
        assert_eq!(connections[1].callsign, "LOVV_CTR");
        assert_eq!(connections[1].visual_range, Some(600));
        assert_eq!(connections[2].callsign, "LOWW_A_ATIS");
    }

    #[test]
    fn parse_many_zero_padded_secondary_positions() {
        let line = "1234567,LOVV_CTR,atc,123.450,600,47.66667,14.33333,0,0,0,0,0,0,0,0,";
        let conn: UserConnection = line.parse().unwrap();
        assert!(conn.secondary_positions.is_empty());
    }

    #[test]
    fn parse_multiple_trailing_commas() {
        let line = "1234567,BAW123,pilot,,,51.148056,-0.190278,0,0,0,0,,,,";
        let conn: UserConnection = line.parse().unwrap();
        assert_eq!(conn.callsign, "BAW123");
        assert!(conn.secondary_positions.is_empty());
    }

    #[test]
    fn parse_minimal_valid_line_no_trailing_comma() {
        let line = "1234567,EGLL_TWR,atc,118.500,50,51.4775,-0.461389";
        let conn: UserConnection = line.parse().unwrap();
        assert_eq!(conn.callsign, "EGLL_TWR");
        assert_eq!(conn.frequency.as_deref(), Some("118.500"));
    }

    #[test]
    fn parse_negative_coordinates() {
        let line = "1234567,SAEZ_TWR,atc,118.500,50,-34.8222,-58.5358,";
        let conn: UserConnection = line.parse().unwrap();
        assert!((conn.latitude - (-34.8222)).abs() < f64::EPSILON);
        assert!((conn.longitude - (-58.5358)).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_odd_trailing_field_ignored() {
        let line = "1234567,EGLL_TWR,atc,118.500,50,51.4775,-0.461389,42.0,";
        let conn: UserConnection = line.parse().unwrap();
        assert!(conn.secondary_positions.is_empty());
    }

    #[test]
    fn parse_secondary_positions_with_negative_coords() {
        let line = "1234567,EGLL_APP,atc,119.725,100,51.4775,-0.461389,-33.86,151.21,51.15,-0.18,";
        let conn: UserConnection = line.parse().unwrap();
        assert_eq!(conn.secondary_positions.len(), 2);
        assert!((conn.secondary_positions[0].0 - (-33.86)).abs() < f64::EPSILON);
        assert!((conn.secondary_positions[0].1 - 151.21).abs() < f64::EPSILON);
    }
}
