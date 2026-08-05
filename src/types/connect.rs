//! Types for the VATSIM Connect (`OAuth2`) API.
//!
//! These model the responses from the
//! [Connect API](https://vatsim.dev/api/connect-api), specifically the
//! [user details](https://vatsim.dev/api/connect-api/get-user) endpoint
//! (`GET /api/user`).

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Response from `GET /api/user`, wrapping the user data in a `data` field.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ConnectUserResponse {
    /// The user record.
    pub data: ConnectUser,
}

/// A VATSIM user as returned by the Connect `/api/user` endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ConnectUser {
    /// VATSIM Certificate ID (CID), serialized as a string.
    #[cfg_attr(feature = "serde", serde(with = "cid_as_string"))]
    pub cid: crate::types::CertificateId,
    /// Name, email and country, subject to the scopes granted.
    pub personal: PersonalDetails,
    /// Ratings, region, division and subdivision.
    pub vatsim: VatsimDetails,
    /// Validity of the token used to make the request.
    pub oauth: OAuthInfo,
}

/// Personal details of a VATSIM user.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PersonalDetails {
    /// Given name.
    pub name_first: String,
    /// Family name.
    pub name_last: String,
    /// Full name as VATSIM renders it.
    pub name_full: String,
    /// Email address. Present only if the `email` scope was granted.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub email: Option<String>,
    /// Country of residence. Present only if the `country` scope was granted.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub country: Option<CountryInfo>,
}

/// Country of residence.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CountryInfo {
    /// ISO 3166-1 alpha-2 country code, e.g. `"AT"`.
    pub id: String,
    /// Country name, e.g. `"Austria"`.
    pub name: String,
}

/// VATSIM-specific details (ratings, region, division, subdivision).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VatsimDetails {
    /// ATC rating.
    pub rating: ConnectRatingInfo,
    /// Pilot rating.
    pub pilotrating: ConnectRatingInfo,
    /// VATSIM region, e.g. `"EMEA"`.
    pub region: NamedInfo,
    /// VATSIM division, e.g. `"EUD"`.
    pub division: NamedInfo,
    /// VATSIM subdivision, e.g. `"AUS"`. Absent for users not in one.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub subdivision: Option<NamedInfo>,
}

/// A rating with numeric ID, short code, and long name.
///
/// Used for both controller and pilot ratings in the Connect response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ConnectRatingInfo {
    /// Numeric rating ID.
    pub id: i8,
    /// Short code, e.g. `"C1"`.
    pub short: String,
    /// Full name, e.g. `"Enroute"`.
    pub long: String,
}

/// A named entity with string ID and name (region, division, subdivision).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NamedInfo {
    /// Identifier, e.g. `"EMEA"`. Absent if VATSIM did not report one.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub id: Option<String>,
    /// Human-readable name. Absent if VATSIM did not report one.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub name: Option<String>,
}

/// OAuth token validity info returned inside the user response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OAuthInfo {
    /// Whether the access token is still valid, as the string `"true"` or `"false"`.
    pub token_valid: String,
}

/// Token response from `POST /oauth/token`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TokenResponse {
    /// Token type, always `"Bearer"`.
    pub token_type: String,
    /// Lifetime of the access token in seconds.
    pub expires_in: u64,
    /// Token to send as `Authorization: Bearer <token>`.
    pub access_token: String,
    /// Token for obtaining a new access token once this one expires.
    pub refresh_token: String,
    /// Scopes actually granted, which may be narrower than those requested.
    pub scopes: Vec<String>,
}

/// OAuth error response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OAuthError {
    /// Machine-readable error code, e.g. `"invalid_grant"`.
    pub error: String,
    /// Human-readable description of the failure.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub error_description: Option<String>,
    /// Alternative message field used by some VATSIM error responses.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub message: Option<String>,
    /// Additional hint about how to correct the request.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub hint: Option<String>,
}

/// Serde helper that serializes a [`CertificateId`](crate::types::CertificateId) as a string.
///
/// The default `Serialize` impl writes a numeric `u32` (matching the datafeed),
/// but the Connect API represents CIDs as strings. Deserialization is handled
/// by the Visitor impl on `CertificateId` itself, which accepts both formats.
#[cfg(feature = "serde")]
mod cid_as_string {
    use serde::{Deserialize, Deserializer, Serializer};

    use crate::types::CertificateId;

    #[allow(clippy::trivially_copy_pass_by_ref)] // serde requires &T
    pub fn serialize<S>(cid: &CertificateId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(cid)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<CertificateId, D::Error>
    where
        D: Deserializer<'de>,
    {
        CertificateId::deserialize(deserializer)
    }
}
