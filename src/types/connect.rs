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
    pub data: ConnectUser,
}

/// A VATSIM user as returned by the Connect `/api/user` endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ConnectUser {
    /// VATSIM Certificate ID (CID), serialized as a string.
    #[cfg_attr(feature = "serde", serde(with = "cid_as_string"))]
    pub cid: crate::types::CertificateId,
    pub personal: PersonalDetails,
    pub vatsim: VatsimDetails,
    pub oauth: OAuthInfo,
}

/// Personal details of a VATSIM user.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PersonalDetails {
    pub name_first: String,
    pub name_last: String,
    pub name_full: String,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub email: Option<String>,
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
    pub id: String,
    pub name: String,
}

/// VATSIM-specific details (ratings, region, division, subdivision).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VatsimDetails {
    pub rating: ConnectRatingInfo,
    pub pilotrating: ConnectRatingInfo,
    pub region: NamedInfo,
    pub division: NamedInfo,
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
    pub id: i8,
    pub short: String,
    pub long: String,
}

/// A named entity with string ID and name (region, division, subdivision).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NamedInfo {
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub id: Option<String>,
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
    pub token_valid: String,
}

/// Token response from `POST /oauth/token`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TokenResponse {
    pub token_type: String,
    pub expires_in: u64,
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: Vec<String>,
}

/// OAuth error response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OAuthError {
    pub error: String,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub error_description: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub message: Option<String>,
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
