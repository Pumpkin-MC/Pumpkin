use std::sync::Arc;

use pumpkin_data::dimension::Dimension;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use rustc_hash::FxHashSet;

use super::super::OneShot;
use crate::block::blocks::doors::DoorBlock;
use crate::entity::EntityBase;
use crate::entity::ai::brain::BrainState;
use crate::entity::ai::brain::memory::{GlobalPos, MemoryStatus, types};
use crate::entity::mob::Mob;
use crate::world::World;

const COOLDOWN_BEFORE_RERUNNING_IN_SAME_NODE: i32 = 20;
const SKIP_CLOSING_DOOR_IF_FURTHER_AWAY_THAN: f64 = 3.0;
const MAX_DISTANCE_TO_HOLD_DOOR_OPEN_FOR_OTHER_MOBS: f64 = 2.0;

#[must_use]
pub fn create() -> OneShot {
    let mut last_checked_node: Option<BlockPos> = None;
    let mut remaining_cooldown = 0;

    OneShot::new(
        "InteractWithDoor",
        vec![
            (types::PATH.id(), MemoryStatus::ValuePresent),
            (types::DOORS_TO_CLOSE.id(), MemoryStatus::Registered),
            (
                types::NEAREST_LIVING_ENTITIES.id(),
                MemoryStatus::Registered,
            ),
        ],
        move |tick| {
            let Some(path) = tick.state.get(types::PATH) else {
                return false;
            };
            if path.not_started() || path.is_done() {
                return false;
            }
            let Some(next_node) = path.get_next_node() else {
                return false;
            };
            let next_pos = next_node.pos;
            let previous_pos = path.get_previous_node().map(|node| node.pos);

            let (new_cooldown, skip) =
                cooldown_decision(last_checked_node == Some(next_pos), remaining_cooldown);
            remaining_cooldown = new_cooldown;
            if skip {
                return false;
            }
            last_checked_node = Some(next_pos);

            let Some(from_pos) = previous_pos else {
                return false;
            };
            let Some(dimension) = Dimension::from_name(tick.world.dimension.minecraft_name) else {
                return false;
            };

            let mut doors = tick.state.get(types::DOORS_TO_CLOSE).cloned();
            let from_block = tick.world.get_block_and_state_id(&from_pos).0;
            if is_mob_interactable_door(from_block) {
                if !DoorBlock::is_open(tick.world, &from_pos) {
                    DoorBlock::set_open(tick.world, &from_pos, true);
                }
                doors = Some(remember_door_to_close(
                    tick.state, doors, dimension, from_pos,
                ));
            }

            let to_pos = next_pos;
            let to_block = tick.world.get_block_and_state_id(&to_pos).0;
            if is_mob_interactable_door(to_block) && !DoorBlock::is_open(tick.world, &to_pos) {
                DoorBlock::set_open(tick.world, &to_pos, true);
                doors = Some(remember_door_to_close(tick.state, doors, dimension, to_pos));
            }

            if let Some(door_set) = doors.as_mut() {
                let nearest_entities = tick
                    .state
                    .get(types::NEAREST_LIVING_ENTITIES)
                    .map(Vec::as_slice);
                close_doors_that_i_have_opened_or_passed_through(
                    tick.world,
                    tick.mob,
                    Some(from_pos),
                    Some(to_pos),
                    door_set,
                    nearest_entities,
                );
            }
            tick.state.set_optional(types::DOORS_TO_CLOSE, doors);
            true
        },
    )
}

const fn cooldown_decision(same_node: bool, remaining_cooldown: i32) -> (i32, bool) {
    if same_node {
        (COOLDOWN_BEFORE_RERUNNING_IN_SAME_NODE, false)
    } else {
        let remaining = remaining_cooldown - 1;
        (remaining, remaining > 0)
    }
}

fn is_mob_interactable_door(block: &'static pumpkin_data::Block) -> bool {
    block.has_tag(&tag::Block::MINECRAFT_MOB_INTERACTABLE_DOORS)
        && block.has_tag(&tag::Block::MINECRAFT_DOORS)
}

fn closer_to_center_than(pos: BlockPos, entity_pos: Vector3<f64>, distance: f64) -> bool {
    pos.dist_to_center_sqr(entity_pos.x, entity_pos.y, entity_pos.z) < distance * distance
}

#[expect(
    clippy::implicit_hasher,
    reason = "the doors memory is an FxHashSet, so the concrete hasher keeps them in sync"
)]
pub fn close_doors_that_i_have_opened_or_passed_through(
    world: &Arc<World>,
    body: &dyn Mob,
    moving_from: Option<BlockPos>,
    moving_to: Option<BlockPos>,
    doors: &mut FxHashSet<GlobalPos>,
    nearest_entities: Option<&[Arc<dyn EntityBase>]>,
) {
    let body_pos = body.get_entity().pos.load();

    doors.retain(|door_global_pos| {
        let door_pos = door_global_pos.pos;
        if moving_from.is_some_and(|pos| pos == door_pos)
            || moving_to.is_some_and(|pos| pos == door_pos)
        {
            return true;
        }
        if is_door_too_far_away(world, body_pos, door_global_pos) {
            return false;
        }
        let block = world.get_block_and_state_id(&door_pos).0;
        if !is_mob_interactable_door(block) {
            return false;
        }
        if !DoorBlock::is_open(world, &door_pos) {
            return false;
        }
        if are_other_mobs_coming_through_door(body, door_pos, nearest_entities) {
            return false;
        }
        DoorBlock::set_open(world, &door_pos, false);
        false
    });
}

fn are_other_mobs_coming_through_door(
    body: &dyn Mob,
    door_pos: BlockPos,
    nearest_entities: Option<&[Arc<dyn EntityBase>]>,
) -> bool {
    let Some(entities) = nearest_entities else {
        return false;
    };
    entities.iter().any(|other| {
        let other_entity = other.get_entity();
        other_entity.entity_type == body.get_entity().entity_type
            && closer_to_center_than(
                door_pos,
                other_entity.pos.load(),
                MAX_DISTANCE_TO_HOLD_DOOR_OPEN_FOR_OTHER_MOBS,
            )
            && other
                .get_mob()
                .is_some_and(|other_mob| is_mob_coming_through_door(other_mob, door_pos))
    })
}

fn is_mob_coming_through_door(other_mob: &dyn Mob, door_pos: BlockPos) -> bool {
    let mob_entity = other_mob.get_mob_entity();
    if !mob_entity.brain_has_memory_value(types::PATH.id()) {
        return false;
    }
    mob_entity.with_brain(other_mob, |brain| {
        let Some(path) = brain.get(types::PATH) else {
            return false;
        };
        if path.is_done() {
            return false;
        }
        let Some(moving_from_node) = path.get_previous_node() else {
            return false;
        };
        door_pos == moving_from_node.pos
            || path
                .get_next_node()
                .is_some_and(|node| door_pos == node.pos)
    })
}

fn is_door_too_far_away(
    world: &World,
    body_pos: Vector3<f64>,
    door_global_pos: &GlobalPos,
) -> bool {
    door_global_pos.dimension.id != world.dimension.id
        || !closer_to_center_than(
            door_global_pos.pos,
            body_pos,
            SKIP_CLOSING_DOOR_IF_FURTHER_AWAY_THAN,
        )
}

fn remember_door_to_close(
    state: &mut BrainState,
    doors: Option<FxHashSet<GlobalPos>>,
    dimension: &'static Dimension,
    door_pos: BlockPos,
) -> FxHashSet<GlobalPos> {
    let global_door_pos = GlobalPos::new(dimension, door_pos);
    doors.map_or_else(
        || {
            let mut set = FxHashSet::default();
            set.insert(global_door_pos);
            state.set(types::DOORS_TO_CLOSE, set.clone());
            set
        },
        |mut set| {
            set.insert(global_door_pos);
            set
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooldown_resets_on_the_same_node_and_counts_down_otherwise() {
        assert_eq!(cooldown_decision(true, 0), (20, false));
        assert_eq!(cooldown_decision(true, 19), (20, false));
        assert_eq!(cooldown_decision(false, 20), (19, true));
        assert_eq!(cooldown_decision(false, 2), (1, true));
        assert_eq!(cooldown_decision(false, 1), (0, false));
        assert_eq!(cooldown_decision(false, 0), (-1, false));
    }
}
