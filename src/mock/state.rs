//! Mutable state for the mock VATSIM server.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::types::CertificateId;
use crate::types::connect::ConnectUser;
use crate::types::datafeed::{
    Atis, Controller, DataFeed, FacilityInfo, GeneralInfo, MilitaryRatingInfo, Pilot,
    PilotRatingInfo, Prefile, RatingInfo, Server,
};

/// The complete state of the mock VATSIM server.
///
/// Each field corresponds to a section of the
/// [data feed](https://vatsim.dev/api/data-api/get-network-data) response.
/// The state is wrapped in an [`Arc<RwLock<_>>`] so that route handlers and
/// test code can read and mutate it concurrently.
///
/// Every collection defaults to empty, so a seed file only needs to list the
/// entities it actually cares about. This is deliberate: `MockState`'s serde
/// shape is the on-disk seed-file format, and requiring a field would break
/// every existing seed file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MockState {
    pub pilots: Vec<Pilot>,
    pub controllers: Vec<Controller>,
    pub atis: Vec<Atis>,
    pub servers: Vec<Server>,
    pub prefiles: Vec<Prefile>,
    pub facilities: Vec<FacilityInfo>,
    pub ratings: Vec<RatingInfo>,
    pub pilot_ratings: Vec<PilotRatingInfo>,
    pub military_ratings: Vec<MilitaryRatingInfo>,
    /// Connect users available for OAuth authentication.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<ConnectUser>,
    /// Pending authorization codes mapped to the CID they authenticate.
    /// Populated by `GET /oauth/authorize`, consumed by `POST /oauth/token`.
    #[serde(skip)]
    pub(crate) auth_codes: HashMap<String, CertificateId>,
    /// Active access tokens mapped to the CID they belong to.
    /// Populated by `POST /oauth/token`, looked up by `GET /api/user`.
    #[serde(skip)]
    pub(crate) access_tokens: HashMap<String, CertificateId>,
    /// Seed snapshot for resetting the state. Not serialized.
    /// `Box` is required because the type is recursive.
    #[serde(skip)]
    seed: Option<Box<MockState>>,
}

/// Thread-safe, shared handle to the [`MockState`].
pub type SharedState = Arc<RwLock<MockState>>;

impl MockState {
    /// Snapshots the current state as the seed for future [`reset`](Self::reset) calls.
    pub fn snapshot_seed(&mut self) {
        let mut snapshot = self.clone();
        snapshot.seed = None;
        self.seed = Some(Box::new(snapshot));
    }

    /// Replaces all live data from another [`MockState`] while preserving the
    /// seed snapshot. This is used by `PUT /api/state` so that a full state
    /// replacement cannot erase the immutable seed.
    pub fn replace(&mut self, other: MockState) {
        let seed = self.seed.take();
        *self = other;
        self.seed = seed;
    }

    /// Resets the live state to the seed snapshot.
    ///
    /// If no seed has been saved, all collections are cleared.
    pub fn reset(&mut self) {
        let seed = self.seed.take();
        if let Some(snapshot) = seed {
            *self = *snapshot;
            self.snapshot_seed();
        } else {
            self.pilots.clear();
            self.controllers.clear();
            self.atis.clear();
            self.servers.clear();
            self.prefiles.clear();
            self.facilities.clear();
            self.ratings.clear();
            self.pilot_ratings.clear();
            self.military_ratings.clear();
            self.users.clear();
        }
        self.auth_codes.clear();
        self.access_tokens.clear();
    }

    /// Returns a [`GeneralInfo`] derived from the current state.
    ///
    /// The `version` is always `3`, `connected_clients` is computed from the
    /// number of pilots + controllers + ATIS stations, and `unique_users` is
    /// the number of distinct CIDs across all connection types.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // mock server won't have > u32::MAX connections
    pub fn general_info(&self) -> GeneralInfo {
        let connected_clients =
            (self.pilots.len() + self.controllers.len() + self.atis.len()) as u32;

        let mut cids: Vec<CertificateId> = self
            .pilots
            .iter()
            .map(|p| p.cid)
            .chain(self.controllers.iter().map(|c| c.cid))
            .chain(self.atis.iter().map(|a| a.cid))
            .collect();
        cids.sort_unstable();
        cids.dedup();

        GeneralInfo {
            version: 3,
            update_timestamp: chrono::Utc::now(),
            connected_clients,
            unique_users: cids.len() as u32,
        }
    }

    /// Returns the pilot with the given CID, if any.
    #[must_use]
    pub fn pilot(&self, cid: CertificateId) -> Option<&Pilot> {
        self.pilots.iter().find(|p| p.cid == cid)
    }

    /// Returns the controller with the given CID, if any.
    #[must_use]
    pub fn controller(&self, cid: CertificateId) -> Option<&Controller> {
        self.controllers.iter().find(|c| c.cid == cid)
    }

    /// Returns the ATIS station with the given callsign, if any.
    #[must_use]
    pub fn atis(&self, callsign: impl AsRef<str>) -> Option<&Atis> {
        let callsign = callsign.as_ref();
        self.atis.iter().find(|a| a.callsign == callsign)
    }

    /// Returns the prefile with the given CID, if any.
    #[must_use]
    pub fn prefile(&self, cid: CertificateId) -> Option<&Prefile> {
        self.prefiles.iter().find(|p| p.cid == cid)
    }

    /// Returns the server with the given ident, if any.
    #[must_use]
    pub fn server(&self, ident: impl AsRef<str>) -> Option<&Server> {
        let ident = ident.as_ref();
        self.servers.iter().find(|s| s.ident == ident)
    }

    /// Returns the Connect user with the given CID, if any.
    #[must_use]
    pub fn user(&self, cid: CertificateId) -> Option<&ConnectUser> {
        self.users.iter().find(|u| u.cid == cid)
    }

    /// Removes a pilot by CID. Returns `true` if one was removed.
    pub fn remove_pilot(&mut self, cid: CertificateId) -> bool {
        let before = self.pilots.len();
        self.pilots.retain(|p| p.cid != cid);
        self.pilots.len() != before
    }

    /// Removes a controller by CID. Returns `true` if one was removed.
    pub fn remove_controller(&mut self, cid: CertificateId) -> bool {
        let before = self.controllers.len();
        self.controllers.retain(|c| c.cid != cid);
        self.controllers.len() != before
    }

    /// Removes an ATIS station by callsign. Returns `true` if one was removed.
    pub fn remove_atis(&mut self, callsign: impl AsRef<str>) -> bool {
        let callsign = callsign.as_ref();
        let before = self.atis.len();
        self.atis.retain(|a| a.callsign != callsign);
        self.atis.len() != before
    }

    /// Removes a prefile by CID. Returns `true` if one was removed.
    pub fn remove_prefile(&mut self, cid: CertificateId) -> bool {
        let before = self.prefiles.len();
        self.prefiles.retain(|p| p.cid != cid);
        self.prefiles.len() != before
    }

    /// Removes a server by ident. Returns `true` if one was removed.
    pub fn remove_server(&mut self, ident: impl AsRef<str>) -> bool {
        let ident = ident.as_ref();
        let before = self.servers.len();
        self.servers.retain(|s| s.ident != ident);
        self.servers.len() != before
    }

    /// Removes a Connect user by CID. Returns `true` if one was removed.
    pub fn remove_user(&mut self, cid: CertificateId) -> bool {
        let before = self.users.len();
        self.users.retain(|u| u.cid != cid);
        self.users.len() != before
    }

    /// Inserts or replaces a pilot (matched by CID).
    pub fn upsert_pilot(&mut self, pilot: Pilot) {
        if let Some(existing) = self.pilots.iter_mut().find(|p| p.cid == pilot.cid) {
            *existing = pilot;
        } else {
            self.pilots.push(pilot);
        }
    }

    /// Inserts or replaces a controller (matched by CID).
    pub fn upsert_controller(&mut self, controller: Controller) {
        if let Some(existing) = self
            .controllers
            .iter_mut()
            .find(|c| c.cid == controller.cid)
        {
            *existing = controller;
        } else {
            self.controllers.push(controller);
        }
    }

    /// Inserts or replaces an ATIS station (matched by callsign).
    pub fn upsert_atis(&mut self, atis: Atis) {
        if let Some(existing) = self.atis.iter_mut().find(|a| a.callsign == atis.callsign) {
            *existing = atis;
        } else {
            self.atis.push(atis);
        }
    }

    /// Inserts or replaces a prefile (matched by CID).
    pub fn upsert_prefile(&mut self, prefile: Prefile) {
        if let Some(existing) = self.prefiles.iter_mut().find(|p| p.cid == prefile.cid) {
            *existing = prefile;
        } else {
            self.prefiles.push(prefile);
        }
    }

    /// Inserts or replaces a server (matched by ident).
    pub fn upsert_server(&mut self, server: Server) {
        if let Some(existing) = self.servers.iter_mut().find(|s| s.ident == server.ident) {
            *existing = server;
        } else {
            self.servers.push(server);
        }
    }

    /// Inserts or replaces a Connect user (matched by CID).
    pub fn upsert_user(&mut self, user: ConnectUser) {
        if let Some(existing) = self.users.iter_mut().find(|u| u.cid == user.cid) {
            *existing = user;
        } else {
            self.users.push(user);
        }
    }
}

impl From<DataFeed> for MockState {
    fn from(feed: DataFeed) -> Self {
        Self {
            pilots: feed.pilots,
            controllers: feed.controllers,
            atis: feed.atis,
            servers: feed.servers,
            prefiles: feed.prefiles,
            facilities: feed.facilities,
            ratings: feed.ratings,
            pilot_ratings: feed.pilot_ratings,
            military_ratings: feed.military_ratings,
            users: Vec::new(),
            auth_codes: HashMap::new(),
            access_tokens: HashMap::new(),
            seed: None,
        }
    }
}

impl From<&MockState> for DataFeed {
    fn from(state: &MockState) -> Self {
        Self {
            general: state.general_info(),
            pilots: state.pilots.clone(),
            controllers: state.controllers.clone(),
            atis: state.atis.clone(),
            servers: state.servers.clone(),
            prefiles: state.prefiles.clone(),
            facilities: state.facilities.clone(),
            ratings: state.ratings.clone(),
            pilot_ratings: state.pilot_ratings.clone(),
            military_ratings: state.military_ratings.clone(),
        }
    }
}
