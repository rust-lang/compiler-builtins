//! Integers used for wide operations, larger than `u128`.

#[cfg(test)]
mod tests;

use core::{fmt, ops};

use super::{DInt, HInt, Int, MinInt};
use crate::support::Word;

const U128_LO_MASK: u128 = u64::MAX as u128;
const U128_WORDS: usize = (u128::BITS / Word::BITS) as usize;
const U256_WORDS: usize = U128_WORDS * 2;

/// A 256-bit unsigned integer represented as two 128-bit native-endian limbs.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct u256 {
    pub hi: u128,
    pub lo: u128,
}

impl u256 {
    pub const MAX: Self = Self {
        lo: u128::MAX,
        hi: u128::MAX,
    };
    pub const MIN: Self = Self { lo: 0, hi: 0 };

    /// Reinterpret as a signed integer
    pub fn signed(self) -> i256 {
        i256 {
            lo: self.lo,
            hi: self.hi as i128,
        }
    }

    /// Split into words, with the least significant word first.
    fn to_words(self) -> [Word; U256_WORDS] {
        // The result with 64-bit words will be: [lo.lo(), lo.hi(), hi.lo(), hi.hi()].
        let mut ret: [Word; _] = [0; U256_WORDS];
        for i in 0..U128_WORDS {
            let shift = i as u32 * Word::BITS;
            ret[i] = (self.lo >> shift) as Word;
            ret[i + U128_WORDS] = (self.hi >> shift) as Word;
        }
        ret
    }

    /// Perform the opposite of [`to_words`].
    fn from_words(words: [Word; U256_WORDS]) -> Self {
        let mut ret = u256::ZERO;
        for i in 0..U128_WORDS {
            let shift = i as u32 * usize::BITS;
            ret.lo |= (words[i] as u128) << shift;
            ret.hi |= (words[i + U128_WORDS] as u128) << shift;
        }
        ret
    }
}

/// A 256-bit signed integer represented as two 128-bit native-endian limbs.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct i256 {
    pub hi: i128,
    pub lo: u128,
}

impl i256 {
    pub const MAX: Self = Self {
        lo: u128::MAX,
        hi: i128::MAX,
    };
    pub const MIN: Self = Self {
        lo: u128::MIN,
        hi: i128::MIN,
    };

    /// Reinterpret as an unsigned integer
    pub fn unsigned(self) -> u256 {
        u256 {
            lo: self.lo,
            hi: self.hi as u128,
        }
    }

    /// Split into words, with the least significant word first.
    fn to_words(self) -> [Word; U256_WORDS] {
        self.unsigned().to_words()
    }

    /// Perform the opposite of [`to_words`].
    fn from_words(words: [Word; U256_WORDS]) -> Self {
        u256::from_words(words).signed()
    }
}

impl MinInt for u256 {
    type OtherSign = i256;

    type Unsigned = u256;

    const SIGNED: bool = false;
    const BITS: u32 = 256;
    const ZERO: Self = Self { lo: 0, hi: 0 };
    const ONE: Self = Self { lo: 1, hi: 0 };
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
}

impl MinInt for i256 {
    type OtherSign = u256;

    type Unsigned = u256;

    const SIGNED: bool = true;
    const BITS: u32 = 256;
    const ZERO: Self = Self { lo: 0, hi: 0 };
    const ONE: Self = Self { lo: 1, hi: 0 };
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
}

macro_rules! impl_common {
    ($ty:ty) => {
        impl ops::BitOr for $ty {
            type Output = Self;

            fn bitor(mut self, rhs: Self) -> Self::Output {
                self.lo |= rhs.lo;
                self.hi |= rhs.hi;
                self
            }
        }

        impl ops::Not for $ty {
            type Output = Self;

            fn not(mut self) -> Self::Output {
                self.lo = !self.lo;
                self.hi = !self.hi;
                self
            }
        }

        impl ops::Add<Self> for $ty {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                let (lo, carry) = self.lo.overflowing_add(rhs.lo);
                let (hi, of) = Int::carrying_add(self.hi, rhs.hi, carry);
                debug_assert!(!of, "attempt to add with overflow");
                Self { lo, hi }
            }
        }

        impl ops::Sub<Self> for $ty {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                let (lo, borrow) = self.lo.overflowing_sub(rhs.lo);
                let (hi, of) = Int::borrowing_sub(self.hi, rhs.hi, borrow);
                debug_assert!(!of, "attempt to subtract with overflow");
                Self { lo, hi }
            }
        }

        impl ops::Shr<u32> for $ty {
            type Output = Self;

            fn shr(self, rhs: u32) -> Self::Output {
                debug_assert!(rhs < Self::BITS, "attempt to shift right with overflow");

                // Set up an array with the input in the low half, zeros in the upper half
                let mut words = [Word::ZERO; U256_WORDS * 2];
                words[..U256_WORDS].copy_from_slice(&self.to_words());

                if <$ty>::SIGNED {
                    // For i256, branchlessly set the upper words to all ones if the input
                    // is negative.
                    let top_word = words[U256_WORDS - 1].signed() >> (Word::BITS - 1);
                    for x in &mut words[U256_WORDS..] {
                        *x = top_word.unsigned();
                    }
                }

                let shift = rhs & 255; // limit to 255 in cases of overflow
                let word_shift = (shift / Word::BITS) as usize;
                let bit_shift = shift % Word::BITS;

                let mut ret: [Word; U256_WORDS] = [0; _];

                // Each output word is a coarse (word-sized) shift plus a small bit shift. Note that
                // these loops get unrolled.
                for i in 0..U256_WORDS {
                    if i < (U256_WORDS - 1) {
                        let hi = words[word_shift + i + 1];
                        let lo = words[word_shift + i];

                        ret[i] = <Word as HInt>::funnel_shr(hi, lo, bit_shift);
                    } else if <$ty>::SIGNED {
                        // The upper word doesn't get any sign bits via a funnel shift, so we need
                        // an arithmetic shift to preserve sign.
                        let mut x = words[word_shift + i].signed();
                        x >>= bit_shift;
                        ret[i] = x.unsigned();
                    } else {
                        ret[i] = words[word_shift + i] >> bit_shift;
                    }
                }

                <$ty>::from_words(ret)
            }
        }
    };
}

impl ops::Shl<u32> for u256 {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        debug_assert!(rhs < Self::BITS, "attempt to shift left with overflow");

        // Set up an array with the input in the low half, zeros in the upper half
        let mut words = [Word::ZERO; U256_WORDS * 2];
        words[U256_WORDS..].copy_from_slice(&self.to_words());

        let shift = rhs & 255; // limit to 255 in cases of overflow
        let word_shift = U256_WORDS - (shift / Word::BITS) as usize;
        let bit_shift = shift % Word::BITS;

        let mut ret: [Word; U256_WORDS] = [0; _];

        // Each output word is a coarse (word-sized) shift plus a small bit shift. Note that
        // these loops get unrolled.
        for i in 0..U256_WORDS {
            if i == 0 {
                ret[i] = words[word_shift + i] << bit_shift;
            } else {
                let hi = words[word_shift + i];
                let lo = words[word_shift + i - 1];

                ret[i] = <Word as HInt>::funnel_shl(hi, lo, bit_shift);
            }
        }

        u256::from_words(ret)
    }
}

impl ops::Shl<u32> for i256 {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        (self.unsigned() << rhs).signed()
    }
}

impl_common!(i256);
impl_common!(u256);

impl HInt for u128 {
    type D = u256;

    fn widen(self) -> Self::D {
        u256 { lo: self, hi: 0 }
    }

    fn zero_widen(self) -> Self::D {
        self.widen()
    }

    fn zero_widen_mul(self, rhs: Self) -> Self::D {
        let l0 = self & U128_LO_MASK;
        let l1 = rhs & U128_LO_MASK;
        let h0 = self >> 64;
        let h1 = rhs >> 64;

        let p_ll: u128 = l0.overflowing_mul(l1).0;
        let p_lh: u128 = l0.overflowing_mul(h1).0;
        let p_hl: u128 = h0.overflowing_mul(l1).0;
        let p_hh: u128 = h0.overflowing_mul(h1).0;

        let s0 = p_hl + (p_ll >> 64);
        let s1 = (p_ll & U128_LO_MASK) + (s0 << 64);
        let s2 = p_lh + (s1 >> 64);

        let lo = (p_ll & U128_LO_MASK) + (s2 << 64);
        let hi = p_hh + (s0 >> 64) + (s2 >> 64);

        u256 { lo, hi }
    }

    fn widen_mul(self, rhs: Self) -> Self::D {
        self.zero_widen_mul(rhs)
    }

    fn widen_hi(self) -> Self::D {
        u256 { lo: 0, hi: self }
    }
}

impl HInt for i128 {
    type D = i256;

    fn widen(self) -> Self::D {
        i256 {
            lo: self as u128,
            hi: if self < 0 { -1 } else { 0 },
        }
    }

    fn zero_widen(self) -> Self::D {
        self.unsigned().zero_widen().signed()
    }

    fn zero_widen_mul(self, rhs: Self) -> Self::D {
        self.unsigned().zero_widen_mul(rhs.unsigned()).signed()
    }

    fn widen_mul(self, _rhs: Self) -> Self::D {
        unimplemented!("signed i128 widening multiply is not used")
    }

    fn widen_hi(self) -> Self::D {
        i256 { lo: 0, hi: self }
    }
}

impl DInt for u256 {
    type H = u128;

    fn lo(self) -> Self::H {
        self.lo
    }

    fn hi(self) -> Self::H {
        self.hi
    }
}

impl DInt for i256 {
    type H = i128;

    fn lo(self) -> Self::H {
        self.lo as i128
    }

    fn hi(self) -> Self::H {
        self.hi
    }
}

impl fmt::Debug for u256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(self, f)
    }
}

impl fmt::Debug for i256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.unsigned(), f)
    }
}

impl fmt::LowerHex for u256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        cfg_if! {
            if #[cfg(feature = "compiler-builtins")] {
                let _ = f;
                unimplemented!()
            } else {
                let pfx= if f.alternate() { "0x"} else {""};
                write!(f, "{pfx}{:032x}{:032x}", self.hi, self.lo)
            }
        }
    }
}

impl fmt::LowerHex for i256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.unsigned(), f)
    }
}
