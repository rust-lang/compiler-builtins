//! Builtins exports are nested in modules with GCC names and potentially different ABIs. Wrap
//! these to make them a bit more similar to the rest of the libm functions.

macro_rules! binop {
    ($op:ident, $ty:ty, $sfx:ident) => {
        paste::paste! {
            pub fn [< $op $ty >](a: $ty, b: $ty) -> $ty {
                compiler_builtins::float::$op::[< __ $op $sfx >](a, b)
            }
        }
    };
}

binop!(add, f32, sf3);
binop!(sub, f32, sf3);
binop!(mul, f32, sf3);
binop!(div, f32, sf3);
binop!(add, f64, df3);
binop!(sub, f64, df3);
binop!(mul, f64, df3);
binop!(div, f64, df3);
#[cfg(f128_enabled)]
binop!(add, f128, tf3);
#[cfg(f128_enabled)]
binop!(sub, f128, tf3);
#[cfg(f128_enabled)]
binop!(mul, f128, tf3);
#[cfg(f128_enabled)]
binop!(div, f128, tf3);

pub fn powif32(a: f32, b: i32) -> f32 {
    compiler_builtins::float::pow::__powisf2(a, b)
}

pub fn powif64(a: f64, b: i32) -> f64 {
    compiler_builtins::float::pow::__powidf2(a, b)
}

#[cfg(f128_enabled)]
pub fn powif128(a: f128, b: i32) -> f128 {
    compiler_builtins::float::pow::__powitf2(a, b)
}
