#![allow(unused_imports)]

use core::intrinsics;

// NOTE These functions are implemented using assembly because they using a custom
// calling convention which can't be implemented using a normal Rust function

// NOTE These functions are never mangled as they are not tested against compiler-rt

intrinsics! {
    #[unsafe(naked)]
    #[cfg(all(
        any(all(windows, target_env = "gnu"), target_os = "uefi"),
        not(feature = "no-asm")
    ))]
    pub unsafe extern "C" fn __chkstk() {
        core::arch::naked_asm!(
            "jmp __alloca", // Jump to __alloca since fallthrough may be unreliable"
        );
    }

    #[unsafe(naked)]
    #[cfg(all(
        any(all(windows, target_env = "gnu"), target_os = "uefi"),
        not(feature = "no-asm")
    ))]
    pub unsafe extern "C" fn _alloca() {
        // __chkstk and _alloca are the same function
        core::arch::naked_asm!(
            "push   ecx",
            "cmp    eax, 0x1000",
            "lea    ecx, [esp + 8]", // esp before calling this routine -> ecx
            "jb     3f",
            "2:",
            "sub    ecx, 0x1000",
            "test   [ecx], ecx",
            "sub    eax, 0x1000",
            "cmp    eax, 0x1000",
            "ja     2b",
            "3:",
            "sub    ecx, eax",
            "test   [ecx], ecx",
            "lea    eax, [esp + 4]",  // load pointer to the return address into eax
            "mov    esp, ecx",     // install the new top of stack pointer into esp
            "mov    ecx, [eax - 4]", // restore ecx
            "push   [eax]",        // push return address onto the stack
            "sub    eax, esp",     // restore the original value in eax
            "ret",
        );
    }
}
