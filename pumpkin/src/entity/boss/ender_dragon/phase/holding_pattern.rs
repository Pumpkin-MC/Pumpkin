use super::EnderDragonPhase;
use crate::entity::EntityBase;
use crate::entity::boss::ender_dragon::{DragonPath, EnderDragonEntity, Vector3Ext, find_path};
use crate::entity::player::Player;
use futures::future::BoxFuture;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub struct HoldingPatternPhase;

impl super::Phase for HoldingPatternPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::HoldingPattern
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.path.lock().await = None;
            *dragon.target_location.lock().await = None;
        })
    }

    #[expect(clippy::too_many_lines)]
    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let pos = dragon.mob_entity.living_entity.entity.pos.load();
            let mut target_location = dragon.target_location.lock().await;

            let d0 = target_location.map_or(0.0, |loc| pos.distance_squared(loc));

            if target_location.is_none()
                || !(100.0..=22500.0).contains(&d0)
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

                // Check phase transitions ONLY if a path was already set and is now finished
                if let Some(ref path) = *path_guard
                    && path.is_finished()
                {
                    let world = dragon.mob_entity.living_entity.entity.world.load();
                    let crystals_alive = if let Some(ref fight) = world.dragon_fight {
                        fight.lock().await.alive_crystals()
                    } else {
                        0
                    };

                    let portal_top = dragon.portal_top().await;

                    if rand::random_range(0..(crystals_alive as i32 + 3)) == 0 {
                        drop(path_guard);
                        drop(target_location);
                        dragon.set_phase(EnderDragonPhase::LandingApproach).await;
                        return;
                    }

                    let closest_player_to_portal = {
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

                    if let Some(player) = closest_player_to_portal {
                        let player_pos = player.living_entity.entity.pos.load();
                        let dist_to_portal_sq = player_pos.distance_squared(portal_top);
                        let dist_factor = (dist_to_portal_sq / 512.0 + 2.0) as i32;
                        if rand::random_range(0..dist_factor.max(1)) == 0
                            || rand::random_range(0..(crystals_alive as i32 + 2)) == 0
                        {
                            *dragon.target_player.lock().await = Some(player.gameprofile.id);
                            drop(path_guard);
                            drop(target_location);
                            dragon.set_phase(EnderDragonPhase::StrafePlayer).await;
                            return;
                        }
                    }
                }

                // Generate new path if none exists or current path is finished
                let is_finished_or_none = path_guard.as_ref().is_none_or(DragonPath::is_finished);
                if is_finished_or_none {
                    let i = dragon.find_closest_node().await;
                    let mut j = i as i32;

                    let mut clockwise = dragon.holding_pattern_clockwise.lock().await;
                    if rand::random_range(0..8) == 0 {
                        *clockwise = !*clockwise;
                        j = i as i32 + 6;
                    }
                    if *clockwise {
                        j += 1;
                    } else {
                        j -= 1;
                    }
                    drop(clockwise);

                    // Java: `getAliveEndCrystals() >= 0` is always true once a fight
                    // exists (the count can't be negative) - so this only takes the
                    // "no fight" branch (nodes 12-19) when there's no fight at all,
                    // unlike Strafe/Takeoff which switch there once crystals hit 0.
                    let world = dragon.mob_entity.living_entity.entity.world.load();
                    let crystals_alive = if let Some(ref fight) = world.dragon_fight {
                        fight.lock().await.alive_crystals()
                    } else {
                        0
                    };
                    let j = if world.dragon_fight.is_some() {
                        j.rem_euclid(12) as usize
                    } else {
                        (j - 12).rem_euclid(8) as usize + 12
                    };

                    // Java: minimumNodeIndex = 12 when no alive crystals (uses
                    // only inner rings 12-23), 0 otherwise.
                    let minimum_node_index = if crystals_alive > 0 { 0 } else { 12 };

                    let nodes = dragon.nodes.lock().await;
                    let new_nodes = find_path(&nodes, i, j, None, minimum_node_index);
                    drop(nodes);
                    *path_guard = Some(DragonPath::new(new_nodes));
                }

                // Follow path step: advance to next node and update target_location
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
        })
    }

    fn crystal_destroyed<'a>(
        &'a self,
        dragon: &'a EnderDragonEntity,
        player: Option<Arc<Player>>,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if let Some(player) = player {
                *dragon.target_player.lock().await = Some(player.gameprofile.id);
                dragon.set_phase(EnderDragonPhase::StrafePlayer).await;
            }
        })
    }
}
