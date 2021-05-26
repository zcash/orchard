use super::{util, CellValue, EccConfig, EccPoint};

use ff::Field;
use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Permutation, Selector},
    poly::Rotation,
};

#[derive(Clone, Debug)]
pub struct Config {
    q_double: Selector,
    // x-coordinate of P in [2]P = R
    x_p: Column<Advice>,
    // y-coordinate of P in [2]P = R
    y_p: Column<Advice>,
    // x-coordinate of R in [2]P = R
    x_r: Column<Advice>,
    // y-coordinate of R in [2]P = R
    y_r: Column<Advice>,
    // Permutation
    perm: Permutation,
}

impl From<&EccConfig> for Config {
    fn from(ecc_config: &EccConfig) -> Self {
        Self {
            q_double: ecc_config.q_double,
            x_p: ecc_config.P.0,
            y_p: ecc_config.P.1,
            x_r: ecc_config.extras[0],
            y_r: ecc_config.extras[1],
            perm: ecc_config.perm.clone(),
        }
    }
}

impl Config {
    pub(super) fn create_gate<F: FieldExt>(&self, meta: &mut ConstraintSystem<F>) {
        let q_double = meta.query_selector(self.q_double, Rotation::cur());
        let x_p = meta.query_advice(self.x_p, Rotation::cur());
        let y_p = meta.query_advice(self.y_p, Rotation::cur());
        let x_r = meta.query_advice(self.x_r, Rotation::cur());
        let y_r = meta.query_advice(self.y_r, Rotation::cur());

        let x_p_2 = x_p.clone() * x_p.clone();
        let x_p_4 = x_p_2.clone() * x_p_2.clone();

        // 4⋅(y_p)^2⋅(x_r + 2⋅x_p) − 9⋅(x_p)^4 = 0
        meta.create_gate("point doubling expr1", |_| {
            let expr1 = y_p.clone()
                * y_p.clone()
                * (x_r.clone() + x_p.clone() * F::from_u64(2))
                * F::from_u64(4)
                - x_p_4 * F::from_u64(9);
            q_double.clone() * expr1
        });

        // 2⋅y_p⋅(y_r + y_p) − 3⋅(x_p)^2⋅(x_p − x_r) = 0
        meta.create_gate("point doubling expr2", |_| {
            let expr2 =
                y_p.clone() * (y_r + y_p) * F::from_u64(2) - x_p_2 * (x_p - x_r) * F::from_u64(3);

            q_double * expr2
        });
    }

    pub(super) fn assign_region<C: CurveAffine>(
        &self,
        p: &EccPoint<C>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccPoint<C>, Error> {
        // Enable `q_double` selector
        self.q_double.enable(region, offset)?;

        // Return error if `P` is point at infinity
        let (x_p, y_p) = (p.x.value, p.y.value);
        x_p.zip(y_p)
            .map(|(x, y)| {
                if x == C::Base::zero() && y == C::Base::zero() {
                    Err(Error::SynthesisError)
                } else {
                    Ok(())
                }
            })
            .transpose()?;

        // Copy the point `P` into `x_p`, `y_p` columns
        util::assign_and_constrain(region, || "x_p", self.x_p, offset, &p.x, &self.perm)?;
        util::assign_and_constrain(region, || "y_p", self.y_p, offset, &p.y, &self.perm)?;

        // Compute the doubled point
        let r = p.point().map(|p| {
            let r = p * C::Scalar::from_u64(2);
            let r = r.to_affine().coordinates().unwrap();

            (*r.x(), *r.y())
        });
        let x_r = r.map(|r| r.0);
        let y_r = r.map(|r| r.1);

        // Assign the doubled point to `x_r`, `y_r` columns
        let x_r_var = region.assign_advice(
            || "x_r",
            self.x_r,
            offset,
            || x_r.ok_or(Error::SynthesisError),
        )?;
        let y_r_var = region.assign_advice(
            || "y_r",
            self.y_r,
            offset,
            || y_r.ok_or(Error::SynthesisError),
        )?;

        Ok(EccPoint {
            x: CellValue::<C::Base>::new(x_r_var, x_r),
            y: CellValue::<C::Base>::new(y_r_var, y_r),
        })
    }
}
