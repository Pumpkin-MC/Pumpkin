use std::{path::PathBuf, sync::Arc};

use pumpkin_config::world::LevelConfig;
use serde::Deserialize;

use crate::{level::Level, world::BlockRegistryExt};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    Overworld,
    Nether,
    End,
}

impl Dimension {
    pub fn into_level(
        &self,
        level_config: &LevelConfig,
        mut base_directory: PathBuf,
        block_registry: Arc<dyn BlockRegistryExt>,
        seed: i64,
    ) -> Arc<Level> {
        match self {
            Dimension::Overworld => {}
            Dimension::Nether => base_directory.push("DIM-1"),
            Dimension::End => base_directory.push("DIM1"),
        }
        Level::from_root_folder(level_config, base_directory, block_registry, seed, *self)
    }
}
