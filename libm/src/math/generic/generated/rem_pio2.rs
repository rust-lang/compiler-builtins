use super::super::rem_pio2::RemPio2Support;
use crate::support::{HalfRep, f64_to_bits};

impl RemPio2Support for f64 {
    const TO_INT: Self = 1.5 / f64::EPSILON;
    const INV_PIO2: Self = hf64!("0xa.2f9836e4e4418p-4");
    const PIO2_1: Self = 1.57079632673412561417e+00;
    const PIO2_1T: Self = 6.07710050650619224932e-11;
    const PIO2_2: Self = 6.07710050630396597660e-11;
    const PIO2_2T: Self = 2.02226624879595063154e-21;
    const PIO2_3: Self = 2.02226624871116645580e-21;
    const PIO2_3T: Self = 8.47842766036889956997e-32;

    const FRAC_5PI_4_HI: HalfRep<Self> = (f64_to_bits(hf64!("0x3.ed4f452aa70bcp+0")) >> 32) as u32;
    const FRAC_3PI_4_HI: HalfRep<Self> = (f64_to_bits(hf64!("0x2.5b2f8fe6643a4p+0")) >> 32) as u32;
    const FRAC_9PI_4_HI: HalfRep<Self> = (f64_to_bits(hf64!("0x7.118eafb32caecp+0")) >> 32) as u32;
    const FRAC_7PI_4_HI: HalfRep<Self> = (f64_to_bits(hf64!("0x5.7f6efa6ee9dd4p+0")) >> 32) as u32;
    const FRAC_3PI_2_HI: HalfRep<Self> = (f64_to_bits(hf64!("0x4.b65f1fccc8748p+0")) >> 32) as u32;
    const TAU_HI: HalfRep<Self> = (f64_to_bits(hf64!("0x6.487ed5110b46p+0")) >> 32) as u32;
    const FRAC_PI_2_HI: HalfRep<Self> = 0x921fb;
    const FRAC_2_POW_20_PI_2: HalfRep<Self> =
        (f64_to_bits(hf64!("0x1.921fb54442d18p+20")) >> 32) as u32;

    const MAGIC_F: Self = hf64!("0x1p24");

    fn large(x: &[Self], y: &mut [Self], e0: i32, prec: usize) -> i32 {
        super::super::super::rem_pio2_large(x, y, e0, prec)
    }
}
