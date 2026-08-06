use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;
use uuid::Uuid;

use super::silverfish_util::{host_for_infested, zigzag_range};
use super::{Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::entity::mob::silverfish::SilverfishEntity;
use crate::item::items::state_with_properties_of;

/// Vanilla `Silverfish.SilverfishWakeUpFriendsGoal`.
///
/// Once the silverfish has been hurt, scans a `y in [-5,5]`, `x,z in [-10,10]` box around it
/// and either breaks (mob-griefing on, spawning a fresh silverfish like a normal infested-block
/// break) or silently reverts (mob-griefing off) one nearby infested block, matching vanilla's
/// outward zig-zag search order.
pub struct SilverfishWakeUpFriendsGoal {
    silverfish: Arc<SilverfishEntity>,
}

impl SilverfishWakeUpFriendsGoal {
    #[must_use]
    pub const fn new(silverfish: Arc<SilverfishEntity>) -> Self {
        Self { silverfish }
    }
}

impl Goal for SilverfishWakeUpFriendsGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.silverfish.wake_up_friends_timer.load(Relaxed) > 0 })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let remaining = self.silverfish.wake_up_friends_timer.fetch_sub(1, Relaxed) - 1;
            if remaining > 0 {
                return;
            }
            self.silverfish.wake_up_friends_timer.store(0, Relaxed);

            let entity = mob.get_entity();
            let world = entity.world.load_full();
            let base_pos = entity.block_pos.load();
            let mob_griefing = world.level_info.load().game_rules.mob_griefing;

            let y_offsets = zigzag_range(5);
            let xz_offsets = zigzag_range(10);

            for y_off in y_offsets {
                for &x_off in &xz_offsets {
                    for &z_off in &xz_offsets {
                        let test_pos = base_pos.add(x_off, y_off, z_off);
                        let (block, state) = world.get_block_and_state(&test_pos);
                        let Some(host_id) = host_for_infested(block.id) else {
                            continue;
                        };

                        if mob_griefing {
                            world
                                .break_block(&test_pos, None, BlockFlags::NOTIFY_ALL)
                                .await;
                            if world.level_info.load().game_rules.block_drops {
                                let spawn_pos = Vector3::new(
                                    f64::from(test_pos.0.x) + 0.5,
                                    f64::from(test_pos.0.y),
                                    f64::from(test_pos.0.z) + 0.5,
                                );
                                let uuid = Uuid::new_v4();
                                let new_entity = crate::entity::r#type::from_type(
                                    &EntityType::SILVERFISH,
                                    spawn_pos,
                                    &world,
                                    uuid,
                                );
                                world.spawn_entity(new_entity.clone()).await;
                                world.send_entity_status(
                                    new_entity.get_entity(),
                                    pumpkin_data::entity::EntityStatus::SilverfishMergeAnim,
                                );
                            }
                        } else {
                            let host_block = host_id.to_block();
                            let new_state_id =
                                state_with_properties_of(block, state.id, host_block);
                            world
                                .set_block_state(&test_pos, new_state_id, BlockFlags::NOTIFY_ALL)
                                .await;
                        }

                        // Drawn fresh each time (not held across the awaits above) since
                        // `ThreadRng` is not `Send`.
                        if mob.get_random().random_bool(0.5) {
                            return;
                        }
                    }
                }
            }
        })
    }
}
