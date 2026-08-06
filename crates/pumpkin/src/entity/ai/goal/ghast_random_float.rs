use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::ChunkHeightmapType;
use rand::RngExt;

use crate::entity::ai::goal::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;

const MAX_ATTEMPTS: i32 = 64;

/// Vanilla: `Ghast.RandomFloatAroundGoal` (Ghast.java:390-489, priority 5 at Ghast.java:57).
///
/// Ghast always constructs this with `distanceToBlocks = 0` (Ghast.java:57 uses the no-arg
/// overload at Ghast.java:395-397), which makes `isGoodTarget` (Ghast.java:456-476) always
/// return `true`. That block-distance branch is therefore dead for `Ghast` and is not ported.
pub struct GhastRandomFloatAroundGoal;

impl Default for GhastRandomFloatAroundGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl GhastRandomFloatAroundGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Vanilla: `Math.abs` re-roll condition inside `RandomFloatAroundGoal.canUse`
    /// (Ghast.java:412-416): re-roll if the wanted position is almost reached (`dd < 1.0`) or
    /// implausibly far away (`dd > 3600.0`, i.e. more than 60 blocks).
    #[must_use]
    pub const fn should_reroll(distance_sq: f64) -> bool {
        distance_sq < 1.0 || distance_sq > 3600.0
    }

    fn choose_random_position(
        center: Vector3<f64>,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Vector3<f64> {
        let x = center.x + f64::from(rng.random::<f32>() * 2.0 - 1.0) * 16.0;
        let y = center.y + f64::from(rng.random::<f32>() * 2.0 - 1.0) * 16.0;
        let z = center.z + f64::from(rng.random::<f32>() * 2.0 - 1.0) * 16.0;
        Vector3::new(x, y, z)
    }

    /// Vanilla: `RandomFloatAroundGoal.getSuitableFlyToPosition` (Ghast.java:430-454).
    fn get_suitable_fly_to_position(mob: &dyn Mob) -> Vector3<f64> {
        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let center = entity.pos.load();
        let world = entity.world.load();
        let mut rng = mob.get_random();

        let mut candidate = None;
        for _ in 0..MAX_ATTEMPTS {
            let pos = Self::choose_random_position(center, &mut rng);
            let block_pos = BlockPos::new(
                pos.x.floor() as i32,
                pos.y.floor() as i32,
                pos.z.floor() as i32,
            );
            // Vanilla: `chooseRandomPositionWithRestriction` returns `null` only when
            // `mob.hasHome() && !mob.isWithinHome(target)`; `is_in_position_target_range_pos`
            // already returns `true` unconditionally when unrestricted (Ghast has no home).
            if mob_entity.is_in_position_target_range_pos(&block_pos) {
                candidate = Some(pos);
                break;
            }
        }

        let mut result =
            candidate.unwrap_or_else(|| Self::choose_random_position(center, &mut rng));

        let block_pos = BlockPos::new(
            result.x.floor() as i32,
            result.y.floor() as i32,
            result.z.floor() as i32,
        );
        let height_y = world.get_heightmap_height(
            ChunkHeightmapType::MotionBlocking,
            block_pos.0.x,
            block_pos.0.z,
        );
        if height_y < block_pos.0.y && height_y > world.dimension.min_y {
            result.y = center.y - (center.y - result.y).abs();
        }

        result
    }
}

impl Goal for GhastRandomFloatAroundGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let move_control = mob.get_mob_entity().move_control.lock().unwrap();
            if !move_control.has_wanted() {
                return true;
            }

            let pos = mob.get_mob_entity().living_entity.entity.pos.load();
            let xd = move_control.get_wanted_x() - pos.x;
            let yd = move_control.get_wanted_y() - pos.y;
            let zd = move_control.get_wanted_z() - pos.z;
            Self::should_reroll(xd * xd + yd * yd + zd * zd)
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { false })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let target = Self::get_suitable_fly_to_position(mob);
            mob.get_mob_entity()
                .move_control
                .lock()
                .unwrap()
                .set_wanted_position(target.x, target.y, target.z, 1.0);
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

#[cfg(test)]
mod tests {
    use super::GhastRandomFloatAroundGoal;

    #[test]
    fn almost_arrived_triggers_reroll() {
        assert!(GhastRandomFloatAroundGoal::should_reroll(0.5));
    }

    #[test]
    fn implausibly_far_triggers_reroll() {
        assert!(GhastRandomFloatAroundGoal::should_reroll(3600.1));
    }

    #[test]
    fn in_progress_destination_is_kept() {
        assert!(!GhastRandomFloatAroundGoal::should_reroll(1800.0));
        assert!(!GhastRandomFloatAroundGoal::should_reroll(1.0));
        assert!(!GhastRandomFloatAroundGoal::should_reroll(3600.0));
    }
}
