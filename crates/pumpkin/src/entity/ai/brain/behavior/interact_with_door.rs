use std::sync::Arc;

use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashSet;

use crate::block::blocks::doors::DoorBlock;
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::node::Node;
use crate::world::World;

use super::super::BrainTick;
use super::super::memory::{GlobalPos, MemoryStatus, types};
use super::one_shot::OneShot;
use super::utils::global_pos_in;

const COOLDOWN_BEFORE_RERUNNING_IN_SAME_NODE: i32 = 20;
const SKIP_CLOSING_DOOR_IF_FURTHER_AWAY_THAN: f64 = 3.0;
const MAX_DISTANCE_TO_HOLD_DOOR_OPEN_FOR_OTHER_MOBS: f64 = 2.0;

#[must_use]
pub fn interact_with_door() -> OneShot {
    let mut last_checked_node: Option<BlockPos> = None;
    let mut remaining_cooldown = 0;

    OneShot::with_required(
        "InteractWithDoor",
        vec![(types::PATH.id(), MemoryStatus::ValuePresent)],
        vec![
            types::DOORS_TO_CLOSE.id(),
            types::NEAREST_LIVING_ENTITIES.id(),
        ],
        move |tick| {
            let Some((from_pos, to_pos)) = tick.brain.get(types::PATH).and_then(|path| {
                if path.not_started() || path.is_done() {
                    return None;
                }
                Some((
                    path.get_previous_node().map(|node: &Node| node.pos),
                    path.get_next_node().map(|node: &Node| node.pos),
                ))
            }) else {
                return false;
            };

            if last_checked_node == to_pos {
                remaining_cooldown = COOLDOWN_BEFORE_RERUNNING_IN_SAME_NODE;
            } else {
                remaining_cooldown -= 1;
                if remaining_cooldown > 0 {
                    return false;
                }
            }
            last_checked_node = to_pos;

            let world = tick.world;
            let mut doors: FxHashSet<GlobalPos> = tick
                .brain
                .get(types::DOORS_TO_CLOSE)
                .cloned()
                .unwrap_or_default();

            if let Some(from_pos) = from_pos
                && is_mob_interactable_door(world, &from_pos)
            {
                if !DoorBlock::is_open(world, &from_pos) {
                    DoorBlock::set_open(world, &from_pos, true);
                }
                if let Some(door) = global_pos_in(world, from_pos) {
                    doors.insert(door);
                }
            }
            if let Some(to_pos) = to_pos
                && is_mob_interactable_door(world, &to_pos)
                && !DoorBlock::is_open(world, &to_pos)
            {
                DoorBlock::set_open(world, &to_pos, true);
                if let Some(door) = global_pos_in(world, to_pos) {
                    doors.insert(door);
                }
            }

            close_doors_i_opened_or_passed_through(tick, &mut doors, from_pos, to_pos);
            tick.brain.set(types::DOORS_TO_CLOSE, doors);
            true
        },
    )
}

fn is_mob_interactable_door(world: &World, pos: &BlockPos) -> bool {
    world
        .get_block(pos)
        .has_tag(&tag::Block::MINECRAFT_MOB_INTERACTABLE_DOORS)
}

fn close_doors_i_opened_or_passed_through(
    tick: &BrainTick<'_>,
    doors: &mut FxHashSet<GlobalPos>,
    moving_from: Option<BlockPos>,
    moving_to: Option<BlockPos>,
) {
    let world = tick.world;
    let body_pos = tick.mob.get_entity().pos.load();
    let nearest = tick.brain.get(types::NEAREST_LIVING_ENTITIES);

    doors.retain(|door| {
        let door_pos = door.pos;
        if moving_from == Some(door_pos) || moving_to == Some(door_pos) {
            return true;
        }
        if world.dimension.id != door.dimension.id
            || door_pos.dist_to_center_sqr(body_pos.x, body_pos.y, body_pos.z)
                >= SKIP_CLOSING_DOOR_IF_FURTHER_AWAY_THAN * SKIP_CLOSING_DOOR_IF_FURTHER_AWAY_THAN
        {
            return false;
        }
        if !is_mob_interactable_door(world, &door_pos) || !DoorBlock::is_open(world, &door_pos) {
            return false;
        }
        if other_mobs_are_coming_through(tick, &door_pos, nearest) {
            return false;
        }
        DoorBlock::set_open(world, &door_pos, false);
        false
    });
}

fn other_mobs_are_coming_through(
    tick: &BrainTick<'_>,
    door_pos: &BlockPos,
    nearest: Option<&Vec<Arc<dyn EntityBase>>>,
) -> bool {
    let Some(nearest) = nearest else {
        return false;
    };
    let body_type = tick.mob.get_entity().entity_type;
    nearest.iter().any(|other| {
        if other.get_entity().entity_type != body_type {
            return false;
        }
        let other_pos = other.get_entity().pos.load();
        if door_pos.dist_to_center_sqr(other_pos.x, other_pos.y, other_pos.z)
            >= MAX_DISTANCE_TO_HOLD_DOOR_OPEN_FOR_OTHER_MOBS
                * MAX_DISTANCE_TO_HOLD_DOOR_OPEN_FOR_OTHER_MOBS
        {
            return false;
        }
        is_mob_coming_through_door(other.as_ref(), door_pos)
    })
}

// Contended lock reads as "not coming through"
fn is_mob_coming_through_door(other: &dyn EntityBase, door_pos: &BlockPos) -> bool {
    let Some(mob_entity) = other.as_mob_entity() else {
        return false;
    };
    let Ok(brain) = mob_entity.brain.try_lock() else {
        return false;
    };
    brain.get(types::PATH).is_some_and(|path| {
        if path.is_done() {
            return false;
        }
        let Some(from) = path.get_previous_node() else {
            return false;
        };
        from.pos == *door_pos || path.get_next_node().is_some_and(|to| to.pos == *door_pos)
    })
}
