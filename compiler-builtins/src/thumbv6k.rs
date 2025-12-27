// Armv6k supports atomic instructions, but they are unavailable in Thumb mode
// unless Thumb-2 instructions available (v6t2).
// Using Thumb interworking allows us to use these instructions even from Thumb mode
// without Thumb-2 instructions, but LLVM does not implement that processing (as of LLVM 21),
// so we implement it here at this time.

use core::arch::asm;
use core::mem;

// Data Memory Barrier (DMB) operation.
//
// Armv6 does not support DMB instruction, so use use special instruction equivalent to it.
//
// Refs: https://developer.arm.com/documentation/ddi0360/f/control-coprocessor-cp15/register-descriptions/c7--cache-operations-register
macro_rules! dmb {
    () => {
        "mcr p15, #0, {zero}, c7, c10, #5"
    };
}
macro_rules! asm_use_dmb {
    ($($asm:tt)*) => {
        // dmb! calls `mcr p15, 0, <Rd>, c7, c10, 5`, and
        // the value in the Rd register should be zero (SBZ).
        asm!(
            $($asm)*
            zero = inout(reg) 0_u32 => _,
        )
    };
}

#[instruction_set(arm::a32)]
unsafe fn fence() {
    unsafe {
        asm_use_dmb!(dmb!(), options(nostack, preserves_flags),);
    }
}

trait Atomic: Copy + Eq {
    unsafe fn load_relaxed(src: *const Self) -> Self;
    unsafe fn cmpxchg(dst: *mut Self, current: Self, new: Self) -> Self;
}

#[rustfmt::skip]
macro_rules! atomic {
    ($ty:ident, $suffix:tt) => {
        impl Atomic for $ty {
            // #[instruction_set(arm::a32)] is unneeded for ldr.
            #[inline]
            unsafe fn load_relaxed(
                src: *const Self,
            ) -> Self {
                let out: Self;
                unsafe {
                    asm!(
                        concat!("ldr", $suffix, " {out}, [{src}]"), // atomic { out = *src }
                        src = in(reg) src,
                        out = lateout(reg) out,
                        options(nostack, preserves_flags),
                    );
                }
                out
            }
            #[inline]
            #[instruction_set(arm::a32)]
            unsafe fn cmpxchg(
                dst: *mut Self,
                old: Self,
                new: Self,
            ) -> Self {
                let mut out: Self;
                unsafe {
                    asm_use_dmb!(
                            concat!("ldrex", $suffix, " {out}, [{dst}]"),      // atomic { out = *dst; EXCLUSIVE = dst }
                            "cmp {out}, {old}",                                // if out == old { Z = 1 } else { Z = 0 }
                            "bne 3f",                                          // if Z == 0 { jump 'cmp-fail }
                            dmb!(),                                            // fence
                        "2:", // 'retry:
                            concat!("strex", $suffix, " {r}, {new}, [{dst}]"), // atomic { if EXCLUSIVE == dst { *dst = new; r = 0 } else { r = 1 }; EXCLUSIVE = None }
                            "cmp {r}, #0",                                     // if r == 0 { Z = 1 } else { Z = 0 }
                            "beq 3f",                                          // if Z == 1 { jump 'success }
                            concat!("ldrex", $suffix, " {out}, [{dst}]"),      // atomic { out = *dst; EXCLUSIVE = dst }
                            "cmp {out}, {old}",                                // if out == old { Z = 1 } else { Z = 0 }
                            "beq 2b",                                          // if Z == 1 { jump 'retry }
                        "3:", // 'cmp-fail | 'success:
                            dmb!(),                                            // fence
                        dst = in(reg) dst,
                        // Note: this cast must be a zero-extend since loaded value
                        // which compared to it is zero-extended.
                        old = in(reg) old as u32,
                        new = in(reg) new,
                        out = out(reg) out,
                        r = out(reg) _,
                        // Do not use `preserves_flags` because CMP modifies the condition flags.
                        options(nostack),
                    );
                    out
                }
            }
        }
    };
}
atomic!(u8, "b");
atomic!(u16, "h");
atomic!(u32, "");

// To avoid the annoyance of sign extension, we implement signed CAS using
// unsigned CAS. (See note in cmpxchg impl in atomic! macro)
macro_rules! delegate_signed {
    ($ty:ident, $base:ident) => {
        const _: () = {
            assert!(mem::size_of::<$ty>() == mem::size_of::<$base>());
            assert!(mem::align_of::<$ty>() == mem::align_of::<$base>());
        };
        impl Atomic for $ty {
            #[inline]
            unsafe fn load_relaxed(src: *const Self) -> Self {
                // SAFETY: the caller must uphold the safety contract.
                // casts are okay because $ty and $base implement the same layout.
                unsafe { <$base as Atomic>::load_relaxed(src.cast::<$base>()) as Self }
            }
            #[inline]
            unsafe fn cmpxchg(dst: *mut Self, old: Self, new: Self) -> Self {
                // SAFETY: the caller must uphold the safety contract.
                // casts are okay because $ty and $base implement the same layout.
                unsafe {
                    <$base as Atomic>::cmpxchg(dst.cast::<$base>(), old as $base, new as $base)
                        as Self
                }
            }
        }
    };
}
delegate_signed!(i8, u8);
delegate_signed!(i16, u16);
delegate_signed!(i32, u32);

// Generic atomic read-modify-write operation
//
// We could implement RMW more efficiently as an assembly LL/SC loop per operation,
// but we won't do that for now because it would make the implementation more complex.
//
// We also do not implement LL and SC as separate functions. This is because it
// is theoretically possible for the compiler to insert operations that might
// clear the reservation between LL and SC. See https://github.com/taiki-e/portable-atomic/blob/58ef7f27c9e20da4cc1ef0abf8b8ce9ac5219ec3/src/imp/atomic128/aarch64.rs#L44-L55
// for more details.
unsafe fn atomic_rmw<T: Atomic, F: Fn(T) -> T, G: Fn(T, T) -> T>(ptr: *mut T, f: F, g: G) -> T {
    loop {
        // FIXME(safety): preconditions review needed
        let curval = unsafe { T::load_relaxed(ptr) };
        let newval = f(curval);
        // FIXME(safety): preconditions review needed
        if unsafe { T::cmpxchg(ptr, curval, newval) } == curval {
            return g(curval, newval);
        }
    }
}

macro_rules! atomic_rmw {
    ($name:ident, $ty:ty, $op:expr, $fetch:expr) => {
        intrinsics! {
            pub unsafe extern "C" fn $name(ptr: *mut $ty, val: $ty) -> $ty {
                // FIXME(safety): preconditions review needed
                unsafe {
                    atomic_rmw(
                        ptr,
                        |x| $op(x as $ty, val),
                        |old, new| $fetch(old, new)
                    ) as $ty
                }
            }
        }
    };

    (@old $name:ident, $ty:ty, $op:expr) => {
        atomic_rmw!($name, $ty, $op, |old, _| old);
    };

    (@new $name:ident, $ty:ty, $op:expr) => {
        atomic_rmw!($name, $ty, $op, |_, new| new);
    };
}
macro_rules! atomic_cmpxchg {
    ($name:ident, $ty:ty) => {
        intrinsics! {
            pub unsafe extern "C" fn $name(ptr: *mut $ty, oldval: $ty, newval: $ty) -> $ty {
                // FIXME(safety): preconditions review needed
                unsafe { <$ty as Atomic>::cmpxchg(ptr, oldval, newval) }
            }
        }
    };
}

atomic_rmw!(@old __sync_fetch_and_add_1, u8, |a: u8, b: u8| a.wrapping_add(b));
atomic_rmw!(@old __sync_fetch_and_add_2, u16, |a: u16, b: u16| a
    .wrapping_add(b));
atomic_rmw!(@old __sync_fetch_and_add_4, u32, |a: u32, b: u32| a
    .wrapping_add(b));

atomic_rmw!(@new __sync_add_and_fetch_1, u8, |a: u8, b: u8| a.wrapping_add(b));
atomic_rmw!(@new __sync_add_and_fetch_2, u16, |a: u16, b: u16| a
    .wrapping_add(b));
atomic_rmw!(@new __sync_add_and_fetch_4, u32, |a: u32, b: u32| a
    .wrapping_add(b));

atomic_rmw!(@old __sync_fetch_and_sub_1, u8, |a: u8, b: u8| a.wrapping_sub(b));
atomic_rmw!(@old __sync_fetch_and_sub_2, u16, |a: u16, b: u16| a
    .wrapping_sub(b));
atomic_rmw!(@old __sync_fetch_and_sub_4, u32, |a: u32, b: u32| a
    .wrapping_sub(b));

atomic_rmw!(@new __sync_sub_and_fetch_1, u8, |a: u8, b: u8| a.wrapping_sub(b));
atomic_rmw!(@new __sync_sub_and_fetch_2, u16, |a: u16, b: u16| a
    .wrapping_sub(b));
atomic_rmw!(@new __sync_sub_and_fetch_4, u32, |a: u32, b: u32| a
    .wrapping_sub(b));

atomic_rmw!(@old __sync_fetch_and_and_1, u8, |a: u8, b: u8| a & b);
atomic_rmw!(@old __sync_fetch_and_and_2, u16, |a: u16, b: u16| a & b);
atomic_rmw!(@old __sync_fetch_and_and_4, u32, |a: u32, b: u32| a & b);

atomic_rmw!(@new __sync_and_and_fetch_1, u8, |a: u8, b: u8| a & b);
atomic_rmw!(@new __sync_and_and_fetch_2, u16, |a: u16, b: u16| a & b);
atomic_rmw!(@new __sync_and_and_fetch_4, u32, |a: u32, b: u32| a & b);

atomic_rmw!(@old __sync_fetch_and_or_1, u8, |a: u8, b: u8| a | b);
atomic_rmw!(@old __sync_fetch_and_or_2, u16, |a: u16, b: u16| a | b);
atomic_rmw!(@old __sync_fetch_and_or_4, u32, |a: u32, b: u32| a | b);

atomic_rmw!(@new __sync_or_and_fetch_1, u8, |a: u8, b: u8| a | b);
atomic_rmw!(@new __sync_or_and_fetch_2, u16, |a: u16, b: u16| a | b);
atomic_rmw!(@new __sync_or_and_fetch_4, u32, |a: u32, b: u32| a | b);

atomic_rmw!(@old __sync_fetch_and_xor_1, u8, |a: u8, b: u8| a ^ b);
atomic_rmw!(@old __sync_fetch_and_xor_2, u16, |a: u16, b: u16| a ^ b);
atomic_rmw!(@old __sync_fetch_and_xor_4, u32, |a: u32, b: u32| a ^ b);

atomic_rmw!(@new __sync_xor_and_fetch_1, u8, |a: u8, b: u8| a ^ b);
atomic_rmw!(@new __sync_xor_and_fetch_2, u16, |a: u16, b: u16| a ^ b);
atomic_rmw!(@new __sync_xor_and_fetch_4, u32, |a: u32, b: u32| a ^ b);

atomic_rmw!(@old __sync_fetch_and_nand_1, u8, |a: u8, b: u8| !(a & b));
atomic_rmw!(@old __sync_fetch_and_nand_2, u16, |a: u16, b: u16| !(a & b));
atomic_rmw!(@old __sync_fetch_and_nand_4, u32, |a: u32, b: u32| !(a & b));

atomic_rmw!(@new __sync_nand_and_fetch_1, u8, |a: u8, b: u8| !(a & b));
atomic_rmw!(@new __sync_nand_and_fetch_2, u16, |a: u16, b: u16| !(a & b));
atomic_rmw!(@new __sync_nand_and_fetch_4, u32, |a: u32, b: u32| !(a & b));

atomic_rmw!(@old __sync_fetch_and_max_1, i8, |a: i8, b: i8| if a > b {
    a
} else {
    b
});
atomic_rmw!(@old __sync_fetch_and_max_2, i16, |a: i16, b: i16| if a > b {
    a
} else {
    b
});
atomic_rmw!(@old __sync_fetch_and_max_4, i32, |a: i32, b: i32| if a > b {
    a
} else {
    b
});

atomic_rmw!(@old __sync_fetch_and_umax_1, u8, |a: u8, b: u8| if a > b {
    a
} else {
    b
});
atomic_rmw!(@old __sync_fetch_and_umax_2, u16, |a: u16, b: u16| if a > b {
    a
} else {
    b
});
atomic_rmw!(@old __sync_fetch_and_umax_4, u32, |a: u32, b: u32| if a > b {
    a
} else {
    b
});

atomic_rmw!(@old __sync_fetch_and_min_1, i8, |a: i8, b: i8| if a < b {
    a
} else {
    b
});
atomic_rmw!(@old __sync_fetch_and_min_2, i16, |a: i16, b: i16| if a < b {
    a
} else {
    b
});
atomic_rmw!(@old __sync_fetch_and_min_4, i32, |a: i32, b: i32| if a < b {
    a
} else {
    b
});

atomic_rmw!(@old __sync_fetch_and_umin_1, u8, |a: u8, b: u8| if a < b {
    a
} else {
    b
});
atomic_rmw!(@old __sync_fetch_and_umin_2, u16, |a: u16, b: u16| if a < b {
    a
} else {
    b
});
atomic_rmw!(@old __sync_fetch_and_umin_4, u32, |a: u32, b: u32| if a < b {
    a
} else {
    b
});

atomic_rmw!(@old __sync_lock_test_and_set_1, u8, |_: u8, b: u8| b);
atomic_rmw!(@old __sync_lock_test_and_set_2, u16, |_: u16, b: u16| b);
atomic_rmw!(@old __sync_lock_test_and_set_4, u32, |_: u32, b: u32| b);

atomic_cmpxchg!(__sync_val_compare_and_swap_1, u8);
atomic_cmpxchg!(__sync_val_compare_and_swap_2, u16);
atomic_cmpxchg!(__sync_val_compare_and_swap_4, u32);

intrinsics! {
    pub unsafe extern "C" fn __sync_synchronize() {
       // SAFETY: preconditions are the same as the calling function.
       unsafe { fence() };
    }
}
