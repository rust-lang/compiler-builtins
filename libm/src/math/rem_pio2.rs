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

// return the remainder of x rem pi/2 in y[0]+y[1]
// use rem_pio2_large() for large x
//
// caller must handle the case when reduction is not needed: |x| ~<= pi/4 */
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub(crate) fn rem_pio2(x: f64) -> (i32, f64, f64) {
    return super::generic::rem_pio2(x);
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
