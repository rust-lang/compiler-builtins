#![feature(test, f16, f128)]

extern crate test;
use core::hint::black_box;
use test::Bencher;

extern crate compiler_builtins;

macro_rules! test_values {
    ($ty:ty) => {
        &[
            <$ty>::MIN,
            <$ty>::MAX,
            <$ty>::NAN,
            <$ty>::INFINITY,
            <$ty>::NEG_INFINITY,
            <$ty>::MIN_POSITIVE,
            0.0,
            1.0,
            -1.0,
        ]
    };
}

fn combine2<T: Copy>(vals: &[T]) -> Vec<(T, T)> {
    let mut ret = Vec::new();
    for x in vals.iter().copied() {
        for y in vals.iter().copied() {
            ret.push((x, y));
        }
    }
    ret
}

macro_rules! test_iter {
    ($b:ident, $ty:ty, $fn:path) => {{
        let vals = combine2(test_values!($ty));
        let iter_loop = || {
            for (a, b) in vals.iter().copied() {
                black_box($fn(black_box(a), black_box(b)));
            }
        };

        // Warmup
        for _ in 0..1000 {
            iter_loop();
        }

        $b.iter(iter_loop);
    }};
}

macro_rules! foobar {
    ($($ty:ty, $rust_fn:ident, $builtin_fn:ident, $mod:ident::$sym:ident);* $(;)?) => {
        $(
            #[bench]
            fn $rust_fn(b: &mut Bencher) {
                // Equalize with the builtin function which is called separately
                #[inline(never)]
                fn inline_wrapper(a: $ty, b: $ty) -> $ty {
                    compiler_builtins::float::$mod::$sym(black_box(a), black_box(b))
                }

                test_iter!(b, $ty, inline_wrapper);
            }

            #[bench]
            fn $builtin_fn(b: &mut Bencher) {
                extern "C" {
                    fn $sym(a: $ty, b: $ty) -> $ty;
                }

                unsafe {
                    test_iter!(b, $ty, $sym);
                }
            }
        )*
    };
}

foobar! {
    f32, addsf3_rust, addsf3_builtin, add::__addsf3;
    f32, subsf3_rust, subsf3_builtin, sub::__subsf3;
    f32, mulsf3_rust, mulsf3_builtin, mul::__mulsf3;
    f32, divsf3_rust, divsf3_builtin, div::__divsf3;
    f64, adddf3_rust, adddf3_builtin, add::__adddf3;
    f64, subdf3_rust, subdf3_builtin, sub::__subdf3;
    f64, muldf3_rust, muldf3_builtin, mul::__muldf3;
    f64, divdf3_rust, divdf3_builtin, div::__divdf3;
}
