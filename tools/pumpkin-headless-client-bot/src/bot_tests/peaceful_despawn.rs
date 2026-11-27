use std::sync::{Arc, atomic::Ordering::Relaxed};

use pumpkin_core::{
    entity::{EntityBase, RemovalReason, r#type::from_type, vehicle::minecart::MinecartEntity},
    server::Server,
    world::{BlockFlags, World},
};
use pumpkin_data::{
    Block,
    block_properties::{PoweredRailLikeProperties, RailShapeStraight},
    entity::EntityType,
};
use pumpkin_util::{Difficulty, math::position::BlockPos};
use uuid::Uuid;

use crate::bot::HeadlessBot;

pub fn run_peaceful_despawn_test(
    server: &Arc<Server>,
    bot: &HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let player = server
        .get_player_by_name(&bot.name)
        .ok_or("Missing despawn test player")?;
    let world = player.world();
    let position = player.position().add_raw(0.0, 8.0, 0.0);
    let fixtures = [
        (&EntityType::ZOMBIE, false, false),
        (&EntityType::ZOMBIE, true, false),
        (&EntityType::PIGLIN, true, true),
    ];
    let mut mobs = Vec::new();
    for (index, (entity_type, persistent, survives)) in fixtures.into_iter().enumerate() {
        let mob = from_type(
            entity_type,
            position.add_raw(0.0, index as f64 * 2.0, 0.0),
            &world,
            Uuid::new_v4(),
        );
        let behavior = mob
            .get_mob()
            .ok_or("Despawn fixture must be a mob")?
            .get_mob_entity();
        behavior.set_no_ai(true);
        if persistent {
            behavior.set_persistence_required();
        }
        mob.get_entity().set_has_no_gravity(true);
        if !world.spawn_entity(mob.clone()) {
            return Err("Could not spawn a Peaceful despawn fixture".into());
        }
        mobs.push((mob, persistent, survives));
    }

    let original_difficulty = server.get_difficulty();
    let tick_rate = &server.tick_rate_manager;
    let originally_frozen = tick_rate.is_frozen();
    let tick = || {
        tick_rate.tick();
        server.tick();
    };
    let verify = |expected_ticks, peaceful_tick| {
        for (mob, persistent, survives) in &mobs {
            let entity = mob.get_entity();
            let alive = world.get_entity_by_id(entity.entity_id).is_some();
            let discarded = entity.removal_reason.load() == Some(RemovalReason::Discarded);
            let should_discard = peaceful_tick && !survives;
            if alive == should_discard
                || discarded != should_discard
                || (alive && entity.tick_count.load(Relaxed) != expected_ticks)
            {
                return Err(format!(
                    "Unexpected mob state after {expected_ticks} tick(s): type {}, persistent {persistent}, alive {alive}, discarded {discarded}",
                    entity.entity_type.resource_name
                ));
            }
        }
        Ok(())
    };
    let result = (|| {
        server.set_difficulty(Difficulty::Normal, true);
        tick_rate.set_frozen(server, false);
        tick();
        verify(1, false)?;

        tick_rate.set_frozen(server, true);
        server.set_difficulty(Difficulty::Peaceful, true);
        tick();
        verify(1, false)?;

        if !tick_rate.step_game_if_paused(server, 1) {
            return Err("Could not step the frozen Peaceful world".into());
        }
        tick();
        verify(2, true)
    })();
    server.set_difficulty(original_difficulty, true);
    tick_rate.set_frozen(server, originally_frozen);
    tick_rate.tick();
    for (mob, _, _) in mobs {
        world.remove_entity(mob.as_ref());
    }
    result?;
    summary_lines.push("[PASS] bot:peaceful_mob_despawn_after_frozen_switch".into());
    Ok(())
}

pub fn run_mid_tick_difficulty_test(
    server: &Arc<Server>,
    bot: &HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let player = server
        .get_player_by_name(&bot.name)
        .ok_or("Missing difficulty test player")?;
    let world = player.world();
    let rail = player.position().add_raw(0.0, 8.0, 0.0).to_block_pos();
    let floor = rail.down();
    let old_rail = world.get_block_state(&rail).id;
    let old_floor = world.get_block_state(&floor).id;
    let difficulty = server.get_difficulty();
    let tick_rate = &server.tick_rate_manager;
    let frozen = tick_rate.is_frozen();
    let mut fixtures = Vec::new();
    let mut props = PoweredRailLikeProperties::default(&Block::ACTIVATOR_RAIL);
    props.shape = RailShapeStraight::EastWest;
    props.powered = true;
    world.set_block_state(
        &floor,
        Block::REDSTONE_BLOCK.default_state.id,
        BlockFlags::NOTIFY_ALL,
    );
    world.set_block_state(
        &rail,
        props.to_state_id(&Block::ACTIVATOR_RAIL),
        BlockFlags::NOTIFY_ALL,
    );

    let result = (|| {
        // Two separated groups make island planning possible without affecting the cart or mob.
        for height in [-112.0, 112.0] {
            for _ in 0..8 {
                let marker = from_type(
                    &EntityType::MARKER,
                    rail.to_f64().add_raw(0.0, height, 0.0),
                    &world,
                    Uuid::new_v4(),
                );
                if !world.spawn_entity(marker.clone()) {
                    return Err("Could not spawn island fixture".into());
                }
                fixtures.push(marker);
            }
        }

        tick_rate.set_frozen(server, false);
        check_mid_tick_transitions(server, &world, rail, &mut fixtures)
    })();
    for entity in fixtures {
        world.remove_entity(entity.as_ref());
    }
    world.set_block_state(&rail, old_rail, BlockFlags::NOTIFY_ALL);
    world.set_block_state(&floor, old_floor, BlockFlags::NOTIFY_ALL);
    server.set_difficulty(difficulty, true);
    tick_rate.set_frozen(server, frozen);
    tick_rate.tick();
    result?;
    summary_lines.push("[PASS] bot:peaceful_despawn_uses_mid_tick_difficulty".into());
    Ok(())
}

fn check_mid_tick_transitions(
    server: &Arc<Server>,
    world: &Arc<World>,
    rail: BlockPos,
    fixtures: &mut Vec<Arc<dyn EntityBase>>,
) -> Result<(), String> {
    let marker_count = fixtures.len();
    let tick = || {
        server.tick_rate_manager.tick();
        server.tick();
    };
    let mut failures = Vec::new();
    for (before, after) in [
        (Difficulty::Peaceful, Difficulty::Normal),
        (Difficulty::Normal, Difficulty::Peaceful),
    ] {
        server.set_difficulty(before, true);
        let cart = from_type(
            &EntityType::COMMAND_BLOCK_MINECART,
            rail.to_f64(),
            world,
            Uuid::new_v4(),
        );
        let command_block = cart
            .cast_any()
            .downcast_ref::<MinecartEntity>()
            .and_then(MinecartEntity::command_block)
            .ok_or("Missing minecart command block")?;
        command_block.set_command(format!("difficulty {}", after.name()));
        if !world.spawn_entity(cart.clone()) {
            return Err("Could not spawn command block minecart".into());
        }
        fixtures.push(cart.clone());
        for _ in 0..3 {
            tick();
        }
        if world.level_info.load().difficulty != before
            || !fixtures[..marker_count]
                .iter()
                .all(|marker| marker.get_entity().tick_count.load(Relaxed) >= 3)
        {
            return Err("Cart ran early or island fixtures did not tick".into());
        }

        let zombie = from_type(
            &EntityType::ZOMBIE,
            rail.to_f64().add_raw(0.0, 4.0, 0.0),
            world,
            Uuid::new_v4(),
        );
        let mob = zombie
            .get_mob()
            .ok_or("Expected a zombie mob")?
            .get_mob_entity();
        mob.set_no_ai(true);
        mob.set_persistence_required();
        zombie.get_entity().set_has_no_gravity(true);
        if !world.spawn_entity(zombie.clone()) {
            return Err("Could not spawn difficulty test zombie".into());
        }
        fixtures.push(zombie.clone());
        tick();
        if world.level_info.load().difficulty != after {
            return Err(format!("Minecart did not set difficulty to {after:?}"));
        }
        let entity = zombie.get_entity();
        let alive = world.get_entity_by_id(entity.entity_id).is_some();
        let discarded = entity.removal_reason.load() == Some(RemovalReason::Discarded);
        if alive != (after == Difficulty::Normal)
            || discarded != (after == Difficulty::Peaceful)
            || (alive && entity.tick_count.load(Relaxed) != 1)
        {
            failures.push(format!(
                "{before:?} to {after:?}: zombie alive {alive}, discarded {discarded}, ticks {}",
                entity.tick_count.load(Relaxed)
            ));
        }
        world.remove_entity(zombie.as_ref());
        world.remove_entity(cart.as_ref());
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
