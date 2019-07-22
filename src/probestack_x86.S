// This is the same as x86_64, only translated for 32-bit sizes. Note that on
// Unix we're expected to restore everything as it was, this function basically
// can't tamper with anything.
//
// The ABI here is the same as x86_64, except everything is 32-bits large.

.text
.globl  __rust_probestack
.type   __rust_probestack, @function
__rust_probestack:
    .cfi_startproc
    pushq  %ebp
    .cfi_def_cfa_offset 8
    .cfi_offset 6, -8
    movq   %esp, %ebp
    .cfi_def_cfa_register 6
    push   %ecx
    mov    %eax,%ecx

    cmp    $0x1000,%ecx
    jna    3f
2:
    sub    $0x1000,%esp
    test   %esp,8(%esp)
    sub    $0x1000,%ecx
    cmp    $0x1000,%ecx
    ja     2b

3:
    sub    %ecx,%esp
    test   %esp,8(%esp)

    add    %eax,%esp
    pop    %ecx
    leave
    .cfi_def_cfa 7, 8
    ret
    .cfi_endproc
