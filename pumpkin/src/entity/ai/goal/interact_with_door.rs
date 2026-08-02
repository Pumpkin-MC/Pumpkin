use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::world::World;
use pumpkin_data::block_properties::{BlockProperties, OakDoorLikeProperties};
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::position::BlockPos;

/// Enables mobs to interact with doors (open and close) while navigating.
/// Ported from vanilla's `DoorInteractGoal` and `OpenDoorGoal`.
///
/// This goal allows mobs to open mob-interactable doors when walking through them,
/// and optionally close them after passing through.
pub struct InteractWithDoorGoal {
    close_door_after: bool,
    door_pos: Option<BlockPos>,
    passed_door: bool,
    door_open_dir_x: f32,
    door_open_dir_z: f32,
}

impl InteractWithDoorGoal {
    #[must_use]
    pub const fn new(close_door_after: bool) -> Self {
        Self {
            close_door_after,
            door_pos: None,
            passed_door: false,
            door_open_dir_x: 0.0,
            door_open_dir_z: 0.0,
        }
    }

    /// Checks if a block is a mob-interactable door
    fn is_mob_interactable_door(world: &World, pos: &BlockPos) -> bool {
        let block = world.get_block(pos);
        block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_MOB_INTERACTABLE_DOORS)
    }

    /// Checks if the door at the given position is currently open
    fn is_door_open(world: &World, pos: &BlockPos) -> bool {
        let (block, state_id) = world.get_block_and_state_id(pos);
        let door_props = OakDoorLikeProperties::from_state_id(state_id, block);
        door_props.open
    }
}

impl Goal for InteractWithDoorGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let mob_entity = mob.get_mob_entity();
            let entity = &mob_entity.living_entity.entity;

            // Must have horizontal collision to be trying to open a door
            if !entity
                .horizontal_collision
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                return false;
            }

            // Check if there's an active path
            let navigator = mob_entity.navigator.lock().unwrap();
            let Some(path) = navigator.get_current_path() else {
                return false;
            };

            // Path must not be done
            if path.is_done() {
                return false;
            }

            // Scan through upcoming path nodes looking for doors
            let next_index = path.get_next_node_index();
            let node_count = path.get_node_count();
            let scan_end = std::cmp::min(next_index + 2, node_count);

            let world = entity.world.load_full();
            for i in next_index..scan_end {
                if let Some(node) = path.get_node(i) {
                    // Check at y + 1 as per vanilla (doors are two blocks tall)
                    let check_pos = BlockPos::new(node.pos.0.x, node.pos.0.y + 1, node.pos.0.z);

                    // Door must be within 1.5 blocks (2.25 squared distance)
                    let dx = check_pos.0.x as f64 + 0.5 - entity.pos.load().x;
                    let dz = check_pos.0.z as f64 + 0.5 - entity.pos.load().z;
                    let dist_sq = dx * dx + dz * dz;

                    if dist_sq <= 2.25 && Self::is_mob_interactable_door(&world, &check_pos) {
                        self.door_pos = Some(check_pos);
                        drop(navigator); // Release the lock before returning
                        return true;
                    }
                }
            }

            // Also check the block above the mob itself
            let mob_pos = entity.block_pos.load();
            let check_pos = BlockPos::new(mob_pos.0.x, mob_pos.0.y + 1, mob_pos.0.z);
            if Self::is_mob_interactable_door(&world, &check_pos) {
                self.door_pos = Some(check_pos);
                drop(navigator); // Release the lock before returning
                return true;
            }

            false
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            // Stop once we've passed through the door
            !self.passed_door
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.passed_door = false;
            if let Some(door_pos) = self.door_pos {
                // Calculate direction vector from mob to door
                // This will be used to detect when the mob has passed through
                let entity_pos = mob.get_entity().pos.load();
                self.door_open_dir_x = (door_pos.0.x as f32 + 0.5) - entity_pos.x as f32;
                self.door_open_dir_z = (door_pos.0.z as f32 + 0.5) - entity_pos.z as f32;
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(door_pos) = self.door_pos {
                let world = mob.get_entity().world.load_full();
                let entity = &mob.get_mob_entity().living_entity.entity;
                let entity_pos = entity.pos.load();

                // Calculate new direction vector
                let new_door_dir_x = (door_pos.0.x as f32 + 0.5) - entity_pos.x as f32;
                let new_door_dir_z = (door_pos.0.z as f32 + 0.5) - entity_pos.z as f32;

                // Dot product to check if we've passed through the door
                let dot =
                    self.door_open_dir_x * new_door_dir_x + self.door_open_dir_z * new_door_dir_z;
                if dot < 0.0 {
                    self.passed_door = true;
                    // Close the door if requested and it's open
                    if self.close_door_after && Self::is_door_open(&world, &door_pos) {
                        crate::block::blocks::doors::set_door_open(&world, &door_pos, false).await;
                    }
                    return;
                }

                // Open the door if it's not already open
                if !Self::is_door_open(&world, &door_pos) {
                    crate::block::blocks::doors::set_door_open(&world, &door_pos, true).await;
                }
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}
