use super::EnderDragonPhase;
use crate::entity::{
    Entity,
    area_effect_cloud::AreaEffectCloudEntity,
    boss::ender_dragon::{EnderDragonEntity, Vector3Ext, find_path},
};
use futures::future::BoxFuture;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering;

pub struct StrafingPhase;

impl super::Phase for StrafingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Strafing
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.fireball_charge.lock().await = 0;
            *dragon.target_location.lock().await = None;
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
                dragon.set_phase(EnderDragonPhase::Circling).await;
                return;
            };

            let player_pos = player.living_entity.entity.pos.load();
            let mut path = dragon.path.lock().await;
            let mut target_location = dragon.target_location.lock().await;

            if path.is_empty() {
                let d2 = player_pos.x - pos.x;
                let d3 = player_pos.z - pos.z;
                let d4 = d2.hypot(d3);
                let d5 = (0.4 + d4 / 80.0 - 1.0).clamp(0.0, 10.0);
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
            {
                if path.is_empty() {
                    drop(path);
                    let i = dragon.find_closest_node().await;
                    let j = dragon.find_closest_node_to(player_pos).await;

                    let mut path_lock = dragon.path.lock().await;
                    let nodes = dragon.nodes.lock().await;
                    *path_lock = find_path(&nodes, i, j, None);
                    drop(nodes);
                    path = path_lock;
                }

                if let Some(next_node_idx) = path.first().copied() {
                    path.remove(0);
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
            drop(path);
            drop(target_location);

            if player_pos.distance_squared(pos) < 4096.0 {
                let mut charge = dragon.fireball_charge.lock().await;

                let aim_diff = player_pos - pos;
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

                let dot = dir_norm.dot(&aim) as f32;
                let angle_degs = dot.acos().to_degrees() + 0.5;

                *charge += 1;

                if *charge >= 5 && angle_degs < 10.0 {
                    *charge = 0;
                    drop(charge);

                    let cloud_entity =
                        Entity::new(world.clone(), player_pos, &EntityType::AREA_EFFECT_CLOUD);
                    let cloud = AreaEffectCloudEntity::create(
                        cloud_entity,
                        pumpkin_data::item_stack::ItemStack::new(
                            0,
                            &pumpkin_data::item::Item::DRAGON_BREATH,
                        ),
                        vec![(
                            &pumpkin_data::effect::StatusEffect::INSTANT_DAMAGE,
                            1,
                            0,
                            false,
                            true,
                            true,
                        )],
                        600,
                        3.0,
                        20,
                        20,
                        0.5,
                        -100,
                    );
                    world.spawn_entity(cloud).await;

                    dragon.path.lock().await.clear();
                    dragon.set_phase(EnderDragonPhase::Circling).await;
                }
            } else {
                let mut charge = dragon.fireball_charge.lock().await;
                if *charge > 0 {
                    *charge -= 1;
                }
            }
        })
    }
}
