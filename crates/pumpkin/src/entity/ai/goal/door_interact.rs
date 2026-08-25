use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use std::sync::atomic::Ordering;

use super::{Controls, Goal, GoalFuture};
use crate::block::blocks::doors::DoorBlock;
use crate::entity::mob::Mob;

pub struct DoorInteractGoal {
    pub door_pos: BlockPos,
    pub has_door: bool,
    pub passed: bool,
    pub door_open_dir_x: f32,
    pub door_open_dir_z: f32,
}

impl Default for DoorInteractGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl DoorInteractGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            door_pos: BlockPos::ZERO,
            has_door: false,
            passed: false,
            door_open_dir_x: 0.0,
            door_open_dir_z: 0.0,
        }
    }

    pub fn is_open(&mut self, mob: &dyn Mob) -> bool {
        if !self.has_door {
            return false;
        }
        let world = mob.get_entity().world.load();
        let (block, _) = world.get_block_and_state_id(&self.door_pos);
        if !block.has_tag(&tag::Block::MINECRAFT_DOORS) {
            self.has_door = false;
            return false;
        }
        DoorBlock::is_open(&world, &self.door_pos)
    }

    pub async fn set_open(&mut self, mob: &dyn Mob, open: bool) {
        if self.has_door {
            let world = mob.get_entity().world.load_full();
            let (block, _) = world.get_block_and_state_id(&self.door_pos);
            if block.has_tag(&tag::Block::MINECRAFT_DOORS) {
                DoorBlock::set_open(&world, &self.door_pos, open).await;
            }
        }
    }

    pub fn can_use(&mut self, mob: &dyn Mob) -> bool {
        if !mob
            .get_entity()
            .horizontal_collision
            .load(Ordering::Relaxed)
        {
            return false;
        }

        let navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let Some(path) = navigator.get_path() else {
            return false;
        };

        if path.is_done() {
            return false;
        }

        let world = mob.get_entity().world.load();
        let mob_pos = mob.get_entity().pos.load();
        let limit = (path.get_next_node_index() + 2).min(path.get_node_count());

        for i in 0..limit {
            let Some(node) = path.get_node(i) else {
                continue;
            };
            let door_pos = BlockPos::new(node.pos.0.x, node.pos.0.y + 1, node.pos.0.z);
            let dx = mob_pos.x - f64::from(door_pos.0.x);
            let dz = mob_pos.z - f64::from(door_pos.0.z);
            let dist_sqr = dx * dx + dz * dz;

            if dist_sqr <= 2.25 {
                self.door_pos = door_pos;
                self.has_door = DoorBlock::is_wooden_door(&world, &self.door_pos);
                if self.has_door {
                    return true;
                }
            }
        }

        let above_pos = mob.get_entity().block_pos.load().up();
        self.door_pos = above_pos;
        self.has_door = DoorBlock::is_wooden_door(&world, &self.door_pos);
        self.has_door
    }

    #[must_use]
    pub const fn can_continue_to_use(&self) -> bool {
        !self.passed
    }

    pub fn start_interaction(&mut self, mob: &dyn Mob) {
        self.passed = false;
        let mob_pos = mob.get_entity().pos.load();
        self.door_open_dir_x = (self.door_pos.0.x as f32 + 0.5) - mob_pos.x as f32;
        self.door_open_dir_z = (self.door_pos.0.z as f32 + 0.5) - mob_pos.z as f32;
    }

    pub fn tick_interaction(&mut self, mob: &dyn Mob) {
        let mob_pos = mob.get_entity().pos.load();
        let new_door_dir_x = (self.door_pos.0.x as f32 + 0.5) - mob_pos.x as f32;
        let new_door_dir_z = (self.door_pos.0.z as f32 + 0.5) - mob_pos.z as f32;
        let dot = self.door_open_dir_x * new_door_dir_x + self.door_open_dir_z * new_door_dir_z;
        if dot < 0.0 {
            self.passed = true;
        }
    }
}

impl Goal for DoorInteractGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.can_use(mob) })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.can_continue_to_use() })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.start_interaction(mob);
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.tick_interaction(mob);
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passed_when_crossing_door() {
        let mut goal = DoorInteractGoal::new();
        goal.door_pos = BlockPos::new(10, 64, 10);

        // Simulate start: mob at (10.5, 64.0, 8.0), moving towards door at Z=10.5
        // door_open_dir = (10.5 - 10.5, 10.5 - 8.0) = (0.0, 2.5)
        goal.door_open_dir_x = 0.0;
        goal.door_open_dir_z = 2.5;
        goal.passed = false;

        assert!(goal.can_continue_to_use());

        // Still before door: mob at (10.5, 64.0, 9.5) -> new_door_dir = (0.0, 1.0)
        // dot = 0*0 + 2.5*1.0 = 2.5 > 0
        let new_x = 0.0f32;
        let new_z = 1.0f32;
        let dot = goal.door_open_dir_x * new_x + goal.door_open_dir_z * new_z;
        if dot < 0.0 {
            goal.passed = true;
        }
        assert!(!goal.passed);
        assert!(goal.can_continue_to_use());

        // Crossed door: mob at (10.5, 64.0, 12.0) -> new_door_dir = (0.0, -1.5)
        // dot = 0*0 + 2.5*(-1.5) = -3.75 < 0
        let new_x2 = 0.0f32;
        let new_z2 = -1.5f32;
        let dot2 = goal.door_open_dir_x * new_x2 + goal.door_open_dir_z * new_z2;
        if dot2 < 0.0 {
            goal.passed = true;
        }
        assert!(goal.passed);
        assert!(!goal.can_continue_to_use());
    }
}
