use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize},
};

use bytes::Buf;
use pumpkin_core::{
    block::entities::block_entity_from_nbt,
    entity::{Entity, EntityBase, decoration::end_crystal::EndCrystalEntity, player::Player},
    plugin::{
        BoxFuture, EventPriority, EventRwHandler,
        api::events::{
            block::block_explode::BlockExplodeEvent,
            entity::entity_knockback::EntityKnockbackEvent,
            player::player_velocity::PlayerVelocityEvent,
        },
    },
    server::Server,
    world::{BlockFlags, ExplosionInteraction, World},
};
use pumpkin_data::{Block, damage::DamageType, entity::EntityType, packet::CURRENT_MC_VERSION};
use pumpkin_protocol::{
    codec::{lp_vector_3d::LpVector3d, var_int::VarInt},
    java::client::play::{CEntityVelocity, CExplosion},
    packet::MultiVersionJavaPacket,
};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::bot::HeadlessBot;

const STANDING_KNOCKBACK: Vector3<f64> =
    Vector3::new(-0.48149676911700806, 0.7800247682655086, 0.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AbsoluteMotionTiming {
    EntityKnockback,
    BlockExplode,
    PlayerVelocity,
}

struct ZeroVelocityOnBlockExplode {
    calls: Arc<AtomicUsize>,
    enabled: Arc<AtomicBool>,
}

struct ZeroVelocityOnEntityKnockback {
    calls: Arc<AtomicUsize>,
    enabled: Arc<AtomicBool>,
    player_id: i32,
}

struct ZeroVelocityOnPlayerVelocity {
    calls: Arc<AtomicUsize>,
    enabled: Arc<AtomicBool>,
    player_id: i32,
}

fn zero_velocity(server: &Arc<Server>, calls: &AtomicUsize, enabled: &AtomicBool) {
    if !enabled.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    if let Some(player) = server.get_player_by_name("TestBot2") {
        player.set_velocity(Vector3::default());
    }
}

impl EventRwHandler<BlockExplodeEvent> for ZeroVelocityOnBlockExplode {
    fn handle<'a>(
        &'a self,
        server: &'a Arc<Server>,
        _event: &'a mut BlockExplodeEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move { zero_velocity(server, &self.calls, &self.enabled) })
    }
}

impl EventRwHandler<EntityKnockbackEvent> for ZeroVelocityOnEntityKnockback {
    fn handle<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a mut EntityKnockbackEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.player_id {
                zero_velocity(server, &self.calls, &self.enabled);
            }
        })
    }
}

impl EventRwHandler<PlayerVelocityEvent> for ZeroVelocityOnPlayerVelocity {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerVelocityEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.player.entity_id() == self.player_id
                && self.enabled.load(std::sync::atomic::Ordering::SeqCst)
                && self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0
            {
                event.player.set_velocity(Vector3::default());
            }
        })
    }
}

pub async fn run_explosion_knockback_test(
    server: &Arc<Server>,
    bot1: &mut HeadlessBot,
    bot2: &mut HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let player = server
        .get_player_by_name("TestBot1")
        .ok_or("Missing TestBot1")?;
    let world = player.world();
    let base = BlockPos::new(
        player.position().x.floor() as i32,
        player.position().y.floor() as i32 - 1,
        player.position().z.floor() as i32,
    );
    let center = Vector3::new(
        f64::from(base.0.x) + 0.5,
        f64::from(base.0.y + 1),
        f64::from(base.0.z) + 0.5,
    );

    let mut previous_blocks = Vec::new();
    for x in base.0.x - 2..=base.0.x + 2 {
        for y in base.0.y - 1..=base.0.y + 3 {
            for z in base.0.z - 2..=base.0.z + 2 {
                let pos = BlockPos::new(x, y, z);
                previous_blocks.push((
                    pos,
                    world.get_block_state(&pos).id,
                    world.get_block_entity_nbt(&pos),
                ));
                world.set_block_state(&pos, Block::AIR.default_state.id, BlockFlags::NOTIFY_ALL);
            }
        }
    }
    world.set_block_state(
        &base,
        Block::OBSIDIAN.default_state.id,
        BlockFlags::NOTIFY_ALL,
    );

    let health_states: Vec<_> = server
        .get_all_players()
        .into_iter()
        .map(|other| {
            let max_health = other.living_entity.get_max_health();
            let health = other.living_entity.health.load();
            other.set_max_health(200.0);
            other.set_health(200.0);
            (other, max_health, health)
        })
        .collect();
    let bot2_original_position = bot2.position;
    let result: Result<(), String> = async {
        let standing = center.add_raw(-1.0, 0.0, 0.0);
        bot1.move_to(standing, true).await?;
        server.tick();
        // Expected from vanilla geometry: one block horizontally away and eye height 1.62.
        detonate_crystal(server, bot1, &player, &world, center, STANDING_KNOCKBACK).await?;

        let jumping = center.add_raw(-1.0, 0.25, 0.0);
        bot1.move_to(jumping, false).await?;
        server.tick();
        detonate_crystal(
            server,
            bot1,
            &player,
            &world,
            center,
            Vector3::new(-0.43106030120475536, 0.8060827632528925, 0.0),
        )
        .await?;

        check_distant_explosion_knockback(server, bot2, &world, center, bot2_original_position)
            .await
    }
    .await;

    for (other, max_health, health) in health_states {
        other.set_max_health(max_health);
        other.set_health(health);
    }
    for (pos, state, block_entity_nbt) in previous_blocks {
        world.set_block_state(&pos, state, BlockFlags::NOTIFY_ALL);
        if let Some(nbt) = block_entity_nbt {
            let entity = block_entity_from_nbt(&nbt)
                .ok_or_else(|| format!("Could not restore the block entity at {pos:?}"))?;
            world.remove_block_entity(&pos);
            world.add_block_entity(entity);
            world.add_block_entity_nbt(pos, &nbt);
        }
    }
    result?;
    summary_lines.push("[PASS] bot:explosion_player_knockback".into());
    Ok(())
}

async fn check_distant_explosion_knockback(
    server: &Arc<Server>,
    bot: &mut HeadlessBot,
    world: &Arc<World>,
    center: Vector3<f64>,
    original_position: Vector3<f64>,
) -> Result<(), String> {
    bot.move_to(center.add_raw(0.0, 20.0, 0.0), false).await?;
    server.tick();
    // Consume every earlier packet through a marker that cannot hurt either bot.
    let marker_center = center.add_raw(0.0, 17.0, 0.0);
    world.explode(marker_center, 1.0, ExplosionInteraction::None);
    loop {
        let (packet_center, radius, knockback) = next_explosion_packet(bot).await?;
        if packet_center == marker_center {
            if radius != 1.0 || knockback.is_some() {
                return Err("Invalid explosion packet marker".into());
            }
            break;
        }
    }

    let player = server
        .get_player_by_name("TestBot2")
        .ok_or("Missing TestBot2")?;
    let entity = player.get_entity();
    entity.velocity.store(Vector3::default());
    entity.last_sent_velocity.store(Vector3::default());
    entity
        .sync_velocity
        .store(false, std::sync::atomic::Ordering::SeqCst);
    world.explode(
        center.add_raw(65.0, 20.0, 0.0),
        33.0,
        ExplosionInteraction::None,
    );
    let knockback = next_own_motion(bot, player.entity_id()).await?;
    // Feet give 1/66 power and the eye position gives its direction; the wire quantizes to this.
    let expected = Vector3::new(-0.015_137_642_678_386_132, 0.000_366_233_290_606_032_8, 0.0);
    if knockback != expected {
        return Err(format!(
            "Distant explosion knockback mismatch: expected {expected:?}, got {knockback:?}"
        ));
    }

    let trailing_center = center.add_raw(0.0, 16.0, 0.0);
    world.explode(trailing_center, 1.0, ExplosionInteraction::None);
    let (packet_center, _, _) = next_explosion_packet(bot).await?;
    if packet_center != trailing_center {
        return Err("Distant player also received CExplosion after its motion".into());
    }

    for timing in [
        AbsoluteMotionTiming::BlockExplode,
        AbsoluteMotionTiming::EntityKnockback,
        AbsoluteMotionTiming::PlayerVelocity,
    ] {
        check_absolute_motion_supersedes_knockback(server, bot, world, center, timing)
            .await
            .map_err(|error| format!("{timing:?} scenario: {error}"))?;
    }

    bot.move_to(original_position, true).await?;
    server.tick();
    Ok(())
}

async fn check_absolute_motion_supersedes_knockback(
    server: &Arc<Server>,
    bot: &mut HeadlessBot,
    world: &Arc<World>,
    center: Vector3<f64>,
    timing: AbsoluteMotionTiming,
) -> Result<(), String> {
    bot.move_to(center.add_raw(-1.0, 0.0, 0.0), true).await?;
    server.tick();

    let marker_center = center.add_raw(0.0, 17.0, 0.0);
    world.explode(marker_center, 1.0, ExplosionInteraction::None);
    loop {
        let (packet_center, _, _) = next_explosion_packet(bot).await?;
        if packet_center == marker_center {
            break;
        }
    }

    let player = server
        .get_player_by_name("TestBot2")
        .ok_or("Missing TestBot2")?;
    let entity = player.get_entity();
    entity.velocity.store(Vector3::default());
    entity.last_sent_velocity.store(Vector3::default());
    entity
        .sync_velocity
        .store(false, std::sync::atomic::Ordering::SeqCst);

    let (calls, enabled, interaction) = register_zero_velocity_handler(server, &player, timing);
    world.explode(center, 6.0, interaction);
    enabled.store(false, std::sync::atomic::Ordering::SeqCst);

    let motion = next_own_motion(bot, player.entity_id()).await?;
    if motion != Vector3::default() {
        return Err(format!(
            "The listener sent an unexpected absolute motion: {motion:?}"
        ));
    }
    if timing == AbsoluteMotionTiming::PlayerVelocity {
        let accepted = next_own_motion(bot, player.entity_id()).await?;
        let tolerance = 1.0 / 16383.0;
        if (accepted.x - STANDING_KNOCKBACK.x).abs() > tolerance
            || (accepted.y - STANDING_KNOCKBACK.y).abs() > tolerance
            || (accepted.z - STANDING_KNOCKBACK.z).abs() > tolerance
        {
            return Err(format!(
                "The accepted velocity was not sent after the nested zero: {accepted:?}"
            ));
        }
    }
    let (packet_center, radius, knockback) = next_explosion_packet(bot).await?;
    if packet_center != center || radius != 6.0 {
        return Err("Missing the explosion after absolute zero motion".into());
    }
    if knockback.is_some() {
        return Err(format!(
            "Absolute zero motion did not invalidate the recorded knockback: {knockback:?}"
        ));
    }
    let expected_velocity = if timing == AbsoluteMotionTiming::PlayerVelocity {
        STANDING_KNOCKBACK
    } else {
        Vector3::default()
    };
    let expected_calls = if timing == AbsoluteMotionTiming::PlayerVelocity {
        2
    } else {
        1
    };
    if entity.velocity.load() != expected_velocity
        || calls.load(std::sync::atomic::Ordering::SeqCst) != expected_calls
    {
        return Err(format!(
            "Unexpected final velocity or event count: expected {expected_velocity:?}/{expected_calls}, got {:?}/{}",
            entity.velocity.load(),
            calls.load(std::sync::atomic::Ordering::SeqCst),
        ));
    }
    Ok(())
}

fn register_zero_velocity_handler(
    server: &Arc<Server>,
    player: &Player,
    timing: AbsoluteMotionTiming,
) -> (Arc<AtomicUsize>, Arc<AtomicBool>, ExplosionInteraction) {
    let calls = Arc::new(AtomicUsize::new(0));
    let enabled = Arc::new(AtomicBool::new(true));
    let interaction = match timing {
        AbsoluteMotionTiming::BlockExplode => {
            server.plugin_manager.register_blocking(
                Arc::new(ZeroVelocityOnBlockExplode {
                    calls: calls.clone(),
                    enabled: enabled.clone(),
                }),
                EventPriority::Normal,
                None,
            );
            ExplosionInteraction::Block
        }
        AbsoluteMotionTiming::EntityKnockback => {
            server.plugin_manager.register_blocking(
                Arc::new(ZeroVelocityOnEntityKnockback {
                    calls: calls.clone(),
                    enabled: enabled.clone(),
                    player_id: player.entity_id(),
                }),
                EventPriority::Normal,
                None,
            );
            ExplosionInteraction::None
        }
        AbsoluteMotionTiming::PlayerVelocity => {
            server.plugin_manager.register_blocking(
                Arc::new(ZeroVelocityOnPlayerVelocity {
                    calls: calls.clone(),
                    enabled: enabled.clone(),
                    player_id: player.entity_id(),
                }),
                EventPriority::Normal,
                None,
            );
            ExplosionInteraction::None
        }
    };
    (calls, enabled, interaction)
}

async fn detonate_crystal(
    server: &Arc<Server>,
    bot: &mut HeadlessBot,
    player: &Arc<Player>,
    world: &Arc<World>,
    center: Vector3<f64>,
    expected_knockback: Vector3<f64>,
) -> Result<(), String> {
    let crystal = Arc::new(EndCrystalEntity::new(Entity::new(
        world.clone(),
        center,
        &EntityType::END_CRYSTAL,
    )));
    if !world.spawn_entity(crystal.clone()) {
        return Err("Could not spawn the End Crystal".into());
    }
    if !crystal.damage_with_context(
        player.as_ref(),
        1.0,
        DamageType::PLAYER_ATTACK,
        None,
        None,
        Some(player.as_ref()),
    ) {
        return Err("Could not detonate the End Crystal".into());
    }

    let (packet_center, knockback) = next_explosion(bot).await?;
    if packet_center != center {
        return Err(format!(
            "Explosion center mismatch: expected {center:?}, got {packet_center:?}"
        ));
    }
    if (knockback.x - expected_knockback.x).abs() > 1.0e-6
        || (knockback.y - expected_knockback.y).abs() > 1.0e-6
        || (knockback.z - expected_knockback.z).abs() > 1.0e-6
    {
        return Err(format!(
            "Explosion knockback mismatch: expected {expected_knockback:?}, got {knockback:?}"
        ));
    }
    server.tick();
    Ok(())
}

async fn next_own_motion(bot: &mut HeadlessBot, entity_id: i32) -> Result<Vector3<f64>, String> {
    let motion_id = CEntityVelocity::to_id(CURRENT_MC_VERSION);
    let explosion_id = CExplosion::to_id(CURRENT_MC_VERSION);
    loop {
        let packet = bot.next_packet().await?;
        if packet.id == explosion_id {
            return Err("Received CExplosion while waiting for absolute self-motion".into());
        }
        if packet.id != motion_id {
            continue;
        }

        let mut payload = &packet.payload[..];
        let packet_entity_id = VarInt::decode(&mut payload)
            .map_err(|err| format!("Failed to read motion entity ID: {err}"))?;
        let motion = LpVector3d::read(&mut payload)
            .map_err(|err| format!("Failed to read motion vector: {err}"))?
            .0;
        if packet_entity_id.0 == entity_id {
            return Ok(motion);
        }
    }
}

async fn next_explosion(bot: &mut HeadlessBot) -> Result<(Vector3<f64>, Vector3<f64>), String> {
    let (center, radius, knockback) = next_explosion_packet(bot).await?;
    if radius != 6.0 {
        return Err(format!(
            "Expected an End Crystal explosion, got radius {radius}"
        ));
    }
    let knockback = knockback.ok_or("Explosion packet had no knockback for the hit player")?;
    Ok((center, knockback))
}

async fn next_explosion_packet(
    bot: &mut HeadlessBot,
) -> Result<(Vector3<f64>, f32, Option<Vector3<f64>>), String> {
    let explosion_id = CExplosion::to_id(CURRENT_MC_VERSION);
    let packet = loop {
        let packet = bot.next_packet().await?;
        if packet.id == explosion_id {
            break packet;
        }
    };

    let mut payload = &packet.payload[..];
    if payload.len() < 33 {
        return Err("Explosion packet ended before the knockback flag".into());
    }
    let center = Vector3::new(payload.get_f64(), payload.get_f64(), payload.get_f64());
    let radius = payload.get_f32();
    let _block_count = payload.get_i32();
    let has_knockback = payload.get_u8() != 0;
    if has_knockback && payload.len() < 24 {
        return Err("Explosion packet ended inside the knockback vector".into());
    }
    let knockback = has_knockback
        .then(|| Vector3::new(payload.get_f64(), payload.get_f64(), payload.get_f64()));
    if knockback.is_some_and(|knockback| {
        !knockback.x.is_finite() || !knockback.y.is_finite() || !knockback.z.is_finite()
    }) {
        return Err("Explosion knockback was not finite".into());
    }
    Ok((center, radius, knockback))
}
