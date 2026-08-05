# vatsim-api

[![Crates.io](https://img.shields.io/crates/v/vatsim-api.svg)](https://crates.io/crates/vatsim-api)
[![Documentation](https://docs.rs/vatsim-api/badge.svg)](https://docs.rs/vatsim-api)
[![License: Apache-2.0 OR MIT](https://img.shields.io/badge/License-Apache--2.0%20OR%20MIT-blue.svg)](#license)

Rust types, an async client, and a mock server for the [VATSIM](https://vatsim.net/) network APIs.

The crate covers three of the APIs documented at [vatsim.dev](https://vatsim.dev/):

- **[Data feed](https://vatsim.dev/api/data-api/get-network-data)** - a JSON snapshot of every
  pilot, controller, and ATIS station currently connected to the network.
- **[Slurper](https://vatsim.dev/api/slurper-api/get-user-info)** - a CSV endpoint returning a
  single user's active connections.
- **[Connect](https://vatsim.dev/api/connect-api)** - VATSIM's OAuth2 provider, covered by the
  types and the mock server. The client does not implement the OAuth2 flow.

## Features

- **Strongly typed** - facilities, ratings and flight rules are enums rather than magic integers,
  and deserialization accepts both the numeric IDs and the string names the APIs use
  interchangeably.
- **Async client** - data feed fetching with mirror discovery through the VATSIM status endpoint
  and a configurable response cache.
- **Mock server** - the real endpoints served from in-memory state, so integration tests never
  touch the live network. Includes a management API for driving that state at runtime.
- **Standalone binary** - `vatsim-mock` runs the same mock server as a process, for test suites
  that are not written in Rust.
- **Minimal by default** - the client, the mock and their dependency trees are all feature-gated.
  `thiserror` is the only unconditional dependency.

## Quick start

```toml
[dependencies]
vatsim-api = { version = "0.1", features = ["client"] }
```

```rust
use vatsim_api::{CachePolicy, VatsimClient};

#[tokio::main]
async fn main() -> Result<(), vatsim_api::ClientError> {
    let client = VatsimClient::new();

    let feed = client.datafeed(CachePolicy::Cached).await?;
    for controller in &feed.controllers {
        println!("{} on {}", controller.callsign, controller.frequency);
    }

    Ok(())
}
```

## Testing against the mock server

```toml
[dev-dependencies]
vatsim-api = { version = "0.1", features = ["client", "mock"] }
```

```rust
use vatsim_api::mock::MockServer;
use vatsim_api::types::datafeed::Controller;
use vatsim_api::{CachePolicy, CertificateId};

#[tokio::test]
async fn controller_shows_up_in_the_feed() {
    let handle = MockServer::builder()
        .controllers(vec![Controller {
            cid: CertificateId::new(1_000_001),
            callsign: "LOWW_TWR".to_owned(),
            frequency: "121.500".to_owned(),
            ..Default::default()
        }])
        .spawn()
        .await
        .unwrap();

    let feed = handle.client().datafeed(CachePolicy::Refresh).await.unwrap();
    assert_eq!(feed.controllers.len(), 1);

    // Take the controller offline part-way through the test.
    handle
        .state()
        .write()
        .await
        .remove_controller(CertificateId::new(1_000_001));
}
```

The server shuts down when the handle is dropped.

## Standalone mock server

```sh
cargo install vatsim-api --features mock-bin
vatsim-mock --bind 127.0.0.1:8080 --seed seed.json
```

Prebuilt binaries for Linux, macOS and Windows are attached to each
[release](https://github.com/MorpheusXAUT/vatsim-api/releases).

### Endpoints

VATSIM-compatible:

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/v3/vatsim-data.json` | Data feed, rendered from the current state |
| `GET` | `/users/info?cid=` | Slurper, as CSV |
| `GET` | `/oauth/authorize` | Auto-approves; see `login_hint` below |
| `POST` | `/oauth/token` | Exchanges an authorization code for a token |
| `GET` | `/api/user` | Connect user details, requires a bearer token |

Management API, for driving the state from tests:

| Method | Path | Notes |
| --- | --- | --- |
| `GET`, `PUT` | `/api/state` | Dump or replace the entire state |
| `POST` | `/api/reset` | Restore the state the server started with |
| `GET`, `POST` | `/api/{collection}` | List, or insert and update |
| `GET`, `PUT`, `DELETE` | `/api/{collection}/{key}` | Fetch, replace, or remove one entry |

`{collection}` is one of `pilots`, `controllers`, `atis`, `prefiles`, `servers` or `users`. The key
is the CID, except for `atis` which is keyed by callsign because one controller can run several
ATIS connections, and `servers` which is keyed by ident.

`GET /oauth/authorize` accepts a **`login_hint`** query parameter holding a CID. This is a
mock-only extension that the real Connect API does not have; it selects which seeded user to
authenticate as, instead of always picking the first one.

### Seed files

`--seed` takes a JSON object whose keys are the state's collections. Every key is optional, so a
seed file only needs to list what it cares about:

```json
{
  "controllers": [
    {
      "cid": 1000001,
      "name": "Test Controller",
      "callsign": "LOWW_TWR",
      "frequency": "121.500",
      "facility": 4,
      "rating": 5,
      "server": "MOCK",
      "visual_range": 50,
      "text_atis": [],
      "last_updated": "1970-01-01T00:00:00Z",
      "logon_time": "1970-01-01T00:00:00Z"
    }
  ],
  "users": []
}
```

The loaded state also becomes the snapshot that `POST /api/reset` restores.

## Feature flags

| Feature | Default | Description |
| --- | --- | --- |
| `serde` | yes | `Serialize`/`Deserialize` derives on all types |
| `chrono` | yes | Timestamps are `chrono::DateTime<Utc>` rather than raw strings |
| `client` | no | The async HTTP client |
| `mock` | no | The embeddable mock server |
| `mock-bin` | no | The standalone `vatsim-mock` binary |
| `tracing` | no | `tracing` spans and events from the mock server |

## Verifying release artifacts

Release binaries are built by GitHub Actions, checksummed, signed with
[cosign](https://github.com/sigstore/cosign), and covered by a build provenance attestation.

```sh
gh attest verify vatsim-mock-x86_64-unknown-linux-gnu.tar.gz --repo MorpheusXAUT/vatsim-api
sha256sum -c SHA256SUMS --ignore-missing
```

The crate published to crates.io is built from the same tag, using
[Trusted Publishing](https://crates.io/docs/trusted-publishing) so that no long-lived registry
token exists. crates.io does not yet surface provenance for `.crate` files, so the attestation
attached to the GitHub release is the only signed record of that artifact.

## Minimum supported Rust version

1.85. Raising it is a minor version bump.

## Public dependencies

These crates appear in the public API, so a major version bump of any of them is a breaking change
here:

| Dependency | Where it is public | Feature |
| --- | --- | --- |
| `chrono` | timestamp fields on the data feed types, plus the `chrono` re-export | `chrono` |
| `serde` | `Serialize`/`Deserialize` bounds on all types, and the mock's seed-file format | `serde` |
| `tokio` | `SharedState`, returned by `MockServerHandle::state()` | `mock` |
| `reqwest` | `ClientError::Http`, plus the `reqwest` re-export | `client` |

`chrono` and `reqwest` are re-exported so consumers can depend on exactly the versions this crate
was built against. `axum` is deliberately not public: the mock's router is crate-private, so the
axum version stays an implementation detail.

The data feed types are intentionally not `#[non_exhaustive]`, so they can be constructed as struct
literals in tests. They derive `Default`, so use `..Default::default()` to stay forward-compatible
when VATSIM adds a field.

## Disclaimer

This is an unofficial, community-maintained project. It is not affiliated with, endorsed by, or
supported by VATSIM.

## License

The `vatsim-api` project and all its crates and packages are dual-licensed as

- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or https://opensource.org/license/apache-2-0)
- **MIT license** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

This means you can choose to use `vatsim-api` under either the Apache-2.0 license or the MIT license.
