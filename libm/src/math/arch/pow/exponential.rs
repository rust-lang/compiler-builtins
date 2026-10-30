// Copyright (c) 2018-2026, Arm Limited. SPDX-License-Identifier: MIT
use super::super::exp_data::{EXP_POLY as C, EXP_TAB, INV_LN2_N, NEG_LN2_HI_N, NEG_LN2_LO_N, SHIFT};

#[inline]
pub(super) fn evaluate(x: f64, tail: f64, sign: u32) -> f64 {
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
