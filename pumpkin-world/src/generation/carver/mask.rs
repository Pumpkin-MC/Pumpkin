pub type AdditionalCarvingMask = Box<dyn Fn(i32, i32, i32) -> bool + Send + Sync>;

pub struct CarvingMask {
    min_y: i8,
    height: u16,
    mask: Vec<bool>,
    column_mask: [bool; 256],
    additional_mask: Option<AdditionalCarvingMask>,
}

impl CarvingMask {
    #[must_use]
    pub fn new(height: u16, min_y: i8) -> Self {
        let size = 16usize * 16usize * height as usize;
        Self {
            min_y,
            height,
            mask: vec![false; size],
            column_mask: [false; 256],
            additional_mask: None,
        }
    }

    pub fn set_additional_mask<F>(&mut self, mask: F)
    where
        F: Fn(i32, i32, i32) -> bool + Send + Sync + 'static,
    {
        self.additional_mask = Some(Box::new(mask));
    }

    pub fn clear_additional_mask(&mut self) {
        self.additional_mask = None;
    }

    pub fn reset_column_mask(&mut self) {
        self.column_mask.fill(false);
    }

    fn in_range(&self, y: i32) -> bool {
        y >= self.min_y as i32 && y < self.min_y as i32 + self.height as i32
    }

    fn get_index(&self, offset_x: i32, y: i32, offset_z: i32) -> usize {
        (offset_x & 0xF | (offset_z & 0xF) << 4 | (y - self.min_y as i32) << 8) as usize
    }

    pub fn set(&mut self, offset_x: i32, y: i32, offset_z: i32) {
        if !self.in_range(y) {
            return;
        }
        self.set_column(offset_x, offset_z);
        let index = self.get_index(offset_x, y, offset_z);
        if let Some(entry) = self.mask.get_mut(index) {
            *entry = true;
        }
    }

    #[must_use]
    pub fn get(&self, offset_x: i32, y: i32, offset_z: i32) -> bool {
        if let Some(mask) = &self.additional_mask {
            if mask(offset_x, y, offset_z) {
                return true;
            }
        }
        if !self.in_range(y) {
            return false;
        }
        let index = self.get_index(offset_x, y, offset_z);
        self.mask.get(index).copied().unwrap_or(false)
    }

    #[must_use]
    pub fn is_masked(&self, offset_x: i32, y: i32, offset_z: i32) -> bool {
        self.get(offset_x, y, offset_z)
    }

    #[must_use]
    pub fn to_long_array(&self) -> Box<[i64]> {
        let longs = (self.mask.len() + 63) / 64;
        let mut words = vec![0u64; longs];
        for (index, masked) in self.mask.iter().enumerate() {
            if *masked {
                let word_index = index >> 6;
                let bit_index = index & 63;
                words[word_index] |= 1u64 << bit_index;
            }
        }
        words.into_iter().map(|value| value as i64).collect()
    }

    pub fn load_long_array(&mut self, data: &[i64]) {
        let words: Vec<u64> = data.iter().map(|value| *value as u64).collect();
        for index in 0..self.mask.len() {
            let word_index = index >> 6;
            let bit_index = index & 63;
            let word = words.get(word_index).copied().unwrap_or(0u64);
            self.mask[index] = ((word >> bit_index) & 1u64) != 0;
        }
    }

    #[must_use]
    pub fn from_long_array(height: u16, min_y: i8, data: &[i64]) -> Self {
        let mut mask = Self::new(height, min_y);
        mask.load_long_array(data);
        mask
    }

    #[must_use]
    pub fn marked_columns(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.column_mask
            .iter()
            .enumerate()
            .filter(|(_, marked)| **marked)
            .map(|(index, _)| ((index & 0xF) as i32, (index >> 4) as i32))
    }

    #[must_use]
    pub fn marked_columns_union(&self, other: &Self) -> Vec<(i32, i32)> {
        let mut marked = [false; 256];
        for (index, (left, right)) in self
            .column_mask
            .iter()
            .zip(other.column_mask.iter())
            .enumerate()
        {
            if *left || *right {
                marked[index] = true;
            }
        }
        self.mark_additional_columns(&mut marked);
        other.mark_additional_columns(&mut marked);

        marked
            .iter()
            .enumerate()
            .filter(|(_, marked)| **marked)
            .map(|(index, _)| ((index & 0xF) as i32, (index >> 4) as i32))
            .collect()
    }

    fn mark_additional_columns(&self, marked: &mut [bool; 256]) {
        let Some(mask) = &self.additional_mask else {
            return;
        };
        let min_y = self.min_y as i32;
        let max_y = min_y + self.height as i32;
        for x in 0..16 {
            for z in 0..16 {
                let index = ((z as usize) << 4) | (x as usize);
                if marked[index] {
                    continue;
                }
                for y in min_y..max_y {
                    if mask(x, y, z) {
                        marked[index] = true;
                        break;
                    }
                }
            }
        }
    }

    fn set_column(&mut self, offset_x: i32, offset_z: i32) {
        let index = ((offset_z & 0xF) << 4 | (offset_x & 0xF)) as usize;
        if let Some(entry) = self.column_mask.get_mut(index) {
            *entry = true;
        }
    }
}
