#!/usr/bin/env bash
# Faster iteration during development. Each invocation runs a narrower
# slice of the test suite than `cargo test --verbose`.
set -euo pipefail

# Run only the keys module's tests.
cargo test --lib keys::tests

# Run a single test by name.
cargo test --lib keys::tests::test_address_components -- --nocapture

# Skip the slow circuit tests.
cargo test --lib --no-default-features --features std
