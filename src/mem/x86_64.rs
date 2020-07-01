use super::c_int;

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
