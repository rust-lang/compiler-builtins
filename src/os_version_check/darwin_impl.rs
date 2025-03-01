use core::ffi::CStr;
use core::{
    ffi::{c_char, c_int, c_long, c_uint, c_void},
    num::NonZero,
    ptr::null_mut,
    slice,
    sync::atomic::{AtomicU32, Ordering},
};

/// Get the current OS version.
#[inline]
pub(super) fn current_version() -> u32 {
    // Cache the lookup for performance.
    //
    // 0.0.0 is never gonna be a valid version, so we use that as our sentinel value.
    static CURRENT_VERSION: AtomicU32 = AtomicU32::new(0);

    // We use relaxed atomics, it doesn't matter if multiple threads end up racing to read or write
    // the version, `lookup_version` should be idempotent and always return the same value.
    //
    // `compiler-rt` uses `dispatch_once`, but that's overkill for the reasons above.
    let version = CURRENT_VERSION.load(Ordering::Relaxed);
    if version == 0 {
        let version = lookup_version().get();
        CURRENT_VERSION.store(version, Ordering::Relaxed);
        version
    } else {
        version
    }
}

#[cold]
// Use `extern "C"` to abort on panic, allowing `current_version` to be free of panic handling.
pub(super) extern "C" fn lookup_version() -> NonZero<OSVersion> {
    // Since macOS 10.15, libSystem has provided the undocumented `_availability_version_check` via
    // `libxpc` (zippered, so requires platform parameter to differentiate between on macOS and Mac
    // Catalyst) for doing the version lookup, though it's usage may be a bit dangerous, see:
    // - https://reviews.llvm.org/D150397
    // - https://github.com/llvm/llvm-project/issues/64227
    //
    // So instead, we use the safer approach of reading from `sysctl` (which is faster), and if that
    // fails, we fall back to the property list (this is what `_availability_version_check` does
    // internally).
    let version = version_from_sysctl().unwrap_or_else(version_from_plist);

    // Use `NonZero` to try to make it clearer to the optimizer that this will never return 0.
    NonZero::new(version).expect("version cannot be 0.0.0")
}

/// Look up the current OS version(s) from `/System/Library/CoreServices/SystemVersion.plist`.
///
/// More specifically, from the `ProductVersion` and `iOSSupportVersion` keys, and from
/// `$IPHONE_SIMULATOR_ROOT/System/Library/CoreServices/SystemVersion.plist` on the simulator.
///
/// This file was introduced in macOS 10.3, which is well below the minimum supported version by
/// `rustc`, which is currently macOS 10.12.
///
/// # Panics
///
/// Panics if reading or parsing the version fails (or if the system was out of memory).
///
/// We deliberately choose to panic, as having this silently return an invalid OS version would be
/// impossible for a user to debug.
#[allow(non_upper_case_globals, non_snake_case)]
pub(super) fn version_from_plist() -> OSVersion {
    #[allow(clippy::upper_case_acronyms)]
    enum FILE {}

    const SEEK_END: c_int = 2;

    const RTLD_LAZY: c_int = 0x1;
    const RTLD_LOCAL: c_int = 0x4;

    // SAFETY: Same signatures as in `libc`.
    //
    // NOTE: We do not need to link these; that will be done by `std` by linking `libSystem`
    // (which is required on macOS/Darwin).
    unsafe extern "C" {
        unsafe fn getenv(s: *const c_char) -> *mut c_char;
        safe fn malloc(size: usize) -> *mut c_void;
        unsafe fn free(p: *mut c_void);
        unsafe fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
        unsafe fn strcat(s: *mut c_char, ct: *const c_char) -> *mut c_char;

        unsafe fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE;
        unsafe fn fseek(stream: *mut FILE, offset: c_long, whence: c_int) -> c_int;
        unsafe fn ftell(stream: *mut FILE) -> c_long;
        unsafe fn rewind(stream: *mut FILE);
        unsafe fn fread(ptr: *mut c_void, size: usize, nobj: usize, stream: *mut FILE) -> usize;
        unsafe fn fclose(file: *mut FILE) -> c_int;

        unsafe fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
        unsafe fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        // NOTE: Cannot use this because we cannot Debug print `CStr` in `compiler-builtins`.
        // safe fn dlerror() -> *mut c_char;
        unsafe fn dlclose(handle: *mut c_void) -> c_int;
    }

    // We do not need to do a similar thing as what Zig does to handle the fake 10.16 versions
    // returned when the SDK version of the binary is less than 11.0:
    // <https://github.com/ziglang/zig/blob/0.13.0/lib/std/zig/system/darwin/macos.zig>
    //
    // <https://github.com/apple-oss-distributions/xnu/blob/xnu-11215.81.4/libsyscall/wrappers/system-version-compat.c>
    //
    // The reasoning is that we _want_ to follow Apple's behaviour here, and return 10.16 when
    // compiled with an older SDK; the user should upgrade their tooling.
    //
    // NOTE: `rustc` currently doesn't set the right SDK version when linking with ld64, so this
    // will have the wrong behaviour with `-Clinker=ld` on x86_64. But that's a `rustc` bug:
    // <https://github.com/rust-lang/rust/issues/129432>

    struct Deferred<F: FnMut()>(F);
    impl<F: FnMut()> Drop for Deferred<F> {
        fn drop(&mut self) {
            (self.0)();
        }
    }

    let path = c"/System/Library/CoreServices/SystemVersion.plist";
    let _path_free;
    let path = if cfg!(target_abi = "sim") {
        let root = unsafe { getenv(c"IPHONE_SIMULATOR_ROOT".as_ptr()) };
        if root.is_null() {
            panic!(
                "environment variable `IPHONE_SIMULATOR_ROOT` must be set when executing under simulator"
            );
        }
        let root = unsafe { CStr::from_ptr(root) };

        let ptr = malloc(root.count_bytes() + path.count_bytes() + 1);
        assert!(!ptr.is_null(), "failed allocating path");
        _path_free = Deferred(move || unsafe { free(ptr) });

        let ptr = ptr.cast::<c_char>();
        unsafe { strcpy(ptr, root.as_ptr()) };
        unsafe { strcat(ptr, path.as_ptr()) };
        unsafe { CStr::from_ptr(ptr) }
    } else {
        path
    };

    let plist_file = unsafe { fopen(path.as_ptr(), c"r".as_ptr()) };
    assert!(!plist_file.is_null(), "failed opening SystemVersion.plist");
    let _plist_file_close = Deferred(|| {
        if unsafe { fclose(plist_file) } != 0 {
            panic!("failed closing SystemVersion.plist");
        }
    });

    let ret = unsafe { fseek(plist_file, 0, SEEK_END) };
    assert!(ret == 0, "failed seeking SystemVersion.plist");
    let file_size = unsafe { ftell(plist_file) };
    assert!(
        0 <= file_size,
        "failed reading file length of SystemVersion.plist"
    );
    unsafe { rewind(plist_file) };

    let plist_buffer = malloc(file_size as usize);
    assert!(
        !plist_buffer.is_null(),
        "failed allocating buffer to hold PList"
    );
    let _plist_buffer_free = Deferred(|| unsafe { free(plist_buffer) });

    let num_read = unsafe { fread(plist_buffer, 1, file_size as usize, plist_file) };
    assert!(
        num_read == file_size as usize,
        "failed reading all bytes from SystemVersion.plist"
    );

    let plist_buffer = unsafe { slice::from_raw_parts(plist_buffer.cast::<u8>(), num_read) };

    // We do roughly the same thing here as `compiler-rt`, and dynamically look up CoreFoundation
    // utilities for reading PLists (to avoid having to re-implement that in here).

    // Link to the CoreFoundation dylib. Explicitly use non-versioned path here, to allow this to
    // work on older iOS devices.
    let cf = c"/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation";
    let _cf_free;
    let cf = if cfg!(target_abi = "sim") {
        let root = unsafe { getenv(c"IPHONE_SIMULATOR_ROOT".as_ptr()) };
        if root.is_null() {
            panic!(
                "environment variable `IPHONE_SIMULATOR_ROOT` must be set when executing under simulator"
            );
        }
        let root = unsafe { CStr::from_ptr(root) };

        let ptr = malloc(root.count_bytes() + cf.count_bytes() + 1);
        assert!(
            !ptr.is_null(),
            "failed allocating CoreFoundation framework path"
        );
        _cf_free = Deferred(move || unsafe { free(ptr) });

        let ptr = ptr.cast::<c_char>();
        unsafe { strcpy(ptr, root.as_ptr()) };
        unsafe { strcat(ptr, cf.as_ptr()) };
        unsafe { CStr::from_ptr(ptr) }
    } else {
        cf
    };

    let cf_handle = unsafe { dlopen(cf.as_ptr(), RTLD_LAZY | RTLD_LOCAL) };
    if cf_handle.is_null() {
        // let err = unsafe { CStr::from_ptr(dlerror()) };
        panic!("could not open CoreFoundation.framework");
    }
    let _handle_free = Deferred(|| {
        // Ignore errors when closing. This is also what `libloading` does:
        // https://docs.rs/libloading/0.8.6/src/libloading/os/unix/mod.rs.html#374
        let _ = unsafe { dlclose(cf_handle) };
    });

    macro_rules! dlsym {
        (
            unsafe fn $name:ident($($param:ident: $param_ty:ty),* $(,)?) $(-> $ret:ty)?;
        ) => {{
            let ptr = unsafe { dlsym(cf_handle, concat!(stringify!($name), '\0').as_bytes().as_ptr().cast()) };
            if ptr.is_null() {
                // let err = unsafe { CStr::from_ptr(dlerror()) };
                panic!("could not find function {}", stringify!($name));
            }
            unsafe { core::mem::transmute::<*mut c_void, unsafe extern "C-unwind" fn($($param_ty),*) $(-> $ret)?>(ptr) }
        }};
    }

    // MacTypes.h
    type Boolean = u8;
    // CoreFoundation/CFBase.h
    type CFTypeID = usize;
    type CFOptionFlags = usize;
    type CFIndex = isize;
    type CFTypeRef = *mut c_void;
    type CFAllocatorRef = CFTypeRef;
    const kCFAllocatorDefault: CFAllocatorRef = null_mut();
    let allocator_null = unsafe { dlsym(cf_handle, c"kCFAllocatorNull".as_ptr()) };
    if allocator_null.is_null() {
        // let err = unsafe { CStr::from_ptr(dlerror()) };
        panic!("could not find kCFAllocatorNull");
    }
    let kCFAllocatorNull = unsafe { *allocator_null.cast::<CFAllocatorRef>() };
    let CFRelease = dlsym!(
        unsafe fn CFRelease(cf: CFTypeRef);
    );
    let CFGetTypeID = dlsym!(
        unsafe fn CFGetTypeID(cf: CFTypeRef) -> CFTypeID;
    );
    // CoreFoundation/CFError.h
    type CFErrorRef = CFTypeRef;
    // CoreFoundation/CFData.h
    type CFDataRef = CFTypeRef;
    let CFDataCreateWithBytesNoCopy = dlsym!(
        unsafe fn CFDataCreateWithBytesNoCopy(
            allocator: CFAllocatorRef,
            bytes: *const u8,
            length: CFIndex,
            bytes_deallocator: CFAllocatorRef,
        ) -> CFDataRef;
    );
    // CoreFoundation/CFPropertyList.h
    const kCFPropertyListImmutable: CFOptionFlags = 0;
    type CFPropertyListFormat = CFIndex;
    type CFPropertyListRef = CFTypeRef;
    let CFPropertyListCreateWithData = dlsym!(
        unsafe fn CFPropertyListCreateWithData(
            allocator: CFAllocatorRef,
            data: CFDataRef,
            options: CFOptionFlags,
            format: *mut CFPropertyListFormat,
            error: *mut CFErrorRef,
        ) -> CFPropertyListRef;
    );
    // CoreFoundation/CFString.h
    type CFStringRef = CFTypeRef;
    type CFStringEncoding = u32;
    const kCFStringEncodingUTF8: CFStringEncoding = 0x08000100;
    let CFStringGetTypeID = dlsym!(
        unsafe fn CFStringGetTypeID() -> CFTypeID;
    );
    let CFStringCreateWithCStringNoCopy = dlsym!(
        unsafe fn CFStringCreateWithCStringNoCopy(
            alloc: CFAllocatorRef,
            c_str: *const c_char,
            encoding: CFStringEncoding,
            contents_deallocator: CFAllocatorRef,
        ) -> CFStringRef;
    );
    let CFStringGetCString = dlsym!(
        unsafe fn CFStringGetCString(
            the_string: CFStringRef,
            buffer: *mut c_char,
            buffer_size: CFIndex,
            encoding: CFStringEncoding,
        ) -> Boolean;
    );
    // CoreFoundation/CFDictionary.h
    type CFDictionaryRef = CFTypeRef;
    let CFDictionaryGetTypeID = dlsym!(
        unsafe fn CFDictionaryGetTypeID() -> CFTypeID;
    );
    let CFDictionaryGetValue = dlsym!(
        unsafe fn CFDictionaryGetValue(
            the_dict: CFDictionaryRef,
            key: *const c_void,
        ) -> *const c_void;
    );

    let plist_data = unsafe {
        CFDataCreateWithBytesNoCopy(
            kCFAllocatorDefault,
            plist_buffer.as_ptr(),
            plist_buffer.len() as CFIndex,
            kCFAllocatorNull,
        )
    };
    assert!(!plist_data.is_null(), "failed creating data");
    let _plist_data_release = Deferred(|| unsafe { CFRelease(plist_data) });

    let plist = unsafe {
        CFPropertyListCreateWithData(
            kCFAllocatorDefault,
            plist_data,
            kCFPropertyListImmutable,
            null_mut(), // Don't care about the format of the PList.
            null_mut(), // Don't care about the error data.
        )
    };
    assert!(
        !plist.is_null(),
        "failed reading PList in SystemVersion.plist"
    );
    let _plist_release = Deferred(|| unsafe { CFRelease(plist) });

    assert!(
        unsafe { CFGetTypeID(plist) } == unsafe { CFDictionaryGetTypeID() },
        "SystemVersion.plist did not contain a dictionary at the top level"
    );
    let plist = plist as CFDictionaryRef;

    // NOTE: Have to use a macro here instead of a closure, because a closure errors with:
    // "`compiler_builtins` cannot call functions through upstream monomorphizations".
    let get_string_key = |plist, lookup_key: &CStr| {
        let cf_lookup_key = unsafe {
            CFStringCreateWithCStringNoCopy(
                kCFAllocatorDefault,
                lookup_key.as_ptr(),
                kCFStringEncodingUTF8,
                kCFAllocatorNull,
            )
        };
        assert!(!cf_lookup_key.is_null(), "failed creating CFString");
        let _lookup_key_release = Deferred(|| unsafe { CFRelease(cf_lookup_key) });

        let value = unsafe { CFDictionaryGetValue(plist, cf_lookup_key) as CFTypeRef };
        // ^ getter, so don't release.

        if !value.is_null() {
            assert!(
                unsafe { CFGetTypeID(value) } == unsafe { CFStringGetTypeID() },
                "key in SystemVersion.plist must be a string"
            );
            let value = value as CFStringRef;

            let mut version_str = [0u8; 32];
            let ret = unsafe {
                CFStringGetCString(
                    value,
                    version_str.as_mut_ptr().cast::<c_char>(),
                    version_str.len() as CFIndex,
                    kCFStringEncodingUTF8,
                )
            };
            assert!(ret != 0, "failed getting string from CFString");

            let version_str = trim_trailing_nul(&version_str);

            Some(parse_os_version(version_str))
        } else {
            None
        }
    };

    // When `target_os = "ios"`, we may be in many different states:
    // - Native iOS device.
    // - iOS Simulator.
    // - Mac Catalyst.
    // - Mac + "Designed for iPad".
    // - Native visionOS device + "Designed for iPad".
    // - visionOS simulator + "Designed for iPad".
    //
    // Of these, only native, Mac Catalyst and simulators can be differentiated at compile-time
    // (with `target_abi = ""`, `target_abi = "macabi"` and `target_abi = "sim"` respectively).
    //
    // That is, "Designed for iPad" will act as iOS at compile-time, but the `ProductVersion` will
    // still be the host macOS or visionOS version.
    //
    // Furthermore, we can't even reliably differentiate between these at runtime, since
    // `dyld_get_active_platform` isn't publically available.
    //
    // Fortunately, we won't need to know any of that; we can simply attempt to get the
    // `iOSSupportVersion` (which may be set on native iOS too, but then it will be set to the host
    // iOS version), and if that fails, fall back to the `ProductVersion`.
    if cfg!(target_os = "ios") {
        if let Some(ios_support_version) = get_string_key(plist, c"iOSSupportVersion") {
            return ios_support_version;
        }

        // On Mac Catalyst, if we failed looking up `iOSSupportVersion`, we don't want to
        // accidentally fall back to `ProductVersion`.
        if cfg!(target_abi = "macabi") {
            panic!("expected iOSSupportVersion in SystemVersion.plist");
        }
    }

    // On all other platforms, we can find the OS version by simply looking at `ProductVersion`.
    get_string_key(plist, c"ProductVersion")
        .unwrap_or_else(|| panic!("expected ProductVersion in SystemVersion.plist"))
}

/// Read the version from `kern.osproductversion` or `kern.iossupportversion`.
///
/// This is faster than `version_from_plist`, since it doesn't need to invoke `dlsym`.
pub(super) fn version_from_sysctl() -> Option<OSVersion> {
    // This won't work in the simulator, as `kern.osproductversion` returns the host macOS version,
    // and `kern.iossupportversion` returns the host macOS' iOSSupportVersion (while you can run
    // simulators with many different iOS versions).
    if cfg!(target_abi = "sim") {
        return None;
    }

    // SAFETY: Same signatures as in `libc`.
    //
    // NOTE: We do not need to link this, that will be done by `std` by linking `libSystem`
    // (which is required on macOS/Darwin).
    unsafe extern "C" {
        unsafe fn sysctlbyname(
            name: *const c_char,
            oldp: *mut c_void,
            oldlenp: *mut usize,
            newp: *mut c_void,
            newlen: usize,
        ) -> c_uint;
    }

    // Same logic as in `version_from_plist`.
    if cfg!(target_os = "ios") {
        // https://github.com/apple-oss-distributions/xnu/blob/xnu-11215.81.4/bsd/kern/kern_sysctl.c#L2077-L2100
        let name = c"kern.iossupportversion".as_ptr();
        let mut buf: [u8; 32] = [0; 32];
        let mut size = buf.len();
        let ret = unsafe { sysctlbyname(name, buf.as_mut_ptr().cast(), &mut size, null_mut(), 0) };
        if ret != 0 {
            // This sysctl is not available.
            return None;
        }
        let buf = &buf[..(size - 1)];

        // The buffer may be empty when using `kern.iossupportversion` on iOS, or on visionOS when
        // running under "Designed for iPad". In that case, fall back to `kern.osproductversion`.
        if !buf.is_empty() {
            return Some(parse_os_version(buf));
        }

        // Force Mac Catalyst to use the iOSSupportVersion.
        if cfg!(target_abi = "macabi") {
            return None;
        }
    }

    // Introduced in macOS 10.13.4.
    // https://github.com/apple-oss-distributions/xnu/blob/xnu-11215.81.4/bsd/kern/kern_sysctl.c#L2015-L2051
    let name = c"kern.osproductversion".as_ptr();
    let mut buf: [u8; 32] = [0; 32];
    let mut size = buf.len();
    let ret = unsafe { sysctlbyname(name, buf.as_mut_ptr().cast(), &mut size, null_mut(), 0) };
    if ret != 0 {
        // This sysctl is not available.
        return None;
    }
    let buf = &buf[..(size - 1)];

    Some(parse_os_version(buf))
}

/// The version of the operating system.
///
/// We use a packed u32 here to allow for fast comparisons and to match Mach-O's `LC_BUILD_VERSION`.
pub(super) type OSVersion = u32;

/// Combine parts of a version into an [`OSVersion`].
///
/// The size of the parts are inherently limited by Mach-O's `LC_BUILD_VERSION`.
#[inline]
pub(super) const fn pack_os_version(major: u16, minor: u8, patch: u8) -> OSVersion {
    let (major, minor, patch) = (major as u32, minor as u32, patch as u32);
    (major << 16) | (minor << 8) | patch
}

/// We'd usually use `CStr::from_bytes_until_nul`, but that can't be used in `compiler-builtins`.
#[inline]
fn trim_trailing_nul(mut bytes: &[u8]) -> &[u8] {
    while let Some((b'\0', rest)) = bytes.split_last() {
        bytes = rest;
    }
    bytes
}

/// Parse an OS version from a bytestring like b"10.1" or b"14.3.7".
#[track_caller]
pub(super) const fn parse_os_version(bytes: &[u8]) -> OSVersion {
    let (major, bytes) = parse_usize(bytes);
    if major > u16::MAX as usize {
        panic!("major version is too large");
    }
    let major = major as u16;

    let bytes = if let Some((period, bytes)) = bytes.split_first() {
        if *period != b'.' {
            panic!("expected period between major and minor version")
        }
        bytes
    } else {
        return pack_os_version(major, 0, 0);
    };

    let (minor, bytes) = parse_usize(bytes);
    if minor > u8::MAX as usize {
        panic!("minor version is too large");
    }
    let minor = minor as u8;

    let bytes = if let Some((period, bytes)) = bytes.split_first() {
        if *period != b'.' {
            panic!("expected period after minor version")
        }
        bytes
    } else {
        return pack_os_version(major, minor, 0);
    };

    let (patch, bytes) = parse_usize(bytes);
    if patch > u8::MAX as usize {
        panic!("patch version is too large");
    }
    let patch = patch as u8;

    if !bytes.is_empty() {
        panic!("too many parts to version");
    }

    pack_os_version(major, minor, patch)
}

#[track_caller]
const fn parse_usize(mut bytes: &[u8]) -> (usize, &[u8]) {
    // Ensure we have at least one digit (that is not just a period).
    let mut ret: usize = if let Some((&ascii, rest)) = bytes.split_first() {
        bytes = rest;

        match ascii {
            b'0'..=b'9' => (ascii - b'0') as usize,
            _ => panic!("found invalid digit when parsing version"),
        }
    } else {
        panic!("found empty version number part")
    };

    // Parse the remaining digits.
    while let Some((&ascii, rest)) = bytes.split_first() {
        let digit = match ascii {
            b'0'..=b'9' => ascii - b'0',
            _ => break,
        };

        bytes = rest;

        // This handles leading zeroes as well.
        match ret.checked_mul(10) {
            Some(val) => match val.checked_add(digit as _) {
                Some(val) => ret = val,
                None => panic!("version is too large"),
            },
            None => panic!("version is too large"),
        };
    }

    (ret, bytes)
}
