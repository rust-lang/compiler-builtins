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
