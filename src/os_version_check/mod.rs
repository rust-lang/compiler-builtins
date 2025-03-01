//! os_version_check.c
//! <https://github.com/llvm/llvm-project/blob/llvmorg-20.1.0/compiler-rt/lib/builtins/os_version_check.c>
//!
//! Used by Objective-C's `@available` / Clang's `__builtin_available` macro / Swift's `#available`,
//! and is useful when linking together with code written in those languages.
#![allow(non_snake_case)]

#[cfg(target_vendor = "apple")]
mod darwin_impl;

intrinsics! {
    /// Old entry point for availability. Used when compiling with older Clang versions.
    #[inline]
    #[cfg(target_vendor = "apple")]
    pub extern "C" fn __isOSVersionAtLeast(major: u32, minor: u32, subminor: u32) -> i32 {
        let version = darwin_impl::pack_os_version(
            major as u16,
            minor as u8,
            subminor as u8,
        );
        (version <= darwin_impl::current_version()) as i32
    }

    /// Whether the current platform's OS version is higher than or equal to the given version.
    ///
    /// The first argument is the _base_ Mach-O platform (i.e. `PLATFORM_MACOS`, `PLATFORM_IOS`,
    /// etc., but not `PLATFORM_IOSSIMULATOR` or `PLATFORM_MACCATALYST`) of the invoking binary.
    //
    // Versions are specified statically by the compiler. Inlining with LTO should allow them to be
    // combined into a single `u32`, which should make comparisons faster, and make the
    // `BASE_TARGET_PLATFORM` check a no-op.
    #[inline]
    #[cfg(target_vendor = "apple")]
    // extern "C" is correct, LLVM assumes the function cannot unwind:
    // https://github.com/llvm/llvm-project/blob/llvmorg-20.1.0/clang/lib/CodeGen/CGObjC.cpp#L3980
    pub extern "C" fn __isPlatformVersionAtLeast(platform: i32, major: u32, minor: u32, subminor: u32) -> i32 {
        let version = darwin_impl::pack_os_version(
            major as u16,
            minor as u8,
            subminor as u8,
        );

        // Mac Catalyst is a technology that allows macOS to run in a different "mode" that closely
        // resembles iOS (and has iOS libraries like UIKit available).
        //
        // (Apple has added a "Designed for iPad" mode later on that allows running iOS apps
        // natively, but we don't need to think too much about those, since they link to
        // iOS-specific system binaries as well).
        //
        // To support Mac Catalyst, Apple has the concept of a "zippered" binary, which is a single
        // binary that can be run on both macOS and Mac Catalyst (has two `LC_BUILD_VERSION` Mach-O
        // commands, one set to `PLATFORM_MACOS` and one to `PLATFORM_MACCATALYST`).
        //
        // Most system libraries are zippered, which allows re-use across macOS and Mac Catalyst.
        // This includes the `libclang_rt.osx.a` shipped with Xcode! This means that `compiler-rt`
        // can't statically know whether it's compiled for macOS or Mac Catalyst, and thus this new
        // API (which replaces `__isOSVersionAtLeast`) is needed.
        //
        // In short:
        //      normal  binary calls  normal  compiler-rt --> `__isOSVersionAtLeast` was enough
        //      normal  binary calls zippered compiler-rt --> `__isPlatformVersionAtLeast` required
        //     zippered binary calls zippered compiler-rt --> `__isPlatformOrVariantPlatformVersionAtLeast` called

        // FIXME(madsmtm): `rustc` doesn't support zippered binaries yet, see rust-lang/rust#131216.
        // But once it does, we need the pre-compiled `std`/`compiler-builtins` shipped with rustup
        // to be zippered, and thus we also need to handle the `platform` difference here:
        //
        // if cfg!(target_os = "macos") && platform == 2 /* PLATFORM_IOS */ && cfg!(zippered) {
        //     return (version.to_u32() <= darwin_impl::current_ios_version()) as i32;
        // }
        //
        // `__isPlatformOrVariantPlatformVersionAtLeast` would also need to be implemented.

        // The base Mach-O platform for the current target.
        const BASE_TARGET_PLATFORM: i32 = if cfg!(target_os = "macos") {
            1 // PLATFORM_MACOS
        } else if cfg!(target_os = "ios") {
            2 // PLATFORM_IOS
        } else if cfg!(target_os = "tvos") {
            3 // PLATFORM_TVOS
        } else if cfg!(target_os = "watchos") {
            4 // PLATFORM_WATCHOS
        } else if cfg!(target_os = "visionos") {
            11 // PLATFORM_VISIONOS
        } else {
            0 // PLATFORM_UNKNOWN
        };
        debug_assert!(platform == BASE_TARGET_PLATFORM, "invalid platform provided to __isPlatformVersionAtLeast");

        (version <= darwin_impl::current_version()) as i32
    }
}
