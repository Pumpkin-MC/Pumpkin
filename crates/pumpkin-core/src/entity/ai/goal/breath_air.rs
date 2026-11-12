use pumpkin_data::{Block, fluid::Fluid};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::{Controls, Goal};
use crate::entity::{ai::pathfinder::node::PathComputationType, mob::Mob};

pub struct BreathAirGoal;

impl BreathAirGoal {
    pub(crate) const MOVEMENT_SPEED: f32 = 0.02;

    fn find_air_position(mob: &dyn Mob) {
        let entity = mob.get_entity();
        let origin = entity.block_pos.load();
        let world = entity.world.load();
        let mut destination = origin.offset(Vector3::new(0, 8, 0));
        'search: for (x, z) in [(0, 0), (0, -1), (1, 0), (0, 1), (-1, 0)] {
            for y in 0..=8 {
                let pos = origin.offset(Vector3::new(x, y, z));
                let (block, state) = world.get_block_and_state(&pos);
                if (world.get_fluid(&pos) == &Fluid::EMPTY || block == &Block::BUBBLE_COLUMN)
                    && world
                        .block_registry
                        .is_pathfindable(block, state, PathComputationType::Land)
                {
                    destination = pos;
                    break 'search;
                }
            }
        }
        let living = &mob.get_mob_entity().living_entity;
        mob.get_mob_entity().steered().navigator().move_to_pos(
            BlockPos::new(destination.0.x, destination.0.y + 1, destination.0.z),
            1.0,
            living,
        );
    }
}

impl Goal for BreathAirGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        mob.get_entity()
            .air_supply
            .load(std::sync::atomic::Ordering::Relaxed)
            < 140
    }

    fn can_stop(&self) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity().steered().navigator().stop();
        Self::find_air_position(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        Self::find_air_position(mob);
        let living = &mob.get_mob_entity().living_entity;
        living.entity.update_velocity_from_input(
            living.movement_input.load(),
            f64::from(Self::MOVEMENT_SPEED),
        );
        living
            .entity
            .move_entity(mob, living.entity.velocity.load());
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
