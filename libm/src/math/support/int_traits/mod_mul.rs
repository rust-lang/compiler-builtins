use super::{DInt, HInt, Int};

/// Barrett reduction using the constant `R == (1 << K) == (1 << U::BITS)`
///
/// For a more detailed description, see
/// <https://en.wikipedia.org/wiki/Barrett_reduction>.
///
/// After constructing as `Reducer::new(b, n)`,
/// has operations to efficiently compute
///  - `(a * b) / n` and `(a * b) % n`
///  - `Reducer::new((a * b * b) % n, n)`, as long as `a * (n - 1) < R`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Reducer<U> {
    // the multiplying factor `b in 0..n`
    num: U,
    // the modulus `n in 1..=R/2`
    div: U,
    // the precomputed quotient, `q = (b << K) / n`
    quo: U,
    // the remainder of that division, `r = (b << K) % n`,
    // (could always be recomputed as `(b << K) - q * n`,
    // but it is convenient to save)
    rem: U,
}

impl<U> Reducer<U>
where
    U: Int + HInt,
    U::D: core::ops::Div<Output = U::D>,
    U::D: core::ops::Rem<Output = U::D>,
{
    /// Requires `num < div <= R/2`, will panic otherwise
    #[inline]
    pub fn new(num: U, div: U) -> Self {
        let _0 = U::ZERO;
        let _1 = U::ONE;

        assert!(num < div);
        assert!(div.wrapping_sub(_1).leading_zeros() >= 1);

        let bk = num.widen_hi();
        let n = div.widen();
        let quo = (bk / n).lo();
        let rem = (bk % n).lo();

        Self { num, div, quo, rem }
    }
}

impl<U> Reducer<U>
where
    U: Int + HInt,
    U::D: Int,
{
    /// Return the unique pair `(quotient, remainder)`
    /// s.t. `a * b == quotient * n + remainder`, and `0 <= remainder < n`
    #[inline]
    pub fn mul_into_div_rem(&self, a: U) -> (U, U) {
        let (q, mut r) = self.mul_into_unnormalized_div_rem(a);
        // The unnormalized remainder is still guaranteed to be less than `2n`, so
        // one checked subtraction is sufficient.
        (q + U::cast_from(self.fixup(&mut r) as u8), r)
    }

    #[inline(always)]
    pub fn fixup(&self, x: &mut U) -> bool {
        x.checked_sub(self.div).map(|r| *x = r).is_some()
    }

    /// Return some pair `(quotient, remainder)`
    /// s.t. `a * b == quotient * n + remainder`, and `0 <= remainder < 2n`
    #[inline]
    pub fn mul_into_unnormalized_div_rem(&self, a: U) -> (U, U) {
        // General idea: Estimate the quotient `quotient = t in 0..a` s.t.
        // the remainder `ab - tn` is close to zero, so `t ~= ab / n`

        // Note: we use `R == 1 << U::BITS`, which means that
        //  - wrapping arithmetic with `U` is modulo `R`
        //  - all inputs are less than `R`

        // Range analysis:
        //
        // Using the definition of euclidean division on the two divisions done:
        // ```
        // bR = qn + r,  with 0 <= r < n
        // aq = tR + s,  with 0 <= s < R
        // ```
        let (_s, t) = a.widen_mul(self.quo).lo_hi();
        // Then
        // ```
        // (ab - tn)R
        // = abR - ntR
        // = a(qn + r) - n(aq - s)
        // = ar + ns
        // ```
        #[cfg(debug_assertions)]
        {
            assert!(t < a || (a == t && t.is_zero()));
            let ab_tn = a.widen_mul(self.num) - t.widen_mul(self.div);
            let ar_ns = a.widen_mul(self.rem) + _s.widen_mul(self.div);
            assert!(ab_tn.hi().is_zero());
            assert!(ar_ns.lo().is_zero());
            assert!(ab_tn.lo() == ar_ns.hi());
        }
        // Since `s < R` and `r < n`,
        // ```
        // 0 <= ns < nR
        // 0 <= ar < an
        // 0 <= (ab - tn) == (ar + ns)/R < n(1 + a/R)
        // ```
        // Since `a < R` and we check on construction that `n <= R/2`, the result
        // is `0 <= ab - tn < R`, so it can be computed modulo `R`
        // even though the intermediate terms generally wrap.
        let ab = a.wrapping_mul(self.num);
        let tn = t.wrapping_mul(self.div);
        (t, ab.wrapping_sub(tn))
    }

    /// Constructs a new reducer with `b` set to `(ab * b) % n`
    ///
    /// Requires `r * ab == ra * b`, where `r = bR % n`.
    #[inline(always)]
    fn with_scaled_num_rem(&self, ab: U, ra: U) -> Self {
        debug_assert!(ab.widen_mul(self.rem) == ra.widen_mul(self.num));
        // The new factor `v = abb mod n`:
        let (_, v) = self.mul_into_div_rem(ab);

        // `rab = cn + d`, where `0 <= d < n`
        let (c, d) = self.mul_into_div_rem(ra);

        // We need `abbR = Xn + Y`:
        // abbR
        // = ab(qn + r)
        // = abqn + rab
        // = abqn + cn + d
        // = (abq + c)n + d

        Self {
            num: v,
            div: self.div,
            quo: self.quo.wrapping_mul(ab).wrapping_add(c),
            rem: d,
        }
    }

    /// Computes the reducer with the factor `b` set to `(a * b * b) % n`
    /// Requires that `a * (n - 1)` does not overflow.
    #[allow(dead_code)]
    #[inline]
    pub fn squared_with_scale(&self, a: U) -> Self {
        debug_assert!(a.widen_mul(self.div - U::ONE).hi().is_zero());
        self.with_scaled_num_rem(a * self.num, a * self.rem)
    }

    /// Computes the reducer with the factor `b` set to `(b * b << s) % n`
    /// Requires that `(n - 1) << s` does not overflow.
    #[inline]
    pub fn squared_with_shift(&self, s: u32) -> Self {
        debug_assert!((self.div - U::ONE).leading_zeros() >= s);
        self.with_scaled_num_rem(self.num << s, self.rem << s)
    }
}

#[cfg(test)]
mod test {
    use super::Reducer;

    #[test]
    fn u8_all() {
        for y in 1..=128_u8 {
            for r in 0..y {
                let m = Reducer::new(r, y);
                assert_eq!(m.quo, ((r as f32 * 256.0) / (y as f32)) as u8);
                for x in 0..=u8::MAX {
                    let (quo, rem) = m.mul_into_div_rem(x);

                    let q0 = x as u32 * r as u32 / y as u32;
                    let r0 = x as u32 * r as u32 % y as u32;
                    assert_eq!(
                        (quo as u32, rem as u32),
                        (q0, r0),
                        "\n\
                        {x} * {r} = {xr}\n\
                        expected: = {q0} * {y} + {r0}\n\
                        returned: = {quo} * {y} + {rem} (== {})\n",
                        quo as u32 * y as u32 + rem as u32,
                        xr = x as u32 * r as u32,
                    );
                }
                for s in 0..=y.leading_zeros() {
                    assert_eq!(
                        m.squared_with_shift(s),
                        Reducer::new(((r << s) as u32 * r as u32 % y as u32) as u8, y)
                    );
                }
                for a in 0..=u8::MAX {
                    if a.checked_mul(y).is_some() {
                        let abb = a as u32 * r as u32 * r as u32;
                        assert_eq!(
                            m.squared_with_scale(a),
                            Reducer::new((abb % y as u32) as u8, y)
                        );
                    } else {
                        break;
                    }
                }
                for x0 in 0..=u8::MAX {
                    if m.num == 0 || x0 as u32 * m.rem as u32 % m.num as u32 != 0 {
                        continue;
                    }
                    let y0 = x0 as u32 * m.rem as u32 / m.num as u32;
                    let Ok(y0) = u8::try_from(y0) else { continue };

                    assert_eq!(
                        m.with_scaled_num_rem(x0, y0),
                        Reducer::new((x0 as u32 * m.num as u32 % y as u32) as u8, y)
                    );
                }
            }
        }
    }
}
