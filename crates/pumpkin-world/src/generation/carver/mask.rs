use super::CarverOutput;

pub struct CarvingMask {
    min_y: i32,
    max_y: i32,
    height: usize,
    mask: Vec<u64>,
}

impl CarvingMask {
    #[must_use]
    pub fn new(min_y: i32, max_y: i32) -> Self {
        let height = ((max_y - min_y) + 1).max(0) as usize;
        let total_bits = 256 * height;
        let num_u64s = total_bits.div_ceil(64);
        Self {
            min_y,
            max_y,
            height,
            mask: vec![0; num_u64s],
        }
    }

    #[must_use]
    pub const fn min_y(&self) -> i32 {
        self.min_y
    }

    #[must_use]
    pub const fn max_y(&self) -> i32 {
        self.max_y
    }

    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    pub fn set(&mut self, x: usize, y: i32, z: usize) {
        if y < self.min_y || y > self.max_y || x >= 16 || z >= 16 {
            return;
        }
        let index = (y - self.min_y) as usize + ((z + (x << 4)) * self.height);
        let u64_idx = index / 64;
        let bit_in_u64 = index % 64;
        self.mask[u64_idx] |= 1 << bit_in_u64;
    }

    #[must_use]
    pub fn get(&self, x: usize, y: i32, z: usize) -> bool {
        if y < self.min_y || y > self.max_y || x >= 16 || z >= 16 {
            return false;
        }
        let index = (y - self.min_y) as usize + ((z + (x << 4)) * self.height);
        let u64_idx = index / 64;
        let bit_in_u64 = index % 64;
        (self.mask[u64_idx] >> bit_in_u64) & 1 != 0
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mask.iter().all(|&w| w == 0)
    }

    #[must_use]
    pub fn next_set_bit(&self, from: usize) -> Option<usize> {
        let total_bits = 256 * self.height;
        if from >= total_bits {
            return None;
        }
        let mut word_idx = from / 64;
        let bit_idx = from % 64;
        let mut word = self.mask[word_idx] & (!0u64 << bit_idx);
        loop {
            if word != 0 {
                let bit = word.trailing_zeros() as usize;
                let idx = word_idx * 64 + bit;
                return (idx < total_bits).then_some(idx);
            }
            word_idx += 1;
            if word_idx >= self.mask.len() {
                return None;
            }
            word = self.mask[word_idx];
        }
    }

    #[must_use]
    pub fn next_clear_bit(&self, from: usize) -> usize {
        let total_bits = 256 * self.height;
        if from >= total_bits {
            return total_bits;
        }
        let mut word_idx = from / 64;
        let bit_idx = from % 64;
        let mut word = (!self.mask[word_idx]) & (!0u64 << bit_idx);
        loop {
            if word != 0 {
                let bit = word.trailing_zeros() as usize;
                let idx = word_idx * 64 + bit;
                return idx.min(total_bits);
            }
            word_idx += 1;
            if word_idx >= self.mask.len() {
                return total_bits;
            }
            word = !self.mask[word_idx];
        }
    }

    pub fn visit<F>(&self, mut visitor: F)
    where
        F: FnMut(usize, usize, i32, i32),
    {
        let total_bits = 256 * self.height;
        let mut next_set = self.next_set_bit(0);
        while let Some(start_index) = next_set {
            let end_index = self.next_clear_bit(start_index) - 1;
            self.visit_segment(&mut visitor, start_index, end_index);
            if end_index + 1 >= total_bits {
                break;
            }
            next_set = self.next_set_bit(end_index + 1);
        }
    }

    fn visit_segment<F>(&self, visitor: &mut F, start_index: usize, end_index: usize)
    where
        F: FnMut(usize, usize, i32, i32),
    {
        let start_column = start_index / self.height;
        let end_column = end_index / self.height;
        for column in start_column..=end_column {
            let column_x = (column >> 4) & 15;
            let column_z = column & 15;
            let column_base_index = column * self.height;
            let bottom_y =
                (start_index.saturating_sub(column_base_index) as i32).max(0) + self.min_y;
            let top_y =
                ((end_index - column_base_index) as i32).min(self.height as i32 - 1) + self.min_y;
            visitor(column_x, column_z, bottom_y, top_y);
        }
    }
}

impl CarverOutput for CarvingMask {
    fn carve(&mut self, x: usize, y: i32, z: usize) {
        self.set(x, y, z);
    }

    fn min_y(&self) -> i32 {
        self.min_y
    }

    fn max_y(&self) -> i32 {
        self.max_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carving_mask_visit() {
        let mut mask = CarvingMask::new(-64, 320);
        assert!(mask.is_empty());

        mask.set(0, 10, 0);
        mask.set(0, 11, 0);
        mask.set(0, 12, 0);
        mask.set(1, 20, 2);

        let mut visits = Vec::new();
        mask.visit(|x, z, bottom_y, top_y| {
            visits.push((x, z, bottom_y, top_y));
        });

        assert_eq!(visits, vec![(0, 0, 10, 12), (1, 2, 20, 20)]);
    }
}
