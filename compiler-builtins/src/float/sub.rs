intrinsics! {
    #[cfg(f16_enabled)]
    pub extern "C" fn __subhf3(a: f16, b: f16) -> f16 {
        crate::float::add::addsub::<_,true>(a, b)
    }

    #[arm_aeabi_alias = __aeabi_fsub]
    pub extern "C" fn __subsf3(a: f32, b: f32) -> f32 {
        crate::float::add::addsub::<_, true>(a, b)
    }

    #[arm_aeabi_alias = __aeabi_dsub]
    pub extern "C" fn __subdf3(a: f64, b: f64) -> f64 {
        crate::float::add::addsub::<_, true>(a, b)
    }

    #[ppc_name = __subkf3]
    #[cfg(f128_enabled)]
    pub extern "C" fn __subtf3(a: f128, b: f128) -> f128 {
        crate::float::add::addsub::<_, true>(a, b)
    }
}
