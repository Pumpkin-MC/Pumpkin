use std::sync::{Arc, Weak};

use arc_swap::ArcSwap;
use pumpkin_config::world::LevelConfig;
use pumpkin_data::{Block, BlockStateId, dimension::Dimension};
use pumpkin_util::{
    math::{position::BlockPos, vector2::Vector2, vector3::Vector3},
    world_seed::Seed,
};
use pumpkin_world::{chunk::ChunkData, level::Level, world_info::LevelData};
use rustc_hash::FxHashSet;
use tempfile::TempDir;

use super::{World, border::Worldborder};
use crate::block::registry::BlockRegistry;

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
        let mut info = LevelData::default(Seed(0));
        info.game_rules.respawn_radius = 0;
        let fixture = Self {
            world: World::load(
                level,
                Arc::new(ArcSwap::from_pointee(info)),
                Dimension::OVERWORLD,
                Arc::new(BlockRegistry::default()),
                Weak::new(),
            ),
            _directory: directory,
        };
        fixture.set_border(0.0, 10.0);
        fixture
    }

    fn set_border(&self, center: f64, diameter: f64) {
        *self.world.worldborder.lock().unwrap() =
            Worldborder::new(center, center, diameter, 0, 5, 300);
    }

    fn set_block(&self, position: BlockPos, state: BlockStateId) {
        self.world
            .level
            .read_chunk_sync(&position.chunk_position(), |chunk| {
                chunk.set_blocks_batch([(
                    (position.0.x & 15) as usize,
                    position.0.y,
                    (position.0.z & 15) as usize,
                    state,
                )]);
            })
            .unwrap();
    }

    async fn shutdown(self) {
        self.world.level.shutdown().await;
    }
}

#[tokio::test]
async fn zero_radius_checks_the_suggested_column() {
    let fixture = SpawnWorld::new();
    assert_eq!(
        fixture.world.get_safe_player_spawn_position(0, 0, 64).await,
        Vector3::new(0.5, 65.0, 0.5),
    );
    fixture.shutdown().await;
}

#[tokio::test]
async fn world_border_limits_radius_but_does_not_reject_spawn_positions() {
    let fixture = SpawnWorld::new();
    let mut loaded_chunks = FxHashSet::default();
    assert_eq!(
        fixture
            .world
            .check_spawn_column(10, 10, &mut loaded_chunks)
            .await,
        Some(BlockPos::new(10, 65, 10)),
    );
    assert_eq!(
        fixture
            .world
            .fixup_spawn_height_at(BlockPos::new(10, 64, 10)),
        BlockPos::new(10, 65, 10),
    );
    for diameter in [0.0, 0.5] {
        fixture.set_border(0.0, diameter);
        let position = fixture.world.get_safe_player_spawn_position(0, 0, 64).await;
        assert!((-0.5..=1.5).contains(&position.x));
        assert!((-0.5..=1.5).contains(&position.z));
        assert_eq!(position.y, 65.0);
    }
    fixture.shutdown().await;
}

#[tokio::test]
async fn collision_checks_include_ground_and_neighbouring_overhangs() {
    let fixture = SpawnWorld::new();
    let feet = BlockPos::new(0, 65, 0);
    assert!(fixture.world.no_collision_no_liquid_at(&feet));

    // A fence below the feet reaches half a block into the player's box.
    fixture.set_block(feet.down(), Block::OAK_FENCE.default_state.id);
    assert!(!fixture.world.no_collision_no_liquid_at(&feet));
    fixture.set_block(feet.down(), Block::STONE.default_state.id);

    // The default piston head's rod extends across its neighbouring block edge.
    let neighbour = feet.add(0, 0, -1);
    fixture.set_block(neighbour, Block::PISTON_HEAD.default_state.id);
    assert!(!fixture.world.no_collision_no_liquid_at(&feet));
    fixture.set_block(neighbour, Block::AIR.default_state.id);
    assert!(fixture.world.no_collision_no_liquid_at(&feet));
    fixture.shutdown().await;
}

#[tokio::test]
async fn fluid_columns_are_rejected_even_when_their_collision_shape_is_empty() {
    let fixture = SpawnWorld::new();
    let feet = BlockPos::new(0, 65, 0);
    let mut loaded_chunks = FxHashSet::default();
    for block in [&Block::SEAGRASS, &Block::KELP_PLANT, &Block::BUBBLE_COLUMN] {
        fixture.set_block(feet, block.default_state.id);
        assert!(
            fixture.world.no_collision_no_liquid_at(&feet),
            "{}",
            block.name
        );
        assert_eq!(
            fixture
                .world
                .check_spawn_column(0, 0, &mut loaded_chunks)
                .await,
            None
        );
    }
    fixture.set_block(feet, Block::WATER.default_state.id);
    assert!(!fixture.world.no_collision_no_liquid_at(&feet));
    fixture.shutdown().await;
}

#[tokio::test]
async fn height_fixup_preserves_vanillas_best_effort_and_lower_boundary() {
    let fixture = SpawnWorld::new();
    assert_eq!(
        fixture.world.fixup_spawn_height_at(BlockPos::new(0, 64, 0)),
        BlockPos::new(0, 65, 0),
    );
    let bottom = fixture.world.get_bottom_y();
    for y in [bottom - 1, bottom] {
        assert_eq!(
            fixture.world.fixup_spawn_height_at(BlockPos::new(0, y, 0)),
            BlockPos::new(0, y, 0),
        );
    }
    let top = fixture.world.get_top_y();
    for y in top - 2..=top {
        fixture.set_block(BlockPos::new(0, y, 0), Block::STONE.default_state.id);
    }
    assert_eq!(
        fixture
            .world
            .fixup_spawn_height_at(BlockPos::new(0, top - 1, 0)),
        BlockPos::new(0, top, 0),
    );
    fixture.shutdown().await;
}

#[tokio::test]
async fn cave_world_uses_generator_spawn_height_instead_of_the_roof() {
    let mut fixture = SpawnWorld::new();
    fixture.world.dimension.has_ceiling = true;
    fixture.set_block(BlockPos::new(0, 100, 0), Block::BEDROCK.default_state.id);
    assert_eq!(
        fixture.world.get_safe_player_spawn_position(0, 0, 64).await,
        Vector3::new(0.5, 65.0, 0.5)
    );
    fixture.shutdown().await;
}

#[tokio::test]
async fn removed_collidable_entities_do_not_block_spawn() {
    use crate::entity::{Entity, EntityBase, RemovalReason, vehicle::boat::BoatEntity};
    use pumpkin_data::entity::EntityType;

    let fixture = SpawnWorld::new();
    let world = Arc::new(fixture.world);
    let feet = BlockPos::new(0, 65, 0);
    let boat = Arc::new(BoatEntity::new(Entity::from_uuid(
        uuid::Uuid::new_v4(),
        world.clone(),
        feet.to_f64(),
        &EntityType::OAK_BOAT,
    )));
    world.entities.store(Arc::new(vec![boat.clone()]));
    assert!(!world.no_collision_no_liquid_at(&feet));
    boat.get_entity()
        .removal_reason
        .store(Some(RemovalReason::Discarded));
    assert!(world.no_collision_no_liquid_at(&feet));
    world.entities.store(Arc::new(Vec::new()));
    world.level.shutdown().await;
}

#[tokio::test]
async fn null_source_spawn_collision_uses_vanilla_entity_filters() {
    use crate::entity::{
        ageable::AgeableMob, passive::happy_ghast::HappyGhastEntity, r#type::from_type,
    };
    use pumpkin_data::entity::EntityType;

    let fixture = SpawnWorld::new();
    let world = Arc::new(fixture.world);
    let feet = BlockPos::new(0, 65, 0);
    for entity_type in [&EntityType::COW, &EntityType::MINECART] {
        let entity = from_type(entity_type, feet.to_f64(), &world, uuid::Uuid::new_v4());
        world.entities.store(Arc::new(vec![entity]));
        assert!(world.no_collision_no_liquid_at(&feet));
    }
    let shulker = from_type(
        &EntityType::SHULKER,
        feet.to_f64(),
        &world,
        uuid::Uuid::new_v4(),
    );
    world.entities.store(Arc::new(vec![shulker.clone()]));
    assert!(!world.no_collision_no_liquid_at(&feet));
    shulker.get_living_entity().unwrap().health.store(0.0);
    assert!(world.no_collision_no_liquid_at(&feet));

    let entity = from_type(
        &EntityType::HAPPY_GHAST,
        feet.to_f64(),
        &world,
        uuid::Uuid::new_v4(),
    );
    let ghast = (entity.as_ref() as &dyn std::any::Any)
        .downcast_ref::<HappyGhastEntity>()
        .unwrap();
    world.entities.store(Arc::new(vec![entity.clone()]));
    assert!(world.no_collision_no_liquid_at(&feet));
    ghast.set_server_still_timeout(20);
    assert!(!world.no_collision_no_liquid_at(&feet));
    ghast.set_baby(true);
    assert!(world.no_collision_no_liquid_at(&feet));
    world.entities.store(Arc::new(Vec::new()));
    world.level.shutdown().await;
}

#[tokio::test]
async fn the_highest_spawn_floor_can_place_feet_above_the_build_limit() {
    let fixture = SpawnWorld::new();
    let top = fixture.world.get_top_y();
    fixture.set_block(BlockPos::new(0, top, 0), Block::STONE.default_state.id);
    assert_eq!(
        fixture
            .world
            .get_safe_player_spawn_position(0, 0, top)
            .await,
        Vector3::new(0.5, f64::from(top + 1), 0.5),
    );
    fixture.shutdown().await;
}

#[tokio::test]
async fn split_collision_shapes_can_form_a_full_spawn_floor() {
    use pumpkin_data::block_properties::{
        Half, HorizontalFacing, OakStairsProperties, StairsShape,
    };

    let fixture = SpawnWorld::new();
    let floor = BlockPos::new(0, 64, 0);
    let mut properties = OakStairsProperties::from_state_id(Block::OAK_STAIRS.default_state.id);
    properties.half = Half::Top;
    let mut loaded_chunks = FxHashSet::default();
    for facing in [
        HorizontalFacing::North,
        HorizontalFacing::South,
        HorizontalFacing::East,
        HorizontalFacing::West,
    ] {
        properties.facing = facing;
        for shape in [
            StairsShape::Straight,
            StairsShape::InnerLeft,
            StairsShape::InnerRight,
            StairsShape::OuterLeft,
            StairsShape::OuterRight,
        ] {
            properties.shape = shape;
            fixture.set_block(floor, properties.to_state_id(&Block::OAK_STAIRS));
            assert_eq!(
                fixture
                    .world
                    .check_spawn_column(0, 0, &mut loaded_chunks)
                    .await,
                Some(BlockPos::new(0, 65, 0))
            );
        }
    }
    properties.half = Half::Bottom;
    fixture.set_block(floor, properties.to_state_id(&Block::OAK_STAIRS));
    assert_eq!(
        fixture
            .world
            .check_spawn_column(0, 0, &mut loaded_chunks)
            .await,
        None
    );
    fixture.shutdown().await;
}

#[tokio::test]
async fn motion_heightmap_uses_the_26_3_tag_not_legacy_sign_solidity() {
    let mut fixture = SpawnWorld::new();
    fixture.set_block(BlockPos::new(0, 63, 0), Block::STONE.default_state.id);
    fixture.set_block(BlockPos::new(0, 64, 0), Block::OAK_SIGN.default_state.id);
    let mut loaded_chunks = FxHashSet::default();
    assert_eq!(
        fixture
            .world
            .check_spawn_column(0, 0, &mut loaded_chunks)
            .await,
        Some(BlockPos::new(0, 64, 0))
    );
    fixture.world.dimension.has_ceiling = true;
    assert_eq!(
        fixture
            .world
            .check_spawn_column(0, 0, &mut loaded_chunks)
            .await,
        None
    );
    fixture.shutdown().await;
}

#[tokio::test]
async fn waterlogged_support_and_partial_top_faces_are_not_spawn_floors() {
    let fixture = SpawnWorld::new();
    let floor = BlockPos::new(0, 64, 0);
    fixture.set_block(
        floor,
        Block::OAK_LEAVES
            .default_state
            .set_waterlogged(true)
            .unwrap()
            .id,
    );
    let mut loaded_chunks = FxHashSet::default();
    assert_eq!(
        fixture
            .world
            .check_spawn_column(0, 0, &mut loaded_chunks)
            .await,
        None
    );
    // A full outline/sturdy face is not necessarily a full collision face.
    let snow = pumpkin_data::block_properties::SnowLikeProperties { layers: 8 };
    fixture.set_block(floor, snow.to_state_id(&Block::SNOW));
    assert_eq!(
        fixture
            .world
            .check_spawn_column(0, 0, &mut loaded_chunks)
            .await,
        None
    );
    fixture.shutdown().await;
}
