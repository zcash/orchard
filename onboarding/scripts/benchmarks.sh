#!/usr/bin/env bash
# Run the orchard benchmarks. The circuit bench is the only one that
# exercises the proof system end to end; budget about a minute per run.
set -euo pipefail

cargo bench --bench circuit -- --quick
cargo bench --bench note_decryption
cargo bench --bench small
