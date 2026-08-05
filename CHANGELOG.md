# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0]

### Breaking changes


- [**breaking**] Shrink and harden the public API ([#5](https://github.com/MorpheusXAUT/vatsim-api/pull/5)) - ([13a2281](https://github.com/MorpheusXAUT/vatsim-api/commit/13a22810a2e2ed93ec859b8246e5d9dfbaf1a256))

### Features


- Rewrite the vatsim-mock CLI with clap and graceful shutdown ([#8](https://github.com/MorpheusXAUT/vatsim-api/pull/8)) - ([f98ebd0](https://github.com/MorpheusXAUT/vatsim-api/commit/f98ebd04b2bc7a6c6d157cb81bb2ee9d34a7dc3d))
- Align the mock server with the live VATSIM API ([#7](https://github.com/MorpheusXAUT/vatsim-api/pull/7)) - ([f9aac72](https://github.com/MorpheusXAUT/vatsim-api/commit/f9aac729b9855710a9ef29037abe2f534765f929))
- Add login_hint to Connect authorize endpoint - ([299fb61](https://github.com/MorpheusXAUT/vatsim-api/commit/299fb617d343c1c83f9f4fea040f0b66afe5ae7e))
- Add VATSIM Connect types and mock endpoints - ([acef3ef](https://github.com/MorpheusXAUT/vatsim-api/commit/acef3efa5f130208dc38d7d2e768af83e88c43c3))
- Add basic mock server with datafeed/slurper endpoints and CRUD API - ([defb4df](https://github.com/MorpheusXAUT/vatsim-api/commit/defb4dffcbac221a5202e460151b9ea0bbb8ab9a))
- Add client for fetching datafeed and slurper userinfo - ([b593388](https://github.com/MorpheusXAUT/vatsim-api/commit/b593388516522a67037f0ca389ad685080105fb4))
- Add datafeed and slurper types - ([99c3e65](https://github.com/MorpheusXAUT/vatsim-api/commit/99c3e6504832ef0d3aa328c90f23d2beafae2589))

### Bug Fixes


- Fix names for ATC rating and facility - ([7b2d86b](https://github.com/MorpheusXAUT/vatsim-api/commit/7b2d86bd11661db7c8d24ab23de3de03fcb0a6d3))

### Security


- Prepare the README, changelog and crate metadata for crates.io ([#10](https://github.com/MorpheusXAUT/vatsim-api/pull/10)) - ([d26c153](https://github.com/MorpheusXAUT/vatsim-api/commit/d26c153c41ea680b774eea7a89798716d072ac93))

### Documentation


- Document every public item and enable missing_docs ([#9](https://github.com/MorpheusXAUT/vatsim-api/pull/9)) - ([cefdbe3](https://github.com/MorpheusXAUT/vatsim-api/commit/cefdbe391984250d20a1814f16246902f6b2936a))

### Deps


- Add chrono feature gate - ([e652cbf](https://github.com/MorpheusXAUT/vatsim-api/commit/e652cbf795771a408a9af23c58e358b79992e9c6))

