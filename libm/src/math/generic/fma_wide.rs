/* SPDX-License-Identifier: MIT */
/* origin: musl src/math/fmaf.c Ported to generic Rust algorithm in 2025, TG. */
/* The musl subnormal rounding bug is fixed using the formally proven algorithm from */
/* "Emulation of FMA and correctly-rounded sums: proved algorithms using rounding to odd" */
/* by Sylvie Boldo and Guillaume Melquiond, https://guillaume.melquiond.fr/doc/08-tc.pdf */

use crate::support::{
    CastFrom, CastInto, Float, FpResult, IntTy, MinInt, NarrowFloat, Round, WideFloat,
};

/// Fma implementation when a hardware-backed larger float type is available.
/// The larger type has enough precision and exponent range to represent the exact product,
/// leaving only the addition and the final narrowing susceptible to double rounding.
#[inline]
pub fn fma_wide_round<F, B>(x: F, y: F, z: F, round: Round) -> FpResult<F>
where
    F: Float + NarrowFloat<D = B>,
    B: Float + WideFloat<H = F>,
    B::Int: CastInto<i32>,
    i32: CastFrom<i32>,
{
    let one = IntTy::<B>::ONE;

    let xy: B = x.widen() * y.widen();
    let mut result: B = xy + z.widen();
    let mut ui: B::Int = result.to_bits();
    let re = result.ex();
    let zb: B = z.widen();

    // The round-to-odd method from the paper is only needed for non-exceptional values
    // and only when rounding to nearest
    if re == B::EXP_SAT || round != Round::Nearest {
        return FpResult::ok(result.narrow());
    }

    // TwoSum recovers the exact residual of the widened addition. If the addition was inexact and
    // its rounded significand is even, move it one ULP toward the residual to produce a round-to-odd
    // intermediate. Theorem 3 proves that rounding this intermediate to nearest in `F` gives the
    // correctly rounded result, including for subnormals and underflow.
    let virtual_z = result - xy;
    let residual = (xy - (result - virtual_z)) + (zb - virtual_z);
    let neg = ui >> (B::BITS - 1) != IntTy::<B>::ZERO;
    if residual != B::ZERO && (ui & one) == IntTy::<B>::ZERO {
        if neg == (residual < B::ZERO) {
            ui += one;
        } else {
            ui -= one;
        }

        result = B::from_bits(ui);
    }

    FpResult::ok(result.narrow())
}
