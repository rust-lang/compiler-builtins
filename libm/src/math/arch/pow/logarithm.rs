// Copyright (c) 2018-2026, Arm Limited. SPDX-License-Identifier: MIT
use super::log_data::{LN2_HI, LN2_LO, LOG_POLY as A, LOG_TAB};

#[inline]
pub(super) fn evaluate(ix: u64) -> (f64, f64) {
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
