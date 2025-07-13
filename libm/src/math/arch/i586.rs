//! Architecture-specific support for x86-32 without SSE2

/// Use an alternative implementation on x86, because the
/// main implementation fails with the x87 FPU used by
/// debian i386, probably due to excess precision issues.
///
/// Based on https://github.com/NetBSD/src/blob/trunk/lib/libm/arch/i387/s_ceil.S
/// (written by J.T. Conklin <jtc@NetBSD.org>).
#[unsafe(naked)]
pub extern "C" fn ceil(_: f64) -> f64 {
    core::arch::naked_asm!(
        "pushl  %ebp",
        "movl   %esp,%ebp",
        "subl   $8,%esp",
        // Store fpu control word.
        "fstcw   -4(%ebp)",
        "movw    -4(%ebp),%dx",
        // Round towards +oo.
        "orw $0x0800,%dx",
        "andw    $0xfbff,%dx",
        "movw    %dx,-8(%ebp)",
        // Load modified control word
        "fldcw   -8(%ebp)",
        // Round.
        "fldl    8(%ebp)",
        "frndint",
        // Restore original control word.
        "fldcw   -4(%ebp)",
        // Restore esp and ebp and return
        "leave",
        "ret",
        options(att_syntax)
    )
}

/// Use an alternative implementation on x86, because the
/// main implementation fails with the x87 FPU used by
/// debian i386, probably due to excess precision issues.
///
/// Based on https://github.com/NetBSD/src/blob/trunk/lib/libm/arch/i387/s_floor.S
/// (written by J.T. Conklin <jtc@NetBSD.org>).
#[unsafe(naked)]
pub extern "C" fn floor(_: f64) -> f64 {
    core::arch::naked_asm!(
        "pushl  %ebp",
        "movl   %esp,%ebp",
        "subl   $8,%esp",
        // Store fpu control word.
        "fstcw   -4(%ebp)",
        "movw    -4(%ebp),%dx",
        // Round towards -oo.
        "orw	$0x0400,%dx",
        "andw	$0xf7ff,%dx",
        "movw   %dx,-8(%ebp)",
        // Load modified control word
        "fldcw   -8(%ebp)",
        // Round.
        "fldl    8(%ebp)",
        "frndint",
        // Restore original control word.
        "fldcw   -4(%ebp)",
        // Restore esp and ebp and return
        "leave",
        "ret",
        options(att_syntax)
    )
}
