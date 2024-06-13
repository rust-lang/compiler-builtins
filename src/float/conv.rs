use core::ops::Neg;

use crate::int::{CastFrom, CastInto, Int, MinInt};

use super::Float;

/// Conversions from integers to floats.
///
/// These are hand-optimized bit twiddling code,
/// which unfortunately isn't the easiest kind of code to read.
///
/// The algorithm is explained here: <https://blog.m-ou.se/floats/>. It roughly does the following:
/// - Calculate the exponent based on the base-2 logarithm of `i` (leading zeros)
/// - Calculate a base mantissa by shifting the integer into mantissa position
/// - Figure out if rounding needs to occour by classifying truncated bits. Some patterns apply
///   here, so they may be "squashed" into smaller numbers to spmplifiy the classification.
mod int_to_float {
    use super::*;

    /// Calculate the exponent from the number of leading zeros.
    fn exp<I: Int, F: Float<Int: CastFrom<u32>>>(n: u32) -> F::Int {
        F::Int::cast_from(I::BITS + F::EXPONENT_BIAS - 2 - n)
    }

    /// Shift the integer into the float's mantissa bits. Keep the lowest exponent bit intact.
    fn m_base<I: Int, F: Float<Int: CastFrom<I>>>(i_m: I) -> F::Int {
        F::Int::cast_from(i_m >> ((I::BITS - F::BITS) + F::EXPONENT_BITS))
    }

    /// Calculate the mantissa in cases where the float size is greater than integer size
    fn m_f_gt_i<I: Int, F: Float<Int: CastFrom<I>>>(i: I, n: u32) -> F::Int {
        F::Int::cast_from(i) << (F::SIGNIFICAND_BITS - I::BITS + 1 + n)
    }

    /// Calculate the mantissa and a dropped bit adjustment  when `f` and `i` are equal sizes
    fn m_f_eq_i<I: Int + CastInto<F::Int>, F: Float<Int = I>>(i: I, n: u32) -> (F::Int, F::Int) {
        let base = (i << n) >> F::EXPONENT_BITS;

        // Only the lowest `F::EXPONENT_BITS` bits will be truncated. Shift them
        // to the highest position
        let adj = (i << n) << (F::SIGNIFICAND_BITS + 1);

        (base, adj)
    }

    /// Adjust a mantissa with dropped bits
    fn m_adj<F: Float>(m_base: F::Int, dropped_bits: F::Int) -> F::Int {
        // Branchlessly extract a `1` if rounding up should happen
        let adj = (dropped_bits - (dropped_bits >> (F::BITS - 1) & !m_base)) >> (F::BITS - 1);

        // Add one when we need to round up. Break ties to even.
        m_base + adj
    }

    /// Combine a final float repr from an exponent and mantissa.
    fn repr<F: Float>(e: F::Int, m: F::Int) -> F::Int {
        // + rather than | so the mantissa can overflow into the exponent
        (e << F::SIGNIFICAND_BITS) + m
    }

    /// Perform a signed operation as unsigned, then add the sign back
    pub fn signed<I, F, Conv>(i: I, conv: Conv) -> F
    where
        F: Float,
        I: Int,
        F::Int: CastFrom<I>,
        Conv: Fn(I::UnsignedInt) -> F::Int,
    {
        let sign_bit = F::Int::cast_from(i >> (I::BITS - 1)) << (F::BITS - 1);
        F::from_repr(conv(i.unsigned_abs()) | sign_bit)
    }

    pub fn u32_to_f32_bits(i: u32) -> u32 {
        if i == 0 {
            return 0;
        }
        let n = i.leading_zeros();
        let (m_base, adj) = m_f_eq_i::<u32, f32>(i, n);
        let m = m_adj::<f32>(m_base, adj);
        let e = exp::<u32, f32>(n);
        repr::<f32>(e, m)
    }

    pub fn u32_to_f64_bits(i: u32) -> u64 {
        if i == 0 {
            return 0;
        }
        let n = i.leading_zeros();
        let m = m_f_gt_i::<_, f64>(i, n);
        let e = exp::<u32, f64>(n);
        repr::<f64>(e, m)
    }

    pub fn u64_to_f32_bits(i: u64) -> u32 {
        let n = i.leading_zeros();
        let i_m = i.wrapping_shl(n); // Mantissa, shifted so the first bit is nonzero
        let m_base = m_base::<_, f32>(i_m);
        // The entire lower half of `i` will be truncated (masked portion), plus the
        // next `EXPONENT_BITS` bits.
        let adj = (i_m >> f32::EXPONENT_BITS | i_m & 0xFFFF) as u32;
        let m = m_adj::<f32>(m_base, adj);
        let e = if i == 0 { 0 } else { exp::<u64, f32>(n) };
        repr::<f32>(e, m)
    }

    pub fn u64_to_f64_bits(i: u64) -> u64 {
        if i == 0 {
            return 0;
        }
        let n = i.leading_zeros();
        let (m_base, adj) = m_f_eq_i::<u64, f64>(i, n);
        let m = m_adj::<f64>(m_base, adj);
        let e = exp::<u64, f64>(n);
        repr::<f64>(e, m)
    }

    pub fn u128_to_f32_bits(i: u128) -> u32 {
        let n = i.leading_zeros();
        let i_m = i.wrapping_shl(n); // Mantissa, shifted so the first bit is nonzero
        let m_base = m_base::<_, f32>(i_m);

        // Within the upper `F::BITS`, everything except for the signifcand
        // gets truncated
        let d1: u32 = (i_m >> (u128::BITS - f32::BITS - f32::SIGNIFICAND_BITS - 1)).cast();

        // The entire rest of `i_m` gets truncated. Zero the upper `F::BITS` then just
        // check if it is nonzero.
        let d2: u32 = (i_m << f32::BITS >> f32::BITS != 0).into();
        let adj = d1 | d2;

        let m = m_adj::<f32>(m_base, adj);
        let e = if i == 0 { 0 } else { exp::<u128, f32>(n) };
        repr::<f32>(e, m)
    }

    pub fn u128_to_f64_bits(i: u128) -> u64 {
        let n = i.leading_zeros();
        let i_m = i.wrapping_shl(n); // Mantissa, shifted so the first bit is nonzero
        let m_base = m_base::<_, f64>(i_m);
        // The entire lower half of `i` will be truncated (masked portion), plus the
        // next `EXPONENT_BITS` bits.
        let adj = (i_m >> f64::EXPONENT_BITS | i_m & 0xFFFF_FFFF) as u64;
        let m = m_adj::<f64>(m_base, adj);
        let e = if i == 0 { 0 } else { exp::<u128, f64>(n) };
        repr::<f64>(e, m)
    }
}

// Conversions from unsigned integers to floats.
intrinsics! {
    #[arm_aeabi_alias = __aeabi_ui2f]
    pub extern "C" fn __floatunsisf(i: u32) -> f32 {
        f32::from_bits(int_to_float::u32_to_f32_bits(i))
    }

    #[arm_aeabi_alias = __aeabi_ui2d]
    pub extern "C" fn __floatunsidf(i: u32) -> f64 {
        f64::from_bits(int_to_float::u32_to_f64_bits(i))
    }

    #[arm_aeabi_alias = __aeabi_ul2f]
    pub extern "C" fn __floatundisf(i: u64) -> f32 {
        f32::from_bits(int_to_float::u64_to_f32_bits(i))
    }

    #[arm_aeabi_alias = __aeabi_ul2d]
    pub extern "C" fn __floatundidf(i: u64) -> f64 {
        f64::from_bits(int_to_float::u64_to_f64_bits(i))
    }

    #[cfg_attr(target_os = "uefi", unadjusted_on_win64)]
    pub extern "C" fn __floatuntisf(i: u128) -> f32 {
        f32::from_bits(int_to_float::u128_to_f32_bits(i))
    }

    #[cfg_attr(target_os = "uefi", unadjusted_on_win64)]
    pub extern "C" fn __floatuntidf(i: u128) -> f64 {
        f64::from_bits(int_to_float::u128_to_f64_bits(i))
    }
}

// Conversions from signed integers to floats.
intrinsics! {
    #[arm_aeabi_alias = __aeabi_i2f]
    pub extern "C" fn __floatsisf(i: i32) -> f32 {
        int_to_float::signed(i, int_to_float::u32_to_f32_bits)
    }

    #[arm_aeabi_alias = __aeabi_i2d]
    pub extern "C" fn __floatsidf(i: i32) -> f64 {
        int_to_float::signed(i, int_to_float::u32_to_f64_bits)
    }

    #[arm_aeabi_alias = __aeabi_l2f]
    pub extern "C" fn __floatdisf(i: i64) -> f32 {
        int_to_float::signed(i, int_to_float::u64_to_f32_bits)
    }

    #[arm_aeabi_alias = __aeabi_l2d]
    pub extern "C" fn __floatdidf(i: i64) -> f64 {
        int_to_float::signed(i, int_to_float::u64_to_f64_bits)
    }

    #[cfg_attr(target_os = "uefi", unadjusted_on_win64)]
    pub extern "C" fn __floattisf(i: i128) -> f32 {
        int_to_float::signed(i, int_to_float::u128_to_f32_bits)
    }

    #[cfg_attr(target_os = "uefi", unadjusted_on_win64)]
    pub extern "C" fn __floattidf(i: i128) -> f64 {
        int_to_float::signed(i, int_to_float::u128_to_f64_bits)
    }
}

/// Generic float to unsigned int conversions.
fn float_to_unsigned_int<F, U>(f: F) -> U
where
    F: Float,
    U: Int<UnsignedInt = U>,
    F::Int: CastInto<U>,
    F::Int: CastFrom<u32>,
    F::Int: CastInto<U::UnsignedInt>,
    u32: CastFrom<F::Int>,
{
    float_to_int_inner::<F, U, _, _>(f.repr(), |i: U| i, || U::MAX)
}

/// Generic float to signed int conversions.
fn float_to_signed_int<F, I>(f: F) -> I
where
    F: Float,
    I: Int + Neg<Output = I>,
    I::UnsignedInt: Int,
    F::Int: CastInto<I::UnsignedInt>,
    F::Int: CastFrom<u32>,
    u32: CastFrom<F::Int>,
{
    float_to_int_inner::<F, I, _, _>(
        f.repr() & !F::SIGN_MASK,
        |i: I| if f.is_sign_negative() { -i } else { i },
        || if f.is_sign_negative() { I::MIN } else { I::MAX },
    )
}

/// Float to int conversions, generic for both signed and unsigned.
///
/// Parameters:
/// - `fbits`: `abg(f)` bitcasted to an integer.
/// - `map_inbounds`: apply this transformation to integers that are within range (add the sign
///    back).
/// - `out_of_bounds`: return value when out of range for `I`.
fn float_to_int_inner<F, I, FnFoo, FnOob>(
    fbits: F::Int,
    map_inbounds: FnFoo,
    out_of_bounds: FnOob,
) -> I
where
    F: Float,
    I: Int,
    FnFoo: FnOnce(I) -> I,
    FnOob: FnOnce() -> I,
    I::UnsignedInt: Int,
    F::Int: CastInto<I::UnsignedInt>,
    F::Int: CastFrom<u32>,
    u32: CastFrom<F::Int>,
{
    let int_max_exp = F::EXPONENT_BIAS + I::MAX.ilog2() + 1;
    let foobar = F::EXPONENT_BIAS + I::UnsignedInt::BITS - 1;

    if fbits < F::ONE.repr() {
        // < 0 gets rounded to 0
        I::ZERO
    } else if fbits < F::Int::cast_from(int_max_exp) << F::SIGNIFICAND_BITS {
        // >= 1, < integer max
        let m_base = if I::UnsignedInt::BITS >= F::Int::BITS {
            I::UnsignedInt::cast_from(fbits) << (I::BITS - F::SIGNIFICAND_BITS - 1)
        } else {
            I::UnsignedInt::cast_from(fbits >> (F::SIGNIFICAND_BITS - I::BITS + 1))
        };

        // Set the implicit 1-bit.
        let m: I::UnsignedInt = I::UnsignedInt::ONE << (I::BITS - 1) | m_base;

        // Shift based on the exponent and bias.
        let s: u32 = (foobar) - u32::cast_from(fbits >> F::SIGNIFICAND_BITS);

        let unsigned = m >> s;
        map_inbounds(I::from_unsigned(unsigned))
    } else if fbits <= F::EXPONENT_MASK {
        // >= max (incl. inf)
        out_of_bounds()
    } else {
        I::ZERO
    }
}

// Conversions from floats to unsigned integers.
intrinsics! {
    #[arm_aeabi_alias = __aeabi_f2uiz]
    pub extern "C" fn __fixunssfsi(f: f32) -> u32 {
        float_to_unsigned_int(f)
    }

    #[arm_aeabi_alias = __aeabi_f2ulz]
    pub extern "C" fn __fixunssfdi(f: f32) -> u64 {
        float_to_unsigned_int(f)
    }

    #[win64_128bit_abi_hack]
    pub extern "C" fn __fixunssfti(f: f32) -> u128 {
        float_to_unsigned_int(f)
    }

    #[arm_aeabi_alias = __aeabi_d2uiz]
    pub extern "C" fn __fixunsdfsi(f: f64) -> u32 {
        float_to_unsigned_int(f)
    }

    #[arm_aeabi_alias = __aeabi_d2ulz]
    pub extern "C" fn __fixunsdfdi(f: f64) -> u64 {
        float_to_unsigned_int(f)
    }

    #[win64_128bit_abi_hack]
    pub extern "C" fn __fixunsdfti(f: f64) -> u128 {
        float_to_unsigned_int(f)
    }

    #[ppc_alias = __fixunskfsi]
    #[cfg(not(feature = "no-f16-f128"))]
    pub extern "C" fn __fixunstfsi(f: f128) -> u32 {
        float_to_unsigned_int(f)
    }

    #[ppc_alias = __fixunskfdi]
    #[cfg(not(feature = "no-f16-f128"))]
    pub extern "C" fn __fixunstfdi(f: f128) -> u64 {
        float_to_unsigned_int(f)
    }

    #[ppc_alias = __fixunskfti]
    #[cfg(not(feature = "no-f16-f128"))]
    pub extern "C" fn __fixunstfti(f: f128) -> u128 {
        float_to_unsigned_int(f)
    }
}

// Conversions from floats to signed integers.
intrinsics! {
    #[arm_aeabi_alias = __aeabi_f2iz]
    pub extern "C" fn __fixsfsi(f: f32) -> i32 {
        float_to_signed_int(f)
    }

    #[arm_aeabi_alias = __aeabi_f2lz]
    pub extern "C" fn __fixsfdi(f: f32) -> i64 {
        float_to_signed_int(f)
    }

    #[win64_128bit_abi_hack]
    pub extern "C" fn __fixsfti(f: f32) -> i128 {
        float_to_signed_int(f)
    }

    #[arm_aeabi_alias = __aeabi_d2iz]
    pub extern "C" fn __fixdfsi(f: f64) -> i32 {
        float_to_signed_int(f)
    }

    #[arm_aeabi_alias = __aeabi_d2lz]
    pub extern "C" fn __fixdfdi(f: f64) -> i64 {
        float_to_signed_int(f)
    }

    #[win64_128bit_abi_hack]
    pub extern "C" fn __fixdfti(f: f64) -> i128 {
        float_to_signed_int(f)
    }

    #[ppc_alias = __fixkfsi]
    #[cfg(not(feature = "no-f16-f128"))]
    pub extern "C" fn __fixtfsi(f: f128) -> i32 {
        float_to_signed_int(f)
    }

    #[ppc_alias = __fixkfdi]
    #[cfg(not(feature = "no-f16-f128"))]
    pub extern "C" fn __fixtfdi(f: f128) -> i64 {
        float_to_signed_int(f)
    }

    #[ppc_alias = __fixkfti]
    #[cfg(not(feature = "no-f16-f128"))]
    pub extern "C" fn __fixtfti(f: f128) -> i128 {
        float_to_signed_int(f)
    }
}
