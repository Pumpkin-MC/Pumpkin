use std::sync::PoisonError;

use pumpkin_data::dimension::Dimension;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashSet;

use crate::block::blocks::doors::DoorBlock;

use super::super::BrainTick;
use super::super::memory::{GlobalPos, MemoryStatus, types};
use super::OneShot;

const COOLDOWN_BEFORE_RERUNNING_IN_SAME_NODE: i32 = 20;
const SKIP_CLOSING_DOOR_IF_FURTHER_AWAY_THAN: f64 = 3.0;
const MAX_DISTANCE_TO_HOLD_DOOR_OPEN_FOR_OTHER_MOBS: f64 = 2.0;

#[must_use]
pub fn create() -> OneShot {
    let mut last_checked_node = None;
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
            // PATH memories own a copy; navigation has the current node index.
            let path = tick
                .mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .get_path()
                .cloned();
            let Some(path) = path.filter(|path| !path.not_started() && !path.is_done()) else {
                return false;
            };
            let (Some(from_node), Some(to_node)) = (path.get_previous_node(), path.get_next_node())
            else {
                return false;
            };
            if last_checked_node == Some(to_node.pos) {
                remaining_cooldown = COOLDOWN_BEFORE_RERUNNING_IN_SAME_NODE;
            } else {
                remaining_cooldown -= 1;
                if remaining_cooldown > 0 {
                    return false;
                }
            }
            last_checked_node = Some(to_node.pos);

            let mut doors = tick
                .brain
                .get(types::DOORS_TO_CLOSE)
                .cloned()
                .unwrap_or_default();
            let from_pos = from_node.pos;
            if is_interactable_door(tick, &from_pos) {
                DoorBlock::set_open(tick.world, &from_pos, true);
                remember_door_to_close(tick, &mut doors, from_pos);
            }
            let to_pos = to_node.pos;
            if is_interactable_door(tick, &to_pos) && !DoorBlock::is_open(tick.world, &to_pos) {
                DoorBlock::set_open(tick.world, &to_pos, true);
                remember_door_to_close(tick, &mut doors, to_pos);
            }
            close_doors_that_i_have_opened_or_passed_through(
                tick,
                Some(from_pos),
                Some(to_pos),
                &mut doors,
            );
            tick.brain.set(types::DOORS_TO_CLOSE, doors);
            true
        },
    )
}

fn is_interactable_door(tick: &BrainTick<'_>, position: &BlockPos) -> bool {
    let block = tick.world.get_block(position);
    block.has_tag(&tag::Block::MINECRAFT_MOB_INTERACTABLE_DOORS)
        && block.has_tag(&tag::Block::MINECRAFT_DOORS)
}

fn close_doors_that_i_have_opened_or_passed_through(
    tick: &BrainTick<'_>,
    moving_from: Option<BlockPos>,
    moving_to: Option<BlockPos>,
    doors: &mut FxHashSet<GlobalPos>,
) {
    doors.retain(|door| {
        if moving_from == Some(door.pos) || moving_to == Some(door.pos) {
            return true;
        }
        if !is_door_too_far_away(tick, door)
            && is_interactable_door(tick, &door.pos)
            && DoorBlock::is_open(tick.world, &door.pos)
            && !are_other_mobs_coming_through_door(tick, &door.pos)
        {
            DoorBlock::set_open(tick.world, &door.pos, false);
        }
        false
    });
}

fn are_other_mobs_coming_through_door(tick: &BrainTick<'_>, door: &BlockPos) -> bool {
    tick.brain
        .get(types::NEAREST_LIVING_ENTITIES)
        .is_some_and(|nearest| {
            nearest.iter().any(|other| {
                if other.get_entity().entity_type != tick.mob.get_entity().entity_type {
                    return false;
                }
                let position = other.get_entity().pos.load();
                if door.dist_to_center_sqr(position.x, position.y, position.z)
                    >= MAX_DISTANCE_TO_HOLD_DOOR_OPEN_FOR_OTHER_MOBS
                        * MAX_DISTANCE_TO_HOLD_DOOR_OPEN_FOR_OTHER_MOBS
                {
                    return false;
                }
                let Some(other_mob) = other.get_mob() else {
                    return false;
                };
                let navigation = other_mob
                    .get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                navigation.get_path().is_some_and(|path| {
                    !path.is_done()
                        && path.get_previous_node().is_some_and(|previous| {
                            previous.pos == *door
                                || path.get_next_node().is_some_and(|next| next.pos == *door)
                        })
                })
            })
        })
}

fn is_door_too_far_away(tick: &BrainTick<'_>, door: &GlobalPos) -> bool {
    let position = tick.mob.get_entity().pos.load();
    door.dimension.id != tick.world.dimension.id
        || door
            .pos
            .dist_to_center_sqr(position.x, position.y, position.z)
            >= SKIP_CLOSING_DOOR_IF_FURTHER_AWAY_THAN * SKIP_CLOSING_DOOR_IF_FURTHER_AWAY_THAN
}

fn remember_door_to_close(
    tick: &BrainTick<'_>,
    doors: &mut FxHashSet<GlobalPos>,
    position: BlockPos,
) {
    if let Some(dimension) = Dimension::from_name(tick.world.dimension.minecraft_name) {
        doors.insert(GlobalPos::new(dimension, position));
    }
}
