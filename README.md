# orchard [![Crates.io](https://img.shields.io/crates/v/orchard.svg)](https://crates.io/crates/orchard) #

Requires Rust 1.88+.

## Documentation

- [The Orchard Book](https://zcash.github.io/orchard/)
- [Crate documentation](https://docs.rs/orchard)

## Batch Merkle hashing

`MerkleHashOrchard::combine_batch` and `Hashable::combine_pairs` hash borrowed
pairs of public tree nodes in input order. They are variable time. The
`multicore` feature splits wide `combine_pairs` batches across Rayon tasks and
requires `std`.

The optional `runtime-precomputation` feature enables Sinsemilla's `merkle-table`
feature and position-weighted tables, built on first use. It guarantees coverage
of Orchard's 52-word Merkle messages with at least `3473408` bytes (53 rows of
64 KiB) of generator tables on every target. The build-time
`SINSEMILLA_TABLE_LIMIT` setting can increase this budget, but cannot reduce it
below the Merkle minimum. Table metadata and hashing scratch space require
additional memory.

To benchmark batches with tables enabled:

```sh
cargo bench --features runtime-precomputation --bench merkle_crh
```

`runtime-precomputation` does not itself require `std`.

## `no_std` compatibility

In order to take advantage of `no_std` builds, downstream users of this crate
must enable the `spin_no_std` feature of the `lazy_static` crate. This is
needed because the `--no-default-features` build of `lazy_static` still relies
on `std`.

## License

Copyright 2020-2023 The Electric Coin Company.

All code in this workspace is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
