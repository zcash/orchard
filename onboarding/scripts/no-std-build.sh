#!/usr/bin/env bash
# Build the orchard crate against a no_std target. A new dependency on
# std-only APIs in any non-test file will break this. CI does the same
# for thumbv7em-none-eabihf.
set -euo pipefail

rustup target add wasm32-wasip1
cargo build --release --no-default-features --target wasm32-wasip1
