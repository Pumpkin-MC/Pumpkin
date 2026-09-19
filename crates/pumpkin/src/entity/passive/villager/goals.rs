use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::Block;
use pumpkin_data::block_properties::{BedPart, WhiteBedLikeProperties as BedProperties};
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::{Entity, EntityBase, r#type::from_type};
use crate::world::World;

/// Sends a status to both editions. Bedrock only shows one if it is named explicitly,
/// and the statuses villagers use carry the same name on each.
fn send_status(entity: &Entity, status: EntityStatus) {
    let bedrock = match status {
        EntityStatus::LoveHearts => Some(ActorEventID::LoveHearts),
        EntityStatus::InLoveHearts => Some(ActorEventID::InLoveHearts),
        EntityStatus::VillagerAngry => Some(ActorEventID::VillagerAngry),
        _ => None,
    };
    entity
        .world
        .load()
        .send_entity_status(entity, status, bedrock);
}

#[must_use]
pub fn bed_head_at(world: &World, position: BlockPos) -> Option<BlockPos> {
    let (block, state) = world.get_block_and_state(&position);
    if !block.has_tag(&tag::Block::MINECRAFT_VILLAGERS_CAN_SLEEP_ON_BED) {
        return None;
    }
    let bed_props = BedProperties::from_state_id(state.id);
    Some(if bed_props.part == BedPart::Head {
        position
    } else {
        position.offset(bed_props.facing.to_offset())
    })
}

#[must_use]
pub fn is_bed_free(world: &World, bed_head_pos: BlockPos) -> bool {
    let (block, state) = world.get_block_and_state(&bed_head_pos);
    if !block.has_tag(&tag::Block::MINECRAFT_VILLAGERS_CAN_SLEEP_ON_BED) {
        return false;
    }
    let bed_props = BedProperties::from_state_id(state.id);
    bed_props.part == BedPart::Head && !bed_props.occupied
}

#[must_use]
pub fn find_free_bed(
    world: &World,
    origin: BlockPos,
    horizontal_range: i32,
    vertical_range: i32,
    claimant: Option<i32>,
) -> Option<BlockPos> {
    let start = BlockPos::new(
        origin.0.x - horizontal_range,
        origin.0.y - vertical_range,
        origin.0.z - horizontal_range,
    );
    let end = BlockPos::new(
        origin.0.x + horizontal_range,
        origin.0.y + vertical_range,
        origin.0.z + horizontal_range,
    );

    let origin_pos = origin.to_centered_f64();
    let claimed = {
        let aabb = BoundingBox::new(
            origin_pos.sub_raw(32.0, 16.0, 32.0),
            origin_pos.add_raw(32.0, 16.0, 32.0),
        );
        world
            .get_entities_at_box(&aabb)
            .into_iter()
            .filter(|entity| {
                entity.get_entity().entity_type == &EntityType::VILLAGER
                    && claimant.is_none_or(|id| entity.get_entity().entity_id != id)
            })
            .filter_map(|entity| entity.get_home_pos())
            .collect::<Vec<_>>()
    };

    let mut best: Option<(f64, BlockPos)> = None;
    for position in BlockPos::iterate(start, end) {
        let Some(bed_head_pos) = bed_head_at(world, position) else {
            continue;
        };
        if claimed.contains(&bed_head_pos) || !is_bed_free(world, bed_head_pos) {
            continue;
        }
        let distance = bed_head_pos
            .to_centered_f64()
            .squared_distance_to_vec(&origin_pos);
        if best.is_none_or(|(best_distance, _)| distance < best_distance) {
            best = Some((distance, bed_head_pos));
        }
    }
    best.map(|(_, bed)| bed)
}

pub struct SleepInBedGoal {
    speed: f64,
    bed: Option<BlockPos>,
    slept: bool,
}

#[must_use]
pub fn is_rest_time(world: &World) -> bool {
    let daytime = world.get_time_of_day().rem_euclid(24_000);
    (12_000..23_000).contains(&daytime)
}

impl SleepInBedGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            speed,
            bed: None,
            slept: false,
        }
    }

    fn is_rest_time(mob: &dyn Mob) -> bool {
        is_rest_time(&mob.get_entity().world.load())
    }

    fn is_bed_intact(mob: &dyn Mob, bed_head_pos: BlockPos) -> bool {
        let world = mob.get_entity().world.load();
        let (block, state) = world.get_block_and_state(&bed_head_pos);
        block.has_tag(&tag::Block::MINECRAFT_VILLAGERS_CAN_SLEEP_ON_BED)
            && BedProperties::from_state_id(state.id).part == BedPart::Head
    }

    fn is_bed_free(mob: &dyn Mob, bed_head_pos: BlockPos) -> bool {
        is_bed_free(&mob.get_entity().world.load(), bed_head_pos)
    }
}

impl Goal for SleepInBedGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(bed) = mob.get_home() else {
            return false;
        };
        if !Self::is_rest_time(mob)
            || mob.get_trading_player().is_some()
            || mob.get_entity().has_passengers()
        {
            return false;
        }
        if mob.is_sleeping() {
            if !Self::is_bed_intact(mob, bed) {
                return false;
            }
        } else if !mob.is_ready_to_sleep() || !Self::is_bed_free(mob, bed) {
            return false;
        }
        self.bed = Some(bed);
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(bed) = self.bed else {
            return false;
        };
        if mob.get_home() != Some(bed) || !Self::is_rest_time(mob) {
            return false;
        }
        if mob.is_sleeping() {
            return Self::is_bed_intact(mob, bed);
        }
        if self.slept {
            return false;
        }
        let position = mob.get_mob_entity().living_entity.entity.pos.load();
        let at_bed = bed.to_centered_f64().squared_distance_to_vec(&position) <= 4.0;
        Self::is_bed_free(mob, bed) && (at_bed || !mob.is_navigator_idle())
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.slept = mob.is_sleeping();
        if mob.is_sleeping() {
            return;
        }
        if let Some(bed) = self.bed {
            let entity = &mob.get_mob_entity().living_entity.entity;
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_progress(NavigatorGoal::new(
                    entity.pos.load(),
                    bed.to_centered_f64(),
                    self.speed,
                ));
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(bed) = self.bed else {
            return;
        };
        if mob.is_sleeping() {
            return;
        }
        if self.slept || !mob.is_ready_to_sleep() {
            return;
        }
        let position = mob.get_mob_entity().living_entity.entity.pos.load();
        if bed.to_centered_f64().squared_distance_to_vec(&position) <= 4.0 && mob.sleep_in_bed(bed)
        {
            self.slept = true;
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .stop();
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if mob.is_sleeping() {
            mob.wake_up();
        }
        self.slept = false;
        self.bed = None;
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP
    }
}

pub struct JumpOnBedGoal {
    speed: f64,
    target_bed: Option<BlockPos>,
    remaining_time_to_reach_bed: i32,
    remaining_jumps: i32,
    remaining_cooldown_until_next_jump: i32,
    ticks_until_search: i32,
}

impl JumpOnBedGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            speed,
            target_bed: None,
            remaining_time_to_reach_bed: 0,
            remaining_jumps: 0,
            remaining_cooldown_until_next_jump: 0,
            ticks_until_search: 0,
        }
    }

    fn is_baby(mob: &dyn Mob) -> bool {
        mob.get_entity().age.load(Ordering::Relaxed) < 0
    }

    fn is_jumpable(mob: &dyn Mob, position: BlockPos) -> bool {
        let world = mob.get_entity().world.load();
        let (block, _state) = world.get_block_and_state(&position);
        block.has_tag(&tag::Block::MINECRAFT_VILLAGER_BABIES_CAN_JUMP_ON_BED)
    }

    fn on_or_over_bed(mob: &dyn Mob) -> bool {
        let position = mob.get_entity().block_pos.load();
        Self::is_jumpable(mob, position) || Self::is_jumpable(mob, position.down())
    }

    fn on_bed_surface(mob: &dyn Mob) -> bool {
        Self::is_jumpable(mob, mob.get_entity().block_pos.load())
    }

    fn nearest_bed(mob: &dyn Mob) -> Option<BlockPos> {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let origin = entity.block_pos.load();
        let start = BlockPos::new(origin.0.x - 8, origin.0.y - 4, origin.0.z - 8);
        let end = BlockPos::new(origin.0.x + 8, origin.0.y + 4, origin.0.z + 8);

        let position = entity.pos.load();
        let mut best: Option<(f64, BlockPos)> = None;
        for candidate in BlockPos::iterate(start, end) {
            let (block, _state) = world.get_block_and_state(&candidate);
            if !block.has_tag(&tag::Block::MINECRAFT_VILLAGER_BABIES_CAN_JUMP_ON_BED) {
                continue;
            }
            let distance = candidate
                .to_centered_f64()
                .squared_distance_to_vec(&position);
            if best.is_none_or(|(best_distance, _)| distance < best_distance) {
                best = Some((distance, candidate));
            }
        }
        best.map(|(_, bed)| bed)
    }

    fn tired_of_walking(&self, mob: &dyn Mob) -> bool {
        !Self::on_or_over_bed(mob) && self.remaining_time_to_reach_bed <= 0
    }

    fn tired_of_jumping(&self, mob: &dyn Mob) -> bool {
        Self::on_or_over_bed(mob) && self.remaining_jumps <= 0
    }
}

impl Goal for JumpOnBedGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !Self::is_baby(mob) || mob.is_sleeping() {
            return false;
        }
        if self.ticks_until_search > 0 {
            self.ticks_until_search -= 1;
            return false;
        }
        self.ticks_until_search = 20;
        self.target_bed = Self::nearest_bed(mob);
        self.target_bed.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(target_bed) = self.target_bed else {
            return false;
        };
        Self::is_baby(mob)
            && !mob.is_sleeping()
            && Self::is_jumpable(mob, target_bed)
            && !self.tired_of_walking(mob)
            && !self.tired_of_jumping(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(target_bed) = self.target_bed else {
            return;
        };
        self.remaining_time_to_reach_bed = 100;
        self.remaining_jumps = mob.get_random().random_range(3..=6);
        self.remaining_cooldown_until_next_jump = 0;

        let entity = mob.get_entity();
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_progress(NavigatorGoal::new(
                entity.pos.load(),
                target_bed.to_centered_f64(),
                self.speed,
            ));
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target_bed = None;
        self.remaining_time_to_reach_bed = 0;
        self.remaining_jumps = 0;
        self.remaining_cooldown_until_next_jump = 0;
        let mob_entity = mob.get_mob_entity();
        mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
        mob_entity
            .living_entity
            .jumping
            .store(false, Ordering::SeqCst);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if !Self::on_or_over_bed(mob) {
            self.remaining_time_to_reach_bed -= 1;
            return;
        }
        if self.remaining_cooldown_until_next_jump > 0 {
            self.remaining_cooldown_until_next_jump -= 1;
            return;
        }
        if Self::on_bed_surface(mob) {
            mob.get_mob_entity()
                .living_entity
                .jumping
                .store(true, Ordering::SeqCst);
            self.remaining_jumps -= 1;
            self.remaining_cooldown_until_next_jump = 5;
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::JUMP
    }
}

pub struct SocializeAtBellGoal {
    speed: f64,
    target: Option<Arc<dyn EntityBase>>,
    ticks: i32,
}

impl SocializeAtBellGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            speed,
            target: None,
            ticks: 0,
        }
    }

    fn near_bell(mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let origin = entity.block_pos.load();
        let position = entity.pos.load();
        let start = BlockPos::new(origin.0.x - 4, origin.0.y - 4, origin.0.z - 4);
        let end = BlockPos::new(origin.0.x + 4, origin.0.y + 4, origin.0.z + 4);

        BlockPos::iterate(start, end).any(|candidate| {
            let (block, _state) = world.get_block_and_state(&candidate);
            block.id == Block::BELL.id
                && candidate
                    .to_centered_f64()
                    .squared_distance_to_vec(&position)
                    <= 16.0
        })
    }

    fn find_partner(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let entity = mob.get_entity();
        let position = entity.pos.load();
        let world = entity.world.load();

        let mut closest: Option<(f64, Arc<dyn EntityBase>)> = None;
        for candidate in world.get_nearby_entities(position, 6.0).values() {
            let candidate_entity = candidate.get_entity();
            if candidate_entity.entity_id == entity.entity_id
                || candidate_entity.entity_type != &EntityType::VILLAGER
                || !candidate_entity.is_alive()
                || candidate.get_mob().is_some_and(Mob::is_sleeping)
            {
                continue;
            }
            let distance = position.squared_distance_to_vec(&candidate_entity.pos.load());
            if distance <= 32.0 && closest.as_ref().is_none_or(|(best, _)| distance < *best) {
                closest = Some((distance, candidate.clone()));
            }
        }
        closest.map(|(_, partner)| partner)
    }
}

impl Goal for SocializeAtBellGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.is_sleeping()
            || mob.get_trading_player().is_some()
            || mob.get_random().random_range(0..100) != 0
            || !Self::near_bell(mob)
        {
            return false;
        }
        self.target = Self::find_partner(mob);
        self.target.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(target) = &self.target else {
            return false;
        };
        if !target.get_entity().is_alive() || mob.is_sleeping() || self.ticks >= 100 {
            return false;
        }
        let position = mob.get_entity().pos.load();
        position.squared_distance_to_vec(&target.get_entity().pos.load()) > 1.0
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.ticks = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target = None;
        self.ticks = 0;
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(target) = self.target.clone() else {
            return;
        };
        self.ticks += 1;

        let mob_entity = mob.get_mob_entity();
        {
            let mut look_control = mob_entity
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            look_control.look_at_entity(mob, &target);
        };

        let position = mob.get_entity().pos.load();
        mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_progress(NavigatorGoal::new(
                position,
                target.get_entity().pos.load(),
                self.speed,
            ));
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

pub struct VillagerMakeLoveGoal {
    speed: f64,
    partner: Option<Arc<dyn EntityBase>>,
    ticks_until_birth: i32,
}

impl VillagerMakeLoveGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            speed,
            partner: None,
            ticks_until_birth: 0,
        }
    }

    fn find_partner(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let entity = mob.get_entity();
        let position = entity.pos.load();
        let world = entity.world.load();

        let mut closest: Option<(f64, Arc<dyn EntityBase>)> = None;
        for candidate in world.get_nearby_entities(position, 8.0).values() {
            let candidate_entity = candidate.get_entity();
            if candidate_entity.entity_id == entity.entity_id
                || candidate_entity.entity_type != &EntityType::VILLAGER
                || !candidate_entity.is_alive()
            {
                continue;
            }
            if !candidate.get_mob().is_some_and(Mob::wants_to_breed) {
                continue;
            }
            let distance = position.squared_distance_to_vec(&candidate_entity.pos.load());
            if distance <= 64.0 && closest.as_ref().is_none_or(|(best, _)| distance < *best) {
                closest = Some((distance, candidate.clone()));
            }
        }
        closest.map(|(_, partner)| partner)
    }

    fn is_breeding_possible(mob: &dyn Mob, partner: &Arc<dyn EntityBase>) -> bool {
        partner.get_entity().is_alive()
            && mob.wants_to_breed()
            && partner.get_mob().is_some_and(Mob::wants_to_breed)
    }

    fn try_to_give_birth(mob: &dyn Mob, partner: &Arc<dyn EntityBase>) {
        let entity = mob.get_entity();
        let world = entity.world.load_full();

        let bed = find_free_bed(&world, entity.block_pos.load(), 16, 4, None);
        let Some(bed) = bed else {
            send_status(entity, EntityStatus::VillagerAngry);
            send_status(partner.get_entity(), EntityStatus::VillagerAngry);
            return;
        };

        mob.consume_breeding_food();
        mob.get_mob_entity()
            .breeding_cooldown
            .store(6000, std::sync::atomic::Ordering::Relaxed);
        if let Some(partner_mob) = partner.get_mob() {
            partner_mob.consume_breeding_food();
        }
        partner.set_breeding_cooldown(6000);

        let baby = from_type(
            entity.entity_type,
            entity.pos.load(),
            &world,
            Uuid::new_v4(),
        );
        baby.get_entity().set_age(-24000);
        if let Some(baby_mob) = baby.get_mob() {
            baby_mob.set_home(Some(bed));
        }
        world.spawn_entity(baby.clone());

        send_status(baby.get_entity(), EntityStatus::LoveHearts);
        send_status(entity, EntityStatus::LoveHearts);
        send_status(partner.get_entity(), EntityStatus::LoveHearts);
    }
}

impl Goal for VillagerMakeLoveGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !mob.wants_to_breed() || mob.is_sleeping() || mob.get_trading_player().is_some() {
            return false;
        }
        self.partner = Self::find_partner(mob);
        self.partner.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(partner) = &self.partner else {
            return false;
        };
        self.ticks_until_birth > 0 && !mob.is_sleeping() && Self::is_breeding_possible(mob, partner)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.ticks_until_birth = 275 + mob.get_random().random_range(0..50);

        let entity = mob.get_entity();
        send_status(entity, EntityStatus::InLoveHearts);
        if let Some(partner) = &self.partner {
            send_status(partner.get_entity(), EntityStatus::InLoveHearts);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.partner = None;
        self.ticks_until_birth = 0;
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(partner) = self.partner.clone() else {
            return;
        };

        let mob_entity = mob.get_mob_entity();
        {
            let mut look_control = mob_entity
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            look_control.look_at_entity(mob, &partner);
        };

        let position = mob.get_entity().pos.load();
        let partner_position = partner.get_entity().pos.load();
        let distance = position.squared_distance_to_vec(&partner_position);
        if distance > 5.0 {
            mob_entity
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_progress(NavigatorGoal::new(position, partner_position, self.speed));
            return;
        }

        self.ticks_until_birth -= 1;
        if self.ticks_until_birth <= 0 {
            Self::try_to_give_birth(mob, &partner);
        } else if mob.get_random().random_range(0..35) == 0 {
            send_status(mob.get_entity(), EntityStatus::LoveHearts);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
