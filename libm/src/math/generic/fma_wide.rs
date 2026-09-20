/* SPDX-License-Identifier: MIT */
/* origin: musl src/math/fmaf.c Ported to generic Rust algorithm in 2025, TG. */
/* The musl subnormal rounding bug is fixed using the formally proven algorithm from */
/* "Emulation of FMA and correctly-rounded sums: proved algorithms using rounding to odd" */
/* by Sylvie Boldo and Guillaume Melquiond, https://guillaume.melquiond.fr/doc/08-tc.pdf */

use crate::support::{
    CastFrom, CastInto, Float, FpResult, IntTy, MinInt, NarrowFloat, Round, Status, WideFloat,
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

    let prec_diff = B::SIG_BITS - F::SIG_BITS;
    let excess_prec = ui & ((one << prec_diff) - one);
    let halfway = one << (prec_diff - 1);
    let min_normal_exp = (B::EXP_BIAS as i32 + F::EXP_MIN) as u32;

    // Common case: the larger precision is fine if...
    // This is a normal result and not a halfway case
    if (re >= min_normal_exp && excess_prec != halfway)
        // Or the result is NaN
        || re == B::EXP_SAT
        // Or the result is exact
        || (result - xy == zb && result - zb == xy)
        // Or the mode is something other than round to nearest
        || round != Round::Nearest
    {
        let min_inexact_exp = (B::EXP_BIAS as i32 + F::EXP_MIN_SUBNORM) as u32;

        let mut status = Status::OK;

        if (min_inexact_exp..min_normal_exp).contains(&re) && status.inexact() {
            // This branch is never hit; requires previous operations to set a status
            status.set_inexact(false);

            result = xy + z.widen();
            if status.inexact() {
                status.set_underflow(true);
            } else {
                status.set_inexact(true);
            }
        }

        return FpResult {
            val: result.narrow(),
            status,
        };
    }

    // FastTwoSum recovers the exact residual of the widened addition. If the addition was inexact
    // and its rounded significand is even, move it one ULP toward the residual to produce a
    // round-to-odd intermediate. Theorem 3 proves that rounding this intermediate to nearest in `F`
    // gives the correctly rounded result, including for subnormals and underflow.
    let neg = ui >> (B::BITS - 1) != IntTy::<B>::ZERO;
    let err = if neg == (zb > xy) {
        xy - result + zb
    } else {
        zb - result + xy
    };
    // Exact sums need no correction, and odd inexact sums are already round-to-odd.
    if err == B::ZERO || (ui & one) != IntTy::<B>::ZERO {
        return FpResult::ok(result.narrow());
    }
    if neg == (err < B::ZERO) {
        ui += one;
    } else {
        ui -= one;
    }

    FpResult::ok(B::from_bits(ui).narrow())
}
