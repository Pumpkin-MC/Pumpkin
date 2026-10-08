use std::{
    any::TypeId,
    sync::{Arc, Weak, atomic::Ordering::Relaxed},
};

use arc_swap::ArcSwap;
use pumpkin_config::world::LevelConfig;
use pumpkin_data::{Block, dimension::Dimension, entity::EntityType};
use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
use pumpkin_world::{chunk::ChunkData, level::Level, world_info::LevelData};
use tempfile::TempDir;

use super::Controls;
use crate::{
    block::registry::default_registry,
    entity::{
        Entity,
        ai::goal::swim::SwimGoal,
        mob::{
            Mob,
            zombie::{drowned::DrownedEntity, zombie::ZombieEntity},
        },
    },
    world::World,
};

async fn water_world() -> std::io::Result<(TempDir, Arc<World>)> {
    let directory = TempDir::new()?;
    let level = Level::from_root_folder(
        &LevelConfig::default(),
        directory.path().to_path_buf(),
        0,
        Dimension::OVERWORLD,
    );
    // This fixture only needs one in-memory chunk, not generation workers.
    level.shutdown().await;
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
    world
        .level_time
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_time(6000);
    world
        .level
        .set_block_state(&BlockPos::new(4, 0, 4), Block::STONE.default_state.id);
    assert!(world.is_bright_outside());
    Ok((directory, world))
}

fn check_float_transitions(name: &str, mob: &dyn Mob, world: &World, floats: bool) {
    let mob_entity = mob.get_mob_entity();
    let living = &mob_entity.living_entity;
    let entity = &living.entity;
    assert!(
        mob_entity
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_none()
    );
    let mut selector = mob_entity
        .goals_selector
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // Isolate JUMP arbitration without replacing the registered goals or their predicates.
    for control in [Controls::MOVE, Controls::LOOK, Controls::TARGET] {
        selector.disable_control(control);
    }

    for (layers, body_wet, eyes_wet) in [
        (0, false, false),
        (1, true, false),
        (2, true, true),
        (0, false, false),
    ] {
        for y in 1..=2 {
            let block = if y <= layers {
                &Block::WATER
            } else {
                &Block::AIR
            };
            world
                .level
                .set_block_state(&BlockPos::new(4, y, 4), block.default_state.id);
        }
        let _ = entity.update_fluid_interaction(mob);
        assert_eq!(entity.is_in_water(), body_wet, "{name}: body, {layers}");
        assert_eq!(
            entity.is_submerged_in_water(),
            eyes_wet,
            "{name}: eyes, {layers}"
        );
        living.jumping.store(false, Relaxed);
        selector.tick(mob);

        // Goal activation is deterministic; SwimGoal::tick has an unrelated 80% jump roll.
        let floating = selector
            .get_goal_by_control(Controls::JUMP)
            .is_some_and(|goal| goal.running && goal.type_id == TypeId::of::<SwimGoal>());
        assert_eq!(
            floating,
            floats && body_wet,
            "{name}: float activation, {layers}"
        );
        if body_wet {
            assert_eq!(
                mob_entity
                    .navigator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .can_float(),
                floats,
                "{name}: navigation floating"
            );
        }
        if !floats {
            assert!(!living.jumping.load(Relaxed), "{name}: unsolicited jump");
        }
    }
}

#[tokio::test]
async fn drowned_does_not_activate_generic_floating_in_water() -> std::io::Result<()> {
    let (_directory, world) = water_world().await?;
    let entity = |entity_type: &'static EntityType| {
        Entity::new(world.clone(), Vector3::new(4.5, 1.0, 4.5), entity_type)
    };
    let cases: [(&str, Arc<dyn Mob>, bool); 5] = [
        (
            "zombie",
            ZombieEntity::new(entity(&EntityType::ZOMBIE)),
            true,
        ),
        (
            "zombie with doors",
            ZombieEntity::with_can_break_doors(entity(&EntityType::ZOMBIE), true),
            true,
        ),
        (
            "drowned",
            DrownedEntity::new(entity(&EntityType::DROWNED)),
            false,
        ),
        (
            "drowned without doors",
            DrownedEntity::with_can_break_doors(entity(&EntityType::DROWNED), false),
            false,
        ),
        (
            "drowned with doors",
            DrownedEntity::with_can_break_doors(entity(&EntityType::DROWNED), true),
            false,
        ),
    ];
    for (name, mob, floats) in cases {
        check_float_transitions(name, mob.as_ref(), &world, floats);
    }
    Ok(())
}
