/// Round `x` to the nearest integer, breaking ties away from zero.
#[cfg(f16_enabled)]
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub fn roundf16(x: f16) -> f16 {
    select_implementation! {
        name: roundf16,
        use_arch: all(target_arch = "aarch64", target_feature = "fp16"),
        args: x,
    }

    super::generic::round(x)
}

/// Round `x` to the nearest integer, breaking ties away from zero.
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub fn roundf(x: f32) -> f32 {
    select_implementation! {
        name: roundf,
        use_arch: all(target_arch = "aarch64", target_feature = "neon"),
        args: x,
    }

    super::generic::round(x)
}

/// Round `x` to the nearest integer, breaking ties away from zero.
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub fn round(x: f64) -> f64 {
    select_implementation! {
        name: round,
        use_arch: all(target_arch = "aarch64", target_feature = "neon"),
        args: x,
    }

    super::generic::round(x)
}

/// Round `x` to the nearest integer, breaking ties away from zero.
#[cfg(f128_enabled)]
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub fn roundf128(x: f128) -> f128 {
    super::generic::round(x)
}
