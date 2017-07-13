use core::mem;
use core::ops;

use ::int::Int;

pub mod conv;
pub mod add;
pub mod pow;
pub mod sub;

/// Trait for some basic operations on floats
pub trait Float: 
    Copy +
    PartialEq +
    PartialOrd +
    ops::AddAssign +
    ops::Add<Output = Self> +
    ops::Sub<Output = Self> +
    ops::Div<Output = Self> +
    ops::Rem<Output = Self> +
{
    /// A uint of the same with as the float
    type Int: Int;

    fn zero() -> Self;
    fn one() -> Self;

    fn zero_int() -> Self::Int {
        Self::zero().repr()
    }
    fn one_int() -> Self::Int {
        Self::one().repr()
    }

    /// Returns the bitwidth of the float type
    fn bits() -> u32;

    /// Returns the bitwidth of the significand
    fn significand_bits() -> u32;

    /// Returns the bitwidth of the exponent
    fn exponent_bits() -> u32 {
        Self::bits() - Self::significand_bits() - 1
    }
    
    /// Returns the maximum value of the exponent
    fn exponent_max() -> u32 {
        (1 << Self::exponent_bits()) - 1
    }

    /// Returns the exponent bias value
    fn exponent_bias() -> u32 {
        Self::exponent_max() >> 1
    }

    /// Returns a mask for the sign bit
    fn sign_mask() -> Self::Int;

    /// Returns a mask for the significand
    fn significand_mask() -> Self::Int;

    // Returns the implicit bit of the float format
    fn implicit_bit() -> Self::Int;

    /// Returns a mask for the exponent
    fn exponent_mask() -> Self::Int {
        !(Self::sign_mask() | Self::significand_mask())
    }

    /// Returns `self` transmuted to `Self::Int`
    fn repr(self) -> Self::Int;

    #[cfg(test)]
    /// Checks if two floats have the same bit representation. *Except* for NaNs! NaN can be
    /// represented in multiple different ways. This method returns `true` if two NaNs are
    /// compared.
    fn eq_repr(self, rhs: Self) -> bool;

    /// Returns a `Self::Int` transmuted back to `Self`
    fn from_repr(a: Self::Int) -> Self;

    /// Constructs a `Self` from its parts. Inputs are treated as bits and shifted into position.
    fn from_parts(sign: bool, exponent: Self::Int, significand: Self::Int) -> Self;

    /// Returns (normalized exponent, normalized significand)
    fn normalize(significand: Self::Int) -> (i32, Self::Int);
}

macro_rules! float_impl {
    ($fty:ty, $uty:ty, $bits:expr, $sigbits:expr) => {
        impl Float for $fty {
            type Int = $uty;
            fn zero() -> Self {
                0.0
            }
            fn one() -> Self {
                1.0
            }
            fn bits() -> u32 {
                $bits
            }
            fn significand_bits() -> u32 {
                $sigbits
            }
            fn implicit_bit() -> Self::Int {
                1 << Self::significand_bits()
            }
            fn sign_mask() -> Self::Int {
                1 << (Self::bits() - 1)
            }
            fn significand_mask() -> Self::Int {
                (1 << Self::significand_bits()) - 1
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
            // FIXME: This can be implemented in the trait if RFC Issue #1424 is resolved
            //        https://github.com/rust-lang/rfcs/issues/1424
            fn from_repr(a: Self::Int) -> Self {
                unsafe { mem::transmute(a) }
            }
            fn from_parts(sign: bool, exponent: Self::Int, significand: Self::Int) -> Self {
                Self::from_repr(((sign as Self::Int) << (Self::bits() - 1)) |
                    ((exponent << Self::significand_bits()) & Self::exponent_mask()) |
                    (significand & Self::significand_mask()))
            }
            fn normalize(significand: Self::Int) -> (i32, Self::Int) {
                let shift = significand.leading_zeros()
                    .wrapping_sub((1 as $uty << Self::significand_bits()).leading_zeros());
                (1i32.wrapping_sub(shift as i32), significand << shift as Self::Int)
            }
        }
    }
}

float_impl!(f32, u32, 32, 23);
float_impl!(f64, u64, 64, 52);
