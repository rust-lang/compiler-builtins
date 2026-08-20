use super::super::sqrt;

const SPLIT: f64 = 134217728. + 1.; // 0x1p27 + 1 === (2 ^ 27) + 1

fn sq(x: f64) -> (f64, f64) {
    let xh: f64;
    let xl: f64;
    let xc: f64;

    xc = x * SPLIT;
    xh = x - xc + xc;
    xl = x - xh;
    let hi = x * x;
    let lo = xh * xh - hi + 2. * xh * xl + xl * xl;
    (hi, lo)
}

#[cfg_attr(assert_no_panic, no_panic::no_panic)]
pub fn hypotf64(mut x: f64, mut y: f64) -> f64 {
    let x1p700 = f64::from_bits(0x6bb0000000000000); // 0x1p700 === 2 ^ 700
    let x1p_700 = f64::from_bits(0x1430000000000000); // 0x1p-700 === 2 ^ -700

    let mut uxi = x.to_bits();
    let mut uyi = y.to_bits();
    let uti;
    let ex: i64;
    let ey: i64;
    let mut z: f64;

    /* arrange |x| >= |y| */
    uxi &= -1i64 as u64 >> 1;
    uyi &= -1i64 as u64 >> 1;
    if uxi < uyi {
        uti = uxi;
        uxi = uyi;
        uyi = uti;
    }

    /* special cases */
    ex = (uxi >> 52) as i64;
    ey = (uyi >> 52) as i64;
    x = f64::from_bits(uxi);
    y = f64::from_bits(uyi);
    /* note: hypot(inf,nan) == inf */
    if ey == 0x7ff {
        return y;
    }
    if ex == 0x7ff || uyi == 0 {
        return x;
    }
    /* note: hypot(x,y) ~= x + y*y/x/2 with inexact for small y/x */
    /* 64 difference is enough for ld80 double_t */
    if ex - ey > 64 {
        return x + y;
    }

    /* precise sqrt argument in nearest rounding mode without overflow */
    /* xh*xh must not overflow and xl*xl must not underflow in sq */
    z = 1.;
    if ex > 0x3ff + 510 {
        z = x1p700;
        x *= x1p_700;
        y *= x1p_700;
    } else if ey < 0x3ff - 450 {
        z = x1p_700;
        x *= x1p700;
        y *= x1p700;
    }
    let (hx, lx) = sq(x);
    let (hy, ly) = sq(y);
    z * sqrt(ly + lx + hy + hx)
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
        // hypot(±0, ±0) is +0.
        assert_biteq!(hypotf64(0.0, 0.0), 0.0);
        assert_biteq!(hypotf64(-0.0, 0.0), 0.0);
        assert_biteq!(hypotf64(0.0, -0.0), 0.0);

        // hypot(±inf, qNaN) and hypot(qNaN, ±inf) are +inf, while hypot(x, NaN)
        // for finite x is NaN.
        assert_biteq!(hypotf64(f64::INFINITY, f64::NAN), f64::INFINITY);
        assert_biteq!(hypotf64(f64::NAN, f64::INFINITY), f64::INFINITY);
        assert_biteq!(hypotf64(1.0, f64::INFINITY), f64::INFINITY);
        assert!(hypotf64(1.0, f64::NAN).is_nan());
        assert!(hypotf64(f64::NAN, 1.0).is_nan());

        // Large-magnitude arguments overflow to infinity, while subnormals
        // must not underflow to zero.
        assert!(hypotf64(f64::MAX, f64::MAX).is_infinite());
        let min_sub = f64::from_bits(0x0000000000000001);
        assert!(hypotf64(min_sub, min_sub) > 0.0);
    }

    #[test]
    fn pythagorean_triples() {
        // Exact right triangles give exactly representable results.
        for (x, y, expected) in [(3.0, 4.0, 5.0), (5.0, 12.0, 13.0), (6.0, 8.0, 10.0)] {
            let got = hypotf64(x, y);
            assert!(
                ulps(got, expected) <= 4,
                "hypotf64({x}, {y}) = {got}, expected {expected} ({} ulps off)",
                ulps(got, expected)
            );
        }

        // hypot(1, 1) == sqrt(2)
        let got = hypotf64(1.0, 1.0);
        let want = core::f64::consts::SQRT_2;
        assert!(
            ulps(got, want) <= 4,
            "hypotf64(1.0, 1.0) = {got}, expected sqrt(2) ({} ulps off)",
            ulps(got, want)
        );
    }

    #[test]
    fn close_to_main_hypot() {
        // Sweep a grid of exponents (including the scaling boundaries around
        // 2^510 and 2^-450) and mantissas, with both signs, and check that the
        // relaxed implementation stays within a few ULPs of the main `hypot`.
        let exponents = [
            0u64, 1, 2, 100, 573, 574, 637, 1023, 1024, 1500, 1532, 1533, 2000, 2046,
        ];
        let mantissas = [
            0x0,
            0x1,
            0x8000_0000_0000,
            0x000f_ffff_ffff_ffff,
            0xaaaa_aaaa_aaaa_aaaa,
        ];
        let mut checked = 0u64;
        for &e1 in &exponents {
            for &e2 in &exponents {
                for &m in &mantissas {
                    for sign in [0u64, 0x8000_0000_0000_0000] {
                        let x = f64::from_bits(sign | (e1 << 52) | m);
                        let y = f64::from_bits(
                            sign | (e2 << 52) | (m.rotate_left(17) & 0x000f_ffff_ffff_ffff),
                        );
                        let got = hypotf64(x, y);
                        let want = super::super::super::hypot::hypot(x, y);
                        let d = ulps(got, want);
                        assert!(
                            d <= 8,
                            "hypotf64({x:e}, {y:e}) = {got:e} ({:#x}), main hypot = {want:e} ({:#x}), {d} ulps off",
                            got.to_bits(),
                            want.to_bits(),
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked >= 1_900, "sweep covered {checked} values");
    }
}
