use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use pumpkin_util::math::vector3::Vector3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DensityVolume {
    pub size_x: usize,
    pub size_y: usize,
    pub size_z: usize,
    pub min_block_x: i32,
    pub min_block_y: i32,
    pub min_block_z: i32,
    pub step_block_x: i32,
    pub step_block_y: i32,
    pub step_block_z: i32,
}

impl DensityVolume {
    #[must_use]
    #[expect(clippy::too_many_arguments)]
    pub const fn new(
        size_x: usize,
        size_y: usize,
        size_z: usize,
        min_block_x: i32,
        min_block_y: i32,
        min_block_z: i32,
        step_block_x: i32,
        step_block_y: i32,
        step_block_z: i32,
    ) -> Self {
        debug_assert!(size_x > 0 && size_y > 0 && size_z > 0);
        debug_assert!(step_block_x > 0 && step_block_y > 0 && step_block_z > 0);
        Self {
            size_x,
            size_y,
            size_z,
            min_block_x,
            min_block_y,
            min_block_z,
            step_block_x,
            step_block_y,
            step_block_z,
        }
    }

    #[must_use]
    pub const fn with_block_step(
        size_x: usize,
        size_y: usize,
        size_z: usize,
        min_block_x: i32,
        min_block_y: i32,
        min_block_z: i32,
    ) -> Self {
        Self::new(
            size_x,
            size_y,
            size_z,
            min_block_x,
            min_block_y,
            min_block_z,
            1,
            1,
            1,
        )
    }

    #[inline]
    #[must_use]
    pub const fn index_unchecked(&self, x: usize, y: usize, z: usize) -> usize {
        y + (x + z * self.size_x) * self.size_y
    }

    #[inline]
    #[must_use]
    pub const fn block_x(&self, x: usize) -> i32 {
        self.min_block_x + x as i32 * self.step_block_x
    }

    #[inline]
    #[must_use]
    pub const fn block_y(&self, y: usize) -> i32 {
        self.min_block_y + y as i32 * self.step_block_y
    }

    #[inline]
    #[must_use]
    pub const fn block_z(&self, z: usize) -> i32 {
        self.min_block_z + z as i32 * self.step_block_z
    }

    #[must_use]
    pub const fn max_block_x(&self) -> i32 {
        self.min_block_x + self.size_x as i32 * self.step_block_x - 1
    }

    #[must_use]
    pub const fn max_block_y(&self) -> i32 {
        self.min_block_y + self.size_y as i32 * self.step_block_y - 1
    }

    #[must_use]
    pub const fn max_block_z(&self) -> i32 {
        self.min_block_z + self.size_z as i32 * self.step_block_z - 1
    }

    #[inline]
    #[must_use]
    pub const fn size(&self) -> usize {
        self.size_x * self.size_y * self.size_z
    }

    #[must_use]
    pub const fn is_block_step(&self) -> bool {
        self.step_block_x == 1 && self.step_block_y == 1 && self.step_block_z == 1
    }

    const fn contains_block_relative(
        &self,
        relative_x: i32,
        relative_y: i32,
        relative_z: i32,
    ) -> bool {
        relative_x >= 0
            && relative_y >= 0
            && relative_z >= 0
            && relative_x < self.size_x as i32 * self.step_block_x
            && relative_y < self.size_y as i32 * self.step_block_y
            && relative_z < self.size_z as i32 * self.step_block_z
            && relative_x.rem_euclid(self.step_block_x) == 0
            && relative_y.rem_euclid(self.step_block_y) == 0
            && relative_z.rem_euclid(self.step_block_z) == 0
    }

    #[must_use]
    pub const fn index_of_block(&self, block_x: i32, block_y: i32, block_z: i32) -> Option<usize> {
        let relative_x = block_x - self.min_block_x;
        let relative_y = block_y - self.min_block_y;
        let relative_z = block_z - self.min_block_z;
        if self.is_block_step() {
            if relative_x >= 0
                && relative_y >= 0
                && relative_z >= 0
                && relative_x < self.size_x as i32
                && relative_y < self.size_y as i32
                && relative_z < self.size_z as i32
            {
                return Some(self.index_unchecked(
                    relative_x as usize,
                    relative_y as usize,
                    relative_z as usize,
                ));
            }
        } else if self.contains_block_relative(relative_x, relative_y, relative_z) {
            return Some(self.index_unchecked(
                relative_x.div_euclid(self.step_block_x) as usize,
                relative_y.div_euclid(self.step_block_y) as usize,
                relative_z.div_euclid(self.step_block_z) as usize,
            ));
        }
        None
    }

    pub fn fill_with(&self, buffer: &mut [f32], mut sample: impl FnMut(&Vector3<i32>) -> f32) {
        debug_assert_eq!(buffer.len(), self.size());
        let mut index = 0;
        for z in 0..self.size_z {
            let block_z = self.block_z(z);
            for x in 0..self.size_x {
                let block_x = self.block_x(x);
                for y in 0..self.size_y {
                    buffer[index] = sample(&Vector3::new(block_x, self.block_y(y), block_z));
                    index += 1;
                }
            }
        }
    }
}

const BUFFER_SIZE_INCREMENT: usize = 16;
const MAX_REUSE_SIZE_FACTOR: usize = 2;
const MAX_POOLED_BUFFERS: usize = 1024;

thread_local! {
    static DENSITY_BUFFER_POOL: RefCell<Vec<Box<[f32]>>> = const { RefCell::new(Vec::new()) };
}

fn take_best(
    pool: &mut Vec<Box<[f32]>>,
    min_capacity: usize,
    max_capacity: usize,
) -> Option<Box<[f32]>> {
    let mut best_index = None;
    let mut best_capacity = max_capacity + 1;
    for i in (0..pool.len()).rev() {
        let capacity = pool[i].len();
        if capacity == min_capacity {
            return Some(pool.remove(i));
        }
        if capacity > min_capacity && capacity < best_capacity {
            best_index = Some(i);
            best_capacity = capacity;
        }
    }
    best_index.map(|i| pool.remove(i))
}

pub struct DensityBuffer {
    values: Box<[f32]>,
    len: usize,
}

impl DensityBuffer {
    #[must_use]
    pub fn acquire(volume: &DensityVolume) -> Self {
        Self::with_len(volume.size())
    }

    #[must_use]
    pub fn with_len(len: usize) -> Self {
        let min_capacity = len.next_multiple_of(BUFFER_SIZE_INCREMENT);
        let values = DENSITY_BUFFER_POOL
            .with(|pool| {
                take_best(
                    &mut pool.borrow_mut(),
                    min_capacity,
                    min_capacity * MAX_REUSE_SIZE_FACTOR,
                )
            })
            .unwrap_or_else(|| vec![0.0; min_capacity].into_boxed_slice());
        Self { values, len }
    }

    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.values.len()
    }
}

impl Deref for DensityBuffer {
    type Target = [f32];

    #[inline]
    fn deref(&self) -> &[f32] {
        &self.values[..self.len]
    }
}

impl DerefMut for DensityBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut [f32] {
        &mut self.values[..self.len]
    }
}

impl Drop for DensityBuffer {
    fn drop(&mut self) {
        let values = std::mem::take(&mut self.values);
        DENSITY_BUFFER_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            if pool.len() < MAX_POOLED_BUFFERS {
                pool.push(values);
            }
        });
    }
}
