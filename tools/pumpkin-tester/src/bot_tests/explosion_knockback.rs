use std::sync::Arc;

use bytes::Buf;
use pumpkin_core::{server::Server, world::ExplosionInteraction};
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_protocol::{java::client::play::CExplosion, packet::MultiVersionJavaPacket};
use pumpkin_util::math::vector3::Vector3;

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
    let center = player.position().add_raw(1.0, 0.0, 0.0);
    let health = player.living_entity.health.load();
    world.explode(center, 1.0, ExplosionInteraction::None);

    let explosion_id = CExplosion::to_id(CURRENT_MC_VERSION);
    let packet = loop {
        let packet = bot1.next_packet().await?;
        if packet.id == explosion_id {
            break packet;
        }
    };
    player.set_health(health);

    // 26.3: three f64 center coordinates, f32 radius, i32 block count, then optional knockback.
    let mut payload = &packet.payload[..];
    if payload.len() < 33 {
        return Err("Explosion packet ended before the knockback flag".into());
    }
    payload.advance(32);
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
    if knockback.x >= 0.0 || knockback.y <= 0.0 {
        return Err(format!(
            "Explosion knockback pointed in an unexpected direction: {knockback:?}"
        ));
    }

    summary_lines.push("[PASS] bot:explosion_player_knockback".into());
    Ok(())
}
