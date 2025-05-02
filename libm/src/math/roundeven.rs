use super::support::{Float, Round};

/// Round `x` to the nearest integer, breaking ties toward even. This is IEEE 754
/// `roundToIntegralTiesToEven`.
#[cfg(f16_enabled)]
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub fn roundevenf16(x: f16) -> f16 {
    select_implementation! {
        name: roundevenf16,
        use_arch: all(target_arch = "aarch64", target_feature = "fp16"),
        args: x,
    }

    roundeven_impl(x)
}

/// Round `x` to the nearest integer, breaking ties toward even. This is IEEE 754
/// `roundToIntegralTiesToEven`.
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub fn roundevenf(x: f32) -> f32 {
    select_implementation! {
        name: roundevenf,
        use_arch: all(target_arch = "aarch64", target_feature = "fp16"),
        args: x,
    }

    roundeven_impl(x)
}

/// Round `x` to the nearest integer, breaking ties toward even. This is IEEE 754
/// `roundToIntegralTiesToEven`.
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub fn roundeven(x: f64) -> f64 {
    select_implementation! {
        name: roundeven,
        use_arch: all(target_arch = "aarch64", target_feature = "fp16"),
        args: x,
    }

    roundeven_impl(x)
}

/// Round `x` to the nearest integer, breaking ties toward even. This is IEEE 754
/// `roundToIntegralTiesToEven`.
#[cfg(f128_enabled)]
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub fn roundevenf128(x: f128) -> f128 {
    roundeven_impl(x)
}

#[inline]
pub fn roundeven_impl<F: Float>(x: F) -> F {
    super::generic::rint_round(x, Round::Nearest).val
}
