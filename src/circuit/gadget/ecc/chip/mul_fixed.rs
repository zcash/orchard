use super::{
    add, add_incomplete, load::WindowUs, util, witness_point, CellValue, EccConfig, EccPoint,
    EccScalarFixed, EccScalarFixedShort, OrchardFixedBase, OrchardFixedBaseShort,
    OrchardFixedBases,
};
use crate::constants;

use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed, Permutation, Selector},
    poly::Rotation,
};

mod full_width;
mod short;

pub struct Config {
    q_mul_fixed: Selector,
    q_mul_fixed_short: Selector,
    // The fixed Lagrange interpolation coefficients for `x_p`.
    lagrange_coeffs: [Column<Fixed>; constants::H],
    // The fixed `z` for each window such that `y + z = u^2`.
    fixed_z: Column<Fixed>,
    // k-bit decomposition of an `n-1`-bit scalar:
    // a = a_0 + 2^k(a_1) + 2^{2k}(a_2) + ... + 2^{(n-1)k}(a_{n-1})
    k: Column<Advice>,
    // x-coordinate of the multiple of the fixed base at the current window.
    x_p: Column<Advice>,
    // y-coordinate of the multiple of the fixed base at the current window.
    y_p: Column<Advice>,
    // y-coordinate of accumulator (only used in the final row).
    u: Column<Advice>,
    // Permutation
    perm: Permutation,
    // Configuration for `add`
    add_config: add::Config,
    // Configuration for `add_incomplete`
    add_incomplete_config: add_incomplete::Config,
    // Configuration for `witness_point`
    witness_point_config: witness_point::Config,
}

impl From<&EccConfig> for Config {
    fn from(ecc_config: &EccConfig) -> Self {
        let config = Self {
            q_mul_fixed: ecc_config.q_mul_fixed,
            q_mul_fixed_short: ecc_config.q_mul_fixed_short,
            lagrange_coeffs: ecc_config.lagrange_coeffs,
            fixed_z: ecc_config.fixed_z,
            x_p: ecc_config.advices[0],
            y_p: ecc_config.advices[1],
            k: ecc_config.advices[4],
            u: ecc_config.advices[5],
            perm: ecc_config.perm.clone(),
            add_config: ecc_config.into(),
            add_incomplete_config: ecc_config.into(),
            witness_point_config: ecc_config.into(),
        };

        // Check relationships between this config and `add_config`.
        assert_eq!(
            config.x_p, config.add_config.x_p,
            "add is used internally in mul_fixed."
        );
        assert_eq!(
            config.y_p, config.add_config.y_p,
            "add is used internally in mul_fixed."
        );

        // Check relationships between this config and `add_incomplete_config`.
        assert_eq!(
            config.x_p, config.add_incomplete_config.x_p,
            "add_incomplete is used internally in mul_fixed."
        );
        assert_eq!(
            config.y_p, config.add_incomplete_config.y_p,
            "add_incomplete is used internally in mul_fixed."
        );
        for advice in [config.x_p, config.y_p, config.k, config.u].iter() {
            assert_ne!(
                *advice, config.add_config.x_qr,
                "Do not overlap with output columns of add."
            );
            assert_ne!(
                *advice, config.add_config.y_qr,
                "Do not overlap with output columns of add."
            );
        }

        // Check relationships between this config and `witness_point_config`.
        assert_eq!(
            config.x_p, config.witness_point_config.x,
            "witness_point is used internally in mul_fixed."
        );
        assert_eq!(
            config.y_p, config.witness_point_config.y,
            "witness_point is used internally in mul_fixed."
        );

        config
    }
}

impl Config {
    pub(super) fn create_gate<C: CurveAffine>(&self, meta: &mut ConstraintSystem<C::Base>) {
        self.create_gate_inner(meta);
        let short_config: short::Config<C> = self.into();
        short_config.create_gate(meta);
    }

    pub(super) fn assign_region_full<C: CurveAffine>(
        &self,
        scalar: &EccScalarFixed<C>,
        base: &OrchardFixedBase<C>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccPoint<C>, Error> {
        let full_width_config: full_width::Config<C> = self.into();
        full_width_config.assign_region(region, offset, scalar, base)
    }

    pub(super) fn assign_region_short<C: CurveAffine>(
        &self,
        scalar: &EccScalarFixedShort<C>,
        base: &OrchardFixedBaseShort<C>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccPoint<C>, Error> {
        let short_config: short::Config<C> = self.into();
        short_config.assign_region(region, offset, scalar, base)
    }

    fn create_gate_inner<F: FieldExt>(&self, meta: &mut ConstraintSystem<F>) {
        let q_mul_fixed = meta.query_selector(self.q_mul_fixed, Rotation::cur());
        let y_p = meta.query_advice(self.y_p, Rotation::cur());

        // Check interpolation of x-coordinate
        meta.create_gate("fixed-base scalar mul (x)", |meta| {
            let k = meta.query_advice(self.k, Rotation::cur());
            let x_p = meta.query_advice(self.x_p, Rotation::cur());

            let k_pow: Vec<Expression<F>> = (0..constants::H)
                .map(|pow| (0..pow).fold(Expression::Constant(F::one()), |acc, _| acc * k.clone()))
                .collect();

            let interpolated_x = k_pow.iter().zip(self.lagrange_coeffs.iter()).fold(
                Expression::Constant(F::zero()),
                |acc, (k_pow, coeff)| {
                    acc + (k_pow.clone() * meta.query_fixed(*coeff, Rotation::cur()))
                },
            );

            q_mul_fixed.clone() * (interpolated_x - x_p)
        });

        // Check that `y + z = u^2`, where `z` is fixed and `u`, `y` are witnessed
        meta.create_gate("fixed-base scalar mul (y)", |meta| {
            let z = meta.query_fixed(self.fixed_z, Rotation::cur());
            let u = meta.query_advice(self.u, Rotation::cur());

            q_mul_fixed * (u.clone() * u - y_p - z)
        });
    }
}

enum FixedBase<C: CurveAffine> {
    FullWidth(OrchardFixedBase<C>),
    Short(OrchardFixedBaseShort<C>),
}

impl<C: CurveAffine> From<&OrchardFixedBase<C>> for FixedBase<C> {
    fn from(fixed_base: &OrchardFixedBase<C>) -> Self {
        Self::FullWidth(fixed_base.clone())
    }
}

impl<C: CurveAffine> From<&OrchardFixedBaseShort<C>> for FixedBase<C> {
    fn from(fixed_base: &OrchardFixedBaseShort<C>) -> Self {
        Self::Short(fixed_base.clone())
    }
}

impl<C: CurveAffine> FixedBase<C> {
    fn value(&self) -> C {
        match self {
            FixedBase::FullWidth(base) => match base.base {
                OrchardFixedBases::CommitIvkR(inner) => inner.0.value(),
                OrchardFixedBases::NoteCommitR(inner) => inner.0.value(),
                OrchardFixedBases::NullifierK(inner) => inner.0.value(),
                OrchardFixedBases::ValueCommitR(inner) => inner.0.value(),
            },
            FixedBase::Short(base) => base.base.0 .0.value(),
        }
    }

    fn u(&self) -> Vec<WindowUs<C::Base>> {
        match self {
            FixedBase::FullWidth(base) => base.u.0.as_ref().to_vec(),
            FixedBase::Short(base) => base.u_short.0.as_ref().to_vec(),
        }
    }
}

enum ScalarFixed<C: CurveAffine> {
    FullWidth(EccScalarFixed<C>),
    Short(EccScalarFixedShort<C>),
}

impl<C: CurveAffine> From<&EccScalarFixed<C>> for ScalarFixed<C> {
    fn from(scalar_fixed: &EccScalarFixed<C>) -> Self {
        Self::FullWidth(scalar_fixed.clone())
    }
}

impl<C: CurveAffine> From<&EccScalarFixedShort<C>> for ScalarFixed<C> {
    fn from(scalar_fixed: &EccScalarFixedShort<C>) -> Self {
        Self::Short(scalar_fixed.clone())
    }
}

impl<C: CurveAffine> ScalarFixed<C> {
    fn k_bits(&self) -> &[CellValue<C::Base>] {
        match self {
            ScalarFixed::FullWidth(scalar) => &scalar.k_bits,
            ScalarFixed::Short(scalar) => &scalar.k_bits,
        }
    }

    // The scalar decomposition was done in the base field. For computation
    // outside the circuit, we now convert them back into the scalar field.
    fn k_field(&self) -> Vec<Option<C::Scalar>> {
        self.k_bits()
            .iter()
            .map(|bits| {
                bits.value
                    .map(|value| C::Scalar::from_bytes(&value.to_bytes()).unwrap())
            })
            .collect::<Vec<_>>()
    }

    // The scalar decomposition is guaranteed to be in three-bit windows,
    // so we also cast the least significant byte in their serialisation
    // into usize for convenient indexing into `u`-values
    fn k_usize(&self) -> Vec<Option<usize>> {
        self.k_bits()
            .iter()
            .map(|bits| bits.value.map(|value| value.to_bytes()[0] as usize))
            .collect::<Vec<_>>()
    }
}

trait MulFixed<C: CurveAffine> {
    const NUM_WINDOWS: usize;

    fn q_mul_fixed(&self) -> Selector;
    fn lagrange_coeffs(&self) -> [Column<Fixed>; constants::H];
    fn fixed_z(&self) -> Column<Fixed>;
    fn k(&self) -> Column<Advice>;
    fn u(&self) -> Column<Advice>;
    fn x_p(&self) -> Column<Advice>;
    fn y_p(&self) -> Column<Advice>;
    fn perm(&self) -> &Permutation;
    fn witness_point_config(&self) -> &witness_point::Config;
    fn add_config(&self) -> &add::Config;
    fn add_incomplete_config(&self) -> &add_incomplete::Config;

    #[allow(clippy::type_complexity)]
    fn assign_region_inner(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        scalar: &ScalarFixed<C>,
        base: &FixedBase<C>,
    ) -> Result<(EccPoint<C>, EccPoint<C>), Error> {
        // Assign fixed columns for given fixed base
        self.assign_fixed_constants(region, offset, base)?;

        // Copy the scalar decomposition
        self.copy_scalar(region, offset, scalar)?;

        // Initialize accumulator
        let acc = self.initialize_accumulator(region, offset, base, scalar)?;

        // Process all windows excluding least and most significant windows
        let acc = self.add_incomplete(region, offset, acc, base, scalar)?;

        // Process most significant window using complete addition
        let mul_b = self.process_msb(region, offset, base, scalar)?;

        Ok((acc, mul_b))
    }

    fn assign_fixed_constants(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        base: &FixedBase<C>,
    ) -> Result<(), Error> {
        let (base_lagrange_coeffs, base_z) = match base {
            FixedBase::FullWidth(base) => (
                base.lagrange_coeffs.0.as_ref().to_vec(),
                base.z.0.as_ref().to_vec(),
            ),
            FixedBase::Short(base) => (
                base.lagrange_coeffs_short.0.as_ref().to_vec(),
                base.z_short.0.as_ref().to_vec(),
            ),
        };

        // Assign fixed columns for given fixed base
        for window in 0..Self::NUM_WINDOWS {
            // Enable `q_mul_fixed` selector
            self.q_mul_fixed().enable(region, window + offset)?;

            // Assign x-coordinate Lagrange interpolation coefficients
            for k in 0..(constants::H) {
                region.assign_fixed(
                    || {
                        format!(
                            "Lagrange interpolation coeff for window: {:?}, k: {:?}",
                            window, k
                        )
                    },
                    self.lagrange_coeffs()[k],
                    window + offset,
                    || Ok(base_lagrange_coeffs[window].0[k]),
                )?;
            }

            // Assign z-values for each window
            region.assign_fixed(
                || format!("z-value for window: {:?}", window),
                self.fixed_z(),
                window + offset,
                || Ok(base_z[window]),
            )?;
        }

        Ok(())
    }

    fn copy_scalar(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        scalar: &ScalarFixed<C>,
    ) -> Result<(), Error> {
        // Copy the scalar decomposition
        for (window, k) in scalar.k_bits().iter().enumerate() {
            util::assign_and_constrain(
                region,
                || format!("k[{:?}]", window),
                self.k(),
                window + offset,
                k,
                self.perm(),
            )?;
        }

        Ok(())
    }

    fn initialize_accumulator(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        base: &FixedBase<C>,
        scalar: &ScalarFixed<C>,
    ) -> Result<EccPoint<C>, Error> {
        // Witness `m0` in `x_p`, `y_p` cells on row 0
        let k0 = scalar.k_field()[0];
        let m0 = k0.map(|k0| base.value() * (k0 + C::Scalar::from_u64(2)));
        let m0 = self.witness_point_config().assign_region(
            m0.map(|point| point.to_affine()),
            offset,
            region,
        )?;

        // Assign u = (y_p + z_w).sqrt() for `m0`
        {
            let k0 = scalar.k_usize()[0];
            let u0 = &base.u()[0];
            let u0 = k0.map(|k0| u0.0[k0]);

            region.assign_advice(|| "u", self.u(), offset, || u0.ok_or(Error::SynthesisError))?;
        }

        // Copy `m0` into `x_qr`, `y_qr` cells on row 1
        let x = util::assign_and_constrain(
            region,
            || "initialize acc x",
            self.add_incomplete_config().x_qr,
            offset + 1,
            &m0.x,
            self.perm(),
        )?;
        let y = util::assign_and_constrain(
            region,
            || "initialize acc y",
            self.add_incomplete_config().y_qr,
            offset + 1,
            &m0.y,
            self.perm(),
        )?;

        Ok(EccPoint { x, y })
    }

    fn add_incomplete(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        mut acc: EccPoint<C>,
        base: &FixedBase<C>,
        scalar: &ScalarFixed<C>,
    ) -> Result<EccPoint<C>, Error> {
        // This is 2^w, where w is the window width
        let h = C::Scalar::from_u64(constants::H as u64);

        let base_value = base.value();
        let base_u = base.u();
        let scalar_k_field = scalar.k_field();
        let scalar_k_usize = scalar.k_usize();

        for (w, k) in scalar_k_field[1..(scalar_k_field.len() - 1)]
            .iter()
            .enumerate()
        {
            // Offset window index by 1 since we are starting on k_1
            let w = w + 1;

            // Compute [(k_w + 2) ⋅ 8^w]B
            let mul_b =
                k.map(|k| base_value * (k + C::Scalar::from_u64(2)) * h.pow(&[w as u64, 0, 0, 0]));
            let mul_b = self.witness_point_config().assign_region(
                mul_b.map(|point| point.to_affine()),
                offset + w,
                region,
            )?;

            // Assign u = (y_p + z_w).sqrt()
            let u_val = scalar_k_usize[w].map(|k| base_u[w].0[k]);
            region.assign_advice(
                || "u",
                self.u(),
                offset + w,
                || u_val.ok_or(Error::SynthesisError),
            )?;

            // Add to the accumulator
            acc = self
                .add_incomplete_config()
                .assign_region(&mul_b, &acc, offset + w, region)?;
        }
        Ok(acc)
    }

    fn process_msb(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        base: &FixedBase<C>,
        scalar: &ScalarFixed<C>,
    ) -> Result<EccPoint<C>, Error> {
        // This is 2^w, where w is the window width
        let h = C::Scalar::from_u64(constants::H as u64);

        // Assign u = (y_p + z_w).sqrt() for the most significant window
        {
            let u_val = scalar.k_usize()[Self::NUM_WINDOWS - 1]
                .map(|k| base.u()[Self::NUM_WINDOWS - 1].0[k]);
            region.assign_advice(
                || "u",
                self.u(),
                offset + Self::NUM_WINDOWS - 1,
                || u_val.ok_or(Error::SynthesisError),
            )?;
        }

        // offset_acc = \sum_{j = 0}^{NUM_WINDOWS - 2} 2^{FIXED_BASE_WINDOW_SIZE * j+1}
        let offset_acc = (0..(Self::NUM_WINDOWS - 1)).fold(C::ScalarExt::zero(), |acc, w| {
            acc + C::Scalar::from_u64(2).pow(&[
                constants::FIXED_BASE_WINDOW_SIZE as u64 * w as u64 + 1,
                0,
                0,
                0,
            ])
        });

        // `scalar = [k * 8^84 - offset_acc]`, where `offset_acc = \sum_{j = 0}^{83} 2^{FIXED_BASE_WINDOW_SIZE * j + 1}`.
        let scalar = scalar.k_field()[scalar.k_field().len() - 1]
            .map(|k| k * h.pow(&[(Self::NUM_WINDOWS - 1) as u64, 0, 0, 0]) - offset_acc);

        let mul_b = scalar.map(|scalar| base.value() * scalar);
        self.witness_point_config().assign_region(
            mul_b.map(|point| point.to_affine()),
            offset + Self::NUM_WINDOWS - 1,
            region,
        )
    }
}
