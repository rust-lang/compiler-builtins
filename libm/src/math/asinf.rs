use super::fabsf::fabsf;
use super::sqrt::sqrt;

const PIO2: f64 = 1.570796326794896558e+00;

/* coefficients for R(x^2) */
const P_S0: f32 = 1.6666586697e-01;
const P_S1: f32 = -4.2743422091e-02;
const P_S2: f32 = -8.6563630030e-03;
const Q_S1: f32 = -7.0662963390e-01;

fn r(z: f32) -> f32 {
    let p = z * (P_S0 + z * (P_S1 + z * P_S2));
    let q = 1. + z * Q_S1;
    p / q
}

#[inline]
pub fn asinf(mut x: f32) -> f32 {
    let x1p_120 = f64::from_bits(0x3870000000000000); // 0x1p-120 === 2 ^ (-120)

    let hx = x.to_bits();
    let ix = hx & 0x7fffffff;

    if ix >= 0x3f800000 {
        /* |x| >= 1 */
        if ix == 0x3f800000 {
            /* |x| == 1 */
            return ((x as f64) * PIO2 + x1p_120) as f32; /* asin(+-1) = +-pi/2 with inexact */
        }
        return 0. / (x - x); /* asin(|x|>1) is NaN */
    }

    if ix < 0x3f000000 {
        /* |x| < 0.5 */
        /* if 0x1p-126 <= |x| < 0x1p-12, avoid raising underflow */
        if (ix < 0x39800000) && (ix >= 0x00800000) {
            return x;
        }
        return x + x * r(x * x);
    }

    /* 1 > |x| >= 0.5 */
    let z = (1. - fabsf(x)) * 0.5;
    let s = sqrt(z as f64);
    x = (PIO2 - 2. * (s + s * (r(z) as f64))) as f32;
    if (hx >> 31) != 0 {
        -x
    } else {
        x
    }
}
