use super::EnderDragonPhase;
use crate::entity::EntityBase;
use crate::entity::{
    Entity,
    boss::ender_dragon::{DragonPath, EnderDragonEntity, Vector3Ext, find_path},
    projectile::dragon_fireball::DragonFireballEntity,
};
use futures::future::BoxFuture;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub struct StrafePlayerPhase;

impl super::Phase for StrafePlayerPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::StrafePlayer
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.target_location.lock().await = None;
            *dragon.path.lock().await = None;
            *dragon.seen_target_times.lock().await = 0;

            // Initialize path toward the target player, matching Java's
            // StrafePlayerPhase.setTargetEntity().
            let target_id = *dragon.target_player.lock().await;
            if let Some(id) = target_id {
                let world = dragon.mob_entity.living_entity.entity.world.load();
                let player = world
                    .players
                    .load()
                    .iter()
                    .find(|p| p.gameprofile.id == id)
                    .cloned();
                if let Some(player) = player {
                    let player_pos = player.get_entity().pos.load();
                    let i = dragon.find_closest_node().await;
                    let j = dragon.find_closest_node_to(player_pos).await;

                    let dx = player_pos.x - dragon.mob_entity.living_entity.entity.pos.load().x;
                    let dz = player_pos.z - dragon.mob_entity.living_entity.entity.pos.load().z;
                    let dist = dx.hypot(dz);
                    let g = (0.4f64 + dist / 80.0 - 1.0).min(10.0);

                    let target_node = crate::entity::boss::ender_dragon::DragonNode::new(
                        player_pos.x,
                        player_pos.y + g,
                        player_pos.z,
                    );

                    let nodes = dragon.nodes.lock().await;
                    let new_nodes = find_path(&nodes, i, j, Some(target_node), 0);
                    drop(nodes);

                    let mut path_guard = dragon.path.lock().await;
                    *path_guard = Some(DragonPath::new(new_nodes));

                    if let Some(ref mut path) = *path_guard
                        && !path.is_finished()
                        && let Some(next_node_idx) = path.current_node()
                    {
                        path.advance();
                        let nodes = dragon.nodes.lock().await;
                        if let Some(node) = nodes[next_node_idx] {
                            let mut y_target = node.y + rand::random_range(0.0..20.0);
                            while y_target < node.y {
                                y_target = node.y + rand::random_range(0.0..20.0);
                            }
                            *dragon.target_location.lock().await =
                                Some(Vector3::new(node.x, y_target, node.z));
                        }
                    }
                }
            }
        })
    }

    #[expect(clippy::too_many_lines)]
    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let target_id = *dragon.target_player.lock().await;
            let world = dragon.mob_entity.living_entity.entity.world.load();
            let pos = dragon.mob_entity.living_entity.entity.pos.load();

            let player_target = if let Some(id) = target_id {
                world
                    .players
                    .load()
                    .iter()
                    .find(|p| p.gameprofile.id == id)
                    .cloned()
            } else {
                None
            };

            let Some(player) = player_target else {
                dragon.set_phase(EnderDragonPhase::HoldingPattern).await;
                return;
            };

            // Java: TargetingConditions.forCombat() checks canBeSeenAsEnemy(),
            // which returns false for creative (abilities.invulnerable) and
            // spectator players. If the target switches gamemode mid-strafe,
            // abort the strafe.
            if player.is_spectator() || player.is_creative() {
                dragon.set_phase(EnderDragonPhase::HoldingPattern).await;
                return;
            }

            let player_pos = player.get_entity().pos.load();
            let mut path_guard = dragon.path.lock().await;
            let mut target_location = dragon.target_location.lock().await;

            let is_finished = path_guard.as_ref().is_none_or(DragonPath::is_finished);
            if is_finished {
                let d2 = player_pos.x - pos.x;
                let d3 = player_pos.z - pos.z;
                let d4 = d2.hypot(d3);
                let d5 = (0.4 + d4 / 80.0 - 1.0).min(10.0);
                *target_location =
                    Some(Vector3::new(player_pos.x, player_pos.y + d5, player_pos.z));
            }

            let d11 = target_location
                .map(|loc| pos.distance_squared(loc))
                .unwrap_or(0.0);
            if !(100.0..=22500.0).contains(&d11)
                || dragon
                    .mob_entity
                    .living_entity
                    .entity
                    .horizontal_collision
                    .load(Ordering::Relaxed)
                || dragon
                    .mob_entity
                    .living_entity
                    .entity
                    .vertical_collision
                    .load(Ordering::Relaxed)
            {
                if is_finished {
                    let i = dragon.find_closest_node().await;
                    let mut j = i as i32;

                    let mut should_find = dragon.strafe_should_find_new_path.lock().await;
                    if rand::random_range(0..8) == 0 {
                        *should_find = !*should_find;
                        j = i as i32 + 6;
                    }
                    if *should_find {
                        j += 1;
                    } else {
                        j -= 1;
                    }
                    drop(should_find);

                    let world_ref = dragon.mob_entity.living_entity.entity.world.load();
                    let crystals_alive = if let Some(ref fight) = world_ref.dragon_fight {
                        fight.lock().await.alive_crystals()
                    } else {
                        0
                    };

                    let j = if crystals_alive > 0 {
                        j.rem_euclid(12) as usize
                    } else {
                        (j - 12).rem_euclid(8) as usize + 12
                    };

                    // Java: minimumNodeIndex = 12 when no alive crystals
                    let minimum_node_index = if crystals_alive > 0 { 0 } else { 12 };

                    let nodes = dragon.nodes.lock().await;
                    let new_nodes = find_path(&nodes, i, j, None, minimum_node_index);
                    drop(nodes);
                    *path_guard = Some(DragonPath::new(new_nodes));
                }

                if let Some(ref mut path) = *path_guard
                    && !path.is_finished()
                    && let Some(next_node_idx) = path.current_node()
                {
                    path.advance();
                    let nodes = dragon.nodes.lock().await;
                    if let Some(node) = nodes[next_node_idx] {
                        let mut y_target = node.y + rand::random_range(0.0..20.0);
                        while y_target < node.y {
                            y_target = node.y + rand::random_range(0.0..20.0);
                        }
                        *target_location = Some(Vector3::new(node.x, y_target, node.z));
                    }
                }
            }
            drop(path_guard);
            drop(target_location);

            if player_pos.distance_squared(pos) < 4096.0
                && dragon.can_see(player.get_entity().get_eye_pos()).await
            {
                let mut seen = dragon.seen_target_times.lock().await;
                *seen += 1;

                let aim_diff = Vector3::new(player_pos.x - pos.x, 0.0, player_pos.z - pos.z);
                let aim = if aim_diff.length_squared() > 1e-6 {
                    aim_diff.normalize()
                } else {
                    Vector3::new(0.0, 0.0, 0.0)
                };

                let yaw = dragon.mob_entity.living_entity.entity.yaw.load();
                let dir = Vector3::new(
                    (yaw * (std::f32::consts::PI / 180.0)).sin() as f64,
                    0.0,
                    -(yaw * (std::f32::consts::PI / 180.0)).cos() as f64,
                );

                let dir_norm = if dir.length_squared() > 1e-6 {
                    dir.normalize()
                } else {
                    Vector3::new(0.0, 0.0, 0.0)
                };

                let dot = (dir_norm.dot(&aim) as f32).clamp(-1.0, 1.0);
                let angle_degs = dot.acos().to_degrees() + 0.5;

                if *seen >= 5 && (0.0..10.0).contains(&angle_degs) {
                    // Java: `getRotationVec(1.0F)` is the plain pitch/yaw look vector; for
                    // `StrafePlayerPhase` (not landing/takeoff/sitting) our phase-aware
                    // helper already falls through to the same computation.
                    let rot_vec = dragon.get_rotation_vector_from_phase(1.0).await;

                    let head_pos = dragon.parts[0].entity.pos.load();
                    // `head.getBodyY(0.5) + 0.5` == `head.y + head_height * 0.5 + 0.5`;
                    // the head part is 1 block tall, so that's `head.y + 1.0`.
                    let spawn_pos = Vector3::new(
                        head_pos.x - rot_vec.x,
                        head_pos.y + 1.0,
                        head_pos.z - rot_vec.z,
                    );

                    let target_body_center = Vector3::new(
                        player_pos.x,
                        player_pos.y + f64::from(player.get_entity().height()) * 0.5,
                        player_pos.z,
                    );
                    let aim = target_body_center - spawn_pos;
                    let aim = if aim.length_squared() > 1e-6 {
                        aim.normalize()
                    } else {
                        Vector3::new(0.0, 0.0, -1.0)
                    };

                    world.sync_world_event(
                        pumpkin_data::world::WorldEvent::SoundDragonFireball,
                        dragon.mob_entity.living_entity.entity.block_pos.load(),
                        0,
                    );

                    let fireball_entity =
                        Entity::new(world.clone(), spawn_pos, &EntityType::DRAGON_FIREBALL);
                    let fireball = DragonFireballEntity::new_shot(
                        fireball_entity,
                        &dragon.mob_entity.living_entity.entity,
                    );
                    fireball.hurting.entity.set_pos(spawn_pos);
                    // Java AbstractHurtingProjectile constructor calls
                    // `assignDirectionalMovement(direction, accelerationPower)` which stores
                    // `direction.normalize().scale(0.1)`. The inertia in applyInertia() then
                    // accelerates the projectile toward its terminal velocity (~1.9 blocks/tick).
                    let speed = fireball.hurting.acceleration_power;
                    fireball
                        .hurting
                        .entity
                        .velocity
                        .store(aim.multiply(speed, speed, speed));
                    world.spawn_entity(Arc::new(fireball)).await;

                    *seen = 0;
                    if let Some(ref mut path) = *dragon.path.lock().await {
                        path.finish();
                    }
                    dragon.set_phase(EnderDragonPhase::HoldingPattern).await;
                }
            } else {
                let mut seen = dragon.seen_target_times.lock().await;
                if *seen > 0 {
                    *seen -= 1;
                }
            }
        })
    }
}
