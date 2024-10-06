// FIXME: This module existing reflects a failure of the codegen backend to always legalize bitops.
// LLVM should be taught how to always emit llvm.fcopysign for f{16,32,64,128} without needing this!

// FIXME: delete when we move fabs to core and it reaches stable
#[cfg(not(any(target_env = "msvc", target_vendor = "apple")))]
fn fabs_f32(f: f32) -> f32 {
    f32::from_bits(f.to_bits() & const { !(i32::MIN as u32) })
}

// FIXME: delete when we move fabs to core and it reaches stable
#[cfg(not(any(target_env = "msvc", target_vendor = "apple")))]
fn fabs_f64(f: f64) -> f64 {
    f64::from_bits(f.to_bits() & const { !(i64::MIN as u64) })
}

// FIXME: delete when we move fabs to core and it reaches stable
#[cfg(f128_enabled)]
fn fabs_128(f: f128) -> f128 {
    f128::from_bits(f.to_bits() & const { !(i128::MIN as u128) })
}

#[cfg(not(any(target_env = "msvc", target_vendor = "apple")))]
fn copysign_f32(magnitude: f32, sign: f32) -> f32 {
    let sign = fabs_f32(sign).to_bits() ^ sign.to_bits();
    f32::from_bits(fabs_f32(magnitude).to_bits() | sign)
}

#[cfg(not(any(target_env = "msvc", target_vendor = "apple")))]
fn copysign_f64(magnitude: f64, sign: f64) -> f64 {
    let sign = fabs_f64(sign).to_bits() ^ sign.to_bits();
    f64::from_bits(fabs_f64(magnitude).to_bits() | sign)
}

#[cfg(f128_enabled)]
fn copysign_f128(magnitude: f128, sign: f128) -> f128 {
    let sign = fabs_128(sign).to_bits() ^ sign.to_bits();
    f128::from_bits(fabs_128(magnitude).to_bits() | sign)
}

intrinsics! {
    #[cfg(not(any(target_env = "msvc", target_vendor = "apple")))]
    pub extern "C" fn fcopysign(magnitude: f32, sign: f32) -> f32 {
        copysign_f32(magnitude, sign)
    }

    #[cfg(not(any(target_env = "msvc", target_vendor = "apple")))]
    pub extern "C" fn copysign(magnitude: f64, sign: f64) -> f64 {
        copysign_f64(magnitude, sign)
    }

    #[cfg(f128_enabled)]
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "powerpc", target_arch = "powerpc64")))]
    pub extern "C" fn copysignl(magnitude: f128, sign: f128) -> f128 {
        copysign_f128(magnitude, sign)
    }

    #[cfg(f128_enabled)]
    #[cfg(any(target_arch = "x86_64", target_arch = "x86_64", target_arch = "powerpc", target_arch = "powerpc64"))]
    pub extern "C" fn copysignf128(magnitude: f128, sign: f128) -> f128 {
        copysign_f128(magnitude, sign)
    }
}
