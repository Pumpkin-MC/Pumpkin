pub struct BitStorage<T: AsRef<[i64]>> {
    data: T,
    bits: u8,
    size: usize,
    mask: u64,
}

impl<T: AsRef<[i64]>> BitStorage<T> {
    pub const fn new(bits: u8, size: usize, data: T) -> Self {
        let mask = (1u64 << bits) - 1;
        Self {
            data,
            bits,
            size,
            mask,
        }
    }

    pub fn get(&self, index: usize) -> u32 {
        debug_assert!(index < self.size);
        let bit_offset = index * self.bits as usize;
        let array_idx = bit_offset / 64;
        let bit_pos = (bit_offset % 64) as u32;

        let data = self.data.as_ref();
        let val = (data[array_idx] as u64) >> bit_pos;
        let next_idx = array_idx + 1;

        let res = if bit_pos + self.bits as u32 > 64 && next_idx < data.len() {
            val | (data[next_idx] as u64) << (64 - bit_pos)
        } else {
            val
        };

        (res & self.mask) as u32
    }

    pub const fn data(&self) -> &T {
        &self.data
    }
}

impl<T: AsRef<[i64]> + AsMut<[i64]>> BitStorage<T> {
    pub fn set(&mut self, index: usize, value: u32) {
        debug_assert!(index < self.size);
        debug_assert!(value as u64 <= self.mask);
        let bit_offset = index * self.bits as usize;
        let array_idx = bit_offset / 64;
        let bit_pos = (bit_offset % 64) as u32;

        let data = self.data.as_mut();
        data[array_idx] = (data[array_idx] as u64 & !(self.mask << bit_pos)
            | (value as u64 & self.mask) << bit_pos) as i64;
        let next_idx = array_idx + 1;

        if bit_pos + self.bits as u32 > 64 && next_idx < data.len() {
            let next_bits = bit_pos + self.bits as u32 - 64;
            data[next_idx] = (data[next_idx] as u64 & !((1u64 << next_bits) - 1)
                | (value as u64 & self.mask) >> (64 - bit_pos)) as i64;
        }
    }
}
