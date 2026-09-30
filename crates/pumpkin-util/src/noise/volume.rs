use crate::math::vector3::Vector3;

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
