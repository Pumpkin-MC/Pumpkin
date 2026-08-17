use std::{path::PathBuf, sync::Arc};

use pumpkin_config::world::LevelConfig;
use pumpkin_util::identifier::Identifier;

use crate::{generation::generator::ChunkGenerator, level::Level};

#[must_use]
pub fn into_level(
    world_key: Identifier,
    generator: Arc<dyn ChunkGenerator>,
    level_config: &LevelConfig,
    base_directory: PathBuf,
    gen_pool: Option<Arc<rayon::ThreadPool>>,
) -> Arc<Level> {
    Level::from_root_folder(level_config, base_directory, world_key, generator, gen_pool)
}
