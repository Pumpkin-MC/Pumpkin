use std::sync::Arc;

use bytes::Buf;
use pumpkin_core::{
    entity::{Entity, EntityBase, decoration::end_crystal::EndCrystalEntity, player::Player},
    server::Server,
    world::{BlockFlags, World},
};
use pumpkin_data::{Block, damage::DamageType, entity::EntityType, packet::CURRENT_MC_VERSION};
use pumpkin_protocol::{java::client::play::CExplosion, packet::MultiVersionJavaPacket};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::bot::HeadlessBot;

pub async fn run_explosion_knockback_test(
    server: &Arc<Server>,
    bot1: &mut HeadlessBot,
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
                previous_blocks.push((pos, world.get_block_state(&pos).id));
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
    let result: Result<(), String> = async {
        let standing = center.add_raw(-1.0, 0.0, 0.0);
        bot1.move_to(standing, true).await?;
        server.tick();
        // Expected from vanilla geometry: one block horizontally away and eye height 1.62.
        detonate_crystal(
            server,
            bot1,
            &player,
            &world,
            center,
            Vector3::new(-0.48149676911700806, 0.7800247682655086, 0.0),
        )
        .await?;

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
        .await
    }
    .await;

    for (pos, state) in previous_blocks {
        world.set_block_state(&pos, state, BlockFlags::NOTIFY_ALL);
    }
    for (other, max_health, health) in health_states {
        other.set_max_health(max_health);
        other.set_health(health);
    }
    result?;
    summary_lines.push("[PASS] bot:explosion_player_knockback".into());
    Ok(())
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

async fn next_explosion(bot: &mut HeadlessBot) -> Result<(Vector3<f64>, Vector3<f64>), String> {
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
    if radius != 6.0 {
        return Err(format!(
            "Expected an End Crystal explosion, got radius {radius}"
        ));
    }
    let _block_count = payload.get_i32();
    if payload.get_u8() == 0 {
        return Err("Explosion packet had no knockback for the hit player".into());
    }
    if payload.len() < 24 {
        return Err("Explosion packet ended inside the knockback vector".into());
    }
    let knockback = Vector3::new(payload.get_f64(), payload.get_f64(), payload.get_f64());
    if !knockback.x.is_finite() || !knockback.y.is_finite() || !knockback.z.is_finite() {
        return Err("Explosion knockback was not finite".into());
    }
    Ok((center, knockback))
}
