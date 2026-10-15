/* SPDX-License-Identifier: MIT */
/* origin: core-math/src/binary32/log2p1/log2p1f.c
 * Copyright (c) 2022-2026 Alexei Sibidanov.
 * Ported to Rust in 2026, malezjaa
 * CORE-MATH commit: 8ea8ea35c518d06b72de4cf1d528e7a2d16f104c
 */

use super::super::support::{CastFrom, CastInto, cold_path};

/// Base-2 logarithm of x plus one (f32).
///
/// Calculate `log2(x + 1)`, that is, the base-2 logarithm of `x + 1`.
#[cfg_attr(assert_no_panic, no_panic::no_panic)]
pub fn log2p1f(x: f32) -> f32 {
    cr_log2p1f(x)
}

#[inline(never)]
fn as_special(x: f32) -> f32 {
    let ux = x.to_bits();
    let ax = ux << 1;
    if ux == 0xbf800000 {
        // x = -1.0
        return -1.0 / 0.0; // to raise FE_DIVBYZERO
    }
    if ux == 0x7f800000 {
        return x; // +inf
    }
    if ax > 0xff000000 {
        return x + x; // nan
    }
    0.0 / 0.0 // to raise FE_INVALID and return nan
}

fn cr_log2p1f(x: f32) -> f32 {
    let z = f64::from(x);
    let ux = x.to_bits();

    if ux >= 0xbf800000 {
        cold_path();
        return as_special(x); // x<=-1, x=-inf, x=-nan
    }

    let ax = ux & (u32::MAX >> 1);
    if ax >= 0x7f800000 {
        cold_path();
        return as_special(x); // x=+inf, x=+nan
    }

    if ax < 0x3cc00000 {
        // |x|<0.0234375
        cold_path();
        if ax <= 0x58b90b {
            // |x|<=0x1-126*ln(2)
            if ax == 0 {
                return x; // log2p1(-0.0) = -0.0 and log2p1(+0.0) = +0.0
            }
            return (z * hf64!("0x1.71547652b82fep+0")).cast_lossy();
        } else {
            let z2 = z * z;
            let z4 = z2 * z2;
            let mut f = z
                * ((G[0] + z * G[1])
                    + z2 * (G[2] + z * G[3])
                    + z4 * ((G[4] + z * G[5]) + z2 * (G[6] + z * G[7])));
            f += z * hf64!("0x1.715476p+0"); // the product is exact
            return f.cast_lossy();
        }
    }

    let tp = (z + 1.0).to_bits();
    let e = i32::cast_from(tp >> 52) - 0x3ff;
    let m = tp & (u64::MAX >> 12);
    if m == 0 {
        cold_path();
        return e.cast(); // do not raise the inexact exception for 1+x = 2^n
    }

    let j = usize::cast_from((m + (1 << (52 - 7))) >> (52 - 6));
    let xd = f64::from_bits(m | (0x3ff << 52));
    let d = xd * i!(IX, j) - 1.0; // d is exact for x < 0x1.04p+29
    let d2 = d * d;
    let el = f64::from(e) - i!(LIX, j);
    let mut f = (el + d * B[0]) + d2 * (B[1] + d * B[2]);
    let mut lb: f32 = f.cast_lossy();
    let ub: f32 = (f + hf64!("0x1.661p-32")).cast_lossy();
    if lb == ub {
        return lb;
    }

    cold_path();
    for &(tx, f, df) in &TB {
        if ux == tx.to_bits() {
            return f + df;
        }
    }
    let mut c0 = C[0] + d * C[1];
    let c2 = C[2] + d * C[3];
    let c4 = C[4] + d * C[5];
    c0 += d2 * (c2 + d2 * c4);
    f = f64::from(e) + (hf64!("0x1.dp-45") - i!(LIX, j)) + d * c0;

    f.cast_lossy()
}

#[rustfmt::skip]
static TB: [(f32, f32, f32); 2] = [
    (hf32!("0x1.7a13c6p+30"), hf32!("0x1.e90026p+4"), hf32!("0x1p-21")),
    (hf32!("-0x1.da285cp-5"), hf32!("-0x1.60549p-4"), hf32!("0x1p-29")),
];

/// The reciprocal 1/(1+j/64) is rounded to 24 bits.
static IX: [f64; 65] = [
    hf64!("0x1p+0"),
    hf64!("0x1.f81f82p-1"),
    hf64!("0x1.f07c2p-1"),
    hf64!("0x1.e9131ap-1"),
    hf64!("0x1.e1e1e2p-1"),
    hf64!("0x1.dae608p-1"),
    hf64!("0x1.d41d42p-1"),
    hf64!("0x1.cd8568p-1"),
    hf64!("0x1.c71c72p-1"),
    hf64!("0x1.c0e07p-1"),
    hf64!("0x1.bacf92p-1"),
    hf64!("0x1.b4e81cp-1"),
    hf64!("0x1.af286cp-1"),
    hf64!("0x1.a98ef6p-1"),
    hf64!("0x1.a41a42p-1"),
    hf64!("0x1.9ec8eap-1"),
    hf64!("0x1.99999ap-1"),
    hf64!("0x1.948b1p-1"),
    hf64!("0x1.8f9c18p-1"),
    hf64!("0x1.8acb9p-1"),
    hf64!("0x1.861862p-1"),
    hf64!("0x1.818182p-1"),
    hf64!("0x1.7d05f4p-1"),
    hf64!("0x1.78a4c8p-1"),
    hf64!("0x1.745d18p-1"),
    hf64!("0x1.702e06p-1"),
    hf64!("0x1.6c16c2p-1"),
    hf64!("0x1.681682p-1"),
    hf64!("0x1.642c86p-1"),
    hf64!("0x1.605816p-1"),
    hf64!("0x1.5c9882p-1"),
    hf64!("0x1.58ed24p-1"),
    hf64!("0x1.555556p-1"),
    hf64!("0x1.51d07ep-1"),
    hf64!("0x1.4e5e0ap-1"),
    hf64!("0x1.4afd6ap-1"),
    hf64!("0x1.47ae14p-1"),
    hf64!("0x1.446f86p-1"),
    hf64!("0x1.414142p-1"),
    hf64!("0x1.3e22ccp-1"),
    hf64!("0x1.3b13b2p-1"),
    hf64!("0x1.381382p-1"),
    hf64!("0x1.3521dp-1"),
    hf64!("0x1.323e34p-1"),
    hf64!("0x1.2f684cp-1"),
    hf64!("0x1.2c9fb4p-1"),
    hf64!("0x1.29e412p-1"),
    hf64!("0x1.27350cp-1"),
    hf64!("0x1.24924ap-1"),
    hf64!("0x1.21fb78p-1"),
    hf64!("0x1.1f7048p-1"),
    hf64!("0x1.1cf06ap-1"),
    hf64!("0x1.1a7b96p-1"),
    hf64!("0x1.181182p-1"),
    hf64!("0x1.15b1e6p-1"),
    hf64!("0x1.135c82p-1"),
    hf64!("0x1.111112p-1"),
    hf64!("0x1.0ecf56p-1"),
    hf64!("0x1.0c9714p-1"),
    hf64!("0x1.0a681p-1"),
    hf64!("0x1.08421p-1"),
    hf64!("0x1.0624dep-1"),
    hf64!("0x1.041042p-1"),
    hf64!("0x1.020408p-1"),
    hf64!("0x1p-1"),
];

/// The logarithm of the reciprocal is biased by `0x1.dp-45` so `log2p1_fast(x) - log2p1(x) < 0`.
static LIX: [f64; 65] = [
    hf64!("0x1.dp-45"),
    hf64!("-0x1.6e7966ead50c5p-6"),
    hf64!("-0x1.6bad2043a6a91p-5"),
    hf64!("-0x1.0eb392fe78f6fp-4"),
    hf64!("-0x1.663f6e3b3bd32p-4"),
    hf64!("-0x1.bc841cd433853p-4"),
    hf64!("-0x1.08c587b8a7d19p-3"),
    hf64!("-0x1.32aea1c2dd96p-3"),
    hf64!("-0x1.5c01a22e687e4p-3"),
    hf64!("-0x1.84c2be74443dap-3"),
    hf64!("-0x1.acf5de2afbd5ap-3"),
    hf64!("-0x1.d49ee012d2a36p-3"),
    hf64!("-0x1.fbc16a1ed1966p-3"),
    hf64!("-0x1.11307dc445c4cp-2"),
    hf64!("-0x1.2440796db6523p-2"),
    hf64!("-0x1.37124a7b0e1dap-2"),
    hf64!("-0x1.49a7834b7d089p-2"),
    hf64!("-0x1.5c01a2e712f36p-2"),
    hf64!("-0x1.6e22207523bcdp-2"),
    hf64!("-0x1.800a59ccb4b43p-2"),
    hf64!("-0x1.91bba6c447a2fp-2"),
    hf64!("-0x1.a3375ec336f01p-2"),
    hf64!("-0x1.b47ebfcfdd0dap-2"),
    hf64!("-0x1.c592fb2eea99p-2"),
    hf64!("-0x1.d6753b20857bp-2"),
    hf64!("-0x1.e726a9208b01ep-2"),
    hf64!("-0x1.f7a8543486f32p-2"),
    hf64!("-0x1.03fda781da376p-1"),
    hf64!("-0x1.0c104f268ec39p-1"),
    hf64!("-0x1.140c9fb5a8dafp-1"),
    hf64!("-0x1.1bf31371c6a2p-1"),
    hf64!("-0x1.23c41b2f88f63p-1"),
    hf64!("-0x1.2b803302a31a2p-1"),
    hf64!("-0x1.3327c82828c7dp-1"),
    hf64!("-0x1.3abb40a7ec0afp-1"),
    hf64!("-0x1.423b07f511315p-1"),
    hf64!("-0x1.49a785d1d0f4ap-1"),
    hf64!("-0x1.51011934bf518p-1"),
    hf64!("-0x1.584820b2f56a4p-1"),
    hf64!("-0x1.5f7cfece7619p-1"),
    hf64!("-0x1.66a00716cedc6p-1"),
    hf64!("-0x1.6db194ce2d40ap-1"),
    hf64!("-0x1.74b1fcac361d3p-1"),
    hf64!("-0x1.7ba1911bb9cf6p-1"),
    hf64!("-0x1.82809cff91ccap-1"),
    hf64!("-0x1.894f76c358469p-1"),
    hf64!("-0x1.900e62e869cdfp-1"),
    hf64!("-0x1.96bdabfeb6a28p-1"),
    hf64!("-0x1.9d5d9dab023d9p-1"),
    hf64!("-0x1.a3ee7f670bf3cp-1"),
    hf64!("-0x1.aa708efbac12bp-1"),
    hf64!("-0x1.b0e414a155bfcp-1"),
    hf64!("-0x1.b74949237d9f7p-1"),
    hf64!("-0x1.bda06f68b3e6ep-1"),
    hf64!("-0x1.c3e9ca1704a0fp-1"),
    hf64!("-0x1.ca258b4fc9ea1p-1"),
    hf64!("-0x1.d053f44c0c9e7p-1"),
    hf64!("-0x1.d675400a8d4b1p-1"),
    hf64!("-0x1.dc899d687d98ep-1"),
    hf64!("-0x1.e29144ae898b8p-1"),
    hf64!("-0x1.e88c6ca77b00ep-1"),
    hf64!("-0x1.ee7b44ce9bdc6p-1"),
    hf64!("-0x1.f45e05f15ca47p-1"),
    hf64!("-0x1.fa34e145a695p-1"),
    hf64!("-0x1.ffffffffffe3p-1"),
];

static B: [f64; 3] = [
    hf64!("0x1.7154765bab3edp+0"),
    hf64!("-0x1.71574d692522fp-1"),
    hf64!("0x1.ec60b55c8f05p-2"),
];

static C: [f64; 6] = [
    hf64!("0x1.71547652b8314p+0"),
    hf64!("-0x1.71547652b7f67p-1"),
    hf64!("0x1.ec709db872c6dp-2"),
    hf64!("-0x1.715476b06590ep-2"),
    hf64!("0x1.277c72c128c69p-2"),
    hf64!("-0x1.ec4ff30af701bp-3"),
];

static G: [f64; 8] = [
    hf64!("0x1.4ae0bf64f73a1p-26"),
    hf64!("-0x1.71547652b82fap-1"),
    hf64!("0x1.ec709dc3bd7dep-2"),
    hf64!("-0x1.71547652e6faap-2"),
    hf64!("0x1.2776c0ff5c16ep-2"),
    hf64!("-0x1.ec70942dfbb5bp-3"),
    hf64!("0x1.a673c6b6e2fa3p-3"),
    hf64!("-0x1.71b0db8113c46p-3"),
];

#[cfg(test)]
mod tests {
    use super::super::super::ldexpf;
    use super::super::super::support::{CastFrom, Float};
    use super::log2p1f;

    #[test]
    fn known_results() {
        for (x, expected) in [
            (f32::ZERO, f32::ZERO),
            (f32::NEG_ZERO, f32::NEG_ZERO),
            (f32::ONE, f32::ONE),
            (f32::NEG_ONE, f32::NEG_INFINITY),
            (f32::INFINITY, f32::INFINITY),
            (f32::MAX, 128.0),
            (f32::from_bits(1), f32::from_bits(1)),
            (f32::from_bits(2), f32::from_bits(3)),
            (-f32::from_bits(1), -f32::from_bits(1)),
            (-f32::from_bits(2), -f32::from_bits(3)),
            (f32::MIN_POSITIVE, hf32!("0x1.715476p-126")),
            (-f32::MIN_POSITIVE, -hf32!("0x1.715476p-126")),
            (0.5, hf32!("0x1.2b8034p-1")),
            (2.0, hf32!("0x1.95c01ap+0")),
            (hf32!("0x1.7a13c6p+30"), hf32!("0x1.e90026p+4")),
            (hf32!("-0x1.da285cp-5"), hf32!("-0x1.60549p-4")),
        ] {
            assert_biteq!(log2p1f(x), expected, "x = {x:?}");
        }

        for x in [
            f32::NEG_INFINITY,
            f32::MIN,
            -2.0,
            f32::from_bits(0xbf80_0001),
            f32::NAN,
            f32::NEG_NAN,
            f32::SNAN,
            f32::NEG_SNAN,
        ] {
            assert!(log2p1f(x).is_nan(), "x = {x:?}");
        }

        for n in 1..=24 {
            let expected = f32::cast_from(n);
            let positive = ldexpf(1.0, n) - 1.0;
            let negative = ldexpf(1.0, -n) - 1.0;
            assert_biteq!(log2p1f(positive), expected, "x = {positive:?}");
            assert_biteq!(log2p1f(negative), -expected, "x = {negative:?}");
        }
    }
}
