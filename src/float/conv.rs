use core::ops::Neg;

use crate::int::{CastFrom, CastInto, Int, MinInt};

use super::Float;

/// Conversions from integers to floats.
///
/// These are hand-optimized bit twiddling code,
/// which unfortunately isn't the easiest kind of code to read.
///
/// The algorithm is explained here: <https://blog.m-ou.se/floats/>
mod int_to_float {
    pub fn u32_to_f32_bits(i: u32) -> u32 {
        if i == 0 {
            return 0;
        }
        let n = i.leading_zeros();
        let a = (i << n) >> 8; // Significant bits, with bit 24 still in tact.
        let b = (i << n) << 24; // Insignificant bits, only relevant for rounding.
        let m = a + ((b - (b >> 31 & !a)) >> 31); // Add one when we need to round up. Break ties to even.
        let e = 157 - n; // Exponent plus 127, minus one.
        (e << 23) + m // + not |, so the mantissa can overflow into the exponent.
    }

    pub fn u32_to_f64_bits(i: u32) -> u64 {
        if i == 0 {
            return 0;
        }
        let n = i.leading_zeros();
        let m = (i as u64) << (21 + n); // Significant bits, with bit 53 still in tact.
        let e = 1053 - n as u64; // Exponent plus 1023, minus one.
        (e << 52) + m // Bit 53 of m will overflow into e.
    }

    pub fn u64_to_f32_bits(i: u64) -> u32 {
        let n = i.leading_zeros();
        let y = i.wrapping_shl(n);
        let a = (y >> 40) as u32; // Significant bits, with bit 24 still in tact.
        let b = (y >> 8 | y & 0xFFFF) as u32; // Insignificant bits, only relevant for rounding.
        let m = a + ((b - (b >> 31 & !a)) >> 31); // Add one when we need to round up. Break ties to even.
        let e = if i == 0 { 0 } else { 189 - n }; // Exponent plus 127, minus one, except for zero.
        (e << 23) + m // + not |, so the mantissa can overflow into the exponent.
    }

    pub fn u64_to_f64_bits(i: u64) -> u64 {
        if i == 0 {
            return 0;
        }
        let n = i.leading_zeros();
        let a = (i << n) >> 11; // Significant bits, with bit 53 still in tact.
        let b = (i << n) << 53; // Insignificant bits, only relevant for rounding.
        let m = a + ((b - (b >> 63 & !a)) >> 63); // Add one when we need to round up. Break ties to even.
        let e = 1085 - n as u64; // Exponent plus 1023, minus one.
        (e << 52) + m // + not |, so the mantissa can overflow into the exponent.
    }

    pub fn u128_to_f32_bits(i: u128) -> u32 {
        let n = i.leading_zeros();
        let y = i.wrapping_shl(n);
        let a = (y >> 104) as u32; // Significant bits, with bit 24 still in tact.
        let b = (y >> 72) as u32 | ((y << 32) >> 32 != 0) as u32; // Insignificant bits, only relevant for rounding.
        let m = a + ((b - (b >> 31 & !a)) >> 31); // Add one when we need to round up. Break ties to even.
        let e = if i == 0 { 0 } else { 253 - n }; // Exponent plus 127, minus one, except for zero.
        (e << 23) + m // + not |, so the mantissa can overflow into the exponent.
    }

    pub fn u128_to_f64_bits(i: u128) -> u64 {
        let n = i.leading_zeros();
        let y = i.wrapping_shl(n);
        let a = (y >> 75) as u64; // Significant bits, with bit 53 still in tact.
        let b = (y >> 11 | y & 0xFFFF_FFFF) as u64; // Insignificant bits, only relevant for rounding.
        let m = a + ((b - (b >> 63 & !a)) >> 63); // Add one when we need to round up. Break ties to even.
        let e = if i == 0 { 0 } else { 1149 - n as u64 }; // Exponent plus 1023, minus one, except for zero.
        (e << 52) + m // + not |, so the mantissa can overflow into the exponent.
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
        let sign_bit = ((i >> 31) as u32) << 31;
        f32::from_bits(int_to_float::u32_to_f32_bits(i.unsigned_abs()) | sign_bit)
    }

    #[arm_aeabi_alias = __aeabi_i2d]
    pub extern "C" fn __floatsidf(i: i32) -> f64 {
        let sign_bit = ((i >> 31) as u64) << 63;
        f64::from_bits(int_to_float::u32_to_f64_bits(i.unsigned_abs()) | sign_bit)
    }

    #[arm_aeabi_alias = __aeabi_l2f]
    pub extern "C" fn __floatdisf(i: i64) -> f32 {
        let sign_bit = ((i >> 63) as u32) << 31;
        f32::from_bits(int_to_float::u64_to_f32_bits(i.unsigned_abs()) | sign_bit)
    }

    #[arm_aeabi_alias = __aeabi_l2d]
    pub extern "C" fn __floatdidf(i: i64) -> f64 {
        let sign_bit = ((i >> 63) as u64) << 63;
        f64::from_bits(int_to_float::u64_to_f64_bits(i.unsigned_abs()) | sign_bit)
    }

    #[cfg_attr(target_os = "uefi", unadjusted_on_win64)]
    pub extern "C" fn __floattisf(i: i128) -> f32 {
        let sign_bit = ((i >> 127) as u32) << 31;
        f32::from_bits(int_to_float::u128_to_f32_bits(i.unsigned_abs()) | sign_bit)
    }

    #[cfg_attr(target_os = "uefi", unadjusted_on_win64)]
    pub extern "C" fn __floattidf(i: i128) -> f64 {
        let sign_bit = ((i >> 127) as u64) << 63;
        f64::from_bits(int_to_float::u128_to_f64_bits(i.unsigned_abs()) | sign_bit)
    }
}

fn float_to_unsigned_int<F, U>(f: F) -> U
where
    F: Float,
    U: Int,
    u32: CastFrom<F::Int>,
    F::Int: CastInto<U>,
    F::Int: CastFrom<u32>,
{
    let uint_max_exp: u32 = F::EXPONENT_BIAS + U::MAX.ilog2() + 1;
    let fbits = f.repr();

    if fbits < F::ONE.repr() {
        // >= 0.0, < 1.0 (< 0.0 are > 1.0 in int repr)
        U::ZERO
    } else if fbits < F::Int::cast_from(uint_max_exp) << F::SIGNIFICAND_BITS {
        // >= 1, < U::max
        let mantissa = if U::BITS >= F::Int::BITS {
            U::cast_from(fbits) << (U::BITS - F::SIGNIFICAND_BITS - 1)
        } else {
            // FIXME magic number for when we go smaller
            U::cast_from(fbits >> 21)
        };

        // Set the implicit 1-bit.
        let m: U = U::ONE << (U::BITS - 1) | mantissa;
        // Shift based on the exponent and bias.
        let s: u32 = (uint_max_exp - 1) - u32::cast_from(fbits >> F::SIGNIFICAND_BITS);

        m >> s
    } else if fbits <= F::EXPONENT_MASK {
        // >= max (incl. inf)
        U::MAX
    } else {
        U::ZERO
    }
}

fn float_to_signed_int<F, I>(f: F) -> I
where
    F: Float,
    I: Int + Neg<Output = I>,
    I::UnsignedInt: Int,
    u32: CastFrom<F::Int>,
    F::Int: CastInto<I::UnsignedInt>,
    F::Int: CastFrom<u32>,
{
    let int_max_exp: u32 = F::EXPONENT_BIAS + I::MAX.ilog2() + 1;
    let fbits = f.repr() & !F::SIGN_MASK;

    if fbits < F::ONE.repr() {
        // >= 0.0, < 1.0 (< 0.0 are > 1.0 in int repr)
        I::ZERO
    } else if fbits < F::Int::cast_from(int_max_exp) << F::SIGNIFICAND_BITS {
        // >= 1, < U::max
        let mantissa = if I::BITS >= F::Int::BITS {
            I::UnsignedInt::cast_from(fbits) << (I::BITS - F::SIGNIFICAND_BITS - 1)
        } else {
            I::UnsignedInt::cast_from(fbits >> 21)
        };

        // Set the implicit 1-bit.
        let m: I::UnsignedInt = I::UnsignedInt::ONE << (I::BITS - 1) | mantissa;
        // Shift based on the exponent and bias.
        let s: u32 = int_max_exp - u32::cast_from(fbits >> F::SIGNIFICAND_BITS);
        let u: I = I::from_unsigned(m >> s);
        if f.is_sign_negative() {
            -u
        } else {
            u
        }
    } else if fbits <= F::EXPONENT_MASK {
        // >= max (incl. inf)
        if f.is_sign_negative() {
            I::MIN
        } else {
            I::MAX
        }
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
}
