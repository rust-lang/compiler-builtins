// Copyright (c) 2018-2026, Arm Limited.
// SPDX-License-Identifier: MIT
/*
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
// Scalar algorithm from ARM optimized-routines math/pow.c.

mod exponential;
mod logarithm;
mod log_data;

const SIGN_BIAS: u32 = 0x800 << 7;
const ONE: u64 = 0x3ff0000000000000;
const INF: u64 = 0x7ff0000000000000;

/// Returns `x` to the power of `y` (f64).
#[cfg_attr(assert_no_panic, no_panic::no_panic)]
pub fn pow(x: f64, y: f64) -> f64 {
    let mut ix = x.to_bits();
    let iy = y.to_bits();
    let mut sign = 0;
    let topx = (ix >> 52) as u32;
    let topy = ((iy >> 52) as u32) & 0x7ff;
    if topx.wrapping_sub(1) >= 0x7fe || topy.wrapping_sub(0x3be) >= 0x80 {
        if let Some(result) = exceptional(x, y, &mut ix, &mut sign) {
            return result;
        }
    }
    let (hi, lo) = logarithm::evaluate(ix);
    // Splitting avoids a software FMA on targets without a fused instruction.
    let yhi = f64::from_bits(iy & (!0u64 << 27));
    let ylo = y - yhi;
    let lhi = f64::from_bits(hi.to_bits() & (!0u64 << 27));
    let llo = hi - lhi + lo;
    exponential::evaluate(yhi * lhi, ylo * lhi + y * llo, sign)
}

fn exceptional(x: f64, y: f64, ix: &mut u64, sign: &mut u32) -> Option<f64> {
    let iy = y.to_bits();
    let ax2 = ix.wrapping_mul(2);
    let ay2 = iy.wrapping_mul(2);
    if zero_inf_nan(iy) {
        if ay2 == 0 || *ix == ONE {
            return Some(1.0);
        }
        if ax2 > INF * 2 || ay2 > INF * 2 {
            return Some(x + y);
        }
        if ax2 == ONE * 2 {
            return Some(1.0);
        }
        return Some(if (ax2 < ONE * 2) == (iy >> 63 == 0) {
            0.0
        } else {
            y * y
        });
    }
    if zero_inf_nan(*ix) {
        let mut x2 = x * x;
        if *ix >> 63 != 0 && checkint(iy) == 1 {
            x2 = -x2;
        }
        return Some(if iy >> 63 != 0 { 1.0 / x2 } else { x2 });
    }
    if *ix >> 63 != 0 {
        match checkint(iy) {
            0 => return Some(f64::NAN),
            1 => *sign = SIGN_BIAS,
            _ => {}
        }
        *ix &= 0x7fffffffffffffff;
    }
    let topy = ((iy >> 52) as u32) & 0x7ff;
    if topy.wrapping_sub(0x3be) >= 0x80 {
        if *ix == ONE || topy < 0x3be {
            return Some(1.0);
        }
        return Some(if (*ix > ONE) == (iy >> 63 == 0) {
            f64::INFINITY
        } else {
            0.0
        });
    }
    if *ix >> 52 == 0 {
        *ix = ((x * 4503599627370496.0).to_bits() & 0x7fffffffffffffff).wrapping_sub(52u64 << 52);
    }
    None
}

fn zero_inf_nan(bits: u64) -> bool {
    bits.wrapping_mul(2).wrapping_sub(1) >= (INF * 2).wrapping_sub(1)
}

fn checkint(bits: u64) -> u32 {
    let e = (bits >> 52) & 0x7ff;
    if e < 0x3ff {
        return 0;
    }
    if e > 0x3ff + 52 {
        return 2;
    }
    let shift = 0x3ff + 52 - e;
    if bits & ((1u64 << shift) - 1) != 0 {
        0
    } else if bits & (1u64 << shift) != 0 {
        1
    } else {
        2
    }
}
