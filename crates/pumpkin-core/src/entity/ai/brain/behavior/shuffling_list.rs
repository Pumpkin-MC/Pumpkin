use rand::{Rng, RngExt};

pub struct WeightedEntry<T> {
    pub data: T,
    pub weight: i32,
    rand_weight: f64,
}

impl<T> WeightedEntry<T> {
    #[must_use]
    pub const fn rand_weight(&self) -> f64 {
        self.rand_weight
    }
}

pub struct ShufflingList<T> {
    entries: Vec<WeightedEntry<T>>,
}

impl<T> Default for ShufflingList<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<T> ShufflingList<T> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, data: T, weight: i32) {
        self.entries.push(WeightedEntry {
            data,
            weight,
            rand_weight: 0.0,
        });
    }

    // Java computes `1.0F / weight` in float before widening, so the float division stays.
    pub fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        for entry in &mut self.entries {
            let sample: f32 = rng.random();
            entry.rand_weight = -f64::from(sample).powf(f64::from(1.0f32 / entry.weight as f32));
        }
        self.entries
            .sort_by(|a, b| a.rand_weight.total_cmp(&b.rand_weight));
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries.iter().map(|entry| &entry.data)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries.iter_mut().map(|entry| &mut entry.data)
    }

    #[must_use]
    pub fn entries(&self) -> &[WeightedEntry<T>] {
        &self.entries
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use rand::{SeedableRng, TryRng, rngs::StdRng};

    use super::*;

    struct ConstantRng;

    impl TryRng for ConstantRng {
        type Error = Infallible;

        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            Ok(0x4000_0000)
        }

        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            Ok(0x4000_0000_4000_0000)
        }

        fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
            dst.fill(0x40);
            Ok(())
        }
    }

    #[test]
    fn heavier_weights_sort_first() {
        let mut list = ShufflingList::new();
        list.add("light", 1);
        list.add("heavy", i32::MAX);
        let mut rng = StdRng::seed_from_u64(7);
        for _ in 0..50 {
            list.shuffle(&mut rng);
            assert_eq!(list.iter().next(), Some(&"heavy"));
        }
    }

    #[test]
    fn weight_one_stays_within_minus_one_to_zero() {
        let mut list = ShufflingList::new();
        for index in 0..32 {
            list.add(index, 1);
        }
        let mut rng = StdRng::seed_from_u64(11);
        list.shuffle(&mut rng);
        for entry in list.entries() {
            assert!(entry.rand_weight() > -1.0 && entry.rand_weight() <= 0.0);
        }
    }

    #[test]
    fn equal_random_weights_keep_insertion_order() {
        let mut list = ShufflingList::new();
        for index in 0..8 {
            list.add(index, 1);
        }
        list.shuffle(&mut ConstantRng);
        assert_eq!(
            list.iter().copied().collect::<Vec<_>>(),
            (0..8).collect::<Vec<_>>()
        );
    }
}
