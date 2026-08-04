use super::EnderDragonPhase;
use crate::entity::EntityBase;
use crate::entity::boss::ender_dragon::{
    DragonNode, DragonPath, EnderDragonEntity, Vector3Ext, find_path,
};
use futures::future::BoxFuture;
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering;

pub struct LandingApproachPhase;

impl super::Phase for LandingApproachPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::LandingApproach
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.path.lock().await = None;
            *dragon.target_location.lock().await = None;
        })
    }

    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let pos = dragon.mob_entity.living_entity.entity.pos.load();
            let target_location = *dragon.target_location.lock().await;

            let d0 = target_location.map_or(0.0, |loc| pos.distance_squared(loc));
            if !(100.0..=22500.0).contains(&d0)
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
                let mut path_guard = dragon.path.lock().await;

                // Check if path is finished -> transition to Landing phase
                if let Some(ref path) = *path_guard
                    && path.is_finished()
                {
                    drop(path_guard);
                    dragon.set_phase(EnderDragonPhase::Landing).await;
                    return;
                }

                // Generate approach path if none exists
                let is_finished_or_none = path_guard.as_ref().is_none_or(DragonPath::is_finished);
                if is_finished_or_none {
                    let world = dragon.mob_entity.living_entity.entity.world.load();
                    let origin = {
                        let guard = dragon.fight_origin.lock().await;
                        guard.0
                    };

                    let portal_top = dragon.portal_top().await;

                    let closest_player = {
                        let players = world.players.load();
                        players
                            .iter()
                            .filter(|p| !p.is_spectator() && !p.is_creative())
                            .min_by(|a, b| {
                                let a_pos = a.living_entity.entity.pos.load();
                                let b_pos = b.living_entity.entity.pos.load();
                                a_pos
                                    .distance_squared(portal_top)
                                    .partial_cmp(&b_pos.distance_squared(portal_top))
                                    .unwrap()
                            })
                            .cloned()
                    };

                    let i = dragon.find_closest_node().await;
                    let j = if let Some(player) = closest_player {
                        let player_pos = player.living_entity.entity.pos.load();
                        let rot_vec = Vector3::new(player_pos.x, 0.0, player_pos.z).normalize();
                        let target_x = -rot_vec.x * 40.0;
                        let target_z = -rot_vec.z * 40.0;
                        dragon
                            .find_closest_node_to(Vector3::new(target_x, 105.0, target_z))
                            .await
                    } else {
                        dragon
                            .find_closest_node_to(Vector3::new(40.0, origin.y as f64, 0.0))
                            .await
                    };

                    let portal_node = DragonNode::new(portal_top.x, portal_top.y, portal_top.z);
                    let nodes = dragon.nodes.lock().await;
                    let new_nodes = find_path(&nodes, i, j, Some(portal_node), 0);
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
                        *dragon.target_location.lock().await =
                            Some(Vector3::new(node.x, y_target, node.z));
                    }
                }
            }
        })
    }
}
