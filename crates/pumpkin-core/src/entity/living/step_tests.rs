use std::sync::{Arc, Weak, atomic::Ordering::Relaxed};

use arc_swap::ArcSwap;
use pumpkin_config::world::LevelConfig;
use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{RedstoneOreLikeProperties, SnowLikeProperties},
    dimension::Dimension,
    entity::EntityType,
};
use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
use pumpkin_world::{chunk::ChunkData, level::Level, world_info::LevelData};
use tempfile::TempDir;
use uuid::Uuid;

use super::LivingEntity;
use crate::{block::registry::default_registry, entity::Entity, world::World};

fn test_world() -> std::io::Result<(TempDir, Arc<World>)> {
    let directory = TempDir::new()?;
    let level = Level::from_root_folder(
        &LevelConfig::default(),
        directory.path().to_path_buf(),
        0,
        Dimension::OVERWORLD,
    );
    level
        .loaded_chunks
        .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
    let info = Arc::new(ArcSwap::from_pointee(LevelData::default(level.seed)));
    let world = Arc::new(World::load(
        level,
        info,
        Dimension::OVERWORLD,
        default_registry(),
        Weak::new(),
    ));
    Ok((directory, world))
}

fn snow(layers: u8) -> BlockStateId {
    SnowLikeProperties { layers }.to_state_id(&Block::SNOW)
}

fn entity_at(world: &Arc<World>, pos: Vector3<f64>, grounded: bool) -> LivingEntity {
    let living = LivingEntity::new(Entity::from_uuid(
        Uuid::new_v4(),
        world.clone(),
        pos,
        &EntityType::PLAYER,
    ));
    living.entity.on_ground.store(grounded, Relaxed);
    // Exercise the player's contact lookup without a network client or server.
    let supporting = living.entity.find_supporting_block_pos(&living);
    living.entity.supporting_block_pos.store(supporting);
    living
}

#[tokio::test]
async fn elevated_support_does_not_reach_snow_covered_magma() -> std::io::Result<()> {
    let (_directory, world) = test_world()?;
    let support = BlockPos::new(4, 1, 4);
    world
        .level
        .set_block_state(&support, Block::STONE.default_state.id);
    world
        .level
        .set_block_state(&BlockPos::new(5, 0, 4), Block::MAGMA_BLOCK.default_state.id);
    world
        .level
        .set_block_state(&BlockPos::new(5, 1, 4), snow(1));
    // The centre is over the snow, but only the adjacent stone touches the feet.
    let living = entity_at(&world, Vector3::new(5.05, 2.0, 4.5), true);
    let supporting = living.entity.supporting_block_pos.load();
    living.tick_block_step(&living);
    let damage = living.last_damage_taken.load();
    // Just beyond the stone edge there is no collision support at this height.
    let beyond_edge = entity_at(&world, Vector3::new(5.31, 2.0, 4.5), true);
    let beyond_support = beyond_edge.entity.supporting_block_pos.load();
    world.level.shutdown().await;

    assert_eq!(supporting, Some(support));
    assert_eq!(damage, 0.0);
    assert_eq!(beyond_support, None);
    Ok(())
}

#[tokio::test]
async fn magma_step_damage_respects_cover_height() -> std::io::Result<()> {
    let (_directory, world) = test_world()?;
    let mut results = Vec::new();
    for base_y in [0, -2] {
        let magma = BlockPos::new(8, base_y, 8);
        world
            .level
            .set_block_state(&magma, Block::MAGMA_BLOCK.default_state.id);
        for (cover, y, grounded) in [
            (Block::AIR.default_state.id, 1.0, true),
            (snow(1), 1.0, true),
            (snow(2), 1.125, true),
            (snow(3), 1.25, true),
            (snow(8), 1.875, true),
            (Block::WHITE_CARPET.default_state.id, 1.0625, true),
            (snow(2), 1.125, false),
            (snow(3), 1.200_000_001, true),
            (snow(3), 1.200_000_01, true),
        ] {
            world.level.set_block_state(&magma.up(), cover);
            let living = entity_at(
                &world,
                Vector3::new(8.5, f64::from(base_y) + y, 8.5),
                grounded,
            );
            living.tick_block_step(&living);
            // Damage metadata is recorded before the server-dependent animation/health update.
            results.push(living.last_damage_taken.load());
        }
    }
    world.level.shutdown().await;

    // Thin snow and carpet intentionally still expose magma; thicker snow does not.
    let expected = [1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0];
    assert_eq!(results, expected.repeat(2));
    Ok(())
}

#[tokio::test]
async fn legacy_step_height_also_controls_redstone_ore() -> std::io::Result<()> {
    let (_directory, world) = test_world()?;
    let ore = BlockPos::new(8, 0, 8);
    let mut results = Vec::new();
    for (cover, y) in [
        (Block::AIR.default_state.id, 1.0),
        (snow(2), 1.125),
        (snow(3), 1.25),
    ] {
        world
            .level
            .set_block_state(&ore, Block::REDSTONE_ORE.default_state.id);
        world.level.set_block_state(&ore.up(), cover);
        let living = entity_at(&world, Vector3::new(8.5, y, 8.5), true);
        living.tick_block_step(&living);
        let state = world.level.get_block_state(&ore);
        results.push(RedstoneOreLikeProperties::from_state_id(state).lit);
    }
    world.level.shutdown().await;

    assert_eq!(results, [true, true, false]);
    Ok(())
}

#[tokio::test]
async fn tall_support_keeps_fence_wall_and_gate_as_step_recipient() -> std::io::Result<()> {
    let (_directory, world) = test_world()?;
    let pos = BlockPos::new(8, 0, 8);
    let mut supports = Vec::new();
    let mut recipients = Vec::new();
    for block in [
        &Block::OAK_FENCE,
        &Block::COBBLESTONE_WALL,
        &Block::OAK_FENCE_GATE,
    ] {
        world.level.set_block_state(&pos, block.default_state.id);
        let living = entity_at(&world, Vector3::new(8.5, 1.5, 8.5), true);
        let support = living.entity.supporting_block_pos.load();
        supports.push(support);
        recipients.push(living.entity.get_on_pos_legacy(support));
    }
    world.level.shutdown().await;

    assert_eq!(supports, [Some(pos); 3]);
    assert_eq!(recipients, [pos; 3]);
    Ok(())
}
