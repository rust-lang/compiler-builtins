#![allow(unused_imports)]

use core::intrinsics;

// NOTE These functions are implemented using assembly because they using a custom
// calling convention which can't be implemented using a normal Rust function

// NOTE These functions are never mangled as they are not tested against compiler-rt

intrinsics! {
    #[naked]
    #[cfg(all(
        any(
            all(windows, target_env = "gnu"),
            target_os = "cygwin",
            target_os = "uefi"
        ),
        not(feature = "no-asm")
    ))]
    pub unsafe extern "C" fn ___chkstk_ms() {
        core::arch::naked_asm!(
            "push   %rcx",
            "push   %rax",
            "cmp    $0x1000,%rax",
            "lea    24(%rsp),%rcx",
            "jb     1f",
            "2:",
            "sub    $0x1000,%rcx",
            "test   %rcx,(%rcx)",
            "sub    $0x1000,%rax",
            "cmp    $0x1000,%rax",
            "ja     2b",
            "1:",
            "sub    %rax,%rcx",
            "test   %rcx,(%rcx)",
            "pop    %rax",
            "pop    %rcx",
            "ret",
            options(att_syntax)
        );
    }
}

// HACK(https://github.com/rust-lang/rust/issues/62785): x86_64-unknown-uefi needs special LLVM
// support unless we emit the _fltused
mod _fltused {
    #[no_mangle]
    #[used]
    #[cfg(target_os = "uefi")]
    static _fltused: i32 = 0;
}
