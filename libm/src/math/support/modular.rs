use crate::support::int_traits::NarrowingDiv;
use crate::support::{DInt, HInt, Int};

/// Contains:
///  n in (R/8, R/4)
///  x in [0, 2n)
#[derive(Debug, Clone, PartialEq, Eq)]
struct Reducer<U: HInt> {
    // let m = 2n
    m: U,
    // RR/2 = qm + r
    r: U,
    xq2: U::D,
}

impl<U> Reducer<U>
where
    U: HInt,
    U: Int<Unsigned = U>,
{
    /// Construct a reducer for `(x << _) mod n`.
    ///
    /// Requires `R/8 < n < R/4` and `x < 2n`.
    fn new(x: U, n: U) -> Self
    where
        U::D: NarrowingDiv,
    {
        let _1 = U::ONE;
        assert!(n > (_1 << (U::BITS - 3)));
        assert!(n < (_1 << (U::BITS - 2)));
        let m = n << 1;
        assert!(x < m);

        // We need q and r s.t. RR/2 = qm + r
        // As R/4 < m < R/2,
        // we have R <= q < 2R
        // so let q = R + f
        // RR/2 = (R + f)m + r
        // R(R/2 - m) = fm + r

        // v = R/2 - m < R/4 < m
        let v = (_1 << (U::BITS - 1)) - m;
        let (f, r) = v.widen_hi().checked_narrowing_div_rem(m).unwrap();

        // xq < qm <= RR/2
        // 2xq < RR
        // 2xq = 2xR + 2xf;
        let x2: U = x << 1;
        let xq2 = x2.widen_hi() + x2.widen_mul(f);
        Self { m, r, xq2 }
    }

    /// Extract the current remainder in the range `[0, 2n)`
    fn partial_remainder(&self) -> U {
        // RR/2 = qm + r, 0 <= r < m
        // 2xq = uR + v, 0 <= v < R
        // muR = 2mxq - mv
        // = xRR - 2xr - mv
        // mu + (2xr + mv)/R == xR

        // 0 <= 2xq < RR
        // R <= q < 2R
        // 0 <= x < R/2
        // R/4 < m < R/2
        // 0 <= r < m
        // 0 <= mv < mR
        // 0 <= 2xr < rR < mR

        // 0 <= (2xr + mv)/R < 2m
        // Add `mu` to each term to obtain:
        // mu <= xR < mu + 2m

        // Since `0 <= 2m < R`, `xR` is the only multiple of `R` between
        // `mu` and `m(u+2)`, so we can truncate the latter to find `x`.
        let _1 = U::ONE;
        self.m.widen_mul(self.xq2.hi() + (_1 + _1)).hi()
    }

    /// Maps the remainder `x` to `(x << k) - un`,
    /// for a suitable quotient `u`, which is returned.
    fn shift_reduce(&mut self, k: u32) -> U {
        assert!(k < U::BITS);
        // 2xq << k = aRR/2 + b;
        let a = self.xq2.hi() >> (U::BITS - 1 - k);
        let (lo, hi) = (self.xq2 << k).lo_hi();
        let b = U::D::from_lo_hi(lo, hi & (U::MAX >> 1));

        // (2xq << k) - aqm
        // = aRR/2 + b - aqm
        // = a(RR/2 - qm) + b
        // = ar + b
        self.xq2 = a.widen_mul(self.r) + b;
        a
    }

    /// Maps the remainder `x` to `x(R/2) - un`,
    /// for a suitable quotient `u`, which is returned.
    fn word_reduce(&mut self) -> U {
        // 2xq = uR + v
        let (v, u) = self.xq2.lo_hi();
        // xqR - uqm
        // = uRR/2 + vR/2 - uRR/2 + ur
        // = ur + (v/2)R
        self.xq2 = u.widen_mul(self.r) + U::widen_hi(v >> 1);
        u
    }
}

/// Compute the remainder `(x << e) % y` with unbounded integers.
/// Requires `x < 2y` and `y.leading_zeros() >= 2`
pub fn linear_mul_reduction<U>(x: U, mut e: u32, y: U) -> U
where
    U: HInt + Int<Unsigned = U>,
    U::D: NarrowingDiv,
{
    assert!(y <= U::MAX >> 2);
    assert!(x < (y << 1));
    let _0 = U::ZERO;
    let _1 = U::ONE;

    // power of two divisor
    if (y & (y - _1)).is_zero() {
        if e < U::BITS {
            return (x << e) & (y - _1);
        } else {
            return _0;
        }
    }

    // shift the divisor so it has exactly two leading zeros
    let y_shift = y.leading_zeros() - 2;
    let mut m = Reducer::new(x, y << y_shift);
    e += y_shift;

    while e >= U::BITS - 1 {
        m.word_reduce();
        e -= U::BITS - 1;
    }
    m.shift_reduce(e);

    let rem = m.partial_remainder() >> y_shift;
    rem.checked_sub(y).unwrap_or(rem)
}

#[cfg(test)]
mod test {
    use crate::support::linear_mul_reduction;
    use crate::support::modular::Reducer;

    #[test]
    fn reducer_ops() {
        for n in 33..=63_u8 {
            for x in 0..2 * n {
                let temp = Reducer::new(x, n);
                let n = n as u32;
                let x0 = temp.partial_remainder() as u32;
                assert_eq!(x as u32, x0);
                for k in 0..=7 {
                    let mut red = temp.clone();
                    let u = red.shift_reduce(k) as u32;
                    let x1 = red.partial_remainder() as u32;
                    assert_eq!(x1, (x0 << k) - u * n);
                    assert!(x1 < 2 * n);
                    assert!((red.xq2 as u32).is_multiple_of(2 * x1));

                    // `word_reduce` is equivalent to
                    // `shift_reduce(U::BITS - 1)`
                    if k == 7 {
                        let mut alt = temp.clone();
                        let w = alt.word_reduce();
                        assert_eq!(u, w as u32);
                        assert_eq!(alt, red);
                    }
                }
            }
        }
    }
    #[test]
    fn reduction() {
        for y in 1..64u8 {
            for x in 0..2 * y {
                let mut r = x % y;
                for e in 0..100 {
                    assert_eq!(r, linear_mul_reduction(x, e, y));
                    // maintain the correct expected remainder
                    r <<= 1;
                    if r >= y {
                        r -= y;
                    }
                }
            }
        }
    }
}
