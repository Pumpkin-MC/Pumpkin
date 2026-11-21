use super::{Controls, Goal};
use crate::entity::mob::Mob;
use crate::world::World;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub struct RandomFloatAroundGoal {
    pub distance_to_blocks: i32,
}

impl Default for RandomFloatAroundGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomFloatAroundGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            distance_to_blocks: 0,
        }
    }

    #[must_use]
    pub const fn with_distance(distance_to_blocks: i32) -> Self {
        Self { distance_to_blocks }
    }

    fn choose_random_position(center: Vector3<f64>, mob: &dyn Mob) -> Vector3<f64> {
        let mut rng = mob.get_random();
        let target_x = center.x + f64::from(rng.random::<f32>() * 2.0 - 1.0) * 16.0;
        let target_y = center.y + f64::from(rng.random::<f32>() * 2.0 - 1.0) * 16.0;
        let target_z = center.z + f64::from(rng.random::<f32>() * 2.0 - 1.0) * 16.0;
        Vector3::new(target_x, target_y, target_z)
    }

    fn choose_random_position_with_restriction(
        mob: &dyn Mob,
        center: Vector3<f64>,
    ) -> Option<Vector3<f64>> {
        let target = Self::choose_random_position(center, mob);
        let block_pos = BlockPos::floored_v(target);
        let mob_entity = mob.get_mob_entity();
        if mob_entity.has_position_target()
            && !mob_entity.is_in_position_target_range_pos(&block_pos)
        {
            return None;
        }
        Some(target)
    }

    fn is_good_target(world: &World, target: Vector3<f64>, distance_to_blocks: i32) -> bool {
        const DIRS: [(i32, i32, i32); 6] = [
            (0, -1, 0),
            (0, 1, 0),
            (0, 0, -1),
            (0, 0, 1),
            (-1, 0, 0),
            (1, 0, 0),
        ];

        if distance_to_blocks <= 0 {
            return true;
        }
        let pos = BlockPos::floored_v(target);
        if !world.get_block_state(&pos).is_air() {
            return false;
        }
        for (dx, dy, dz) in DIRS {
            for i in 1..distance_to_blocks {
                let offset = BlockPos::new(pos.0.x + dx * i, pos.0.y + dy * i, pos.0.z + dz * i);
                if !world.get_block_state(&offset).is_air() {
                    return true;
                }
            }
        }
        false
    }

    fn get_suitable_fly_to_position(mob: &dyn Mob, distance_to_blocks: i32) -> Vector3<f64> {
        let entity = mob.get_entity();
        let center = entity.pos.load();
        let world = entity.world.load();
        let mut result = None;
        for _ in 0..64 {
            result = Self::choose_random_position_with_restriction(mob, center);
            if let Some(target) = result
                && Self::is_good_target(&world, target, distance_to_blocks)
            {
                return target;
            }
        }
        let mut final_pos = result.unwrap_or_else(|| Self::choose_random_position(center, mob));
        let pos = BlockPos::floored_v(final_pos);
        let height_y = f64::from(world.get_top_block(Vector2::new(pos.0.x, pos.0.z)));
        let min_y = f64::from(world.dimension.min_y);
        if height_y < final_pos.y && height_y > min_y {
            final_pos.y = center.y - (center.y - final_pos.y).abs();
        }
        final_pos
    }
}

impl Goal for RandomFloatAroundGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        let move_control = mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if move_control.has_wanted() {
            let wanted = move_control.get_wanted_position();
            let pos = mob.get_entity().pos.load();
            let dist_sq = pos.squared_distance_to_vec(&wanted);
            dist_sq < 1.0 || dist_sq > 3600.0
        } else {
            true
        }
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        let target = Self::get_suitable_fly_to_position(mob, self.distance_to_blocks);
        let mut move_control = mob
            .get_mob_entity()
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        move_control.set_wanted_position(target.x, target.y, target.z, 1.0);
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
