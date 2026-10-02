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

/// Bit storage whose values never span two words, vanilla's `SimpleBitStorage`.
///
/// Chunk sections use this layout in memory, on disk and on the network since 1.16, so a
/// palette can hand its words out without repacking them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimpleBitStorage {
    data: Box<[u64]>,
    bits: u8,
    mask: u64,
    size: usize,
    values_per_long: usize,
    divide_mul: u64,
    divide_add: u64,
    divide_shift: u32,
}

impl SimpleBitStorage {
    /// Words needed for `size` values of `bits` each.
    #[must_use]
    pub const fn word_count(bits: u8, size: usize) -> usize {
        size.div_ceil(64 / bits as usize)
    }

    #[must_use]
    pub fn new(bits: u8, size: usize) -> Self {
        Self::from_data(
            bits,
            size,
            vec![0; Self::word_count(bits, size)].into_boxed_slice(),
        )
    }

    /// Wraps words already in this layout; `data` holds at least `word_count(bits, size)` words.
    #[must_use]
    pub fn from_data(bits: u8, size: usize, data: Box<[u64]>) -> Self {
        debug_assert!((1..=32).contains(&bits));
        debug_assert!(data.len() >= Self::word_count(bits, size));
        let values_per_long = 64 / bits as usize;
        // Vanilla takes these from its MAGIC table. They are the fixed-point reciprocal of
        // `values_per_long` that `cell_index` multiplies by instead of dividing.
        let (divide_mul, divide_add, divide_shift) = if values_per_long.is_power_of_two() {
            (1 << 31, 0, values_per_long.trailing_zeros() - 1)
        } else {
            let reciprocal = (1u64 << 32) / values_per_long as u64;
            (reciprocal, reciprocal, 0)
        };
        Self {
            data,
            bits,
            mask: (1u64 << bits) - 1,
            size,
            values_per_long,
            divide_mul,
            divide_add,
            divide_shift,
        }
    }

    const fn cell_index(&self, index: usize) -> usize {
        ((index as u64 * self.divide_mul + self.divide_add) >> 32 >> self.divide_shift) as usize
    }

    #[must_use]
    pub fn get(&self, index: usize) -> u32 {
        debug_assert!(index < self.size);
        let cell = self.cell_index(index);
        let shift = (index - cell * self.values_per_long) * self.bits as usize;
        ((self.data[cell] >> shift) & self.mask) as u32
    }

    pub fn set(&mut self, index: usize, value: u32) {
        debug_assert!(index < self.size);
        debug_assert!(u64::from(value) <= self.mask);
        let cell = self.cell_index(index);
        let shift = (index - cell * self.values_per_long) * self.bits as usize;
        self.data[cell] = (self.data[cell] & !(self.mask << shift)) | (u64::from(value) << shift);
    }

    /// The same values stored `bits` wide.
    #[must_use]
    pub fn resized(&self, bits: u8) -> Self {
        let mut resized = Self::new(bits, self.size);
        for index in 0..self.size {
            resized.set(index, self.get(index));
        }
        resized
    }

    #[must_use]
    pub const fn bits(&self) -> u8 {
        self.bits
    }

    #[must_use]
    pub const fn size(&self) -> usize {
        self.size
    }

    #[must_use]
    pub const fn data(&self) -> &[u64] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::SimpleBitStorage;

    #[test]
    fn simple_bit_storage_keeps_every_width_apart() {
        for bits in 1..=16u8 {
            let mut storage = SimpleBitStorage::new(bits, 4096);
            let mask = (1u32 << bits) - 1;
            for index in 0..4096 {
                storage.set(index, index as u32 & mask);
            }
            for index in 0..4096 {
                assert_eq!(
                    storage.get(index),
                    index as u32 & mask,
                    "bits {bits} index {index}"
                );
            }
            assert_eq!(storage.resized(16).get(4095), 4095 & mask);
        }
    }

    #[test]
    fn simple_bit_storage_matches_the_vanilla_layout() {
        let mut storage = SimpleBitStorage::new(4, 32);
        for index in 0..32 {
            storage.set(index, index as u32 & 0xF);
        }
        // First value in the lowest bits, sixteen values per word.
        assert_eq!(
            storage.data(),
            &[0xFEDC_BA98_7654_3210, 0xFEDC_BA98_7654_3210]
        );
    }
}
