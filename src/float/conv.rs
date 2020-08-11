intrinsics! {
    #[arm_aeabi_alias = __aeabi_i2f]
    pub extern "C" fn __floatsisf(i: i32) -> f32 {
        floatconv::fast::i32_to_f32_round(i)
    }

    #[arm_aeabi_alias = __aeabi_i2d]
    pub extern "C" fn __floatsidf(i: i32) -> f64 {
        floatconv::fast::i32_to_f64(i)
    }

    #[arm_aeabi_alias = __aeabi_l2f]
    pub extern "C" fn __floatdisf(i: i64) -> f32 {
        floatconv::fast::i64_to_f32_round(i)
    }

    #[arm_aeabi_alias = __aeabi_l2d]
    pub extern "C" fn __floatdidf(i: i64) -> f64 {
        floatconv::fast::i64_to_f64_round(i)
    }

    #[unadjusted_on_win64]
    pub extern "C" fn __floattisf(i: i128) -> f32 {
        floatconv::fast::i128_to_f32_round(i)
    }

    #[unadjusted_on_win64]
    pub extern "C" fn __floattidf(i: i128) -> f64 {
        floatconv::fast::i128_to_f64_round(i)
    }

    #[arm_aeabi_alias = __aeabi_ui2f]
    pub extern "C" fn __floatunsisf(i: u32) -> f32 {
        floatconv::fast::u32_to_f32_round(i)
    }

    #[arm_aeabi_alias = __aeabi_ui2d]
    pub extern "C" fn __floatunsidf(i: u32) -> f64 {
        floatconv::fast::u32_to_f64(i)
    }

    #[arm_aeabi_alias = __aeabi_ul2f]
    pub extern "C" fn __floatundisf(i: u64) -> f32 {
        floatconv::fast::u64_to_f32_round(i)
    }

    #[arm_aeabi_alias = __aeabi_ul2d]
    pub extern "C" fn __floatundidf(i: u64) -> f64 {
        floatconv::fast::u64_to_f64_round(i)
    }

    #[unadjusted_on_win64]
    pub extern "C" fn __floatuntisf(i: u128) -> f32 {
        floatconv::fast::u128_to_f32_round(i)
    }

    #[unadjusted_on_win64]
    pub extern "C" fn __floatuntidf(i: u128) -> f64 {
        floatconv::fast::u128_to_f64_round(i)
    }

    #[arm_aeabi_alias = __aeabi_f2iz]
    pub extern "C" fn __fixsfsi(f: f32) -> i32 {
        floatconv::fast::f32_to_i32(f)
    }

    #[arm_aeabi_alias = __aeabi_f2lz]
    pub extern "C" fn __fixsfdi(f: f32) -> i64 {
        floatconv::fast::f32_to_i64(f)
    }

    #[unadjusted_on_win64]
    pub extern "C" fn __fixsfti(f: f32) -> i128 {
        floatconv::fast::f32_to_i128(f)
    }

    #[arm_aeabi_alias = __aeabi_d2iz]
    pub extern "C" fn __fixdfsi(f: f64) -> i32 {
        floatconv::fast::f64_to_i32(f)
    }

    #[arm_aeabi_alias = __aeabi_d2lz]
    pub extern "C" fn __fixdfdi(f: f64) -> i64 {
        floatconv::fast::f64_to_i64(f)
    }

    #[unadjusted_on_win64]
    pub extern "C" fn __fixdfti(f: f64) -> i128 {
        floatconv::fast::f64_to_i128(f)
    }

    #[arm_aeabi_alias = __aeabi_f2uiz]
    pub extern "C" fn __fixunssfsi(f: f32) -> u32 {
        floatconv::fast::f32_to_u32(f)
    }

    #[arm_aeabi_alias = __aeabi_f2ulz]
    pub extern "C" fn __fixunssfdi(f: f32) -> u64 {
        floatconv::fast::f32_to_u64(f)
    }

    #[unadjusted_on_win64]
    pub extern "C" fn __fixunssfti(f: f32) -> u128 {
        floatconv::fast::f32_to_u128(f)
    }

    #[arm_aeabi_alias = __aeabi_d2uiz]
    pub extern "C" fn __fixunsdfsi(f: f64) -> u32 {
        floatconv::fast::f64_to_u32(f)
    }

    #[arm_aeabi_alias = __aeabi_d2ulz]
    pub extern "C" fn __fixunsdfdi(f: f64) -> u64 {
        floatconv::fast::f64_to_u64(f)
    }

    #[unadjusted_on_win64]
    pub extern "C" fn __fixunsdfti(f: f64) -> u128 {
        floatconv::fast::f64_to_u128(f)
    }
}
