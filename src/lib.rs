#![cfg_attr(feature = "compiler-builtins", compiler_builtins)]
#![cfg_attr(not(feature = "no-asm"), feature(asm))]
#![feature(abi_unadjusted)]
#![cfg_attr(not(feature = "no-asm"), feature(global_asm))]
#![feature(cfg_target_has_atomic)]
#![feature(compiler_builtins)]
#![feature(core_ffi_c)]
#![feature(intrinsics)]
#![feature(rustc_attrs)]
#![feature(inline_const)]
#![feature(lang_items)]
#![feature(linkage)]
#![feature(naked_functions)]
#![feature(repr_simd)]
#![no_builtins]
#![no_std]
#![allow(unused_features)]
#![allow(internal_features)]
// We use `u128` in a whole bunch of places which we currently agree with the
// compiler on ABIs and such, so we should be "good enough" for now and changes
// to the `u128` ABI will be reflected here.
#![allow(improper_ctypes, improper_ctypes_definitions)]
// `mem::swap` cannot be used because it may generate references to memcpy in unoptimized code.
#![allow(clippy::manual_swap)]
// Support compiling on both stage0 and stage1 which may differ in supported stable features.
#![allow(stable_features)]

// We disable #[no_mangle] for tests so that we can verify the test results
// against the native compiler-rt implementations of the builtins.

// NOTE cfg(all(feature = "c", ..)) indicate that compiler-rt provides an arch optimized
// implementation of that intrinsic and we'll prefer to use that

// NOTE(aapcs, aeabi, arm) ARM targets use intrinsics named __aeabi_* instead of the intrinsics
// that follow "x86 naming convention" (e.g. addsf3). Those aeabi intrinsics must adhere to the
// AAPCS calling convention (`extern "aapcs"`) because that's how LLVM will call them.

#[cfg(test)]
extern crate core;

#[macro_use]
mod macros;

pub mod float;
pub mod int;

#[cfg(any(
    all(target_family = "wasm", target_os = "unknown"),
    target_os = "uefi",
    target_os = "none",
    target_os = "xous",
    all(target_vendor = "fortanix", target_env = "sgx"),
    target_os = "windows"
))]
pub mod math;
pub mod mem;

#[cfg(target_arch = "arm")]
pub mod arm;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(all(target_arch = "aarch64", target_os = "linux", not(feature = "no-asm"),))]
pub mod aarch64_linux;

#[cfg(all(
    kernel_user_helpers,
    any(target_os = "linux", target_os = "android"),
    target_arch = "arm"
))]
pub mod arm_linux;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub mod riscv;

#[cfg(target_arch = "x86")]
pub mod x86;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub mod probestack;

// `core` is changing the feature name for the `intrinsics` module.
// To permit that transition, we avoid using that feature for now.
mod intrinsics {
    extern "rust-intrinsic" {
        #[rustc_nounwind]
        pub fn atomic_load_unordered<T: Copy>(src: *const T) -> T;

        #[rustc_nounwind]
        pub fn atomic_store_unordered<T: Copy>(dst: *mut T, val: T);

        /// Informs the optimizer that this point in the code is not reachable,
        /// enabling further optimizations.
        ///
        /// N.B., this is very different from the `unreachable!()` macro: Unlike the
        /// macro, which panics when it is executed, it is *undefined behavior* to
        /// reach code marked with this function.
        ///
        /// The stabilized version of this intrinsic is [`core::hint::unreachable_unchecked`].
        #[rustc_nounwind]
        pub fn unreachable() -> !;

        /// Performs an exact division, resulting in undefined behavior where
        /// `x % y != 0` or `y == 0` or `x == T::MIN && y == -1`
        ///
        /// This intrinsic does not have a stable counterpart.
        #[rustc_nounwind]
        pub fn exact_div<T: Copy>(x: T, y: T) -> T;

        /// Performs an unchecked division, resulting in undefined behavior
        /// where `y == 0` or `x == T::MIN && y == -1`
        ///
        /// Safe wrappers for this intrinsic are available on the integer
        /// primitives via the `checked_div` method. For example,
        /// [`u32::checked_div`]
        #[rustc_nounwind]
        pub fn unchecked_div<T: Copy>(x: T, y: T) -> T;
        /// Returns the remainder of an unchecked division, resulting in
        /// undefined behavior when `y == 0` or `x == T::MIN && y == -1`
        ///
        /// Safe wrappers for this intrinsic are available on the integer
        /// primitives via the `checked_rem` method. For example,
        /// [`u32::checked_rem`]
        #[rustc_nounwind]
        pub fn unchecked_rem<T: Copy>(x: T, y: T) -> T;
    }
}
