/* SPDX-License-Identifier: MIT */
/* origin: core-math/src/binary64/log2p1/log2p1.c
 * Copyright (c) 2022-2025 INRIA and CERN.
 * Authors: Paul Zimmermann and Tom Hubrecht.
 * Ported to Rust in 2026, malezjaa
 * CORE-MATH commit: dae5dd8c0d5bd2691b45de28c9edbe12f067c3d3
 */

mod dint;
mod log2p1f;

#[cfg(f16_enabled)]
mod log2p1f16;

#[cfg(f16_enabled)]
pub use log2p1f16::log2p1f16;

use self::dint::Dint;
pub use self::log2p1f::log2p1f;
use super::support::{CastFrom, CastInto, Float, cold_path};

/// Base-2 logarithm of x plus one (f64).
///
/// Calculate `log2(x + 1)`, that is, the base-2 logarithm of `x + 1`.
#[cfg_attr(assert_no_panic, no_panic::no_panic)]
pub fn log2p1(x: f64) -> f64 {
    cr_log2p1(x)
}

fn cr_log2p1(x: f64) -> f64 {
    let bits = x.to_bits();
    let e = i32::cast_from((bits >> 52) & 0x7ff) - 0x3ff;

    // case NaN/Inf, +/-0 or x <= -1
    if e == 0x400 || x == 0.0 || x <= -1.0 {
        cold_path();
        let minf = f64::NEG_INFINITY;

        // NaN or + Inf
        if e == 0x400 && x != minf {
            return x + x;
        }

        // we use the fact that NaN < -1 is false
        if x <= -1.0 {
            // log2p(x<-1) is NaN, log2p(-1) is -Inf and raises DivByZero
            if x < -1.0 {
                return 0.0 / 0.0;
            } else {
                // x=-1
                return 1.0 / -0.0;
            }
        }
        return x + x; // +/-0
    }

    // now x > -1

    // check x=2^n-1 for 0 <= n <= 53, where log2p1(x) is exact,
    // and we shouldn't raise the inexact flag
    if (0..=52).contains(&e) {
        // T[e]=2^(e+1)-1, i.e., the unique value of the form 2^n-1
        // in the interval [2^e, 2^(e+1)).
        static T: [f64; 53] = [
            hf64!("0x1p+0"),
            hf64!("0x1.8p+1"),
            hf64!("0x1.cp+2"),
            hf64!("0x1.ep+3"),
            hf64!("0x1.fp+4"),
            hf64!("0x1.f8p+5"),
            hf64!("0x1.fcp+6"),
            hf64!("0x1.fep+7"),
            hf64!("0x1.ffp+8"),
            hf64!("0x1.ff8p+9"),
            hf64!("0x1.ffcp+10"),
            hf64!("0x1.ffep+11"),
            hf64!("0x1.fffp+12"),
            hf64!("0x1.fff8p+13"),
            hf64!("0x1.fffcp+14"),
            hf64!("0x1.fffep+15"),
            hf64!("0x1.ffffp+16"),
            hf64!("0x1.ffff8p+17"),
            hf64!("0x1.ffffcp+18"),
            hf64!("0x1.ffffep+19"),
            hf64!("0x1.fffffp+20"),
            hf64!("0x1.fffff8p+21"),
            hf64!("0x1.fffffcp+22"),
            hf64!("0x1.fffffep+23"),
            hf64!("0x1.ffffffp+24"),
            hf64!("0x1.ffffff8p+25"),
            hf64!("0x1.ffffffcp+26"),
            hf64!("0x1.ffffffep+27"),
            hf64!("0x1.fffffffp+28"),
            hf64!("0x1.fffffff8p+29"),
            hf64!("0x1.fffffffcp+30"),
            hf64!("0x1.fffffffep+31"),
            hf64!("0x1.ffffffffp+32"),
            hf64!("0x1.ffffffff8p+33"),
            hf64!("0x1.ffffffffcp+34"),
            hf64!("0x1.ffffffffep+35"),
            hf64!("0x1.fffffffffp+36"),
            hf64!("0x1.fffffffff8p+37"),
            hf64!("0x1.fffffffffcp+38"),
            hf64!("0x1.fffffffffep+39"),
            hf64!("0x1.ffffffffffp+40"),
            hf64!("0x1.ffffffffff8p+41"),
            hf64!("0x1.ffffffffffcp+42"),
            hf64!("0x1.ffffffffffep+43"),
            hf64!("0x1.fffffffffffp+44"),
            hf64!("0x1.fffffffffff8p+45"),
            hf64!("0x1.fffffffffffcp+46"),
            hf64!("0x1.fffffffffffep+47"),
            hf64!("0x1.ffffffffffffp+48"),
            hf64!("0x1.ffffffffffff8p+49"),
            hf64!("0x1.ffffffffffffcp+50"),
            hf64!("0x1.ffffffffffffep+51"),
            hf64!("0x1.fffffffffffffp+52"),
        ];

        if x == T[e as usize] {
            return f64::from(e + 1);
        }
    }

    // For x=2^k-1, -53 <= k <= -1, log2p1(x) = k is also exact.
    if e == -1 && x < 0.0 {
        // -1 < x <= -1/2
        cold_path();

        let w = 1.0 + x; // 1+x is exact
        let w_bits = w.to_bits();

        if w_bits << 12 == 0 {
            // 1+x = 2^k
            let k = i32::cast_from(w_bits >> 52) - 0x3ff;
            return k.cast();
        }
    }

    // now x = m*2^e with 1 <= m < 2 (m = v.f) and -1074 <= e <= 1023
    let (hi, lo, err) = cr_log2p1_fast(x, e);
    let left = hi + (lo - err);
    let right = hi + (lo + err);
    if left == right {
        return left;
    }

    cold_path();
    cr_log2p1_accurate(x)
}

/// Given `x > -1`, put in `(h,l)` a double-double approximation of `log2(1+x)`,
/// and return a bound err on the maximal absolute error so that:
/// `|h + l - log2(1+x)| < err`.
/// We have `x = m*2^e` with `1 <= m < 2 (m = v.f)` and `-1074 <= e <= 1023`.
/// This routine is adapted from cr_log1p_fast.
fn cr_log2p1_fast(x: f64, e: i32) -> (f64, f64, f64) {
    // e <= -6 thus |x| < 2^-5
    if e < -5 {
        if e <= -920 {
            // For e <= -969, |x| might be as small as 2^-969, thus h=x/log(2) might
            // be in the binade [2^-969,2^-968), with ulp(h) = 2^-1021, and if
            // |l| < ulp(h), then l.ulp() might be smaller than 2^-1074. We defer
            // that case to the accurate path.
            // Moreover, for e <= -962, the computation 0x1.d4p-62 * *h of the
            // error term below might produce a spurious underflow. We also defer
            // that case to the accurate path.
            // In addition some spurious underflows might happen in the d_mul()
            // call.
            return (0.0, 0.0, 1.0);
        }

        let (h, lo) = p_1a(x);
        let (h, mut l) = fast_two_sum(x, h);
        l += lo;

        // from analyze_x_plus_p1a(rel=true,Xmax=2^-5.) in the accompanying file
        // log1p.sage, the relative error is bounded by 2^-61.14 with respect to
        // h. We use the fact that we don't need the return value err to be
        // positive, since we add/subtract it in the rounding test.
        // We also get that the ratio |l/h| is bounded by 2^-50.96.
        // now we multiply h+l by 1/log(2)
        let (h, l) = d_mul(h, l, INVLOG2H, INVLOG2L);
        // the d_mul() call decomposes into:
        //  a_mul (h_out, l1, h, INVLOG2H)
        //  l2 = __builtin_fma (h, INVLOG2L, l1)
        //  l_out = __builtin_fma (l, INVLOG2H, l2)
        //  we have |l1| <= ulp(h_out)
        //  since |INVLOG2L/INVLOG2H| < 2^-55, then |h*INVLOG2L| <= 2^-55*|h_out|
        //  and since |x| < 2^53*ulp(x): |h*INVLOG2L| <= ulp(h_out)/4
        //  thus |l2| <= 5/4*ulp(h_out).
        //  Now since |l/h| < 2^-50.96, |l*INVLOG2H| < 2^-50.96*|h*INVLOG2H|
        //  < 2^-50.96*(1+2^-52)*|h_out| < 2^-50.95*|h_out| < 4.15*ulp(h_out),
        //  thus |l_out| < o(4.15*ulp(h_out)+5/4*ulp(h_out)) < 5.5*ulp(h_out).
        //  The rounding errors are bounded by ulp(l2)+ulp(l_out)
        //  <= ulp(5/4*ulp(h_out)) + ulp(5.5*ulp(h_out))
        //  <= 2^-52*(5/4*ulp(h_out)+5.5*ulp(h_out)) [using ulp(x) <= 2^-52*|x|]
        //  <= 2^-49.2*ulp(h_out)
        //  We also have to take into account the ignored term l*INVLOG2L:
        //  |l*INVLOG2L| < 2^-50.96*|h|*2^-55.97*|INVLOG2H|
        //               < 2^-106.93*(1+2^-52)*|h_out|
        //               < 2^-106.92*|h_out|
        //               < 2^-51.92*ulp(h_out) [using |x| < 2^53*ulp(x)]
        // and the approximation error in INVLOG2H+INVLOG2L:
        // |INVLOG2H + INVLOG2L - 1/log(2)| < 2^-110/log(2)
        // The total error of d_mul() is thus bounded by:
        // (2^-49.2+2^-51.92)*ulp(h_out) < 2^-48.99*ulp(h_out) < 2^-100.99*|h_out|,
        // using again ulp(x) <= 2^-52*|x|.
        //
        // The relative error is thus bounded by
        // (1+2^-61.14)*(1+2^-100.99)*(1+2^-110)-1 < 2^-61.13
        return (h, l, hf64!("0x1.d4p-62") * h); // 2^-61.13 < 0x1.d4p-62
    }

    // (xh,xl) <- 1+x
    let (xh, xl) = if x > 1.0 {
        if x < f64::MAX {
            fast_two_sum(x, 1.0)
        } else {
            cold_path();
            // avoid spurious overflow for RNDU
            (x, 1.0)
        }
    } else {
        fast_two_sum(1.0, x)
    };

    let bits = xh.to_bits();
    let e = i32::cast_from(bits >> 52) - 0x3ff;
    let v = (0x3ff << 52) | (bits & 0xfffffffffffff);
    let (h, mut l) = cr_log_fast(e, v);

    // log(xh+xl) = log(xh) + log(1+xl/xh)
    let c = if xh <= hf64!("0x1p1022") || xl.abs() >= 4.0 {
        xl / xh
    } else {
        cold_path();
        // avoid spurious underflow
        0.0
    };

    // Since |xl| < ulp(xh), we have |xl| < 2^-52 |xh|,
    // thus |c| < 2^-52, and since |log(1+x)-x| < x^2 for |x| < 0.5,
    // we have |log(1+c)-c)| < c^2 < 2^-104.
    l += c;

    // Since |l_in| < 2^-18.69 (from the analysis of cr_log_fast, see file
    // ../log/log.c), and |c| < 2^-52, we have |l| < 2^-18.68, thus the
    // rounding error in *l += c is bounded by ulp(2^-18.68) = 2^-71.
    // The total absolute error is thus bounded by:
    // 0x1.b6p-69 + 2^-104 + 2^-71 < 2^-68.02.

    // now multiply h+l by 1/log(2)
    let (h, l) = d_mul(h, l, INVLOG2H, INVLOG2L);
    // the d_mul() call decomposes into:
    // a_mul (h_out, l1, h, INVLOG2H)
    // l2 = __builtin_fma (h, INVLOG2L, l1)
    // l_out = __builtin_fma (l, INVLOG2H, l2)
    // We have three errors:
    // * the rounding error in l2 = __builtin_fma (h, INVLOG2L, l1)
    // * the rounding error in l_out = __builtin_fma (l, INVLOG2H, l2)
    // * the ignored term l * INVLOG2L
    // We have |h| < 745 thus |h*INVLOG2H| < 1075 thus |h_out| <= 1075
    // and |l1| <= ulp(h_out) <= 2^-42.
    // Then |h*INVLOG2L+l1| <= 745*INVLOG2L+2^-42 < 2^-41.9
    // thus |l2| < 2^-41.9*(1+2^-52) < 2^-41.8
    // and the first rounding error is bounded by ulp(2^-41.8) = 2^-94.
    // Now |l*INVLOG2H+l2| < 2^-18.68*INVLOG2H+2^-41.8 < 2^-18.1
    // thus |l_out| < 2^-18.1*(1+2^-52) < 2^-18.09
    // and the second rounding error is bounded by ulp(2^-18.09) = 2^-71.
    // The ignored term is bounded by |l*INVLOG2L| < 2^-18.68*INVLOG2L < 2^-74.1.
    // Thus the absolute error from d_mul() is bounded by:
    // 2^-94 + 2^-71 + 2^-74.1 < 2^-70.84.
    //
    // Adding to the maximal absolute error of 2^-68.02 before d_mul(),
    // we get 2^-68.02 + 2^-70.84 < 2^-67.82.
    (h, l, hf64!("0x1.23p-68")) // 2^-67.82 < 0x1.23p-68
}

// accurate path, using Tom Hubrecht's code below
fn cr_log2p1_accurate(x: f64) -> f64 {
    let ax = x.abs();
    if ax < hf64!("0x1p-5") {
        if ax < hf64!("0x1p-105") {
            return cr_log2p1_accurate_tiny(x);
        } else {
            return cr_log2p1_accurate_small(x);
        };
    }

    // (xh,xl) <- 1+x
    let (xh, xl) = if x > 1.0 {
        fast_two_sum(x, 1.0)
    } else {
        fast_two_sum(1.0, x)
    };

    // log2p1(x) is exact when 1+x = 2^e, thus when 2^e-1 is exactly
    // representable. This can only occur when xl=0 here.
    if xl == 0.0 {
        // check if xh is a power of two
        let bits = xh.to_bits();
        if bits << 12 == 0 {
            let e = i32::cast_from(bits >> 52) - 0x3ff;
            return e.cast();
        }
    }

    // if x=2^e, the accurate path will fail for directed roundings
    let bits = x.to_bits();
    if bits << 12 == 0 {
        let e = i32::cast_from((bits >> 52) & 0x7ff) - 0x3ff; // x = 2^e
        // for e >= 49, log2p1(x) rounds to e for rounding to nearest;
        // for e >= 48, log2p1(x) rounds to e for rounding toward zero;
        // for e >= 48, log2p1(x) rounds to nextabove(e) for rounding up;
        // for e >= 48, log2p1(x) rounds to e for rounding down.

        if e >= 49 {
            // 0x1p-48 = 1/2 ulp(49)
            return f64::from(e) + hf64!("0x1p-48");
        }
    }

    let y = log_2(Dint::from_double(xh));
    let c = Dint::from_ratio(xl, xh);
    let mut correction = c.mul(c);
    // multiply X by -1/2
    correction.ex -= 1;
    correction.sgn = 1;
    // C <- C - C^2/2
    let c = c.add(correction);
    // |C-log(1+xl/xh)| ~ 2e-64 and multiply by 1/log(2)
    let mut y = y.add(c).mul(Dint::LOG2_INV);
    y.ex -= 12; // since LOG2_INV approximates 2^12/log(2)
    y.to_double()
}

/// deal with |x| < 2^-900, then log2p1(x) ~ x/log(2)
fn cr_log2p1_accurate_tiny(x: f64) -> f64 {
    // exceptional values
    if x.abs() == hf64!("0x0.2c316a14459d8p-1022") {
        if x > 0.0 {
            return hf64!("0x1p-600").fma(hf64!("0x1p-600"), hf64!("0x1.fe0e7458ac1f8p-1025"));
        }

        return hf64!("-0x1p-600").fma(hf64!("0x1p-600"), hf64!("-0x1.fe0e7458ac1f8p-1025"));
    }

    // first scale x to avoid truncation of l in the underflow region
    let sx = x * hf64!("0x1p106");
    let (h, l) = s_mul(sx, INVLOG2H, INVLOG2L);
    let res = (h + l) * hf64!("0x1p-106"); // expected result
    let l = (-res).fma(hf64!("0x1p106"), h) + l;

    // the correction to apply to res is l*2^-106
    // For all rounding modes, we have underflow
    // for |x| <= 0x1.62e42fefa39eep-1023
    l.fma(hf64!("0x1p-106"), res)
}

// deal with 2^-900 <= x < 2^-5, using the polynomial Pacc
fn cr_log2p1_accurate_small(x: f64) -> f64 {
    let mut a = 0;
    let mut b = EXCEPTIONS_TABLE.len();
    while a + 1 != b {
        // Invariant: if x is an exceptional case, we have
        // exceptions[a][0] <= x and (x < exceptions[b][0] or b = EXCEPTIONS)
        let c = (a + b) / 2;
        if i!(EXCEPTIONS_TABLE, c).0 <= x {
            a = c;
        } else {
            b = c;
        }
    }

    if x == i!(EXCEPTIONS_TABLE, a).0 {
        let h = i!(EXCEPTIONS_TABLE, a).1;
        let del = if h > 0.0 {
            i!(EXCEPTIONS_RND, a)
        } else {
            -i!(EXCEPTIONS_RND, a)
        };
        return h + h * hf64!("0x1p-54") * f64::from(del);
    }

    // for degree 11 or more, ulp(c[d]*x^d) < 2^-105.5*|log2p1(x)|
    // where c[d] is the degree-d coefficient of Pacc, thus we can compute
    // with a double only
    let mut h = PACC[23].fma(x, PACC[22]); // degree 16
    for i in (11..=15).rev() {
        h = h.fma(x, PACC[i + 6]); // degree i
    }

    let mut l = 0.0;
    for i in (8..=10).rev() {
        (h, l) = s_mul(x, h, l);
        let (hi, t) = fast_two_sum(PACC[i + 6], h);
        h = hi;
        l += t;
    }
    for i in (1..=7).rev() {
        (h, l) = s_mul(x, h, l);
        let (hi, t) = fast_two_sum(PACC[2 * i - 2], h);
        h = hi;
        l += t + PACC[2 * i - 1];
    }

    // final multiplication by x
    let (h, l) = s_mul(x, h, l);
    h + l
}

// the following code was copied from Tom Hubrecht's implementation of
// correctly rounded pow for CORE-MATH

// Approximation for the second iteration
fn p_2(z: Dint) -> Dint {
    let mut r = P_2[0];
    for coefficient in &P_2[1..] {
        r = z.mul(r);
        r = coefficient.add(r);
    }
    z.mul(r)
}

fn log_2(mut x: Dint) -> Dint {
    let mut e = x.ex;
    // Find the lookup index
    let mut i = usize::cast_from(x.hi >> 55);
    if x.hi > 0xb504f333f9de6484 {
        e += 1;
        i >>= 1;
    }
    x.ex -= e;

    let z = x.mul(i!(_INVERSE_2, i - 128));
    let z = Dint::M_ONE.add(z);
    // E·log(2)
    let r = Dint::LOG2.mul_with_int(e);
    let p = i!(_LOG_INV_2, i - 128).add(p_2(z));
    p.add(r)
}

// Given 1 <= x < 2, where x = v.f, put in h+l a double-double approximation
// of log(2^e*x), with absolute error bounded by 2^-69.99 (details below).
fn cr_log_fast(mut e: i32, v: u64) -> (f64, f64) {
    let m = 0x10000000000000 + (v & 0xfffffffffffff);

    // x = m/2^52
    // if x > sqrt(2), we divide it by 2 to avoid cancellation
    let (i, y) = if m >= 0x16a09e667f3bcd {
        e += 1;
        (m >> 44, f64::from_bits(v) * 0.5)
    } else {
        (m >> 43, f64::from_bits(v))
    };

    const OFFSET: u64 = 362;

    // now -1074 <= e <= 1024
    let i = usize::cast_from(i - OFFSET);
    let r = _INVERSE[i];
    let (l1, l2) = _LOG_INV[i];
    let z = r.fma(y, -1.0); // exact
    // evaluate P(z), for |z| < 0.00212097167968735
    let (ph, pl) = p_1(z);

    // Add e*log(2) to (h,l), where -1074 <= e <= 1023, thus e has at most
    // 11 bits. log2_h is an integer multiple of 2^-42, so that e*log2_h
    // is exact.
    const LOG2_H: f64 = hf64!("0x1.62e42fefa38p-1");
    const LOG2_L: f64 = hf64!("0x1.ef35793c7673p-45");

    // |log(2) - (h+l)| < 2^-102.01
    // let hh = e * log2_h: hh is an integer multiple of 2^-42,
    // with |hh| <= 1074*log2_h
    // = 3274082061039582*2^-42. l1 is also an integer multiple of 2^-42,
    // with |l1| <= 1524716581803*2^-42. Thus hh+l1 is an integer multiple of
    // 2^-42, with 2^42*|hh+l1| <= 3275606777621385 < 2^52, thus hh+l1 is exactly
    // representable.
    let ee = f64::from(e);
    let (h, l) = fast_two_sum(ee.fma(LOG2_H, l1), z);
    // here |hh+l1|+|z| <= 3275606777621385*2^-42 + 0.0022 < 745
    // thus |h| < 745, and the additional error from the fast_two_sum() call is
    // bounded by 2^-105*745 < 2^-95.4.
    // add ph + pl + l2 to l
    let l = ph + (l + (l2 + pl));
    // here |ph| < 2.26e-6, |l| < ulp(h) = 2^-43, |l2| < 2^-43 and
    // |pl| < 2^-69.99, thus |l2 + pl| < 2^-42 and |*l + l2 + pl| < 2^-41.99,
    // and the rounding error on l2 + pl is bounded by 2^-95 (l2 + pl cannot
    // be > 2^-42), and that on *l + (...) by 2^-94.
    // Now |ph + (*l + (l2 + pl))| < 2.26e-6 + 2^-41.99 < 2^-18.7, thus the
    // rounding error on ph + ... is bounded by ulp(2^-18.7) = 2^-71, which
    // yields a cumulated error bound of 2^-71 + 2^-95 + 2^-94 < 2^-70.99.
    let l = ee.fma(LOG2_L, l);
    // let l_in be the input value of *l, and l_out the output value.
    // We have |l_in| < 2^-18.7 (from above)
    // and |e*log2_l| <= 1074*0x1.ef35793c7673p-45
    // thus |l_out| < 2^-18.69 and err(l_out) <= ulp(2^-18.69) = 2^-71

    // The absolute error on h + l is bounded by:
    // 2^-78.25 from the error in the Sollya polynomial plus the rounding errors
    //          in p_1 (&ph, &pl, z)
    // 2^-91.94 for the maximal difference |e*(log(2)-(log2_h + log2_l))|
    //          (|e| <= 1074 and |log(2)-(log2_h + log2_l)| < 2^-102.01)
    // 2^-97 for the maximal difference |l1 + l2 - (-log(r))|
    // 2^-95.4 from the fast_two_sum call
    // 2^-70.99 from the *l = ph + (*l + l2) instruction
    // 2^-71 from the last __builtin_fma call.
    // This gives an absolute error bounded by < 2^-69.99.
    (h, l)
}

// put in h+l a double-double approximation of log(z)-z for
// |z| < 0.00212097167968735, with absolute error bounded by 2^-78.25
// (see analyze_p1(-0.00212097167968735,0.00212097167968735)
// from accompanying file log1p.sage, which also yields |l| < 2^-69.99)
fn p_1(z: f64) -> (f64, f64) {
    let (z2h, z2l) = a_mul(z, z); // exact: z2h+z2l = z*z
    let p56 = P[6].fma(z, P[5]);
    let p34 = P[4].fma(z, P[3]);

    let mut ph = p56.fma(z2h, p34);
    // ph approximates P[3]+P[4]*z+P[5]*z^2+P[6]*z^3
    ph = ph.fma(z, P[2]);
    // ph approximates P[2]+P[3]*z+P[4]*z^2+P[5]*z^3+P[6]*z^4
    ph *= z2h;
    // ph approximates P[2]*z^2+P[3]*z^3+P[4]*z^4+P[5]*z^5+P[6]*z^6

    let (h, mut l) = fast_two_sum(-0.5 * z2h, ph * z);
    l += -0.5 * z2l;

    (h, l)
}

// put in h+l a double-double approximation of log(z)-z for
// |z| < 0.03125, with absolute error bounded by 2^-67.14
// (see analyze_p1a(-0.03125,0.03125) from log1p.sage)
fn p_1a(z: f64) -> (f64, f64) {
    let (z2h, z2l) = if z.abs() >= hf64!("0x1p-255") {
        a_mul(z, z)
    } else {
        cold_path();
        // avoid spurious underflow
        (0.0, 0.0)
    };

    let z4h = z2h * z2h;

    let p910 = PA[10].fma(z, PA[9]);
    let p78 = PA[8].fma(z, PA[7]);
    let p56 = PA[6].fma(z, PA[5]);
    let p34 = PA[4].fma(z, PA[3]);

    let p710 = p910.fma(z2h, p78);
    let p36 = p56.fma(z2h, p34);

    let mut ph = p710.fma(z4h, p36);
    ph = ph.fma(z, PA[2]);
    ph *= z2h;

    let (h, mut l) = fast_two_sum(-0.5 * z2h, ph * z);
    l += -0.5 * z2l;

    (h, l)
}

// Add a + b, such that *hi + *lo approximates a + b.
// Assumes |a| >= |b|.
fn fast_two_sum(a: f64, b: f64) -> (f64, f64) {
    let hi = a + b;
    let e = hi - a; // exact
    let lo = b - e; // exact

    // Now hi + lo = a + b exactly for rounding to nearest.
    // For directed rounding modes, this is not always true.
    // Take for example a = 1, b = 2^-200, and rounding up,
    // then hi = 1 + 2^-52, e = 2^-52 (it can be proven that
    // e is always exact), and lo = -2^52 + 2^-105, thus
    // hi + lo = 1 + 2^-105 <> a + b = 1 + 2^-200.
    // A bound on the error is given
    // in "Note on FastTwoSum with Directed Roundings"
    // by Paul Zimmermann, https://hal.inria.fr/hal-03798376, 2022.
    // Theorem 1 says that
    // the difference between a+b and hi+lo is bounded by 2u^2|a+b|
    // and also by 2u^2|hi|. Here u=2^-53, thus we get:
    // |(a+b)-(hi+lo)| <= 2^-105 min(|a+b|,|hi|)

    (hi, lo)
}

// Multiply exactly a and b, such that *hi + *lo = a * b.
#[inline]
fn a_mul(a: f64, b: f64) -> (f64, f64) {
    let hi = a * b;
    (hi, a.fma(b, -hi))
}

// Multiply a double with a double double : a * (bh + bl)
#[inline]
fn s_mul(a: f64, bh: f64, bl: f64) -> (f64, f64) {
    let (hi, s) = a_mul(a, bh); // exact
    (hi, a.fma(bl, s))
}

// Returns (ah + al) * (bh + bl) - (al * bl)
#[inline]
fn d_mul(ah: f64, al: f64, bh: f64, bl: f64) -> (f64, f64) {
    let (hi, lo) = a_mul(ah, bh);
    let lo = ah.fma(bl, lo);
    (hi, al.fma(bh, lo))
}

// INVLOG2H+INVLOG2L is a double-double approximation of 1/log(2):
// | INVLOG2H + INVLOG2L - 1/log(2) | < 2^-109.53
const INVLOG2H: f64 = hf64!("0x1.71547652b82fep+0");
const INVLOG2L: f64 = hf64!("0x1.777d0ffda0d24p-56");

// Approximation tables
#[rustfmt::skip]
static _INVERSE_2: [Dint; 240] = [
    Dint { hi: 0x8000000000000000, lo: 0x0, ex: 1, sgn: 0x0 },
    Dint { hi: 0xfe03f80fe03f80ff, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xfc0fc0fc0fc0fc10, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xfa232cf252138ac0, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xf83e0f83e0f83e10, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xf6603d980f6603da, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xf4898d5f85bb3951, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xf2b9d6480f2b9d65, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xf0f0f0f0f0f0f0f1, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xef2eb71fc4345239, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xed7303b5cc0ed731, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xebbdb2a5c1619c8c, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xea0ea0ea0ea0ea0f, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xe865ac7b7603a197, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xe6c2b4481cd8568a, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xe525982af70c880f, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xe38e38e38e38e38f, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xe1fc780e1fc780e2, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xe070381c0e070382, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xdee95c4ca037ba58, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xdd67c8a60dd67c8b, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xdbeb61eed19c5958, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xda740da740da740e, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xd901b2036406c80e, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xd79435e50d79435f, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xd62b80d62b80d62c, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xd4c77b03531dec0e, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xd3680d3680d3680e, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xd20d20d20d20d20e, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xd0b69fcbd2580d0c, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xcf6474a8819ec8ea, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xce168a7725080ce2, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xcccccccccccccccd, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xcb8727c065c393e1, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xca4587e6b74f032a, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xc907da4e871146ad, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xc7ce0c7ce0c7ce0d, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xc6980c6980c6980d, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xc565c87b5f9d4d1c, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xc4372f855d824ca6, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xc30c30c30c30c30d, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xc1e4bbd595f6e948, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xc0c0c0c0c0c0c0c1, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xbfa02fe80bfa02ff, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xbe82fa0be82fa0bf, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xbd69104707661aa3, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xbc52640bc52640bd, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xbb3ee721a54d880c, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xba2e8ba2e8ba2e8c, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xb92143fa36f5e02f, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xb81702e05c0b8171, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xb70fbb5a19be3659, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xb60b60b60b60b60c, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xb509e68a9b948220, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xb40b40b40b40b40c, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xb30f63528917c80c, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xb21642c8590b2165, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xb11fd3b80b11fd3c, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xb02c0b02c0b02c0c, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xaf3addc680af3ade, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xae4c415c9882b932, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xad602b580ad602b6, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xac7691840ac76919, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xab8f69e28359cd12, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xaaaaaaaaaaaaaaab, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xa9c84a47a07f5638, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xa8e83f5717c0a8e9, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xa80a80a80a80a80b, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xa72f05397829cbc2, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xa655c4392d7b73a8, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xa57eb50295fad40b, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xa4a9cf1d96833752, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xa3d70a3d70a3d70b, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xa3065e3fae7cd0e1, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xa237c32b16cfd773, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xa16b312ea8fc377d, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xa0a0a0a0a0a0a0a1, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x9fd809fd809fd80a, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x9f1165e7254813e3, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x9e4cad23dd5f3a21, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x9d89d89d89d89d8a, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x9cc8e160c3fb19b9, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x9c09c09c09c09c0a, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x9b4c6f9ef03a3caa, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x9a90e7d95bc609aa, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x99d722dabde58f07, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x991f1a515885fb38, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x9868c809868c8099, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x97b425ed097b425f, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x97012e025c04b80a, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x964fda6c0964fda7, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x95a02568095a0257, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x94f2094f2094f20a, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x9445809445809446, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x939a85c40939a85d, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x92f113840497889d, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x924924924924924a, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x91a2b3c4d5e6f80a, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x90fdbc090fdbc091, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x905a38633e06c43b, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8fb823ee08fb823f, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8f1779d9fdc3a219, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8e78356d1408e784, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8dda520237694809, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8d3dcb08d3dcb08e, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8ca29c046514e024, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8c08c08c08c08c09, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8b70344a139bc75b, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8ad8f2fba9386823, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8a42f8705669db47, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x89ae4089ae4089af, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x891ac73ae9819b51, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8888888888888889, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x87f78087f78087f8, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8767ab5f34e47ef2, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x86d905447a34acc7, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x864b8a7de6d1d609, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x85bf37612cee3c9b, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8534085340853409, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x84a9f9c8084a9f9d, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8421084210842109, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x839930523fbe3368, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x83126e978d4fdf3c, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x828cbfbeb9a020a4, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8208208208208209, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x81848da8faf0d278, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8102040810204082, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8000000000000000, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0x8000000000000000, lo: 0x0, ex: 0, sgn: 0x0 },
    Dint { hi: 0xff00ff00ff00ff02, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xfe03f80fe03f80ff, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xfd08e5500fd08e56, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xfc0fc0fc0fc0fc11, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xfb18856506ddaba7, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xfa232cf252138ac1, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xf92fb2211855a866, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xf83e0f83e0f83e11, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xf74e3fc22c700f76, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xf6603d980f6603db, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xf57403d5d00f5741, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xf4898d5f85bb3951, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xf3a0d52cba872337, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xf2b9d6480f2b9d66, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xf1d48bcee0d399fb, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xf0f0f0f0f0f0f0f2, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xf00f00f00f00f010, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xef2eb71fc4345239, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xee500ee500ee5010, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xed7303b5cc0ed731, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xec979118f3fc4da3, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xebbdb2a5c1619c8d, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xeae56403ab959010, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xea0ea0ea0ea0ea10, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xe939651fe2d8d35d, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xe865ac7b7603a198, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xe79372e225fe30da, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xe6c2b4481cd8568a, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xe5f36cb00e5f36cc, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xe525982af70c880f, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xe45932d7dc52100f, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xe38e38e38e38e38f, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xe2c4a6886a4c2e11, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xe1fc780e1fc780e3, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xe135a9c97500e137, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xe070381c0e070383, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xdfac1f74346c5760, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xdee95c4ca037ba58, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xde27eb2c41f3d9d2, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xdd67c8a60dd67c8b, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xdca8f158c7f91ab9, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xdbeb61eed19c5959, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xdb2f171df770291a, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xda740da740da740f, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd9ba4256c0366e92, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd901b2036406c80f, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd84a598ec9151f44, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd79435e50d79435f, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd6df43fca482f00e, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd62b80d62b80d62d, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd578e97c3f5fe552, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd4c77b03531dec0e, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd4173289870ac52f, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd3680d3680d3680e, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd2ba083b445250ac, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd20d20d20d20d20e, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd161543e28e50275, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd0b69fcbd2580d0c, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xd00d00d00d00d00e, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xcf6474a8819ec8ea, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xcebcf8bb5b4169cc, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xce168a7725080ce2, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xcd712752a886d243, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xccccccccccccccce, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xcc29786c7607f9a0, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xcb8727c065c393e1, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xcae5d85f1bbd6c96, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xca4587e6b74f032a, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc9a633fcd967300e, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc907da4e871146ae, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc86a78900c86a78a, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc7ce0c7ce0c7ce0d, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc73293d789b9f839, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc6980c6980c6980d, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc5fe740317f9d00d, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc565c87b5f9d4d1d, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc4ce07b00c4ce07c, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc4372f855d824ca7, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc3a13de60495c774, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc30c30c30c30c30d, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc2780613c0309e03, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc1e4bbd595f6e948, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc152500c152500c2, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc0c0c0c0c0c0c0c2, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xc0300c0300c0300d, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xbfa02fe80bfa0300, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xbf112a8ad278e8de, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xbe82fa0be82fa0c0, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xbdf59c91700bdf5b, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xbd69104707661aa4, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xbcdd535db1cc5b7c, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xbc52640bc52640bd, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xbbc8408cd63069a2, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xbb3ee721a54d880d, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xbab656100bab6562, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xba2e8ba2e8ba2e8d, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb9a7862a0ff46589, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb92143fa36f5e02f, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb89bc36ce3e0453b, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb81702e05c0b8171, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb79300b79300b794, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb70fbb5a19be365a, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb68d31340e4307d9, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb60b60b60b60b60c, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb58a485518d1e7e5, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb509e68a9b948220, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb48a39d44685fe98, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb40b40b40b40b40c, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb38cf9b00b38cf9c, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb30f63528917c80c, lo: 0x0, ex: -1, sgn: 0x0 },
    Dint { hi: 0xb2927c29da5519d0, lo: 0x0, ex: -1, sgn: 0x0 },
];

#[rustfmt::skip]
static _LOG_INV_2: [Dint; 240] = [
    Dint { hi: 0xb17217f7d1cf79ab, lo: 0xc9e3b39803f2f6af, ex: -1, sgn: 0x1 },
    Dint { hi: 0xaf74155120c9011d, lo: 0x46d235ee63073dc, ex: -1, sgn: 0x1 },
    Dint { hi: 0xad7a02e1b24efd32, lo: 0x160864fd949b4bd3, ex: -1, sgn: 0x1 },
    Dint { hi: 0xab83d135dc633301, lo: 0xffe6607ba902ef3b, ex: -1, sgn: 0x1 },
    Dint { hi: 0xa991713433c2b999, lo: 0xba4aea614d05700, ex: -1, sgn: 0x1 },
    Dint { hi: 0xa7a2d41ad270c9d7, lo: 0xcd362382a7688479, ex: -1, sgn: 0x1 },
    Dint { hi: 0xa5b7eb7cb860fb89, lo: 0x7b6a62a0dec6e072, ex: -1, sgn: 0x1 },
    Dint { hi: 0xa3d0a93f45169a4b, lo: 0x9594fab088c0d64, ex: -1, sgn: 0x1 },
    Dint { hi: 0xa1ecff97c91e267b, lo: 0x1b7efae08e597e16, ex: -1, sgn: 0x1 },
    Dint { hi: 0xa00ce1092e5498c4, lo: 0x69879c5a30cd1241, ex: -1, sgn: 0x1 },
    Dint { hi: 0x9e304061b5fda91a, lo: 0x4603d87b6df81ac, ex: -1, sgn: 0x1 },
    Dint { hi: 0x9c5710b8cbb73a42, lo: 0xaa554b2dd4619e63, ex: -1, sgn: 0x1 },
    Dint { hi: 0x9a81456cec642e10, lo: 0x4d49f9aaea3cb5e0, ex: -1, sgn: 0x1 },
    Dint { hi: 0x98aed221a03458b6, lo: 0x732f89321647b358, ex: -1, sgn: 0x1 },
    Dint { hi: 0x96dfaabd86fa1647, lo: 0xd61188fbc94e2f14, ex: -1, sgn: 0x1 },
    Dint { hi: 0x9513c36876083696, lo: 0xb5cbc416a2418011, ex: -1, sgn: 0x1 },
    Dint { hi: 0x934b1089a6dc93c2, lo: 0xbf5bb3b60554e151, ex: -1, sgn: 0x1 },
    Dint { hi: 0x918586c5f5e4bf01, lo: 0x9f92199ed1a4bab0, ex: -1, sgn: 0x1 },
    Dint { hi: 0x8fc31afe30b2c6de, lo: 0xe300bf167e95da66, ex: -1, sgn: 0x1 },
    Dint { hi: 0x8e03c24d7300395a, lo: 0xcddae1ccce247837, ex: -1, sgn: 0x1 },
    Dint { hi: 0x8c47720791e53314, lo: 0x762ad19415fe25a5, ex: -1, sgn: 0x1 },
    Dint { hi: 0x8a8e1fb794b09134, lo: 0x9eb628dba173c82d, ex: -1, sgn: 0x1 },
    Dint { hi: 0x88d7c11e3ad53cdc, lo: 0x8a3111a707b6de2c, ex: -1, sgn: 0x1 },
    Dint { hi: 0x87244c308e670a66, lo: 0x85e005d06dbfa8f7, ex: -1, sgn: 0x1 },
    Dint { hi: 0x8573b71682a7d21b, lo: 0xb21f9f89c1ab80b2, ex: -1, sgn: 0x1 },
    Dint { hi: 0x83c5f8299e2b4091, lo: 0xb8f6fafe8fbb68b8, ex: -1, sgn: 0x1 },
    Dint { hi: 0x821b05f3b01d6774, lo: 0xdb0d58c3f7e2ea1e, ex: -1, sgn: 0x1 },
    Dint { hi: 0x8072d72d903d588c, lo: 0x7dd1b09c70c40109, ex: -1, sgn: 0x1 },
    Dint { hi: 0xfd9ac57bd2442180, lo: 0xaf05924d258c14c4, ex: -2, sgn: 0x1 },
    Dint { hi: 0xfa553f7018c966f4, lo: 0x2780a545a1b54dce, ex: -2, sgn: 0x1 },
    Dint { hi: 0xf7150ab5a09f27f6, lo: 0xa470250d40ebe8e, ex: -2, sgn: 0x1 },
    Dint { hi: 0xf3da161eed6b9ab1, lo: 0x248d42f78d3e65d2, ex: -2, sgn: 0x1 },
    Dint { hi: 0xf0a450d139366ca7, lo: 0x7c66eb6408ff6432, ex: -2, sgn: 0x1 },
    Dint { hi: 0xed73aa4264b0adeb, lo: 0x5391cf4b33e42996, ex: -2, sgn: 0x1 },
    Dint { hi: 0xea481236f7d35bb2, lo: 0x39a767a80d6d97e6, ex: -2, sgn: 0x1 },
    Dint { hi: 0xe72178c0323a1a0f, lo: 0xcc4e1653e71d9973, ex: -2, sgn: 0x1 },
    Dint { hi: 0xe3ffce3a2aa64923, lo: 0x8eadb651b49ac539, ex: -2, sgn: 0x1 },
    Dint { hi: 0xe0e30349fd1cec82, lo: 0x3e8e1802aba24d5, ex: -2, sgn: 0x1 },
    Dint { hi: 0xddcb08dc0717d85c, lo: 0x940a666c87842842, ex: -2, sgn: 0x1 },
    Dint { hi: 0xdab7d02231484a93, lo: 0xbec20cca6efe2ac4, ex: -2, sgn: 0x1 },
    Dint { hi: 0xd7a94a92466e833c, lo: 0xcd88bba7d0cee8df, ex: -2, sgn: 0x1 },
    Dint { hi: 0xd49f69e456cf1b7b, lo: 0x7f53bd2e406e66e6, ex: -2, sgn: 0x1 },
    Dint { hi: 0xd19a201127d3c646, lo: 0x279d79f51dcc7301, ex: -2, sgn: 0x1 },
    Dint { hi: 0xce995f50af69d863, lo: 0x432f3f4f861ad6a8, ex: -2, sgn: 0x1 },
    Dint { hi: 0xcb9d1a189ab56e77, lo: 0x7d7e9307c70c0667, ex: -2, sgn: 0x1 },
    Dint { hi: 0xc8a5431adfb44ca6, lo: 0x48ce7c1a75e341a, ex: -2, sgn: 0x1 },
    Dint { hi: 0xc5b1cd44596fa51f, lo: 0xf218fb8f9f9ef27f, ex: -2, sgn: 0x1 },
    Dint { hi: 0xc2c2abbb6e5fd570, lo: 0x3337789d592e296, ex: -2, sgn: 0x1 },
    Dint { hi: 0xbfd7d1dec0a8df70, lo: 0x37eda996244bccaf, ex: -2, sgn: 0x1 },
    Dint { hi: 0xbcf13343e7d9ec7f, lo: 0x2afd17781bb3afea, ex: -2, sgn: 0x1 },
    Dint { hi: 0xba0ec3b633dd8b0b, lo: 0x91dc60b2b059a609, ex: -2, sgn: 0x1 },
    Dint { hi: 0xb730773578cb90b3, lo: 0xaa1116c3466beb6c, ex: -2, sgn: 0x1 },
    Dint { hi: 0xb45641f4e350a0d4, lo: 0xe756eba00bc33976, ex: -2, sgn: 0x1 },
    Dint { hi: 0xb1801859d56249de, lo: 0x98ce51fff99479cb, ex: -2, sgn: 0x1 },
    Dint { hi: 0xaeadeefacaf97d37, lo: 0x9dd6e688ebb13b01, ex: -2, sgn: 0x1 },
    Dint { hi: 0xabdfba9e468fd6f9, lo: 0x472ea07749ce6bd1, ex: -2, sgn: 0x1 },
    Dint { hi: 0xa9157039c51ebe72, lo: 0xe164c759686a2207, ex: -2, sgn: 0x1 },
    Dint { hi: 0xa64f04f0b961df78, lo: 0x54f5275c2d15c21e, ex: -2, sgn: 0x1 },
    Dint { hi: 0xa38c6e138e20d834, lo: 0xd698298adddd7f30, ex: -2, sgn: 0x1 },
    Dint { hi: 0xa0cda11eaf46390e, lo: 0x632438273918db7d, ex: -2, sgn: 0x1 },
    Dint { hi: 0x9e1293b9998c1dad, lo: 0x3b035eae273a855c, ex: -2, sgn: 0x1 },
    Dint { hi: 0x9b5b3bb5f088b768, lo: 0x5078bbe3d392be24, ex: -2, sgn: 0x1 },
    Dint { hi: 0x98a78f0e9ae71d87, lo: 0x64dec34784707838, ex: -2, sgn: 0x1 },
    Dint { hi: 0x95f783e6e49a9cfc, lo: 0x25004f3ef063312, ex: -2, sgn: 0x1 },
    Dint { hi: 0x934b1089a6dc93c2, lo: 0xdf5bb3b60554e151, ex: -2, sgn: 0x1 },
    Dint { hi: 0x90a22b6875c6a1f8, lo: 0x8e91aeba609c8876, ex: -2, sgn: 0x1 },
    Dint { hi: 0x8dfccb1ad35ca6ef, lo: 0x9947bdb6ddcaf59a, ex: -2, sgn: 0x1 },
    Dint { hi: 0x8b5ae65d67db9acf, lo: 0x7ba5168126a58b99, ex: -2, sgn: 0x1 },
    Dint { hi: 0x88bc74113f23def3, lo: 0xbc5a0fe396f40f1c, ex: -2, sgn: 0x1 },
    Dint { hi: 0x86216b3b0b17188c, lo: 0x363ceae88f720f1d, ex: -2, sgn: 0x1 },
    Dint { hi: 0x8389c3026ac3139d, lo: 0x6adda9d2270fa1f3, ex: -2, sgn: 0x1 },
    Dint { hi: 0x80f572b1363487bc, lo: 0xedbd0b5b3479d5f2, ex: -2, sgn: 0x1 },
    Dint { hi: 0xfcc8e3659d9bcbf1, lo: 0x8a0cdf301431b60b, ex: -3, sgn: 0x1 },
    Dint { hi: 0xf7ad6f26e7ff2efc, lo: 0x9cd2238f75f969ad, ex: -3, sgn: 0x1 },
    Dint { hi: 0xf29877ff38809097, lo: 0x2b020fa1820c948d, ex: -3, sgn: 0x1 },
    Dint { hi: 0xed89ed86a44a01ab, lo: 0x9d49f96cb88317a, ex: -3, sgn: 0x1 },
    Dint { hi: 0xe881bf932af3dac3, lo: 0x2524848e3443e03f, ex: -3, sgn: 0x1 },
    Dint { hi: 0xe37fde37807b84e3, lo: 0x5e9a750b6b68781c, ex: -3, sgn: 0x1 },
    Dint { hi: 0xde8439c1dec5687c, lo: 0x9d57da945b5d0aa6, ex: -3, sgn: 0x1 },
    Dint { hi: 0xd98ec2bade71e53e, lo: 0xd0a98f2ad65bee96, ex: -3, sgn: 0x1 },
    Dint { hi: 0xd49f69e456cf1b7a, lo: 0x5f53bd2e406e66e7, ex: -3, sgn: 0x1 },
    Dint { hi: 0xcfb6203844b3209b, lo: 0x18cb02f33f79c16b, ex: -3, sgn: 0x1 },
    Dint { hi: 0xcad2d6e7b80bf915, lo: 0xcc507fb7a3d0bf69, ex: -3, sgn: 0x1 },
    Dint { hi: 0xc5f57f59c7f46156, lo: 0x9a8b6997a402bf30, ex: -3, sgn: 0x1 },
    Dint { hi: 0xc11e0b2a8d1e0de1, lo: 0xda631e830fd308fe, ex: -3, sgn: 0x1 },
    Dint { hi: 0xbc4c6c2a226399f6, lo: 0x276ebcfb2016a433, ex: -3, sgn: 0x1 },
    Dint { hi: 0xb780945bab55dcea, lo: 0xb4c7bc3d32750fd9, ex: -3, sgn: 0x1 },
    Dint { hi: 0xb2ba75f46099cf8f, lo: 0x243c2e77904afa76, ex: -3, sgn: 0x1 },
    Dint { hi: 0xadfa035aa1ed8fdd, lo: 0x549767e410316d2b, ex: -3, sgn: 0x1 },
    Dint { hi: 0xa93f2f250dac67d5, lo: 0x9ad2fb8d48054add, ex: -3, sgn: 0x1 },
    Dint { hi: 0xa489ec199dab06f4, lo: 0x59fb6cf0ecb411b7, ex: -3, sgn: 0x1 },
    Dint { hi: 0x9fda2d2cc9465c52, lo: 0x6b2b9565f5355180, ex: -3, sgn: 0x1 },
    Dint { hi: 0x9b2fe580ac80b182, lo: 0x11a5b944aca8705, ex: -3, sgn: 0x1 },
    Dint { hi: 0x968b08643409ceb9, lo: 0xd5c0da506a088482, ex: -3, sgn: 0x1 },
    Dint { hi: 0x91eb89524e100d28, lo: 0xbfd3df5c52d67e77, ex: -3, sgn: 0x1 },
    Dint { hi: 0x8d515bf11fb94f22, lo: 0xa0713268840cbcbb, ex: -3, sgn: 0x1 },
    Dint { hi: 0x88bc74113f23def7, lo: 0x9c5a0fe396f40f19, ex: -3, sgn: 0x1 },
    Dint { hi: 0x842cc5acf1d0344b, lo: 0x6fecdfa819b96092, ex: -3, sgn: 0x1 },
    Dint { hi: 0xff4489cedeab2ca6, lo: 0xe17bd40d8d9291ec, ex: -4, sgn: 0x1 },
    Dint { hi: 0xf639cc185088fe62, lo: 0x5066e87f2c0f733d, ex: -4, sgn: 0x1 },
    Dint { hi: 0xed393b1c22351281, lo: 0xff4e2e660317d55f, ex: -4, sgn: 0x1 },
    Dint { hi: 0xe442c00de2591b4c, lo: 0xe96ab34ce0bccd10, ex: -4, sgn: 0x1 },
    Dint { hi: 0xdb56446d6ad8df09, lo: 0x28112e35a60e636f, ex: -4, sgn: 0x1 },
    Dint { hi: 0xd273b2058de1bd4b, lo: 0x36bbf837b4d320c6, ex: -4, sgn: 0x1 },
    Dint { hi: 0xc99af2eaca4c457b, lo: 0xeaf51f66692844b2, ex: -4, sgn: 0x1 },
    Dint { hi: 0xc0cbf17a071f80e9, lo: 0x396ffdf76a147cc2, ex: -4, sgn: 0x1 },
    Dint { hi: 0xb8069857560707a7, lo: 0xa677b4c8bec22e0, ex: -4, sgn: 0x1 },
    Dint { hi: 0xaf4ad26cbc8e5bef, lo: 0x9e8b8b88a14ff0c9, ex: -4, sgn: 0x1 },
    Dint { hi: 0xa6988ae903f562f1, lo: 0x7e858f08597b3a68, ex: -4, sgn: 0x1 },
    Dint { hi: 0x9defad3e8f732186, lo: 0x476d3b5b45f6ca02, ex: -4, sgn: 0x1 },
    Dint { hi: 0x9550252238bd2468, lo: 0x658e5a0b811c596d, ex: -4, sgn: 0x1 },
    Dint { hi: 0x8cb9de8a32ab3694, lo: 0x97c9859530a4514c, ex: -4, sgn: 0x1 },
    Dint { hi: 0x842cc5acf1d0344c, lo: 0x1fecdfa819b96094, ex: -4, sgn: 0x1 },
    Dint { hi: 0xf7518e0035c3dd92, lo: 0x606d89093278a931, ex: -5, sgn: 0x1 },
    Dint { hi: 0xe65b9e6eed965c4f, lo: 0x609f5fe2058d5ff2, ex: -5, sgn: 0x1 },
    Dint { hi: 0xd5779687d887e0ee, lo: 0x49dda17056e45ebb, ex: -5, sgn: 0x1 },
    Dint { hi: 0xc4a550a4fd9a19bb, lo: 0x3e97660a23cc5402, ex: -5, sgn: 0x1 },
    Dint { hi: 0xb3e4a796a5dac213, lo: 0x7cca0bcc06c2f8e, ex: -5, sgn: 0x1 },
    Dint { hi: 0xa33576a16f1f4c79, lo: 0x121016bd904dc95a, ex: -5, sgn: 0x1 },
    Dint { hi: 0x9297997c68c1f4e6, lo: 0x610db3d4dd423bc9, ex: -5, sgn: 0x1 },
    Dint { hi: 0x820aec4f3a222397, lo: 0xb9e3aea6c444eef6, ex: -5, sgn: 0x1 },
    Dint { hi: 0xe31e9760a5578c6d, lo: 0xf9eb2f284f31c35a, ex: -6, sgn: 0x1 },
    Dint { hi: 0xc24929464655f482, lo: 0xda5f3cc0b3251da6, ex: -6, sgn: 0x1 },
    Dint { hi: 0xa195492cc0660519, lo: 0x4a18dff7cdb4ae33, ex: -6, sgn: 0x1 },
    Dint { hi: 0x8102b2c49ac23a86, lo: 0x91d082dce3ddcd08, ex: -6, sgn: 0x1 },
    Dint { hi: 0xc122451c45155150, lo: 0xb16137f09a002b0e, ex: -7, sgn: 0x1 },
    Dint { hi: 0x8080abac46f389c4, lo: 0x662d417ced0079c9, ex: -7, sgn: 0x1 },
    Dint { hi: 0x0, lo: 0x0, ex: 127, sgn: 0x0 },
    Dint { hi: 0x0, lo: 0x0, ex: 127, sgn: 0x0 },
    Dint { hi: 0xff805515885e014e, lo: 0x435ab4da6a5bb50f, ex: -9, sgn: 0x0 },
    Dint { hi: 0xff015358833c4762, lo: 0xbb481c8ee1416999, ex: -8, sgn: 0x0 },
    Dint { hi: 0xbee23afc0853b6a8, lo: 0xa89782c20df350c2, ex: -7, sgn: 0x0 },
    Dint { hi: 0xfe054587e01f1e2b, lo: 0xf6d3a69bd5eab72f, ex: -7, sgn: 0x0 },
    Dint { hi: 0x9e75221a352ba751, lo: 0x452b7ea62f2198ea, ex: -6, sgn: 0x0 },
    Dint { hi: 0xbdc8d83ead88d518, lo: 0x7faa638b5e00ee90, ex: -6, sgn: 0x0 },
    Dint { hi: 0xdcfe013d7c8cbfc5, lo: 0x632dbac46f30d009, ex: -6, sgn: 0x0 },
    Dint { hi: 0xfc14d873c1980236, lo: 0xc7e09e3de453f5fc, ex: -6, sgn: 0x0 },
    Dint { hi: 0x8d86cc491ecbfe03, lo: 0xf1776453b7e82558, ex: -5, sgn: 0x0 },
    Dint { hi: 0x9cf43dcff5eafd2f, lo: 0x2ad90155c8a7236a, ex: -5, sgn: 0x0 },
    Dint { hi: 0xac52dd7e4726a456, lo: 0xa47a963a91bb3018, ex: -5, sgn: 0x0 },
    Dint { hi: 0xbba2c7b196e7e224, lo: 0xe7950f7252c163cf, ex: -5, sgn: 0x0 },
    Dint { hi: 0xcae41876471f5bde, lo: 0x91d00a417e330f8e, ex: -5, sgn: 0x0 },
    Dint { hi: 0xda16eb88cb8df5fb, lo: 0x28a63ecfb66e94c0, ex: -5, sgn: 0x0 },
    Dint { hi: 0xe93b5c56d85a9083, lo: 0xce2992bfea38e76b, ex: -5, sgn: 0x0 },
    Dint { hi: 0xf85186008b1532f9, lo: 0xe64b8b7759978998, ex: -5, sgn: 0x0 },
    Dint { hi: 0x83acc1acc7238978, lo: 0x5a5333c45b7f442e, ex: -4, sgn: 0x0 },
    Dint { hi: 0x8b29b7751bd7073b, lo: 0x2e0b9ee992f2372, ex: -4, sgn: 0x0 },
    Dint { hi: 0x929fb17850a0b7be, lo: 0x5b4d3807660516a4, ex: -4, sgn: 0x0 },
    Dint { hi: 0x9a0ebcb0de8e848e, lo: 0x2c1bb082689ba814, ex: -4, sgn: 0x0 },
    Dint { hi: 0xa176e5f5323781d2, lo: 0xdcf935996c92e8d4, ex: -4, sgn: 0x0 },
    Dint { hi: 0xa8d839f830c1fb40, lo: 0x4c7343517c8ac264, ex: -4, sgn: 0x0 },
    Dint { hi: 0xb032c549ba861d83, lo: 0x774e27bc92ce3373, ex: -4, sgn: 0x0 },
    Dint { hi: 0xb78694572b5a5cd3, lo: 0x24cdcf68cdb2067c, ex: -4, sgn: 0x0 },
    Dint { hi: 0xbed3b36bd8966419, lo: 0x7c0644d7d9ed08b4, ex: -4, sgn: 0x0 },
    Dint { hi: 0xc61a2eb18cd907a1, lo: 0xe5a1532f6d5a1ac1, ex: -4, sgn: 0x0 },
    Dint { hi: 0xcd5a1231019d66d7, lo: 0x761e3e7b171e44b2, ex: -4, sgn: 0x0 },
    Dint { hi: 0xd49369d256ab1b1f, lo: 0x9e9154e1d5263cda, ex: -4, sgn: 0x0 },
    Dint { hi: 0xdbc6415d876d0839, lo: 0x3e33c0c9f8824f54, ex: -4, sgn: 0x0 },
    Dint { hi: 0xe2f2a47ade3a18a8, lo: 0xa0bf7c0b0d8bb4ef, ex: -4, sgn: 0x0 },
    Dint { hi: 0xea189eb3659aeaeb, lo: 0x93b2a3b21f448259, ex: -4, sgn: 0x0 },
    Dint { hi: 0xf1383b7157972f48, lo: 0x543fff0ff4f0aaf1, ex: -4, sgn: 0x0 },
    Dint { hi: 0xf85186008b153302, lo: 0x5e4b8b7759978993, ex: -4, sgn: 0x0 },
    Dint { hi: 0xff64898edf55d548, lo: 0x428ccfc99271dffa, ex: -4, sgn: 0x0 },
    Dint { hi: 0x8338a89652cb714a, lo: 0xb247eb86498c2ce7, ex: -3, sgn: 0x0 },
    Dint { hi: 0x86bbf3e68472cb2f, lo: 0xb8bd20615747126, ex: -3, sgn: 0x0 },
    Dint { hi: 0x8a3c2c233a156341, lo: 0x9027c74fe0e6f64f, ex: -3, sgn: 0x0 },
    Dint { hi: 0x8db956a97b3d0143, lo: 0xf023472cd739f9e1, ex: -3, sgn: 0x0 },
    Dint { hi: 0x913378c852d65be6, lo: 0x977e3013d10f7525, ex: -3, sgn: 0x0 },
    Dint { hi: 0x94aa97c0ffa91a5d, lo: 0x4ee3880fb7d34429, ex: -3, sgn: 0x0 },
    Dint { hi: 0x981eb8c723fe97f2, lo: 0x1f1c134fb702d433, ex: -3, sgn: 0x0 },
    Dint { hi: 0x9b8fe100f47ba1d8, lo: 0x4b62af189fcba0d, ex: -3, sgn: 0x0 },
    Dint { hi: 0x9efe158766314e4f, lo: 0x4d71827efe892fc8, ex: -3, sgn: 0x0 },
    Dint { hi: 0xa2695b665be8f338, lo: 0x4eca87c3f0f06211, ex: -3, sgn: 0x0 },
    Dint { hi: 0xa5d1b79cd2af2aca, lo: 0x8837986ceabfbed6, ex: -3, sgn: 0x0 },
    Dint { hi: 0xa9372f1d0da1bd10, lo: 0x580eb71e58cd36e5, ex: -3, sgn: 0x0 },
    Dint { hi: 0xac99c6ccc1042e94, lo: 0x3dd557528315838d, ex: -3, sgn: 0x0 },
    Dint { hi: 0xaff983853c9e9e40, lo: 0x5f105039091dd7f5, ex: -3, sgn: 0x0 },
    Dint { hi: 0xb3566a13956a86f4, lo: 0x471b1e1574d9fd55, ex: -3, sgn: 0x0 },
    Dint { hi: 0xb6b07f38ce90e463, lo: 0x7bb2e265d0de37e1, ex: -3, sgn: 0x0 },
    Dint { hi: 0xba07c7aa01bd2648, lo: 0x43f9d57b324bd05f, ex: -3, sgn: 0x0 },
    Dint { hi: 0xbd5c481086c848db, lo: 0xbb596b5030403242, ex: -3, sgn: 0x0 },
    Dint { hi: 0xc0ae050a1abf56ad, lo: 0x2f7f8c5fa9c50d76, ex: -3, sgn: 0x0 },
    Dint { hi: 0xc3fd03290648847d, lo: 0x30480bee4cbbd698, ex: -3, sgn: 0x0 },
    Dint { hi: 0xc74946f4436a054e, lo: 0xf4f5cb531201c0d3, ex: -3, sgn: 0x0 },
    Dint { hi: 0xca92d4e7a2b5a3ad, lo: 0xc983a9c5c4b3b135, ex: -3, sgn: 0x0 },
    Dint { hi: 0xcdd9b173efdc1aaa, lo: 0x8863e007c184a1e7, ex: -3, sgn: 0x0 },
    Dint { hi: 0xd11de0ff15ab18c6, lo: 0xd88d83d4cc613f21, ex: -3, sgn: 0x0 },
    Dint { hi: 0xd45f67e44178c612, lo: 0x5486e73c615158b4, ex: -3, sgn: 0x0 },
    Dint { hi: 0xd79e4a7405ff96c3, lo: 0x1300c9be67ae5da0, ex: -3, sgn: 0x0 },
    Dint { hi: 0xdada8cf47dad236d, lo: 0xdffb833c3409ee7e, ex: -3, sgn: 0x0 },
    Dint { hi: 0xde1433a16c66b14c, lo: 0xde744870f54f0f18, ex: -3, sgn: 0x0 },
    Dint { hi: 0xe14b42ac60c60512, lo: 0x4e38eb8092a01f06, ex: -3, sgn: 0x0 },
    Dint { hi: 0xe47fbe3cd4d10d5b, lo: 0x2ec0f797fdcd125c, ex: -3, sgn: 0x0 },
    Dint { hi: 0xe7b1aa704e2ee240, lo: 0xb40faab6d2ad0841, ex: -3, sgn: 0x0 },
    Dint { hi: 0xeae10b5a7ddc8ad8, lo: 0x806b2fc9a8038790, ex: -3, sgn: 0x0 },
    Dint { hi: 0xee0de5055f63eb01, lo: 0x90a33316df83ba5a, ex: -3, sgn: 0x0 },
    Dint { hi: 0xf1383b7157972f4a, lo: 0xb43fff0ff4f0aaf1, ex: -3, sgn: 0x0 },
    Dint { hi: 0xf460129552d2ff41, lo: 0xe62e3201bb2bbdce, ex: -3, sgn: 0x0 },
    Dint { hi: 0xf7856e5ee2c9b28a, lo: 0x76f2a1b84190a7dc, ex: -3, sgn: 0x0 },
    Dint { hi: 0xfaa852b25bd9b833, lo: 0xa6dbfa03186e0666, ex: -3, sgn: 0x0 },
    Dint { hi: 0xfdc8c36af1f15468, lo: 0xa3361bca696504a, ex: -3, sgn: 0x0 },
    Dint { hi: 0x8073622d6a80e631, lo: 0xe897009015316073, ex: -2, sgn: 0x0 },
    Dint { hi: 0x82012ca5a68206d5, lo: 0x8fde85afdd2bc88a, ex: -2, sgn: 0x0 },
    Dint { hi: 0x838dc2fe6ac868e7, lo: 0x1a3fcbdef40100cb, ex: -2, sgn: 0x0 },
    Dint { hi: 0x851927139c871af8, lo: 0x67bd00c38061c51f, ex: -2, sgn: 0x0 },
    Dint { hi: 0x86a35abcd5ba5901, lo: 0x5481c3cbd925ccd2, ex: -2, sgn: 0x0 },
    Dint { hi: 0x882c5fcd7256a8c1, lo: 0x39055a6598e7c29e, ex: -2, sgn: 0x0 },
    Dint { hi: 0x89b438149d4582f5, lo: 0x34531dba493eb5a6, ex: -2, sgn: 0x0 },
    Dint { hi: 0x8b3ae55d5d30701a, lo: 0xc63eab8837170480, ex: -2, sgn: 0x0 },
    Dint { hi: 0x8cc0696ea11b7b36, lo: 0x94361c9a28d38a6a, ex: -2, sgn: 0x0 },
    Dint { hi: 0x8e44c60b4ccfd7dc, lo: 0x1473aa01c7778679, ex: -2, sgn: 0x0 },
    Dint { hi: 0x8fc7fcf24517946a, lo: 0x380cbe769f2c6793, ex: -2, sgn: 0x0 },
    Dint { hi: 0x914a0fde7bcb2d0e, lo: 0xc429ed3aea197a60, ex: -2, sgn: 0x0 },
    Dint { hi: 0x92cb0086fbb1cf75, lo: 0xa29d47c50b1182d0, ex: -2, sgn: 0x0 },
    Dint { hi: 0x944ad09ef4351af1, lo: 0xa49827e081cb16ba, ex: -2, sgn: 0x0 },
    Dint { hi: 0x95c981d5c4e924ea, lo: 0x45404f5aa577d6b4, ex: -2, sgn: 0x0 },
    Dint { hi: 0x974715d708e984dd, lo: 0x6648d42840d9e6fb, ex: -2, sgn: 0x0 },
    Dint { hi: 0x98c38e4aa20c27d2, lo: 0x846767ec990d7333, ex: -2, sgn: 0x0 },
    Dint { hi: 0x9a3eecd4c3eaa6ae, lo: 0xdb3a7f6e6087b947, ex: -2, sgn: 0x0 },
    Dint { hi: 0x9bb93315fec2d790, lo: 0x7f589fba0865790f, ex: -2, sgn: 0x0 },
    Dint { hi: 0x9d3262ab4a2f4e37, lo: 0xa1ae6ba06846fae0, ex: -2, sgn: 0x0 },
    Dint { hi: 0x9eaa7d2e0fb87c35, lo: 0xff472bc6ce648a7d, ex: -2, sgn: 0x0 },
    Dint { hi: 0xa0218434353f1de4, lo: 0xd493efa632530acc, ex: -2, sgn: 0x0 },
    Dint { hi: 0xa197795027409daa, lo: 0x1dd1d4a6df960357, ex: -2, sgn: 0x0 },
    Dint { hi: 0xa30c5e10e2f613e4, lo: 0x9bd9bd99e39a20b3, ex: -2, sgn: 0x0 },
    Dint { hi: 0xa4803402004e865c, lo: 0x31cbe0e8824116cd, ex: -2, sgn: 0x0 },
    Dint { hi: 0xa5f2fcabbbc506d8, lo: 0x68ca4fb7ec323d74, ex: -2, sgn: 0x0 },
    Dint { hi: 0xa764b99300134d79, lo: 0xd04d10474301862, ex: -2, sgn: 0x0 },
    Dint { hi: 0xa8d56c396fc1684c, lo: 0x1eb067d578c4756, ex: -2, sgn: 0x0 },
    Dint { hi: 0xaa45161d6e93167b, lo: 0x9b081cf72249f5b2, ex: -2, sgn: 0x0 },
    Dint { hi: 0xabb3b8ba2ad362a1, lo: 0x1db6506cc17a01f5, ex: -2, sgn: 0x0 },
    Dint { hi: 0xad215587a67f0cdf, lo: 0xe890422cb86b7cb1, ex: -2, sgn: 0x0 },
    Dint { hi: 0xae8dedfac04e5282, lo: 0xac707b8ffc22b3e8, ex: -2, sgn: 0x0 },
    Dint { hi: 0xaff983853c9e9e3f, lo: 0xc5105039091dd7f8, ex: -2, sgn: 0x0 },
    Dint { hi: 0xb1641795ce3ca978, lo: 0xfaf915300e517393, ex: -2, sgn: 0x0 },
    Dint { hi: 0xb2cdab981f0f940b, lo: 0xc857c77dc1df600f, ex: -2, sgn: 0x0 },
    Dint { hi: 0xb43640f4d8a5761f, lo: 0xf5f080a71c34b25d, ex: -2, sgn: 0x0 },
    Dint { hi: 0xb59dd911aca1ec48, lo: 0x1d2664cf09a0c1bf, ex: -2, sgn: 0x0 },
    Dint { hi: 0xb70475515d0f1c5e, lo: 0x4c98c6b8be17818d, ex: -2, sgn: 0x0 },
    Dint { hi: 0xb86a1713c491aeaa, lo: 0xd37ee2872a6f1cd6, ex: -2, sgn: 0x0 },
];

#[rustfmt::skip]
const P_2: [Dint; 13] = [
    Dint { hi: 0x99df88a0430813ca, lo: 0xa1cffb6e966a70f6, ex: -4, sgn: 0x0 },
    Dint { hi: 0xaaa02d43f696c3e4, lo: 0x4dbe754667b6bc48, ex: -4, sgn: 0x1 },
    Dint { hi: 0xba2e7a1eaf856174, lo: 0x70e5c5a5ebbe0226, ex: -4, sgn: 0x0 },
    Dint { hi: 0xccccccb9ec017492, lo: 0xf934e28d924e76d4, ex: -4, sgn: 0x1 },
    Dint { hi: 0xe38e38e3807cfa4b, lo: 0xc976e6cbd22e203f, ex: -4, sgn: 0x0 },
    Dint { hi: 0xfffffffffff924cc, lo: 0x5b308e39fa7dfb5, ex: -4, sgn: 0x1 },
    Dint { hi: 0x924924924924911d, lo: 0x862bc3d33abb3649, ex: -3, sgn: 0x0 },
    Dint { hi: 0xaaaaaaaaaaaaaaaa, lo: 0x6637fd4b19743eec, ex: -3, sgn: 0x1 },
    Dint { hi: 0xcccccccccccccccc, lo: 0xccc2ca18b08fe343, ex: -3, sgn: 0x0 },
    Dint { hi: 0xffffffffffffffff, lo: 0xffffff2245823ae0, ex: -3, sgn: 0x1 },
    Dint { hi: 0xaaaaaaaaaaaaaaaa, lo: 0xaaaaaaaaa5c48b54, ex: -2, sgn: 0x0 },
    Dint { hi: 0xffffffffffffffff, lo: 0xffffffffffffebd8, ex: -2, sgn: 0x1 },
    Dint { hi: 0x8000000000000000, lo: 0x0, ex: 0, sgn: 0x0 },
];

// For 362 <= i <= 724, r[i] = _INVERSE[i-362] is a 10-bit approximation of
// 1/x[i], where i*2^-9 <= x[i] < (i+1)*2^-9.
// More precisely r[i] is a 10-bit value such that r[i]*y-1 is representable
// exactly on 53 bits for any y, i*2^-9 <= y < (i+1)*2^-9.
// Moreover |r[i]*y-1| <= 0.00212097167968735.
#[rustfmt::skip]
static _INVERSE: [f64; 363] = [
    hf64!("0x1.698p+0"), hf64!("0x1.688p+0"), hf64!("0x1.678p+0"), hf64!("0x1.668p+0"), hf64!("0x1.658p+0"), hf64!("0x1.648p+0"), hf64!("0x1.638p+0"),
    hf64!("0x1.63p+0"), hf64!("0x1.62p+0"), hf64!("0x1.61p+0"), hf64!("0x1.6p+0"), hf64!("0x1.5fp+0"), hf64!("0x1.5ep+0"), hf64!("0x1.5dp+0"),
    hf64!("0x1.5cp+0"), hf64!("0x1.5bp+0"), hf64!("0x1.5a8p+0"), hf64!("0x1.598p+0"), hf64!("0x1.588p+0"), hf64!("0x1.578p+0"), hf64!("0x1.568p+0"),
    hf64!("0x1.56p+0"), hf64!("0x1.55p+0"), hf64!("0x1.54p+0"), hf64!("0x1.53p+0"), hf64!("0x1.52p+0"), hf64!("0x1.518p+0"), hf64!("0x1.508p+0"),
    hf64!("0x1.4f8p+0"), hf64!("0x1.4fp+0"), hf64!("0x1.4ep+0"), hf64!("0x1.4dp+0"), hf64!("0x1.4cp+0"), hf64!("0x1.4b8p+0"), hf64!("0x1.4a8p+0"),
    hf64!("0x1.4ap+0"), hf64!("0x1.49p+0"), hf64!("0x1.48p+0"), hf64!("0x1.478p+0"), hf64!("0x1.468p+0"), hf64!("0x1.458p+0"), hf64!("0x1.45p+0"),
    hf64!("0x1.44p+0"), hf64!("0x1.43p+0"), hf64!("0x1.428p+0"), hf64!("0x1.418p+0"), hf64!("0x1.41p+0"), hf64!("0x1.4p+0"), hf64!("0x1.3f8p+0"),
    hf64!("0x1.3e8p+0"), hf64!("0x1.3ep+0"), hf64!("0x1.3dp+0"), hf64!("0x1.3cp+0"), hf64!("0x1.3b8p+0"), hf64!("0x1.3a8p+0"), hf64!("0x1.3ap+0"),
    hf64!("0x1.39p+0"), hf64!("0x1.388p+0"), hf64!("0x1.378p+0"), hf64!("0x1.37p+0"), hf64!("0x1.36p+0"), hf64!("0x1.358p+0"), hf64!("0x1.35p+0"),
    hf64!("0x1.34p+0"), hf64!("0x1.338p+0"), hf64!("0x1.328p+0"), hf64!("0x1.32p+0"), hf64!("0x1.31p+0"), hf64!("0x1.308p+0"), hf64!("0x1.3p+0"),
    hf64!("0x1.2fp+0"), hf64!("0x1.2e8p+0"), hf64!("0x1.2d8p+0"), hf64!("0x1.2dp+0"), hf64!("0x1.2c8p+0"), hf64!("0x1.2b8p+0"), hf64!("0x1.2bp+0"),
    hf64!("0x1.2ap+0"), hf64!("0x1.298p+0"), hf64!("0x1.29p+0"), hf64!("0x1.28p+0"), hf64!("0x1.278p+0"), hf64!("0x1.27p+0"), hf64!("0x1.26p+0"),
    hf64!("0x1.258p+0"), hf64!("0x1.25p+0"), hf64!("0x1.24p+0"), hf64!("0x1.238p+0"), hf64!("0x1.23p+0"), hf64!("0x1.228p+0"), hf64!("0x1.218p+0"),
    hf64!("0x1.21p+0"), hf64!("0x1.208p+0"), hf64!("0x1.2p+0"), hf64!("0x1.1fp+0"), hf64!("0x1.1e8p+0"), hf64!("0x1.1ep+0"), hf64!("0x1.1dp+0"),
    hf64!("0x1.1c8p+0"), hf64!("0x1.1cp+0"), hf64!("0x1.1b8p+0"), hf64!("0x1.1bp+0"), hf64!("0x1.1ap+0"), hf64!("0x1.198p+0"), hf64!("0x1.19p+0"),
    hf64!("0x1.188p+0"), hf64!("0x1.18p+0"), hf64!("0x1.17p+0"), hf64!("0x1.168p+0"), hf64!("0x1.16p+0"), hf64!("0x1.158p+0"), hf64!("0x1.15p+0"),
    hf64!("0x1.14p+0"), hf64!("0x1.138p+0"), hf64!("0x1.13p+0"), hf64!("0x1.128p+0"), hf64!("0x1.12p+0"), hf64!("0x1.118p+0"), hf64!("0x1.11p+0"),
    hf64!("0x1.1p+0"), hf64!("0x1.0f8p+0"), hf64!("0x1.0fp+0"), hf64!("0x1.0e8p+0"), hf64!("0x1.0ep+0"), hf64!("0x1.0d8p+0"), hf64!("0x1.0dp+0"),
    hf64!("0x1.0c8p+0"), hf64!("0x1.0cp+0"), hf64!("0x1.0bp+0"), hf64!("0x1.0a8p+0"), hf64!("0x1.0ap+0"), hf64!("0x1.098p+0"), hf64!("0x1.09p+0"),
    hf64!("0x1.088p+0"), hf64!("0x1.08p+0"), hf64!("0x1.078p+0"), hf64!("0x1.07p+0"), hf64!("0x1.068p+0"), hf64!("0x1.06p+0"), hf64!("0x1.058p+0"),
    hf64!("0x1.05p+0"), hf64!("0x1.048p+0"), hf64!("0x1.04p+0"), hf64!("0x1.038p+0"), hf64!("0x1.03p+0"), hf64!("0x1.028p+0"), hf64!("0x1.02p+0"),
    hf64!("0x1.018p+0"), hf64!("0x1.01p+0"), hf64!("0x1.008p+0"), hf64!("0x1.ff8p-1"), hf64!("0x1.fe8p-1"), hf64!("0x1.fd8p-1"), hf64!("0x1.fc8p-1"),
    hf64!("0x1.fb8p-1"), hf64!("0x1.fa8p-1"), hf64!("0x1.f98p-1"), hf64!("0x1.f88p-1"), hf64!("0x1.f78p-1"), hf64!("0x1.f68p-1"), hf64!("0x1.f58p-1"),
    hf64!("0x1.f5p-1"), hf64!("0x1.f4p-1"), hf64!("0x1.f3p-1"), hf64!("0x1.f2p-1"), hf64!("0x1.f1p-1"), hf64!("0x1.fp-1"), hf64!("0x1.efp-1"),
    hf64!("0x1.eep-1"), hf64!("0x1.edp-1"), hf64!("0x1.ec8p-1"), hf64!("0x1.eb8p-1"), hf64!("0x1.ea8p-1"), hf64!("0x1.e98p-1"), hf64!("0x1.e88p-1"),
    hf64!("0x1.e78p-1"), hf64!("0x1.e7p-1"), hf64!("0x1.e6p-1"), hf64!("0x1.e5p-1"), hf64!("0x1.e4p-1"), hf64!("0x1.e3p-1"), hf64!("0x1.e28p-1"),
    hf64!("0x1.e18p-1"), hf64!("0x1.e08p-1"), hf64!("0x1.df8p-1"), hf64!("0x1.dfp-1"), hf64!("0x1.dep-1"), hf64!("0x1.ddp-1"), hf64!("0x1.dcp-1"),
    hf64!("0x1.db8p-1"), hf64!("0x1.da8p-1"), hf64!("0x1.d98p-1"), hf64!("0x1.d9p-1"), hf64!("0x1.d8p-1"), hf64!("0x1.d7p-1"), hf64!("0x1.d6p-1"),
    hf64!("0x1.d58p-1"), hf64!("0x1.d48p-1"), hf64!("0x1.d38p-1"), hf64!("0x1.d3p-1"), hf64!("0x1.d2p-1"), hf64!("0x1.d1p-1"), hf64!("0x1.d08p-1"),
    hf64!("0x1.cf8p-1"), hf64!("0x1.ce8p-1"), hf64!("0x1.cep-1"), hf64!("0x1.cdp-1"), hf64!("0x1.cc8p-1"), hf64!("0x1.cb8p-1"), hf64!("0x1.ca8p-1"),
    hf64!("0x1.cap-1"), hf64!("0x1.c9p-1"), hf64!("0x1.c88p-1"), hf64!("0x1.c78p-1"), hf64!("0x1.c68p-1"), hf64!("0x1.c6p-1"), hf64!("0x1.c5p-1"),
    hf64!("0x1.c48p-1"), hf64!("0x1.c38p-1"), hf64!("0x1.c3p-1"), hf64!("0x1.c2p-1"), hf64!("0x1.c18p-1"), hf64!("0x1.c08p-1"), hf64!("0x1.bf8p-1"),
    hf64!("0x1.bfp-1"), hf64!("0x1.bep-1"), hf64!("0x1.bd8p-1"), hf64!("0x1.bc8p-1"), hf64!("0x1.bcp-1"), hf64!("0x1.bbp-1"), hf64!("0x1.ba8p-1"),
    hf64!("0x1.b98p-1"), hf64!("0x1.b9p-1"), hf64!("0x1.b8p-1"), hf64!("0x1.b78p-1"), hf64!("0x1.b68p-1"), hf64!("0x1.b6p-1"), hf64!("0x1.b58p-1"),
    hf64!("0x1.b48p-1"), hf64!("0x1.b4p-1"), hf64!("0x1.b3p-1"), hf64!("0x1.b28p-1"), hf64!("0x1.b18p-1"), hf64!("0x1.b1p-1"), hf64!("0x1.bp-1"),
    hf64!("0x1.af8p-1"), hf64!("0x1.afp-1"), hf64!("0x1.aep-1"), hf64!("0x1.ad8p-1"), hf64!("0x1.ac8p-1"), hf64!("0x1.acp-1"), hf64!("0x1.ab8p-1"),
    hf64!("0x1.aa8p-1"), hf64!("0x1.aap-1"), hf64!("0x1.a9p-1"), hf64!("0x1.a88p-1"), hf64!("0x1.a8p-1"), hf64!("0x1.a7p-1"), hf64!("0x1.a68p-1"),
    hf64!("0x1.a6p-1"), hf64!("0x1.a5p-1"), hf64!("0x1.a48p-1"), hf64!("0x1.a4p-1"), hf64!("0x1.a3p-1"), hf64!("0x1.a28p-1"), hf64!("0x1.a2p-1"),
    hf64!("0x1.a1p-1"), hf64!("0x1.a08p-1"), hf64!("0x1.ap-1"), hf64!("0x1.9fp-1"), hf64!("0x1.9e8p-1"), hf64!("0x1.9ep-1"), hf64!("0x1.9dp-1"),
    hf64!("0x1.9c8p-1"), hf64!("0x1.9cp-1"), hf64!("0x1.9bp-1"), hf64!("0x1.9a8p-1"), hf64!("0x1.9ap-1"), hf64!("0x1.998p-1"), hf64!("0x1.988p-1"),
    hf64!("0x1.98p-1"), hf64!("0x1.978p-1"), hf64!("0x1.968p-1"), hf64!("0x1.96p-1"), hf64!("0x1.958p-1"), hf64!("0x1.95p-1"), hf64!("0x1.94p-1"),
    hf64!("0x1.938p-1"), hf64!("0x1.93p-1"), hf64!("0x1.928p-1"), hf64!("0x1.92p-1"), hf64!("0x1.91p-1"), hf64!("0x1.908p-1"), hf64!("0x1.9p-1"),
    hf64!("0x1.8f8p-1"), hf64!("0x1.8e8p-1"), hf64!("0x1.8ep-1"), hf64!("0x1.8d8p-1"), hf64!("0x1.8dp-1"), hf64!("0x1.8c8p-1"), hf64!("0x1.8b8p-1"),
    hf64!("0x1.8bp-1"), hf64!("0x1.8a8p-1"), hf64!("0x1.8ap-1"), hf64!("0x1.898p-1"), hf64!("0x1.888p-1"), hf64!("0x1.88p-1"), hf64!("0x1.878p-1"),
    hf64!("0x1.87p-1"), hf64!("0x1.868p-1"), hf64!("0x1.86p-1"), hf64!("0x1.85p-1"), hf64!("0x1.848p-1"), hf64!("0x1.84p-1"), hf64!("0x1.838p-1"),
    hf64!("0x1.83p-1"), hf64!("0x1.828p-1"), hf64!("0x1.82p-1"), hf64!("0x1.81p-1"), hf64!("0x1.808p-1"), hf64!("0x1.8p-1"), hf64!("0x1.7f8p-1"),
    hf64!("0x1.7fp-1"), hf64!("0x1.7e8p-1"), hf64!("0x1.7ep-1"), hf64!("0x1.7d8p-1"), hf64!("0x1.7c8p-1"), hf64!("0x1.7cp-1"), hf64!("0x1.7b8p-1"),
    hf64!("0x1.7bp-1"), hf64!("0x1.7a8p-1"), hf64!("0x1.7ap-1"), hf64!("0x1.798p-1"), hf64!("0x1.79p-1"), hf64!("0x1.788p-1"), hf64!("0x1.78p-1"),
    hf64!("0x1.778p-1"), hf64!("0x1.77p-1"), hf64!("0x1.76p-1"), hf64!("0x1.758p-1"), hf64!("0x1.75p-1"), hf64!("0x1.748p-1"), hf64!("0x1.74p-1"),
    hf64!("0x1.738p-1"), hf64!("0x1.73p-1"), hf64!("0x1.728p-1"), hf64!("0x1.72p-1"), hf64!("0x1.718p-1"), hf64!("0x1.71p-1"), hf64!("0x1.708p-1"),
    hf64!("0x1.7p-1"), hf64!("0x1.6f8p-1"), hf64!("0x1.6fp-1"), hf64!("0x1.6e8p-1"), hf64!("0x1.6ep-1"), hf64!("0x1.6d8p-1"), hf64!("0x1.6dp-1"),
    hf64!("0x1.6c8p-1"), hf64!("0x1.6cp-1"), hf64!("0x1.6b8p-1"), hf64!("0x1.6bp-1"), hf64!("0x1.6a8p-1"), hf64!("0x1.6ap-1"),
];

// For 362 <= i <= 724, (h,l) = _LOG_INV[i-362] is a double-double
// approximation of -log(r) with r=INVERSE[i-362]), with h an integer multiple
// of 2^-42, and |l| < 2^-43. The maximal difference between -log(r) and h+l
// is bounded by 1/2 ulp(l) < 2^-97.
#[rustfmt::skip]
static _LOG_INV: [(f64, f64); 363] = [
    (hf64!("-0x1.615ddb4becp-2"), hf64!("-0x1.3c7ca90bc04b2p-46")),
    (hf64!("-0x1.5e87b20c29p-2"), hf64!("-0x1.527d18f7738fap-44")),
    (hf64!("-0x1.5baf846aa2p-2"), hf64!("0x1.39ae8f873fa41p-44")),
    (hf64!("-0x1.58d54f86ep-2"), hf64!("-0x1.791f30a795215p-45")),
    (hf64!("-0x1.55f9107a44p-2"), hf64!("0x1.1e64778df4a62p-46")),
    (hf64!("-0x1.531ac457eep-2"), hf64!("-0x1.df83b7d931501p-44")),
    (hf64!("-0x1.503a682cb2p-2"), hf64!("0x1.a68c8f16f9b5dp-45")),
    (hf64!("-0x1.4ec97326p-2"), hf64!("-0x1.34d7aaf04d104p-45")),
    (hf64!("-0x1.4be5f95778p-2"), hf64!("0x1.d7c92cd9ad824p-44")),
    (hf64!("-0x1.4900680401p-2"), hf64!("0x1.8bccffe1a0f8cp-44")),
    (hf64!("-0x1.4618bc21c6p-2"), hf64!("0x1.3d82f484c84ccp-46")),
    (hf64!("-0x1.432ef2a04fp-2"), hf64!("0x1.fb129931715adp-44")),
    (hf64!("-0x1.404308686ap-2"), hf64!("-0x1.f8ef43049f7d3p-44")),
    (hf64!("-0x1.3d54fa5c1fp-2"), hf64!("-0x1.c3e1cd9a395e3p-44")),
    (hf64!("-0x1.3a64c55694p-2"), hf64!("-0x1.7a71cbcd735dp-44")),
    (hf64!("-0x1.3772662bfep-2"), hf64!("0x1.e9436ac53b023p-44")),
    (hf64!("-0x1.35f865c933p-2"), hf64!("0x1.b07de4ea1a54ap-44")),
    (hf64!("-0x1.3302c16586p-2"), hf64!("-0x1.6217dc2a3e08bp-44")),
    (hf64!("-0x1.300aead063p-2"), hf64!("-0x1.42f568b75fcacp-44")),
    (hf64!("-0x1.2d10dec508p-2"), hf64!("-0x1.60c61f7088353p-44")),
    (hf64!("-0x1.2a1499f763p-2"), hf64!("0x1.0dbbf51f3aadcp-44")),
    (hf64!("-0x1.2895a13de8p-2"), hf64!("-0x1.a8d7ad24c13fp-44")),
    (hf64!("-0x1.2596010df7p-2"), hf64!("-0x1.8e7bc224ea3e3p-44")),
    (hf64!("-0x1.22941fbcf8p-2"), hf64!("0x1.a6976f5eb0963p-44")),
    (hf64!("-0x1.1f8ff9e48ap-2"), hf64!("-0x1.7946c040cbe77p-45")),
    (hf64!("-0x1.1c898c169ap-2"), hf64!("0x1.81410e5c62affp-44")),
    (hf64!("-0x1.1b05791f08p-2"), hf64!("0x1.2dd466dc55e2dp-44")),
    (hf64!("-0x1.17fb98e151p-2"), hf64!("0x1.a8a8ba74a2684p-44")),
    (hf64!("-0x1.14ef67f887p-2"), hf64!("0x1.e97a65dfc9794p-44")),
    (hf64!("-0x1.136870293bp-2"), hf64!("0x1.d3e8499d67123p-44")),
    (hf64!("-0x1.1058bf9ae5p-2"), hf64!("0x1.4ab9d817d52cdp-44")),
    (hf64!("-0x1.0d46b579abp-2"), hf64!("-0x1.d2c81f640e1e6p-44")),
    (hf64!("-0x1.0a324e2739p-2"), hf64!("-0x1.c6bee7ef4030ep-47")),
    (hf64!("-0x1.08a73667c5p-2"), hf64!("-0x1.ebc1d40c5a329p-44")),
    (hf64!("-0x1.058f3c703fp-2"), hf64!("0x1.0e866bcd236adp-44")),
    (hf64!("-0x1.0402594b4dp-2"), hf64!("-0x1.036b89ef42d7fp-48")),
    (hf64!("-0x1.00e6c45ad5p-2"), hf64!("-0x1.cc68d52e01203p-50")),
    (hf64!("-0x1.fb9186d5e4p-3"), hf64!("0x1.d572aab993c87p-47")),
    (hf64!("-0x1.f871b28956p-3"), hf64!("0x1.f75fd6a526efep-44")),
    (hf64!("-0x1.f22e5e72f2p-3"), hf64!("0x1.f454f1417e41fp-44")),
    (hf64!("-0x1.ebe61f4dd8p-3"), hf64!("0x1.3d45330fdca4dp-45")),
    (hf64!("-0x1.e8c0252aa6p-3"), hf64!("0x1.6805b80e8e6ffp-45")),
    (hf64!("-0x1.e27076e2bp-3"), hf64!("0x1.a342c2af0003cp-44")),
    (hf64!("-0x1.dc1bca0abep-3"), hf64!("-0x1.8fac1a628ccc6p-44")),
    (hf64!("-0x1.d8ef91af32p-3"), hf64!("0x1.5105fc364c784p-46")),
    (hf64!("-0x1.d293581b6cp-3"), hf64!("0x1.83270128aaa5fp-44")),
    (hf64!("-0x1.cf6354e09cp-3"), hf64!("-0x1.771239a07d55bp-45")),
    (hf64!("-0x1.c8ff7c79aap-3"), hf64!("0x1.7794f689f8434p-45")),
    (hf64!("-0x1.c5cba543aep-3"), hf64!("-0x1.0929decb454fcp-45")),
    (hf64!("-0x1.bf601bb0e4p-3"), hf64!("-0x1.386a947c378b5p-45")),
    (hf64!("-0x1.bc286742d8p-3"), hf64!("-0x1.9ac53f39d121cp-44")),
    (hf64!("-0x1.b5b519e8fcp-3"), hf64!("0x1.4b722ec011f31p-44")),
    (hf64!("-0x1.af3c94e80cp-3"), hf64!("0x1.a4e633fcd9066p-52")),
    (hf64!("-0x1.abfe5ae462p-3"), hf64!("0x1.b68f5395f139dp-44")),
    (hf64!("-0x1.a57df28244p-3"), hf64!("-0x1.b99c8ca1d9abbp-44")),
    (hf64!("-0x1.a23bc1fe2cp-3"), hf64!("0x1.539cd91dc9f0bp-44")),
    (hf64!("-0x1.9bb362e7ep-3"), hf64!("0x1.1f2a8a1ce0ffcp-45")),
    (hf64!("-0x1.986d322818p-3"), hf64!("-0x1.93b564dd44p-48")),
    (hf64!("-0x1.91dcc8c34p-3"), hf64!("-0x1.7bc6abddeff46p-44")),
    (hf64!("-0x1.8e928de886p-3"), hf64!("-0x1.a8154b13d72d5p-44")),
    (hf64!("-0x1.87fa06520cp-3"), hf64!("-0x1.22120401202fcp-44")),
    (hf64!("-0x1.84abb75866p-3"), hf64!("0x1.d8daadf4e2bd2p-44")),
    (hf64!("-0x1.815c0a1436p-3"), hf64!("0x1.02a52f9201ce8p-44")),
    (hf64!("-0x1.7ab890210ep-3"), hf64!("0x1.bdb9072534a58p-45")),
    (hf64!("-0x1.7764c128f2p-3"), hf64!("-0x1.274903479e3d1p-47")),
    (hf64!("-0x1.70b8f97a1ap-3"), hf64!("-0x1.4ea64f6a95befp-44")),
    (hf64!("-0x1.6d60fe719ep-3"), hf64!("0x1.bc6e557134767p-44")),
    (hf64!("-0x1.66acd4272ap-3"), hf64!("-0x1.aa1bdbfc6c785p-44")),
    (hf64!("-0x1.6350a28aaap-3"), hf64!("-0x1.d5ec0ab8163afp-45")),
    (hf64!("-0x1.5ff3070a7ap-3"), hf64!("0x1.8586f183bebf2p-44")),
    (hf64!("-0x1.59338d9982p-3"), hf64!("-0x1.0ba68b7555d4ap-48")),
    (hf64!("-0x1.55d1ad4232p-3"), hf64!("-0x1.add94dda647e8p-44")),
    (hf64!("-0x1.4f099f4a24p-3"), hf64!("0x1.e9bf2fafeaf27p-44")),
    (hf64!("-0x1.4ba36f39a6p-3"), hf64!("0x1.4354bb3f219e5p-44")),
    (hf64!("-0x1.483bccce6ep-3"), hf64!("-0x1.eea52723f6369p-46")),
    (hf64!("-0x1.41682bf728p-3"), hf64!("0x1.10047081f849dp-45")),
    (hf64!("-0x1.3dfc2b0eccp-3"), hf64!("-0x1.8a72a62b8c13fp-45")),
    (hf64!("-0x1.371fc201e8p-3"), hf64!("-0x1.ee8779b2d8abcp-44")),
    (hf64!("-0x1.33af57577p-3"), hf64!("-0x1.c9ecca2fe72a5p-44")),
    (hf64!("-0x1.303d718e48p-3"), hf64!("0x1.680b5ce3ecb05p-50")),
    (hf64!("-0x1.29552f82p-3"), hf64!("0x1.5b967f4471dfcp-44")),
    (hf64!("-0x1.25ded0abc6p-3"), hf64!("-0x1.5a3854f176449p-44")),
    (hf64!("-0x1.2266f190a6p-3"), hf64!("0x1.4d20ab840e7f6p-45")),
    (hf64!("-0x1.1b72ad52f6p-3"), hf64!("-0x1.e80a41811a396p-45")),
    (hf64!("-0x1.17f6458fcap-3"), hf64!("-0x1.843fad093c8dcp-45")),
    (hf64!("-0x1.1478584674p-3"), hf64!("-0x1.563451027c75p-46")),
    (hf64!("-0x1.0d77e7cd08p-3"), hf64!("-0x1.cb2cd2ee2f482p-44")),
    (hf64!("-0x1.09f561ee72p-3"), hf64!("0x1.8f3057157d1a8p-45")),
    (hf64!("-0x1.0671512ca6p-3"), hf64!("0x1.a47579cdc0a3dp-45")),
    (hf64!("-0x1.02ebb42bf4p-3"), hf64!("0x1.5a8fa5ce00e5dp-46")),
    (hf64!("-0x1.f7b79fec38p-4"), hf64!("0x1.10987e897ed01p-47")),
    (hf64!("-0x1.f0a30c0118p-4"), hf64!("0x1.d599e83368e91p-44")),
    (hf64!("-0x1.e98b54967p-4"), hf64!("-0x1.4677489c50e97p-44")),
    (hf64!("-0x1.e27076e2bp-4"), hf64!("0x1.a342c2af0003cp-45")),
    (hf64!("-0x1.d4313d66ccp-4"), hf64!("0x1.9454379135713p-45")),
    (hf64!("-0x1.cd0cdbf8cp-4"), hf64!("-0x1.3e14db50dd743p-44")),
    (hf64!("-0x1.c5e548f5bcp-4"), hf64!("-0x1.d0c57585fbe06p-46")),
    (hf64!("-0x1.b78c82bb1p-4"), hf64!("0x1.25ef7bc3987e7p-44")),
    (hf64!("-0x1.b05b49bee4p-4"), hf64!("-0x1.ff22c18f84a5ep-47")),
    (hf64!("-0x1.a926d3a4acp-4"), hf64!("-0x1.563650bd22a9cp-44")),
    (hf64!("-0x1.a1ef1d806p-4"), hf64!("-0x1.cd4176df97bcbp-44")),
    (hf64!("-0x1.9ab4246204p-4"), hf64!("0x1.8a64826787061p-45")),
    (hf64!("-0x1.8c345d6318p-4"), hf64!("-0x1.b20f5acb42a66p-44")),
    (hf64!("-0x1.84ef898e84p-4"), hf64!("0x1.7d5cd246977c9p-44")),
    (hf64!("-0x1.7da766d7bp-4"), hf64!("-0x1.2cc844480c89bp-44")),
    (hf64!("-0x1.765bf23a6cp-4"), hf64!("0x1.ecbc035c4256ap-48")),
    (hf64!("-0x1.6f0d28ae58p-4"), hf64!("0x1.4b4641b664613p-44")),
    (hf64!("-0x1.60658a9374p-4"), hf64!("-0x1.0c3b1dee9c4f8p-44")),
    (hf64!("-0x1.590cafdfp-4"), hf64!("-0x1.c284f5722abaap-44")),
    (hf64!("-0x1.51b073f06p-4"), hf64!("-0x1.83f69278e686ap-44")),
    (hf64!("-0x1.4a50d3aa1cp-4"), hf64!("0x1.f7fe1308973e2p-45")),
    (hf64!("-0x1.42edcbea64p-4"), hf64!("-0x1.bc0eeea7c9acdp-46")),
    (hf64!("-0x1.341d7961bcp-4"), hf64!("-0x1.1d0929983761p-44")),
    (hf64!("-0x1.2cb0283f5cp-4"), hf64!("-0x1.e1ee2ca657021p-44")),
    (hf64!("-0x1.253f62f0ap-4"), hf64!("-0x1.416f8fb69a701p-44")),
    (hf64!("-0x1.1dcb263dbp-4"), hf64!("-0x1.9444f5e9e8981p-44")),
    (hf64!("-0x1.16536eea38p-4"), hf64!("0x1.47c5e768fa309p-46")),
    (hf64!("-0x1.0ed839b554p-4"), hf64!("0x1.901f46d48abb4p-44")),
    (hf64!("-0x1.075983599p-4"), hf64!("0x1.b8ecfe4b59987p-44")),
    (hf64!("-0x1.f0a30c0118p-5"), hf64!("0x1.d599e83368e91p-45")),
    (hf64!("-0x1.e19070c278p-5"), hf64!("0x1.fea4664629e86p-45")),
    (hf64!("-0x1.d276b8adbp-5"), hf64!("-0x1.6a423c78a64bp-46")),
    (hf64!("-0x1.c355dd092p-5"), hf64!("-0x1.f2ccc9abf8388p-45")),
    (hf64!("-0x1.b42dd71198p-5"), hf64!("0x1.c827ae5d6704cp-46")),
    (hf64!("-0x1.a4fe9ffa4p-5"), hf64!("0x1.6e584a0402925p-44")),
    (hf64!("-0x1.95c830ec9p-5"), hf64!("0x1.c148297c5feb8p-45")),
    (hf64!("-0x1.868a83084p-5"), hf64!("0x1.2623a134ac693p-46")),
    (hf64!("-0x1.77458f633p-5"), hf64!("0x1.181dce586af09p-44")),
    (hf64!("-0x1.58a5bafc9p-5"), hf64!("0x1.b2b739570ad39p-45")),
    (hf64!("-0x1.494acc34d8p-5"), hf64!("-0x1.11c78a56fd247p-45")),
    (hf64!("-0x1.39e87b9fe8p-5"), hf64!("-0x1.eafd480ad9015p-44")),
    (hf64!("-0x1.2a7ec2215p-5"), hf64!("0x1.78ce77a9163fep-45")),
    (hf64!("-0x1.1b0d98924p-5"), hf64!("0x1.3401e9ae889bbp-44")),
    (hf64!("-0x1.0b94f7c198p-5"), hf64!("0x1.e89896f022783p-45")),
    (hf64!("-0x1.f829b0e78p-6"), hf64!("-0x1.980267c7e09e4p-45")),
    (hf64!("-0x1.d91a66c54p-6"), hf64!("-0x1.e61f1658cfb9ap-45")),
    (hf64!("-0x1.b9fc027bp-6"), hf64!("0x1.b9a010ae6922ap-44")),
    (hf64!("-0x1.9ace7551dp-6"), hf64!("0x1.d75d97ec7c41p-45")),
    (hf64!("-0x1.7b91b07d6p-6"), hf64!("0x1.3b955b602ace4p-44")),
    (hf64!("-0x1.5c45a51b9p-6"), hf64!("0x1.63bb6216d87d8p-45")),
    (hf64!("-0x1.3cea44347p-6"), hf64!("0x1.6a2c432d6a40bp-44")),
    (hf64!("-0x1.1d7f7eb9fp-6"), hf64!("0x1.4193a83fcc7a6p-46")),
    (hf64!("-0x1.fc0a8b0fcp-7"), hf64!("-0x1.f1e7cf6d3a69cp-50")),
    (hf64!("-0x1.bcf712c74p-7"), hf64!("-0x1.c25e097bd9771p-46")),
    (hf64!("-0x1.7dc475f82p-7"), hf64!("0x1.eb1245b5da1f5p-44")),
    (hf64!("-0x1.3e7295d26p-7"), hf64!("0x1.609c1ff29a114p-45")),
    (hf64!("-0x1.fe02a6b1p-8"), hf64!("-0x1.9e23f0dda40e4p-46")),
    (hf64!("-0x1.7ee11ebd8p-8"), hf64!("-0x1.749d3c2d23a07p-47")),
    (hf64!("-0x1.ff00aa2bp-9"), hf64!("-0x1.0bc04a086b56ap-45")),
    (hf64!("-0x1.ff802a9bp-10"), hf64!("0x1.3bc661d61c5ebp-44")),
    (hf64!("0x1.00200556p-10"), hf64!("0x1.56224cd5f35f8p-44")),
    (hf64!("0x1.809048288p-9"), hf64!("0x1.85c0696a70c0cp-45")),
    (hf64!("0x1.40c8a7478p-8"), hf64!("0x1.e3871df070002p-46")),
    (hf64!("0x1.c189cbb1p-8"), hf64!("-0x1.d80551258856p-44")),
    (hf64!("0x1.2145e939ep-7"), hf64!("0x1.e3d1238c4eap-44")),
    (hf64!("0x1.61e77e8b6p-7"), hf64!("-0x1.8073eeaf8eaf3p-44")),
    (hf64!("0x1.a2a9c6c18p-7"), hf64!("-0x1.f73bc4d6d3472p-44")),
    (hf64!("0x1.e38ce3034p-7"), hf64!("-0x1.9de88a3da281ap-44")),
    (hf64!("0x1.12487a55p-6"), hf64!("0x1.fdbe5fed4b393p-44")),
    (hf64!("0x1.32db0ea13p-6"), hf64!("0x1.710cb130895fcp-45")),
    (hf64!("0x1.537e3f45fp-6"), hf64!("0x1.ab259d2d7f253p-45")),
    (hf64!("0x1.63d617869p-6"), hf64!("0x1.7abf389596542p-47")),
    (hf64!("0x1.8492528c9p-6"), hf64!("-0x1.aa0ba325a0c34p-45")),
    (hf64!("0x1.a55f548c6p-6"), hf64!("-0x1.de0709f2d03c9p-45")),
    (hf64!("0x1.c63d2ec15p-6"), hf64!("-0x1.5439ce030a687p-44")),
    (hf64!("0x1.e72bf2814p-6"), hf64!("-0x1.8d75149774d47p-45")),
    (hf64!("0x1.0415d89e78p-5"), hf64!("-0x1.dddc7f461c516p-44")),
    (hf64!("0x1.149e3e4008p-5"), hf64!("-0x1.2b98a9a4168fdp-44")),
    (hf64!("0x1.252f32f8dp-5"), hf64!("0x1.83e9ae021b67bp-45")),
    (hf64!("0x1.35c8bfaa1p-5"), hf64!("0x1.8357d5ef9eb35p-44")),
    (hf64!("0x1.3e18c1ca08p-5"), hf64!("0x1.748ed3f6e378ep-44")),
    (hf64!("0x1.4ebf4334ap-5"), hf64!("-0x1.d9150f73be773p-45")),
    (hf64!("0x1.5f6e73079p-5"), hf64!("-0x1.0485a8012494cp-45")),
    (hf64!("0x1.70265a551p-5"), hf64!("-0x1.888df11fd5ce7p-45")),
    (hf64!("0x1.80e7023d9p-5"), hf64!("-0x1.99dc16f28bf45p-44")),
    (hf64!("0x1.91b073efd8p-5"), hf64!("-0x1.9d7c53f76ca96p-46")),
    (hf64!("0x1.9a187b574p-5"), hf64!("-0x1.0c22e4ec4d90dp-44")),
    (hf64!("0x1.aaef2d0fbp-5"), hf64!("0x1.0fc1a353bb42ep-45")),
    (hf64!("0x1.bbcebfc69p-5"), hf64!("-0x1.7bf868c317c2ap-46")),
    (hf64!("0x1.ccb73cddd8p-5"), hf64!("0x1.965c36e09f5fep-44")),
    (hf64!("0x1.dda8adc68p-5"), hf64!("-0x1.1b1ac64d9e42fp-45")),
    (hf64!("0x1.e624c4a0b8p-5"), hf64!("-0x1.0f25c74676689p-44")),
    (hf64!("0x1.f723b518p-5"), hf64!("-0x1.d6eb0dd5610d3p-44")),
    (hf64!("0x1.0415d89e74p-4"), hf64!("0x1.111c05cf1d753p-46")),
    (hf64!("0x1.0c9e615ac4p-4"), hf64!("0x1.c2da80974d976p-45")),
    (hf64!("0x1.10e45b3cbp-4"), hf64!("-0x1.7cf69284a3465p-44")),
    (hf64!("0x1.1973bd1464p-4"), hf64!("0x1.566d154f930b3p-44")),
    (hf64!("0x1.2207b5c784p-4"), hf64!("0x1.49d8cfc10c7bfp-44")),
    (hf64!("0x1.2aa04a447p-4"), hf64!("0x1.7a48ba8b1cb41p-44")),
    (hf64!("0x1.2eee507b4p-4"), hf64!("0x1.8081edd77c86p-47")),
    (hf64!("0x1.378dd7f748p-4"), hf64!("0x1.7141128f1facap-44")),
    (hf64!("0x1.403207b414p-4"), hf64!("0x1.6fd84aa8157cp-45")),
    (hf64!("0x1.4485e03dbcp-4"), hf64!("0x1.fad46e8d26ab7p-44")),
    (hf64!("0x1.4d3115d208p-4"), hf64!("-0x1.53a2582f4e1efp-48")),
    (hf64!("0x1.55e10050ep-4"), hf64!("0x1.c1d740c53c72ep-47")),
    (hf64!("0x1.5e95a4d978p-4"), hf64!("0x1.1cb7ce1d17171p-44")),
    (hf64!("0x1.62f1be7d78p-4"), hf64!("-0x1.179957ed63c4ep-45")),
    (hf64!("0x1.6bad83c188p-4"), hf64!("0x1.daf3cc08926aep-47")),
    (hf64!("0x1.746e100228p-4"), hf64!("-0x1.126d16e1e21d2p-44")),
    (hf64!("0x1.78d02263d8p-4"), hf64!("0x1.69b5794b69fb7p-47")),
    (hf64!("0x1.8197e2f41p-4"), hf64!("-0x1.c0fe460d20041p-44")),
    (hf64!("0x1.8a6477a91cp-4"), hf64!("0x1.c28c0af9bd6dfp-44")),
    (hf64!("0x1.8ecc933aecp-4"), hf64!("-0x1.22f39be67f7aap-45")),
    (hf64!("0x1.97a07024ccp-4"), hf64!("-0x1.8bcc1732093cep-48")),
    (hf64!("0x1.a0792e9278p-4"), hf64!("-0x1.a9ce6c9ad51bfp-47")),
    (hf64!("0x1.a4e7640b1cp-4"), hf64!("-0x1.e42b6b94407c8p-47")),
    (hf64!("0x1.adc77ee5bp-4"), hf64!("-0x1.573b209c31904p-44")),
    (hf64!("0x1.b23965a53p-4"), hf64!("-0x1.ff64eea137079p-49")),
    (hf64!("0x1.bb20e936d8p-4"), hf64!("-0x1.68ba835459b8ep-44")),
    (hf64!("0x1.c40d6425a4p-4"), hf64!("0x1.cb1121d1930ddp-44")),
    (hf64!("0x1.c885801bc4p-4"), hf64!("0x1.646d1c65aacd3p-45")),
    (hf64!("0x1.d179788218p-4"), hf64!("0x1.36433b5efbeedp-44")),
    (hf64!("0x1.d5f556592p-4"), hf64!("0x1.0e239cc185469p-44")),
    (hf64!("0x1.def0d8d468p-4"), hf64!("-0x1.24750412e9a74p-44")),
    (hf64!("0x1.e7f1691a34p-4"), hf64!("-0x1.2c1c59bc77bfap-44")),
    (hf64!("0x1.ec739830ap-4"), hf64!("0x1.11fcba80cdd1p-44")),
    (hf64!("0x1.f57bc7d9p-4"), hf64!("0x1.76a6c9ea8b04ep-46")),
    (hf64!("0x1.fa01c9db58p-4"), hf64!("-0x1.8f351fa48a73p-47")),
    (hf64!("0x1.0188d2ecf6p-3"), hf64!("0x1.3f9651cff9dfep-47")),
    (hf64!("0x1.03cdc0a51ep-3"), hf64!("0x1.81a9cf169fc5cp-44")),
    (hf64!("0x1.08598b59e4p-3"), hf64!("-0x1.7e5dd7009902cp-45")),
    (hf64!("0x1.0aa0691268p-3"), hf64!("-0x1.45519d7032129p-44")),
    (hf64!("0x1.0f301717dp-3"), hf64!("-0x1.e09b441ae86c5p-44")),
    (hf64!("0x1.13c2605c3ap-3"), hf64!("-0x1.cf5fdd94f6509p-45")),
    (hf64!("0x1.160c8024b2p-3"), hf64!("0x1.ec2d2a9009e3dp-45")),
    (hf64!("0x1.1aa2b7e24p-3"), hf64!("-0x1.1ac38dde3b366p-44")),
    (hf64!("0x1.1ceed09854p-3"), hf64!("-0x1.15c1c39192af9p-44")),
    (hf64!("0x1.2188fd9808p-3"), hf64!("-0x1.b3a1e7f50c701p-44")),
    (hf64!("0x1.23d712a49cp-3"), hf64!("0x1.00d238fd3df5cp-46")),
    (hf64!("0x1.28753bc11ap-3"), hf64!("0x1.7494e359302e6p-44")),
    (hf64!("0x1.2ac55095f6p-3"), hf64!("-0x1.d3466d0c6c8a8p-46")),
    (hf64!("0x1.2f677cbbcp-3"), hf64!("0x1.52b302160f40dp-44")),
    (hf64!("0x1.31b994d3a4p-3"), hf64!("0x1.f098ee3a5081p-44")),
    (hf64!("0x1.365fcb015ap-3"), hf64!("-0x1.fd3a0afb9691bp-44")),
    (hf64!("0x1.38b3e9e028p-3"), hf64!("-0x1.70ef0545c17f9p-44")),
    (hf64!("0x1.3d5e3126bcp-3"), hf64!("0x1.3fb2f85096c4bp-46")),
    (hf64!("0x1.3fb45a5992p-3"), hf64!("0x1.19713c0cae559p-44")),
    (hf64!("0x1.420b32741p-3"), hf64!("-0x1.16282c85a0884p-46")),
    (hf64!("0x1.46baf0f9f6p-3"), hf64!("-0x1.249cd0790841ap-46")),
    (hf64!("0x1.4913d8333cp-3"), hf64!("-0x1.53e43558124c4p-44")),
    (hf64!("0x1.4dc7b897bcp-3"), hf64!("0x1.c79b60ae1ff0fp-47")),
    (hf64!("0x1.5022b292f6p-3"), hf64!("0x1.48a05ff36a25bp-44")),
    (hf64!("0x1.54dabc261p-3"), hf64!("0x1.746fee5c8d0d8p-45")),
    (hf64!("0x1.5737cc9018p-3"), hf64!("0x1.9baa7a6b887f6p-44")),
    (hf64!("0x1.5bf406b544p-3"), hf64!("-0x1.27023eb68981cp-46")),
    (hf64!("0x1.5e533144c2p-3"), hf64!("-0x1.1ce0bf3b290eap-44")),
    (hf64!("0x1.60b3100b0ap-3"), hf64!("-0x1.71456c988f814p-44")),
    (hf64!("0x1.6574ebe8c2p-3"), hf64!("-0x1.98c1d34f0f462p-44")),
    (hf64!("0x1.67d6e9d786p-3"), hf64!("-0x1.11e8830a706d3p-44")),
    (hf64!("0x1.6c9d07d204p-3"), hf64!("-0x1.c73fafd9b2dcap-50")),
    (hf64!("0x1.6f0128b756p-3"), hf64!("0x1.577390d31ef0fp-44")),
    (hf64!("0x1.716600c914p-3"), hf64!("0x1.51b157cec3838p-49")),
    (hf64!("0x1.7631d82936p-3"), hf64!("-0x1.5e77dc7c5f3e1p-45")),
    (hf64!("0x1.7898d85444p-3"), hf64!("0x1.8e67be3dbaf3fp-44")),
    (hf64!("0x1.7d6903caf6p-3"), hf64!("-0x1.4c06b17c301d7p-45")),
    (hf64!("0x1.7fd22ff59ap-3"), hf64!("-0x1.58bebf457b7d2p-46")),
    (hf64!("0x1.823c16551ap-3"), hf64!("0x1.e0ddb9a631e83p-46")),
    (hf64!("0x1.871213750ep-3"), hf64!("0x1.328eb42f9af75p-44")),
    (hf64!("0x1.897e2b17b2p-3"), hf64!("-0x1.96b37380cbe9ep-45")),
    (hf64!("0x1.8beafeb39p-3"), hf64!("-0x1.73d54aae92cd1p-47")),
    (hf64!("0x1.90c6db9fccp-3"), hf64!("-0x1.935f57718d7cap-46")),
    (hf64!("0x1.9335e5d594p-3"), hf64!("0x1.3115c3abd47dap-44")),
    (hf64!("0x1.95a5adcf7p-3"), hf64!("0x1.7f22858a0ff6fp-47")),
    (hf64!("0x1.9a8778debap-3"), hf64!("0x1.470fa3efec39p-44")),
    (hf64!("0x1.9cf97cdcep-3"), hf64!("0x1.d862f10c414e3p-44")),
    (hf64!("0x1.9f6c40708ap-3"), hf64!("-0x1.337d94bcd3f43p-44")),
    (hf64!("0x1.a454082e6ap-3"), hf64!("0x1.60a77c81f7171p-44")),
    (hf64!("0x1.a6c90d44b8p-3"), hf64!("-0x1.f63b7f037b0c6p-44")),
    (hf64!("0x1.a93ed3c8aep-3"), hf64!("-0x1.8724350562169p-45")),
    (hf64!("0x1.ae2ca6f672p-3"), hf64!("0x1.7a8d5ae54f55p-44")),
    (hf64!("0x1.b0a4b48fc2p-3"), hf64!("-0x1.2e72d5c3998edp-45")),
    (hf64!("0x1.b31d8575bcp-3"), hf64!("0x1.c794e562a63cbp-44")),
    (hf64!("0x1.b811730b82p-3"), hf64!("0x1.e90683b9cd768p-46")),
    (hf64!("0x1.ba8c90ae4ap-3"), hf64!("0x1.a32e7f44432dap-44")),
    (hf64!("0x1.bd087383bep-3"), hf64!("-0x1.d4bc4595412b6p-45")),
    (hf64!("0x1.c2028ab18p-3"), hf64!("-0x1.92e0ee55c7ac6p-45")),
    (hf64!("0x1.c480c0005cp-3"), hf64!("0x1.9a294d5e44e76p-44")),
    (hf64!("0x1.c6ffbc6fp-3"), hf64!("0x1.ee138d3a69d43p-44")),
    (hf64!("0x1.c97f8079d4p-3"), hf64!("0x1.3b161a8c6e6c5p-45")),
    (hf64!("0x1.ce816157f2p-3"), hf64!("-0x1.9e0aba2099515p-45")),
    (hf64!("0x1.d1037f2656p-3"), hf64!("-0x1.84a7e75b6f6e4p-47")),
    (hf64!("0x1.d38666872p-3"), hf64!("-0x1.73650b38932bcp-44")),
    (hf64!("0x1.d88e93fb3p-3"), hf64!("-0x1.75f280234bf51p-44")),
    (hf64!("0x1.db13db0d48p-3"), hf64!("0x1.2806a847527e6p-44")),
    (hf64!("0x1.dd99edaf6ep-3"), hf64!("-0x1.02ec669c756ebp-44")),
    (hf64!("0x1.e020cc6236p-3"), hf64!("-0x1.52b00adb91424p-45")),
    (hf64!("0x1.e530effe72p-3"), hf64!("-0x1.fdbdbb13f7c18p-44")),
    (hf64!("0x1.e7ba35eb78p-3"), hf64!("-0x1.d5eee23793649p-47")),
    (hf64!("0x1.ea4449f04ap-3"), hf64!("0x1.5e91663732a36p-44")),
    (hf64!("0x1.eccf2c8feap-3"), hf64!("-0x1.bec63a3e7564p-44")),
    (hf64!("0x1.ef5ade4ddp-3"), hf64!("-0x1.a211565bb8e11p-51")),
    (hf64!("0x1.f474b134ep-3"), hf64!("-0x1.bae49f1df7b5ep-44")),
    (hf64!("0x1.f702d36778p-3"), hf64!("-0x1.0819516673e23p-46")),
    (hf64!("0x1.f991c6cb3cp-3"), hf64!("-0x1.90d04cd7cc834p-44")),
    (hf64!("0x1.fc218be62p-3"), hf64!("0x1.4bba46f1cf6ap-44")),
    (hf64!("0x1.00a1c6addap-2"), hf64!("0x1.1cd8d688b9e18p-44")),
    (hf64!("0x1.01eae5626cp-2"), hf64!("0x1.a43dcfade85aep-44")),
    (hf64!("0x1.03346e0106p-2"), hf64!("0x1.89ff8a966395cp-48")),
    (hf64!("0x1.047e60cde8p-2"), hf64!("0x1.dbdf10d397f3cp-45")),
    (hf64!("0x1.05c8be0d96p-2"), hf64!("0x1.ad0f1c77ccb58p-45")),
    (hf64!("0x1.085eb8f8aep-2"), hf64!("0x1.e5d513f45fe7bp-44")),
    (hf64!("0x1.09aa572e6cp-2"), hf64!("0x1.b50a1e1734342p-44")),
    (hf64!("0x1.0af660eb9ep-2"), hf64!("0x1.3c7c3f528d80ap-45")),
    (hf64!("0x1.0c42d67616p-2"), hf64!("0x1.7188b163ceae9p-45")),
    (hf64!("0x1.0d8fb813ebp-2"), hf64!("0x1.ee8c88753fa35p-46")),
    (hf64!("0x1.102ac0a35dp-2"), hf64!("-0x1.f1fbddfdfd686p-45")),
    (hf64!("0x1.1178e8227ep-2"), hf64!("0x1.1ef78ce2d07f2p-44")),
    (hf64!("0x1.12c77cd007p-2"), hf64!("0x1.3b2948a11f797p-46")),
    (hf64!("0x1.14167ef367p-2"), hf64!("0x1.e0c07824daaf5p-44")),
    (hf64!("0x1.1565eed456p-2"), hf64!("-0x1.e75adfb6aba25p-49")),
    (hf64!("0x1.16b5ccbadp-2"), hf64!("-0x1.23299042d74bfp-44")),
    (hf64!("0x1.1956d3b9bcp-2"), hf64!("0x1.7d2f73ad1aa14p-45")),
    (hf64!("0x1.1aa7fd638dp-2"), hf64!("0x1.9f60a9616f7ap-45")),
    (hf64!("0x1.1bf99635a7p-2"), hf64!("-0x1.1ac89575c2125p-44")),
    (hf64!("0x1.1d4b9e796cp-2"), hf64!("0x1.22a667c42e56dp-45")),
    (hf64!("0x1.1e9e16788ap-2"), hf64!("-0x1.82eaed3c8b65ep-44")),
    (hf64!("0x1.1ff0fe7cf4p-2"), hf64!("0x1.e9d5b513ff0c1p-44")),
    (hf64!("0x1.214456d0ecp-2"), hf64!("-0x1.caf0428b728a3p-44")),
    (hf64!("0x1.23ec5991ecp-2"), hf64!("-0x1.6dbe448a2e522p-44")),
    (hf64!("0x1.25410494e5p-2"), hf64!("0x1.b1d7ac0ef77f2p-44")),
    (hf64!("0x1.269621134ep-2"), hf64!("-0x1.1b61f10522625p-44")),
    (hf64!("0x1.27ebaf58d9p-2"), hf64!("-0x1.b198800b4bda7p-45")),
    (hf64!("0x1.2941afb187p-2"), hf64!("-0x1.210c2b730e28bp-44")),
    (hf64!("0x1.2a982269a4p-2"), hf64!("-0x1.2058e557285cfp-45")),
    (hf64!("0x1.2bef07cdc9p-2"), hf64!("0x1.a9cfa4a5004f4p-45")),
    (hf64!("0x1.2d46602addp-2"), hf64!("-0x1.88d0ddcd54196p-45")),
    (hf64!("0x1.2ff66b04ebp-2"), hf64!("-0x1.8aed2541e6e2ep-44")),
    (hf64!("0x1.314f1e1d36p-2"), hf64!("-0x1.8e27ad3213cb8p-45")),
    (hf64!("0x1.32a8456512p-2"), hf64!("0x1.4f928139af5d6p-47")),
    (hf64!("0x1.3401e12aedp-2"), hf64!("-0x1.17c73556e291dp-44")),
    (hf64!("0x1.355bf1bd83p-2"), hf64!("-0x1.ba99b8964f0e8p-45")),
    (hf64!("0x1.36b6776be1p-2"), hf64!("0x1.16ecdb0f177c8p-46")),
    (hf64!("0x1.3811728565p-2"), hf64!("-0x1.a71e493a0702bp-45")),
    (hf64!("0x1.396ce359bcp-2"), hf64!("-0x1.5839c5663663dp-47")),
    (hf64!("0x1.3ac8ca38e6p-2"), hf64!("-0x1.d0befbc02be4ap-45")),
    (hf64!("0x1.3c25277333p-2"), hf64!("0x1.83b54b606bd5cp-46")),
    (hf64!("0x1.3d81fb5947p-2"), hf64!("-0x1.22c7c2a9d37a4p-45")),
    (hf64!("0x1.3edf463c17p-2"), hf64!("-0x1.f067c297f2c3fp-44")),
    (hf64!("0x1.419b423d5fp-2"), hf64!("-0x1.ce379226de3ecp-44")),
    (hf64!("0x1.42f9f3ff62p-2"), hf64!("0x1.906440f7d3354p-44")),
    (hf64!("0x1.44591e053ap-2"), hf64!("-0x1.6e95892923d88p-47")),
    (hf64!("0x1.45b8c0a17ep-2"), hf64!("-0x1.d9120e7d0a853p-47")),
    (hf64!("0x1.4718dc271cp-2"), hf64!("0x1.06c18fb4c14c5p-44")),
    (hf64!("0x1.487970e958p-2"), hf64!("0x1.dc1b8465cf25fp-44")),
    (hf64!("0x1.49da7f3bccp-2"), hf64!("0x1.07b334daf4b9ap-44")),
    (hf64!("0x1.4b3c077268p-2"), hf64!("-0x1.65b4681052b9fp-46")),
    (hf64!("0x1.4c9e09e173p-2"), hf64!("-0x1.e20891b0ad8a4p-45")),
    (hf64!("0x1.4e0086dd8cp-2"), hf64!("-0x1.4d692a1e44788p-44")),
    (hf64!("0x1.4f637ebbaap-2"), hf64!("-0x1.fc158cb3124b9p-44")),
    (hf64!("0x1.50c6f1d11cp-2"), hf64!("-0x1.a0e6b7e827c2cp-44")),
    (hf64!("0x1.522ae0738ap-2"), hf64!("0x1.ebe708164c759p-45")),
    (hf64!("0x1.538f4af8f7p-2"), hf64!("0x1.7ec02e45547cep-45")),
    (hf64!("0x1.54f431b7bep-2"), hf64!("0x1.a8954c0910952p-46")),
    (hf64!("0x1.5659950695p-2"), hf64!("0x1.4c5fd2badc774p-46")),
    (hf64!("0x1.57bf753c8dp-2"), hf64!("0x1.fadedee5d40efp-46")),
    (hf64!("0x1.5925d2b113p-2"), hf64!("-0x1.69bf5a7a56f34p-44")),
    (hf64!("0x1.5a8cadbbeep-2"), hf64!("-0x1.7c79b0af7ecf8p-48")),
    (hf64!("0x1.5bf406b544p-2"), hf64!("-0x1.27023eb68981cp-45")),
    (hf64!("0x1.5d5bddf596p-2"), hf64!("-0x1.a0b2a08a465dcp-47")),
    (hf64!("0x1.5ec433d5c3p-2"), hf64!("0x1.6b71a1229d17fp-44")),
    (hf64!("0x1.602d08af09p-2"), hf64!("0x1.ebe9176df3f65p-46")),
    (hf64!("0x1.61965cdb03p-2"), hf64!("-0x1.f08ad603c488ep-45")),
    (hf64!("0x1.630030b3abp-2"), hf64!("-0x1.db623e731aep-45")),
];

// The following is a degree-7 polynomial generated by Sollya
// approximating log(1+x) for |x| < 0.00212097167968735,
// with absolute error < 2^-79.592 and relative error < 2^-70.467
// (see file Prel.sollya).
// The polynomial is P[0]*x + P[1]*x^2 + ... + P[6]*x^7.
// The algorithm assumes that the degree-1 coefficient P[0] is 1,
// and the degree-2 coefficient P[1] is -0.5.
#[rustfmt::skip]
const P: [f64; 7] = [
    hf64!("0x1p0"),                 // degree 1
    hf64!("-0x1p-1"),               // degree 2
    hf64!("0x1.555555555555p-2"),   // degree 3
    hf64!("-0x1.fffffffff572dp-3"), // degree 4
    hf64!("0x1.999999a2d7868p-3"),  // degree 5
    hf64!("-0x1.5555c0d31b08ep-3"), // degree 6
    hf64!("0x1.2476b9058e396p-3"),  // degree 7
];

// The following is a degree-11 polynomial generated by Sollya
// approximating log(1+x) for |x| < 0.03125,
// with absolute error < 2^-73.441 and relative error < 2^-67.088
// (see file Pabs_a.sollya).
// The polynomial is P[0]*x + P[1]*x^2 + ... + P[10]*x^11.
// The algorithm assumes that the degree-1 coefficient P[0] is 1
// and the degree-2 coefficient P[1] is -0.5.
#[rustfmt::skip]
const PA: [f64; 11] = [
    hf64!("0x1p0"),                // degree 1
    hf64!("-0x1p-1"),               // degree 2
    hf64!("0x1.5555555555555p-2"),  // degree 3
    hf64!("-0x1.ffffffffffe5fp-3"), // degree 4
    hf64!("0x1.999999999aa82p-3"),  // degree 5
    hf64!("-0x1.555555583a8c8p-3"), // degree 6
    hf64!("0x1.2492491c359e6p-3"),  // degree 7
    hf64!("-0x1.ffffc728edeeap-4"), // degree 8
    hf64!("0x1.c71c961f3498p-4"),   // degree 9
    hf64!("-0x1.9a82ac77c05f4p-4"), // degree 10
    hf64!("0x1.74b40dd1707d3p-4"),  // degree 11
];

// the following is a degree-17 polynomial approximating log2p1(x) for
// |x| <= 2^-5 with relative error < 2^-105.02, cf log2p1_accurate.sollya
#[rustfmt::skip]
const PACC: [f64; 24] = [
    hf64!("0x1.71547652b82fep+0"), hf64!("0x1.777d0ffda0d24p-56"),   // degree 1: Pacc[0], Pacc[1]
    hf64!("-0x1.71547652b82fep-1"), hf64!("-0x1.777d0ffd9ddb8p-57"), // degree 2: Pacc[2], Pacc[3]
    hf64!("0x1.ec709dc3a03fdp-2"), hf64!("0x1.d27f055481523p-56"),   // degree 3: Pacc[4], Pacc[5]
    hf64!("-0x1.71547652b82fep-2"), hf64!("-0x1.777d1456a14c4p-58"), // degree 4: Pacc[6], Pacc[7]
    hf64!("0x1.2776c50ef9bfep-2"), hf64!("0x1.e4b2a04f81513p-56"),   // degree 5: Pacc[8], Pacc[9]
    hf64!("-0x1.ec709dc3a03fdp-3"), hf64!("-0x1.d2072e751087ap-57"), // degree 6: Pacc[10], Pacc[11]
    hf64!("0x1.a61762a7aded9p-3"), hf64!("0x1.f90f4895378acp-58"),   // degree 7: Pacc[12], Pacc[13]
    hf64!("-0x1.71547652b8301p-3"),                         // degree 8: Pacc[14]
    hf64!("0x1.484b13d7c02aep-3"),                          // degree 9: Pacc[15]
    hf64!("-0x1.2776c50ef7591p-3"),                         // degree 10: Pacc[16]
    hf64!("0x1.0c9a84993cabbp-3"),                          // degree 11: Pacc[17]
    hf64!("-0x1.ec709de7b1612p-4"),                         // degree 12: Pacc[18]
    hf64!("0x1.c68f56ba73fd1p-4"),                          // degree 13: Pacc[19]
    hf64!("-0x1.a616c83da87e7p-4"),                         // degree 14: Pacc[20]
    hf64!("0x1.89f3042097218p-4"),                          // degree 15: Pacc[21]
    hf64!("-0x1.72b376930a3fap-4"),                         // degree 16: Pacc[22]
    hf64!("0x1.5d0211d5ab53p-4"),                           // degree 17: Pacc[23]
];

// The following table contains entries (x,h) with x an exceptional case
// and h the rounding to nearest of log2p1(x). It should be sorted by
// increasing values of x.
#[rustfmt::skip]
static EXCEPTIONS_TABLE: [(f64, f64); 247] = [
    (hf64!("-0x1.f5baee010ccc6p-6"), hf64!("-0x1.6f94484e5e1fdp-5")),
    (hf64!("-0x1.98e2aeed83e64p-7"), hf64!("-0x1.28cddfa8611e5p-6")),
    (hf64!("-0x1.7ff01253739d1p-7"), hf64!("-0x1.16967dd4eee8ep-6")),
    (hf64!("-0x1.ddabc67cec3c1p-9"), hf64!("-0x1.593238c1d2c35p-8")),
    (hf64!("-0x1.921c00a04d8a9p-9"), hf64!("-0x1.2281a6c541e39p-8")),
    (hf64!("-0x1.ae85103cce3e5p-10"), hf64!("-0x1.36cf465265029p-9")),
    (hf64!("-0x1.5955d05fd828cp-10"), hf64!("-0x1.f28ab65bc3763p-10")),
    (hf64!("-0x1.3623eff91de91p-10"), hf64!("-0x1.bfb3efcdf2bc4p-10")),
    (hf64!("-0x1.73ccf8ee62819p-12"), hf64!("-0x1.0c3ebbd1a501fp-11")),
    (hf64!("-0x1.fa5579e9a6941p-16"), hf64!("-0x1.6d3f8a9ba2e55p-15")),
    (hf64!("-0x1.75e4f1fb4d0f9p-17"), hf64!("-0x1.0db590d8385dep-16")),
    (hf64!("-0x1.b53dee1a96ca4p-18"), hf64!("-0x1.3b6786a5a9a69p-17")),
    (hf64!("-0x1.c66cb81b37ebbp-19"), hf64!("-0x1.47cc75e849dfcp-18")),
    (hf64!("-0x1.a1b96d88621b3p-19"), hf64!("-0x1.2d532b20e90b5p-18")),
    (hf64!("-0x1.bdb50ccf06b7ap-20"), hf64!("-0x1.418293ee25cd9p-19")),
    (hf64!("-0x1.77d66368613b4p-21"), hf64!("-0x1.0f1c08e43f217p-20")),
    (hf64!("-0x1.752326c780f68p-21"), hf64!("-0x1.0d296993d1368p-20")),
    (hf64!("-0x1.40e3f4e392fc8p-24"), hf64!("-0x1.cef2743b49f63p-24")),
    (hf64!("-0x1.16bba98a001c1p-24"), hf64!("-0x1.922076a30742p-24")),
    (hf64!("-0x1.51fb235627c5cp-28"), hf64!("-0x1.e79a80a1dd85bp-28")),
    (hf64!("-0x1.9b1805f30b4edp-31"), hf64!("-0x1.288aa13d73d39p-30")),
    (hf64!("-0x1.37a6f3079625dp-31"), hf64!("-0x1.c19e772379002p-31")),
    (hf64!("-0x1.672dad63d9469p-32"), hf64!("-0x1.0317abf72cb8p-31")),
    (hf64!("-0x1.025b55debc13p-32"), hf64!("-0x1.74bae4288af9fp-32")),
    (hf64!("-0x1.3278a26ed0162p-34"), hf64!("-0x1.ba24ff5dea796p-34")),
    (hf64!("-0x1.d88aedf0484a7p-36"), hf64!("-0x1.54de147b2643bp-35")),
    (hf64!("-0x1.6d4141037f32dp-37"), hf64!("-0x1.0779c878c2738p-36")),
    (hf64!("-0x1.40739786837bp-40"), hf64!("-0x1.ce50577370365p-40")),
    (hf64!("-0x1.ba43bd8edc2a3p-42"), hf64!("-0x1.3f06c76fa0047p-41")),
    (hf64!("-0x1.5892a86231e56p-42"), hf64!("-0x1.f11d1417baf8bp-42")),
    (hf64!("-0x1.21b2c54479c4dp-42"), hf64!("-0x1.a1f242e670d73p-42")),
    (hf64!("-0x1.526d1450f8572p-43"), hf64!("-0x1.e83ee27f027afp-43")),
    (hf64!("-0x1.bab64e105226ap-45"), hf64!("-0x1.3f596b7b1c5e2p-44")),
    (hf64!("-0x1.e9c2de7f3aa23p-47"), hf64!("-0x1.6149bc823c50fp-46")),
    (hf64!("-0x1.eaba84e0bd618p-49"), hf64!("-0x1.61fc60c60a648p-48")),
    (hf64!("-0x1.2516dafdf17adp-50"), hf64!("-0x1.a6d6a49f2187fp-50")),
    (hf64!("-0x1.9914a112c8dadp-51"), hf64!("-0x1.2716da024f6d9p-50")),
    (hf64!("-0x1.a052e3d5e791p-52"), hf64!("-0x1.2c506b0368099p-51")),
    (hf64!("-0x1.92ba5fbf0a946p-54"), hf64!("-0x1.2281c1a6e942bp-53")),
    (hf64!("-0x1.6f332a62b59f9p-56"), hf64!("-0x1.08e0f34d36269p-55")),
    (hf64!("-0x1.21530a306cc85p-56"), hf64!("-0x1.a16826a8e825dp-56")),
    (hf64!("-0x1.292f4e5d19917p-57"), hf64!("-0x1.acbf3cd5bd7eap-57")),
    (hf64!("-0x1.12b592f889516p-59"), hf64!("-0x1.8c525b64ed08ep-59")),
    (hf64!("-0x1.7780d5e5cf5c5p-60"), hf64!("-0x1.0ede4c1293a9bp-59")),
    (hf64!("-0x1.32ed98e196cf5p-60"), hf64!("-0x1.bacdbd3005cd7p-60")),
    (hf64!("-0x1.d73ad29747834p-61"), hf64!("-0x1.53eba1534455ap-60")),
    (hf64!("-0x1.354976dca453cp-61"), hf64!("-0x1.be34ef62d61b1p-61")),
    (hf64!("-0x1.1483eb37c9829p-61"), hf64!("-0x1.8eed6122bf62fp-61")),
    (hf64!("-0x1.fa53cebe640ccp-62"), hf64!("-0x1.6d3ced43408f1p-61")),
    (hf64!("-0x1.6923c31228cd3p-62"), hf64!("-0x1.0481d96a2dfcap-61")),
    (hf64!("-0x1.796ad95f38488p-63"), hf64!("-0x1.103fc46963aafp-62")),
    (hf64!("-0x1.470187381f305p-63"), hf64!("-0x1.d7c5178ca1765p-63")),
    (hf64!("-0x1.9234c78cc23acp-64"), hf64!("-0x1.2221636cd4353p-63")),
    (hf64!("-0x1.c7c8b78d5a272p-65"), hf64!("-0x1.48c7588c6213bp-64")),
    (hf64!("-0x1.a595e9f7d6e25p-65"), hf64!("-0x1.301c17252a9f4p-64")),
    (hf64!("-0x1.47aea7608c02bp-65"), hf64!("-0x1.d8bedc057858cp-65")),
    (hf64!("-0x1.1b19925f9fc06p-65"), hf64!("-0x1.986d43391ffdp-65")),
    (hf64!("-0x1.d62c98023cdbap-66"), hf64!("-0x1.5328b386ca8b2p-65")),
    (hf64!("-0x1.3b65311720a11p-66"), hf64!("-0x1.c704eabcff94fp-66")),
    (hf64!("-0x1.026e1ee1c55aep-68"), hf64!("-0x1.74d5fe17d2fe3p-68")),
    (hf64!("-0x1.cf96e02037643p-70"), hf64!("-0x1.4e68b66aa8cbbp-69")),
    (hf64!("-0x1.b9ab928f8614ap-70"), hf64!("-0x1.3e99035861d26p-69")),
    (hf64!("-0x1.297ee84082dafp-70"), hf64!("-0x1.ad3213e9d1878p-70")),
    (hf64!("-0x1.f63a8ce2364f3p-71"), hf64!("-0x1.6a480c34c7c5bp-70")),
    (hf64!("-0x1.ed7ff50423e4dp-71"), hf64!("-0x1.63fc211fe70aap-70")),
    (hf64!("-0x1.6856506d8234ap-71"), hf64!("-0x1.03eda6663a4b2p-70")),
    (hf64!("-0x1.62a8f9300de6fp-71"), hf64!("-0x1.ffaa928c83f63p-71")),
    (hf64!("-0x1.de8c6ef8a4dcep-73"), hf64!("-0x1.5933279c15d49p-72")),
    (hf64!("-0x1.698f4eb77100ap-73"), hf64!("-0x1.04cf6d4c9032bp-72")),
    (hf64!("-0x1.5536e12eb7335p-74"), hf64!("-0x1.ec44ae4bc644p-74")),
    (hf64!("-0x1.2b19ae19e4f3ep-74"), hf64!("-0x1.af82b29eef2ecp-74")),
    (hf64!("-0x1.7711d03d5c7bdp-75"), hf64!("-0x1.0e8e362e16fe6p-74")),
    (hf64!("-0x1.6104ba44449ep-75"), hf64!("-0x1.fd4c4933c4aadp-75")),
    (hf64!("-0x1.15108908835fp-75"), hf64!("-0x1.8fb83f0161f49p-75")),
    (hf64!("-0x1.1288bfd6f822ap-76"), hf64!("-0x1.8c11b0478bf38p-76")),
    (hf64!("-0x1.dbf55aa9a7eb9p-77"), hf64!("-0x1.5754d81696c42p-76")),
    (hf64!("-0x1.a175bf47b2093p-79"), hf64!("-0x1.2d223a2622b15p-78")),
    (hf64!("-0x1.75817dcf08a91p-81"), hf64!("-0x1.0d6d70bb89ce9p-80")),
    (hf64!("-0x1.6340571968b67p-82"), hf64!("-0x1.0042796d534fdp-81")),
    (hf64!("-0x1.06501e96466cfp-83"), hf64!("-0x1.7a70079e554fdp-83")),
    (hf64!("-0x1.86e6a6125ee5fp-85"), hf64!("-0x1.19f9b6ddcc3e9p-84")),
    (hf64!("-0x1.ce9392feefd4dp-86"), hf64!("-0x1.4dadaa8d9897dp-85")),
    (hf64!("-0x1.60fa7c8eee569p-86"), hf64!("-0x1.fd3d82e0680cdp-86")),
    (hf64!("-0x1.30876af870c12p-86"), hf64!("-0x1.b757aa6005d47p-86")),
    (hf64!("-0x1.1cf977003bdd6p-86"), hf64!("-0x1.9b219a1974289p-86")),
    (hf64!("-0x1.2c23f2bc02277p-87"), hf64!("-0x1.b102d7663223fp-87")),
    (hf64!("-0x1.05ae004891094p-87"), hf64!("-0x1.7986247373017p-87")),
    (hf64!("-0x1.b86b45d2c7c74p-88"), hf64!("-0x1.3db1f733be456p-87")),
    (hf64!("-0x1.63c4a608aac21p-89"), hf64!("-0x1.00a1ea24493b7p-88")),
    (hf64!("-0x1.ffc87ce076dfap-91"), hf64!("-0x1.712c6b2a267f5p-90")),
    (hf64!("-0x1.127630515eb89p-91"), hf64!("-0x1.8bf6e9484dd46p-91")),
    (hf64!("-0x1.db7daf1e016dfp-92"), hf64!("-0x1.56fe8536a47e9p-91")),
    (hf64!("-0x1.ca7e01c959788p-92"), hf64!("-0x1.4abb72eb058bep-91")),
    (hf64!("-0x1.4106468e7b671p-94"), hf64!("-0x1.cf23f6232620ep-94")),
    (hf64!("-0x1.cd79e208d957ep-95"), hf64!("-0x1.4ce2780c1bb7bp-94")),
    (hf64!("-0x1.0c5a1aeb10a7cp-95"), hf64!("-0x1.83266a65e67b3p-95")),
    (hf64!("-0x1.de8de3ca3cb0fp-97"), hf64!("-0x1.5934348aa44c5p-96")),
    (hf64!("-0x1.8ba5360a2b553p-99"), hf64!("-0x1.1d65d5fc31246p-98")),
    (hf64!("-0x1.618b50a22cecp-105"), hf64!("-0x1.fe0e7458ac1fcp-105")),
    (hf64!("-0x1.09287c79a1b1p-105"), hf64!("-0x1.7e8ad7428117dp-105")),
    (hf64!("-0x1.618b50a22cecp-106"), hf64!("-0x1.fe0e7458ac1fcp-106")),
    (hf64!("-0x1.09287c79a1b1p-106"), hf64!("-0x1.7e8ad7428117dp-106")),
    (hf64!("0x1.198cc57cb19cfp-106"), hf64!("0x1.9630ccfb659e1p-106")),
    (hf64!("0x1.71ef99a53cd7fp-106"), hf64!("0x1.0ada3508c853p-105")),
    (hf64!("0x1.198cc57cb19cfp-105"), hf64!("0x1.9630ccfb659e1p-105")),
    (hf64!("0x1.71ef99a53cd7fp-105"), hf64!("0x1.0ada3508c853p-104")),
    (hf64!("0x1.198cc57cb19cfp-104"), hf64!("0x1.9630ccfb659e1p-104")),
    (hf64!("0x1.71ef99a53cd7fp-104"), hf64!("0x1.0ada3508c853p-103")),
    (hf64!("0x1.198cc57cb19cfp-103"), hf64!("0x1.9630ccfb659e1p-103")),
    (hf64!("0x1.8253e2a84cc3ep-103"), hf64!("0x1.16ad2fe53a962p-102")),
    (hf64!("0x1.dab6b6d0d7feep-103"), hf64!("0x1.566efe70501a1p-102")),
    (hf64!("0x1.29f10e7fc188ep-102"), hf64!("0x1.add6c2b44a245p-102")),
    (hf64!("0x1.92b82bab5cafdp-102"), hf64!("0x1.22802ac1acd94p-101")),
    (hf64!("0x1.4ab9a085e160cp-101"), hf64!("0x1.dd22ae261330dp-101")),
    (hf64!("0x1.d4494fb79c5f9p-101"), hf64!("0x1.51cc163375e5cp-100")),
    (hf64!("0x1.e4ad98baac4b8p-101"), hf64!("0x1.5d9f110fe828ep-100")),
    (hf64!("0x1.f511e1bdbc377p-101"), hf64!("0x1.69720bec5a6cp-100")),
    (hf64!("0x1.ebc08e5bdda62p-99"), hf64!("0x1.62b965d563ddcp-98")),
    (hf64!("0x1.6c27c113f6f3fp-98"), hf64!("0x1.06aeb9548f9f3p-97")),
    (hf64!("0x1.af040230221a5p-97"), hf64!("0x1.36e97dd79baccp-96")),
    (hf64!("0x1.e743d2da8338cp-97"), hf64!("0x1.5f7cc3326e2bp-96")),
    (hf64!("0x1.34e55b50b5476p-95"), hf64!("0x1.bda482a6d3947p-95")),
    (hf64!("0x1.4265b80b22415p-95"), hf64!("0x1.d11efcab7f66ep-95")),
    (hf64!("0x1.4fe614c58f3b4p-95"), hf64!("0x1.e49976b02b395p-95")),
    (hf64!("0x1.7bf0a5c56ed66p-95"), hf64!("0x1.12119cc1d532cp-94")),
    (hf64!("0x1.d896d7b343ddap-94"), hf64!("0x1.54e6ac6b33b3ap-93")),
    (hf64!("0x1.4b7dadb0f0318p-93"), hf64!("0x1.de3d85d01317bp-93")),
    (hf64!("0x1.6201aba46fb8p-93"), hf64!("0x1.feb934937f571p-93")),
    (hf64!("0x1.05221bfa8a0fap-92"), hf64!("0x1.78bc523768e9ap-92")),
    (hf64!("0x1.3d5b61f8af06p-91"), hf64!("0x1.c9d96ce244a42p-91")),
    (hf64!("0x1.572552ca58e6ep-91"), hf64!("0x1.ef0e032ee98acp-91")),
    (hf64!("0x1.3a41b77e1cfcap-90"), hf64!("0x1.c560684cd2e49p-90")),
    (hf64!("0x1.94d5d40edd1e3p-90"), hf64!("0x1.2406e3f8308dap-89")),
    (hf64!("0x1.02f8c70e9f28p-89"), hf64!("0x1.759e083fbdf8ap-89")),
    (hf64!("0x1.61de0b2528516p-89"), hf64!("0x1.fe85ce7ae82bp-89")),
    (hf64!("0x1.64cc21a428f53p-89"), hf64!("0x1.015ffa3105406p-88")),
    (hf64!("0x1.31cdce0e38005p-88"), hf64!("0x1.b92e8ae1c84e8p-88")),
    (hf64!("0x1.5b9ab5b571503p-88"), hf64!("0x1.f57caf57c945dp-88")),
    (hf64!("0x1.7654fc15c252dp-88"), hf64!("0x1.0e0600211a7d8p-87")),
    (hf64!("0x1.abf231afdb1f5p-88"), hf64!("0x1.34b2a57159769p-87")),
    (hf64!("0x1.8586b6e5cbb06p-87"), hf64!("0x1.18fbd8ef99db2p-86")),
    (hf64!("0x1.ae2b54125b144p-87"), hf64!("0x1.364d309c81b79p-86")),
    (hf64!("0x1.1d640ce51c6b5p-86"), hf64!("0x1.9bbb5f52ce771p-86")),
    (hf64!("0x1.3a0e97e84bb1ep-86"), hf64!("0x1.c516a6e7e1fb3p-86")),
    (hf64!("0x1.742efa2dfc234p-86"), hf64!("0x1.0c79410624d52p-85")),
    (hf64!("0x1.63eb00801d3d6p-85"), hf64!("0x1.00bd94ab13e1ep-84")),
    (hf64!("0x1.a8b9097e3a9aep-85"), hf64!("0x1.325f6bda4b8f1p-84")),
    (hf64!("0x1.807d37ffa6069p-83"), hf64!("0x1.1559ac41e95fdp-82")),
    (hf64!("0x1.3528bc4882f1fp-82"), hf64!("0x1.be05b7a6f8ce9p-82")),
    (hf64!("0x1.982a8e52379abp-82"), hf64!("0x1.266e00e15b035p-81")),
    (hf64!("0x1.317c7aec05f2cp-81"), hf64!("0x1.b8b93738a057ap-81")),
    (hf64!("0x1.4376f83fcc3c7p-81"), hf64!("0x1.d2a9345ec5951p-81")),
    (hf64!("0x1.569aad73361dap-80"), hf64!("0x1.ee45fd1dfacc6p-80")),
    (hf64!("0x1.575476e5e84cp-80"), hf64!("0x1.ef5205c84626cp-80")),
    (hf64!("0x1.f20aa391bfeabp-80"), hf64!("0x1.6742d3ba4b4b5p-79")),
    (hf64!("0x1.181b21bfde59bp-79"), hf64!("0x1.941b8603a63b6p-79")),
    (hf64!("0x1.0934d6a88672dp-78"), hf64!("0x1.7e9ca95365c5ap-78")),
    (hf64!("0x1.0f9c8867c5241p-78"), hf64!("0x1.87da3d843fa48p-78")),
    (hf64!("0x1.4c5324339bb48p-78"), hf64!("0x1.df717c0ff8c99p-78")),
    (hf64!("0x1.f7d8b04b61989p-78"), hf64!("0x1.6b72c90b97cb1p-77")),
    (hf64!("0x1.8f27d6bf246acp-75"), hf64!("0x1.1fee0f0ca892p-74")),
    (hf64!("0x1.f83edae3aa93fp-75"), hf64!("0x1.6bbc7b98f1b2ep-74")),
    (hf64!("0x1.cda9fe2248ec6p-74"), hf64!("0x1.4d052c3c2b0e3p-73")),
    (hf64!("0x1.c0ee5d2d2d30ap-73"), hf64!("0x1.43d5d901cfa0fp-72")),
    (hf64!("0x1.628c0b5b4b9bbp-72"), hf64!("0x1.ff80d6316f1efp-72")),
    (hf64!("0x1.f018c61839599p-72"), hf64!("0x1.65dbb17ad1651p-71")),
    (hf64!("0x1.87a0cd7eeca98p-71"), hf64!("0x1.1a7ffefd05629p-70")),
    (hf64!("0x1.5c5f1081b8abdp-70"), hf64!("0x1.f697f700c874dp-70")),
    (hf64!("0x1.77e590243e48fp-69"), hf64!("0x1.0f26f4ea680bep-68")),
    (hf64!("0x1.c943ea0da88dfp-69"), hf64!("0x1.49d8e0ffc588cp-68")),
    (hf64!("0x1.d33e23beb1528p-68"), hf64!("0x1.510b5cef9b5bcp-67")),
    (hf64!("0x1.186615833f8f7p-67"), hf64!("0x1.9487a81ae609dp-67")),
    (hf64!("0x1.4891077ffbab6p-66"), hf64!("0x1.da057342f40fbp-66")),
    (hf64!("0x1.4193614150725p-64"), hf64!("0x1.cfef882ce5ca8p-64")),
    (hf64!("0x1.572569359da9p-64"), hf64!("0x1.ef0e2386ed35dp-64")),
    (hf64!("0x1.6a5f8252d9f3bp-64"), hf64!("0x1.05659cd6a65d1p-63")),
    (hf64!("0x1.7b5eedadbdbd6p-64"), hf64!("0x1.11a87f8ae134dp-63")),
    (hf64!("0x1.b2669d6e81f95p-64"), hf64!("0x1.395a9dafcdcabp-63")),
    (hf64!("0x1.02754e0d621p-63"), hf64!("0x1.74e05b747c3f1p-63")),
    (hf64!("0x1.42c2504353e38p-63"), hf64!("0x1.d1a492a13d773p-63")),
    (hf64!("0x1.9a8cb37e9acd5p-63"), hf64!("0x1.28262152f2ee4p-62")),
    (hf64!("0x1.bd1c7919023cp-63"), hf64!("0x1.411472cfddceap-62")),
    (hf64!("0x1.e505bd69056d9p-63"), hf64!("0x1.5ddea609f2b25p-62")),
    (hf64!("0x1.2dbbb07d596fep-62"), hf64!("0x1.b34f1670999b4p-62")),
    (hf64!("0x1.9a2e439736a07p-62"), hf64!("0x1.27e2021880394p-61")),
    (hf64!("0x1.112c460eafdcp-61"), hf64!("0x1.8a1af1c4d551fp-61")),
    (hf64!("0x1.7dcdac46803eep-61"), hf64!("0x1.1369b6a328fcdp-60")),
    (hf64!("0x1.4b2ed4f196fe5p-60"), hf64!("0x1.ddcbc560b3595p-60")),
    (hf64!("0x1.bc6526d12f2a9p-60"), hf64!("0x1.409035c53cd34p-59")),
    (hf64!("0x1.95242afed5822p-59"), hf64!("0x1.243f668c8121dp-58")),
    (hf64!("0x1.158d3212d17dfp-58"), hf64!("0x1.906c17ca52236p-58")),
    (hf64!("0x1.f2bb88afdad28p-58"), hf64!("0x1.67c26e08f6021p-57")),
    (hf64!("0x1.057d5711bfd9ep-57"), hf64!("0x1.793ff07d6bc71p-57")),
    (hf64!("0x1.18cebea44da63p-57"), hf64!("0x1.951ea66b69f7fp-57")),
    (hf64!("0x1.4cd45d85f97bp-57"), hf64!("0x1.e02bea523b28cp-57")),
    (hf64!("0x1.596a35cd55af7p-57"), hf64!("0x1.f2540e0d3138dp-57")),
    (hf64!("0x1.9b7e25fae77ffp-56"), hf64!("0x1.28d44c2a1cfe7p-55")),
    (hf64!("0x1.67c00f810444cp-55"), hf64!("0x1.038143d2697e2p-54")),
    (hf64!("0x1.e62ce3c0305e7p-54"), hf64!("0x1.5eb38ddc44f75p-53")),
    (hf64!("0x1.b2078ba8f6835p-52"), hf64!("0x1.391609b20beaap-51")),
    (hf64!("0x1.cbe169f09f95ep-52"), hf64!("0x1.4bbbd21c8c721p-51")),
    (hf64!("0x1.9ce011cf5f46ep-51"), hf64!("0x1.29d3990338b4ep-50")),
    (hf64!("0x1.9b37cdd819b7cp-48"), hf64!("0x1.28a18e0a504b8p-47")),
    (hf64!("0x1.ed87135915039p-48"), hf64!("0x1.640143a8bde38p-47")),
    (hf64!("0x1.389bb3cfc42c2p-47"), hf64!("0x1.c2ff91c241999p-47")),
    (hf64!("0x1.56cd7d201444cp-46"), hf64!("0x1.ee8f4b39c7db4p-46")),
    (hf64!("0x1.fc53c8d303c2p-46"), hf64!("0x1.6eae3d74765fbp-45")),
    (hf64!("0x1.396ae5fe7edc6p-44"), hf64!("0x1.c42a7d73d80c7p-44")),
    (hf64!("0x1.57c17dadff53fp-44"), hf64!("0x1.efef50874d46dp-44")),
    (hf64!("0x1.8e05d9bf54ff5p-43"), hf64!("0x1.1f1ce05fc5585p-42")),
    (hf64!("0x1.7a55eea45459dp-39"), hf64!("0x1.10e95805820a9p-38")),
    (hf64!("0x1.d204a5710c6b4p-38"), hf64!("0x1.502939b175bc4p-37")),
    (hf64!("0x1.0477b177d47a9p-37"), hf64!("0x1.77c676774427ep-37")),
    (hf64!("0x1.912cc5483c4bbp-37"), hf64!("0x1.2162f23d082d5p-36")),
    (hf64!("0x1.32ba6bfc6df61p-36"), hf64!("0x1.ba83e89742ee7p-36")),
    (hf64!("0x1.b881f93d3d7bcp-35"), hf64!("0x1.3dc2574084694p-34")),
    (hf64!("0x1.33c28a8adb35dp-33"), hf64!("0x1.bc00f3c633b0ap-33")),
    (hf64!("0x1.1bab50d82fdf7p-30"), hf64!("0x1.993f86d739365p-30")),
    (hf64!("0x1.5b8a987b105b9p-29"), hf64!("0x1.f5656fdaf6efcp-29")),
    (hf64!("0x1.a7915a8ef7252p-29"), hf64!("0x1.318a2177ecf63p-28")),
    (hf64!("0x1.51cba0164bed2p-27"), hf64!("0x1.e755f4897c124p-27")),
    (hf64!("0x1.1859f2f12c686p-26"), hf64!("0x1.9476260e06ba5p-26")),
    (hf64!("0x1.5004cdd3ec269p-26"), hf64!("0x1.e4c5c9414182ep-26")),
    (hf64!("0x1.970634f93920ap-25"), hf64!("0x1.259b1dcdcb00fp-24")),
    (hf64!("0x1.446805a9fe779p-23"), hf64!("0x1.d404f5eb9474dp-23")),
    (hf64!("0x1.4f86281412f0bp-21"), hf64!("0x1.e40f08f4a16c4p-21")),
    (hf64!("0x1.53e1ee4cd607ep-21"), hf64!("0x1.ea58c18ad7665p-21")),
    (hf64!("0x1.91848ab4215a3p-20"), hf64!("0x1.21a2344dd6315p-19")),
    (hf64!("0x1.7dbc650f5516cp-19"), hf64!("0x1.135d264c757b5p-18")),
    (hf64!("0x1.5ed15c24566e4p-16"), hf64!("0x1.fa1e29df34a0fp-16")),
    (hf64!("0x1.66e3866f48983p-14"), hf64!("0x1.02df58d57f81fp-13")),
    (hf64!("0x1.4a82829ba33efp-13"), hf64!("0x1.dcc98bad34bd4p-13")),
    (hf64!("0x1.ddacfe52ca881p-13"), hf64!("0x1.5887eeb1071c7p-12")),
    (hf64!("0x1.cee52926a8cdep-11"), hf64!("0x1.4dc2ce20fe5dbp-10")),
    (hf64!("0x1.50878a3127c53p-10"), hf64!("0x1.e532b0254b97ep-10")),
    (hf64!("0x1.c60c6b10503bdp-10"), hf64!("0x1.473e526fad0ap-9")),
    (hf64!("0x1.cb331784430e1p-10"), hf64!("0x1.4af3e3d364e57p-9")),
    (hf64!("0x1.d19f229d1451dp-10"), hf64!("0x1.4f93ba71b12bap-9")),
    (hf64!("0x1.15862abd340e8p-9"), hf64!("0x1.8ff59817c7989p-9")),
    (hf64!("0x1.6d42e64d940acp-9"), hf64!("0x1.071d292d678dfp-8")),
    (hf64!("0x1.df670b0e00641p-9"), hf64!("0x1.592f57b46d345p-8")),
    (hf64!("0x1.fe604d486351fp-9"), hf64!("0x1.6f719430581c3p-8")),
    (hf64!("0x1.0b896d69a6a9ap-7"), hf64!("0x1.80682bd3a1419p-7")),
    (hf64!("0x1.c91984e0cebe7p-7"), hf64!("0x1.4772f8c94aecdp-6")),
    (hf64!("0x1.cdfe4fcde7278p-7"), hf64!("0x1.4aee2c1be2fb5p-6")),
    (hf64!("0x1.fa7ca1e176885p-7"), hf64!("0x1.6a8ee6a3521fdp-6")),
    (hf64!("0x1.2cbfdc5da28d9p-6"), hf64!("0x1.adf49cda3ad37p-6")),
];

// the following table contains 0 if log2p1(x) is exact, -1 if it should be
// rounded down with respect to the value in the first table, and +1 if it
// should be rounded up
#[rustfmt::skip]
static EXCEPTIONS_RND: [i8; 247] = [
    1, // -0x1.f5baee010ccc6p-6
    1, // -0x1.98e2aeed83e64p-7
    -1, // -0x1.7ff01253739d1p-7
    1, // -0x1.ddabc67cec3c1p-9
    -1, // -0x1.921c00a04d8a9p-9
    -1, // -0x1.ae85103cce3e5p-10
    1, // -0x1.5955d05fd828cp-10
    -1, // -0x1.3623eff91de91p-10
    -1, // -0x1.73ccf8ee62819p-12
    1, // -0x1.fa5579e9a6941p-16
    -1, // -0x1.75e4f1fb4d0f9p-17
    1, // -0x1.b53dee1a96ca4p-18
    1, // -0x1.c66cb81b37ebbp-19
    1, // -0x1.a1b96d88621b3p-19
    -1, // -0x1.bdb50ccf06b7ap-20
    -1, // -0x1.77d66368613b4p-21
    -1, // -0x1.752326c780f68p-21
    -1, // -0x1.40e3f4e392fc8p-24
    -1, // -0x1.16bba98a001c1p-24
    1, // -0x1.51fb235627c5cp-28
    -1, // -0x1.9b1805f30b4edp-31
    1, // -0x1.37a6f3079625dp-31
    1, // -0x1.672dad63d9469p-32
    1, // -0x1.025b55debc13p-32
    -1, // -0x1.3278a26ed0162p-34
    1, // -0x1.d88aedf0484a7p-36
    -1, // -0x1.6d4141037f32dp-37
    -1, // -0x1.40739786837bp-40
    1, // -0x1.ba43bd8edc2a3p-42
    1, // -0x1.5892a86231e56p-42
    -1, // -0x1.21b2c54479c4dp-42
    1, // -0x1.526d1450f8572p-43
    1, // -0x1.bab64e105226ap-45
    1, // -0x1.e9c2de7f3aa23p-47
    1, // -0x1.eaba84e0bd618p-49
    1, // -0x1.2516dafdf17adp-50
    -1, // -0x1.9914a112c8dadp-51
    1, // -0x1.a052e3d5e791p-52
    1, // -0x1.92ba5fbf0a946p-54
    1, // -0x1.6f332a62b59f9p-56
    1, // -0x1.21530a306cc85p-56
    1, // -0x1.292f4e5d19917p-57
    1, // -0x1.12b592f889516p-59
    1, // -0x1.7780d5e5cf5c5p-60
    1, // -0x1.32ed98e196cf5p-60
    1, // -0x1.d73ad29747834p-61
    1, // -0x1.354976dca453cp-61
    1, // -0x1.1483eb37c9829p-61
    1, // -0x1.fa53cebe640ccp-62
    1, // -0x1.6923c31228cd3p-62
    1, // -0x1.796ad95f38488p-63
    1, // -0x1.470187381f305p-63
    1, // -0x1.9234c78cc23acp-64
    -1, // -0x1.c7c8b78d5a272p-65
    1, // -0x1.a595e9f7d6e25p-65
    1, // -0x1.47aea7608c02bp-65
    1, // -0x1.1b19925f9fc06p-65
    1, // -0x1.d62c98023cdbap-66
    -1, // -0x1.3b65311720a11p-66
    1, // -0x1.026e1ee1c55aep-68
    1, // -0x1.cf96e02037643p-70
    1, // -0x1.b9ab928f8614ap-70
    1, // -0x1.297ee84082dafp-70
    1, // -0x1.f63a8ce2364f3p-71
    1, // -0x1.ed7ff50423e4dp-71
    1, // -0x1.6856506d8234ap-71
    1, // -0x1.62a8f9300de6fp-71
    1, // -0x1.de8c6ef8a4dcep-73
    1, // -0x1.698f4eb77100ap-73
    1, // -0x1.5536e12eb7335p-74
    1, // -0x1.2b19ae19e4f3ep-74
    1, // -0x1.7711d03d5c7bdp-75
    1, // -0x1.6104ba44449ep-75
    1, // -0x1.15108908835fp-75
    1, // -0x1.1288bfd6f822ap-76
    1, // -0x1.dbf55aa9a7eb9p-77
    -1, // -0x1.a175bf47b2093p-79
    -1, // -0x1.75817dcf08a91p-81
    1, // -0x1.6340571968b67p-82
    -1, // -0x1.06501e96466cfp-83
    1, // -0x1.86e6a6125ee5fp-85
    1, // -0x1.ce9392feefd4dp-86
    1, // -0x1.60fa7c8eee569p-86
    1, // -0x1.30876af870c12p-86
    1, // -0x1.1cf977003bdd6p-86
    1, // -0x1.2c23f2bc02277p-87
    -1, // -0x1.05ae004891094p-87
    1, // -0x1.b86b45d2c7c74p-88
    1, // -0x1.63c4a608aac21p-89
    1, // -0x1.ffc87ce076dfap-91
    1, // -0x1.127630515eb89p-91
    1, // -0x1.db7daf1e016dfp-92
    1, // -0x1.ca7e01c959788p-92
    1, // -0x1.4106468e7b671p-94
    -1, // -0x1.cd79e208d957ep-95
    1, // -0x1.0c5a1aeb10a7cp-95
    -1, // -0x1.de8de3ca3cb0fp-97
    1, // -0x1.8ba5360a2b553p-99
    1, // -0x1.618b50a22cecp-105
    1, // -0x1.09287c79a1b1p-105
    1, // -0x1.618b50a22cecp-106
    1, // -0x1.09287c79a1b1p-106
    1, // 0x1.198cc57cb19cfp-106
    1, // 0x1.71ef99a53cd7fp-106
    1, // 0x1.198cc57cb19cfp-105
    1, // 0x1.71ef99a53cd7fp-105
    1, // 0x1.198cc57cb19cfp-104
    1, // 0x1.71ef99a53cd7fp-104
    1, // 0x1.198cc57cb19cfp-103
    1, // 0x1.8253e2a84cc3ep-103
    1, // 0x1.dab6b6d0d7feep-103
    1, // 0x1.29f10e7fc188ep-102
    1, // 0x1.92b82bab5cafdp-102
    1, // 0x1.4ab9a085e160cp-101
    -1, // 0x1.d4494fb79c5f9p-101
    1, // 0x1.e4ad98baac4b8p-101
    1, // 0x1.f511e1bdbc377p-101
    1, // 0x1.ebc08e5bdda62p-99
    1, // 0x1.6c27c113f6f3fp-98
    1, // 0x1.af040230221a5p-97
    1, // 0x1.e743d2da8338cp-97
    1, // 0x1.34e55b50b5476p-95
    1, // 0x1.4265b80b22415p-95
    1, // 0x1.4fe614c58f3b4p-95
    1, // 0x1.7bf0a5c56ed66p-95
    1, // 0x1.d896d7b343ddap-94
    1, // 0x1.4b7dadb0f0318p-93
    1, // 0x1.6201aba46fb8p-93
    1, // 0x1.05221bfa8a0fap-92
    1, // 0x1.3d5b61f8af06p-91
    1, // 0x1.572552ca58e6ep-91
    1, // 0x1.3a41b77e1cfcap-90
    1, // 0x1.94d5d40edd1e3p-90
    1, // 0x1.02f8c70e9f28p-89
    1, // 0x1.61de0b2528516p-89
    1, // 0x1.64cc21a428f53p-89
    1, // 0x1.31cdce0e38005p-88
    1, // 0x1.5b9ab5b571503p-88
    1, // 0x1.7654fc15c252dp-88
    1, // 0x1.abf231afdb1f5p-88
    1, // 0x1.8586b6e5cbb06p-87
    -1, // 0x1.ae2b54125b144p-87
    -1, // 0x1.1d640ce51c6b5p-86
    1, // 0x1.3a0e97e84bb1ep-86
    1, // 0x1.742efa2dfc234p-86
    1, // 0x1.63eb00801d3d6p-85
    1, // 0x1.a8b9097e3a9aep-85
    1, // 0x1.807d37ffa6069p-83
    -1, // 0x1.3528bc4882f1fp-82
    1, // 0x1.982a8e52379abp-82
    1, // 0x1.317c7aec05f2cp-81
    1, // 0x1.4376f83fcc3c7p-81
    1, // 0x1.569aad73361dap-80
    1, // 0x1.575476e5e84cp-80
    1, // 0x1.f20aa391bfeabp-80
    1, // 0x1.181b21bfde59bp-79
    1, // 0x1.0934d6a88672dp-78
    1, // 0x1.0f9c8867c5241p-78
    1, // 0x1.4c5324339bb48p-78
    1, // 0x1.f7d8b04b61989p-78
    1, // 0x1.8f27d6bf246acp-75
    1, // 0x1.f83edae3aa93fp-75
    1, // 0x1.cda9fe2248ec6p-74
    1, // 0x1.c0ee5d2d2d30ap-73
    -1, // 0x1.628c0b5b4b9bbp-72
    1, // 0x1.f018c61839599p-72
    1, // 0x1.87a0cd7eeca98p-71
    1, // 0x1.5c5f1081b8abdp-70
    1, // 0x1.77e590243e48fp-69
    1, // 0x1.c943ea0da88dfp-69
    1, // 0x1.d33e23beb1528p-68
    1, // 0x1.186615833f8f7p-67
    1, // 0x1.4891077ffbab6p-66
    1, // 0x1.4193614150725p-64
    1, // 0x1.572569359da9p-64
    1, // 0x1.6a5f8252d9f3bp-64
    1, // 0x1.7b5eedadbdbd6p-64
    1, // 0x1.b2669d6e81f95p-64
    1, // 0x1.02754e0d621p-63
    1, // 0x1.42c2504353e38p-63
    1, // 0x1.9a8cb37e9acd5p-63
    1, // 0x1.bd1c7919023cp-63
    1, // 0x1.e505bd69056d9p-63
    1, // 0x1.2dbbb07d596fep-62
    1, // 0x1.9a2e439736a07p-62
    1, // 0x1.112c460eafdcp-61
    1, // 0x1.7dcdac46803eep-61
    -1, // 0x1.4b2ed4f196fe5p-60
    1, // 0x1.bc6526d12f2a9p-60
    1, // 0x1.95242afed5822p-59
    1, // 0x1.158d3212d17dfp-58
    1, // 0x1.f2bb88afdad28p-58
    1, // 0x1.057d5711bfd9ep-57
    1, // 0x1.18cebea44da63p-57
    1, // 0x1.4cd45d85f97bp-57
    1, // 0x1.596a35cd55af7p-57
    -1, // 0x1.9b7e25fae77ffp-56
    1, // 0x1.67c00f810444cp-55
    1, // 0x1.e62ce3c0305e7p-54
    1, // 0x1.b2078ba8f6835p-52
    1, // 0x1.cbe169f09f95ep-52
    1, // 0x1.9ce011cf5f46ep-51
    1, // 0x1.9b37cdd819b7cp-48
    1, // 0x1.ed87135915039p-48
    1, // 0x1.389bb3cfc42c2p-47
    -1, // 0x1.56cd7d201444cp-46
    -1, // 0x1.fc53c8d303c2p-46
    -1, // 0x1.396ae5fe7edc6p-44
    -1, // 0x1.57c17dadff53fp-44
    1, // 0x1.8e05d9bf54ff5p-43
    1, // 0x1.7a55eea45459dp-39
    -1, // 0x1.d204a5710c6b4p-38
    -1, // 0x1.0477b177d47a9p-37
    1, // 0x1.912cc5483c4bbp-37
    -1, // 0x1.32ba6bfc6df61p-36
    -1, // 0x1.b881f93d3d7bcp-35
    1, // 0x1.33c28a8adb35dp-33
    -1, // 0x1.1bab50d82fdf7p-30
    -1, // 0x1.5b8a987b105b9p-29
    1, // 0x1.a7915a8ef7252p-29
    1, // 0x1.51cba0164bed2p-27
    -1, // 0x1.1859f2f12c686p-26
    1, // 0x1.5004cdd3ec269p-26
    1, // 0x1.970634f93920ap-25
    1, // 0x1.446805a9fe779p-23
    -1, // 0x1.4f86281412f0bp-21
    -1, // 0x1.53e1ee4cd607ep-21
    1, // 0x1.91848ab4215a3p-20
    1, // 0x1.7dbc650f5516cp-19
    -1, // 0x1.5ed15c24566e4p-16
    1, // 0x1.66e3866f48983p-14
    -1, // 0x1.4a82829ba33efp-13
    -1, // 0x1.ddacfe52ca881p-13
    -1, // 0x1.cee52926a8cdep-11
    1, // 0x1.50878a3127c53p-10
    -1, // 0x1.c60c6b10503bdp-10
    -1, // 0x1.cb331784430e1p-10
    -1, // 0x1.d19f229d1451dp-10
    -1, // 0x1.15862abd340e8p-9
    -1, // 0x1.6d42e64d940acp-9
    1, // 0x1.df670b0e00641p-9
    1, // 0x1.fe604d486351fp-9
    1, // 0x1.0b896d69a6a9ap-7
    1, // 0x1.c91984e0cebe7p-7
    -1, // 0x1.cdfe4fcde7278p-7
    -1, // 0x1.fa7ca1e176885p-7
    1, // 0x1.2cbfdc5da28d9p-6
];

#[cfg(test)]
mod tests {
    use super::super::ldexp;
    use super::super::support::Float;
    use super::{log2p1};

    #[test]
    fn known_results() {
        for (x, expected) in [
            (f64::ZERO, f64::ZERO),
            (f64::NEG_ZERO, f64::NEG_ZERO),
            (f64::ONE, f64::ONE),
            (f64::NEG_ONE, f64::NEG_INFINITY),
            (f64::INFINITY, f64::INFINITY),
            (f64::MAX, 1024.0),
            (f64::from_bits(1), f64::from_bits(1)),
            (f64::from_bits(2), f64::from_bits(3)),
            (-f64::from_bits(1), -f64::from_bits(1)),
            (-f64::from_bits(2), -f64::from_bits(3)),
            (f64::MIN_POSITIVE, hf64!("0x1.71547652b82fep-1022")),
            (-f64::MIN_POSITIVE, hf64!("-0x1.71547652b82fep-1022")),
            (
                hf64!("0x0.2c316a14459d8p-1022"),
                hf64!("0x1.fe0e7458ac1f8p-1025"),
            ),
            (
                hf64!("-0x0.2c316a14459d8p-1022"),
                hf64!("-0x1.fe0e7458ac1f8p-1025"),
            ),
            (0.5, hf64!("0x1.2b803473f7ad1p-1")),
            (2.0, hf64!("0x1.95c01a39fbd68p+0")),
        ] {
            assert_biteq!(log2p1(x), expected, "x = {x:?}");
        }

        for x in [
            f64::NEG_INFINITY,
            f64::MIN,
            -2.0,
            f64::from_bits(0xbff0000000000001),
            f64::NAN,
            f64::NEG_NAN,
            f64::SNAN,
            f64::NEG_SNAN,
        ] {
            assert!(log2p1(x).is_nan(), "x = {x:?}");
        }

        for n in 1..=53 {
            let expected = f64::from(n);
            let positive = ldexp(1.0, n) - 1.0;
            let negative = ldexp(1.0, -n) - 1.0;
            assert_biteq!(log2p1(positive), expected, "x = {positive:?}");
            assert_biteq!(log2p1(negative), -expected, "x = {negative:?}");
        }
    }
}
