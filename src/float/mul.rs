use int::{Int, CastInto, LargeInt};
use float::Float;

trait PseudoLargeInt: Int {
    fn wide_mul(self, other: Self) -> (Self, Self);
    
    fn wide_shl(self, other: Self, count: u32) -> (Self, Self) {
        let hi = (other << count) | (self >> (Self::BITS - count));
        (self << count, hi)
    }

    fn wide_shr(self, other: Self, count: u32) -> (Self, Self) {
        let one = Self::ONE;
        let zero = Self::ZERO;

        if count < Self::BITS {
            let sticky = if self << (Self::BITS - count) == zero { zero } else { one };
            let lo = (other << (Self::BITS - count)) | (self >> count) | sticky;
            let hi = other >> count;

            (lo, hi) 
        } else if count < Self::BITS * 2 {
            let sticky = if other << (Self::BITS * 2 - count) | self == zero { zero } else { one };
            let lo = (other >> (count - Self::BITS)) | sticky;
            let hi = zero;

            (lo, hi) 
        } else {
            let sticky = if other | self == zero { zero } else { one };

            (sticky, zero)
        }
    }
}

impl PseudoLargeInt for u64 {
    fn wide_mul(self, other: Self) -> (Self, Self) {
        let a_lo = CastInto::<u64>::cast(self.low());
        let a_hi = CastInto::<u64>::cast(self.high());
        let b_lo = CastInto::<u64>::cast(other.low());
        let b_hi = CastInto::<u64>::cast(other.high());

        let plolo = a_lo * b_lo;
        let plohi = a_lo * b_hi;
        let philo = a_hi * b_lo;
        let phihi = a_hi * b_hi;

        let r0 = CastInto::<u64>::cast(plolo.low());
        let r1 = CastInto::<u64>::cast(plolo.high() + plohi.low() + philo.low());

        let r_lo = r0 + r1 << 32;
        let r_hi = 
            CastInto::<u64>::cast(plohi.high()) +
            CastInto::<u64>::cast(philo.high()) +
            CastInto::<u64>::cast(r1.high()) + phihi;

        (r_lo, r_hi)
    }
}

impl PseudoLargeInt for u32 {
    fn wide_mul(self, other: Self) -> (Self, Self) {
        let r = CastInto::<u64>::cast(self) * CastInto::<u64>::cast(other);
        (r.low(), r.high())
    }
}

/// Returns `a * b`
fn mul<F: Float>(a: F, b: F) -> F where
    u32: CastInto<F::Int>,
    F::Int: CastInto<u32>,
    i32: CastInto<F::Int>,
    F::Int: CastInto<i32>,
    F::Int: PseudoLargeInt,
{
    let one = F::Int::ONE;
    let zero = F::Int::ZERO;

    let bits =             F::BITS.cast();
    let significand_bits = F::SIGNIFICAND_BITS;
    let max_exponent =     F::EXPONENT_MAX;

    let implicit_bit =     F::IMPLICIT_BIT;
    let significand_mask = F::SIGNIFICAND_MASK;
    let sign_bit =         F::SIGN_MASK as F::Int;
    let abs_mask =         sign_bit - one;
    let exponent_mask =    F::EXPONENT_MASK;
    let inf_rep =          exponent_mask;
    let quiet_bit =        implicit_bit >> 1;
    let qnan_rep =         exponent_mask | quiet_bit;

    let a_rep = a.repr();
    let b_rep = b.repr();

    let a_exponent: i32 = ((a_rep & exponent_mask) >> significand_bits).cast();
    let b_exponent: i32 = ((b_rep & exponent_mask) >> significand_bits).cast();

    let product_sign = (a_rep ^ b_rep) & sign_bit;

    let mut a_significand = a_rep & significand_mask;
    let mut b_significand = b_rep & significand_mask;

    let sone = CastInto::<i32>::cast(one);
    let szero = CastInto::<i32>::cast(zero);
    let sme = CastInto::<i32>::cast(max_exponent);

    let scale =
    // Detect if a or b is zero, denormal, infinity, or NaN.
    if a_exponent.wrapping_sub(sone) >= sme.wrapping_sub(sone)
    || b_exponent.wrapping_sub(sone) >= sme.wrapping_sub(sone) {
        let a_abs = a_rep & abs_mask;
        let b_abs = b_rep & abs_mask;
        let mut scale = szero;
        match (a_abs, b_abs) {

            // NaN * anything = qNaN
            (n, _) if n > inf_rep =>
                return F::from_repr(a_rep | quiet_bit),
            // infinity * zero = NaN
            (n, m) if n == inf_rep && m == zero =>
                return F::from_repr(qnan_rep),
            // infinity * non-zero = +/- infinity
            (n, _) if n == inf_rep => return F::from_repr(a_abs | product_sign),

            // anything * NaN = qNaN
            (_, n) if n > inf_rep =>
                return F::from_repr(b_rep | quiet_bit),
            // zero * infinity = NaN
            (m, n) if n == inf_rep && m == zero =>
                return F::from_repr(qnan_rep),
            // non-zero * infinity = +/- infinity
            (_, n) if n == inf_rep =>
                return F::from_repr(b_abs | product_sign),

            // zero * anything = +/- zero
            (m, n) if n != inf_rep && m == zero =>
                return F::from_repr(product_sign),
            // anything * zero = +/- zero
            (n, m) if n != inf_rep && m == zero =>
                return F::from_repr(product_sign),

            // one or both of a or b is denormal, the other (if applicable) is a
            // normal number.  Renormalize one or both of a and b, and set scale to
            // include the necessary exponent adjustment.
            (a, b) => {
                if a < implicit_bit {
                    let (s, norm) = F::normalize(a_significand);
                    scale += s;
                    a_significand = norm;
                }
                if b < implicit_bit {
                    let (s, norm) = F::normalize(b_significand);
                    scale += s;
                    a_significand = norm;
                }
                scale
            }
        }
    } else {
        szero
    };

    // Or in the implicit significand bit.  (If we fell through from the
    // denormal path it was already set by normalize( ), but setting it twice
    // won't hurt anything.)
    a_significand |= implicit_bit;
    b_significand |= implicit_bit;

    // Get the significand of a*b.  Before multiplying the significands, shift
    // one of them left to left-align it in the field.  Thus, the product will
    // have (exponentBits + 2) integral digits, all but two of which must be
    // zero.  Normalizing this result is just a conditional left-shift by one
    // and bumping the exponent accordingly.
    let (product_lo, product_hi) = a_significand.wide_mul(b_significand << F::EXPONENT_BITS);

    // Normalize the significand, adjust exponent if needed.
    let (product_exponent, ib) = {
        let exponent_bias: i32 = CastInto::<i32>::cast(F::EXPONENT_BIAS);
        let ib = product_hi & implicit_bit != zero;
        (a_exponent + b_exponent - exponent_bias + scale + if ib { sone } else { szero }, ib)
    };

    let (product_lo, product_hi) = if ib {
        (product_lo, product_hi)
    } else {
        product_lo.wide_shl(product_hi, 1)
    };

    // If we have overflowed the type, return +/- infinity.
    if product_exponent >= CastInto::<i32>::cast(max_exponent) {
        return F::from_repr(inf_rep | product_sign);
    }

    let (product_lo, product_hi) = if product_exponent <= szero {
        // Result is denormal before rounding
        //
        // If the result is so small that it just underflows to zero, return
        // a zero of the appropriate sign.  Mathematically there is no need to
        // handle this case separately, but we make it a special case to
        // simplify the shift logic.
        let shift = 1i32.wrapping_sub(product_exponent);
        if shift >= bits.cast() {
            return F::from_repr(product_sign);
        }

        // Otherwise, shift the significand of the result so that the round
        // bit is the high bit of productLo.
        let shift = CastInto::<u32>::cast(shift);
        product_lo.wide_shr(product_hi, shift)
    } else {
        // Result is normal before rounding; insert the exponent.
        let product_hi = product_hi & significand_mask;
        let product_exponent = CastInto::<F::Int>::cast(product_exponent);
        let product_hi = product_hi | (product_exponent << significand_bits);
        (product_lo, product_hi)
    };

    // Insert the sign of the result:
    let signed = product_hi | product_sign;

    // Final rounding.  The final result may overflow to infinity, or underflow
    // to zero, but those are the correct results in those cases.  We use the
    // default IEEE-754 round-to-nearest, ties-to-even rounding mode.    let signed = signed + if product_lo > sign_bit { one } else { zero };
    let signed = signed + if product_lo == sign_bit { signed & one } else { zero };

    F::from_repr(signed)
}

intrinsics! {
    #[arm_aeabi_alias = __aeabi_fmul]
    pub extern "C" fn __mulsf3(a: f32, b: f32) -> f32 {
        mul(a, b)
    }

    #[arm_aeabi_alias = __aeabi_dmul]
    pub extern "C" fn __muldf3(a: f64, b: f64) -> f64 {
        mul(a, b)
    }
}
