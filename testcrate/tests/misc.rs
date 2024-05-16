// makes configuration easier
#![allow(unused_macros)]
#![feature(f128)]

use testcrate::*;

/// Make sure that the the edge case tester and randomized tester don't break, and list examples of
/// fuzz values for documentation purposes.
#[test]
fn fuzz_values() {
    const VALS: [u16; 47] = [
        0b0, // edge cases
        0b1111111111111111,
        0b1111111111111110,
        0b1111111111111100,
        0b1111111110000000,
        0b1111111100000000,
        0b1110000000000000,
        0b1100000000000000,
        0b1000000000000000,
        0b111111111111111,
        0b111111111111110,
        0b111111111111100,
        0b111111110000000,
        0b111111100000000,
        0b110000000000000,
        0b100000000000000,
        0b11111111111111,
        0b11111111111110,
        0b11111111111100,
        0b11111110000000,
        0b11111100000000,
        0b10000000000000,
        0b111111111,
        0b111111110,
        0b111111100,
        0b110000000,
        0b100000000,
        0b11111111,
        0b11111110,
        0b11111100,
        0b10000000,
        0b111,
        0b110,
        0b100,
        0b11,
        0b10,
        0b1,
        0b1010110100000, // beginning of random fuzzing
        0b1100011001011010,
        0b1001100101001111,
        0b1101010100011010,
        0b100010001,
        0b1000000000000000,
        0b1100000000000101,
        0b1100111101010101,
        0b1100010111111111,
        0b1111110101111111,
    ];
    let mut i = 0;
    fuzz(10, |x: u16| {
        assert_eq!(x, VALS[i]);
        i += 1;
    });
}

#[test]
fn leading_zeros() {
    use compiler_builtins::int::__clzsi2;
    use compiler_builtins::int::leading_zeros::{
        usize_leading_zeros_default, usize_leading_zeros_riscv,
    };
    fuzz(N, |x: usize| {
        let lz = x.leading_zeros() as usize;
        let lz0 = __clzsi2(x);
        let lz1 = usize_leading_zeros_default(x);
        let lz2 = usize_leading_zeros_riscv(x);
        if lz0 != lz {
            panic!("__clzsi2({}): std: {}, builtins: {}", x, lz, lz0);
        }
        if lz1 != lz {
            panic!(
                "usize_leading_zeros_default({}): std: {}, builtins: {}",
                x, lz, lz1
            );
        }
        if lz2 != lz {
            panic!(
                "usize_leading_zeros_riscv({}): std: {}, builtins: {}",
                x, lz, lz2
            );
        }
    })
}

// This is approximate because of issues related to
// https://github.com/rust-lang/rust/issues/73920.
// TODO how do we resolve this indeterminacy?
macro_rules! pow {
    ($($f:ty, $tolerance:expr, $fn:ident);*;) => {
        $(
            #[test]
            fn $fn() {
                use compiler_builtins::float::pow::$fn;
                use compiler_builtins::float::Float;
                fuzz_float_2(N, |x: $f, y: $f| {
                    if !(Float::is_subnormal(x) || Float::is_subnormal(y) || x.is_nan()) {
                        let n = y.to_bits() & !<$f as Float>::SIGNIFICAND_MASK;
                        let n = (n as <$f as Float>::SignedInt) >> <$f as Float>::SIGNIFICAND_BITS;
                        let n = n as i32;
                        let tmp0: $f = x.powi(n);
                        let tmp1: $f = $fn(x, n);
                        let (a, b) = if tmp0 < tmp1 {
                            (tmp0, tmp1)
                        } else {
                            (tmp1, tmp0)
                        };
                        let good = {
                            if a == b {
                                // handles infinity equality
                                true
                            } else if a < $tolerance {
                                b < $tolerance
                            } else {
                                let quo = b / a;
                                // FIXME(f16_f128): we do this to block const eval which currently
                                // ICEs on f128. Change this once it works correctly.
                                (quo < (1. + black_box($tolerance)))
                                    && (quo > (1. - black_box($tolerance)))
                            }
                        };
                        if !good {
                            panic!(
                                "{}({:?}, {:?}): std: {:?}, builtins: {:?}",
                                stringify!($fn), x, n, tmp0, tmp1
                            );
                        }
                    }
                });
            }
        )*
    };
}

#[cfg(not(all(target_arch = "x86", not(target_feature = "sse"))))]
mod float_pow {
    use super::*;

    fn black_box<T>(val: T) -> T {
        val
    }

    pow! {
        f32, 1e-4, __powisf2;
        f64, 1e-12, __powidf2;
    }
}

#[cfg(not(all(target_arch = "x86", not(target_feature = "sse"))))]
#[cfg(not(any(feature = "no-f16-f128", feature = "no-sys-f128")))]
mod float_pow_f128 {
    use super::*;
    use core::hint::black_box;

    // Windows can't link the required arithmetic functions. See
    // <https://github.com/rust-lang/compiler-builtins/pull/614#issuecomment-2118636613>
    #[cfg(not(target_family = "windows"))]
    #[cfg(not(any(target_arch = "powerpc", target_arch = "powerpc64")))]
    pow! {
        f128, 1e-36, __powitf2;
    }

    #[cfg(any(target_arch = "powerpc", target_arch = "powerpc64"))]
    pow! {
        f128, 1e-36, __powikf2;
    }
}
