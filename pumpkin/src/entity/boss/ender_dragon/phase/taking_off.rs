use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{
    DragonPath, EnderDragonEntity, NODE_Y, Vector3Ext, find_path,
};
use futures::future::BoxFuture;
use pumpkin_util::math::vector3::Vector3;

pub struct TakeoffPhase;

impl super::Phase for TakeoffPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Takeoff
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.should_find_new_path.lock().await = true;
            *dragon.path.lock().await = None;
            *dragon.target_location.lock().await = None;
        })
    }

    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let should_find = *dragon.should_find_new_path.lock().await;

            if should_find {
                *dragon.should_find_new_path.lock().await = false;

                // Use getRotationVectorFromPhase like Java, which accounts for
                // the pitch adjustment based on distance to the portal.
                let rot_vec = dragon.get_rotation_vector_from_phase(1.0).await;

                let i = dragon.find_closest_node().await;
                let j = dragon
                    .find_closest_node_to(Vector3::new(
                        -rot_vec.x * 40.0,
                        NODE_Y as f64,
                        -rot_vec.z * 40.0,
                    ))
                    .await as i32;

                let world = dragon.mob_entity.living_entity.entity.world.load();
                let crystals_alive = if let Some(ref fight) = world.dragon_fight {
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

                let mut path_guard = dragon.path.lock().await;
                *path_guard = Some(DragonPath::new(new_nodes));

                // `find_path` already removes the starting node from the path,
                // so we use the same consume pattern as HoldingPattern/Strafe/
                // LandingApproach: read the current node, then advance past it.
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
            } else {
                let portal_top = dragon.portal_top().await;
                let pos = dragon.mob_entity.living_entity.entity.pos.load();
                if pos.distance_squared(portal_top) > 100.0 {
                    dragon.set_phase(EnderDragonPhase::HoldingPattern).await;
                }
            }
        })
    }
}
