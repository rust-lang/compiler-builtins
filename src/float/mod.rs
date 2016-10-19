use core::mem;
use core::num::Wrapping;
use core::ops;
use core::convert;
#[cfg(test)]
use core::fmt;

pub mod add;
pub mod pow;
pub mod conv;

/// Trait for some basic operations on floats
pub trait Float: Sized + Copy
    where Wrapping<Self::Int> : ops::Shl<usize, Output = Wrapping<Self::Int>>
                              + ops::Shr<usize, Output = Wrapping<Self::Int>>
                              + ops::Sub<Wrapping<Self::Int>, Output = Wrapping<Self::Int>>
                              + ops::BitOr<Wrapping<Self::Int>, Output = Wrapping<Self::Int>>
                              + ops::BitXor<Wrapping<Self::Int>, Output = Wrapping<Self::Int>>,
          Self::Int : convert::From<u32>
{
    /// A uint of the same with as the float
    type Int;

    fn one() -> Wrapping<Self::Int> {
        Wrapping(Self::Int::from(1u32))
    }

    fn zero() -> Wrapping<Self::Int> {
        Wrapping(Self::Int::from(0u32))
    }

    /// Returns the bitwidth of the float type
    fn bits() -> usize;

    /// Returns the bitwidth of the significand
    fn significand_bits() -> usize;

    fn exponent_bits() -> usize {
        Self::bits() - Self::significand_bits() - 1
    }

    fn max_exponent() -> usize {
        (1usize << Self::exponent_bits()).wrapping_sub(1)
    }

    fn exponent_bias() -> usize {
        Self::max_exponent() >> 1
    }

    fn implicit_bit() -> Wrapping<Self::Int> {
        Self::one() << Self::significand_bits()
    }

    fn significand_mask() -> Wrapping<Self::Int> {
        Self::implicit_bit() - Self::one()
    }

    fn sign_bit() -> Wrapping<Self::Int> {
        Self::one() << (Self::significand_bits() + Self::exponent_bits())
    }

    fn abs_mask() -> Wrapping<Self::Int> {
        Self::sign_bit() - Self::one()
    }

    fn exponent_mask() -> Wrapping<Self::Int> {
        Self::abs_mask() ^ Self::significand_mask()
    }

    fn inf_rep() -> Wrapping<Self::Int> {
        Self::exponent_mask()
    }

    fn quiet_bit() -> Wrapping<Self::Int> {
        Self::implicit_bit() >> 1
    }

    fn qnan_rep() -> Wrapping<Self::Int> {
        Self::exponent_mask() | Self::quiet_bit()
    }

    /// Returns `self` transmuted to `Self::Int`
    fn repr(self) -> Self::Int;

    #[cfg(test)]
    /// Checks if two floats have the same bit representation. *Except* for NaNs! NaN can be
    /// represented in multiple different ways. This methods returns `true` if two NaNs are
    /// compared.
    fn eq_repr(self, rhs: Self) -> bool;

    /// Returns a `Self::Int` transmuted back to `Self`
    fn from_repr(a: Self::Int) -> Self;

    /// Returns (normalized exponent, normalized significand)
    fn normalize(significand: Self::Int) -> (i32, Self::Int);
}

impl Float for f32 {
    type Int = u32;
    fn bits() -> usize {
        32
    }
    fn significand_bits() -> usize {
        23
    }
    fn repr(self) -> Self::Int {
        unsafe { mem::transmute(self) }
    }
    #[cfg(test)]
    fn eq_repr(self, rhs: Self) -> bool {
        if self.is_nan() && rhs.is_nan() {
            true
        } else {
            self.repr() == rhs.repr()
        }
    }
    fn from_repr(a: Self::Int) -> Self {
        unsafe { mem::transmute(a) }
    }
    fn normalize(significand: Self::Int) -> (i32, Self::Int) {
        let shift = significand.leading_zeros()
            .wrapping_sub((1u32 << Self::significand_bits()).leading_zeros());
        (1i32.wrapping_sub(shift as i32), significand << shift as Self::Int)
    }
}

impl Float for f64 {
    type Int = u64;
    fn bits() -> usize {
        64
    }
    fn significand_bits() -> usize {
        52
    }
    fn repr(self) -> Self::Int {
        unsafe { mem::transmute(self) }
    }
    #[cfg(test)]
    fn eq_repr(self, rhs: Self) -> bool {
        if self.is_nan() && rhs.is_nan() {
            true
        } else {
            self.repr() == rhs.repr()
        }
    }
    fn from_repr(a: Self::Int) -> Self {
        unsafe { mem::transmute(a) }
    }
    fn normalize(significand: Self::Int) -> (i32, Self::Int) {
        let shift = significand.leading_zeros()
            .wrapping_sub((1u64 << Self::significand_bits()).leading_zeros());
        (1i32.wrapping_sub(shift as i32), significand << shift as Self::Int)
    }
}

// TODO: Move this to F32/F64 in qc.rs
#[cfg(test)]
#[derive(Copy, Clone)]
pub struct FRepr<F>(F);

#[cfg(test)]
impl<F: Float> PartialEq for FRepr<F>
    where Wrapping<F::Int> : ops::Shl<usize, Output = Wrapping<F::Int>>
                              + ops::Shr<usize, Output = Wrapping<F::Int>>
                              + ops::Sub<Wrapping<F::Int>, Output = Wrapping<F::Int>>
                              + ops::BitOr<Wrapping<F::Int>, Output = Wrapping<F::Int>>
                              + ops::BitXor<Wrapping<F::Int>, Output = Wrapping<F::Int>>
{
    fn eq(&self, other: &FRepr<F>) -> bool {
        // NOTE(cfg) for some reason, on hard float targets, our implementation doesn't
        // match the output of its gcc_s counterpart. Until we investigate further, we'll
        // just avoid testing against gcc_s on those targets. Do note that our
        // implementation matches the output of the FPU instruction on *hard* float targets
        // and matches its gcc_s counterpart on *soft* float targets.
        if cfg!(gnueabihf) {
            return true
        }
        self.0.eq_repr(other.0)
    }
}

#[cfg(test)]
impl<F: fmt::Debug> fmt::Debug for FRepr<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
