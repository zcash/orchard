# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- MSRV is now 1.54.0.
- Bumped dependencies to `pasta_curves 0.3`.
- The following methods now have an additional `rng: impl RngCore` argument:
  - `orchard::builder::Bundle::create_proof`
  - `orchard::builder::InProgress::create_proof`
  - `orchard::circuit::Proof::create`

### Removed
- `orchard::value::ValueSum::from_raw`

## [0.1.0-beta.1] - 2021-12-17
Initial release!
