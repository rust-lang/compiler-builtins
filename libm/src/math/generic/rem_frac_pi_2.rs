#![allow(unused)]

use core::f64::consts;

use crate::support::{CastFrom, CastInto, DInt, Float, HInt, HalfRep, Int, MinInt};

pub(crate) trait RemFracPi2Support: Float<Int: DInt<H: Int>> {
    const TO_INT: Self;
    const INV_PIO2: Self;
    const PIO2_1: Self;
    const PIO2_1T: Self;
    const PIO2_2: Self;
    const PIO2_2T: Self;
    const PIO2_3: Self;
    const PIO2_3T: Self;

    const FRAC_5PI_4_HI: HalfRep<Self>;
    const FRAC_3PI_4_HI: HalfRep<Self>;
    const FRAC_9PI_4_HI: HalfRep<Self>;
    const FRAC_7PI_4_HI: HalfRep<Self>;
    const FRAC_3PI_2_HI: HalfRep<Self>;
    /// 2pi
    const TAU_HI: HalfRep<Self>;
    const FRAC_PI_2_HI: HalfRep<Self>;
    /// (2^20)(pi/2)
    const FRAC_2_POW_20_PI_2: HalfRep<Self>;

    const MAGIC: u32 = 23;
    const MAGIC_F: Self;

    fn large(x: &[Self], y: &mut [Self], e0: i32, prec: usize) -> i32;
}

const EPS: f64 = 2.2204460492503131e-16;

impl RemFracPi2Support for f64 {
    const TO_INT: Self = 1.5 / EPS;
    const INV_PIO2: Self = 6.36619772367581382433e-01;
    const PIO2_1: Self = 1.57079632673412561417e+00;
    const PIO2_1T: Self = 6.07710050650619224932e-11;
    const PIO2_2: Self = 6.07710050630396597660e-11;
    const PIO2_2T: Self = 2.02226624879595063154e-21;
    const PIO2_3: Self = 2.02226624871116645580e-21;
    const PIO2_3T: Self = 8.47842766036889956997e-32;
    const FRAC_5PI_4_HI: HalfRep<Self> = 0x400f6a7a;
    const FRAC_3PI_4_HI: HalfRep<Self> = 0x4002d97c;
    const FRAC_9PI_4_HI: HalfRep<Self> = 0x401c463b;
    const FRAC_7PI_4_HI: HalfRep<Self> = 0x4015fdbc;
    const FRAC_3PI_2_HI: HalfRep<Self> = 0x4012d97c;
    const TAU_HI: HalfRep<Self> = 0x401921fb;
    const FRAC_PI_2_HI: HalfRep<Self> = 0x921fb;
    const FRAC_2_POW_20_PI_2: HalfRep<Self> = 0x413921fb;

    const MAGIC_F: Self = hf64!("0x1p24");

    fn large(x: &[Self], y: &mut [Self], e0: i32, prec: usize) -> i32 {
        super::super::rem_pio2_large(x, y, e0, prec)
    }
}

pub(crate) fn rem_frac_pi_2<F>(x: F) -> (i32, F, F)
where
    F: RemFracPi2Support,
    F: CastInto<i32>,
    HalfRep<F>: Int + MinInt<Unsigned: Int<OtherSign: Int>>,
    // <HalfRep<F> as Int>::Unsigned: Int,
{
    // let sign = x.is_sign_positive()

    let ix: HalfRep<F> = x.abs().to_bits().hi();
    let pos = x.is_sign_positive();

    if ix <= F::FRAC_5PI_4_HI {
        /* |x| ~<= 5pi/4 */
        if (ix & F::SIG_MASK.hi()) == F::FRAC_PI_2_HI {
            /* |x| ~= pi/2 or 2pi/2 */
            return medium(x, ix); /* cancellation -- use medium case */
        }

        if ix <= F::FRAC_3PI_4_HI {
            /* |x| ~<= 3pi/4 */
            if pos {
                let z = x - F::PIO2_1; /* one round good to 85 bits */
                let y0 = z - F::PIO2_1T;
                let y1 = (z - y0) - F::PIO2_1T;
                return (1, y0, y1);
            } else {
                let z = x + F::PIO2_1;
                let y0 = z + F::PIO2_1T;
                let y1 = (z - y0) + F::PIO2_1T;
                return (-1, y0, y1);
            }
        } else if pos {
            let z = x - F::TWO * F::PIO2_1;
            let y0 = z - F::TWO * F::PIO2_1T;
            let y1 = (z - y0) - F::TWO * F::PIO2_1T;
            return (2, y0, y1);
        } else {
            let z = x + F::TWO * F::PIO2_1;
            let y0 = z + F::TWO * F::PIO2_1T;
            let y1 = (z - y0) + F::TWO * F::PIO2_1T;
            return (-2, y0, y1);
        }
    }

    if ix <= F::FRAC_9PI_4_HI {
        /* |x| ~<= 9pi/4 */
        if ix <= F::FRAC_7PI_4_HI {
            /* |x| ~<= 7pi/4 */
            if ix == F::FRAC_3PI_2_HI {
                /* |x| ~= 3pi/2 */
                return medium(x, ix);
            }

            if pos {
                let z = x - F::THREE * F::PIO2_1;
                let y0 = z - F::THREE * F::PIO2_1T;
                let y1 = (z - y0) - F::THREE * F::PIO2_1T;
                return (3, y0, y1);
            } else {
                let z = x + F::THREE * F::PIO2_1;
                let y0 = z + F::THREE * F::PIO2_1T;
                let y1 = (z - y0) + F::THREE * F::PIO2_1T;
                return (-3, y0, y1);
            }
        } else {
            if ix == F::TAU_HI {
                /* |x| ~= 4pi/2 */
                return medium(x, ix);
            }

            if pos {
                let z = x - F::FOUR * F::PIO2_1;
                let y0 = z - F::FOUR * F::PIO2_1T;
                let y1 = (z - y0) - F::FOUR * F::PIO2_1T;
                return (4, y0, y1);
            } else {
                let z = x + F::FOUR * F::PIO2_1;
                let y0 = z + F::FOUR * F::PIO2_1T;
                let y1 = (z - y0) + F::FOUR * F::PIO2_1T;
                return (-4, y0, y1);
            }
        }
    }

    if ix < F::FRAC_2_POW_20_PI_2 {
        /* |x| ~< 2^20*(pi/2), medium size */
        return medium(x, ix);
    }
    /*

    * all other (large) arguments
    */
    if ix >= F::EXP_MASK.hi() {
        /* x is inf or NaN */
        let y0 = x - x;
        let y1 = y0;
        return (0, y0, y1);
    }

    /* set z = scalbn(|x|,-ilogb(x)+23) */
    let mut ui = x.to_bits();
    ui &= F::SIG_MASK;
    // ui |= F::Int::cast_from((F::EXP_BIAS + F::MAGIC) << F::SIG_BITS);
    ui |= F::Int::cast_from(F::EXP_BIAS + F::MAGIC) << F::SIG_BITS;

    let mut z = F::from_bits(ui);
    let mut tx = [F::ZERO; 3];

    for i in 0..2 {
        // ??
        i!(tx, i, =, super::trunc(z));
        z = (z - i!(tx, i)) * F::MAGIC_F;
    }

    i!(tx,2, =, z);

    /* skip zero terms, first term is non-zero */
    let mut i = 2;
    while i != 0 && i!(tx, i) == F::ZERO {
        i -= 1;
    }

    let mut ty = [F::ZERO; 3];

    let ex: i32 = (ix >> (HalfRep::<F>::BITS - F::EXP_BITS - 1)).cast();
    let n = F::large(&tx[..=i], &mut ty, ex - (F::EXP_BIAS + F::MAGIC) as i32, 1);

    if !pos {
        return (-n, -i!(ty, 0), -i!(ty, 1));
    } else {
        (n, i!(ty, 0), i!(ty, 1))
    }
}

pub fn medium<F>(x: F, ix: HalfRep<F>) -> (i32, F, F)
where
    F: RemFracPi2Support,
    F: CastInto<i32>,
    HalfRep<F>: Int,
{
    /* rint(x/(pi/2)), Assume round-to-nearest. */
    let tmp = x * F::INV_PIO2 + F::TO_INT;
    // force rounding of tmp to its storage format on x87 to avoid
    // excess precision issues.
    #[cfg(all(target_arch = "x86", not(target_feature = "sse2")))]
    let tmp = force_eval!(tmp);
    let f_n = tmp - F::TO_INT;
    let n: i32 = f_n.cast();
    let mut r = x - f_n * F::PIO2_1;
    let mut w = f_n * F::PIO2_1T; /* 1st round, good to 85 bits */
    let mut y0 = r - w;
    let ui = y0.to_bits();
    let ey = y0.ex().signed();
    let ex: i32 = (ix >> (HalfRep::<F>::BITS - F::EXP_BITS - 1)).cast();

    // (ix >> 20) as i32;
    if ex - ey > 16 {
        /* 2nd round, good to 118 bits */
        let t = r;
        w = f_n * F::PIO2_2;
        r = t - w;
        w = f_n * F::PIO2_2T - ((t - r) - w);
        y0 = r - w;
        let ey = y0.ex().signed();
        if ex - ey > 49 {
            /* 3rd round, good to 151 bits, covers all cases */
            let t = r;
            w = f_n * F::PIO2_3;
            r = t - w;
            w = f_n * F::PIO2_3T - ((t - r) - w);
            y0 = r - w;
        }
    }
    let y1 = (r - y0) - w;
    (n, y0, y1)
}
