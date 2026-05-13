#[derive(Clone)]
pub struct CarvingMask {
    min_y: i32,
    mask: Vec<u64>,
}

impl CarvingMask {
    #[must_use]
    pub fn new(height: i32, min_y: i32) -> Self {
        // Each u64 stores 64 bits. Total bits needed: 16 * 16 * height.
        let total_bits = 16 * 16 * height;
        let num_u64s = (total_bits as usize).div_ceil(64);
        Self {
            min_y,
            mask: vec![0; num_u64s],
        }
    }

    const fn get_bit_index(&self, x: i32, y: i32, z: i32) -> usize {
        // x: 0..16, z: 0..16, y: min_y..min_y+height
        let rel_x = x & 15;
        let rel_z = z & 15;
        let rel_y = y - self.min_y;
        (rel_x | (rel_z << 4) | (rel_y << 8)) as usize
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32) {
        let bit_index = self.get_bit_index(x, y, z);
        let u64_idx = bit_index / 64;
        let bit_in_u64 = bit_index % 64;
        self.mask[u64_idx] |= 1 << bit_in_u64;
    }

    #[must_use]
    pub fn get(&self, x: i32, y: i32, z: i32) -> bool {
        let bit_index = self.get_bit_index(x, y, z);
        let u64_idx = bit_index / 64;
        let bit_in_u64 = bit_index % 64;
        (self.mask[u64_idx] >> bit_in_u64) & 1 != 0
    }
}
