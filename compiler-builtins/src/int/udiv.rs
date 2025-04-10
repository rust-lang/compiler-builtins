#[cfg(not(feature = "public-test-deps"))]
pub(crate) use crate::int::specialized_div_rem::*;

#[cfg(feature = "public-test-deps")]
pub use crate::int::specialized_div_rem::*;

intrinsics! {
    #[maybe_use_optimized_c_shim]
    #[arm_aeabi_alias = __aeabi_uidiv]
    /// Returns `n / d`
    pub extern "C" fn __udivsi3(n: u32, d: u32) -> u32 {
        u32_div_rem(n, d).0
    }

    #[maybe_use_optimized_c_shim]
    /// Returns `n % d`
    pub extern "C" fn __umodsi3(n: u32, d: u32) -> u32 {
        u32_div_rem(n, d).1
    }
}

#[cfg(not(target_arch = "avr"))]
intrinsics! {
    #[maybe_use_optimized_c_shim]
    /// Returns `n / d` and sets `*rem = n % d`
    pub extern "C" fn __udivmodsi4(n: u32, d: u32, rem: Option<&mut u32>) -> u32 {
        let quo_rem = u32_div_rem(n, d);
        if let Some(rem) = rem {
            *rem = quo_rem.1;
        }
        quo_rem.0
    }
}

#[cfg(target_arch = "avr")]
intrinsics! {
    /// Returns `n / d` and `n % d` packed together.
    ///
    /// Ideally we'd use `-> (u32, u32)` or some kind of a packed struct, but
    /// both force a stack allocation, while our result has to be in R18:R26.
    pub extern "C" fn __udivmodsi4(n: u32, d: u32) -> u64 {
        let (div, rem) = u32_div_rem(n, d);

        ((rem as u64) << 32) | (div as u64)
    }

    #[naked]
    pub unsafe extern "C" fn __udivmodqi4() {
        // Returns unsigned 8-bit `n / d` and `n % d`.
        //
        // Note: GCC implements a [non-standard calling convention](https://gcc.gnu.org/wiki/avr-gcc#Exceptions_to_the_Calling_Convention) for this function.
        // Derived from: https://github.com/gcc-mirror/gcc/blob/95f10974a9190e345776604480a2df0191104308/libgcc/config/avr/lib1funcs.S#L1365
        
        // r25: remainder
        // r24: dividend, quotient
        // r22: divisor
        // r23: loop counter
        core::arch::naked_asm!(
            "clr r25",      // clear remainder
            "ldi r23, 8",   // init loop counter
            "lsl r24",      // shift dividend
            "1:",
            "rol r25",      // shift dividend into remainder
            "cp r25, r22",  // compare remainder & divisor
            "brcs 2f",      // REMAINder <= divisor
            "sub r25, r22", // restore remainder
            "2:",
            "rol r24",      // shift dividend (with CARRY)
            "dec r23",      // decrement loop counter
            "brne 1b",
            "com r24",      // complement result (C flag was complemented in loop)
            "ret",
        );
    }

    #[naked]
    pub unsafe extern "C" fn __divmodqi4() {
        // Returns signed 8-bit `n / d` and `n % d`.
        //
        // Note: GCC implements a [non-standard calling convention](https://gcc.gnu.org/wiki/avr-gcc#Exceptions_to_the_Calling_Convention) for this function.
        // Derived from: https://github.com/gcc-mirror/gcc/blob/95f10974a9190e345776604480a2df0191104308/libgcc/config/avr/lib1funcs.S#L1385

        // r25: remainder
        // r24: dividend, quotient
        // r22: divisor
        // r23: loop counter
        core::arch::naked_asm!(
            "bst r24, 7",           // store sign of dividend
            "mov r0, r24",
            "eor r0, r22",          // r0.7 is sign of result
            "sbrc r24, 7",
            "neg r24",              // dividend negative : negate
            "sbrc r22, 7",
            "neg r22",              // divisor negative : negate
            // TODO: "call" => instruction requires a CPU feature not currently enabled
            // TODO: gcc checks for __AVR_HAVE_JMP_CALL__
            "rcall __udivmodqi4",   // do the unsigned div/mod
            "brtc 1f",
            "neg r25",              // correct remainder sign
            "1:",
            "sbrc r0, 7",
            "neg r24",              // correct result sign
            "ret",
        );
    }

    #[naked]
    pub unsafe extern "C" fn __udivmodhi4() {
        // Returns unsigned 16-bit `n / d` and `n % d`.
        //
        // Note: GCC implements a [non-standard calling convention](https://gcc.gnu.org/wiki/avr-gcc#Exceptions_to_the_Calling_Convention) for this function.
        // Derived from: https://github.com/gcc-mirror/gcc/blob/95f10974a9190e345776604480a2df0191104308/libgcc/config/avr/lib1funcs.S#L1427

        // r26: remainder (low)
        // r27: remainder (high)
        // r24: dividend (low)
        // r25: dividend (high)
        // r22: divisor (low)
        // r23: divisor (high)
        // r21: loop counter
        core::arch::naked_asm!(
            "sub r26, r26",
            "sub r27, r27",     // clear remainder and carry
            "ldi r21, 17",      // init loop counter
            "rjmp 2f",          // jump to entry point
            "1:",
            "rol r26",          // shift dividend into remainder
            "rol r27",
            "cp r26, r22",      // compare remainder & divisor
            "cpc r27, r23",     
            "brcs 2f",          // remainder < divisor
            "sub r26, r22",     // restore remainder
            "sbc r27, r23",
            "2:",
            "rol r24",          // shift dividend (with CARRY)
            "rol r25",
            "dec r21",          // decrement loop counter
            "brne 1b",
            "com r24",
            "com r25",
            // TODO: "movw" => instruction requires a CPU feature not currently enabled
            // TODO: gcc checks for __AVR_HAVE_MOVW__
            "mov r22, r24",
            "mov r23, r25",
            "mov r24, r26",
            "mov r25, r27",
            "ret",
        );
    }

    #[naked]
    pub unsafe extern "C" fn __divmodhi4() {
        // Returns signed 16-bit `n / d` and `n % d`.
        //
        // Note: GCC implements a [non-standard calling convention](https://gcc.gnu.org/wiki/avr-gcc#Exceptions_to_the_Calling_Convention) for this function.
        // Derived from: https://github.com/gcc-mirror/gcc/blob/95f10974a9190e345776604480a2df0191104308/libgcc/config/avr/lib1funcs.S#L1457

        // r26: remainder (low)
        // r27: remainder (high)
        // r24: dividend (low)
        // r25: dividend (high)
        // r22: divisor (low)
        // r23: divisor (high)
        // r21: loop counter
        core::arch::naked_asm!(
            "bst r25, 7",           // store sign of dividend
            "mov r0, r23",
            "brtc 0f",
            "com r0",               // r0.7 is sign of result
            "rcall 1f",             // dividend negative : negate
            "0:",
            "sbrc r23, 7",
            "rcall 2f",             // divisor negative : negate
            // TODO: "call" => instruction requires a CPU feature not currently enabled
            // TODO: gcc checks for __AVR_HAVE_JMP_CALL__
            "rcall __udivmodhi4",   // do the unsigned div/mod
            "sbrc r0, 7",
            "rcall 2f",             // correct remainder sign
            "brtc 3f",
            "1:",
            "com r25",              // correct dividend/remainder sign
            "neg r24",
            "sbci r25, 0xFF",
            "ret",
            "2:",
            "com r23",              // correct divisor/result sign
            "neg r22",
            "sbci r23, 0xFF",
            "3:",
            "ret",
        );
    }
}

intrinsics! {
    #[maybe_use_optimized_c_shim]
    /// Returns `n / d`
    pub extern "C" fn __udivdi3(n: u64, d: u64) -> u64 {
        u64_div_rem(n, d).0
    }

    #[maybe_use_optimized_c_shim]
    /// Returns `n % d`
    pub extern "C" fn __umoddi3(n: u64, d: u64) -> u64 {
        u64_div_rem(n, d).1
    }

    #[maybe_use_optimized_c_shim]
    /// Returns `n / d` and sets `*rem = n % d`
    pub extern "C" fn __udivmoddi4(n: u64, d: u64, rem: Option<&mut u64>) -> u64 {
        let quo_rem = u64_div_rem(n, d);
        if let Some(rem) = rem {
            *rem = quo_rem.1;
        }
        quo_rem.0
    }

    // Note: we use block configuration and not `if cfg!(...)`, because we need to entirely disable
    // the existence of `u128_div_rem` to get 32-bit SPARC to compile, see `u128_divide_sparc` docs.

    /// Returns `n / d`
    pub extern "C" fn __udivti3(n: u128, d: u128) -> u128 {
        #[cfg(not(any(target_arch = "sparc", target_arch = "sparc64")))] {
            u128_div_rem(n, d).0
        }
        #[cfg(any(target_arch = "sparc", target_arch = "sparc64"))] {
            u128_divide_sparc(n, d, &mut 0)
        }
    }

    /// Returns `n % d`
    pub extern "C" fn __umodti3(n: u128, d: u128) -> u128 {
        #[cfg(not(any(target_arch = "sparc", target_arch = "sparc64")))] {
            u128_div_rem(n, d).1
        }
        #[cfg(any(target_arch = "sparc", target_arch = "sparc64"))] {
            let mut rem = 0;
            u128_divide_sparc(n, d, &mut rem);
            rem
        }
    }

    /// Returns `n / d` and sets `*rem = n % d`
    pub extern "C" fn __udivmodti4(n: u128, d: u128, rem: Option<&mut u128>) -> u128 {
        #[cfg(not(any(target_arch = "sparc", target_arch = "sparc64")))] {
            let quo_rem = u128_div_rem(n, d);
            if let Some(rem) = rem {
                *rem = quo_rem.1;
            }
            quo_rem.0
        }
        #[cfg(any(target_arch = "sparc", target_arch = "sparc64"))] {
            let mut tmp = 0;
            let quo = u128_divide_sparc(n, d, &mut tmp);
            if let Some(rem) = rem {
                *rem = tmp;
            }
            quo
        }
    }
}
