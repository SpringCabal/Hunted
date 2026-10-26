/// Per-unit cosmetics must not change gameplay when coroutine wake order changes.
/// The multiply fits exactly in the engine's float32 Lua_Number as well as u32.
#[derive(Clone, Copy)]
pub struct Random(u32);

impl Random {
    pub fn for_unit(unit: i32) -> Self {
        Self(unit as u32 % 65521)
    }

    pub fn float(&mut self) -> f32 {
        self.0 = (self.0 * 251 + 1) % 65521;
        self.0 as f32 / 65521.0
    }

    pub fn int(&mut self, lower: i32, upper: i32) -> i32 {
        lower + (self.float() * (upper - lower + 1) as f32).floor() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streams_are_independent_of_other_units_and_their_wake_order() {
        let mut a = Random::for_unit(19939);
        let mut b = a;
        let mut other = Random::for_unit(9096);
        for _ in 0..1000 {
            other.float();
            other.float();
            assert_eq!(a.float().to_bits(), b.float().to_bits());
        }
    }
    #[test]
    fn float32_lua_arithmetic_reproduces_the_integer_generator() {
        for seed in [0, 1, 19939, 32767, 65520] {
            let mut rng = Random::for_unit(seed);
            let mut lua = seed as f32;
            for _ in 0..1000 {
                lua = (lua * 251.0 + 1.0) % 65521.0;
                assert_eq!(rng.float().to_bits(), (lua / 65521.0).to_bits());
            }
        }
    }
    #[test]
    fn integer_samples_stay_in_the_inclusive_animation_ranges() {
        let mut rng = Random::for_unit(12);
        for _ in 0..10000 {
            assert!((-10..=10).contains(&rng.int(-10, 10)));
            assert!((1..=6).contains(&rng.int(1, 6)));
        }
    }
}
