use std::sync::Arc;

use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::poi_type::PoiType;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::poi::Occupancy;
use rand::RngExt;

use super::super::super::BrainTick;
use super::super::super::memory::{GlobalPos, MemoryModuleId, MemoryStatus, types};
use super::super::timed::Behavior;
use super::super::utils;
use crate::entity::EntityBase;
use crate::entity::ageable::AgeableMob;
use crate::entity::mob::Mob;
use crate::entity::passive::villager::VillagerEntity;

const DURATION: i32 = 350;
const BIRTH_DELAY: i32 = 275;
const BIRTH_DELAY_SPREAD: i32 = 50;
const CLOSE_ENOUGH_DISTANCE: i32 = 2;
const SPEED_MODIFIER: f32 = 0.5;
const INTERACTION_DISTANCE_SQR: f64 = 5.0;
const BED_SEARCH_RANGE: i32 = 48;
const HEART_CHANCE: i32 = 35;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 2] = [
    (types::BREED_TARGET.id(), MemoryStatus::ValuePresent),
    (
        types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
        MemoryStatus::ValuePresent,
    ),
];

pub struct VillagerMakeLove {
    birth_timestamp: i64,
}

impl Default for VillagerMakeLove {
    fn default() -> Self {
        Self::new()
    }
}

impl VillagerMakeLove {
    #[must_use]
    pub const fn new() -> Self {
        Self { birth_timestamp: 0 }
    }
}

fn breeding_target(tick: &BrainTick<'_>) -> Option<Arc<dyn EntityBase>> {
    tick.state
        .get(types::BREED_TARGET)
        .filter(|target| target.get_entity().entity_type == &EntityType::VILLAGER)
        .cloned()
}

fn is_breeding_possible(tick: &BrainTick<'_>) -> bool {
    let Some(target) = breeding_target(tick) else {
        return false;
    };
    let body_can_breed = tick
        .mob
        .cast_any()
        .downcast_ref::<VillagerEntity>()
        .is_some_and(VillagerEntity::can_breed);
    let target_can_breed = target
        .cast_any()
        .downcast_ref::<VillagerEntity>()
        .is_some_and(VillagerEntity::can_breed);
    utils::target_is_valid(&tick.visibility(), types::BREED_TARGET, |target| {
        target.get_entity().entity_type == &EntityType::VILLAGER
    }) && body_can_breed
        && target_can_breed
}

fn try_to_give_birth(tick: &BrainTick<'_>, target: &Arc<dyn EntityBase>) {
    let Some(child_bed) = take_vacant_bed(tick) else {
        broadcast_angry(tick, target.get_entity());
        broadcast_angry(tick, tick.mob.get_entity());
        return;
    };
    let Some(body) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() else {
        release_bed(tick, &child_bed);
        return;
    };
    let Some(target_villager) = target.cast_any().downcast_ref::<VillagerEntity>() else {
        release_bed(tick, &child_bed);
        return;
    };
    let Some(child) = body.get_breed_offspring(tick.world, target_villager) else {
        release_bed(tick, &child_bed);
        return;
    };
    body.set_age(6000);
    target_villager.set_age(6000);
    child.set_age(-24000);
    let pos = body.get_entity().pos.load();
    child.get_entity().set_pos(pos);
    child.get_entity().yaw.store(0.0);
    child.get_entity().pitch.store(0.0);
    tick.world.spawn_entity(child.clone());
    broadcast_hearts(tick, child.get_entity());
    give_bed_to_child(tick, &child, child_bed);
}

fn release_bed(tick: &BrainTick<'_>, bed: &BlockPos) {
    tick.world
        .portal_poi
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .release(bed);
}

fn give_bed_to_child(tick: &BrainTick<'_>, child: &Arc<VillagerEntity>, bed: BlockPos) {
    let Some(child_mob) = child.get_mob() else {
        return;
    };
    let Some(dimension) = Dimension::from_name(tick.world.dimension.minecraft_name) else {
        return;
    };
    child_mob.get_mob_entity().with_brain(child_mob, |brain| {
        brain.set(types::HOME, GlobalPos::new(dimension, bed));
    });
}

impl Behavior for VillagerMakeLove {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        DURATION
    }

    fn max_duration(&self) -> i32 {
        DURATION
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        is_breeding_possible(tick)
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.time <= self.birth_timestamp && is_breeding_possible(tick)
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        let Some(target) = breeding_target(tick) else {
            return;
        };
        utils::lock_gaze_and_walk_to_each_other(
            tick,
            &target,
            SPEED_MODIFIER,
            CLOSE_ENOUGH_DISTANCE,
        );
        broadcast_love_hearts(tick, target.get_entity());
        broadcast_love_hearts(tick, tick.mob.get_entity());
        let duration = BIRTH_DELAY + tick.mob.get_random().random_range(0..BIRTH_DELAY_SPREAD);
        self.birth_timestamp = tick.time + i64::from(duration);
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(target) = breeding_target(tick) else {
            return;
        };
        if tick
            .mob
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&target.get_entity().pos.load())
            > INTERACTION_DISTANCE_SQR
        {
            return;
        }
        utils::lock_gaze_and_walk_to_each_other(
            tick,
            &target,
            SPEED_MODIFIER,
            CLOSE_ENOUGH_DISTANCE,
        );
        if tick.time >= self.birth_timestamp {
            if let Some(body) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() {
                body.eat_and_digest_food();
            }
            if let Some(target_villager) = target.cast_any().downcast_ref::<VillagerEntity>() {
                target_villager.eat_and_digest_food();
            }
            try_to_give_birth(tick, &target);
        } else if tick.mob.get_random().random_range(0..HEART_CHANCE) == 0 {
            broadcast_hearts(tick, target.get_entity());
            broadcast_hearts(tick, tick.mob.get_entity());
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.erase(types::BREED_TARGET.id());
    }

    fn debug_name(&self) -> &'static str {
        "VillagerMakeLove"
    }
}

fn take_vacant_bed(tick: &BrainTick<'_>) -> Option<BlockPos> {
    let center = tick.mob.get_entity().block_pos.load();
    let candidates = tick
        .world
        .portal_poi
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_records_in_range(
            &|poi_type| poi_type == PoiType::Home,
            center,
            BED_SEARCH_RANGE,
            Occupancy::HasSpace,
        );
    let bed = candidates
        .into_iter()
        .find(|record| can_reach(tick.mob, record.pos, record.poi_type))?;
    tick.world
        .portal_poi
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .acquire_ticket(&bed.pos)
        .then_some(bed.pos)
}

fn can_reach(mob: &dyn Mob, poi_pos: BlockPos, poi_type: PoiType) -> bool {
    let mob_entity = mob.get_mob_entity();
    let mut navigator = mob_entity
        .navigator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    navigator
        .create_path_to_targets(
            &mob_entity.living_entity,
            &[poi_pos],
            poi_type.valid_range(),
        )
        .is_some_and(|path| path.can_reach())
}

fn broadcast_love_hearts(tick: &BrainTick<'_>, entity: &crate::entity::Entity) {
    tick.world.send_entity_status(
        entity,
        EntityStatus::InLoveHearts,
        ActorEventID::InLoveHearts,
    );
}

fn broadcast_hearts(tick: &BrainTick<'_>, entity: &crate::entity::Entity) {
    tick.world
        .send_entity_status(entity, EntityStatus::LoveHearts, ActorEventID::LoveHearts);
}

fn broadcast_angry(tick: &BrainTick<'_>, entity: &crate::entity::Entity) {
    tick.world.send_entity_status(
        entity,
        EntityStatus::VillagerAngry,
        ActorEventID::VillagerAngry,
    );
}
