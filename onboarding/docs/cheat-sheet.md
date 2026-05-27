---
sidebar_position: 50
title: Local Development Cheat Sheet
description: Every command needed to clone, build, test, format, lint, bench, and run the canonical example against zcash/orchard.
---

# Local Development Cheat Sheet

Quick reference of every command a contributor needs. Anchored to
the pin
[`f8915bc`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669)
(`0.13.1`). For the full discussion, see
[Chapter 2 (Build, Test, and Contribute)](./02-build-test-contribute.md).

## Clone and Toolchain

```bash
git clone https://github.com/zcash/orchard
cd orchard
git checkout 0.13.1
# rustup will install rust 1.85.1 automatically from rust-toolchain.toml
cargo --version
```

## Build

| Goal                                  | Command                                                           |
| ------------------------------------- | ----------------------------------------------------------------- |
| Default features                      | `cargo build`                                                     |
| All features                          | `cargo build --all-features`                                      |
| No defaults (no `circuit`, no `std`)  | `cargo build --no-default-features`                               |
| `no_std` for wasm                     | `cargo build --no-default-features --target wasm32-wasip1`        |
| `no_std` for embedded ARM             | `cargo build --no-default-features --target thumbv7em-none-eabihf`|
| Release                               | `cargo build --release`                                           |

## Test

| Goal                              | Command                                                          |
| --------------------------------- | ---------------------------------------------------------------- |
| Full test matrix as CI            | `cargo test --verbose`                                           |
| Library tests only                | `cargo test --lib`                                               |
| Integration tests only            | `cargo test --test builder`                                      |
| Doctests                          | `cargo test --doc`                                               |
| One module                        | `cargo test --lib keys::tests`                                   |
| One test with output              | `cargo test --lib keys::tests::foo -- --nocapture`               |
| Skip slow circuit tests           | `cargo test --no-default-features --features std`                |
| Release-speed full suite          | `cargo test --release`                                           |

## Format and Lint

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

A pre-push hook running these two is the cheapest way to avoid CI
round trips.

## Benchmarks

```bash
cargo bench --bench circuit -- --quick     # Action circuit prover / verifier
cargo bench --bench note_decryption        # ZIP 212 decryption
cargo bench --bench small                  # micro-benchmarks
```

Source:
[`benches/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/benches/circuit.rs),
[`benches/note_decryption.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/benches/note_decryption.rs),
[`benches/small.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/benches/small.rs).

## Docs

```bash
cargo doc --no-deps --open
# All features (matches docs.rs):
cargo doc --no-deps --all-features --open
```

The `rustdoc-args` injected by `package.metadata.docs.rs` in
[`Cargo.toml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/Cargo.toml)
embeds the KaTeX header
[`katex-header.html`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/katex-header.html)
so math in rustdoc renders correctly.

## Dependency Audit

```bash
cargo install cargo-audit       # one-time
cargo audit
cargo tree -d                   # find duplicate transitive deps
cargo tree -e features          # see feature propagation
```

## Common Bash Loops

```bash
# Run the integration test, retrying on the slow first build.
cargo test --release --test builder

# Verify no diff after format, as the lints workflow does.
cargo fmt --all && git diff --exit-code

# Rebuild after a Cargo.lock bump, mirroring the "latest deps" CI job.
rm Cargo.lock && cargo build --all-features
```

## CI Workflow Reference

CI definitions live in
[`.github/workflows/`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/.github/workflows).
Local equivalents:

| CI job              | Local command                                                           |
| ------------------- | ----------------------------------------------------------------------- |
| `test`              | `cargo test --verbose`                                                  |
| `build-latest`      | `rm Cargo.lock && cargo build --all-features --verbose`                 |
| `build-nostd`       | `cargo build --release --no-default-features --target wasm32-wasip1`    |
| `lints-stable.fmt`  | `cargo fmt --all -- --check`                                            |
| `lints-stable.clippy` | `cargo clippy --all-targets --all-features -- -D warnings`            |
| `book`              | (separate mdBook setup; not required for contributing to the crate)     |

## Useful gh Commands

```bash
# Browse open issues.
gh issue list --state open --limit 30 --repo zcash/orchard

# Read an issue with comments.
gh issue view 463 --repo zcash/orchard --comments

# Read a PR with all review comments.
gh pr view 496 --repo zcash/orchard --comments
```
