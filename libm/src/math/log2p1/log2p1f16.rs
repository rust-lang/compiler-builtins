/* SPDX-License-Identifier: MIT */
/* origin: core-math/src/binary16/log2p1/log2p1f16.c
 * Copyright (c) 2025 Paul Zimmermann.
 * Ported to Rust in 2026, malezjaa
 * Approximate CORE-MATH commit: 8ea8ea35c518d06b72de4cf1d528e7a2d16f104c
 */

use super::super::support::{CastFrom, CastInto, Float, NarrowFloat, WideFloat, cold_path};

/// Base-2 logarithm of x plus one (f16).
///
/// Calculate `log2(x + 1)`, that is, the base-2 logarithm of `x + 1`.
#[cfg_attr(assert_no_panic, no_panic::no_panic)]
#[cfg(f16_enabled)]
pub fn log2p1f16(x: f16) -> f16 {
    cr_log2p1f16(x)
}

fn cr_log2p1f16(x: f16) -> f16 {
    let mut v = x.widen();
    let mut u = v.to_bits();
    let au = u & (u32::MAX >> 1);

    if u >= 0xbf800000 {
        // x <= -1
        if u == 0xbf800000 {
            // x = -1
            // overflow
            return x / 0.0;
        }
        return (x - x) / 0.0; // nan
    }

    // NaN or +Inf
    if (au >> 23) == 0xff {
        cold_path();
        return x + x;
    }

    // now x > -1

    // 1.0+x is always exactly representable as float, except when
    // x > 0 and x is an odd multiple of 2^-24, which implies 0 < x < 2^-13

    if au < 0x3d000000 {
        // |x| < 2^-5
        let res = log2p1_small(v);

        // deal with exceptional cases
        if u == 0xb99ec000 {
            // x = -0x1.3d8p-12
            return hf32!("-0x1.ca2002p-12").narrow();
        }
        if u == 0xbaa0a000 {
            // x = -0x1.414p-10
            return hf32!("-0x1.cfbffep-10").narrow();
        }
        if u == 0xbcb9e000 {
            // x = -0x1.73cp-6
            return hf32!("-0x1.0f4002p-5").narrow();
        }
        if u == 0xb8424000 {
            // x = -0x1.848p-15
            return hf32!("-0x1.184002p-14").narrow();
        }
        if u == 0x38966000 {
            // x = 0x1.2ccp-14
            return hf32!("0x1.b1dffep-14").narrow();
        }
        if u == 0x3968c000 {
            // x = 0x1.d18p-13
            return hf32!("0x1.4fc002p-12").narrow();
        }
        if u == 0x3cbfe000 {
            // x = 0x1.7fcp-6
            return hf32!("0x1.11a002p-5").narrow();
        }
        if u == 0x39f62000 {
            // x = 0x1.ec4p-12
            return hf32!("0x1.62fffep-11").narrow();
        }
        if u == 0x3bff6000 {
            // x = 0x1.fecp-8
            return hf32!("0x1.6f0002p-7").narrow();
        }

        if au == 0 {
            return x;
        }
        return res.narrow();
    }

    v += 1.0; // exact
    u = v.to_bits();

    // deal with exact cases: -1+1/2^12, ..., -1/2, 1, 3, 7, ..., 2^16-1
    if (u << 9) == 0 && u >= 0x3a000000 {
        return (i32::cast_from(u >> 23) - 127).cast();
    }

    // for x = -0x1.ffcp-1, which is the smallest x > 1, we have v.f = 0x1p-11;
    // for x = 0x1.ffcp+15, which is the largest possible value, we have
    // v.f = 0x1.ffc2p+15.
    let mut i = usize::cast_from(u >> 20); // 928 <= i <= 1143
    i -= 928;
    let (r, log_r) = i!(R, i);
    let z = r.fma(v, -1.0); // exact
    (log_r + log2p1_small(z)).narrow()
}

const INV_LOG2: f32 = hf32!("0x1.715476p+0"); // 1/log(2)

/// Approximates `log2(1+x)` for `x` small.
#[inline]
fn log2p1_small(x: f32) -> f32 {
    let c3 = x.fma(hf32!("-0x1p-2"), hf32!("0x1.555556p-2"));
    let c2 = x.fma(c3, -0.5);

    x.fma(x * c2, x) * INV_LOG2
}

/// For `0 <= i < 224`, `R[i]` stores `{r,logr}` where `r` approximates `1/x`
/// and `logr` approximates `-log2(r)`, where i is related to the binary32
/// encoding of `x`. Table generated in CORE-MATH repo.
#[rustfmt::skip]
static R: [(f32, f32); 224] = [
    (hf32!("0x1.e218p+10"), hf32!("-0x1.5d38bp+3")),
    (hf32!("0x1.af58p+10"), hf32!("-0x1.581612p+3")),
    (hf32!("0x1.864p+10"), hf32!("-0x1.5376d2p+3")),
    (hf32!("0x1.6448p+10"), hf32!("-0x1.4f428ap+3")),
    (hf32!("0x1.47c8p+10"), hf32!("-0x1.4b692ep+3")),
    (hf32!("0x1.2f8p+10"), hf32!("-0x1.47db92p+3")),
    (hf32!("0x1.1a9p+10"), hf32!("-0x1.448ec2p+3")),
    (hf32!("0x1.085p+10"), hf32!("-0x1.4179a8p+3")),
    (hf32!("0x1.e218p+9"), hf32!("-0x1.3d38bp+3")),
    (hf32!("0x1.af58p+9"), hf32!("-0x1.381612p+3")),
    (hf32!("0x1.864p+9"), hf32!("-0x1.3376d2p+3")),
    (hf32!("0x1.6448p+9"), hf32!("-0x1.2f428ap+3")),
    (hf32!("0x1.47c8p+9"), hf32!("-0x1.2b692ep+3")),
    (hf32!("0x1.2f8p+9"), hf32!("-0x1.27db92p+3")),
    (hf32!("0x1.1a9p+9"), hf32!("-0x1.248ec2p+3")),
    (hf32!("0x1.085p+9"), hf32!("-0x1.2179a8p+3")),
    (hf32!("0x1.e218p+8"), hf32!("-0x1.1d38bp+3")),
    (hf32!("0x1.af58p+8"), hf32!("-0x1.181612p+3")),
    (hf32!("0x1.864p+8"), hf32!("-0x1.1376d2p+3")),
    (hf32!("0x1.6448p+8"), hf32!("-0x1.0f428ap+3")),
    (hf32!("0x1.47c8p+8"), hf32!("-0x1.0b692ep+3")),
    (hf32!("0x1.2f8p+8"), hf32!("-0x1.07db92p+3")),
    (hf32!("0x1.1a9p+8"), hf32!("-0x1.048ec2p+3")),
    (hf32!("0x1.085p+8"), hf32!("-0x1.0179a8p+3")),
    (hf32!("0x1.e218p+7"), hf32!("-0x1.fa716p+2")),
    (hf32!("0x1.af58p+7"), hf32!("-0x1.f02c24p+2")),
    (hf32!("0x1.864p+7"), hf32!("-0x1.e6eda6p+2")),
    (hf32!("0x1.6448p+7"), hf32!("-0x1.de8516p+2")),
    (hf32!("0x1.47c8p+7"), hf32!("-0x1.d6d25cp+2")),
    (hf32!("0x1.2f8p+7"), hf32!("-0x1.cfb722p+2")),
    (hf32!("0x1.1a9p+7"), hf32!("-0x1.c91d82p+2")),
    (hf32!("0x1.085p+7"), hf32!("-0x1.c2f35p+2")),
    (hf32!("0x1.e218p+6"), hf32!("-0x1.ba716p+2")),
    (hf32!("0x1.af58p+6"), hf32!("-0x1.b02c24p+2")),
    (hf32!("0x1.864p+6"), hf32!("-0x1.a6eda6p+2")),
    (hf32!("0x1.6448p+6"), hf32!("-0x1.9e8516p+2")),
    (hf32!("0x1.47c8p+6"), hf32!("-0x1.96d25cp+2")),
    (hf32!("0x1.2f8p+6"), hf32!("-0x1.8fb722p+2")),
    (hf32!("0x1.1a9p+6"), hf32!("-0x1.891d82p+2")),
    (hf32!("0x1.085p+6"), hf32!("-0x1.82f35p+2")),
    (hf32!("0x1.e218p+5"), hf32!("-0x1.7a716p+2")),
    (hf32!("0x1.af58p+5"), hf32!("-0x1.702c24p+2")),
    (hf32!("0x1.864p+5"), hf32!("-0x1.66eda6p+2")),
    (hf32!("0x1.6448p+5"), hf32!("-0x1.5e8516p+2")),
    (hf32!("0x1.47c8p+5"), hf32!("-0x1.56d25cp+2")),
    (hf32!("0x1.2f8p+5"), hf32!("-0x1.4fb722p+2")),
    (hf32!("0x1.1a9p+5"), hf32!("-0x1.491d82p+2")),
    (hf32!("0x1.085p+5"), hf32!("-0x1.42f35p+2")),
    (hf32!("0x1.e218p+4"), hf32!("-0x1.3a716p+2")),
    (hf32!("0x1.af58p+4"), hf32!("-0x1.302c24p+2")),
    (hf32!("0x1.864p+4"), hf32!("-0x1.26eda6p+2")),
    (hf32!("0x1.6448p+4"), hf32!("-0x1.1e8516p+2")),
    (hf32!("0x1.47c8p+4"), hf32!("-0x1.16d25cp+2")),
    (hf32!("0x1.2f8p+4"), hf32!("-0x1.0fb722p+2")),
    (hf32!("0x1.1a9p+4"), hf32!("-0x1.091d82p+2")),
    (hf32!("0x1.085p+4"), hf32!("-0x1.02f35p+2")),
    (hf32!("0x1.e218p+3"), hf32!("-0x1.f4e2cp+1")),
    (hf32!("0x1.af58p+3"), hf32!("-0x1.e05848p+1")),
    (hf32!("0x1.864p+3"), hf32!("-0x1.cddb4cp+1")),
    (hf32!("0x1.6448p+3"), hf32!("-0x1.bd0a2ap+1")),
    (hf32!("0x1.47c8p+3"), hf32!("-0x1.ada4bap+1")),
    (hf32!("0x1.2f8p+3"), hf32!("-0x1.9f6e46p+1")),
    (hf32!("0x1.1a9p+3"), hf32!("-0x1.923b06p+1")),
    (hf32!("0x1.085p+3"), hf32!("-0x1.85e6a2p+1")),
    (hf32!("0x1.e218p+2"), hf32!("-0x1.74e2cp+1")),
    (hf32!("0x1.af58p+2"), hf32!("-0x1.605848p+1")),
    (hf32!("0x1.864p+2"), hf32!("-0x1.4ddb4cp+1")),
    (hf32!("0x1.6448p+2"), hf32!("-0x1.3d0a2ap+1")),
    (hf32!("0x1.47c8p+2"), hf32!("-0x1.2da4bap+1")),
    (hf32!("0x1.2f8p+2"), hf32!("-0x1.1f6e46p+1")),
    (hf32!("0x1.1a9p+2"), hf32!("-0x1.123b06p+1")),
    (hf32!("0x1.085p+2"), hf32!("-0x1.05e6a2p+1")),
    (hf32!("0x1.e218p+1"), hf32!("-0x1.e9c58p+0")),
    (hf32!("0x1.af58p+1"), hf32!("-0x1.c0b092p+0")),
    (hf32!("0x1.864p+1"), hf32!("-0x1.9bb696p+0")),
    (hf32!("0x1.6448p+1"), hf32!("-0x1.7a1456p+0")),
    (hf32!("0x1.47c8p+1"), hf32!("-0x1.5b4974p+0")),
    (hf32!("0x1.2f8p+1"), hf32!("-0x1.3edc8ap+0")),
    (hf32!("0x1.1a9p+1"), hf32!("-0x1.24760cp+0")),
    (hf32!("0x1.085p+1"), hf32!("-0x1.0bcd44p+0")),
    (hf32!("0x1.e218p+0"), hf32!("-0x1.d38b02p-1")),
    (hf32!("0x1.af58p+0"), hf32!("-0x1.816124p-1")),
    (hf32!("0x1.864p+0"), hf32!("-0x1.376d2ep-1")),
    (hf32!("0x1.6448p+0"), hf32!("-0x1.e85154p-2")),
    (hf32!("0x1.47c8p+0"), hf32!("-0x1.6d25cep-2")),
    (hf32!("0x1.2f8p+0"), hf32!("-0x1.f6e456p-3")),
    (hf32!("0x1.1a9p+0"), hf32!("-0x1.23b05ap-3")),
    (hf32!("0x1.085p+0"), hf32!("-0x1.79a87cp-5")),
    (hf32!("0x1.e218p-1"), hf32!("0x1.63a7f2p-4")),
    (hf32!("0x1.af58p-1"), hf32!("0x1.fa7b7p-3")),
    (hf32!("0x1.864p-1"), hf32!("0x1.9125a6p-2")),
    (hf32!("0x1.6448p-1"), hf32!("0x1.0bd756p-1")),
    (hf32!("0x1.47c8p-1"), hf32!("0x1.496d1ap-1")),
    (hf32!("0x1.2f8p-1"), hf32!("0x1.8246eap-1")),
    (hf32!("0x1.1a9p-1"), hf32!("0x1.b713eap-1")),
    (hf32!("0x1.085p-1"), hf32!("0x1.e86578p-1")),
    (hf32!("0x1.e218p-2"), hf32!("0x1.163a8p+0")),
    (hf32!("0x1.af58p-2"), hf32!("0x1.3f4f6cp+0")),
    (hf32!("0x1.864p-2"), hf32!("0x1.644968p+0")),
    (hf32!("0x1.6448p-2"), hf32!("0x1.85ebaap+0")),
    (hf32!("0x1.47c8p-2"), hf32!("0x1.a4b68cp+0")),
    (hf32!("0x1.2f8p-2"), hf32!("0x1.c12376p+0")),
    (hf32!("0x1.1a9p-2"), hf32!("0x1.db89f4p+0")),
    (hf32!("0x1.085p-2"), hf32!("0x1.f432bcp+0")),
    (hf32!("0x1.e218p-3"), hf32!("0x1.0b1d4p+1")),
    (hf32!("0x1.af58p-3"), hf32!("0x1.1fa7b8p+1")),
    (hf32!("0x1.864p-3"), hf32!("0x1.3224b4p+1")),
    (hf32!("0x1.6448p-3"), hf32!("0x1.42f5d6p+1")),
    (hf32!("0x1.47c8p-3"), hf32!("0x1.525b46p+1")),
    (hf32!("0x1.2f8p-3"), hf32!("0x1.6091bap+1")),
    (hf32!("0x1.1a9p-3"), hf32!("0x1.6dc4fap+1")),
    (hf32!("0x1.085p-3"), hf32!("0x1.7a195ep+1")),
    (hf32!("0x1.e218p-4"), hf32!("0x1.8b1d4p+1")),
    (hf32!("0x1.af58p-4"), hf32!("0x1.9fa7b8p+1")),
    (hf32!("0x1.864p-4"), hf32!("0x1.b224b4p+1")),
    (hf32!("0x1.6448p-4"), hf32!("0x1.c2f5d6p+1")),
    (hf32!("0x1.47c8p-4"), hf32!("0x1.d25b46p+1")),
    (hf32!("0x1.2f8p-4"), hf32!("0x1.e091bap+1")),
    (hf32!("0x1.1a9p-4"), hf32!("0x1.edc4fap+1")),
    (hf32!("0x1.085p-4"), hf32!("0x1.fa195ep+1")),
    (hf32!("0x1.e218p-5"), hf32!("0x1.058eap+2")),
    (hf32!("0x1.af58p-5"), hf32!("0x1.0fd3dcp+2")),
    (hf32!("0x1.864p-5"), hf32!("0x1.19125ap+2")),
    (hf32!("0x1.6448p-5"), hf32!("0x1.217aeap+2")),
    (hf32!("0x1.47c8p-5"), hf32!("0x1.292da4p+2")),
    (hf32!("0x1.2f8p-5"), hf32!("0x1.3048dep+2")),
    (hf32!("0x1.1a9p-5"), hf32!("0x1.36e27ep+2")),
    (hf32!("0x1.085p-5"), hf32!("0x1.3d0cbp+2")),
    (hf32!("0x1.e218p-6"), hf32!("0x1.458eap+2")),
    (hf32!("0x1.af58p-6"), hf32!("0x1.4fd3dcp+2")),
    (hf32!("0x1.864p-6"), hf32!("0x1.59125ap+2")),
    (hf32!("0x1.6448p-6"), hf32!("0x1.617aeap+2")),
    (hf32!("0x1.47c8p-6"), hf32!("0x1.692da4p+2")),
    (hf32!("0x1.2f8p-6"), hf32!("0x1.7048dep+2")),
    (hf32!("0x1.1a9p-6"), hf32!("0x1.76e27ep+2")),
    (hf32!("0x1.085p-6"), hf32!("0x1.7d0cbp+2")),
    (hf32!("0x1.e218p-7"), hf32!("0x1.858eap+2")),
    (hf32!("0x1.af58p-7"), hf32!("0x1.8fd3dcp+2")),
    (hf32!("0x1.864p-7"), hf32!("0x1.99125ap+2")),
    (hf32!("0x1.6448p-7"), hf32!("0x1.a17aeap+2")),
    (hf32!("0x1.47c8p-7"), hf32!("0x1.a92da4p+2")),
    (hf32!("0x1.2f8p-7"), hf32!("0x1.b048dep+2")),
    (hf32!("0x1.1a9p-7"), hf32!("0x1.b6e27ep+2")),
    (hf32!("0x1.085p-7"), hf32!("0x1.bd0cbp+2")),
    (hf32!("0x1.e218p-8"), hf32!("0x1.c58eap+2")),
    (hf32!("0x1.af58p-8"), hf32!("0x1.cfd3dcp+2")),
    (hf32!("0x1.864p-8"), hf32!("0x1.d9125ap+2")),
    (hf32!("0x1.6448p-8"), hf32!("0x1.e17aeap+2")),
    (hf32!("0x1.47c8p-8"), hf32!("0x1.e92da4p+2")),
    (hf32!("0x1.2f8p-8"), hf32!("0x1.f048dep+2")),
    (hf32!("0x1.1a9p-8"), hf32!("0x1.f6e27ep+2")),
    (hf32!("0x1.085p-8"), hf32!("0x1.fd0cbp+2")),
    (hf32!("0x1.e218p-9"), hf32!("0x1.02c75p+3")),
    (hf32!("0x1.af58p-9"), hf32!("0x1.07e9eep+3")),
    (hf32!("0x1.864p-9"), hf32!("0x1.0c892ep+3")),
    (hf32!("0x1.6448p-9"), hf32!("0x1.10bd76p+3")),
    (hf32!("0x1.47c8p-9"), hf32!("0x1.1496d2p+3")),
    (hf32!("0x1.2f8p-9"), hf32!("0x1.18246ep+3")),
    (hf32!("0x1.1a9p-9"), hf32!("0x1.1b713ep+3")),
    (hf32!("0x1.085p-9"), hf32!("0x1.1e8658p+3")),
    (hf32!("0x1.e218p-10"), hf32!("0x1.22c75p+3")),
    (hf32!("0x1.af58p-10"), hf32!("0x1.27e9eep+3")),
    (hf32!("0x1.864p-10"), hf32!("0x1.2c892ep+3")),
    (hf32!("0x1.6448p-10"), hf32!("0x1.30bd76p+3")),
    (hf32!("0x1.47c8p-10"), hf32!("0x1.3496d2p+3")),
    (hf32!("0x1.2f8p-10"), hf32!("0x1.38246ep+3")),
    (hf32!("0x1.1a9p-10"), hf32!("0x1.3b713ep+3")),
    (hf32!("0x1.085p-10"), hf32!("0x1.3e8658p+3")),
    (hf32!("0x1.e218p-11"), hf32!("0x1.42c75p+3")),
    (hf32!("0x1.af58p-11"), hf32!("0x1.47e9eep+3")),
    (hf32!("0x1.864p-11"), hf32!("0x1.4c892ep+3")),
    (hf32!("0x1.6448p-11"), hf32!("0x1.50bd76p+3")),
    (hf32!("0x1.47c8p-11"), hf32!("0x1.5496d2p+3")),
    (hf32!("0x1.2f8p-11"), hf32!("0x1.58246ep+3")),
    (hf32!("0x1.1a9p-11"), hf32!("0x1.5b713ep+3")),
    (hf32!("0x1.085p-11"), hf32!("0x1.5e8658p+3")),
    (hf32!("0x1.e218p-12"), hf32!("0x1.62c75p+3")),
    (hf32!("0x1.af58p-12"), hf32!("0x1.67e9eep+3")),
    (hf32!("0x1.864p-12"), hf32!("0x1.6c892ep+3")),
    (hf32!("0x1.6448p-12"), hf32!("0x1.70bd76p+3")),
    (hf32!("0x1.47c8p-12"), hf32!("0x1.7496d2p+3")),
    (hf32!("0x1.2f8p-12"), hf32!("0x1.78246ep+3")),
    (hf32!("0x1.1a9p-12"), hf32!("0x1.7b713ep+3")),
    (hf32!("0x1.085p-12"), hf32!("0x1.7e8658p+3")),
    (hf32!("0x1.e218p-13"), hf32!("0x1.82c75p+3")),
    (hf32!("0x1.af58p-13"), hf32!("0x1.87e9eep+3")),
    (hf32!("0x1.864p-13"), hf32!("0x1.8c892ep+3")),
    (hf32!("0x1.6448p-13"), hf32!("0x1.90bd76p+3")),
    (hf32!("0x1.47c8p-13"), hf32!("0x1.9496d2p+3")),
    (hf32!("0x1.2f8p-13"), hf32!("0x1.98246ep+3")),
    (hf32!("0x1.1a9p-13"), hf32!("0x1.9b713ep+3")),
    (hf32!("0x1.085p-13"), hf32!("0x1.9e8658p+3")),
    (hf32!("0x1.e218p-14"), hf32!("0x1.a2c75p+3")),
    (hf32!("0x1.af58p-14"), hf32!("0x1.a7e9eep+3")),
    (hf32!("0x1.864p-14"), hf32!("0x1.ac892ep+3")),
    (hf32!("0x1.6448p-14"), hf32!("0x1.b0bd76p+3")),
    (hf32!("0x1.47c8p-14"), hf32!("0x1.b496d2p+3")),
    (hf32!("0x1.2f8p-14"), hf32!("0x1.b8246ep+3")),
    (hf32!("0x1.1a9p-14"), hf32!("0x1.bb713ep+3")),
    (hf32!("0x1.085p-14"), hf32!("0x1.be8658p+3")),
    (hf32!("0x1.e218p-15"), hf32!("0x1.c2c75p+3")),
    (hf32!("0x1.af58p-15"), hf32!("0x1.c7e9eep+3")),
    (hf32!("0x1.864p-15"), hf32!("0x1.cc892ep+3")),
    (hf32!("0x1.6448p-15"), hf32!("0x1.d0bd76p+3")),
    (hf32!("0x1.47c8p-15"), hf32!("0x1.d496d2p+3")),
    (hf32!("0x1.2f8p-15"), hf32!("0x1.d8246ep+3")),
    (hf32!("0x1.1a9p-15"), hf32!("0x1.db713ep+3")),
    (hf32!("0x1.085p-15"), hf32!("0x1.de8658p+3")),
    (hf32!("0x1.e218p-16"), hf32!("0x1.e2c75p+3")),
    (hf32!("0x1.af58p-16"), hf32!("0x1.e7e9eep+3")),
    (hf32!("0x1.864p-16"), hf32!("0x1.ec892ep+3")),
    (hf32!("0x1.6448p-16"), hf32!("0x1.f0bd76p+3")),
    (hf32!("0x1.47c8p-16"), hf32!("0x1.f496d2p+3")),
    (hf32!("0x1.2f8p-16"), hf32!("0x1.f8246ep+3")),
    (hf32!("0x1.1a9p-16"), hf32!("0x1.fb713ep+3")),
    (hf32!("0x1.085p-16"), hf32!("0x1.fe8658p+3")),
    (hf32!("0x1.e218p-17"), hf32!("0x1.0163a8p+4")),
    (hf32!("0x1.af58p-17"), hf32!("0x1.03f4f6p+4")),
    (hf32!("0x1.864p-17"), hf32!("0x1.064496p+4")),
    (hf32!("0x1.6448p-17"), hf32!("0x1.085ebap+4")),
    (hf32!("0x1.47c8p-17"), hf32!("0x1.0a4b68p+4")),
    (hf32!("0x1.2f8p-17"), hf32!("0x1.0c1238p+4")),
    (hf32!("0x1.1a9p-17"), hf32!("0x1.0db8ap+4")),
    (hf32!("0x1.085p-17"), hf32!("0x1.0f432cp+4")),
];

#[cfg(test)]
mod tests {
    use super::super::super::support::Float;
    use super::log2p1f16;

    #[test]
    fn known_results() {
        for (x, expected) in [
            (f16::ZERO, f16::ZERO),
            (f16::NEG_ZERO, f16::NEG_ZERO),
            (f16::ONE, f16::ONE),
            (f16::NEG_ONE, f16::NEG_INFINITY),
            (f16::INFINITY, f16::INFINITY),
            (f16::MAX, 16.0),
            (f16::from_bits(1), f16::from_bits(1)),
            (f16::from_bits(2), f16::from_bits(3)),
            (-f16::from_bits(1), -f16::from_bits(1)),
            (-f16::from_bits(2), -f16::from_bits(3)),
            (hf16!("-0x1.3d8p-12"), hf16!("-0x1.ca4p-12")),
            (hf16!("-0x1.414p-10"), hf16!("-0x1.cfcp-10")),
            (hf16!("-0x1.73cp-6"), hf16!("-0x1.0f4p-5")),
            (hf16!("-0x1.848p-15"), hf16!("-0x1.184p-14")),
            (hf16!("0x1.2ccp-14"), hf16!("0x1.b1cp-14")),
            (hf16!("0x1.d18p-13"), hf16!("0x1.4fcp-12")),
            (hf16!("0x1.7fcp-6"), hf16!("0x1.11cp-5")),
            (hf16!("0x1.ec4p-12"), hf16!("0x1.63p-11")),
            (hf16!("0x1.fecp-8"), hf16!("0x1.6fp-7")),
        ] {
            assert_biteq!(log2p1f16(x), expected, "x = {x:?}");
        }

        for x in [
            f16::NEG_INFINITY,
            f16::MIN,
            -2.0,
            f16::from_bits(0xbc01),
            f16::NAN,
            f16::NEG_NAN,
            f16::SNAN,
            f16::NEG_SNAN,
        ] {
            assert!(log2p1f16(x).is_nan(), "x = {x:?}");
        }
    }
}
