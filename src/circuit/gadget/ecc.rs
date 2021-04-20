//! Gadgets for elliptic curve operations.

use std::fmt;

use halo2::{
    arithmetic::CurveAffine,
    circuit::{Chip, Layouter},
    plonk::Error,
};

pub mod chip;

/// Trait allowing circuit's fixed points to be enumerated.
pub trait FixedPoints<C: CurveAffine> {}

/// The set of circuit instructions required to use the ECC gadgets.
pub trait EccInstructions<C: CurveAffine>: Chip<C::Base> {
    /// Variable representing an element of the elliptic curve's scalar field, to be used for variable-base scalar mul.
    type ScalarVar: Clone + fmt::Debug;
    /// Variable representing a full-width element of the elliptic curve's scalar field, to be used for fixed-base scalar mul.
    type ScalarFixed: Clone + fmt::Debug;
    /// Variable representing a signed short element of the elliptic curve's scalar field, to be used for fixed-base scalar mul.
    type ScalarFixedShort: Clone + fmt::Debug;
    /// Variable representing an elliptic curve point.
    type Point: Clone + fmt::Debug;
    /// Variable representing the x-coordinate of an elliptic curve point.
    type X: Clone + fmt::Debug;
    /// Variable representing the set of fixed bases in the circuit.
    type FixedPoints: FixedPoints<C>;
    /// Variable representing a fixed elliptic curve point (constant in the circuit).
    type FixedPoint: Clone + fmt::Debug;

    /// Witnesses the given scalar as a private input to the circuit for variable-based scalar mul.
    fn witness_scalar_var(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarVar, Error>;

    /// Witnesses the given full-width scalar as a private input to the circuit for fixed-based scalar mul.
    fn witness_scalar_fixed(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixed, Error>;

    /// Witnesses the given signed short scalar as a private input to the circuit for fixed-based scalar mul.
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

    /// Gets a fixed point into the circuit.
    fn get_fixed(&self, fixed_points: Self::FixedPoints) -> Result<Self::FixedPoint, Error>;

    /// Performs point addition, returning `a + b`.
    fn add(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error>;

    /// Performs complete point addition, returning `a + b`.
    fn add_complete(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error>;

    /// Performs point doubling, returning `[2] a`.
    fn double(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        a: &Self::Point,
    ) -> Result<Self::Point, Error>;

    /// Performs variable-base scalar multiplication, returning `[scalar] base`.
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
        base: &Self::FixedPoint,
    ) -> Result<Self::Point, Error>;
}

/// An element of the given elliptic curve's scalar field, to be used for variable-base scalar mul.
#[derive(Debug)]
pub struct ScalarVar<C: CurveAffine, EccChip: EccInstructions<C>> {
    inner: EccChip::ScalarVar,
}

impl<C: CurveAffine, EccChip: EccInstructions<C>> ScalarVar<C, EccChip> {
    /// Constructs a new ScalarVar with the given value.
    pub fn new(
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self, Error> {
        chip.witness_scalar_var(&mut layouter, value)
            .map(|inner| ScalarVar { inner })
    }
}

/// A full-width element of the given elliptic curve's scalar field, to be used for fixed-base scalar mul.
#[derive(Debug)]
pub struct ScalarFixed<C: CurveAffine, EccChip: EccInstructions<C>> {
    inner: EccChip::ScalarFixed,
}

impl<C: CurveAffine, EccChip: EccInstructions<C>> ScalarFixed<C, EccChip> {
    /// Constructs a new ScalarFixed with the given value.
    pub fn new(
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self, Error> {
        chip.witness_scalar_fixed(&mut layouter, value)
            .map(|inner| ScalarFixed { inner })
    }
}

/// A signed short element of the given elliptic curve's scalar field, to be used for fixed-base scalar mul.
#[derive(Debug)]
pub struct ScalarFixedShort<C: CurveAffine, EccChip: EccInstructions<C>> {
    inner: EccChip::ScalarFixedShort,
}

impl<C: CurveAffine, EccChip: EccInstructions<C>> ScalarFixedShort<C, EccChip> {
    /// Constructs a new ScalarFixedShort with the given value.
    pub fn new(
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self, Error> {
        chip.witness_scalar_fixed_short(&mut layouter, value)
            .map(|inner| ScalarFixedShort { inner })
    }
}

/// An elliptic curve point over the given curve.
#[derive(Debug)]
pub struct Point<C: CurveAffine, EccChip: EccInstructions<C>> {
    inner: EccChip::Point,
}

impl<C: CurveAffine, EccChip: EccInstructions<C>> Point<C, EccChip> {
    /// Constructs a new point with the given value.
    pub fn new(
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        value: Option<C>,
    ) -> Result<Self, Error> {
        chip.witness_point(&mut layouter, value)
            .map(|inner| Point { inner })
    }

    /// Extracts the x-coordinate of a point.
    pub fn extract_p(&self) -> X<C, EccChip> {
        X::from_inner(EccChip::extract_p(&self.inner).clone())
    }

    /// Wraps the given point (obtained directly from an instruction) in a gadget.
    pub fn from_inner(inner: EccChip::Point) -> Self {
        Point { inner }
    }

    /// Returns `self + other`.
    pub fn add(
        &self,
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        other: &Self,
    ) -> Result<Self, Error> {
        chip.add(&mut layouter, &self.inner, &other.inner)
            .map(|inner| Point { inner })
    }

    /// Returns `[2] self`.
    pub fn double(
        &self,
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
    ) -> Result<Self, Error> {
        chip.double(&mut layouter, &self.inner)
            .map(|inner| Point { inner })
    }

    /// Returns `[by] self`.
    pub fn mul(
        &self,
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        by: &ScalarVar<C, EccChip>,
    ) -> Result<Self, Error> {
        chip.mul(&mut layouter, &by.inner, &self.inner)
            .map(|inner| Point { inner })
    }
}

/// The x-coordinate of an elliptic curve point over the given curve.
#[derive(Debug)]
pub struct X<C: CurveAffine, EccChip: EccInstructions<C>> {
    inner: EccChip::X,
}

impl<C: CurveAffine, EccChip: EccInstructions<C>> X<C, EccChip> {
    /// Wraps the given x-coordinate (obtained directly from an instruction) in a gadget.
    pub fn from_inner(inner: EccChip::X) -> Self {
        X { inner }
    }
}

/// A constant elliptic curve point over the given curve, for which scalar multiplication
/// is more efficient.
#[derive(Clone, Debug)]
pub struct FixedPoint<C: CurveAffine, EccChip: EccInstructions<C>> {
    inner: EccChip::FixedPoint,
}

impl<C: CurveAffine, EccChip: EccInstructions<C>> FixedPoints<C> for FixedPoint<C, EccChip> {}

impl<C: CurveAffine, EccChip: EccInstructions<C>> FixedPoint<C, EccChip> {
    /// Gets a reference to the specified fixed point in the circuit.
    pub fn get(chip: EccChip, point: EccChip::FixedPoints) -> Result<Self, Error> {
        chip.get_fixed(point).map(|inner| FixedPoint { inner })
    }

    /// Returns `[by] self`.
    pub fn mul(
        &self,
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        by: &ScalarFixed<C, EccChip>,
    ) -> Result<Point<C, EccChip>, Error> {
        chip.mul_fixed(&mut layouter, &by.inner, &self.inner)
            .map(|inner| Point { inner })
    }
}
