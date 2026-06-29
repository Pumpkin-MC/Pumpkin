use std::{path::PathBuf, sync::Arc};

use pumpkin_config::world::LevelConfig;
use pumpkin_data::dimension::Dimension;

use crate::level::Level;

#[must_use]
pub fn into_level(
    dimension: Dimension,
    level_config: &LevelConfig,
    mut base_directory: PathBuf,
    seed: i64,
    gen_pool: Option<Arc<rayon::ThreadPool>>,
) -> Arc<Level> {
    if dimension.minecraft_name == Dimension::OVERWORLD.minecraft_name {
    } else if dimension.minecraft_name == Dimension::THE_NETHER.minecraft_name {
        base_directory.push("DIM-1");
    } else if dimension.minecraft_name == Dimension::THE_END.minecraft_name {
        base_directory.push("DIM1");
    }
    Level::from_root_folder(level_config, base_directory, seed, dimension, gen_pool)
}
