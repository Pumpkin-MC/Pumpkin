use std::{
    sync::atomic::{AtomicU64, Ordering},
    time,
};

use enum_dispatch::enum_dispatch;
use legacy_rand::{LegacyRand, LegacySplitter};
use xoroshiro128::{Xoroshiro, XoroshiroSplitter};

mod gaussian;
pub mod legacy_rand;
pub mod xoroshiro128;

static SEED_UNIQUIFIER: AtomicU64 = AtomicU64::new(8682522807148012u64);

pub fn get_seed() -> u64 {
    let seed = SEED_UNIQUIFIER
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
            Some(val.wrapping_mul(1181783497276652981u64))
        })
        // We always return `Some``, so there will always be an `Ok` result
        .unwrap();

    let nanos = time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let nano_upper = (nanos >> 8) as u64;
    let nano_lower = nanos as u64;
    seed ^ nano_upper ^ nano_lower
}

#[enum_dispatch(RandomImpl)]
pub enum RandomGenerator {
    Xoroshiro(Xoroshiro),
    Legacy(LegacyRand),
}

#[derive(Clone)]
#[enum_dispatch(RandomDeriverImpl)]
pub enum RandomDeriver {
    Xoroshiro(XoroshiroSplitter),
    Legacy(LegacySplitter),
}

// TODO: Write unit test for this
#[macro_export]
macro_rules! population_seed_fn {
    () => {
        pub fn get_population_seed(world_seed: u64, block_x: i32, block_z: i32) -> u64 {
            let mut rand = Self::from_seed(world_seed);
            let l = rand.next_i64() | 1;
            let m = rand.next_i64() | 1;
            let base = (block_x as i64)
                .wrapping_mul(l)
                .wrapping_add((block_z as i64).wrapping_mul(m));
            (base as u64) ^ world_seed
        }
    };
}

// TODO: Write unit test for this
#[inline]
pub fn get_decorator_seed(population_seed: u64, index: usize, step: usize) -> u64 {
    population_seed
        .wrapping_add(index as u64)
        .wrapping_add(10_000u64.wrapping_mul(step as u64))
}

#[enum_dispatch]
pub trait RandomImpl {
    fn split(&mut self) -> Self;

    fn next_splitter(&mut self) -> RandomDeriver;

    fn next_i32(&mut self) -> i32;

    fn next_bounded_i32(&mut self, bound: i32) -> i32;

    fn next_inbetween_i32(&mut self, min: i32, max: i32) -> i32 {
        self.next_bounded_i32(max - min + 1) + min
    }

    fn next_i64(&mut self) -> i64;

    fn next_bool(&mut self) -> bool;

    fn next_f32(&mut self) -> f32;

    fn next_f64(&mut self) -> f64;

    fn next_gaussian(&mut self) -> f64;

    fn next_triangular(&mut self, mode: f64, deviation: f64) -> f64 {
        mode + deviation * (self.next_f64() - self.next_f64())
    }

    fn skip(&mut self, count: i32) {
        for _ in 0..count {
            self.next_i64();
        }
    }

    fn next_inbetween_i32_exclusive(&mut self, min: i32, max: i32) -> i32 {
        min + self.next_bounded_i32(max - min)
    }
}

#[enum_dispatch]
pub trait RandomDeriverImpl {
    fn split_string(&self, seed: &str) -> RandomGenerator;

    fn split_u64(&self, seed: u64) -> RandomGenerator;

    fn split_pos(&self, x: i32, y: i32, z: i32) -> RandomGenerator;
}

pub fn hash_block_pos(x: i32, y: i32, z: i32) -> i64 {
    let l = (x.wrapping_mul(3129871) as i64) ^ ((z as i64).wrapping_mul(116129781i64)) ^ (y as i64);
    let l = l
        .wrapping_mul(l)
        .wrapping_mul(42317861i64)
        .wrapping_add(l.wrapping_mul(11i64));
    l >> 16
}

#[cfg(test)]
mod tests {

    use super::hash_block_pos;

    #[test]
    fn block_position_hash() {
        let values: [((i32, i32, i32), i64); 8] = [
            ((0, 0, 0), 0),
            ((1, 1, 1), 60311958971344),
            ((4, 4, 4), 120566413180880),
            ((25, 25, 25), 111753446486209),
            ((676, 676, 676), 75210837988243),
            ((458329, 458329, 458329), -43764888250),
            ((-387008604, -387008604, -387008604), 8437923733503),
            ((176771161, 176771161, 176771161), 18421337580760),
        ];

        for ((x, y, z), value) in values {
            assert_eq!(hash_block_pos(x, y, z), value);
        }
    }
}
