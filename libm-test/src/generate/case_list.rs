//! Test cases to verify specific values.
//!
//! Each routine can have a set of inputs and, optinoally, outputs. If an output is provided, it
//! will be used to check against. If only inputs are provided, the case will be checked against
//! a basis.
//!
//! This is useful for adding regression tests or expected failures.

#[cfg(f16_enabled)]
use libm::hf16;
#[cfg(f128_enabled)]
use libm::hf128;
use libm::{hf32, hf64};

use crate::{CheckBasis, CheckCtx, Float, GeneratorKind, MathOp, op};

pub struct TestCase<Op: MathOp> {
    pub input: Op::RustArgs,
    pub output: Option<Op::RustRet>,
}

impl<Op: MathOp> TestCase<Op> {
    /// Turn into a different operation with the same types.
    fn cast<Op2: MathOp>(self) -> TestCase<Op2>
    where
        Op2::RustArgs: From<Op::RustArgs>,
        Op2::RustRet: From<Op::RustRet>,
    {
        TestCase {
            input: self.input.into(),
            output: self.output.map(Into::into),
        }
    }
}

macro_rules! cases {
    (
        $(
            $(#[$($meta:tt)*])*
            ($($tt:tt)*)
        ),* $(,)?
    ) => {{
        Vec::from_iter([
            $(
               $(#[$($meta)*])*
                cases!(@single $($tt)*),
            )*
        ])
    }};

    // Variant without a result, which will check against MPFR.
    (@single ($($arg:expr),* $(,)?), None $(,)?) => {
        TestCase{
            input: ($($arg,)*),
            output: None,
        }
    };

    // Variant for when the result is specified.
    (@single ($($arg:expr),* $(,)?), $res:expr $(,)?) => {
        TestCase{
            input: ($($arg,)*),
            output: Some($res),
        }
    };
}

/********************************
 * compiler-builtins test cases *
 ********************************/

#[cfg(f16_enabled)]
fn addf16_cases() -> Vec<TestCase<op::addf16::Routine>> {
    cases![]
}

fn addf32_cases() -> Vec<TestCase<op::addf32::Routine>> {
    cases![]
}

fn addf64_cases() -> Vec<TestCase<op::addf64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn addf128_cases() -> Vec<TestCase<op::addf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn subf16_cases() -> Vec<TestCase<op::subf16::Routine>> {
    cases![]
}

fn subf32_cases() -> Vec<TestCase<op::subf32::Routine>> {
    cases![]
}

fn subf64_cases() -> Vec<TestCase<op::subf64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn subf128_cases() -> Vec<TestCase<op::subf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn mulf16_cases() -> Vec<TestCase<op::mulf16::Routine>> {
    cases![]
}

fn mulf32_cases() -> Vec<TestCase<op::mulf32::Routine>> {
    cases![]
}

fn mulf64_cases() -> Vec<TestCase<op::mulf64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn mulf128_cases() -> Vec<TestCase<op::mulf128::Routine>> {
    cases![]
}

fn divf32_cases() -> Vec<TestCase<op::divf32::Routine>> {
    cases![]
}

fn divf64_cases() -> Vec<TestCase<op::divf64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn divf128_cases() -> Vec<TestCase<op::divf128::Routine>> {
    cases![]
}

fn powif32_cases() -> Vec<TestCase<op::powif32::Routine>> {
    cases![]
}

fn powif64_cases() -> Vec<TestCase<op::powif64::Routine>> {
    cases![
        // High error
        ((0.9999497584118668, -5858518), 6.823250355352412e127),
    ]
}

#[cfg(f128_enabled)]
fn powif128_cases() -> Vec<TestCase<op::powif128::Routine>> {
    cases![]
}

/* comparison */

#[cfg(f16_enabled)]
fn eqf16_cases() -> Vec<TestCase<op::eqf16::Routine>> {
    cases![]
}

fn eqf32_cases() -> Vec<TestCase<op::eqf32::Routine>> {
    cases![]
}

fn eqf64_cases() -> Vec<TestCase<op::eqf64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn eqf128_cases() -> Vec<TestCase<op::eqf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn gtf16_cases() -> Vec<TestCase<op::gtf16::Routine>> {
    cases![]
}

fn gtf32_cases() -> Vec<TestCase<op::gtf32::Routine>> {
    cases![]
}

fn gtf64_cases() -> Vec<TestCase<op::gtf64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn gtf128_cases() -> Vec<TestCase<op::gtf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn gef16_cases() -> Vec<TestCase<op::gef16::Routine>> {
    cases![]
}

fn gef32_cases() -> Vec<TestCase<op::gef32::Routine>> {
    cases![]
}

fn gef64_cases() -> Vec<TestCase<op::gef64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn gef128_cases() -> Vec<TestCase<op::gef128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn ltf16_cases() -> Vec<TestCase<op::ltf16::Routine>> {
    cases![]
}

fn ltf32_cases() -> Vec<TestCase<op::ltf32::Routine>> {
    cases![]
}

fn ltf64_cases() -> Vec<TestCase<op::ltf64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn ltf128_cases() -> Vec<TestCase<op::ltf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn lef16_cases() -> Vec<TestCase<op::lef16::Routine>> {
    cases![]
}

fn lef32_cases() -> Vec<TestCase<op::lef32::Routine>> {
    cases![]
}

fn lef64_cases() -> Vec<TestCase<op::lef64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn lef128_cases() -> Vec<TestCase<op::lef128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn nef16_cases() -> Vec<TestCase<op::nef16::Routine>> {
    cases![]
}

fn nef32_cases() -> Vec<TestCase<op::nef32::Routine>> {
    cases![]
}

fn nef64_cases() -> Vec<TestCase<op::nef64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn nef128_cases() -> Vec<TestCase<op::nef128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn unordf16_cases() -> Vec<TestCase<op::unordf16::Routine>> {
    cases![]
}

fn unordf32_cases() -> Vec<TestCase<op::unordf32::Routine>> {
    cases![]
}

fn unordf64_cases() -> Vec<TestCase<op::unordf64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn unordf128_cases() -> Vec<TestCase<op::unordf128::Routine>> {
    cases![]
}

/* conversion */

#[cfg(f16_enabled)]
fn extend_f16_f32_cases() -> Vec<TestCase<op::extend_f16_f32::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn extend_f16_f64_cases() -> Vec<TestCase<op::extend_f16_f64::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
#[cfg(f128_enabled)]
fn extend_f16_f128_cases() -> Vec<TestCase<op::extend_f16_f128::Routine>> {
    cases![]
}

fn extend_f32_f64_cases() -> Vec<TestCase<op::extend_f32_f64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn extend_f32_f128_cases() -> Vec<TestCase<op::extend_f32_f128::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn extend_f64_f128_cases() -> Vec<TestCase<op::extend_f64_f128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn narrow_f32_f16_cases() -> Vec<TestCase<op::narrow_f32_f16::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn narrow_f64_f16_cases() -> Vec<TestCase<op::narrow_f64_f16::Routine>> {
    cases![]
}

fn narrow_f64_f32_cases() -> Vec<TestCase<op::narrow_f64_f32::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
#[cfg(f128_enabled)]
fn narrow_f128_f16_cases() -> Vec<TestCase<op::narrow_f128_f16::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn narrow_f128_f32_cases() -> Vec<TestCase<op::narrow_f128_f32::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn narrow_f128_f64_cases() -> Vec<TestCase<op::narrow_f128_f64::Routine>> {
    cases![]
}

fn ftoi_f32_i32_cases() -> Vec<TestCase<op::ftoi_f32_i32::Routine>> {
    cases![]
}

fn ftoi_f32_i64_cases() -> Vec<TestCase<op::ftoi_f32_i64::Routine>> {
    cases![]
}

fn ftoi_f32_i128_cases() -> Vec<TestCase<op::ftoi_f32_i128::Routine>> {
    cases![]
}

fn ftoi_f64_i32_cases() -> Vec<TestCase<op::ftoi_f64_i32::Routine>> {
    cases![]
}

fn ftoi_f64_i64_cases() -> Vec<TestCase<op::ftoi_f64_i64::Routine>> {
    cases![]
}

fn ftoi_f64_i128_cases() -> Vec<TestCase<op::ftoi_f64_i128::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn ftoi_f128_i32_cases() -> Vec<TestCase<op::ftoi_f128_i32::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn ftoi_f128_i64_cases() -> Vec<TestCase<op::ftoi_f128_i64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn ftoi_f128_i128_cases() -> Vec<TestCase<op::ftoi_f128_i128::Routine>> {
    cases![]
}

fn ftoi_f32_u32_cases() -> Vec<TestCase<op::ftoi_f32_u32::Routine>> {
    cases![]
}

fn ftoi_f32_u64_cases() -> Vec<TestCase<op::ftoi_f32_u64::Routine>> {
    cases![]
}

fn ftoi_f32_u128_cases() -> Vec<TestCase<op::ftoi_f32_u128::Routine>> {
    cases![]
}

fn ftoi_f64_u32_cases() -> Vec<TestCase<op::ftoi_f64_u32::Routine>> {
    cases![]
}

fn ftoi_f64_u64_cases() -> Vec<TestCase<op::ftoi_f64_u64::Routine>> {
    cases![]
}

fn ftoi_f64_u128_cases() -> Vec<TestCase<op::ftoi_f64_u128::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn ftoi_f128_u32_cases() -> Vec<TestCase<op::ftoi_f128_u32::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn ftoi_f128_u64_cases() -> Vec<TestCase<op::ftoi_f128_u64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn ftoi_f128_u128_cases() -> Vec<TestCase<op::ftoi_f128_u128::Routine>> {
    cases![]
}

fn itof_i32_f32_cases() -> Vec<TestCase<op::itof_i32_f32::Routine>> {
    cases![]
}

fn itof_i64_f32_cases() -> Vec<TestCase<op::itof_i64_f32::Routine>> {
    cases![]
}

fn itof_i128_f32_cases() -> Vec<TestCase<op::itof_i128_f32::Routine>> {
    cases![]
}

fn itof_i32_f64_cases() -> Vec<TestCase<op::itof_i32_f64::Routine>> {
    cases![]
}

fn itof_i64_f64_cases() -> Vec<TestCase<op::itof_i64_f64::Routine>> {
    cases![]
}

fn itof_i128_f64_cases() -> Vec<TestCase<op::itof_i128_f64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn itof_i32_f128_cases() -> Vec<TestCase<op::itof_i32_f128::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn itof_i64_f128_cases() -> Vec<TestCase<op::itof_i64_f128::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn itof_i128_f128_cases() -> Vec<TestCase<op::itof_i128_f128::Routine>> {
    cases![]
}

fn itof_u32_f32_cases() -> Vec<TestCase<op::itof_u32_f32::Routine>> {
    cases![]
}

fn itof_u64_f32_cases() -> Vec<TestCase<op::itof_u64_f32::Routine>> {
    cases![]
}

fn itof_u128_f32_cases() -> Vec<TestCase<op::itof_u128_f32::Routine>> {
    cases![]
}

fn itof_u32_f64_cases() -> Vec<TestCase<op::itof_u32_f64::Routine>> {
    cases![]
}

fn itof_u64_f64_cases() -> Vec<TestCase<op::itof_u64_f64::Routine>> {
    cases![]
}

fn itof_u128_f64_cases() -> Vec<TestCase<op::itof_u128_f64::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn itof_u32_f128_cases() -> Vec<TestCase<op::itof_u32_f128::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn itof_u64_f128_cases() -> Vec<TestCase<op::itof_u64_f128::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn itof_u128_f128_cases() -> Vec<TestCase<op::itof_u128_f128::Routine>> {
    cases![]
}

/* int arithmetic */

fn iadd_i128_cases() -> Vec<TestCase<op::iadd_i128::Routine>> {
    cases![]
}

fn iadd_u128_cases() -> Vec<TestCase<op::iadd_u128::Routine>> {
    cases![]
}

fn iaddo_i128_cases() -> Vec<TestCase<op::iaddo_i128::Routine>> {
    cases![]
}

fn iaddo_u128_cases() -> Vec<TestCase<op::iaddo_u128::Routine>> {
    cases![]
}

fn isub_i128_cases() -> Vec<TestCase<op::isub_i128::Routine>> {
    cases![]
}

fn isub_u128_cases() -> Vec<TestCase<op::isub_u128::Routine>> {
    cases![]
}

fn isubo_i128_cases() -> Vec<TestCase<op::isubo_i128::Routine>> {
    cases![]
}

fn isubo_u128_cases() -> Vec<TestCase<op::isubo_u128::Routine>> {
    cases![]
}

fn idiv_i128_cases() -> Vec<TestCase<op::idiv_i128::Routine>> {
    cases![]
}

fn idiv_i32_cases() -> Vec<TestCase<op::idiv_i32::Routine>> {
    cases![]
}

fn idiv_i64_cases() -> Vec<TestCase<op::idiv_i64::Routine>> {
    cases![]
}

fn idiv_u128_cases() -> Vec<TestCase<op::idiv_u128::Routine>> {
    cases![]
}

fn idiv_u32_cases() -> Vec<TestCase<op::idiv_u32::Routine>> {
    cases![]
}

fn idiv_u64_cases() -> Vec<TestCase<op::idiv_u64::Routine>> {
    cases![]
}

fn idivmod_i128_cases() -> Vec<TestCase<op::idivmod_i128::Routine>> {
    cases![]
}

fn idivmod_i32_cases() -> Vec<TestCase<op::idivmod_i32::Routine>> {
    cases![]
}

fn idivmod_i64_cases() -> Vec<TestCase<op::idivmod_i64::Routine>> {
    cases![]
}

fn idivmod_u128_cases() -> Vec<TestCase<op::idivmod_u128::Routine>> {
    cases![]
}

fn idivmod_u32_cases() -> Vec<TestCase<op::idivmod_u32::Routine>> {
    cases![]
}

fn idivmod_u64_cases() -> Vec<TestCase<op::idivmod_u64::Routine>> {
    cases![]
}

fn imod_i128_cases() -> Vec<TestCase<op::imod_i128::Routine>> {
    cases![]
}

fn imod_i32_cases() -> Vec<TestCase<op::imod_i32::Routine>> {
    cases![]
}

fn imod_i64_cases() -> Vec<TestCase<op::imod_i64::Routine>> {
    cases![]
}

fn imod_u128_cases() -> Vec<TestCase<op::imod_u128::Routine>> {
    cases![]
}

fn imod_u32_cases() -> Vec<TestCase<op::imod_u32::Routine>> {
    cases![]
}

fn imod_u64_cases() -> Vec<TestCase<op::imod_u64::Routine>> {
    cases![]
}

fn imul_i128_cases() -> Vec<TestCase<op::imul_i128::Routine>> {
    cases![]
}

fn imul_u64_cases() -> Vec<TestCase<op::imul_u64::Routine>> {
    cases![]
}

fn imulo_i128_cases() -> Vec<TestCase<op::imulo_i128::Routine>> {
    cases![]
}

fn imulo_i32_cases() -> Vec<TestCase<op::imulo_i32::Routine>> {
    cases![]
}

fn imulo_i64_cases() -> Vec<TestCase<op::imulo_i64::Routine>> {
    cases![]
}

fn imulo_u128_cases() -> Vec<TestCase<op::imulo_u128::Routine>> {
    cases![]
}

/* int shifts */

fn ashl_u32_cases() -> Vec<TestCase<op::ashl_u32::Routine>> {
    cases![]
}

fn ashl_u64_cases() -> Vec<TestCase<op::ashl_u64::Routine>> {
    cases![]
}

fn ashl_u128_cases() -> Vec<TestCase<op::ashl_u128::Routine>> {
    cases![]
}

fn ashr_i32_cases() -> Vec<TestCase<op::ashr_i32::Routine>> {
    cases![]
}

fn ashr_i64_cases() -> Vec<TestCase<op::ashr_i64::Routine>> {
    cases![]
}

fn ashr_i128_cases() -> Vec<TestCase<op::ashr_i128::Routine>> {
    cases![]
}

fn lshr_u32_cases() -> Vec<TestCase<op::lshr_u32::Routine>> {
    cases![]
}

fn lshr_u64_cases() -> Vec<TestCase<op::lshr_u64::Routine>> {
    cases![]
}

fn lshr_u128_cases() -> Vec<TestCase<op::lshr_u128::Routine>> {
    cases![]
}

/* int bitwise ops */

fn leading_zeros_u32_cases() -> Vec<TestCase<op::leading_zeros_u32::Routine>> {
    cases![]
}

fn leading_zeros_u64_cases() -> Vec<TestCase<op::leading_zeros_u64::Routine>> {
    cases![]
}

fn leading_zeros_u128_cases() -> Vec<TestCase<op::leading_zeros_u128::Routine>> {
    cases![]
}

fn trailing_zeros_u32_cases() -> Vec<TestCase<op::trailing_zeros_u32::Routine>> {
    cases![]
}

fn trailing_zeros_u64_cases() -> Vec<TestCase<op::trailing_zeros_u64::Routine>> {
    cases![]
}

fn trailing_zeros_u128_cases() -> Vec<TestCase<op::trailing_zeros_u128::Routine>> {
    cases![]
}

/*******************
 * libm test cases *
 *******************/

fn acos_cases() -> Vec<TestCase<op::acos::Routine>> {
    cases![]
}

fn acosf_cases() -> Vec<TestCase<op::acosf::Routine>> {
    cases![]
}

fn acosh_cases() -> Vec<TestCase<op::acosh::Routine>> {
    cases![]
}

fn acoshf_cases() -> Vec<TestCase<op::acoshf::Routine>> {
    cases![]
}

fn asin_cases() -> Vec<TestCase<op::asin::Routine>> {
    cases![]
}

fn asinf_cases() -> Vec<TestCase<op::asinf::Routine>> {
    cases![]
}

fn asinhf_cases() -> Vec<TestCase<op::asinhf::Routine>> {
    cases![
        // Failure on i586
        ((-0.37330312), -0.3651353),
        ((-0.421092), -0.40954682),
    ]
}

fn asinh_cases() -> Vec<TestCase<op::asinh::Routine>> {
    cases![]
}

fn atan_cases() -> Vec<TestCase<op::atan::Routine>> {
    cases![]
}

fn atan2_cases() -> Vec<TestCase<op::atan2::Routine>> {
    cases![]
}

fn atan2f_cases() -> Vec<TestCase<op::atan2f::Routine>> {
    cases![]
}

fn atanf_cases() -> Vec<TestCase<op::atanf::Routine>> {
    cases![]
}

fn atanh_cases() -> Vec<TestCase<op::atanh::Routine>> {
    cases![]
}

fn atanhf_cases() -> Vec<TestCase<op::atanhf::Routine>> {
    cases![]
}

fn cbrt_cases() -> Vec<TestCase<op::cbrt::Routine>> {
    cases![]
}

fn cbrtf_cases() -> Vec<TestCase<op::cbrtf::Routine>> {
    cases![]
}

fn ceil_cases() -> Vec<TestCase<op::ceil::Routine>> {
    cases![]
}

fn ceilf_cases() -> Vec<TestCase<op::ceilf::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn ceilf128_cases() -> Vec<TestCase<op::ceilf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn ceilf16_cases() -> Vec<TestCase<op::ceilf16::Routine>> {
    cases![]
}

fn copysign_cases() -> Vec<TestCase<op::copysign::Routine>> {
    cases![]
}

fn copysignf_cases() -> Vec<TestCase<op::copysignf::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn copysignf128_cases() -> Vec<TestCase<op::copysignf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn copysignf16_cases() -> Vec<TestCase<op::copysignf16::Routine>> {
    cases![]
}

fn cos_cases() -> Vec<TestCase<op::cos::Routine>> {
    cases![]
}

fn cosf_cases() -> Vec<TestCase<op::cosf::Routine>> {
    cases![]
}

fn cosh_cases() -> Vec<TestCase<op::cosh::Routine>> {
    cases![]
}

fn coshf_cases() -> Vec<TestCase<op::coshf::Routine>> {
    cases![]
}

fn erf_cases() -> Vec<TestCase<op::erf::Routine>> {
    cases![]
}

fn erfc_cases() -> Vec<TestCase<op::erfc::Routine>> {
    cases![]
}

fn erfcf_cases() -> Vec<TestCase<op::erfcf::Routine>> {
    cases![]
}

fn erff_cases() -> Vec<TestCase<op::erff::Routine>> {
    cases![]
}

fn exp_cases() -> Vec<TestCase<op::exp::Routine>> {
    cases![]
}

fn exp10_cases() -> Vec<TestCase<op::exp10::Routine>> {
    cases![]
}

fn exp10f_cases() -> Vec<TestCase<op::exp10f::Routine>> {
    cases![]
}

fn exp2_cases() -> Vec<TestCase<op::exp2::Routine>> {
    cases![]
}

fn exp2f_cases() -> Vec<TestCase<op::exp2f::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn exp2m1f16_cases() -> Vec<TestCase<op::exp2m1f16::Routine>> {
    let mut cases = cases![];

    // Exceptional inputs and branch boundaries from CORE-MATH.
    for bits in [
        0x118c, 0x21bc, 0x8da9, 0x973f, 0x8461, 0x27ff, 0x2800, 0x4bff, 0x4c00,
    ] {
        cases.push(TestCase {
            input: (f16::from_bits(bits),),
            output: None,
        });
    }

    cases
}

fn exp2m1f_cases() -> Vec<TestCase<op::exp2m1f::Routine>> {
    let mut cases = cases![];

    // Boundary values from CORE-MATH's binary32/exp2m1/exp2m1f.c.
    // Test each value and its neighbors with both signs against MPFR.
    for bits in [
        0x0000_0001,
        0x0080_0000,
        0x2538_aa3b,
        0x331f_dd82,
        0x37d3_2ef6,
        0x3a35_8876,
        0x3bac_1405,
        0x3caa_2fee,
        0x3d67_a4cc,
        0x3df9_5f1f,
        0x41c8_0000,
        0x4300_0000,
    ] {
        for adjacent in [bits - 1, bits, bits + 1] {
            for sign in [0, f32::SIGN_MASK] {
                cases.push(TestCase {
                    input: (f32::from_bits(adjacent | sign),),
                    output: None,
                });
            }
        }
    }

    // Special rounding cases from the same CORE-MATH file.
    // Test each value and its neighbors against MPFR.
    for bits in [0xb3d8_5005, 0x3338_428d, 0x388b_ca4f] {
        for adjacent in [bits - 1, bits, bits + 1] {
            cases.push(TestCase {
                input: (f32::from_bits(adjacent),),
                output: None,
            });
        }
    }

    cases
}

fn exp2m1_cases() -> Vec<TestCase<op::exp2m1::Routine>> {
    let mut cases = cases![];

    // Check their neighbors with MPFR.
    for i in 1..=53 {
        let x = i as f64;

        for adjacent in [x.to_bits() - 1, x.to_bits() + 1] {
            for sign in [0, f64::SIGN_MASK] {
                cases.push(TestCase {
                    input: (f64::from_bits(adjacent | sign),),
                    output: None,
                });
            }
        }
    }

    // Subnormal, polynomial, saturation, and overflow boundaries, with both signs.
    for x in [
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        hf64!("0x1.71547652b82fdp-1022"),
        hf64!("0x1p-104"),
        hf64!("0x1.0527dbd87e24dp-51"),
        0.125,
        54.0,
        1024.0,
    ] {
        let bits = x.to_bits();
        for adjacent in [bits - 1, bits, bits + 1] {
            for sign in [0, f64::SIGN_MASK] {
                cases.push(TestCase {
                    input: (f64::from_bits(adjacent | sign),),
                    output: None,
                });
            }
        }
    }

    // Difficult rounding cases from the CORE-MATH exception tables and their neighbors.
    for x in [
        hf64!("-0x1.a16826a8e825dp-56"),
        hf64!("-0x1.f1bc3ef3e6f36p-65"),
        hf64!("-0x1.1e3a6eaa49c6ep-84"),
        hf64!("-0x1.1d65d5fc31246p-98"),
        hf64!("0x1.8a5dd21ef35afp-104"),
        hf64!("0x1.086cbb0e900cfp-96"),
        hf64!("0x1.453f5d718374ap-88"),
        hf64!("0x1.02f371449097fp-51"),
        hf64!("-0x1.f6ec73d3948c3p-4"),
        hf64!("-0x1.3918e8608bd5bp-8"),
        hf64!("-0x1.2c506b0368099p-51"),
        hf64!("0x1.391609b20beaap-51"),
        hf64!("0x1.4753a08baf7fbp-22"),
        hf64!("0x1.d4a8ebce833a3p-4"),
        hf64!("-0x1.da22611253866p+0"),
        hf64!("-0x1.8827da1dbf35ep-2"),
        hf64!("0x1.12eecf76d63cdp+1"),
        hf64!("0x1.ffef3f31766b7p+5"),
        hf64!("0x1.6f33cdaf56d6p+6"),
    ] {
        let bits = x.to_bits();
        for adjacent in [bits - 1, bits, bits + 1] {
            cases.push(TestCase {
                input: (f64::from_bits(adjacent),),
                output: None,
            });
        }
    }

    cases
}

fn expf_cases() -> Vec<TestCase<op::expf::Routine>> {
    cases![
        ((hf32!("-0x1.2d245ap-8")), hf32!("0x1.fda718p-1")),
        ((hf32!("0x1.db1b7ap-7")), hf32!("0x1.03bd22p+0")),
        ((hf32!("-0x1.dc15fcp+5")), hf32!("0x1.1ae6e6p-86")),
    ]
}

fn expm1_cases() -> Vec<TestCase<op::expm1::Routine>> {
    cases![]
}

fn expm1f_cases() -> Vec<TestCase<op::expm1f::Routine>> {
    cases![]
}

fn fabs_cases() -> Vec<TestCase<op::fabs::Routine>> {
    cases![]
}

fn fabsf_cases() -> Vec<TestCase<op::fabsf::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn fabsf128_cases() -> Vec<TestCase<op::fabsf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn fabsf16_cases() -> Vec<TestCase<op::fabsf16::Routine>> {
    cases![]
}

fn fdim_cases() -> Vec<TestCase<op::fdim::Routine>> {
    cases![
        // Failures on i586
        (
            (
                hf64!("0x1.10d2f8a8dffd1p+355"),
                hf64!("-0x1.5203b17e54a8cp+373")
            ),
            hf64!("0x1.5203f5b312d2fp+373")
        ),
        (
            (
                hf64!("0x1.9ffdf64f0d2f8p+294"),
                hf64!("-0x1.71addd21280b5p+344")
            ),
            hf64!("0x1.71addd21280bbp+344")
        ),
        (
            (
                hf64!("0x1.f3600eb4ad0e0p-953"),
                hf64!("-0x1.0c29b2b40023dp-976")
            ),
            hf64!("0x1.f36010cd00737p-953")
        ),
    ]
}

fn fdimf_cases() -> Vec<TestCase<op::fdimf::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn fdimf128_cases() -> Vec<TestCase<op::fdimf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn fdimf16_cases() -> Vec<TestCase<op::fdimf16::Routine>> {
    cases![]
}

fn floor_cases() -> Vec<TestCase<op::floor::Routine>> {
    cases![]
}

fn floorf_cases() -> Vec<TestCase<op::floorf::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn floorf128_cases() -> Vec<TestCase<op::floorf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn floorf16_cases() -> Vec<TestCase<op::floorf16::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn fmaf16_cases() -> Vec<TestCase<op::fmaf16::Routine>> {
    cases![
        // Subnormal result
        ((hf16!("0x1p-11"), hf16!("0x1p-11"), 0.0), None,),
        ((hf16!("0x1p-24"), hf16!("0x1p-24"), 0.0), None,),
        // Failed during extensive tests
        (
            (
                hf16!("-0x1.c4p-12"),
                hf16!("0x1.22p-14"),
                hf16!("-0x1.f4p-15"),
            ),
            hf16!("-0x1.f48p-15")
        ),
        // Examples from https://github.com/llvm/llvm-project/issues/128450
        (
            (
                hf16!("0x1.400p+8"),
                hf16!("0x1.008p+7"),
                hf16!("0x1.000p-24"),
            ),
            hf16!("0x1.40cp+15")
        ),
        (
            (hf16!("0x1.eb8p-12"), hf16!("0x1.9p-11"), hf16!("-0x1p-11"),),
            None
        ),
        // Previous failures during testing
        ((-569.0, -4.89, 65470.0), None),
        ((-998.0, 0.02596, -998.0), None),
        ((6e-8, 6e-8, -6.104e-5), None),
        ((6e-8, 65300.0, 9.5e-7), None),
        ((-569.0, -4.89, -0.0417), None),
        ((6444.0, 0.003443, 0.003443), None),
        ((6e-8, 6e-8, 6e-8), None),
        ((6e-8, -1.0, 6e-8), None),
        ((1.001, 65500.0, 6e-8), None),
        ((1.001, 65500.0, 65500.0), None),
        ((1.002, 65470.0, 0.0), None),
        ((1.002, 65470.0, 65500.0), None),
        ((0.0002216, -8.464e-5, 4.4e-5), None),
        ((-56700.0, -2.082, -61120.0), None),
        ((-475.3, -475.3, 60450.0), None),
        ((-61120.0, -3.969, 20320.0), None),
        ((6e-8, -6e-8, 0.0), None),
    ]
}

fn fmaf_cases() -> Vec<TestCase<op::fmaf::Routine>> {
    cases![
        // Known rounding error for some implementations (notably MinGW)
        ((-1.9369631e13f32, 2.1513551e-7, -1.7354427e-24), -4167095.8),
        // Failure on i586
        (
            (
                hf32!("-0x1.c92494p+109"),
                hf32!("-0x0.000018p-126"),
                hf32!("-0x1.6db6f0p-91"),
            ),
            hf32!("0x1.56db6ep-36")
        ),
    ]
}

fn fma_cases() -> Vec<TestCase<op::fma::Routine>> {
    cases![
        // Previous failure with incorrect sign
        ((5e-324, -5e-324, 0.0), -0.0),
        // Failure on i586
        (
            (0.999999999999999, 1.0000000000000013, 0.0),
            1.0000000000000002
        ),
        // Failure on musl i686/i586
        (
            (
                hf64!("0x0.0000000100001p-1022"),
                hf64!("0x1.ffffffffffffbp+1023"),
                hf64!("0x0p+0")
            ),
            hf64!("0x1.00000fffffffdp-30")
        )
    ]
}

#[cfg(f128_enabled)]
fn fmaf128_cases() -> Vec<TestCase<op::fmaf128::Routine>> {
    cases![
        (
            // Tricky rounding case that previously failed in extensive tests
            (
                hf128!("-0x1.1966cc01966cc01966cc01966f06p-25"),
                hf128!("-0x1.669933fe69933fe69933fe6997c9p-16358"),
                hf128!("-0x0.000000000000000000000000048ap-16382"),
            ),
            hf128!("0x0.c5171470a3ff5e0f68d751491b18p-16382")
        ),
        (
            // Subnormal edge case that caused a failure
            (
                hf128!("0x0.7ffffffffffffffffffffffffff7p-16382"),
                hf128!("0x1.ffffffffffffffffffffffffffffp-1"),
                hf128!("0x0.8000000000000000000000000009p-16382"),
            ),
            hf128!("0x1.0000000000000000000000000000p-16382")
        ),
    ]
}

#[cfg(f16_enabled)]
fn fmaxf16_cases() -> Vec<TestCase<op::fmaxf16::Routine>> {
    cases![]
}

fn fmaxf_cases() -> Vec<TestCase<op::fmaxf::Routine>> {
    cases![]
}

fn fmax_cases() -> Vec<TestCase<op::fmax::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn fmaxf128_cases() -> Vec<TestCase<op::fmaxf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn fmaximumf16_cases() -> Vec<TestCase<op::fmaximumf16::Routine>> {
    cases![]
}

fn fmaximumf_cases() -> Vec<TestCase<op::fmaximumf::Routine>> {
    cases![]
}

fn fmaximum_cases() -> Vec<TestCase<op::fmaximum::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn fmaximumf128_cases() -> Vec<TestCase<op::fmaximumf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn fmaximum_numf16_cases() -> Vec<TestCase<op::fmaximum_numf16::Routine>> {
    cases![]
}

fn fmaximum_numf_cases() -> Vec<TestCase<op::fmaximum_numf::Routine>> {
    cases![]
}

fn fmaximum_num_cases() -> Vec<TestCase<op::fmaximum_num::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn fmaximum_numf128_cases() -> Vec<TestCase<op::fmaximum_numf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn fminf16_cases() -> Vec<TestCase<op::fminf16::Routine>> {
    cases![]
}

fn fminf_cases() -> Vec<TestCase<op::fminf::Routine>> {
    cases![]
}

fn fmin_cases() -> Vec<TestCase<op::fmin::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn fminf128_cases() -> Vec<TestCase<op::fminf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn fminimumf16_cases() -> Vec<TestCase<op::fminimumf16::Routine>> {
    cases![]
}

fn fminimumf_cases() -> Vec<TestCase<op::fminimumf::Routine>> {
    cases![]
}

fn fminimum_cases() -> Vec<TestCase<op::fminimum::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn fminimumf128_cases() -> Vec<TestCase<op::fminimumf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn fminimum_numf16_cases() -> Vec<TestCase<op::fminimum_numf16::Routine>> {
    cases![]
}

fn fminimum_numf_cases() -> Vec<TestCase<op::fminimum_numf::Routine>> {
    cases![]
}

fn fminimum_num_cases() -> Vec<TestCase<op::fminimum_num::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn fminimum_numf128_cases() -> Vec<TestCase<op::fminimum_numf128::Routine>> {
    cases![]
}

fn fmod_cases() -> Vec<TestCase<op::fmod::Routine>> {
    cases![
        // Previous failure with incorrect loop iteration
        // <https://github.com/rust-lang/libm/pull/469#discussion_r2022337272>
        ((2.1, 3.123e-320), 2.0696e-320),
        ((2.1, 2.253547e-318), 1.772535e-318),
    ]
}

fn fmodf_cases() -> Vec<TestCase<op::fmodf::Routine>> {
    cases![
        // Previous failure with incorrect loop iteration
        // <https://github.com/rust-lang/libm/pull/469#discussion_r2022337272>
        ((2.1, 8.858e-42), 8.085e-42),
        ((2.1, 6.39164e-40), 6.1636e-40),
        ((5.5, 6.39164e-40), 4.77036e-40),
        ((-151.189, 6.39164e-40), -5.64734e-40),
    ]
}

#[cfg(f128_enabled)]
fn fmodf128_cases() -> Vec<TestCase<op::fmodf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn fmodf16_cases() -> Vec<TestCase<op::fmodf16::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn frexpf16_cases() -> Vec<TestCase<op::frexpf16::Routine>> {
    cases![]
}

fn frexpf_cases() -> Vec<TestCase<op::frexpf::Routine>> {
    cases![]
}

fn frexp_cases() -> Vec<TestCase<op::frexp::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn frexpf128_cases() -> Vec<TestCase<op::frexpf128::Routine>> {
    cases![]
}

fn hypotf_cases() -> Vec<TestCase<op::hypotf::Routine>> {
    cases![]
}

fn hypot_cases() -> Vec<TestCase<op::hypot::Routine>> {
    cases![
        // Cases that can overflow exponent if wrapping arithmetic is not used
        (
            (
                hf64!("-0x1.800f800f80100p+1023"),
                hf64!("0x1.8354835473720p+996"),
            ),
            None
        ),
        (
            (hf64!("0x1.201b201b201c0p+0"), hf64!("0x1.b028b028b02a0p-1")),
            None
        ),
        (
            (
                hf64!("-0x1.e538e538e564p+980"),
                hf64!("-0x1.c4dfc4dfc508p+983"),
            ),
            None
        ),
        (
            (
                hf64!("-0x1.2f22e4f77aa58p+983"),
                hf64!("-0x1.44c9f5524c8ccp+980"),
            ),
            None
        ),
    ]
}

#[cfg(f16_enabled)]
fn ilogbf16_cases() -> Vec<TestCase<op::ilogbf16::Routine>> {
    cases![]
}

fn ilogbf_cases() -> Vec<TestCase<op::ilogbf::Routine>> {
    cases![]
}

fn ilogb_cases() -> Vec<TestCase<op::ilogb::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn ilogbf128_cases() -> Vec<TestCase<op::ilogbf128::Routine>> {
    cases![]
}

fn j0_cases() -> Vec<TestCase<op::j0::Routine>> {
    cases![]
}

fn j0f_cases() -> Vec<TestCase<op::j0f::Routine>> {
    cases![]
}

fn j1_cases() -> Vec<TestCase<op::j1::Routine>> {
    cases![]
}

fn j1f_cases() -> Vec<TestCase<op::j1f::Routine>> {
    cases![]
}

fn jnf_cases() -> Vec<TestCase<op::jnf::Routine>> {
    cases![]
}

fn jn_cases() -> Vec<TestCase<op::jn::Routine>> {
    cases![
        // Inputs that produce high errors
        ((190, 1005.366268038242), 7.328620335959289e-10),
        ((238, -311.0349), 7.270196433535006e-8),
    ]
}

fn ldexp_cases() -> Vec<TestCase<op::ldexp::Routine>> {
    cases![]
}

fn ldexpf_cases() -> Vec<TestCase<op::ldexpf::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn ldexpf128_cases() -> Vec<TestCase<op::ldexpf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn ldexpf16_cases() -> Vec<TestCase<op::ldexpf16::Routine>> {
    cases![]
}

fn lgamma_cases() -> Vec<TestCase<op::lgamma::Routine>> {
    cases![]
}

fn lgammaf_cases() -> Vec<TestCase<op::lgammaf::Routine>> {
    cases![
        // High error
        ((-4.933393,), -1.9580022),
    ]
}

fn lgamma_r_cases() -> Vec<TestCase<op::lgamma_r::Routine>> {
    cases![]
}

fn lgammaf_r_cases() -> Vec<TestCase<op::lgammaf_r::Routine>> {
    cases![]
}

fn log_cases() -> Vec<TestCase<op::log::Routine>> {
    cases![]
}

fn log10_cases() -> Vec<TestCase<op::log10::Routine>> {
    cases![]
}

fn log10f_cases() -> Vec<TestCase<op::log10f::Routine>> {
    cases![]
}

fn log1pf_cases() -> Vec<TestCase<op::log1pf::Routine>> {
    cases![
        // Musl failures on i586
        ((hf32!("-0x1.8292f6p-2")), hf32!("-0x1.e56918p-2")),
        ((hf32!("0x1.12d15ep-1")), hf32!("0x1.b7fbf8p-2")),
        ((hf32!("-0x1.904ebep-2")), hf32!("-0x1.fbb6cap-2")),
    ]
}

fn log1p_cases() -> Vec<TestCase<op::log1p::Routine>> {
    cases![
        // Musl failure on i586
        (
            (hf64!("-0x1.9094dbf7f2e85p-2"),),
            hf64!("-0x1.fc29f046c88a1p-2")
        ),
    ]
}

fn log2_cases() -> Vec<TestCase<op::log2::Routine>> {
    cases![]
}

fn log2f_cases() -> Vec<TestCase<op::log2f::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn log2p1f16_cases() -> Vec<TestCase<op::log2p1f16::Routine>> {
    let mut cases = cases![];

    // Exceptional inputs and branch boundaries from CORE-MATH.
    for bits in [
        0x8cf6, 0x9505, 0xa5cf, 0x8309, 0x04b3, 0x0b46, 0x25ff, 0x0fb1, 0x1ffb, 0x2800, 0xa800,
        0xbc00,
    ] {
        for adjacent in [bits - 1, bits, bits + 1] {
            cases.push(TestCase {
                input: (f16::from_bits(adjacent),),
                output: None,
            });
        }
    }

    cases
}

fn log2p1f_cases() -> Vec<TestCase<op::log2p1f::Routine>> {
    let mut cases = cases![];

    // Exceptional inputs from CORE-MATH and their neighbors.
    for bits in [
        0x32ff_7045,
        0xb395_efbb,
        0x35a1_4df7,
        0x3841_cb81,
        0xbac9_363d,
        0x5292_8e33,
        0x4ebd_09e3,
        0xbd6d_142e,
    ] {
        for adjacent in [bits - 1, bits, bits + 1] {
            cases.push(TestCase {
                input: (f32::from_bits(adjacent),),
                output: None,
            });
        }
    }

    // Subnormal, polynomial, and domain boundaries, with both signs.
    for bits in [
        0x0000_0001,
        0x0058_b90b,
        0x0080_0000,
        0x329c_5639,
        0x3963_8a7e,
        0x3b9d_9d34,
        0x3cb7_aa26,
        0x3cc0_0000,
        0x3f80_0000,
        0x7f7f_ffff,
    ] {
        for adjacent in [bits - 1, bits, bits + 1] {
            for sign in [0, f32::SIGN_MASK] {
                cases.push(TestCase {
                    input: (f32::from_bits(adjacent | sign),),
                    output: None,
                });
            }
        }
    }

    // Check the neighbors of exact results with MPFR.
    for n in 1..=24 {
        for x in [libm::ldexpf(1.0, n) - 1.0, libm::ldexpf(1.0, -n) - 1.0] {
            for bits in [x.to_bits() - 1, x.to_bits() + 1] {
                cases.push(TestCase {
                    input: (f32::from_bits(bits),),
                    output: None,
                });
            }
        }
    }

    cases
}

fn log2p1_cases() -> Vec<TestCase<op::log2p1::Routine>> {
    let mut cases = cases![];

    // Exceptional inputs from CORE-MATH.
    for x in [
        hf64!("-0x1.f5baee010ccc6p-6"),
        hf64!("-0x1.98e2aeed83e64p-7"),
        hf64!("-0x1.7ff01253739d1p-7"),
        hf64!("-0x1.ddabc67cec3c1p-9"),
        hf64!("-0x1.921c00a04d8a9p-9"),
        hf64!("-0x1.ae85103cce3e5p-10"),
        hf64!("-0x1.5955d05fd828cp-10"),
        hf64!("-0x1.3623eff91de91p-10"),
        hf64!("-0x1.73ccf8ee62819p-12"),
        hf64!("-0x1.fa5579e9a6941p-16"),
        hf64!("-0x1.75e4f1fb4d0f9p-17"),
        hf64!("-0x1.b53dee1a96ca4p-18"),
        hf64!("-0x1.c66cb81b37ebbp-19"),
        hf64!("-0x1.a1b96d88621b3p-19"),
        hf64!("-0x1.bdb50ccf06b7ap-20"),
        hf64!("-0x1.77d66368613b4p-21"),
        hf64!("-0x1.752326c780f68p-21"),
        hf64!("-0x1.40e3f4e392fc8p-24"),
        hf64!("-0x1.16bba98a001c1p-24"),
        hf64!("-0x1.51fb235627c5cp-28"),
        hf64!("-0x1.9b1805f30b4edp-31"),
        hf64!("-0x1.37a6f3079625dp-31"),
        hf64!("-0x1.672dad63d9469p-32"),
        hf64!("-0x1.025b55debc13p-32"),
        hf64!("-0x1.3278a26ed0162p-34"),
        hf64!("-0x1.d88aedf0484a7p-36"),
        hf64!("-0x1.6d4141037f32dp-37"),
        hf64!("-0x1.40739786837bp-40"),
        hf64!("-0x1.ba43bd8edc2a3p-42"),
        hf64!("-0x1.5892a86231e56p-42"),
        hf64!("-0x1.21b2c54479c4dp-42"),
        hf64!("-0x1.526d1450f8572p-43"),
        hf64!("-0x1.bab64e105226ap-45"),
        hf64!("-0x1.e9c2de7f3aa23p-47"),
        hf64!("-0x1.eaba84e0bd618p-49"),
        hf64!("-0x1.2516dafdf17adp-50"),
        hf64!("-0x1.9914a112c8dadp-51"),
        hf64!("-0x1.a052e3d5e791p-52"),
        hf64!("-0x1.92ba5fbf0a946p-54"),
        hf64!("-0x1.6f332a62b59f9p-56"),
        hf64!("-0x1.21530a306cc85p-56"),
        hf64!("-0x1.292f4e5d19917p-57"),
        hf64!("-0x1.12b592f889516p-59"),
        hf64!("-0x1.7780d5e5cf5c5p-60"),
        hf64!("-0x1.32ed98e196cf5p-60"),
        hf64!("-0x1.d73ad29747834p-61"),
        hf64!("-0x1.354976dca453cp-61"),
        hf64!("-0x1.1483eb37c9829p-61"),
        hf64!("-0x1.fa53cebe640ccp-62"),
        hf64!("-0x1.6923c31228cd3p-62"),
        hf64!("-0x1.796ad95f38488p-63"),
        hf64!("-0x1.470187381f305p-63"),
        hf64!("-0x1.9234c78cc23acp-64"),
        hf64!("-0x1.c7c8b78d5a272p-65"),
        hf64!("-0x1.a595e9f7d6e25p-65"),
        hf64!("-0x1.47aea7608c02bp-65"),
        hf64!("-0x1.1b19925f9fc06p-65"),
        hf64!("-0x1.d62c98023cdbap-66"),
        hf64!("-0x1.3b65311720a11p-66"),
        hf64!("-0x1.026e1ee1c55aep-68"),
        hf64!("-0x1.cf96e02037643p-70"),
        hf64!("-0x1.b9ab928f8614ap-70"),
        hf64!("-0x1.297ee84082dafp-70"),
        hf64!("-0x1.f63a8ce2364f3p-71"),
        hf64!("-0x1.ed7ff50423e4dp-71"),
        hf64!("-0x1.6856506d8234ap-71"),
        hf64!("-0x1.62a8f9300de6fp-71"),
        hf64!("-0x1.de8c6ef8a4dcep-73"),
        hf64!("-0x1.698f4eb77100ap-73"),
        hf64!("-0x1.5536e12eb7335p-74"),
        hf64!("-0x1.2b19ae19e4f3ep-74"),
        hf64!("-0x1.7711d03d5c7bdp-75"),
        hf64!("-0x1.6104ba44449ep-75"),
        hf64!("-0x1.15108908835fp-75"),
        hf64!("-0x1.1288bfd6f822ap-76"),
        hf64!("-0x1.dbf55aa9a7eb9p-77"),
        hf64!("-0x1.a175bf47b2093p-79"),
        hf64!("-0x1.75817dcf08a91p-81"),
        hf64!("-0x1.6340571968b67p-82"),
        hf64!("-0x1.06501e96466cfp-83"),
        hf64!("-0x1.86e6a6125ee5fp-85"),
        hf64!("-0x1.ce9392feefd4dp-86"),
        hf64!("-0x1.60fa7c8eee569p-86"),
        hf64!("-0x1.30876af870c12p-86"),
        hf64!("-0x1.1cf977003bdd6p-86"),
        hf64!("-0x1.2c23f2bc02277p-87"),
        hf64!("-0x1.05ae004891094p-87"),
        hf64!("-0x1.b86b45d2c7c74p-88"),
        hf64!("-0x1.63c4a608aac21p-89"),
        hf64!("-0x1.ffc87ce076dfap-91"),
        hf64!("-0x1.127630515eb89p-91"),
        hf64!("-0x1.db7daf1e016dfp-92"),
        hf64!("-0x1.ca7e01c959788p-92"),
        hf64!("-0x1.4106468e7b671p-94"),
        hf64!("-0x1.cd79e208d957ep-95"),
        hf64!("-0x1.0c5a1aeb10a7cp-95"),
        hf64!("-0x1.de8de3ca3cb0fp-97"),
        hf64!("-0x1.8ba5360a2b553p-99"),
        hf64!("-0x1.618b50a22cecp-105"),
        hf64!("-0x1.09287c79a1b1p-105"),
        hf64!("-0x1.618b50a22cecp-106"),
        hf64!("-0x1.09287c79a1b1p-106"),
        hf64!("0x1.198cc57cb19cfp-106"),
        hf64!("0x1.71ef99a53cd7fp-106"),
        hf64!("0x1.198cc57cb19cfp-105"),
        hf64!("0x1.71ef99a53cd7fp-105"),
        hf64!("0x1.198cc57cb19cfp-104"),
        hf64!("0x1.71ef99a53cd7fp-104"),
        hf64!("0x1.198cc57cb19cfp-103"),
        hf64!("0x1.8253e2a84cc3ep-103"),
        hf64!("0x1.dab6b6d0d7feep-103"),
        hf64!("0x1.29f10e7fc188ep-102"),
        hf64!("0x1.92b82bab5cafdp-102"),
        hf64!("0x1.4ab9a085e160cp-101"),
        hf64!("0x1.d4494fb79c5f9p-101"),
        hf64!("0x1.e4ad98baac4b8p-101"),
        hf64!("0x1.f511e1bdbc377p-101"),
        hf64!("0x1.ebc08e5bdda62p-99"),
        hf64!("0x1.6c27c113f6f3fp-98"),
        hf64!("0x1.af040230221a5p-97"),
        hf64!("0x1.e743d2da8338cp-97"),
        hf64!("0x1.34e55b50b5476p-95"),
        hf64!("0x1.4265b80b22415p-95"),
        hf64!("0x1.4fe614c58f3b4p-95"),
        hf64!("0x1.7bf0a5c56ed66p-95"),
        hf64!("0x1.d896d7b343ddap-94"),
        hf64!("0x1.4b7dadb0f0318p-93"),
        hf64!("0x1.6201aba46fb8p-93"),
        hf64!("0x1.05221bfa8a0fap-92"),
        hf64!("0x1.3d5b61f8af06p-91"),
        hf64!("0x1.572552ca58e6ep-91"),
        hf64!("0x1.3a41b77e1cfcap-90"),
        hf64!("0x1.94d5d40edd1e3p-90"),
        hf64!("0x1.02f8c70e9f28p-89"),
        hf64!("0x1.61de0b2528516p-89"),
        hf64!("0x1.64cc21a428f53p-89"),
        hf64!("0x1.31cdce0e38005p-88"),
        hf64!("0x1.5b9ab5b571503p-88"),
        hf64!("0x1.7654fc15c252dp-88"),
        hf64!("0x1.abf231afdb1f5p-88"),
        hf64!("0x1.8586b6e5cbb06p-87"),
        hf64!("0x1.ae2b54125b144p-87"),
        hf64!("0x1.1d640ce51c6b5p-86"),
        hf64!("0x1.3a0e97e84bb1ep-86"),
        hf64!("0x1.742efa2dfc234p-86"),
        hf64!("0x1.63eb00801d3d6p-85"),
        hf64!("0x1.a8b9097e3a9aep-85"),
        hf64!("0x1.807d37ffa6069p-83"),
        hf64!("0x1.3528bc4882f1fp-82"),
        hf64!("0x1.982a8e52379abp-82"),
        hf64!("0x1.317c7aec05f2cp-81"),
        hf64!("0x1.4376f83fcc3c7p-81"),
        hf64!("0x1.569aad73361dap-80"),
        hf64!("0x1.575476e5e84cp-80"),
        hf64!("0x1.f20aa391bfeabp-80"),
        hf64!("0x1.181b21bfde59bp-79"),
        hf64!("0x1.0934d6a88672dp-78"),
        hf64!("0x1.0f9c8867c5241p-78"),
        hf64!("0x1.4c5324339bb48p-78"),
        hf64!("0x1.f7d8b04b61989p-78"),
        hf64!("0x1.8f27d6bf246acp-75"),
        hf64!("0x1.f83edae3aa93fp-75"),
        hf64!("0x1.cda9fe2248ec6p-74"),
        hf64!("0x1.c0ee5d2d2d30ap-73"),
        hf64!("0x1.628c0b5b4b9bbp-72"),
        hf64!("0x1.f018c61839599p-72"),
        hf64!("0x1.87a0cd7eeca98p-71"),
        hf64!("0x1.5c5f1081b8abdp-70"),
        hf64!("0x1.77e590243e48fp-69"),
        hf64!("0x1.c943ea0da88dfp-69"),
        hf64!("0x1.d33e23beb1528p-68"),
        hf64!("0x1.186615833f8f7p-67"),
        hf64!("0x1.4891077ffbab6p-66"),
        hf64!("0x1.4193614150725p-64"),
        hf64!("0x1.572569359da9p-64"),
        hf64!("0x1.6a5f8252d9f3bp-64"),
        hf64!("0x1.7b5eedadbdbd6p-64"),
        hf64!("0x1.b2669d6e81f95p-64"),
        hf64!("0x1.02754e0d621p-63"),
        hf64!("0x1.42c2504353e38p-63"),
        hf64!("0x1.9a8cb37e9acd5p-63"),
        hf64!("0x1.bd1c7919023cp-63"),
        hf64!("0x1.e505bd69056d9p-63"),
        hf64!("0x1.2dbbb07d596fep-62"),
        hf64!("0x1.9a2e439736a07p-62"),
        hf64!("0x1.112c460eafdcp-61"),
        hf64!("0x1.7dcdac46803eep-61"),
        hf64!("0x1.4b2ed4f196fe5p-60"),
        hf64!("0x1.bc6526d12f2a9p-60"),
        hf64!("0x1.95242afed5822p-59"),
        hf64!("0x1.158d3212d17dfp-58"),
        hf64!("0x1.f2bb88afdad28p-58"),
        hf64!("0x1.057d5711bfd9ep-57"),
        hf64!("0x1.18cebea44da63p-57"),
        hf64!("0x1.4cd45d85f97bp-57"),
        hf64!("0x1.596a35cd55af7p-57"),
        hf64!("0x1.9b7e25fae77ffp-56"),
        hf64!("0x1.67c00f810444cp-55"),
        hf64!("0x1.e62ce3c0305e7p-54"),
        hf64!("0x1.b2078ba8f6835p-52"),
        hf64!("0x1.cbe169f09f95ep-52"),
        hf64!("0x1.9ce011cf5f46ep-51"),
        hf64!("0x1.9b37cdd819b7cp-48"),
        hf64!("0x1.ed87135915039p-48"),
        hf64!("0x1.389bb3cfc42c2p-47"),
        hf64!("0x1.56cd7d201444cp-46"),
        hf64!("0x1.fc53c8d303c2p-46"),
        hf64!("0x1.396ae5fe7edc6p-44"),
        hf64!("0x1.57c17dadff53fp-44"),
        hf64!("0x1.8e05d9bf54ff5p-43"),
        hf64!("0x1.7a55eea45459dp-39"),
        hf64!("0x1.d204a5710c6b4p-38"),
        hf64!("0x1.0477b177d47a9p-37"),
        hf64!("0x1.912cc5483c4bbp-37"),
        hf64!("0x1.32ba6bfc6df61p-36"),
        hf64!("0x1.b881f93d3d7bcp-35"),
        hf64!("0x1.33c28a8adb35dp-33"),
        hf64!("0x1.1bab50d82fdf7p-30"),
        hf64!("0x1.5b8a987b105b9p-29"),
        hf64!("0x1.a7915a8ef7252p-29"),
        hf64!("0x1.51cba0164bed2p-27"),
        hf64!("0x1.1859f2f12c686p-26"),
        hf64!("0x1.5004cdd3ec269p-26"),
        hf64!("0x1.970634f93920ap-25"),
        hf64!("0x1.446805a9fe779p-23"),
        hf64!("0x1.4f86281412f0bp-21"),
        hf64!("0x1.53e1ee4cd607ep-21"),
        hf64!("0x1.91848ab4215a3p-20"),
        hf64!("0x1.7dbc650f5516cp-19"),
        hf64!("0x1.5ed15c24566e4p-16"),
        hf64!("0x1.66e3866f48983p-14"),
        hf64!("0x1.4a82829ba33efp-13"),
        hf64!("0x1.ddacfe52ca881p-13"),
        hf64!("0x1.cee52926a8cdep-11"),
        hf64!("0x1.50878a3127c53p-10"),
        hf64!("0x1.c60c6b10503bdp-10"),
        hf64!("0x1.cb331784430e1p-10"),
        hf64!("0x1.d19f229d1451dp-10"),
        hf64!("0x1.15862abd340e8p-9"),
        hf64!("0x1.6d42e64d940acp-9"),
        hf64!("0x1.df670b0e00641p-9"),
        hf64!("0x1.fe604d486351fp-9"),
        hf64!("0x1.0b896d69a6a9ap-7"),
        hf64!("0x1.c91984e0cebe7p-7"),
        hf64!("0x1.cdfe4fcde7278p-7"),
        hf64!("0x1.fa7ca1e176885p-7"),
        hf64!("0x1.2cbfdc5da28d9p-6"),
    ] {
        for bits in [x.to_bits() - 1, x.to_bits(), x.to_bits() + 1] {
            cases.push(TestCase {
                input: (f64::from_bits(bits),),
                output: None,
            });
        }
    }

    // Branch boundaries from CORE-MATH
    for x in [
        hf64!("0x1p-920"),
        hf64!("0x1p-255"),
        hf64!("0x1p-105"),
        hf64!("0x1p-5"),
        hf64!("0x1p1022"),
        hf64!("0x1.6a09e667f3bcdp0"),
        hf64!("0x0.2c316a14459d8p-1022"),
    ] {
        for bits in [x.to_bits() - 1, x.to_bits(), x.to_bits() + 1] {
            for sign in [0, f64::SIGN_MASK] {
                cases.push(TestCase {
                    input: (f64::from_bits(bits | sign),),
                    output: None,
                });
            }
        }
    }

    // Check the neighbors of exact results with MPFR.
    for n in 1..=53 {
        let power = libm::ldexp(1.0, n);
        for x in [power - 1.0, libm::ldexp(1.0, -n) - 1.0] {
            for bits in [x.to_bits() - 1, x.to_bits() + 1] {
                cases.push(TestCase {
                    input: (f64::from_bits(bits),),
                    output: None,
                });
            }
        }
    }

    cases
}

fn logf_cases() -> Vec<TestCase<op::logf::Routine>> {
    cases![]
}

fn modf_cases() -> Vec<TestCase<op::modf::Routine>> {
    cases![]
}

fn modff_cases() -> Vec<TestCase<op::modff::Routine>> {
    cases![]
}

fn nextafter_cases() -> Vec<TestCase<op::nextafter::Routine>> {
    cases![]
}

fn nextafterf_cases() -> Vec<TestCase<op::nextafterf::Routine>> {
    cases![]
}

fn pow_cases() -> Vec<TestCase<op::pow::Routine>> {
    cases![]
}

fn powf_cases() -> Vec<TestCase<op::powf::Routine>> {
    cases![]
}

fn remainder_cases() -> Vec<TestCase<op::remainder::Routine>> {
    cases![]
}

fn remainderf_cases() -> Vec<TestCase<op::remainderf::Routine>> {
    cases![]
}

fn remquo_cases() -> Vec<TestCase<op::remquo::Routine>> {
    cases![]
}

fn remquof_cases() -> Vec<TestCase<op::remquof::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn rintf16_cases() -> Vec<TestCase<op::rintf16::Routine>> {
    // Out rint doesn't respect rounding modes so it is the same as roundeven
    roundevenf16_cases()
        .into_iter()
        .map(TestCase::cast)
        .collect()
}

fn rintf_cases() -> Vec<TestCase<op::rintf::Routine>> {
    // Out rint doesn't respect rounding modes so it is the same as roundeven
    roundevenf_cases().into_iter().map(TestCase::cast).collect()
}

fn rint_cases() -> Vec<TestCase<op::rint::Routine>> {
    // Out rint doesn't respect rounding modes so it is the same as roundeven
    roundeven_cases().into_iter().map(TestCase::cast).collect()
}

#[cfg(f128_enabled)]
fn rintf128_cases() -> Vec<TestCase<op::rintf128::Routine>> {
    // Out rint doesn't respect rounding modes so it is the same as roundeven
    roundevenf128_cases()
        .into_iter()
        .map(TestCase::cast)
        .collect()
}

#[cfg(f16_enabled)]
fn roundf16_cases() -> Vec<TestCase<op::roundf16::Routine>> {
    cases![]
}

fn round_cases() -> Vec<TestCase<op::round::Routine>> {
    cases![
        // Failure on i586
        (
            (hf64!("0x1.9efc6a203d4a9p+52"),),
            hf64!("0x1.9efc6a203d4a9p+52")
        )
    ]
}

fn roundf_cases() -> Vec<TestCase<op::roundf::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn roundf128_cases() -> Vec<TestCase<op::roundf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn roundevenf16_cases() -> Vec<TestCase<op::roundevenf16::Routine>> {
    cases![]
}

fn roundevenf_cases() -> Vec<TestCase<op::roundevenf::Routine>> {
    cases![]
}

fn roundeven_cases() -> Vec<TestCase<op::roundeven::Routine>> {
    cases![
        // Failure on i586
        ((-519629176421.49976,), -519629176421.0),
        // Failures with a previous algorithm
        ((-849751480.5001163,), -849751481.0),
        ((-12493089.499809155,), -12493089.0),
        ((-1308.5000830345912,), -1309.0),
    ]
}

#[cfg(f128_enabled)]
fn roundevenf128_cases() -> Vec<TestCase<op::roundevenf128::Routine>> {
    cases![]
}

fn scalbn_cases() -> Vec<TestCase<op::scalbn::Routine>> {
    cases![]
}

fn scalbnf_cases() -> Vec<TestCase<op::scalbnf::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn scalbnf128_cases() -> Vec<TestCase<op::scalbnf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn scalbnf16_cases() -> Vec<TestCase<op::scalbnf16::Routine>> {
    cases![]
}

fn sin_cases() -> Vec<TestCase<op::sin::Routine>> {
    cases![]
}

fn sincos_cases() -> Vec<TestCase<op::sincos::Routine>> {
    cases![]
}

fn sincosf_cases() -> Vec<TestCase<op::sincosf::Routine>> {
    cases![]
}

fn sinf_cases() -> Vec<TestCase<op::sinf::Routine>> {
    cases![]
}

fn sinh_cases() -> Vec<TestCase<op::sinh::Routine>> {
    cases![]
}

fn sinhf_cases() -> Vec<TestCase<op::sinhf::Routine>> {
    cases![]
}

fn sqrt_cases() -> Vec<TestCase<op::sqrt::Routine>> {
    cases![]
}

fn sqrtf_cases() -> Vec<TestCase<op::sqrtf::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn sqrtf128_cases() -> Vec<TestCase<op::sqrtf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn sqrtf16_cases() -> Vec<TestCase<op::sqrtf16::Routine>> {
    cases![]
}

fn tanf_cases() -> Vec<TestCase<op::tanf::Routine>> {
    cases![]
}

fn tan_cases() -> Vec<TestCase<op::tan::Routine>> {
    cases![
        // Musl failures on i586
        (
            (hf64!("0x1.fffffffffffafp+1023"),),
            hf64!("0x1.c573c6dd8c00ap+0")
        ),
        (
            (hf64!("0x1.fffffffffffafp+1023"),),
            hf64!("0x1.c573c6dd8c00ap+0")
        ),
        (
            (hf64!("-0x1.0b10f6eaf2ca0p+883"),),
            hf64!("0x1.cefbd167e2402p+0")
        ),
    ]
}

fn tanh_cases() -> Vec<TestCase<op::tanh::Routine>> {
    cases![(
        (hf64!("0x1.fbfdb8b31b9b4p-3"),),
        hf64!("0x1.f1d2bcb4e1b45p-3")
    )]
}

fn tanhf_cases() -> Vec<TestCase<op::tanhf::Routine>> {
    cases![
        // Inaccuracy in musl
        ((0.24503659,), 0.24024734),
        ((0.19125812,), 0.18895969),
    ]
}

fn tgamma_cases() -> Vec<TestCase<op::tgamma::Routine>> {
    cases![]
}

fn tgammaf_cases() -> Vec<TestCase<op::tgammaf::Routine>> {
    cases![]
}

fn trunc_cases() -> Vec<TestCase<op::trunc::Routine>> {
    cases![]
}

fn truncf_cases() -> Vec<TestCase<op::truncf::Routine>> {
    cases![]
}

#[cfg(f128_enabled)]
fn truncf128_cases() -> Vec<TestCase<op::truncf128::Routine>> {
    cases![]
}

#[cfg(f16_enabled)]
fn truncf16_cases() -> Vec<TestCase<op::truncf16::Routine>> {
    cases![]
}

fn y0_cases() -> Vec<TestCase<op::y0::Routine>> {
    cases![]
}

fn y0f_cases() -> Vec<TestCase<op::y0f::Routine>> {
    cases![]
}

fn y1_cases() -> Vec<TestCase<op::y1::Routine>> {
    cases![]
}

fn y1f_cases() -> Vec<TestCase<op::y1f::Routine>> {
    cases![]
}

fn ynf_cases() -> Vec<TestCase<op::ynf::Routine>> {
    cases![]
}

fn yn_cases() -> Vec<TestCase<op::yn::Routine>> {
    cases![
        // Inputs that should be finite but tend to round to infinity
        ((228, 120.75621), -3.3293829e38),
        ((148, 61.379253), -3.2585946e38),
        ((184, 87.26689), -3.2943882e38),
    ]
}

pub trait CaseListInput: MathOp + Sized {
    fn get_cases() -> Vec<TestCase<Self>>;
}

macro_rules! impl_case_list {
    (
        fn_name: $fn_name:ident,
        attrs: [$($attr:meta),*],
    ) => {
        paste::paste! {
            $(#[$attr])*
            impl CaseListInput for crate::op::$fn_name::Routine {
                fn get_cases() -> Vec<TestCase<Self>> {
                    [< $fn_name _cases >]()
                }
            }
        }
    };
}

libm_macros::for_each_function! {
    callback: impl_case_list,
}

/// This is the test generator for standalone tests, i.e. those with no basis. For this, it
/// only extracts tests with a known output.
pub fn get_test_cases_standalone<Op>(
    ctx: &CheckCtx,
) -> impl Iterator<Item = (Op::RustArgs, Op::RustRet)> + use<'_, Op>
where
    Op: MathOp + CaseListInput,
{
    assert_eq!(ctx.basis, CheckBasis::None);
    assert_eq!(ctx.gen_kind, GeneratorKind::List);
    Op::get_cases()
        .into_iter()
        .filter_map(|x| x.output.map(|o| (x.input, o)))
}

/// Opposite of the above; extract only test cases that don't have a known output, to be run
/// against a basis.
pub fn get_test_cases_basis<Op>(
    ctx: &CheckCtx,
) -> (impl Iterator<Item = Op::RustArgs> + use<'_, Op>, u64)
where
    Op: MathOp + CaseListInput,
{
    assert_ne!(ctx.basis, CheckBasis::None);
    assert_eq!(ctx.gen_kind, GeneratorKind::List);

    let cases = Op::get_cases();
    let count: u64 = cases
        .iter()
        .filter(|case| case.output.is_none())
        .count()
        .try_into()
        .unwrap();

    (
        cases
            .into_iter()
            .filter(|x| x.output.is_none())
            .map(|x| x.input),
        count,
    )
}
