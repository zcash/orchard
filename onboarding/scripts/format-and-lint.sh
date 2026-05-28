#!/usr/bin/env bash
# Format and lint the orchard crate. The format step is idempotent.
# Clippy uses the workspace default configuration; the crate does not
# ship a clippy.toml at the pin.
set -euo pipefail

cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
