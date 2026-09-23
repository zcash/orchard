# Orchard - Agent Guidelines

Conventions for working in this crate. They mirror the shared conventions in the
`librustzcash` repository's `AGENTS.md`; when a convention changes in one repo,
keep the other aligned. Orchard is `#![no_std]` (the `std` feature is optional)
and is a payment-protocol crate, so some librustzcash conventions about database
/ storage layout do not apply, but the ones below do.

## Code Conventions

- **Never use magic numbers.** Do not inline a bare numeric (or string) literal
  whose meaning is not obvious from context. Give it a `const` with a
  doc-commented rationale, and reuse the protocol's own named constants rather
  than re-deriving their values. This applies to production code, tests, and
  fixtures alike.

- **Public APIs use semantic types, never bare primitives.** A `pub` function,
  trait method, field accessor, or constructor must not represent a domain
  quantity as a bare integer, byte array, or scalar: note values are
  `NoteValue`, value sums are `ValueSum`, tree roots are `Anchor`, commitments
  are `ValueCommitment` / `ExtractedNoteCommitment`, and so on. Reuse the crate's
  existing newtypes, or introduce one when none exists. Bare primitives are
  acceptable only for genuinely unitless quantities (counts, indices) and in
  module-internal arithmetic, converting at the public boundary.

- **Keep domain types whole, and convert only at the wire edge.** A newtype
  hides its inner primitive (private field plus a constructor and accessor), so
  no caller can pass a raw scalar where a value or an anchor is meant.
  Collections and options carry the newtype too (`Vec<NoteValue>`, never
  `Vec<u64>`). Down-convert to a bare primitive in exactly one place, the
  serialization / wire boundary, and convert straight back on read.

- **Canonical binary serialization lives with the type.** A type's on-wire byte
  format is a property of the type, not of any one consumer: define it next to
  the definition (`read<R: Read>(r) -> io::Result<Self>` and
  `write<W: Write>(&self, w) -> io::Result<()>`) over `corez::io::{Read, Write}`,
  so it stays `no_std`. Do NOT hand-roll a bespoke byte codec for an orchard type
  inside a downstream crate, and do NOT use `serde` for the canonical consensus
  binary format (reserve `serde` for JSON / structured metadata such as PCZT
  proprietary fields and test vectors).

- **A `proptest` strategy lives with the type it generates.** An `arb_*` strategy
  belongs in that type's module, in a `pub mod testing` gated
  `#[cfg(any(test, feature = "test-dependencies"))]` (e.g. `arb_note` in `note`,
  `arb_spending_key` in `keys`, `arb_action` in `action`, `arb_spendable_note` in
  `builder`), NOT redefined in each consumer's test module. Before writing an
  `arb_*`, search this repo (and the wider ecosystem, e.g.
  `zcash_protocol::value::testing::arb_zatoshis`) for an existing one and reuse
  it; compose canonical leaf strategies into strategies for larger types rather
  than re-deriving the leaves. Tests receive the generated values as inputs (the
  strategy is the fixture) and focus on assertions. Downstream crates reuse these
  strategies through the `test-dependencies` feature.

## Testing

- Property tests use `proptest`. Expose reusable strategies through the
  `test-dependencies` feature in `pub mod testing` modules (see the strategy
  convention above), so other crates in the ecosystem reuse them instead of
  re-deriving fixtures. A `#[test]` that hand-builds the values it exercises is a
  smell: extract the value-building into an `arb_*` and drive the test with it.
