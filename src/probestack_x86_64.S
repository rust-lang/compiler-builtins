// Our goal here is to touch each page between %rsp+8 and %rsp+8-%rax,
// ensuring that if any pages are unmapped we'll make a page fault.
//
// The ABI here is that the stack frame size is located in `%eax`. Upon
// return we're not supposed to modify `%esp` or `%eax`.

.text
.p2align 4,,15
.globl  __rust_probestack
.type   __rust_probestack, @function
__rust_probestack:
    .cfi_startproc
    pushq   %rbp
    .cfi_def_cfa_offset 16
    .cfi_offset 6, -16
    movq    %rsp, %rbp
    .cfi_def_cfa_register 6
    mov    %rax,%r11
    // duplicate %rax as we're clobbering %r11

    // Main loop, taken in one page increments. We're decrementing rsp by
    // a page each time until there's less than a page remaining. We're
    // guaranteed that this function isn't called unless there's more than a
    // page needed.
    //
    // Note that we're also testing against `8(%rsp)` to account for the 8
    // bytes pushed on the stack orginally with our return address. Using
    // `8(%rsp)` simulates us testing the stack pointer in the caller's
    // context.

    // It's usually called when %rax >= 0x1000, but that's not always true.
    // Dynamic stack allocation, which is needed to implement unsized
    // rvalues, triggers stackprobe even if %rax < 0x1000.
    // Thus we have to check %r11 first to avoid segfault.
    cmp    $0x1000,%r11
    jna    3f
2:
    sub    $0x1000,%rsp
    test   %rsp,8(%rsp)
    sub    $0x1000,%r11
    cmp    $0x1000,%r11
    ja     2b

3:
    // Finish up the last remaining stack space requested, getting the last
    // bits out of r11
    sub    %r11,%rsp
    test   %rsp,8(%rsp)

    // Restore the stack pointer to what it previously was when entering
    // this function. The caller will readjust the stack pointer after we
    // return.
    add    %rax,%rsp
    leave
    .cfi_def_cfa 7, 8
    ret
    .cfi_endproc
