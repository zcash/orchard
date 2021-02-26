//! Gadget and chips for the Sinsemilla hash function.
use halo2::{
    arithmetic::CurveAffine,
    circuit::{Chip, Layouter},
    plonk::Error,
};

mod chip;
pub use chip::{SinsemillaChip, SinsemillaColumns, SinsemillaConfig};

/// The set of circuit instructions required to use the [`Sinsemilla`](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html) gadget.
pub trait SinsemillaInstructions<C: CurveAffine>: Chip<Field = C::Base> {
    type Message: IntoIterator<Item = bool>;

    fn extract(point: &C::Curve) -> C::Base;

    #[allow(non_snake_case)]
    fn Q(domain_prefix: &str) -> C::CurveExt;

    fn hash_to_point(
        layouter: &mut impl Layouter<Self>,
        domain_prefix: &str,
        message: Self::Message,
    ) -> Result<C, Error>;

    fn hash(
        layouter: &mut impl Layouter<Self>,
        domain_prefix: &str,
        message: Self::Message,
    ) -> Result<C::Base, Error>;

    fn commit(domain_prefix: &str, msg: Self::Message, r: &C::Scalar)
        -> Result<C::CurveExt, Error>;

    fn short_commit(
        domain_prefix: &str,
        msg: Self::Message,
        r: &C::Scalar,
    ) -> Result<C::Base, Error>;
}

#[test]
fn test_sinsemilla() {
    use crate::primitives::sinsemilla::HashDomain;
    use group::Curve;
    use halo2::{
        arithmetic::CurveAffine,
        circuit::layouter::SingleChip,
        pasta::{EpAffine, EqAffine},
        plonk::*,
        poly::commitment::Params,
        transcript::{Blake2bRead, Blake2bWrite},
    };
    use std::marker::PhantomData;

    /// This represents an advice column at a certain row in the ConstraintSystem
    #[derive(Copy, Clone, Debug)]
    pub struct Variable(Column<Advice>, usize);

    struct MyCircuit<C: CurveAffine> {
        message: Vec<bool>,
        _marker_c: PhantomData<C>,
    }

    impl<C: CurveAffine> Circuit<C::Base> for MyCircuit<C> {
        type Config = SinsemillaConfig;

        fn configure(meta: &mut ConstraintSystem<C::Base>) -> Self::Config {
            let columns = SinsemillaColumns::new(
                meta.fixed_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
            );

            SinsemillaChip::<C>::configure(meta, columns)
        }

        fn synthesize(
            &self,
            cs: &mut impl Assignment<C::Base>,
            config: Self::Config,
        ) -> Result<(), Error> {
            let mut layouter = SingleChip::new(cs, config)?;
            let point =
                SinsemillaChip::<C>::hash_to_point(&mut layouter, &"prefix", self.message.clone())?;

            // Check against implementation in crate::primitives::sinsemilla
            let point_ref = HashDomain::new(&"prefix")
                .hash_to_point(self.message.clone().into_iter())
                .to_affine();
            assert_eq!(format!("{:?}", point), format!("{:?}", point_ref));

            Ok(())
        }
    }

    // Initialize the polynomial commitment parameters
    let k = 10;
    let params: Params<EqAffine> = Params::new(k);
    let empty_circuit: MyCircuit<EpAffine> = MyCircuit {
        message: Vec::new(),
        _marker_c: PhantomData,
    };

    // Initialize the proving key
    let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
    let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");

    let circuit: MyCircuit<EpAffine> = MyCircuit {
        // 101101101101
        message: vec![
            true, false, true, true, false, true, true, false, true, true, false, false,
        ],
        _marker_c: PhantomData,
    };

    // Create a proof
    let mut transcript = Blake2bWrite::init(vec![]);
    create_proof(&params, &pk, &[circuit], &[&[]], &mut transcript)
        .expect("proof generation should not fail");
    let proof: Vec<u8> = transcript.finalize();

    let msm = params.empty_msm();
    let mut transcript = Blake2bRead::init(&proof[..]);
    let guard = verify_proof(&params, pk.get_vk(), msm, &[&[]], &mut transcript).unwrap();
    let msm = guard.clone().use_challenges();
    assert!(msm.eval());
}
