use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, Vector3Ext, wrap_degrees};
use futures::future::BoxFuture;
use pumpkin_util::math::vector3::Vector3;

pub struct SitScanningPhase;

impl super::Phase for SitScanningPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::SittingScanning
    }

    fn is_sitting_or_hovering(&self) -> bool {
        true
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.target_location.lock().await = None;
            *dragon.sit_scanning_ticks.lock().await = 0;
        })
    }

    #[expect(clippy::too_many_lines)]
    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let ticks = {
                let mut t = dragon.sit_scanning_ticks.lock().await;
                *t += 1;
                *t
            };

            // Java never repositions the dragon here - it just stays wherever
            // `LandingPhase` left it (`AbstractSittingPhase`/`SittingScanningPhase` have
            // no position logic at all).
            let world = dragon.mob_entity.living_entity.entity.world.load();
            let pos = dragon.mob_entity.living_entity.entity.pos.load();

            let close_player = {
                let players = world.players.load();
                players
                    .iter()
                    .filter(|p| {
                        let p_pos = p.living_entity.entity.pos.load();
                        let dx = p_pos.x - pos.x;
                        let dz = p_pos.z - pos.z;
                        let horizontal_dist_sq = dx * dx + dz * dz;
                        let vertical_dist = (p_pos.y - pos.y).abs();
                        horizontal_dist_sq < 400.0 && vertical_dist <= 10.0
                    })
                    .min_by(|a, b| {
                        let a_pos = a.living_entity.entity.pos.load();
                        let b_pos = b.living_entity.entity.pos.load();
                        a_pos
                            .distance_squared(pos)
                            .partial_cmp(&b_pos.distance_squared(pos))
                            .unwrap()
                    })
                    .cloned()
            };

            if let Some(player) = close_player {
                if ticks > 25 {
                    dragon.set_phase(EnderDragonPhase::SittingAttacking).await;
                } else {
                    let player_pos = player.living_entity.entity.pos.load();
                    let to_player = Vector3::new(player_pos.x - pos.x, 0.0, player_pos.z - pos.z);
                    let to_player_norm = if to_player.length_squared() > 1e-6 {
                        to_player.normalize()
                    } else {
                        Vector3::new(0.0, 0.0, 0.0)
                    };

                    let yaw = dragon.mob_entity.living_entity.entity.yaw.load();
                    let facing = Vector3::new(
                        (yaw * (std::f32::consts::PI / 180.0)).sin() as f64,
                        0.0,
                        -(yaw * (std::f32::consts::PI / 180.0)).cos() as f64,
                    );
                    let facing_norm = if facing.length_squared() > 1e-6 {
                        facing.normalize()
                    } else {
                        Vector3::new(0.0, 0.0, 0.0)
                    };

                    let dot = facing_norm.dot(&to_player_norm) as f32;
                    let angle = dot.acos().to_degrees() + 0.5;

                    if !(0.0..=10.0).contains(&angle) {
                        let head_pos = dragon.parts[0].entity.pos.load();
                        let dx = player_pos.x - head_pos.x;
                        let dz = player_pos.z - head_pos.z;
                        let dist_sq = dx * dx + dz * dz;
                        let dist = dist_sq.sqrt() as f32 + 1.0;
                        let clamped_dist = dist.min(40.0);
                        let yaw_clamped =
                            wrap_degrees(180.0 - dx.atan2(dz).to_degrees() as f32 - yaw);
                        let yaw_change = yaw_clamped.clamp(-100.0, 100.0);

                        let mut yaw_accel = dragon.yaw_rot_accel.lock().await;
                        *yaw_accel *= 0.8;
                        *yaw_accel += yaw_change * (0.7 / clamped_dist / dist);
                        dragon
                            .mob_entity
                            .living_entity
                            .entity
                            .yaw
                            .store(yaw + *yaw_accel);
                    }
                }
            } else if ticks >= 100 {
                let far_player = {
                    let players = world.players.load();
                    players
                        .iter()
                        .filter(|p| {
                            let p_pos = p.living_entity.entity.pos.load();
                            p_pos.distance_squared(pos) < 22500.0
                        })
                        .min_by(|a, b| {
                            let a_pos = a.living_entity.entity.pos.load();
                            let b_pos = b.living_entity.entity.pos.load();
                            a_pos
                                .distance_squared(pos)
                                .partial_cmp(&b_pos.distance_squared(pos))
                                .unwrap()
                        })
                        .cloned()
                };

                // Java: always sets TAKEOFF first, then overrides to CHARGING
                // if a player is found.  This matches the exact phase order.
                dragon.set_phase(EnderDragonPhase::Takeoff).await;
                if let Some(player) = far_player {
                    let player_pos = player.living_entity.entity.pos.load();
                    *dragon.target_player.lock().await = Some(player.gameprofile.id);
                    *dragon.strafe_target.lock().await = Some(player_pos);
                    dragon.set_phase(EnderDragonPhase::ChargingPlayer).await;
                }
            }
        })
    }
}
