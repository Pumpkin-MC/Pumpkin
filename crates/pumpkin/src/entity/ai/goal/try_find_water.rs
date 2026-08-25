use std::sync::atomic::Ordering;

use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::world::World;

pub struct TryFindWaterGoal;

impl Default for TryFindWaterGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFindWaterGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub fn is_water(world: &World, pos: &BlockPos) -> bool {
        let (_, state_id) = world.get_block_and_state_id(pos);
        if state_id.to_state().is_waterlogged() {
            return true;
        }
        Fluid::from_state_id(state_id)
            .is_some_and(|fluid| fluid.has_tag(&tag::Fluid::MINECRAFT_WATER))
    }

    #[must_use]
    pub fn find_water_range(
        pos: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> (BlockPos, BlockPos) {
        let min_x = (pos.x - 2.0).floor() as i32;
        let min_y = (pos.y - 2.0).floor() as i32;
        let min_z = (pos.z - 2.0).floor() as i32;
        let max_x = (pos.x + 2.0).floor() as i32;
        let max_y = pos.y.floor() as i32;
        let max_z = (pos.z + 2.0).floor() as i32;

        (
            BlockPos::new(min_x, min_y, min_z),
            BlockPos::new(max_x, max_y, max_z),
        )
    }
}

impl Goal for TryFindWaterGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = mob.get_entity();
            if !entity.on_ground.load(Ordering::Relaxed) {
                return false;
            }

            let world = entity.world.load();
            let block_pos = entity.block_pos.load();
            !Self::is_water(&world, &block_pos)
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = mob.get_entity();
            let world = entity.world.load();
            let mob_pos = entity.pos.load();

            let (min_pos, max_pos) = Self::find_water_range(mob_pos);
            let mut water_pos: Option<BlockPos> = None;

            'outer: for x in min_pos.0.x..=max_pos.0.x {
                for y in min_pos.0.y..=max_pos.0.y {
                    for z in min_pos.0.z..=max_pos.0.z {
                        let pos = BlockPos::new(x, y, z);
                        if Self::is_water(&world, &pos) {
                            water_pos = Some(pos);
                            break 'outer;
                        }
                    }
                }
            }

            if let Some(pos) = water_pos {
                mob.get_mob_entity()
                    .move_control
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .set_wanted_position(
                        f64::from(pos.0.x),
                        f64::from(pos.0.y),
                        f64::from(pos.0.z),
                        1.0,
                    );
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn find_water_range_calculation() {
        let pos = Vector3::new(10.5, 64.0, -5.2);
        let (min, max) = TryFindWaterGoal::find_water_range(pos);

        assert_eq!(min.0.x, 8);
        assert_eq!(min.0.y, 62);
        assert_eq!(min.0.z, -8);

        assert_eq!(max.0.x, 12);
        assert_eq!(max.0.y, 64);
        assert_eq!(max.0.z, -4);
    }
}
