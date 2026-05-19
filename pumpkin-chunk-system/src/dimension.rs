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
    if dimension == Dimension::OVERWORLD {
    } else if dimension == Dimension::THE_NETHER {
        base_directory.push("DIM-1");
    } else if dimension == Dimension::THE_END {
        base_directory.push("DIM1");
    }
    Level::from_root_folder(level_config, base_directory, seed, dimension, gen_pool)
}
