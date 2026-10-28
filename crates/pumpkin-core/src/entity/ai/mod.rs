pub mod behavior;
pub mod brain;
pub mod control;
pub mod goal;
pub mod pathfinder;
pub mod random_pos;
pub mod sensing;
pub mod target_predicate;
pub mod util;

#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::{Arc, Weak};

    use arc_swap::ArcSwap;
    use pumpkin_config::world::LevelConfig;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::world_seed::Seed;
    use pumpkin_world::{level::Level, world_info::LevelData};

    use crate::{block::registry::default_registry, world::World};

    pub fn world(folder: &tempfile::TempDir) -> Arc<World> {
        world_in_dimension(folder, Dimension::OVERWORLD.clone())
    }

    pub fn world_in_dimension(folder: &tempfile::TempDir, dimension: Dimension) -> Arc<World> {
        Arc::new(World::load(
            Level::from_root_folder(
                &LevelConfig::default(),
                folder.path().to_path_buf(),
                0,
                dimension.clone(),
            ),
            Arc::new(ArcSwap::from_pointee(LevelData::default(Seed(0)))),
            dimension,
            default_registry(),
            Weak::new(),
        ))
    }
}
