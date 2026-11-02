use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use pumpkin_data::dimension::Dimension;

use crate::world::World;
use crate::world::wandering_trader_spawner::WanderingTraderSpawner;

/// Vanilla `CustomSpawner`: a spawner ticked once per level after the chunk tick.
pub trait CustomSpawner: Send + std::any::Any {
    fn tick(&mut self, world: &Arc<World>, spawn_enemies: bool);
}

/// Vanilla `MinecraftServer.createLevels`: only the overworld gets custom spawners.
pub(crate) fn create_custom_spawners(
    dimension: &Dimension,
    data_dir: PathBuf,
) -> Vec<Mutex<Box<dyn CustomSpawner + Send>>> {
    let mut spawners: Vec<Mutex<Box<dyn CustomSpawner + Send>>> = Vec::new();
    if dimension == &Dimension::OVERWORLD {
        let spawner: Box<dyn CustomSpawner + Send> =
            Box::new(WanderingTraderSpawner::new(data_dir));
        spawners.push(Mutex::new(spawner));
    }
    spawners
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::world::wandering_trader_spawner::WanderingTraderSpawner;
    use std::any::TypeId;

    fn contains_wandering_trader(spawner: &Mutex<Box<dyn CustomSpawner + Send>>) -> bool {
        let guard = spawner.lock().unwrap();
        guard.as_ref().type_id() == TypeId::of::<WanderingTraderSpawner>()
    }

    #[test]
    fn only_the_overworld_gets_custom_spawners() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let overworld =
            create_custom_spawners(&Dimension::OVERWORLD, temp_dir.path().to_path_buf());

        assert!(overworld.iter().any(contains_wandering_trader));

        for dimension in [Dimension::THE_NETHER, Dimension::THE_END] {
            assert!(create_custom_spawners(&dimension, temp_dir.path().to_path_buf()).is_empty());
        }
    }
}
