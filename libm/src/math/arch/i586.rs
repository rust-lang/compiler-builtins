//! Architecture-specific support for x86-32 without SSE2

/// Use an alternative implementation on x86, because the
/// main implementation fails with the x87 FPU used by
/// debian i386, probably due to excess precision issues.
///
/// Based on https://github.com/NetBSD/src/blob/trunk/lib/libm/arch/i387/s_ceil.S
/// (written by J.T. Conklin <jtc@NetBSD.org>).
pub fn ceil(mut x: f64) -> f64 {
    // We save and later restore the FPU control word.
    let mut cw_stash = core::mem::MaybeUninit::<u16>::uninit();
    let mut cw_tmp = core::mem::MaybeUninit::<u16>::uninit();
    unsafe {
        core::arch::asm!(
            "fstcw   ({stash_ptr})",      // Save the cw
            "movw    ({stash_ptr}), %dx", // ...
            "orw     $0x0800, %dx",    // Set rounding control to 0b10 (+∞),
            "andw    $0xfbff, %dx",    // preserving other controls
            "movw    %dx, ({cw_ptr})", // Apply cw
            "fldcw    ({cw_ptr})",     // ...
            "fldl     ({x_ptr})",      // Push x to the stack
            "frndint",                 // Round
            "fldcw    ({stash_ptr})",  // Restore cw
            "fstpl   ({x_ptr})",       // Save rounded x to mem
            cw_ptr = in(reg) &mut cw_tmp,
            stash_ptr = in(reg) &mut cw_stash,
            x_ptr = in(reg) &mut x,
            out("dx") _,               // Cw scratch
            // All the x87 FPU stack is used, all registers must be clobbered
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(att_syntax)
        )
    }
    x
}

/// Use an alternative implementation on x86, because the
/// main implementation fails with the x87 FPU used by
/// debian i386, probably due to excess precision issues.
///
/// Based on https://github.com/NetBSD/src/blob/trunk/lib/libm/arch/i387/s_floor.S
/// (written by J.T. Conklin <jtc@NetBSD.org>).
pub fn floor(mut x: f64) -> f64 {
    // We save and later restore the FPU control word.
    let mut cw_stash = core::mem::MaybeUninit::<u16>::uninit();
    let mut cw_tmp = core::mem::MaybeUninit::<u16>::uninit();
    unsafe {
        core::arch::asm!(
            "fstcw   ({stash_ptr})",      // Save the cw
            "movw    ({stash_ptr}), %dx", // ...
            "orw     $0x0400, %dx",    // Set rounding control to 0b01 (-∞),
            "andw    $0xf7ff, %dx",    // preserving other controls
            "movw    %dx, ({cw_ptr})", // Apply cw
            "fldcw    ({cw_ptr})",     // ...
            "fldl     ({x_ptr})",      // Push x to the stack
            "frndint",                 // Round
            "fldcw    ({stash_ptr})",  // Restore cw
            "fstpl   ({x_ptr})",       // Save rounded x to mem
            cw_ptr = in(reg) &mut cw_tmp,
            stash_ptr = in(reg) &mut cw_stash,
            x_ptr = in(reg) &mut x,
            out("dx") _,               // Cw scratch
            // All the x87 FPU stack is used, all registers must be clobbered
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(att_syntax)
        )
    }
    x
}
