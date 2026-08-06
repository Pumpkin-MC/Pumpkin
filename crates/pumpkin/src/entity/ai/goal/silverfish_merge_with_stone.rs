use pumpkin_data::BlockDirection;
use pumpkin_data::entity::EntityStatus;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use super::silverfish_util::is_compatible_host_block;
use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::item::items::state_with_properties_of;

/// Vanilla `Silverfish.SilverfishMergeWithStoneGoal`: an extended `RandomStrollGoal` that, with
/// low odds, instead hides the silverfish inside an adjacent stone-family block.
pub struct SilverfishMergeWithStoneGoal {
    selected_direction: Option<BlockDirection>,
    do_merge: bool,
    wander_target: Option<Vector3<f64>>,
}

impl SilverfishMergeWithStoneGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected_direction: None,
            do_merge: false,
            wander_target: None,
        }
    }

    /// Vanilla `RandomStrollGoal.getPosition` -> `DefaultRandomPos.getPos(mob, 10, 7)`.
    fn find_wander_target(mob: &dyn Mob) -> Vector3<f64> {
        let pos = mob.get_entity().pos.load();
        let mut rng = mob.get_random();

        let dx = rng.random_range(-10.0..=10.0);
        let dy = rng.random_range(-7.0..=7.0);
        let dz = rng.random_range(-10.0..=10.0);

        Vector3::new(pos.x + dx, pos.y + dy, pos.z + dz)
    }
}

impl Default for SilverfishMergeWithStoneGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for SilverfishMergeWithStoneGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            // Cleared up front so a stale value from an earlier, unrelated call can never leak
            // into this cycle's `start()` (only ever re-set right before an immediate `true`).
            self.do_merge = false;
            self.selected_direction = None;

            if mob.get_mob_entity().target.lock().await.is_some() {
                return false;
            }

            if !mob.get_mob_entity().navigator.lock().unwrap().is_idle() {
                return false;
            }

            let entity = mob.get_entity();
            let world = entity.world.load();
            let mob_griefing = world.level_info.load().game_rules.mob_griefing;
            let mut rng = mob.get_random();

            if mob_griefing && rng.random_range(0..to_goal_ticks(10)) == 0 {
                let directions = BlockDirection::all();
                let direction = directions[rng.random_range(0..directions.len())];
                let pos = mob.get_entity().pos.load();
                let target_pos =
                    BlockPos::floored(pos.x, pos.y + 0.5, pos.z).offset(direction.to_offset());
                let state = world.get_block_state(&target_pos);
                if is_compatible_host_block(pumpkin_data::Block::from_state_id(state.id)) {
                    self.selected_direction = Some(direction);
                    self.do_merge = true;
                    return true;
                }
            }

            if rng.random_range(0..to_goal_ticks(10)) != 0 {
                return false;
            }

            self.wander_target = Some(Self::find_wander_target(mob));
            true
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.do_merge {
                return false;
            }
            !mob.get_mob_entity().navigator.lock().unwrap().is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if !self.do_merge {
                if let Some(target) = self.wander_target {
                    let pos = mob.get_entity().pos.load();
                    let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                    navigator.set_progress(NavigatorGoal::new(pos, target, 1.0));
                }
                return;
            }

            let Some(direction) = self.selected_direction else {
                return;
            };

            let entity = mob.get_entity();
            let pos = entity.pos.load();
            let target_pos =
                BlockPos::floored(pos.x, pos.y + 0.5, pos.z).offset(direction.to_offset());
            let world = entity.world.load_full();
            let (host_block, host_state) = world.get_block_and_state(&target_pos);

            if !is_compatible_host_block(host_block) {
                return;
            }

            let Some(infested_id) = super::silverfish_util::infested_for_host(host_block.id) else {
                return;
            };
            let infested_block = infested_id.to_block();

            let new_state_id = state_with_properties_of(host_block, host_state.id, infested_block);
            world
                .set_block_state(&target_pos, new_state_id, BlockFlags::NOTIFY_ALL)
                .await;

            world.send_entity_status(entity, EntityStatus::SilverfishMergeAnim);
            entity.remove().await;
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.wander_target = None;
            self.do_merge = false;
            self.selected_direction = None;
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
