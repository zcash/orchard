use criterion::{measurement::Measurement, BenchmarkGroup, Criterion};

use orchard::note_encryption::IronwoodVersion;
use orchard::{
    bundle::BundleVersion,
    note_encryption::{DomainVersion, OrchardVersion, ZSAVersion},
};

/// Marker type selecting the Orchard V2 protocol for the benchmarks.
///
/// Orchard V3 protocol is not benchmarked. In benches, actions are built in a way that gives the
/// sender and receiver different addresses, which is incompatible with Orchard V3 protocol.
pub(crate) struct OrchardV2;
/// Marker type selecting the Ironwood V3 protocol for the benchmarks.
pub(crate) struct IronwoodV3;
/// Marker type selecting the ZSA protocol for the benchmarks.
pub(crate) struct Zsa;

pub(crate) trait OrchardFlavorBench {
    const DEFAULT_BUNDLE_VERSION: BundleVersion;
    type DomainVersion: DomainVersion;

    fn benchmark_group<'a, M: Measurement>(
        c: &'a mut Criterion<M>,
        group_name: &str,
    ) -> BenchmarkGroup<'a, M>;
}

impl OrchardFlavorBench for OrchardV2 {
    const DEFAULT_BUNDLE_VERSION: BundleVersion = BundleVersion::orchard_v2();
    type DomainVersion = OrchardVersion;

    fn benchmark_group<'a, M: Measurement>(
        c: &'a mut Criterion<M>,
        group_name: &str,
    ) -> BenchmarkGroup<'a, M> {
        c.benchmark_group(format!("[OrchardV2] {group_name}"))
    }
}

impl OrchardFlavorBench for IronwoodV3 {
    const DEFAULT_BUNDLE_VERSION: BundleVersion = BundleVersion::ironwood_v3();
    type DomainVersion = IronwoodVersion;

    fn benchmark_group<'a, M: Measurement>(
        c: &'a mut Criterion<M>,
        group_name: &str,
    ) -> BenchmarkGroup<'a, M> {
        c.benchmark_group(format!("[IronwoodV3] {group_name}"))
    }
}

impl OrchardFlavorBench for Zsa {
    const DEFAULT_BUNDLE_VERSION: BundleVersion = BundleVersion::zsa();
    type DomainVersion = ZSAVersion;

    fn benchmark_group<'a, M: Measurement>(
        c: &'a mut Criterion<M>,
        group_name: &str,
    ) -> BenchmarkGroup<'a, M> {
        c.benchmark_group(format!("[Zsa] {group_name}"))
    }
}
