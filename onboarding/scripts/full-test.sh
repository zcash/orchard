#!/usr/bin/env bash
# Run the full orchard test suite, as CI does. Exercises unit tests inline
# in every src/*.rs file plus the integration test in tests/builder.rs and
# the JSON test vectors wired through src/test_vectors.rs.
set -euo pipefail

cargo test --verbose
