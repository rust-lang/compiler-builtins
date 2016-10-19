use core::num::Wrapping;
use float::Float;
use int::Int;


macro_rules! floatunsisf {
    ($intrinsic:ident: $ity:ty => $fty:ty) => {
        /// Returns `a as f32` or `a as f64`
        #[cfg_attr(not(test), no_mangle)]
        pub extern "C" fn $intrinsic(a: $ity) -> $fty {
            // CC This file implements unsigned integer to single-precision conversion for the
            // CC compiler-rt library in the IEEE-754 default round-to-nearest, ties-to-even
            // CC mode.

            // CC    const int aWidth = sizeof a * CHAR_BIT;
            let a_width : usize = <$ity>::bits() as usize;
            // CC
            // CC    // Handle zero as a special case to protect clz
            // CC    if (a == 0) return fromRep(0);
            if a == 0 { return (<$fty as Float>::from_repr)(<$fty>::zero().0); }
            // CC
            // CC    // Exponent of (fp_t)a is the width of abs(a).
            // CC    const int exponent = (aWidth - 1) - __builtin_clz(a);
            let exponent : usize = (a_width - 1usize) - a.leading_zeros() as usize;
            // CC    rep_t result;
            // CC
            let mut result =
            // CC    // Shift a into the significand field, rounding if it is a right-shift
            // CC    if (exponent <= significandBits) {
            if exponent <= <$fty>::significand_bits() {
                // CC        const int shift = significandBits - exponent;
                let shift = <$fty>::significand_bits() - exponent;
                // CC        result = (rep_t)a << shift ^ implicitBit;
                (Wrapping(a as <$fty as Float>::Int) << shift) ^ <$fty>::implicit_bit()
            // CC    } else {
            } else {
                // CC        const int shift = exponent - significandBits;
                let shift = exponent - <$fty>::significand_bits();
                // CC        result = (rep_t)a >> shift ^ implicitBit;
                let mut result = (Wrapping(a as <$fty as Float>::Int) >> shift) ^ <$fty>::implicit_bit();
                // CC        rep_t round = (rep_t)a << (typeWidth - shift);
                let round = Wrapping(a as <$fty as Float>::Int) << (<$fty>::bits() - shift);
                // CC        if (round > signBit) result++;
                if round > <$fty>::sign_bit() { result += <$fty>::one() }
                // CC        if (round == signBit) result += result & 1;
                if round == <$fty>::sign_bit() { result += result & <$fty>::one() }
                result
            // CC    }
            };
            // CC
            // CC    // Insert the exponent
            // CC    result += (rep_t)(exponent + exponentBias) << significandBits;
            result += Wrapping((exponent + <$fty>::exponent_bias()) as <$fty as Float>::Int) << <$fty>::significand_bits();
            // CC    return fromRep(result);
            <$fty as Float>::from_repr(result.0)
        }
    }
}

// __floatunsisf implements unsigned integer to single-precision conversion
// (IEEE-754 default round-to-nearest, ties-to-even mode)
floatunsisf!(__floatunsisf: u32 => f32);

#[cfg(test)]
mod tests {
    use qc::{U32};
    use float::{Float};

    check! {
        fn __floatunsisf(f: extern fn(u32) -> f32, a: U32)
                    -> Option<f32> {
            Some(f(a.0))
        }
    }

    #[test]
    fn test_floatunsisf_conv_zero() {
        let r = super::__floatunsisf(0);
        assert!(r.eq_repr(0.0f32));
    }

    use std::mem;

    #[test]
    fn test_floatunsisf_libcompilerrt() {
        let compiler_rt_fn = ::compiler_rt::get("__floatunsisf");
        let compiler_rt_fn : extern fn(u32) -> f32 = unsafe { mem::transmute(compiler_rt_fn) };
        //println!("1231515 {:?}", compiler_rt_fn);
        let ans = compiler_rt_fn(0);
        println!("{:?}", ans);
        assert!(ans.eq_repr(0.0f32));

    }
}
