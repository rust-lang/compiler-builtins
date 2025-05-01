/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use super::super::{CastFrom, Float, Int, MinInt};
use crate::support::{DInt, HInt, Reducer};

#[inline]
pub fn fmod<F: Float>(x: F, y: F) -> F {
    let _1 = F::Int::ONE;
    let sx = x.to_bits() & F::SIGN_MASK;
    let ux = x.to_bits() & !F::SIGN_MASK;
    let uy = y.to_bits() & !F::SIGN_MASK;

    // Cases that return NaN:
    //   NaN % _
    //   Inf % _
    //     _ % NaN
    //     _ % 0
    let x_nan_or_inf = ux & F::EXP_MASK == F::EXP_MASK;
    let y_nan_or_zero = uy.wrapping_sub(_1) & F::EXP_MASK == F::EXP_MASK;
    if x_nan_or_inf | y_nan_or_zero {
        return (x * y) / (x * y);
    }

    if ux < uy {
        // |x| < |y|
        return x;
    }

    let (num, ex) = into_sig_exp::<F>(ux);
    let (div, ey) = into_sig_exp::<F>(uy);

    // To compute `(num << ex) % (div << ey)`, first
    // evaluate `rem = (num << (ex - ey)) % div` ...
    let rem = reduction(num, ex - ey, div);
    // ... so the result will be `rem << ey`

    if rem.is_zero() {
        // Return zero with the sign of `x`
        return F::from_bits(sx);
    };

    // We would shift `rem` up by `ey`, but have to stop at `F::SIG_BITS`
    let shift = ey.min(F::SIG_BITS - rem.ilog2());
    // Anything past that is added to the exponent field
    let bits = (rem << shift) + (F::Int::cast_from(ey - shift) << F::SIG_BITS);
    F::from_bits(sx + bits)
}

/// Given the bits of a finite float, return a tuple of
///  - the mantissa with the implicit bit (0 if subnormal, 1 otherwise)
///  - the additional exponent past 1, (0 for subnormal, 0 or more otherwise)
fn into_sig_exp<F: Float>(mut bits: F::Int) -> (F::Int, u32) {
    bits &= !F::SIGN_MASK;
    // Subtract 1 from the exponent, clamping at 0
    let sat = bits.checked_sub(F::IMPLICIT_BIT).unwrap_or(F::Int::ZERO);
    (
        bits - (sat & F::EXP_MASK),
        u32::cast_from(sat >> F::SIG_BITS),
    )
}

/// Compute the remainder `(x * 2.pow(e)) % y` without overflow.
fn reduction<I: Int>(mut x: I, e: u32, y: I) -> I {
    // FIXME: This is a temporary hack to get around the lack of `u256 / u256`.
    // Actually, the algorithm only needs the operation `(x << I::BITS) / y`
    // where `x < y`. That is, a division `u256 / u128` where the quotient must
    // not overflow `u128` would be sufficient for `f128`.
    unsafe {
        use core::mem::transmute_copy;
        if I::BITS == 64 {
            let x = transmute_copy::<I, u64>(&x);
            let y = transmute_copy::<I, u64>(&y);
            let r = fast_reduction::<f64, u64>(x, e, y);
            return transmute_copy::<u64, I>(&r);
        }
        if I::BITS == 32 {
            let x = transmute_copy::<I, u32>(&x);
            let y = transmute_copy::<I, u32>(&y);
            let r = fast_reduction::<f32, u32>(x, e, y);
            return transmute_copy::<u32, I>(&r);
        }
        #[cfg(f16_enabled)]
        if I::BITS == 16 {
            let x = transmute_copy::<I, u16>(&x);
            let y = transmute_copy::<I, u16>(&y);
            let r = fast_reduction::<f16, u16>(x, e, y);
            return transmute_copy::<u16, I>(&r);
        }
    }

    x %= y;
    for _ in 0..e {
        x <<= 1;
        x = x.checked_sub(y).unwrap_or(x);
    }
    x
}

trait SafeShift: Float {
    // How many guaranteed leading zeros do the values have?
    // A normalized floating point mantissa has `EXP_BITS` guaranteed leading
    // zeros (exludes the implicit bit, but includes the now-zeroed sign bit)
    // `-1` because we want to shift by either `BASE_SHIFT` or `BASE_SHIFT + 1`
    const BASE_SHIFT: u32 = Self::EXP_BITS - 1;
}
impl<F: Float> SafeShift for F {}

fn fast_reduction<F, I>(x: I, e: u32, y: I) -> I
where
    F: Float<Int = I>,
    I: Int + HInt,
    I::D: Int + DInt<H = I>,
{
    let _0 = I::ZERO;
    let _1 = I::ONE;

    if y == _1 {
        return _0;
    }

    if e <= F::BASE_SHIFT {
        return (x << e) % y;
    }

    // Find least depth s.t. `(e >> depth) < I::BITS`
    let depth = (I::BITS - 1)
        .leading_zeros()
        .saturating_sub(e.leading_zeros());

    let initial = (e >> depth) - F::BASE_SHIFT;

    let max_rem = y.wrapping_sub(_1);
    let max_ilog2 = max_rem.ilog2();
    let mut pow2 = _1 << max_ilog2.min(initial);
    for _ in max_ilog2..initial {
        pow2 <<= 1;
        pow2 = pow2.checked_sub(y).unwrap_or(pow2);
    }

    // At each step `k in [depth, ..., 0]`,
    // `p` is `(e >> k) - BASE_SHIFT`
    // `m` is `(1 << p) % y`
    let mut k = depth;
    let mut p = initial;
    let mut m = Reducer::new(pow2, y);

    while k > 0 {
        k -= 1;
        p = p + p + F::BASE_SHIFT;
        if e & (1 << k) != 0 {
            m = m.squared_with_shift(F::BASE_SHIFT + 1);
            p += 1;
        } else {
            m = m.squared_with_shift(F::BASE_SHIFT);
        };

        debug_assert!(p == (e >> k) - F::BASE_SHIFT);
    }

    // (x << BASE_SHIFT) * (1 << p) == x << e
    m.mul_into_div_rem(x << F::BASE_SHIFT).1
}
