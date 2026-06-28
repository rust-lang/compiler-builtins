/// Positive difference (f16)
///
/// Determines the positive difference between arguments, returning:
/// * x - y if x > y, or
/// * +0 if x <= y, or
/// * NAN if either argument is NAN.
///
/// A range error may occur.
#[cfg(f16_enabled)]
#[cfg_attr(assert_no_panic, no_panic::no_panic)]
pub fn fdimf16(x: f16, y: f16) -> f16 {
    super::generic::fdim(x, y)
}

/// Positive difference (f32)
///
/// Determines the positive difference between arguments, returning:
/// * x - y if x > y, or
/// * +0 if x <= y, or
/// * NAN if either argument is NAN.
///
/// A range error may occur.
#[cfg_attr(assert_no_panic, no_panic::no_panic)]
pub fn fdimf(x: f32, y: f32) -> f32 {
    super::generic::fdim(x, y)
}

/// Positive difference (f64)
///
/// Determines the positive difference between arguments, returning:
/// * x - y if x > y, or
/// * +0 if x <= y, or
/// * NAN if either argument is NAN.
///
/// A range error may occur.
#[cfg_attr(assert_no_panic, no_panic::no_panic)]
pub fn fdim(x: f64, y: f64) -> f64 {
    super::generic::fdim(x, y)
}

/// Positive difference (f128)
///
/// Determines the positive difference between arguments, returning:
/// * x - y if x > y, or
/// * +0 if x <= y, or
/// * NAN if either argument is NAN.
///
/// A range error may occur.
#[cfg(f128_enabled)]
#[cfg_attr(assert_no_panic, no_panic::no_panic)]
pub fn fdimf128(x: f128, y: f128) -> f128 {
    super::generic::fdim(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::Float;

    fn spec_test<F: Float>(f: impl Fn(F, F) -> F) {
        assert_biteq!(f(F::ONE, F::ZERO), F::ONE);
        assert_biteq!(f(F::ZERO, F::NEG_ONE), F::ONE);
        assert_biteq!(f(F::INFINITY, F::ONE), F::INFINITY);
        assert_biteq!(f(F::ONE, F::NEG_INFINITY), F::INFINITY);
        assert_biteq!(f(F::ZERO, F::ONE), F::ZERO);
        assert_biteq!(f(F::NEG_ONE, F::ZERO), F::ZERO);
        assert_biteq!(f(F::NEG_INFINITY, F::ZERO), F::ZERO);
        assert_biteq!(f(F::ZERO, F::ZERO), F::ZERO);
        assert_biteq!(f(F::NEG_ZERO, F::ZERO), F::ZERO);
        assert_biteq!(f(F::ZERO, F::NEG_ZERO), F::ZERO);
        assert_biteq!(f(F::ONE, F::ONE), F::ZERO);
        assert_biteq!(f(F::INFINITY, F::INFINITY), F::ZERO);
        assert_biteq!(f(F::NEG_INFINITY, F::NEG_INFINITY), F::ZERO);
        assert!(f(F::NAN, F::ONE).is_nan());
        assert!(f(F::ONE, F::NAN).is_nan());
        assert!(f(F::NAN, F::NAN).is_nan());
    }

    #[test]
    #[cfg(f16_enabled)]
    fn sanity_check_f16() {
        assert_eq!(fdimf16(5.0f16, 3.0f16), 2.0f16);
        assert_eq!(fdimf16(3.0f16, 5.0f16), 0.0f16);
    }

    #[test]
    #[cfg(f16_enabled)]
    fn spec_tests_f16() {
        spec_test::<f16>(fdimf16);
    }

    #[test]
    fn sanity_check_f32() {
        assert_eq!(fdimf(5.0f32, 3.0f32), 2.0f32);
        assert_eq!(fdimf(3.0f32, 5.0f32), 0.0f32);
    }

    #[test]
    fn spec_tests_f32() {
        spec_test::<f32>(fdimf);
    }

    #[test]
    fn sanity_check_f64() {
        assert_eq!(fdim(5.0f64, 3.0f64), 2.0f64);
        assert_eq!(fdim(3.0f64, 5.0f64), 0.0f64);
    }

    #[test]
    fn spec_tests_f64() {
        spec_test::<f64>(fdim);
    }

    #[test]
    #[cfg(f128_enabled)]
    fn sanity_check_f128() {
        assert_eq!(fdimf128(5.0f128, 3.0f128), 2.0f128);
        assert_eq!(fdimf128(3.0f128, 5.0f128), 0.0f128);
    }

    #[test]
    #[cfg(f128_enabled)]
    fn spec_tests_f128() {
        spec_test::<f128>(fdimf128);
    }
}
