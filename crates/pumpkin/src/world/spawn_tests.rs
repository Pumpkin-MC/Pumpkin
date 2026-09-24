use std::sync::{Arc, Weak};

use arc_swap::ArcSwap;
use pumpkin_config::world::LevelConfig;
use pumpkin_data::{Block, dimension::Dimension};
use pumpkin_util::{
    math::{position::BlockPos, vector2::Vector2, vector3::Vector3},
    world_seed::Seed,
};
use pumpkin_world::{chunk::ChunkData, level::Level, world_info::LevelData};
use rustc_hash::FxHashSet;
use tempfile::TempDir;

use super::{World, border::Worldborder};
use crate::block::registry::BlockRegistry;

/// Real chunk and heightmap data, with identical safe ground on both sides of
/// the border so collision checks cannot hide a missing border check.
struct SpawnWorld {
    world: World,
    _directory: TempDir,
}

impl SpawnWorld {
    fn new() -> Self {
        let directory = TempDir::new().unwrap();
        let level = Level::from_root_folder(
            &LevelConfig::default(),
            directory.path().to_path_buf(),
            0,
            Dimension::OVERWORLD,
        );
        for x in -1..=0 {
            for z in -1..=0 {
                let chunk = ChunkData::empty_sync(x, z);
                chunk.set_blocks_batch((0..16).flat_map(|local_x| {
                    (0..16)
                        .map(move |local_z| (local_x, 64, local_z, Block::STONE.default_state.id))
                }));
                level.loaded_chunks.insert(Vector2::new(x, z), chunk);
            }
        }

        let world = World::load(
            level,
            Arc::new(ArcSwap::from_pointee(LevelData::default(Seed(0)))),
            Dimension::OVERWORLD,
            Arc::new(BlockRegistry::default()),
            Weak::new(),
        );
        let fixture = Self {
            world,
            _directory: directory,
        };
        fixture.set_border(0.0, 10.0);
        fixture
    }

    fn set_border(&self, center: f64, diameter: f64) {
        *self.world.worldborder.lock().unwrap() =
            Worldborder::new(center, center, diameter, 0, 5, 300);
    }

    async fn shutdown(self) {
        self.world.level.shutdown().await;
    }
}

#[tokio::test]
async fn in_border_initial_spawn_is_preserved() {
    let fixture = SpawnWorld::new();
    assert_eq!(
        fixture.world.get_safe_player_spawn_position(0, 0, 64).await,
        Some(Vector3::new(0.5, 65.0, 0.5)),
    );
    fixture.shutdown().await;
}

#[tokio::test]
async fn outside_initial_and_ring_columns_are_skipped() {
    let fixture = SpawnWorld::new();
    // The initial column and the first four ring candidates are all safe to
    // stand on, but outside the border. The fifth ring candidate is inside.
    assert_eq!(
        fixture
            .world
            .get_safe_player_spawn_position(-6, 0, 64)
            .await,
        Some(Vector3::new(-4.5, 65.0, -0.5)),
    );
    // A block touching the exclusive upper edge must also be rejected.
    assert_eq!(
        fixture.world.get_safe_player_spawn_position(4, 0, 64).await,
        Some(Vector3::new(3.5, 65.0, -0.5)),
    );
    fixture.shutdown().await;
}

#[tokio::test]
async fn outside_height_fixup_is_rejected() {
    let fixture = SpawnWorld::new();
    assert_eq!(
        fixture.world.fixup_spawn_height_at(BlockPos::new(0, 64, 0)),
        Some(BlockPos::new(0, 65, 0)),
    );
    assert!(
        fixture
            .world
            .no_collision_no_liquid_at(&BlockPos::new(4, 65, 0))
    );
    assert_eq!(
        fixture.world.fixup_spawn_height_at(BlockPos::new(4, 64, 0)),
        None,
    );
    fixture.shutdown().await;
}

#[tokio::test]
async fn no_spawn_is_returned_when_every_candidate_is_outside() {
    let fixture = SpawnWorld::new();
    let mut loaded_chunks = FxHashSet::default();
    assert_eq!(
        fixture
            .world
            .check_spawn_column(100, 100, &mut loaded_chunks)
            .await,
        None,
    );
    assert!(
        loaded_chunks.is_empty(),
        "rejected columns must not be fetched"
    );
    assert_eq!(
        fixture
            .world
            .get_safe_player_spawn_position(100, 100, 64)
            .await,
        None,
    );
    assert_eq!(fixture.world.level.loaded_chunk_count(), 4);

    // These borders contain no whole block, even at the stored spawn.
    for diameter in [0.0, 0.5] {
        fixture.set_border(0.0, diameter);
        assert_eq!(
            fixture.world.get_safe_player_spawn_position(0, 0, 64).await,
            None,
        );
    }
    fixture.shutdown().await;
}

#[tokio::test]
async fn fractional_border_edges_require_the_whole_spawn_block() {
    let fixture = SpawnWorld::new();
    fixture.set_border(0.25, 10.0);
    assert_eq!(
        fixture
            .world
            .get_safe_player_spawn_position(-4, 0, 64)
            .await,
        Some(Vector3::new(-3.5, 65.0, 0.5)),
    );
    for (spawn_x, expected_x) in [(-5, -3.5), (5, 4.5)] {
        assert_eq!(
            fixture
                .world
                .get_safe_player_spawn_position(spawn_x, 0, 64)
                .await,
            Some(Vector3::new(expected_x, 65.0, -0.5)),
        );
    }
    fixture.shutdown().await;
}
