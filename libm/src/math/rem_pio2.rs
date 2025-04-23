#![allow(unused)]
// origin: FreeBSD /usr/src/lib/msun/src/e_rem_pio2.c
//
// ====================================================
// Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
//
// Developed at SunPro, a Sun Microsystems, Inc. business.
// Permission to use, copy, modify, and distribute this
// software is freely granted, provided that this notice
// is preserved.
// ====================================================
//
// Optimized by Bruce D. Evans. */
use super::rem_pio2_large;

// #if FLT_EVAL_METHOD==0 || FLT_EVAL_METHOD==1
// #define EPS DBL_EPSILON
const EPS: f64 = 2.2204460492503131e-16;
// #elif FLT_EVAL_METHOD==2
// #define EPS LDBL_EPSILON
// #endif

// TODO: Support FLT_EVAL_METHOD?

const TO_INT: f64 = 1.5 / EPS;
/// 53 bits of 2/pi
const INV_PIO2: f64 = 6.36619772367581382433e-01; /* 0x3FE45F30, 0x6DC9C883 */
/// first 33 bits of pi/2
const PIO2_1: f64 = 1.57079632673412561417e+00; /* 0x3FF921FB, 0x54400000 */
/// pi/2 - PIO2_1
const PIO2_1T: f64 = 6.07710050650619224932e-11; /* 0x3DD0B461, 0x1A626331 */
/// second 33 bits of pi/2
const PIO2_2: f64 = 6.07710050630396597660e-11; /* 0x3DD0B461, 0x1A600000 */
/// pi/2 - (PIO2_1+PIO2_2)
const PIO2_2T: f64 = 2.02226624879595063154e-21; /* 0x3BA3198A, 0x2E037073 */
/// third 33 bits of pi/2
const PIO2_3: f64 = 2.02226624871116645580e-21; /* 0x3BA3198A, 0x2E000000 */
/// pi/2 - (PIO2_1+PIO2_2+PIO2_3)
const PIO2_3T: f64 = 8.47842766036889956997e-32; /* 0x397B839A, 0x252049C1 */

// return the remainder of x rem pi/2 in y[0]+y[1]
// use rem_pio2_large() for large x
//
// caller must handle the case when reduction is not needed: |x| ~<= pi/4 */
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub(crate) fn rem_pio2(x: f64) -> (i32, f64, f64) {
    return super::generic::rem_frac_pi_2(x);
}

#[cfg(test)]
mod tests {
    use super::rem_pio2;

    #[test]
    // FIXME(correctness): inaccurate results on i586
    #[cfg_attr(all(target_arch = "x86", not(target_feature = "sse")), ignore)]
    fn test_near_pi() {
        let arg = 3.141592025756836;
        let arg = force_eval!(arg);
        assert_eq!(
            rem_pio2(arg),
            (2, -6.278329573009626e-7, -2.1125998133974653e-23)
        );
        let arg = 3.141592033207416;
        let arg = force_eval!(arg);
        assert_eq!(
            rem_pio2(arg),
            (2, -6.20382377148128e-7, -2.1125998133974653e-23)
        );
        let arg = 3.141592144966125;
        let arg = force_eval!(arg);
        assert_eq!(
            rem_pio2(arg),
            (2, -5.086236681942706e-7, -2.1125998133974653e-23)
        );
        let arg = 3.141592979431152;
        let arg = force_eval!(arg);
        assert_eq!(
            rem_pio2(arg),
            (2, 3.2584135866119817e-7, -2.1125998133974653e-23)
        );
    }

    #[test]
    fn test_overflow_b9b847() {
        let _ = rem_pio2(-3054214.5490637687);
    }

    #[test]
    fn test_overflow_4747b9() {
        let _ = rem_pio2(917340800458.2274);
    }
}
