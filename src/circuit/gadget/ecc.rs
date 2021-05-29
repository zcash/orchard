//! Gadgets for elliptic curve operations.

use ff::Field;
use std::fmt::Debug;

use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Chip, Layouter},
    plonk::Error,
};

pub mod chip;

/// The set of circuit instructions required to use the ECC gadgets.
pub trait EccInstructions<C: CurveAffine>: Chip<C::Base> {
    /// Variable representing an element of the elliptic curve's base field, that
    /// is used as a scalar in variable-base scalar mul.
    ///
    /// It is not true in general that a scalar field element fits in a curve's
    /// base field, and in particular it is untrue for the Pallas curve, whose
    /// scalar field `Fq` is larger than its base field `Fp`.
    ///
    /// However, the only use of variable-base scalar mul in the Orchard protocol
    /// is in deriving diversified addresses `[ivk] g_d`,  and `ivk` is guaranteed
    /// to be in the base field of the curve. (See non-normative notes in
    /// https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents.)
    type ScalarVar: Clone + Debug;
    /// Variable representing a full-width element of the elliptic curve's
    /// scalar field, to be used for fixed-base scalar mul.
    type ScalarFixed: Clone + Debug;
    /// Variable representing a signed short element of the elliptic curve's
    /// scalar field, to be used for fixed-base scalar mul.
    ///
    /// A `ScalarFixedShort` must be in the range [-(2^64 - 1), 2^64 - 1].
    type ScalarFixedShort: Clone + Debug;
    /// Variable representing an elliptic curve point.
    type Point: Clone + Debug;
    /// Variable representing the affine short Weierstrass x-coordinate of an
    /// elliptic curve point.
    type X: Clone + Debug;
    /// Variable representing the set of fixed bases in the circuit.
    type FixedPoints: Clone + Debug;
    /// Variable representing the set of fixed bases to be used in scalar
    /// multiplication with a short signed exponent.
    type FixedPointsShort: Clone + Debug;
    /// Variable representing a fixed elliptic curve point (constant in the circuit).
    type FixedPoint: Clone + Debug;
    /// Variable representing a fixed elliptic curve point (constant in the circuit)
    /// to be used in scalar multiplication with a short signed exponent.
    type FixedPointShort: Clone + Debug;

    /// Witnesses the given base field element as a private input to the circuit
    /// for variable-base scalar mul.
    fn witness_scalar_var(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Base>,
    ) -> Result<Self::ScalarVar, Error>;

    /// Witnesses the given full-width scalar as a private input to the circuit
    /// for fixed-base scalar mul.
    fn witness_scalar_fixed(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixed, Error>;

    /// Witnesses the given signed short scalar as a private input to the circuit
    /// for fixed-base scalar mul.
    fn witness_scalar_fixed_short(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixedShort, Error>;

    /// Witnesses the given point as a private input to the circuit.
    fn witness_point(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C>,
    ) -> Result<Self::Point, Error>;

    /// Extracts the x-coordinate of a point.
    fn extract_p(point: &Self::Point) -> &Self::X;

    /// Returns a fixed point that had been previously loaded into the circuit.
    /// The pre-loaded cells are used to set up equality constraints in other
    /// parts of the circuit where the fixed base is used.
    fn get_fixed(&self, fixed_points: Self::FixedPoints) -> Result<Self::FixedPoint, Error>;

    /// Returns a fixed point to be used in scalar multiplication with a signed
    /// short exponent.
    fn get_fixed_short(
        &self,
        fixed_points: Self::FixedPointsShort,
    ) -> Result<Self::FixedPointShort, Error>;

    /// Performs incomplete point addition, returning `a + b`.
    ///
    /// This returns an error in exceptional cases.
    fn add_incomplete(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error>;

    /// Performs complete point addition, returning `a + b`.
    fn add(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error>;

    /// Performs variable-base scalar multiplication, returning `[scalar] base`.
    /// Multiplication of the identity [a] 𝒪 returns an error.
    fn mul(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarVar,
        base: &Self::Point,
    ) -> Result<Self::Point, Error>;

    /// Performs fixed-base scalar multiplication using a full-width scalar, returning `[scalar] base`.
    fn mul_fixed(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarFixed,
        base: &Self::FixedPoint,
    ) -> Result<Self::Point, Error>;

    /// Performs fixed-base scalar multiplication using a short signed scalar, returning `[scalar] base`.
    fn mul_fixed_short(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarFixedShort,
        base: &Self::FixedPointShort,
    ) -> Result<Self::Point, Error>;
}

/// An element of the given elliptic curve's base field, that is used as a scalar
/// in variable-base scalar mul.
///
/// It is not true in general that a scalar field element fits in a curve's
/// base field, and in particular it is untrue for the Pallas curve, whose
/// scalar field `Fq` is larger than its base field `Fp`.
///
/// However, the only use of variable-base scalar mul in the Orchard protocol
/// is in deriving diversified addresses `[ivk] g_d`,  and `ivk` is guaranteed
/// to be in the base field of the curve. (See non-normative notes in
/// https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents.)
#[derive(Debug)]
pub struct ScalarVar<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> {
    chip: EccChip,
    inner: EccChip::ScalarVar,
}

impl<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> ScalarVar<C, EccChip> {
    /// Constructs a new ScalarVar with the given value.
    pub fn new(
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        value: Option<C::Base>,
    ) -> Result<Self, Error> {
        chip.witness_scalar_var(&mut layouter, value)
            .map(|inner| ScalarVar { chip, inner })
    }
}

/// A full-width element of the given elliptic curve's scalar field, to be used for fixed-base scalar mul.
#[derive(Debug)]
pub struct ScalarFixed<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> {
    chip: EccChip,
    inner: EccChip::ScalarFixed,
}

impl<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> ScalarFixed<C, EccChip> {
    /// Constructs a new ScalarFixed with the given value.
    pub fn new(
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self, Error> {
        chip.witness_scalar_fixed(&mut layouter, value)
            .map(|inner| ScalarFixed { chip, inner })
    }
}

/// A signed short element of the given elliptic curve's scalar field, to be used for fixed-base scalar mul.
#[derive(Debug)]
pub struct ScalarFixedShort<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> {
    chip: EccChip,
    inner: EccChip::ScalarFixedShort,
}

impl<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq>
    ScalarFixedShort<C, EccChip>
{
    /// Constructs a new ScalarFixedShort with the given value.
    ///
    /// # Panics
    ///
    /// The short scalar must be in the range [-(2^64 - 1), (2^64 - 1)].
    pub fn new(
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self, Error> {
        // Check that the scalar is in the range [-(2^64 - 1), (2^64 - 1)]
        if let Some(value) = value {
            let mut sign = C::Scalar::one();

            // T = (p-1) / 2
            let t = (C::Scalar::zero() - C::Scalar::one()) * C::Scalar::TWO_INV;

            if value > t {
                sign = -sign;
            }
            let magnitude = value * sign;
            assert!(magnitude < C::Scalar::from_u128(1 << 64));
        }

        chip.witness_scalar_fixed_short(&mut layouter, value)
            .map(|inner| ScalarFixedShort { chip, inner })
    }
}

/// An elliptic curve point over the given curve.
#[derive(Copy, Clone, Debug)]
pub struct Point<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> {
    chip: EccChip,
    inner: EccChip::Point,
}

impl<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> Point<C, EccChip> {
    /// Constructs a new point with the given value.
    pub fn new(
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        value: Option<C>,
    ) -> Result<Self, Error> {
        let point = chip.witness_point(&mut layouter, value);
        point.map(|inner| Point { chip, inner })
    }

    /// Extracts the x-coordinate of a point.
    pub fn extract_p(&self) -> X<C, EccChip> {
        X::from_inner(self.chip.clone(), EccChip::extract_p(&self.inner).clone())
    }

    /// Wraps the given point (obtained directly from an instruction) in a gadget.
    pub fn from_inner(chip: EccChip, inner: EccChip::Point) -> Self {
        Point { chip, inner }
    }

    /// Returns `self + other` using complete addition.
    pub fn add(&self, mut layouter: impl Layouter<C::Base>, other: &Self) -> Result<Self, Error> {
        assert_eq!(self.chip, other.chip);
        self.chip
            .add(&mut layouter, &self.inner, &other.inner)
            .map(|inner| Point {
                chip: self.chip.clone(),
                inner,
            })
    }

    /// Returns `self + other` using incomplete addition.
    pub fn add_incomplete(
        &self,
        mut layouter: impl Layouter<C::Base>,
        other: &Self,
    ) -> Result<Self, Error> {
        assert_eq!(self.chip, other.chip);
        self.chip
            .add_incomplete(&mut layouter, &self.inner, &other.inner)
            .map(|inner| Point {
                chip: self.chip.clone(),
                inner,
            })
    }

    /// Returns `[by] self`.
    pub fn mul(
        &self,
        mut layouter: impl Layouter<C::Base>,
        by: &ScalarVar<C, EccChip>,
    ) -> Result<Self, Error> {
        assert_eq!(self.chip, by.chip);
        self.chip
            .mul(&mut layouter, &by.inner, &self.inner)
            .map(|inner| Point {
                chip: self.chip.clone(),
                inner,
            })
    }
}

/// The affine short Weierstrass x-coordinate of an elliptic curve point over the
/// given curve.
#[derive(Debug)]
pub struct X<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> {
    chip: EccChip,
    inner: EccChip::X,
}

impl<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> X<C, EccChip> {
    /// Wraps the given x-coordinate (obtained directly from an instruction) in a gadget.
    pub fn from_inner(chip: EccChip, inner: EccChip::X) -> Self {
        X { chip, inner }
    }
}

/// A constant elliptic curve point over the given curve, for which window tables have
/// been provided to make scalar multiplication more efficient.
#[derive(Clone, Debug)]
pub struct FixedPoint<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> {
    chip: EccChip,
    inner: EccChip::FixedPoint,
}

impl<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> FixedPoint<C, EccChip> {
    /// Gets a reference to the specified fixed point in the circuit.
    pub fn get(chip: EccChip, point: EccChip::FixedPoints) -> Result<Self, Error> {
        chip.get_fixed(point)
            .map(|inner| FixedPoint { chip, inner })
    }

    /// Returns `[by] self`.
    pub fn mul(
        &self,
        mut layouter: impl Layouter<C::Base>,
        by: &ScalarFixed<C, EccChip>,
    ) -> Result<Point<C, EccChip>, Error> {
        assert_eq!(self.chip, by.chip);
        self.chip
            .mul_fixed(&mut layouter, &by.inner, &self.inner)
            .map(|inner| Point {
                chip: self.chip.clone(),
                inner,
            })
    }
}

/// A constant elliptic curve point over the given curve, used in scalar multiplication
/// with a short signed exponent
#[derive(Clone, Debug)]
pub struct FixedPointShort<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> {
    chip: EccChip,
    inner: EccChip::FixedPointShort,
}

impl<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Debug + Eq> FixedPointShort<C, EccChip> {
    /// Gets a reference to the specified fixed point in the circuit.
    pub fn get(chip: EccChip, point: EccChip::FixedPointsShort) -> Result<Self, Error> {
        chip.get_fixed_short(point)
            .map(|inner| FixedPointShort { chip, inner })
    }

    /// Returns `[by] self`.
    pub fn mul(
        &self,
        mut layouter: impl Layouter<C::Base>,
        by: &ScalarFixedShort<C, EccChip>,
    ) -> Result<Point<C, EccChip>, Error> {
        assert_eq!(self.chip, by.chip);
        self.chip
            .mul_fixed_short(&mut layouter, &by.inner, &self.inner)
            .map(|inner| Point {
                chip: self.chip.clone(),
                inner,
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::constants;
    use group::{Curve, Group};
    use halo2::{
        arithmetic::CurveAffine,
        circuit::{layouter::SingleChipLayouter, Layouter},
        dev::MockProver,
        pasta::pallas,
        plonk::{Assignment, Circuit, ConstraintSystem, Error},
    };

    use super::chip::{EccChip, EccConfig, OrchardFixedBases, OrchardFixedBasesShort};

    struct MyCircuit<C: CurveAffine> {
        _marker: std::marker::PhantomData<C>,
    }

    #[allow(non_snake_case)]
    impl<C: CurveAffine> Circuit<C::Base> for MyCircuit<C> {
        type Config = EccConfig;

        fn configure(meta: &mut ConstraintSystem<C::Base>) -> Self::Config {
            let advices = [
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
            ];

            EccChip::<C>::configure(meta, advices)
        }

        fn synthesize(
            &self,
            cs: &mut impl Assignment<C::Base>,
            config: Self::Config,
        ) -> Result<(), Error> {
            let mut layouter = SingleChipLayouter::new(cs)?;
            let loaded = EccChip::<C>::load();
            let chip = EccChip::construct(config, loaded);

            // Generate a random point P
            let p_val = C::CurveExt::random(rand::rngs::OsRng).to_affine(); // P
            let p = super::Point::new(chip.clone(), layouter.namespace(|| "point"), Some(p_val))?;
            let p_neg = -p_val;
            let p_neg =
                super::Point::new(chip.clone(), layouter.namespace(|| "point"), Some(p_neg))?;

            // Generate a random point Q
            let q_val = C::CurveExt::random(rand::rngs::OsRng).to_affine(); // P
            let q = super::Point::new(chip.clone(), layouter.namespace(|| "point"), Some(q_val))?;

            // Make sure P and Q are not the same point.
            assert_ne!(p_val, q_val);

            // Generate a (0,0) point to be used in other tests.
            let zero = p.add(layouter.namespace(|| "P + (-P)"), &p_neg)?;

            // Test complete addition
            {
                super::chip::add::tests::test_add(
                    chip.clone(),
                    layouter.namespace(|| "complete addition"),
                    &zero,
                    p_val,
                    &p,
                    q_val,
                    &q,
                    &p_neg,
                )?;
            }

            // Test incomplete addition
            {
                super::chip::add_incomplete::tests::test_add_incomplete(
                    layouter.namespace(|| "incomplete addition"),
                    &zero,
                    &p,
                    &q,
                    &p_neg,
                )?;
            }

            // Test variable-base scalar multiplication
            {
                super::chip::mul::tests::test_mul(
                    chip.clone(),
                    layouter.namespace(|| "variable-base scalar mul"),
                    &zero,
                    &p,
                )?;
            }

            // Test full-width fixed-base scalar multiplication
            {
                let nullifier_k = super::FixedPoint::get(
                    chip.clone(),
                    OrchardFixedBases::NullifierK(constants::nullifier_k::generator()),
                )?;
                super::chip::mul_fixed::full_width::tests::test_mul_fixed(
                    chip.clone(),
                    layouter.namespace(|| "full-width fixed-base scalar mul"),
                    nullifier_k,
                )?;
            }

            // Test signed short fixed-base scalar multiplication
            {
                let value_commit_v = super::FixedPointShort::get(
                    chip.clone(),
                    OrchardFixedBasesShort(constants::value_commit_v::generator()),
                )?;
                super::chip::mul_fixed::short::tests::test_mul_fixed_short(
                    chip.clone(),
                    layouter.namespace(|| "full-width fixed-base scalar mul"),
                    value_commit_v,
                )?;
            }

            Ok(())
        }
    }

    #[test]
    fn ecc() {
        let k = 11;
        let circuit = MyCircuit::<pallas::Affine> {
            _marker: std::marker::PhantomData,
        };
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()))
    }
}
