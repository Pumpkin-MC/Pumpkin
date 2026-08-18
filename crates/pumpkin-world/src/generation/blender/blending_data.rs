use pumpkin_data::chunk::Biome;

#[derive(Clone)]
pub struct BlendingData {
    pub min_y: i32,
    pub max_y: i32,
    // Heights at quart positions (16x16 per chunk)
    pub heights: Vec<f64>,
    // Densities at quart positions (16x16 per chunk, but at what Y levels? 1.18+ uses 4 blocks quart)
    // Simplified: only store for current chunk's Y levels
    pub densities: Vec<f64>,
    // Biomes at quart positions (16x16 per chunk)
    pub biomes: Vec<&'static Biome>,
}

impl BlendingData {
    #[must_use]
    pub fn get_height(&self, cell_x: i32, _cell_y: i32, cell_z: i32) -> f64 {
        if !(0..16).contains(&cell_x) || !(0..16).contains(&cell_z) {
            return f64::MAX;
        }
        self.heights[(cell_z * 16 + cell_x) as usize]
    }

    #[must_use]
    pub fn get_density(&self, cell_x: i32, cell_y: i32, cell_z: i32) -> f64 {
        if !(0..16).contains(&cell_x) || !(0..16).contains(&cell_z) {
            return f64::MAX;
        }
        let cell_count_y = (self.max_y - self.min_y) / 8;
        if !(0..cell_count_y).contains(&cell_y) {
            return f64::MAX;
        }
        let index = (cell_y * 16 * 16 + cell_z * 16 + cell_x) as usize;
        if index < self.densities.len() {
            self.densities[index]
        } else {
            f64::MAX
        }
    }

    pub fn iterate_heights<F>(&self, quart_x: i32, quart_z: i32, mut consumer: F)
    where
        F: FnMut(i32, i32, f64),
    {
        for z in 0..16 {
            for x in 0..16 {
                let h = self.heights[z * 16 + x];
                if h != f64::MAX {
                    consumer(quart_x + x as i32, quart_z + z as i32, h);
                }
            }
        }
    }

    pub fn iterate_densities<F>(
        &self,
        quart_x: i32,
        quart_z: i32,
        min_cell_y: i32,
        max_cell_y: i32,
        mut consumer: F,
    ) where
        F: FnMut(i32, i32, i32, f64),
    {
        let cell_count_y = (self.max_y - self.min_y) / 8;
        for cell_y in min_cell_y..=max_cell_y {
            if (0..cell_count_y).contains(&cell_y) {
                for z in 0..16 {
                    for x in 0..16 {
                        let index = (cell_y * 16 * 16 + z * 16 + x) as usize;
                        if index < self.densities.len() {
                            let d = self.densities[index];
                            if d != f64::MAX {
                                consumer(quart_x + x, cell_y, quart_z + z, d);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn iterate_biomes<F>(&self, quart_x: i32, _quart_y: i32, quart_z: i32, mut consumer: F)
    where
        F: FnMut(i32, i32, &'static Biome),
    {
        for z in 0..16 {
            for x in 0..16 {
                let biome = self.biomes[z * 16 + x];
                consumer(quart_x + x as i32, quart_z + z as i32, biome);
            }
        }
    }
}
