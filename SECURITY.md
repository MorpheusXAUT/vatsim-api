# Security policy

## Supported versions

`vatsim-api` is pre-1.0. Only the latest published version receives fixes; there are no maintained
release branches. Older versions are yanked from crates.io when a vulnerability affects them.

## Reporting a vulnerability

Please report security issues privately through
[GitHub's private vulnerability reporting](https://github.com/MorpheusXAUT/vatsim-api/security/advisories/new)
rather than opening a public issue.

Include what the issue is, which version and feature flags are affected, and how to reproduce it.
You should get an initial response within a week.

## Scope

Worth reporting:

- Anything in the types or client layers that a malicious VATSIM API response could exploit:
  panics, unbounded allocation, or infinite loops while parsing a data feed or slurper response.
- Anything that lets the mock server's management API be reached in a way its documentation says it
  cannot be.
- Supply chain problems with the released artifacts: a build provenance attestation that does not
  verify, a checksum mismatch, or a published `.crate` whose contents do not match the tagged
  source.

Out of scope:

- **The mock server is a test fixture and is not hardened.** It auto-approves every OAuth
  authorization request, does not validate client credentials, issues predictable sequential
  tokens, and exposes an unauthenticated CRUD API over its entire state. That is deliberate and
  documented. Do not run it anywhere it can be reached from an untrusted network, and do not report
  those properties as vulnerabilities.
- Denial of service against the mock server.
- Advisories in dependencies with no reachable path from this crate. Those are tracked by
  `cargo deny` in CI; open a normal issue instead.

## Release integrity

Releases are built by GitHub Actions from a tagged commit. Binaries are checksummed, signed with
cosign, and covered by a build provenance attestation; the crate is published to crates.io through
Trusted Publishing, so no long-lived registry token exists. See the README for how to verify an
artifact.
