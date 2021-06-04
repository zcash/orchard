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
    q_add_incomplete: Selector,
    // x-coordinate of P in P + Q = R
    pub x_p: Column<Advice>,
    // y-coordinate of P in P + Q = R
    pub y_p: Column<Advice>,
    // x-coordinate of Q or R in P + Q = R
    pub x_qr: Column<Advice>,
    // y-coordinate of Q or R in P + Q = R
    pub y_qr: Column<Advice>,
    // Permutation
    perm: Permutation,
}

impl From<&EccConfig> for Config {
    fn from(ecc_config: &EccConfig) -> Self {
        Self {
            q_add_incomplete: ecc_config.q_add_incomplete,
            x_p: ecc_config.advices[0],
            y_p: ecc_config.advices[1],
            x_qr: ecc_config.advices[2],
            y_qr: ecc_config.advices[3],
            perm: ecc_config.perm.clone(),
        }
    }
}

impl Config {
    pub(super) fn create_gate<F: FieldExt>(&self, meta: &mut ConstraintSystem<F>) {
        let q_add_incomplete = meta.query_selector(self.q_add_incomplete, Rotation::cur());
        let x_p = meta.query_advice(self.x_p, Rotation::cur());
        let y_p = meta.query_advice(self.y_p, Rotation::cur());
        let x_q = meta.query_advice(self.x_qr, Rotation::cur());
        let y_q = meta.query_advice(self.y_qr, Rotation::cur());
        let x_r = meta.query_advice(self.x_qr, Rotation::next());
        let y_r = meta.query_advice(self.y_qr, Rotation::next());

        // (x_r + x_q + x_p)⋅(x_p − x_q)^2 − (y_p − y_q)^2 = 0
        meta.create_gate("point addition expr1", |_| {
            let expr1 = (x_r.clone() + x_q.clone() + x_p.clone())
                * (x_p.clone() - x_q.clone())
                * (x_p.clone() - x_q.clone())
                - (y_p.clone() - y_q.clone()) * (y_p.clone() - y_q.clone());

            q_add_incomplete.clone() * expr1
        });

        // (y_r + y_q)(x_p − x_q) − (y_p − y_q)(x_q − x_r) = 0
        meta.create_gate("point addition expr2", |_| {
            let expr2 = (y_r + y_q.clone()) * (x_p - x_q.clone()) - (y_p - y_q) * (x_q - x_r);

            q_add_incomplete * expr2
        });
    }

    pub(super) fn assign_region<C: CurveAffine>(
        &self,
        p: &EccPoint<C>,
        q: &EccPoint<C>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccPoint<C>, Error> {
        // Enable `q_add_incomplete` selector
        self.q_add_incomplete.enable(region, offset)?;

        // Handle exceptional cases
        let (x_p, y_p) = (p.x.value, p.y.value);
        let (x_q, y_q) = (q.x.value, q.y.value);
        x_p.zip(y_p)
            .zip(x_q)
            .zip(y_q)
            .map(|(((x_p, y_p), x_q), y_q)| {
                // P is point at infinity
                if (x_p == C::Base::zero() && y_p == C::Base::zero())
                // Q is point at infinity
                || (x_q == C::Base::zero() && y_q == C::Base::zero())
                // x_p = x_q
                || (x_p == x_q)
                {
                    Err(Error::SynthesisError)
                } else {
                    Ok(())
                }
            })
            .transpose()?;

        // Copy point `p` into `x_p`, `y_p` columns
        util::assign_and_constrain(region, || "x_p", self.x_p, offset, &p.x, &self.perm)?;
        util::assign_and_constrain(region, || "y_p", self.y_p, offset, &p.y, &self.perm)?;

        // Copy point `q` into `x_qr`, `y_qr` columns
        util::assign_and_constrain(region, || "x_q", self.x_qr, offset, &q.x, &self.perm)?;
        util::assign_and_constrain(region, || "y_q", self.y_qr, offset, &q.y, &self.perm)?;

        // Compute the sum `P + Q = R`
        let r = {
            let p = p.point();
            let q = q.point();
            let r = p
                .zip(q)
                .map(|(p, q)| (p + q).to_affine().coordinates().unwrap());
            let r_x = r.map(|r| *r.x());
            let r_y = r.map(|r| *r.y());

            (r_x, r_y)
        };

        // Assign the sum to `x_qr`, `y_qr` columns in the next row
        let x_r = r.0;
        let x_r_var = region.assign_advice(
            || "x_r",
            self.x_qr,
            offset + 1,
            || x_r.ok_or(Error::SynthesisError),
        )?;

        let y_r = r.1;
        let y_r_var = region.assign_advice(
            || "y_r",
            self.y_qr,
            offset + 1,
            || y_r.ok_or(Error::SynthesisError),
        )?;

        let result = EccPoint::<C> {
            x: CellValue::<C::Base>::new(x_r_var, x_r),
            y: CellValue::<C::Base>::new(y_r_var, y_r),
        };

        #[cfg(test)]
        // Check that the correct sum is obtained.
        {
            let p = p.point();
            let q = q.point();
            let real_sum = p.zip(q).map(|(p, q)| p + q);
            let result = result.point();

            assert_eq!(real_sum.unwrap().to_affine(), result.unwrap());
        }

        Ok(result)
    }
}

#[cfg(test)]
pub mod tests {
    use halo2::{arithmetic::CurveAffine, circuit::Layouter, plonk::Error};

    use crate::circuit::gadget::ecc::{EccInstructions, Point};

    pub fn test_add_incomplete<
        C: CurveAffine,
        EccChip: EccInstructions<C> + Clone + Eq + std::fmt::Debug,
    >(
        mut layouter: impl Layouter<C::Base>,
        zero: &Point<C, EccChip>,
        p: &Point<C, EccChip>,
        q: &Point<C, EccChip>,
        p_neg: &Point<C, EccChip>,
    ) -> Result<(), Error> {
        // P + Q
        p.add_incomplete(layouter.namespace(|| "P + Q"), &q)?;

        // P + P should return an error
        p.add_incomplete(layouter.namespace(|| "P + P"), &p)
            .expect_err("P + P should return an error");

        // P + (-P) should return an error
        p.add_incomplete(layouter.namespace(|| "P + (-P)"), &p_neg)
            .expect_err("P + (-P) should return an error");

        // P + 𝒪 should return an error
        p.add_incomplete(layouter.namespace(|| "P + 𝒪"), &zero)
            .expect_err("P + 0 should return an error");

        // 𝒪 + P should return an error
        zero.add_incomplete(layouter.namespace(|| "𝒪 + P"), &p)
            .expect_err("0 + P should return an error");

        // 𝒪 + 𝒪 should return an error
        zero.add_incomplete(layouter.namespace(|| "𝒪 + 𝒪"), &zero)
            .expect_err("𝒪 + 𝒪 should return an error");

        Ok(())
    }
}
