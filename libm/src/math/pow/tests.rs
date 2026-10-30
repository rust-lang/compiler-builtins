use super::pow;

#[test]
fn signed_underflow_and_nan_identities() {
    for nan in [f64::NAN, f64::from_bits(0x7ff0000000000001)] {
        assert_eq!(pow(1.0, nan), 1.0);
        assert_eq!(pow(nan, 0.0), 1.0);
        assert_eq!(pow(nan, -0.0), 1.0);
    }
    assert_eq!(pow(-2.0, -1075.0).to_bits(), (-0.0f64).to_bits());
    assert_eq!(pow(-f64::from_bits(1), 3.0).to_bits(), (-0.0f64).to_bits());
    assert_eq!(pow(f64::MAX, 1.0), f64::MAX);
    assert_eq!(pow(f64::from_bits(1), 1.0).to_bits(), 1);
}

#[test]
fn subnormal_log_reduction() {
    let x = f64::from_bits(0x0000ec140bff07ec);
    let y = f64::from_bits(0x3fd405f374d3e30c);
    // The MPFR-rounded result allows one adjacent representation.
    let reference = 0x2bdf3fea5d20a749u64;
    assert!(pow(x, y).to_bits().abs_diff(reference) <= 1);
}
