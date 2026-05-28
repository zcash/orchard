#!/usr/bin/env bash
# Clone the orchard crate at the pinned tag and run the first cargo build.
# Expect about three to five minutes on a warm machine. Subsequent builds
# are incremental.
set -euo pipefail

git clone https://github.com/zcash/orchard
cd orchard
git checkout 0.13.1
cargo build
