/* SPDX-License-Identifier: MIT */
/* origin: core-math/src/binary64/log2p1/dint_log2p1.h
 * Copyright (c) 2022-2025 CERN.
 * Author: Tom Hubrecht
 * Ported to Rust in 2026, malezjaa
 * CORE-MATH commit: a971c788df744321ca8fb0d38e1b51a96e43b444
 */

//! This file contains type definition and functions to manipulate the dint64_t
//! data type used in the second iteration of Ziv's method. It is composed of two
//! uint64_t values for the significand and the exponent is represented by a
//! signed int64_t value.

use core::cmp::Ordering;

use super::super::support::{CastFrom, CastInto};

#[derive(Clone, Copy, Debug)]
pub struct Dint {
    pub hi: u64,
    pub lo: u64,
    pub ex: i64,
    pub sgn: u64,
}

impl Dint {
    pub const ONE: Dint = Dint {
        hi: 0x8000000000000000,
        lo: 0,
        ex: 0,
        sgn: 0,
    };

    pub const M_ONE: Dint = Dint {
        sgn: 1,
        ..Self::ONE
    };

    /// The following is an approximation of log(2), with absolute error less
    /// than 2^-129.97: |(hi/2^63+lo/2^127)*2^-1 - log(2)| < 2^-129.97
    pub const LOG2: Dint = Dint {
        hi: 0xb17217f7d1cf79ab,
        lo: 0xc9e3b39803f2f6af,
        ex: -1,
        sgn: 0,
    };

    /// The following is an approximation of 2^12/log(2), with absolute error less
    /// than 2^-118.63: |(hi/2^63+lo/2^127)*2^12 - 2^12/log(2)| < 2^-118.63
    pub const LOG2_INV: Dint = Dint {
        hi: 0xb8aa3b295c17f0bb,
        lo: 0xbe87fed0691d3e89,
        ex: 12,
        sgn: 0,
    };

    pub const ZERO: Dint = Dint {
        hi: 0,
        lo: 0,
        ex: 0,
        sgn: 0,
    };

    /// Add two dint64_t values
    #[inline]
    pub fn add(mut self, mut b: Dint) -> Dint {
        if self.hi | self.lo == 0 {
            return b;
        }
        if b.hi | b.lo == 0 {
            return self;
        }

        match (self.ex, self.hi, self.lo).cmp(&(b.ex, b.hi, b.lo)) {
            Ordering::Equal => {
                if self.sgn ^ b.sgn != 0 {
                    return Self::ZERO;
                }
                self.ex += 1;
                return self;
            }
            Ordering::Less => core::mem::swap(&mut self, &mut b),
            Ordering::Greater => {}
        }

        // From now on, |A| > |B|
        let a_sig = (u128::from(self.hi) << 64) | u128::from(self.lo);
        let mut b_sig = (u128::from(b.hi) << 64) | u128::from(b.lo);
        let mut m_ex = self.ex;

        if self.ex > b.ex {
            let sh = self.ex - b.ex;
            // round to nearest
            if sh <= 128 {
                b_sig = b_sig.wrapping_add(0x1 & (b_sig >> (sh - 1)));
            }

            if sh < 128 {
                b_sig >>= sh
            } else {
                b_sig = 0;
            };
        }

        let mut c;
        let sgn = self.sgn;
        if self.sgn ^ b.sgn != 0 {
            // a and b have different signs C = A + (-B)
            c = a_sig.wrapping_sub(b_sig);
        } else {
            // Add two 128 bit integers and return 1 if an overflow occured
            // Return the overflow
            let (sum, overflow) = a_sig.overflowing_add(b_sig);
            c = sum;
            if overflow {
                c = c.wrapping_add(c & 1);
                c = (1 << 127) | (c >> 1);
                m_ex += 1;
            }
        }

        let ex = c.leading_zeros();
        c = c << ex;
        Dint {
            sgn,
            hi: (c >> 64).cast(),
            lo: c.cast_lossy(),
            ex: m_ex - i64::from(ex),
        }
    }

    /// Multiply two dint64_t numbers, with 126 bits of accuracy
    #[inline]
    pub fn mul(self, b: Dint) -> Dint {
        let mut t = u128::from(self.hi) * u128::from(b.hi);
        let m1 = u128::from(self.hi) * u128::from(b.lo);
        let m2 = u128::from(self.lo) * u128::from(b.hi);

        // If we only guarantee 127 bits of accuracy, we improve the simplicity of the
        // code uint64_t l = ((u128)(a->lo) * (u128)(b->lo)) >> 64; m.l += l; m.h +=
        // (m.l < l);
        let (m, overflow) = m1.overflowing_add(m2);
        t = t.wrapping_add(u128::from(overflow) << 64);
        t = t.wrapping_add(m >> 64);

        // Ensure that r->hi starts with a 1
        let ex = u32::from(t >> 127 == 0);
        t = t.wrapping_shl(ex);
        t = t.wrapping_add((m >> 63) & 1);

        Dint {
            hi: (t >> 64).cast(),
            lo: t.cast_lossy(),
            // Exponent and sign
            ex: self.ex + b.ex - i64::from(ex) + 1,
            sgn: self.sgn ^ b.sgn,
        }
    }

    /// Multiply an integer with a dint64_t variable
    #[inline]
    pub fn mul_with_int(self, b: i64) -> Dint {
        if b == 0 {
            return Self::ZERO;
        }

        let c = b.unsigned_abs();
        let sgn = if b < 0 {
            u64::from(self.sgn == 0)
        } else {
            self.sgn
        };
        let mut t = u128::from(self.hi) * u128::from(c);
        let mut m = u64::cast_from(t >> 64).leading_zeros();
        t <<= m;

        // Will pose issues if b is too large but for now we assume it never happens
        // TODO: FIXME
        let l = ((u128::from(self.lo) * u128::from(c)) << (m - 1)) >> 63;
        let (sum, overflow) = l.overflowing_add(t);
        t = sum;
        if overflow {
            t = t.wrapping_add(t & 0x1);
            t = (1 << 127) | (t >> 1);
            m -= 1;
        }

        Dint {
            hi: (t >> 64).cast(),
            lo: t.cast_lossy(),
            ex: self.ex + 64 - i64::from(m),
            sgn,
        }
    }

    /// Convert a double to the corresponding dint64_t value
    #[inline]
    pub fn from_double(b: f64) -> Dint {
        // Extract both the significand and exponent of a double
        let bits = b.to_bits();
        let mut ex = i64::cast_from((bits >> 52) & 0x7ff);
        let mut hi = (bits & (u64::MAX >> 12)) + if ex != 0 { 1 << 52 } else { 0 };
        ex -= 0x3ff;

        let t = if hi != 0 { hi.leading_zeros() } else { 0 };
        hi <<= t;
        ex -= i64::from(if t > 11 { t - 12 } else { 0 });
        Dint {
            ex,
            hi,
            lo: 0,
            sgn: u64::from(b < 0.0),
        }
    }

    /// Convert a dint64_t value to a double
    /// assuming the input is not in the subnormal range
    #[inline]
    pub fn to_double(self) -> f64 {
        let bits = (self.hi >> 11) | (0x3ff << 52) | (self.sgn << 63);
        // r contains the upper 53 bits of a->hi, 1 <= r < 2

        let mut rd = 0.0;
        // if round bit is 1, add 2^-53
        if (self.hi >> 10) & 1 != 0 {
            rd += hf64!("0x1p-53");
        }
        // if trailing bits after the rounding bit are non zero, add 2^-54
        if self.hi & 0x3ff != 0 || self.lo != 0 {
            rd += hf64!("0x1p-54");
        }

        let r = f64::from_bits(bits) + if self.sgn == 0 { rd } else { -rd };
        // For log, the result is always in the normal range,
        // thus a->ex > -1023. Similarly, we cannot have a->ex > 1023.
        let e = f64::from_bits(u64::cast_from((self.ex + 1023) & 0x7ff) << 52);
        r * e
    }

    /// Put in r an approximation of `1/a`, assuming `a` is not zero.
    #[inline]
    fn from_reciprocal(a: f64) -> Dint {
        // we convert 4/a and divide by 4 to avoid a spurious underflow
        let mut r = Dint::from_double(4.0 / a); // accurate to about 53 bits
        r.ex -= 2;
        // we use Newton's iteration: r -> r + r*(1-a*r)
        let a = Dint::from_double(-a);
        let q = a.mul(r); // -a*r
        let q = Dint::ONE.add(q); // 1-a*r
        let q = r.mul(q); // r*(1-a*r)
        r.add(q)
    }

    /// Put in r an approximation of `b/a`, assuming `a` is not zero.
    #[inline]
    pub fn from_ratio(b: f64, a: f64) -> Dint {
        Self::from_reciprocal(a).mul(Dint::from_double(b))
    }
}
