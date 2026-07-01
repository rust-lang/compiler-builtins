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
    use crate::support::{Float, Hex};

    macro_rules! cases {
        ($f:ty) => {
            [
                // Sanity checks
                (5.0, 3.0, 2.0),
                (3.0, 5.0, 0.0),
                // Spec tests
                (1.0, 0.0, 1.0),
                (0.0, -1.0, 1.0),
                (<$f>::INFINITY, 1.0, <$f>::INFINITY),
                (1.0, <$f>::NEG_INFINITY, <$f>::INFINITY),
                (0.0, 1.0, 0.0),
                (-1.0, 0.0, 0.0),
                (<$f>::NEG_INFINITY, 0.0, 0.0),
                (0.0, 0.0, 0.0),
                (-0.0, 0.0, 0.0),
                (0.0, -0.0, 0.0),
                (1.0, 1.0, 0.0),
                (<$f>::INFINITY, <$f>::INFINITY, 0.0),
                (<$f>::NEG_INFINITY, <$f>::NEG_INFINITY, 0.0),
                // NaN inputs
                (<$f>::NAN, 1.0, <$f>::NAN),
                (1.0, <$f>::NAN, <$f>::NAN),
                (<$f>::NAN, <$f>::NAN, <$f>::NAN),
            ]
        };
    }

    #[track_caller]
    fn check<F: Float>(f: fn(F, F) -> F, cases: &[(F, F, F)]) {
        for &(x, y, exp_res) in cases {
            let val = f(x, y);
            if exp_res.is_nan() {
                assert!(
                    val.is_nan(),
                    "fdim({x:?}, {y:?}) expected NaN, got {val:?} ({} {})",
                    Hex(x),
                    Hex(y)
                );
            } else {
                assert_biteq!(val, exp_res, "fdim({x:?}, {y:?}) ({} {})", Hex(x), Hex(y));
            }
        }
    }

    #[test]
    #[cfg(f16_enabled)]
    fn check_f16() {
        check::<f16>(super::super::fdimf16, &cases!(f16));
    }

    #[test]
    fn check_f32() {
        check::<f32>(super::super::fdimf, &cases!(f32));
    }

    #[test]
    fn check_f64() {
        check::<f64>(super::super::fdim, &cases!(f64));
    }

    #[test]
    #[cfg(f128_enabled)]
    fn check_f128() {
        check::<f128>(super::super::fdimf128, &cases!(f128));
    }
}
