use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

use crate::block::BlockStateCodec;

use super::{biome_coords::to_block, height_limit::HeightLimitView, surface::rule::MaterialRule};

pub static GENERATION_SETTINGS: LazyLock<HashMap<GeneratorSetting, GenerationSettings>> =
    LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../assets/chunk_gen_settings.json"))
            .expect("Could not parse chunk_gen_settings.json registry.")
    });

#[derive(Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GeneratorSetting {
    Overworld,
    LargeBiomes,
    Amplified,
    Nether,
    End,
    Caves,
    FloatingIslands,
}
#[derive(Deserialize)]
pub struct GenerationSettings {
    pub legacy_random_source: bool,
    pub sea_level: i32,
    pub noise: GenerationShapeConfig,
    pub surface_rule: MaterialRule,
    pub default_block: BlockStateCodec,
}
#[derive(Deserialize)]
pub struct GenerationShapeConfig {
    pub min_y: i8,
    pub height: u16,
    size_horizontal: u8,
    size_vertical: u8,
}

impl GenerationShapeConfig {
    pub fn vertical_cell_block_count(&self) -> u8 {
        to_block(self.size_vertical)
    }

    pub fn horizontal_cell_block_count(&self) -> u8 {
        to_block(self.size_horizontal)
    }

    pub fn max_y(&self) -> u16 {
        if self.min_y >= 0 {
            self.height + self.min_y as u16
        } else {
            (self.height as i32 + self.min_y as i32) as u16
        }
    }

    pub fn trim_height(&self, limit: &dyn HeightLimitView) -> Self {
        let new_min = self.min_y.max(limit.bottom_y());

        let this_top = if self.min_y >= 0 {
            self.height + self.min_y as u16
        } else {
            self.height - self.min_y.unsigned_abs() as u16
        };

        let new_top = this_top.min(limit.top_y());

        let new_height = if new_min >= 0 {
            new_top - new_min as u16
        } else {
            new_top + new_min.unsigned_abs() as u16
        };

        Self {
            min_y: new_min,
            height: new_height,
            size_horizontal: self.size_horizontal,
            size_vertical: self.size_vertical,
        }
    }
}
