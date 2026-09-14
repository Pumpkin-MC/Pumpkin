use std::{
    path::Path,
    sync::{Arc, Weak},
};

use arc_swap::ArcSwap;
use pumpkin::{block::registry::BlockRegistry, world::World};
use pumpkin_config::world::LevelConfig;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;
use pumpkin_world::{level::Level, world_info::LevelData};

pub fn world(path: &Path) -> Arc<World> {
    let level = Level::from_root_folder(
        &LevelConfig::default(),
        path.to_path_buf(),
        0,
        Dimension::OVERWORLD,
    );
    Arc::new(World::load(
        level,
        Arc::new(ArcSwap::from_pointee(LevelData::default(Seed(0)))),
        Dimension::OVERWORLD,
        Arc::new(BlockRegistry::default()),
        Weak::new(),
    ))
}
