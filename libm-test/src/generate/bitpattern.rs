use libm::support::{Float, Int, MinInt};

/// An efficient equivalent to
/// `(0..=uN::MAX).filter(|x| x & !varying == 0).map(|x| x ^ preset)`
/// That is, "all integers that only differ from the preset in the varying bits"
pub struct BitConfig<I> {
    /// A bitmask of bits to list exhaustively
    varying: I,
    /// Other bits are set according to preset.
    preset: I,
}

impl<I: Int<Unsigned = I>> BitConfig<I> {
    fn into_iter(self) -> impl Iterator<Item = I> + Clone {
        assert!(
            self.varying != I::MAX,
            "to optimize the implementation, varying every bit is not supported"
        );
        let fixed = !self.varying;
        let flip = self.preset ^ fixed;
        let mut counter = fixed - I::ONE;

        // `(counter + 1) & !fixed` is initially 0, and increases after each item returned
        std::iter::from_fn(move || {
            counter = counter.checked_add(I::ONE)? | fixed;
            Some(counter ^ flip)
        })
    }
}

/// Biased generator for floats.
///
/// The returned iterator will produce `fillers.len() << bits_to_vary` items.
pub fn float_gen<F>(bits_to_vary: u32, fillers: Vec<F::Int>) -> impl Iterator<Item = F> + Clone
where
    F: Float,
    F::Int: Int<Unsigned = F::Int>,
{
    assert!(bits_to_vary < F::Int::BITS);

    let mut bit_priority: Vec<_> = (0..F::BITS).rev().collect();
    // sign bit first, otherwise by least distance to any edge of a bitfield,
    bit_priority[1..].sort_by_key(|&i| {
        // avoid a fencepost error by mapping the bit indices to odd numbers,
        // and compare them to the bitfield edges mapped to even integers
        let i = 2 * i + 1;
        i.min(i.abs_diff(F::SIG_BITS * 2))
            .min(i.abs_diff((F::BITS - 1) * 2))
    });

    let varying = bit_priority[..bits_to_vary as usize]
        .iter()
        .map(|&i| F::Int::ONE << i)
        .reduce(std::ops::BitOr::bitor)
        .unwrap_or(F::Int::ZERO);

    fillers
        .into_iter()
        .map(move |preset| BitConfig {
            preset: preset & !varying,
            varying,
        })
        .flat_map(BitConfig::into_iter)
        .map(F::from_bits)
}

#[cfg(test)]
mod test {
    use super::{BitConfig, float_gen};
    #[test]
    fn equivalence() {
        // with a small integer type, we can easily verify that behaviour matches for all arguments
        for varying in 0..u8::MAX {
            for preset in 0..=u8::MAX {
                let expect = (0..=u8::MAX)
                    .filter(|x| x & !varying == 0)
                    .map(|x| x ^ preset);
                let iter = BitConfig { varying, preset }.into_iter();
                assert!(iter.eq(expect));
            }
        }
    }
    #[test]
    fn gen_includes_specials() {
        let v: Vec<_> = float_gen(5, vec![0, 0x7fffff, !0x7fffff, !0])
            .map(f32::to_bits)
            .collect();
        for x in &[
            0.0,
            f32::from_bits(1),
            f32::MIN_POSITIVE,
            f32::MAX,
            f32::INFINITY,
            f32::NAN,
        ] {
            assert!(v.contains(&x.to_bits()), "{x} not found");
            assert!(v.contains(&(-x).to_bits()), "-{x} not found");
        }
    }
    #[test]
    fn count() {
        for k in 0..10 {
            assert!(float_gen::<f32>(k, vec![0]).count() == 1 << k);
            assert!(float_gen::<f32>(k, vec![0, !0]).count() == 2 << k);
            assert!(float_gen::<f32>(k, vec![0, 1, 2]).count() == 3 << k);
        }
    }
    #[test]
    fn specific() {
        let iter = float_gen::<f32>(1, vec![0]).map(f32::to_bits);
        assert!(iter.eq([0.0_f32.to_bits(), (-0.0_f32).to_bits()]));
        let iter = float_gen::<f64>(1, vec![0]).map(f64::to_bits);
        assert!(iter.eq([0.0_f64.to_bits(), (-0.0_f64).to_bits()]));

        let mut v: Vec<_> = float_gen::<f32>(5, vec![0]).map(f32::to_bits).collect();
        assert!(v.len() == 32);
        assert!(v.is_sorted());
        v.dedup();
        assert!(v.len() == 32);
        for bits in v {
            assert!(bits & 0xc0c0_0001 == bits);
        }

        let mut v: Vec<_> = float_gen::<f64>(5, vec![0]).map(f64::to_bits).collect();
        assert!(v.len() == 32);
        assert!(v.is_sorted());
        v.dedup();
        assert!(v.len() == 32);
        for bits in v {
            assert!(bits & 0xc018_0000_0000_0001 == bits);
        }
    }
}
