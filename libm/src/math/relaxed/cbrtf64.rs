/* origin: FreeBSD /usr/src/lib/msun/src/s_cbrt.c */
/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 *
 * Optimized by Bruce D. Evans.
 */
/* cbrt(x)
 * Return cube root of x
 */

use core::f64;

const B1: u32 = 715094163; /* B1 = (1023-1023/3-0.03306235651)*2**20 */
const B2: u32 = 696219795; /* B2 = (1023-1023/3-54/3-0.03306235651)*2**20 */

/* |1/cbrt(x) - p(x)| < 2**-23.5 (~[-7.93e-8, 7.929e-8]). */
const P0: f64 = 1.87595182427177009643; /* 0x3ffe03e6, 0x0f61e692 */
const P1: f64 = -1.88497979543377169875; /* 0xbffe28e0, 0x92f02420 */
const P2: f64 = 1.621429720105354466140; /* 0x3ff9f160, 0x4a49d6c2 */
const P3: f64 = -0.758397934778766047437; /* 0xbfe844cb, 0xbee751d9 */
const P4: f64 = 0.145996192886612446982; /* 0x3fc2b000, 0xd4e4edd7 */

/// Cube root (f64)
///
/// Computes the cube root of the argument.
#[cfg_attr(assert_no_panic, no_panic::no_panic)]
pub fn cbrtf64(x: f64) -> f64 {
    let x1p54 = f64::from_bits(0x4350000000000000); // 0x1p54 === 2 ^ 54

    let mut ui: u64 = x.to_bits();
    let mut r: f64;
    let s: f64;
    let mut t: f64;
    let w: f64;
    let mut hx: u32 = (ui >> 32) as u32 & 0x7fffffff;

    if hx >= 0x7ff00000 {
        /* cbrt(NaN,INF) is itself */
        return x + x;
    }

    /*
     * Rough cbrt to 5 bits:
     *    cbrt(2**e*(1+m) ~= 2**(e/3)*(1+(e%3+m)/3)
     * where e is integral and >= 0, m is real and in [0, 1), and "/" and
     * "%" are integer division and modulus with rounding towards minus
     * infinity.  The RHS is always >= the LHS and has a maximum relative
     * error of about 1 in 16.  Adding a bias of -0.03306235651 to the
     * (e%3+m)/3 term reduces the error to about 1 in 32. With the IEEE
     * floating point representation, for finite positive normal values,
     * ordinary integer divison of the value in bits magically gives
     * almost exactly the RHS of the above provided we first subtract the
     * exponent bias (1023 for doubles) and later add it back.  We do the
     * subtraction virtually to keep e >= 0 so that ordinary integer
     * division rounds towards minus infinity; this is also efficient.
     */
    if hx < 0x00100000 {
        /* zero or subnormal? */
        ui = (x * x1p54).to_bits();
        hx = (ui >> 32) as u32 & 0x7fffffff;
        if hx == 0 {
            return x; /* cbrt(0) is itself */
        }
        hx = hx / 3 + B2;
    } else {
        hx = hx / 3 + B1;
    }
    ui &= 1 << 63;
    ui |= (hx as u64) << 32;
    t = f64::from_bits(ui);

    /*
     * New cbrt to 23 bits:
     *    cbrt(x) = t*cbrt(x/t**3) ~= t*P(t**3/x)
     * where P(r) is a polynomial of degree 4 that approximates 1/cbrt(r)
     * to within 2**-23.5 when |r - 1| < 1/10.  The rough approximation
     * has produced t such than |t/cbrt(x) - 1| ~< 1/32, and cubing this
     * gives us bounds for r = t**3/x.
     *
     * Try to optimize for parallel evaluation as in __tanf.c.
     */
    r = (t * t) * (t / x);
    t = t * ((P0 + r * (P1 + r * P2)) + ((r * r) * r) * (P3 + r * P4));

    /*
     * Round t away from zero to 23 bits (sloppily except for ensuring that
     * the result is larger in magnitude than cbrt(x) but not much more than
     * 2 23-bit ulps larger).  With rounding towards zero, the error bound
     * would be ~5/6 instead of ~4/6.  With a maximum error of 2 23-bit ulps
     * in the rounded t, the infinite-precision error in the Newton
     * approximation barely affects third digit in the final error
     * 0.667; the error in the rounded t can be up to about 3 23-bit ulps
     * before the final error is larger than 0.667 ulps.
     */
    ui = t.to_bits();
    ui = (ui + 0x80000000) & 0xffffffffc0000000;
    t = f64::from_bits(ui);

    /* one step Newton iteration to 53 bits with error < 0.667 ulps */
    s = t * t; /* t*t is exact */
    r = x / s; /* error <= 0.5 ulps; |r| < |t| */
    w = t + t; /* t+t is exact */
    r = (r - t) / (w + r); /* r-t is exact; w+r ~= 3*t */
    t = t + t * r; /* error <= 0.5 + 0.5/3 + epsilon */
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The distance between `a` and `b` measured in ULPs, i.e. the number of
    /// distinct representable `f64` values in between (inclusive of `±0`).
    fn ulps(a: f64, b: f64) -> u64 {
        // Map IEEE-754 bit patterns to a monotonic integer ordering: positive
        // values have the sign bit set and negative values are bitwise
        // inverted, converting from sign-magnitude to a two's complement-like
        // ordering.
        fn order(f: f64) -> u64 {
            let bits = f.to_bits();
            if bits >> 63 == 1 {
                !bits
            } else {
                bits | (1 << 63)
            }
        }
        order(a).abs_diff(order(b))
    }

    #[test]
    fn special_values() {
        assert_biteq!(cbrtf64(0.0), 0.0);
        assert_biteq!(cbrtf64(-0.0), -0.0);
        assert_biteq!(cbrtf64(1.0), 1.0);
        assert_biteq!(cbrtf64(-1.0), -1.0);

        // cbrt(±inf) is ±inf, cbrt(NaN) is NaN.
        assert_biteq!(cbrtf64(f64::INFINITY), f64::INFINITY);
        assert_biteq!(cbrtf64(f64::NEG_INFINITY), f64::NEG_INFINITY);
        assert!(cbrtf64(f64::NAN).is_nan());

        // Neither the largest finite value nor a subnormal may overflow or
        // underflow to an infinite or zero result.
        assert!(cbrtf64(f64::MAX).is_finite());
        assert!(cbrtf64(f64::from_bits(0x0000000000000001)) > 0.0);
    }

    #[test]
    fn perfect_cubes() {
        for (x, expected) in [
            (8.0, 2.0),
            (27.0, 3.0),
            (64.0, 4.0),
            (-8.0, -2.0),
            (0.125, 0.5),
        ] {
            let got = cbrtf64(x);
            assert!(
                ulps(got, expected) <= 4,
                "cbrtf64({x}) = {got}, expected {expected} ({} ulps off)",
                ulps(got, expected)
            );
        }
    }

    #[test]
    fn close_to_main_cbrt() {
        // The relaxed implementation must stay within a few ULPs of the
        // (extensively tested) main `cbrt` across the whole exponent range,
        // including subnormals and the largest finite values.
        let mantissas = [
            0x0,
            0x1,
            0x8000_0000_0000,
            0x000f_ffff_ffff_ffff,
            0xaaaa_aaaa_aaaa_aaaa,
        ];
        let mut checked = 0u64;
        for e in 0..=2046u64 {
            for m in mantissas {
                for sign in [0u64, 0x8000_0000_0000_0000] {
                    let x = f64::from_bits(sign | (e << 52) | m);
                    let got = cbrtf64(x);
                    let want = super::super::super::cbrt::cbrt(x);
                    let d = ulps(got, want);
                    assert!(
                        d <= 8,
                        "cbrtf64({x:e}) = {got:e} ({:#x}), main cbrt = {want:e} ({:#x}), {d} ulps off",
                        got.to_bits(),
                        want.to_bits(),
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked >= 20_000, "sweep covered {checked} values");
    }
}
