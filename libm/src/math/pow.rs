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

use self::{EXP_POLY as C, LOG_POLY as A};

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
    let (hi, lo) = log_inline(ix);
    // Splitting avoids a software FMA on targets without a fused instruction.
    let yhi = f64::from_bits(iy & (!0u64 << 27));
    let ylo = y - yhi;
    let lhi = f64::from_bits(hi.to_bits() & (!0u64 << 27));
    let llo = hi - lhi + lo;
    exp_inline(yhi * lhi, ylo * lhi + y * llo, sign)
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
#[inline]
fn log_inline(ix: u64) -> (f64, f64) {
    let tmp = ix.wrapping_sub(0x3fe6955500000000);
    let index = ((tmp >> 45) & 127) as usize;
    let k = ((tmp as i64) >> 52) as f64;
    let iz = ix.wrapping_sub(tmp & (0xfffu64 << 52));
    let z = f64::from_bits(iz);
    let row = i!(LOG_TAB, index);
    let invc = f64::from_bits(row[0]);
    let logc = f64::from_bits(row[1]);
    let logctail = f64::from_bits(row[2]);
    // The split makes rhi, rlo, and rhi*rhi exact without a fused operation.
    let zhi = f64::from_bits(iz.wrapping_add(1u64 << 31) & (!0u64 << 32));
    let zlo = z - zhi;
    let rhi = zhi * invc - 1.0;
    let rlo = zlo * invc;
    let r = rhi + rlo;
    let t1 = k * LN2_HI + logc;
    let t2 = t1 + r;
    let lo1 = k * LN2_LO + logctail;
    let lo2 = t1 - t2 + r;
    let ar = A[0] * r;
    let ar2 = r * ar;
    let ar3 = r * ar2;
    let arhi = A[0] * rhi;
    let arhi2 = rhi * arhi;
    let hi = t2 + arhi2;
    let lo3 = rlo * (ar + arhi);
    let lo4 = t2 - hi + arhi2;
    let p = ar3 * (A[1] + r * A[2] + ar2 * (A[3] + r * A[4] + ar2 * (A[5] + r * A[6])));
    let lo = lo1 + lo2 + lo3 + lo4 + p;
    let result = hi + lo;
    (result, hi - result + lo)
}
#[inline]
fn exp_inline(x: f64, tail: f64, sign: u32) -> f64 {
    let abstop = ((x.to_bits() >> 52) as u32) & 0x7ff;
    if abstop < 0x3c9 {
        return if sign != 0 { -1.0 } else { 1.0 };
    }
    if abstop >= 0x409 {
        let value = if x.is_sign_negative() {
            0.0
        } else {
            f64::INFINITY
        };
        return if sign != 0 { -value } else { value };
    }
    let kd = x * INV_LN2_N + SHIFT;
    let ki = kd.to_bits();
    let kd = kd - SHIFT;
    let r = (x + kd * NEG_LN2_HI_N + kd * NEG_LN2_LO_N) + tail;
    let row = i!(EXP_TAB, (ki & 127) as usize);
    let top = ki.wrapping_add(u64::from(sign)).wrapping_shl(45);
    let sbits = row[1].wrapping_add(top);
    let r2 = r * r;
    let tmp = f64::from_bits(row[0]) + r + r2 * (C[0] + r * C[1]) + r2 * r2 * (C[2] + r * C[3]);
    if abstop >= 0x408 {
        specialcase(tmp, sbits, ki)
    } else {
        let scale = f64::from_bits(sbits);
        scale + scale * tmp
    }
}

fn specialcase(tmp: f64, sbits: u64, ki: u64) -> f64 {
    if ki & 0x80000000 == 0 {
        let scale = f64::from_bits(sbits.wrapping_sub(1009u64 << 52));
        return (scale + scale * tmp) * f64::from_bits(0x7f00000000000000);
    }
    let sbits = sbits.wrapping_add(1022u64 << 52);
    let scale = f64::from_bits(sbits);
    let mut y = scale + scale * tmp;
    if y.to_bits() & 0x7fff_ffff_ffff_ffff < 1.0f64.to_bits() {
        // Round before scaling to avoid double rounding in the subnormal range.
        let one = if y < 0.0 { -1.0 } else { 1.0 };
        let lo = scale - y + scale * tmp;
        let hi = one + y;
        let lo = one - hi + y + lo;
        y = (hi + lo) - one;
        if y == 0.0 {
            y = f64::from_bits(sbits & (1u64 << 63));
        }
    }
    f64::MIN_POSITIVE * y
}
// Binary64 data from ARM optimized-routines math/pow_log_data.c.
const LN2_HI: f64 = 0.6931471805598903;
const LN2_LO: f64 = 5.497923018708371e-14;
const LOG_POLY: [f64; 7] = [
    -0.5,
    -0.6666666666666679,
    0.5000000000000007,
    0.7999999995323976,
    -0.6666666663487739,
    -1.142909628459501,
    1.0000415263675542,
];
#[rustfmt::skip]
const LOG_TAB: [[u64; 3]; 128] = [
[0x3ff6a00000000000, 0xbfd62c82f2b9c800, 0x3cfab42428375680],
[0x3ff6800000000000, 0xbfd5d1bdbf580800, 0xbd1ca508d8e0f720],
[0x3ff6600000000000, 0xbfd5767717455800, 0xbd2362a4d5b6506d],
[0x3ff6400000000000, 0xbfd51aad872df800, 0xbce684e49eb067d5],
[0x3ff6200000000000, 0xbfd4be5f95777800, 0xbd041b6993293ee0],
[0x3ff6000000000000, 0xbfd4618bc21c6000, 0x3d13d82f484c84cc],
[0x3ff5e00000000000, 0xbfd404308686a800, 0x3cdc42f3ed820b3a],
[0x3ff5c00000000000, 0xbfd3a64c55694800, 0x3d20b1c686519460],
[0x3ff5a00000000000, 0xbfd347dd9a988000, 0x3d25594dd4c58092],
[0x3ff5800000000000, 0xbfd2e8e2bae12000, 0x3d267b1e99b72bd8],
[0x3ff5600000000000, 0xbfd2895a13de8800, 0x3d15ca14b6cfb03f],
[0x3ff5600000000000, 0xbfd2895a13de8800, 0x3d15ca14b6cfb03f],
[0x3ff5400000000000, 0xbfd22941fbcf7800, 0xbd165a242853da76],
[0x3ff5200000000000, 0xbfd1c898c1699800, 0xbd1fafbc68e75404],
[0x3ff5000000000000, 0xbfd1675cababa800, 0x3d1f1fc63382a8f0],
[0x3ff4e00000000000, 0xbfd1058bf9ae4800, 0xbd26a8c4fd055a66],
[0x3ff4c00000000000, 0xbfd0a324e2739000, 0xbd0c6bee7ef4030e],
[0x3ff4a00000000000, 0xbfd0402594b4d000, 0xbcf036b89ef42d7f],
[0x3ff4a00000000000, 0xbfd0402594b4d000, 0xbcf036b89ef42d7f],
[0x3ff4800000000000, 0xbfcfb9186d5e4000, 0x3d0d572aab993c87],
[0x3ff4600000000000, 0xbfcef0adcbdc6000, 0x3d2b26b79c86af24],
[0x3ff4400000000000, 0xbfce27076e2af000, 0xbd172f4f543fff10],
[0x3ff4200000000000, 0xbfcd5c216b4fc000, 0x3d21ba91bbca681b],
[0x3ff4000000000000, 0xbfcc8ff7c79aa000, 0x3d27794f689f8434],
[0x3ff4000000000000, 0xbfcc8ff7c79aa000, 0x3d27794f689f8434],
[0x3ff3e00000000000, 0xbfcbc286742d9000, 0x3d194eb0318bb78f],
[0x3ff3c00000000000, 0xbfcaf3c94e80c000, 0x3cba4e633fcd9066],
[0x3ff3a00000000000, 0xbfca23bc1fe2b000, 0xbd258c64dc46c1ea],
[0x3ff3a00000000000, 0xbfca23bc1fe2b000, 0xbd258c64dc46c1ea],
[0x3ff3800000000000, 0xbfc9525a9cf45000, 0xbd2ad1d904c1d4e3],
[0x3ff3600000000000, 0xbfc87fa06520d000, 0x3d2bbdbf7fdbfa09],
[0x3ff3400000000000, 0xbfc7ab890210e000, 0x3d2bdb9072534a58],
[0x3ff3400000000000, 0xbfc7ab890210e000, 0x3d2bdb9072534a58],
[0x3ff3200000000000, 0xbfc6d60fe719d000, 0xbd10e46aa3b2e266],
[0x3ff3000000000000, 0xbfc5ff3070a79000, 0xbd1e9e439f105039],
[0x3ff3000000000000, 0xbfc5ff3070a79000, 0xbd1e9e439f105039],
[0x3ff2e00000000000, 0xbfc526e5e3a1b000, 0xbd20de8b90075b8f],
[0x3ff2c00000000000, 0xbfc44d2b6ccb8000, 0x3d170cc16135783c],
[0x3ff2c00000000000, 0xbfc44d2b6ccb8000, 0x3d170cc16135783c],
[0x3ff2a00000000000, 0xbfc371fc201e9000, 0x3cf178864d27543a],
[0x3ff2800000000000, 0xbfc29552f81ff000, 0xbd248d301771c408],
[0x3ff2600000000000, 0xbfc1b72ad52f6000, 0xbd2e80a41811a396],
[0x3ff2600000000000, 0xbfc1b72ad52f6000, 0xbd2e80a41811a396],
[0x3ff2400000000000, 0xbfc0d77e7cd09000, 0x3d0a699688e85bf4],
[0x3ff2400000000000, 0xbfc0d77e7cd09000, 0x3d0a699688e85bf4],
[0x3ff2200000000000, 0xbfbfec9131dbe000, 0xbd2575545ca333f2],
[0x3ff2000000000000, 0xbfbe27076e2b0000, 0x3d2a342c2af0003c],
[0x3ff2000000000000, 0xbfbe27076e2b0000, 0x3d2a342c2af0003c],
[0x3ff1e00000000000, 0xbfbc5e548f5bc000, 0xbd1d0c57585fbe06],
[0x3ff1c00000000000, 0xbfba926d3a4ae000, 0x3d253935e85baac8],
[0x3ff1c00000000000, 0xbfba926d3a4ae000, 0x3d253935e85baac8],
[0x3ff1a00000000000, 0xbfb8c345d631a000, 0x3d137c294d2f5668],
[0x3ff1a00000000000, 0xbfb8c345d631a000, 0x3d137c294d2f5668],
[0x3ff1800000000000, 0xbfb6f0d28ae56000, 0xbd269737c93373da],
[0x3ff1600000000000, 0xbfb51b073f062000, 0x3d1f025b61c65e57],
[0x3ff1600000000000, 0xbfb51b073f062000, 0x3d1f025b61c65e57],
[0x3ff1400000000000, 0xbfb341d7961be000, 0x3d2c5edaccf913df],
[0x3ff1400000000000, 0xbfb341d7961be000, 0x3d2c5edaccf913df],
[0x3ff1200000000000, 0xbfb16536eea38000, 0x3d147c5e768fa309],
[0x3ff1000000000000, 0xbfaf0a30c0118000, 0x3d2d599e83368e91],
[0x3ff1000000000000, 0xbfaf0a30c0118000, 0x3d2d599e83368e91],
[0x3ff0e00000000000, 0xbfab42dd71198000, 0x3d1c827ae5d6704c],
[0x3ff0e00000000000, 0xbfab42dd71198000, 0x3d1c827ae5d6704c],
[0x3ff0c00000000000, 0xbfa77458f632c000, 0xbd2cfc4634f2a1ee],
[0x3ff0c00000000000, 0xbfa77458f632c000, 0xbd2cfc4634f2a1ee],
[0x3ff0a00000000000, 0xbfa39e87b9fec000, 0x3cf502b7f526feaa],
[0x3ff0a00000000000, 0xbfa39e87b9fec000, 0x3cf502b7f526feaa],
[0x3ff0800000000000, 0xbf9f829b0e780000, 0xbd2980267c7e09e4],
[0x3ff0800000000000, 0xbf9f829b0e780000, 0xbd2980267c7e09e4],
[0x3ff0600000000000, 0xbf97b91b07d58000, 0xbd288d5493faa639],
[0x3ff0400000000000, 0xbf8fc0a8b0fc0000, 0xbcdf1e7cf6d3a69c],
[0x3ff0400000000000, 0xbf8fc0a8b0fc0000, 0xbcdf1e7cf6d3a69c],
[0x3ff0200000000000, 0xbf7fe02a6b100000, 0xbd19e23f0dda40e4],
[0x3ff0200000000000, 0xbf7fe02a6b100000, 0xbd19e23f0dda40e4],
[0x3ff0000000000000, 0x0000000000000000, 0x0000000000000000],
[0x3ff0000000000000, 0x0000000000000000, 0x0000000000000000],
[0x3fefc00000000000, 0x3f80101575890000, 0xbd10c76b999d2be8],
[0x3fef800000000000, 0x3f90205658938000, 0xbd23dc5b06e2f7d2],
[0x3fef400000000000, 0x3f98492528c90000, 0xbd2aa0ba325a0c34],
[0x3fef000000000000, 0x3fa0415d89e74000, 0x3d0111c05cf1d753],
[0x3feec00000000000, 0x3fa466aed42e0000, 0xbd2c167375bdfd28],
[0x3fee800000000000, 0x3fa894aa149fc000, 0xbd197995d05a267d],
[0x3fee400000000000, 0x3faccb73cdddc000, 0xbd1a68f247d82807],
[0x3fee200000000000, 0x3faeea31c006c000, 0xbd0e113e4fc93b7b],
[0x3fede00000000000, 0x3fb1973bd1466000, 0xbd25325d560d9e9b],
[0x3feda00000000000, 0x3fb3bdf5a7d1e000, 0x3d2cc85ea5db4ed7],
[0x3fed600000000000, 0x3fb5e95a4d97a000, 0xbd2c69063c5d1d1e],
[0x3fed400000000000, 0x3fb700d30aeac000, 0x3cec1e8da99ded32],
[0x3fed000000000000, 0x3fb9335e5d594000, 0x3d23115c3abd47da],
[0x3fecc00000000000, 0x3fbb6ac88dad6000, 0xbd1390802bf768e5],
[0x3feca00000000000, 0x3fbc885801bc4000, 0x3d2646d1c65aacd3],
[0x3fec600000000000, 0x3fbec739830a2000, 0xbd2dc068afe645e0],
[0x3fec400000000000, 0x3fbfe89139dbe000, 0xbd2534d64fa10afd],
[0x3fec000000000000, 0x3fc1178e8227e000, 0x3d21ef78ce2d07f2],
[0x3febe00000000000, 0x3fc1aa2b7e23f000, 0x3d2ca78e44389934],
[0x3feba00000000000, 0x3fc2d1610c868000, 0x3d039d6ccb81b4a1],
[0x3feb800000000000, 0x3fc365fcb0159000, 0x3cc62fa8234b7289],
[0x3feb400000000000, 0x3fc4913d8333b000, 0x3d25837954fdb678],
[0x3feb200000000000, 0x3fc527e5e4a1b000, 0x3d2633e8e5697dc7],
[0x3feae00000000000, 0x3fc6574ebe8c1000, 0x3d19cf8b2c3c2e78],
[0x3feac00000000000, 0x3fc6f0128b757000, 0xbd25118de59c21e1],
[0x3feaa00000000000, 0x3fc7898d85445000, 0xbd1c661070914305],
[0x3fea600000000000, 0x3fc8beafeb390000, 0xbd073d54aae92cd1],
[0x3fea400000000000, 0x3fc95a5adcf70000, 0x3d07f22858a0ff6f],
[0x3fea000000000000, 0x3fca93ed3c8ae000, 0xbd28724350562169],
[0x3fe9e00000000000, 0x3fcb31d8575bd000, 0xbd0c358d4eace1aa],
[0x3fe9c00000000000, 0x3fcbd087383be000, 0xbd2d4bc4595412b6],
[0x3fe9a00000000000, 0x3fcc6ffbc6f01000, 0xbcf1ec72c5962bd2],
[0x3fe9600000000000, 0x3fcdb13db0d49000, 0xbd2aff2af715b035],
[0x3fe9400000000000, 0x3fce530effe71000, 0x3cc212276041f430],
[0x3fe9200000000000, 0x3fcef5ade4dd0000, 0xbcca211565bb8e11],
[0x3fe9000000000000, 0x3fcf991c6cb3b000, 0x3d1bcbecca0cdf30],
[0x3fe8c00000000000, 0x3fd07138604d5800, 0x3cf89cdb16ed4e91],
[0x3fe8a00000000000, 0x3fd0c42d67616000, 0x3d27188b163ceae9],
[0x3fe8800000000000, 0x3fd1178e8227e800, 0xbd2c210e63a5f01c],
[0x3fe8600000000000, 0x3fd16b5ccbacf800, 0x3d2b9acdf7a51681],
[0x3fe8400000000000, 0x3fd1bf99635a6800, 0x3d2ca6ed5147bdb7],
[0x3fe8200000000000, 0x3fd214456d0eb800, 0x3d0a87deba46baea],
[0x3fe7e00000000000, 0x3fd2bef07cdc9000, 0x3d2a9cfa4a5004f4],
[0x3fe7c00000000000, 0x3fd314f1e1d36000, 0xbd28e27ad3213cb8],
[0x3fe7a00000000000, 0x3fd36b6776be1000, 0x3d116ecdb0f177c8],
[0x3fe7800000000000, 0x3fd3c25277333000, 0x3d183b54b606bd5c],
[0x3fe7600000000000, 0x3fd419b423d5e800, 0x3d08e436ec90e09d],
[0x3fe7400000000000, 0x3fd4718dc271c800, 0xbd2f27ce0967d675],
[0x3fe7200000000000, 0x3fd4c9e09e173000, 0xbd2e20891b0ad8a4],
[0x3fe7000000000000, 0x3fd522ae0738a000, 0x3d2ebe708164c759],
[0x3fe6e00000000000, 0x3fd57bf753c8d000, 0x3d1fadedee5d40ef],
[0x3fe6c00000000000, 0x3fd5d5bddf596000, 0xbd0a0b2a08a465dc],
];
// Binary64 data from ARM optimized-routines math/exp_data.c.
const INV_LN2_N: f64 = 184.6649652337873;
const NEG_LN2_HI_N: f64 = -0.005415212348111709;
const NEG_LN2_LO_N: f64 = -1.2864023111638346e-14;
const SHIFT: f64 = 6755399441055744.0;
const EXP_POLY: [f64; 4] = [
    0.49999999999996786,
    0.16666666666665886,
    0.0416666808410674,
    0.008333335853059549,
];
#[rustfmt::skip]
static EXP_TAB: [[u64; 2]; 128] = [
[0x0000000000000000, 0x3ff0000000000000],
[0x3c9b3b4f1a88bf6e, 0x3feff63da9fb3335],
[0xbc7160139cd8dc5d, 0x3fefec9a3e778061],
[0xbc905e7a108766d1, 0x3fefe315e86e7f85],
[0x3c8cd2523567f613, 0x3fefd9b0d3158574],
[0xbc8bce8023f98efa, 0x3fefd06b29ddf6de],
[0x3c60f74e61e6c861, 0x3fefc74518759bc8],
[0x3c90a3e45b33d399, 0x3fefbe3ecac6f383],
[0x3c979aa65d837b6d, 0x3fefb5586cf9890f],
[0x3c8eb51a92fdeffc, 0x3fefac922b7247f7],
[0x3c3ebe3d702f9cd1, 0x3fefa3ec32d3d1a2],
[0xbc6a033489906e0b, 0x3fef9b66affed31b],
[0xbc9556522a2fbd0e, 0x3fef9301d0125b51],
[0xbc5080ef8c4eea55, 0x3fef8abdc06c31cc],
[0xbc91c923b9d5f416, 0x3fef829aaea92de0],
[0x3c80d3e3e95c55af, 0x3fef7a98c8a58e51],
[0xbc801b15eaa59348, 0x3fef72b83c7d517b],
[0xbc8f1ff055de323d, 0x3fef6af9388c8dea],
[0x3c8b898c3f1353bf, 0x3fef635beb6fcb75],
[0xbc96d99c7611eb26, 0x3fef5be084045cd4],
[0x3c9aecf73e3a2f60, 0x3fef54873168b9aa],
[0xbc8fe782cb86389d, 0x3fef4d5022fcd91d],
[0x3c8a6f4144a6c38d, 0x3fef463b88628cd6],
[0x3c807a05b0e4047d, 0x3fef3f49917ddc96],
[0x3c968efde3a8a894, 0x3fef387a6e756238],
[0x3c875e18f274487d, 0x3fef31ce4fb2a63f],
[0x3c80472b981fe7f2, 0x3fef2b4565e27cdd],
[0xbc96b87b3f71085e, 0x3fef24dfe1f56381],
[0x3c82f7e16d09ab31, 0x3fef1e9df51fdee1],
[0xbc3d219b1a6fbffa, 0x3fef187fd0dad990],
[0x3c8b3782720c0ab4, 0x3fef1285a6e4030b],
[0x3c6e149289cecb8f, 0x3fef0cafa93e2f56],
[0x3c834d754db0abb6, 0x3fef06fe0a31b715],
[0x3c864201e2ac744c, 0x3fef0170fc4cd831],
[0x3c8fdd395dd3f84a, 0x3feefc08b26416ff],
[0xbc86a3803b8e5b04, 0x3feef6c55f929ff1],
[0xbc924aedcc4b5068, 0x3feef1a7373aa9cb],
[0xbc9907f81b512d8e, 0x3feeecae6d05d866],
[0xbc71d1e83e9436d2, 0x3feee7db34e59ff7],
[0xbc991919b3ce1b15, 0x3feee32dc313a8e5],
[0x3c859f48a72a4c6d, 0x3feedea64c123422],
[0xbc9312607a28698a, 0x3feeda4504ac801c],
[0xbc58a78f4817895b, 0x3feed60a21f72e2a],
[0xbc7c2c9b67499a1b, 0x3feed1f5d950a897],
[0x3c4363ed60c2ac11, 0x3feece086061892d],
[0x3c9666093b0664ef, 0x3feeca41ed1d0057],
[0x3c6ecce1daa10379, 0x3feec6a2b5c13cd0],
[0x3c93ff8e3f0f1230, 0x3feec32af0d7d3de],
[0x3c7690cebb7aafb0, 0x3feebfdad5362a27],
[0x3c931dbdeb54e077, 0x3feebcb299fddd0d],
[0xbc8f94340071a38e, 0x3feeb9b2769d2ca7],
[0xbc87deccdc93a349, 0x3feeb6daa2cf6642],
[0xbc78dec6bd0f385f, 0x3feeb42b569d4f82],
[0xbc861246ec7b5cf6, 0x3feeb1a4ca5d920f],
[0x3c93350518fdd78e, 0x3feeaf4736b527da],
[0x3c7b98b72f8a9b05, 0x3feead12d497c7fd],
[0x3c9063e1e21c5409, 0x3feeab07dd485429],
[0x3c34c7855019c6ea, 0x3feea9268a5946b7],
[0x3c9432e62b64c035, 0x3feea76f15ad2148],
[0xbc8ce44a6199769f, 0x3feea5e1b976dc09],
[0xbc8c33c53bef4da8, 0x3feea47eb03a5585],
[0xbc845378892be9ae, 0x3feea34634ccc320],
[0xbc93cedd78565858, 0x3feea23882552225],
[0x3c5710aa807e1964, 0x3feea155d44ca973],
[0xbc93b3efbf5e2228, 0x3feea09e667f3bcd],
[0xbc6a12ad8734b982, 0x3feea012750bdabf],
[0xbc6367efb86da9ee, 0x3fee9fb23c651a2f],
[0xbc80dc3d54e08851, 0x3fee9f7df9519484],
[0xbc781f647e5a3ecf, 0x3fee9f75e8ec5f74],
[0xbc86ee4ac08b7db0, 0x3fee9f9a48a58174],
[0xbc8619321e55e68a, 0x3fee9feb564267c9],
[0x3c909ccb5e09d4d3, 0x3feea0694fde5d3f],
[0xbc7b32dcb94da51d, 0x3feea11473eb0187],
[0x3c94ecfd5467c06b, 0x3feea1ed0130c132],
[0x3c65ebe1abd66c55, 0x3feea2f336cf4e62],
[0xbc88a1c52fb3cf42, 0x3feea427543e1a12],
[0xbc9369b6f13b3734, 0x3feea589994cce13],
[0xbc805e843a19ff1e, 0x3feea71a4623c7ad],
[0xbc94d450d872576e, 0x3feea8d99b4492ed],
[0x3c90ad675b0e8a00, 0x3feeaac7d98a6699],
[0x3c8db72fc1f0eab4, 0x3feeace5422aa0db],
[0xbc65b6609cc5e7ff, 0x3feeaf3216b5448c],
[0x3c7bf68359f35f44, 0x3feeb1ae99157736],
[0xbc93091fa71e3d83, 0x3feeb45b0b91ffc6],
[0xbc5da9b88b6c1e29, 0x3feeb737b0cdc5e5],
[0xbc6c23f97c90b959, 0x3feeba44cbc8520f],
[0xbc92434322f4f9aa, 0x3feebd829fde4e50],
[0xbc85ca6cd7668e4b, 0x3feec0f170ca07ba],
[0x3c71affc2b91ce27, 0x3feec49182a3f090],
[0x3c6dd235e10a73bb, 0x3feec86319e32323],
[0xbc87c50422622263, 0x3feecc667b5de565],
[0x3c8b1c86e3e231d5, 0x3feed09bec4a2d33],
[0xbc91bbd1d3bcbb15, 0x3feed503b23e255d],
[0x3c90cc319cee31d2, 0x3feed99e1330b358],
[0x3c8469846e735ab3, 0x3feede6b5579fdbf],
[0xbc82dfcd978e9db4, 0x3feee36bbfd3f37a],
[0x3c8c1a7792cb3387, 0x3feee89f995ad3ad],
[0xbc907b8f4ad1d9fa, 0x3feeee07298db666],
[0xbc55c3d956dcaeba, 0x3feef3a2b84f15fb],
[0xbc90a40e3da6f640, 0x3feef9728de5593a],
[0xbc68d6f438ad9334, 0x3feeff76f2fb5e47],
[0xbc91eee26b588a35, 0x3fef05b030a1064a],
[0x3c74ffd70a5fddcd, 0x3fef0c1e904bc1d2],
[0xbc91bdfbfa9298ac, 0x3fef12c25bd71e09],
[0x3c736eae30af0cb3, 0x3fef199bdd85529c],
[0x3c8ee3325c9ffd94, 0x3fef20ab5fffd07a],
[0x3c84e08fd10959ac, 0x3fef27f12e57d14b],
[0x3c63cdaf384e1a67, 0x3fef2f6d9406e7b5],
[0x3c676b2c6c921968, 0x3fef3720dcef9069],
[0xbc808a1883ccb5d2, 0x3fef3f0b555dc3fa],
[0xbc8fad5d3ffffa6f, 0x3fef472d4a07897c],
[0xbc900dae3875a949, 0x3fef4f87080d89f2],
[0x3c74a385a63d07a7, 0x3fef5818dcfba487],
[0xbc82919e2040220f, 0x3fef60e316c98398],
[0x3c8e5a50d5c192ac, 0x3fef69e603db3285],
[0x3c843a59ac016b4b, 0x3fef7321f301b460],
[0xbc82d52107b43e1f, 0x3fef7c97337b9b5f],
[0xbc892ab93b470dc9, 0x3fef864614f5a129],
[0x3c74b604603a88d3, 0x3fef902ee78b3ff6],
[0x3c83c5ec519d7271, 0x3fef9a51fbc74c83],
[0xbc8ff7128fd391f0, 0x3fefa4afa2a490da],
[0xbc8dae98e223747d, 0x3fefaf482d8e67f1],
[0x3c8ec3bc41aa2008, 0x3fefba1bee615a27],
[0x3c842b94c3a9eb32, 0x3fefc52b376bba97],
[0x3c8a64a931d185ee, 0x3fefd0765b6e4540],
[0xbc8e37bae43be3ed, 0x3fefdbfdad9cbe14],
[0x3c77893b4d91cd9d, 0x3fefe7c1819e90d8],
[0x3c5305c14160cc89, 0x3feff3c22b8f71f1],
];

#[cfg(test)]
mod tests;
