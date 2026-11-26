// With AVX enabled, the VEX encodings avoid the transition penalty that legacy SSE instructions
// pay after code that leaves the upper halves of the vector registers dirty.

pub fn sqrtf(mut x: f32) -> f32 {
    // SAFETY: `sqrtss` is part of `sse2`, which this module is gated behind, and `vsqrtss` is part
    // of `avx`, which the `cfg` checks. Neither has memory access or side effects.
    unsafe {
        #[cfg(target_feature = "avx")]
        core::arch::asm!(
            "vsqrtss {x}, {x}, {x}",
            x = inout(xmm_reg) x,
            options(nostack, nomem, pure),
        );
        #[cfg(not(target_feature = "avx"))]
        core::arch::asm!(
            "sqrtss {x}, {x}",
            x = inout(xmm_reg) x,
            options(nostack, nomem, pure),
        );
    }
    x
}

pub fn sqrt(mut x: f64) -> f64 {
    // SAFETY: `sqrtsd` is part of `sse2`, which this module is gated behind, and `vsqrtsd` is part
    // of `avx`, which the `cfg` checks. Neither has memory access or side effects.
    unsafe {
        #[cfg(target_feature = "avx")]
        core::arch::asm!(
            "vsqrtsd {x}, {x}, {x}",
            x = inout(xmm_reg) x,
            options(nostack, nomem, pure),
        );
        #[cfg(not(target_feature = "avx"))]
        core::arch::asm!(
            "sqrtsd {x}, {x}",
            x = inout(xmm_reg) x,
            options(nostack, nomem, pure),
        );
    }
    x
}
