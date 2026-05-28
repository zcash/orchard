#!/usr/bin/env bash
# Faster iteration during development. Runs a narrower slice of the test
# suite than the full `cargo test --verbose`: the keys module only, then
# a single test by name, then the std-only feature set that skips the
# slow circuit tests.
set -euo pipefail

cargo test --lib keys::tests
cargo test --lib keys::tests::test_address_components -- --nocapture
cargo test --lib --no-default-features --features std
