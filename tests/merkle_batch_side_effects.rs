//! Checks that small Merkle hash batches leave process-wide state untouched.
//!
//! This file is its own test binary, so it runs in its own process, and no other test can
//! build the Sinsemilla tables or the global rayon thread pool before it does. It contains
//! exactly one `#[test]` function, because its checks read process-wide state and must run
//! in a fixed order.

use std::iter;

use incrementalmerkletree::{Hashable, Level};
use orchard::tree::MerkleHashOrchard;

/// The personalization of `MerkleCRH^Orchard`.
///
/// It is repeated here because `orchard::constants` is private.
#[cfg(feature = "runtime-precomputation")]
const MERKLE_CRH_PERSONALIZATION: &str = "z.cash:Orchard-MerkleCRH";

/// The table coverage that `HashDomain::precomputed_message_words` reports before the
/// Sinsemilla tables are built.
#[cfg(feature = "runtime-precomputation")]
const UNBUILT_TABLE_WORDS: usize = 0;

#[test]
fn small_batches_build_neither_tables_nor_the_global_thread_pool() {
    let level = Level::ZERO;

    assert_eq!(
        MerkleHashOrchard::combine_pairs(level, iter::empty()),
        Vec::<MerkleHashOrchard>::new(),
        "combine_pairs of no pairs"
    );
    assert_eq!(
        MerkleHashOrchard::combine_batch(level, iter::empty()),
        Vec::<MerkleHashOrchard>::new(),
        "combine_batch of no pairs"
    );

    #[cfg(feature = "runtime-precomputation")]
    assert_eq!(
        sinsemilla::HashDomain::new(MERKLE_CRH_PERSONALIZATION).precomputed_message_words(),
        UNBUILT_TABLE_WORDS,
        "an empty batch built the Sinsemilla tables"
    );

    let leaf = MerkleHashOrchard::empty_leaf();
    assert_eq!(
        MerkleHashOrchard::combine_pairs(level, [(&leaf, &leaf)]),
        [MerkleHashOrchard::combine(level, &leaf, &leaf)],
        "combine_pairs of one pair"
    );

    #[cfg(feature = "multicore")]
    {
        let global_pool = rayon::ThreadPoolBuilder::new().build_global();
        assert!(
            global_pool.is_ok(),
            "a one-pair batch built the global rayon thread pool: {global_pool:?}"
        );
    }
}
