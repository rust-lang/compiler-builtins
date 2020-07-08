use super::c_int;

// On most modern Intel and AMD processors, "rep movsq" and "rep stosq" have
// been enhanced to perform better than an simple qword loop, making them ideal
// for implementing memcpy/memset. Note that "rep cmps" has received no such
// enhancement, so it is not used to implement memcmp.
//
// On certain recent Intel processors, "rep movsb" and "rep stosb" have been
// further enhanced to automatically select the best microarchitectural
// implementation based on length and alignment. See the following features from
// the "Intel® 64 and IA-32 Architectures Optimization Reference Manual":
//  - ERMSB - Enhanced REP MOVSB and STOSB (Ivy Bridge and later)
//  - FSRM - Fast Short REP MOV (Ice Lake and later)
//  - Fast Zero-Length MOVSB (On no current hardware)
//  - Fast Short STOSB (On no current hardware)
// However, to avoid run-time feature detection, we don't use these byte-based
// instructions for most of the copying, preferring the qword variants.

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, count: usize) -> *mut u8 {
    asm!(
        "rep movsb [rdi], [rsi]",
        inout("rcx") count => _,
        inout("rdi") dest => _,
        inout("rsi") src => _,
        options(nostack, preserves_flags)
    );
    dest
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, count: usize) -> *mut u8 {
    let delta = dest as usize - src as usize;
    if delta >= count {
        // We can copy forwards because either dest is far enough ahead of src,
        // or src is ahead of dest (and delta overflowed).
        return self::memcpy(dest, src, count);
    }
    // copy backwards
    asm!(
        "std",
        "rep movsb [rdi], [rsi]",
        "cld",
        inout("rcx") count => _,
        inout("rdi") dest.add(count).sub(1) => _,
        inout("rsi") src.add(count).sub(1) => _,
        options(nostack, preserves_flags)
    );
    dest
}

#[cfg_attr(all(feature = "mem", not(feature = "mangled-names")), no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut u8, c: c_int, count: usize) -> *mut u8 {
    asm!(
        "rep stosb [rdi], al",
        inout("rcx") count => _,
        inout("rdi") dest => _,
        in("al") c as u8,
        options(nostack, preserves_flags)
    );
    dest
}
