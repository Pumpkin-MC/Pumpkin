use std::{path::PathBuf, sync::Arc};

use pumpkin_config::world::LevelConfig;
use pumpkin_data::dimension::Dimension;
use tokio::task::JoinSet;

use crate::{level::Level, world::BlockRegistryExt};

pub fn into_level(
    dimension: Dimension,
    level_config: &LevelConfig,
    mut base_directory: PathBuf,
    block_registry: Arc<dyn BlockRegistryExt>,
    seed: i64,
    join_set: &mut JoinSet<()>,
) -> Arc<Level> {
    if dimension == Dimension::OVERWORLD {
    } else if dimension == Dimension::THE_NETHER {
        base_directory.push("DIM-1")
    } else if dimension == Dimension::THE_END {
        base_directory.push("DIM1")
    };
    Level::from_root_folder(
        level_config,
        base_directory,
        block_registry,
        seed,
        dimension,
        join_set,
    )
}
