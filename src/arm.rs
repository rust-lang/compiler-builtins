// FIXME: The `*4` and `*8` variants should be defined as aliases.

#[no_mangle]
pub unsafe extern "aapcs" fn __aeabi_memcpy(dest: *mut u8, src: *const u8, n: usize) {
    ::libc::memcpy(dest, src, n);
}
#[no_mangle]
pub unsafe extern "aapcs" fn __aeabi_memcpy4(dest: *mut u8, src: *const u8, n: usize) {
    ::libc::memcpy(dest, src, n);
}
#[no_mangle]
pub unsafe extern "aapcs" fn __aeabi_memcpy8(dest: *mut u8, src: *const u8, n: usize) {
    ::libc::memcpy(dest, src, n);
}

#[no_mangle]
pub unsafe extern "aapcs" fn __aeabi_memmove(dest: *mut u8, src: *const u8, n: usize) {
    ::libc::memmove(dest, src, n);
}
#[no_mangle]
pub unsafe extern "aapcs" fn __aeabi_memmove4(dest: *mut u8, src: *const u8, n: usize) {
    ::libc::memmove(dest, src, n);
}
#[no_mangle]
pub unsafe extern "aapcs" fn __aeabi_memmove8(dest: *mut u8, src: *const u8, n: usize) {
    ::libc::memmove(dest, src, n);
}

// Note the different argument order
#[no_mangle]
pub unsafe extern "aapcs" fn __aeabi_memset(dest: *mut u8, n: usize, c: i32) {
    ::libc::memset(dest, c, n);
}
#[no_mangle]
pub unsafe extern "aapcs" fn __aeabi_memset4(dest: *mut u8, n: usize, c: i32) {
    ::libc::memset(dest, c, n);
}
#[no_mangle]
pub unsafe extern "aapcs" fn __aeabi_memset8(dest: *mut u8, n: usize, c: i32) {
    ::libc::memset(dest, c, n);
}

#[no_mangle]
pub unsafe extern "aapcs" fn __aeabi_memclr(dest: *mut u8, n: usize) {
    ::libc::memset(dest, 0, n);
}
#[no_mangle]
pub unsafe extern "aapcs" fn __aeabi_memclr4(dest: *mut u8, n: usize) {
    ::libc::memset(dest, 0, n);
}
#[no_mangle]
pub unsafe extern "aapcs" fn __aeabi_memclr8(dest: *mut u8, n: usize) {
    ::libc::memset(dest, 0, n);
}
