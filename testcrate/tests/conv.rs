#![cfg_attr(not(feature = "no-f16-f128"), feature(f16))]
#![cfg_attr(not(feature = "no-f16-f128"), feature(f128))]
// makes configuration easier
#![allow(unused_macros)]

use compiler_builtins::float::Float;
use rustc_apfloat::ieee::{Double, Single};
#[cfg(not(feature = "no-f16-f128"))]
use rustc_apfloat::ieee::{Half, Quad};
use rustc_apfloat::{Float as _, FloatConvert as _};
use testcrate::*;

macro_rules! i_to_f {
    ($($from:ty, $into:ty, $fn:ident);*;) => {
        $(
            fuzz(N, |x: $from| {
                let f0 = x as $into;
                let f1: $into = $fn(x);
                // This makes sure that the conversion produced the best rounding possible, and does
                // this independent of `x as $into` rounding correctly.
                // This assumes that float to integer conversion is correct.
                let y_minus_ulp = <$into>::from_bits(f1.to_bits().wrapping_sub(1)) as $from;
                let y = f1 as $from;
                let y_plus_ulp = <$into>::from_bits(f1.to_bits().wrapping_add(1)) as $from;
                let error_minus = <$from as Int>::abs_diff(y_minus_ulp, x);
                let error = <$from as Int>::abs_diff(y, x);
                let error_plus = <$from as Int>::abs_diff(y_plus_ulp, x);
                // The first two conditions check that none of the two closest float values are
                // strictly closer in representation to `x`. The second makes sure that rounding is
                // towards even significand if two float values are equally close to the integer.
                if error_minus < error
                    || error_plus < error
                    || ((error_minus == error || error_plus == error)
                        && ((f0.to_bits() & 1) != 0))
                {
                    if !cfg!(any(
                        target_arch = "powerpc",
                        target_arch = "powerpc64"
                    )) {
                        panic!(
                            "incorrect rounding by {}({}): {}, ({}, {}, {}), errors ({}, {}, {})",
                            stringify!($fn),
                            x,
                            f1.to_bits(),
                            y_minus_ulp,
                            y,
                            y_plus_ulp,
                            error_minus,
                            error,
                            error_plus,
                        );
                    }
                }
                // Test against native conversion. We disable testing on all `x86` because of
                // rounding bugs with `i686`. `powerpc` also has the same rounding bug.
                if f0 != f1 && !cfg!(any(
                    target_arch = "x86",
                    target_arch = "powerpc",
                    target_arch = "powerpc64"
                )) {
                    panic!(
                        "{}({}): std: {}, builtins: {}",
                        stringify!($fn),
                        x,
                        f0,
                        f1,
                    );
                }
            });
        )*
    };
}

#[test]
fn int_to_float() {
    use compiler_builtins::float::conv::{
        __floatdidf, __floatdisf, __floatsidf, __floatsisf, __floattidf, __floattisf,
        __floatundidf, __floatundisf, __floatunsidf, __floatunsisf, __floatuntidf, __floatuntisf,
    };
    use compiler_builtins::int::Int;

    i_to_f!(
        u32, f32, __floatunsisf;
        u32, f64, __floatunsidf;
        i32, f32, __floatsisf;
        i32, f64, __floatsidf;
        u64, f32, __floatundisf;
        u64, f64, __floatundidf;
        i64, f32, __floatdisf;
        i64, f64, __floatdidf;
        u128, f32, __floatuntisf;
        u128, f64, __floatuntidf;
        i128, f32, __floattisf;
        i128, f64, __floattidf;
    );
}

macro_rules! f_to_i {
    ($x:ident, $($f:ty, $fn:ident);*;) => {
        $(
            // it is undefined behavior in the first place to do conversions with NaNs
            if !$x.is_nan() {
                let conv0 = $x as $f;
                let conv1: $f = $fn($x);
                if conv0 != conv1 {
                    panic!("{}({}): std: {}, builtins: {}", stringify!($fn), $x, conv0, conv1);
                }
            }
        )*
    };
}

// PowerPC tests are failing on LLVM 13: https://github.com/rust-lang/rust/issues/88520
#[cfg(not(target_arch = "powerpc64"))]
#[test]
fn float_to_int() {
    use compiler_builtins::float::conv::{
        __fixdfdi, __fixdfsi, __fixdfti, __fixsfdi, __fixsfsi, __fixsfti, __fixunsdfdi,
        __fixunsdfsi, __fixunsdfti, __fixunssfdi, __fixunssfsi, __fixunssfti,
    };
    println!("bias {}", f32::EXPONENT_BIAS);
    println!("bits {}", f32::SIGNIFICAND_BITS);
    println!("bias {}", f64::EXPONENT_BIAS);
    println!("bits {}", f64::SIGNIFICAND_BITS);

    fuzz_float(N, |x: f32| {
        f_to_i!(x,
            u32, __fixunssfsi;
            u64, __fixunssfdi;
            u128, __fixunssfti;
            i32, __fixsfsi;
            i64, __fixsfdi;
            i128, __fixsfti;
        );
    });
    fuzz_float(N, |x: f64| {
        f_to_i!(x,
            u32, __fixunsdfsi;
            u64, __fixunsdfdi;
            u128, __fixunsdfti;
            i32, __fixdfsi;
            i64, __fixdfdi;
            i128, __fixdfti;
        );
    });
}

macro_rules! conv {
    ($fX:ident, $fD:ident, $fn:ident, $apfloatX:ident, $apfloatD:ident) => {
        fuzz_float(N, |x: $fX| {
            let tmp0: $apfloatD = $apfloatX::from_bits(x.to_bits().into())
                .convert(&mut false)
                .value;
            let tmp0 = $fD::from_bits(tmp0.to_bits().try_into().unwrap());
            let tmp1: $fD = $fn(x);
            if !Float::eq_repr(tmp0, tmp1) {
                panic!(
                    "{}({x:?}): apfloat: {tmp0:?}, builtins: {tmp1:?}",
                    stringify!($fn)
                );
            }
        })
    };
}

macro_rules! extend {
    ($fX:ident, $fD:ident, $fn:ident) => {
        fuzz_float(N, |x: $fX| {
            let tmp0 = x as $fD;
            let tmp1: $fD = $fn(x);
            if !Float::eq_repr(tmp0, tmp1) {
                panic!(
                    "{}({}): std: {}, builtins: {}",
                    stringify!($fn),
                    x,
                    tmp0,
                    tmp1
                );
            }
        });
    };
}

// PowerPC tests are failing on LLVM 13: https://github.com/rust-lang/rust/issues/88520
#[cfg(not(target_arch = "powerpc64"))]
#[test]
fn float_extend() {
    use compiler_builtins::float::extend::__extendsfdf2;

    extend!(f32, f64, __extendsfdf2);
    conv!(f32, f64, __extendsfdf2, Single, Double);
    #[cfg(not(feature = "no-f16-f128"))]
    {
        #[cfg(any(target_arch = "powerpc", target_arch = "powerpc64"))]
        use compiler_builtins::float::extend::{
            __extenddfkf2 as __extenddftf2, __extendhfkf2 as __extendhftf2,
            __extendsfkf2 as __extendsftf2,
        };
        #[cfg(not(any(target_arch = "powerpc", target_arch = "powerpc64")))]
        use compiler_builtins::float::extend::{__extenddftf2, __extendhftf2, __extendsftf2};
        use compiler_builtins::float::extend::{__extendhfsf2, __gnu_h2f_ieee};

        // FIXME(f16_f128): Also do extend!() for `f16` and `f128` when builtins are in nightly
        conv!(f16, f32, __extendhfsf2, Half, Single);
        conv!(f16, f32, __gnu_h2f_ieee, Half, Single);
        conv!(f16, f128, __extendhftf2, Half, Quad);
        conv!(f32, f128, __extendsftf2, Single, Quad);
        conv!(f64, f128, __extenddftf2, Double, Quad);
    }
}

#[cfg(target_arch = "arm")]
#[test]
fn float_extend_arm() {
    use compiler_builtins::float::extend::__extendsfdf2vfp;

    extend!(f32, f64, __extendsfdf2vfp);
    conv!(f32, f64, __extendsfdf2vfp, Single, Double);
}

macro_rules! trunc {
    ($fX:ident, $fD:ident, $fn:ident) => {
        fuzz_float(N, |x: $fX| {
            let tmp0 = x as $fD;
            let tmp1: $fD = $fn(x);
            if !Float::eq_repr(tmp0, tmp1) {
                panic!(
                    "{}({}): std: {}, builtins: {}",
                    stringify!($fn),
                    x,
                    tmp0,
                    tmp1
                );
            }
        });
    };
}

#[test]
fn float_trunc() {
    use compiler_builtins::float::trunc::__truncdfsf2;

    trunc!(f64, f32, __truncdfsf2);
    conv!(f64, f32, __truncdfsf2, Double, Single);
    #[cfg(not(feature = "no-f16-f128"))]
    {
        use compiler_builtins::float::trunc::{__gnu_f2h_ieee, __truncdfhf2, __truncsfhf2};
        #[cfg(any(target_arch = "powerpc", target_arch = "powerpc64"))]
        use compiler_builtins::float::trunc::{
            __trunckfdf2 as __trunctfdf2, __trunckfhf2 as __trunctfhf2,
            __trunckfsf2 as __trunctfsf2,
        };
        #[cfg(not(any(target_arch = "powerpc", target_arch = "powerpc64")))]
        use compiler_builtins::float::trunc::{__trunctfdf2, __trunctfhf2, __trunctfsf2};

        // FIXME(f16_f128): Also do trunc!() for `f16` and `f128` when builtins are in nightly
        conv!(f32, f16, __truncsfhf2, Single, Half);
        conv!(f32, f16, __gnu_f2h_ieee, Single, Half);
        conv!(f64, f16, __truncdfhf2, Double, Half);
        conv!(f128, f16, __trunctfhf2, Quad, Half);
        conv!(f128, f32, __trunctfsf2, Quad, Single);
        conv!(f128, f64, __trunctfdf2, Quad, Double);
    }
}

#[cfg(target_arch = "arm")]
#[test]
fn float_trunc_arm() {
    use compiler_builtins::float::trunc::__truncdfsf2vfp;

    trunc!(f64, f32, __truncdfsf2vfp);
    conv!(f64, f32, __truncdfsf2vfp, Double, Single)
}
