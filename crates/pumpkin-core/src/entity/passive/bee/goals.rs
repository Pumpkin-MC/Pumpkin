use std::collections::{HashMap, VecDeque};
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::block_properties::{DoubleBlockHalf, TallSeagrassLikeProperties};
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockStateId, Property};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::{poi::Occupancy, world::BlockFlags};
use rand::RngExt;

use super::{
    BeeEntity, COOLDOWN_BEFORE_LOCATING_NEW_HIVE, MAX_CROPS_GROWABLE,
    MAX_FIND_FLOWER_RETRY_COOLDOWN, MIN_FIND_FLOWER_RETRY_COOLDOWN, TOO_FAR_DISTANCE,
};
use crate::block::{BonemealArgs, entities::beehive::BeehiveBlockEntity};
use crate::entity::EntityBase;
use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::ai::pathfinder::path::Path;
use crate::entity::ai::util::{air_and_water_random_pos, air_random_pos, hover_random_pos};
use crate::entity::mob::{Mob, neutral::NeutralMob};
use crate::world::World;

const COOLDOWN_BEFORE_LOCATING_NEW_FLOWER: i32 = 200;
const MIN_POLLINATION_TICKS: i32 = 400;
const MAX_POLLINATING_TICKS: i32 = 600;
/// Vanilla `findNearbyFlower` skips an unreachable flower for this long (a bare `600L`).
const UNREACHABLE_FLOWER_TICKS: i64 = 600;
const FLOWER_SEARCH_RADIUS: i32 = 5;
const ARRIVAL_THRESHOLD: f64 = 0.1;
const POSITION_CHANGE_CHANCE: i32 = 25;
const SPEED_MODIFIER: f32 = 0.35;
const HOVER_HEIGHT_WITHIN_FLOWER: f32 = 0.6;
const HOVER_POS_OFFSET: f32 = 1.0 / 3.0;
const HIVE_CLOSE_ENOUGH_DISTANCE: i32 = 2;
const HIVE_SEARCH_DISTANCE: i32 = 20;
const MAX_TRAVELLING_TICKS: i32 = 2400;
const TICKS_BEFORE_GOING_TO_KNOWN_FLOWER: i32 = 600;
const PATHFIND_TO_HIVE_WHEN_CLOSER_THAN: i32 = 16;
const MAX_BLACKLISTED_TARGETS: usize = 3;
const TICKS_BEFORE_HIVE_DROP: i32 = 60;
const DEFAULT_WANDER_DISTANCE_REDUCTION: i32 = 16;
const RESTRICTED_WANDER_DISTANCE_REDUCTION: i32 = 24;
const GROW_CHANCE: i32 = 30;

pub(super) fn register(bee: &BeeEntity) {
    use crate::entity::ai::goal::{
        breed::BreedGoal, follow_parent::FollowParentGoal, melee_attack::MeleeAttackGoal,
        swim::SwimGoal, tempt::TemptGoal,
    };
    let selector = &bee.mob_entity.goals_selector;
    selector.add_goal(
        0,
        Box::new(MeleeAttackGoal::new(1.4, true).gated_by(super::can_sting)),
    );
    selector.add_goal(1, Box::new(BeeEnterHiveGoal));
    selector.add_goal(2, BreedGoal::new(1.0));
    selector.add_goal(
        3,
        Box::new(TemptGoal::with_tag(
            1.25,
            &tag::Item::MINECRAFT_BEE_FOOD,
            false,
        )),
    );
    selector.add_goal(
        3,
        Box::new(ValidateHiveGoal {
            cooldown: bee.get_random().random_range(20..=40),
            last_validate_tick: -1,
        }),
    );
    selector.add_goal(
        3,
        Box::new(ValidateFlowerGoal {
            cooldown: bee.get_random().random_range(20..=40),
            last_validate_tick: -1,
        }),
    );
    selector.add_goal(4, Box::new(BeePollinateGoal::default()));
    selector.add_goal(5, Box::new(FollowParentGoal::new(1.25)));
    let blacklist = std::sync::Arc::new(std::sync::Mutex::new(VecDeque::new()));
    selector.add_goal(5, Box::new(BeeLocateHiveGoal(blacklist.clone())));
    selector.add_goal(
        5,
        Box::new(BeeGoToHiveGoal {
            blacklist,
            travelling_ticks: 0,
            ticks_stuck: 0,
            last_path: None,
        }),
    );
    selector.add_goal(6, Box::new(BeeGoToKnownFlowerGoal::default()));
    selector.add_goal(7, Box::new(BeeGrowCropGoal));
    selector.add_goal(8, Box::new(BeeWanderGoal));
    selector.add_goal(9, Box::new(SwimGoal::default()));
}

fn bee(mob: &dyn Mob) -> Option<&BeeEntity> {
    mob.cast_any().downcast_ref()
}

fn attracts_bees(world: &World, pos: &BlockPos) -> bool {
    let (block, state) = world.get_block_and_state(pos);
    block.has_tag(&tag::Block::MINECRAFT_BEE_ATTRACTIVE)
        && !state.is_waterlogged()
        && (block != &Block::SUNFLOWER
            || TallSeagrassLikeProperties::from_state_id(state.id).half == DoubleBlockHalf::Upper)
}

struct ValidateFlowerGoal {
    cooldown: i64,
    last_validate_tick: i64,
}

impl Goal for ValidateFlowerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        bee(mob).is_some_and(|bee| {
            !bee.is_angry()
                && bee.get_entity().world.load().get_world_age()
                    > self.last_validate_tick + self.cooldown
        })
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(bee) = bee(mob) else { return };
        let world = bee.get_entity().world.load();
        if let Some(pos) = bee.flower_pos.load()
            && world.is_loaded(&pos)
            && !attracts_bees(&world, &pos)
        {
            bee.drop_flower();
        }
        self.last_validate_tick = world.get_world_age();
    }
}

#[derive(Default)]
struct BeePollinateGoal {
    successful_ticks: i32,
    last_sound_tick: i32,
    hover_pos: Option<Vector3<f64>>,
    pollinating_ticks: i32,
    unreachable_flowers: HashMap<BlockPos, i64>,
}

impl BeePollinateGoal {
    fn find_nearby_flower(&mut self, bee: &BeeEntity) -> Option<BlockPos> {
        let entity = bee.get_entity();
        let world = entity.world.load();
        let tick = world.get_world_age();
        let mut previous = std::mem::take(&mut self.unreachable_flowers);
        let radius = FLOWER_SEARCH_RADIUS;
        let living = &bee.mob_entity.living_entity;
        let steered = bee.mob_entity.steered();
        let mut navigator = steered.navigator();
        BlockPos::iterate_outwards(entity.block_pos.load(), radius, radius, radius)
            .filter(|pos| attracts_bees(&world, pos))
            .find(|pos| {
                if let Some(until) = previous.remove(pos)
                    && tick < until
                {
                    self.unreachable_flowers.insert(*pos, until);
                } else if navigator
                    .create_path(living, pos.to_centered_f64(), 1)
                    .is_some_and(|path| path.can_reach())
                {
                    return true;
                } else {
                    self.unreachable_flowers
                        .insert(*pos, tick + UNREACHABLE_FLOWER_TICKS);
                }
                false
            })
    }
}

impl Goal for BeePollinateGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(bee) = bee(mob) else { return false };
        if bee.flower_cooldown.load(Relaxed) > 0
            || bee.has_nectar()
            || bee.get_entity().world.load().is_raining()
        {
            return false;
        }
        let Some(pos) = self.find_nearby_flower(bee) else {
            bee.flower_cooldown.store(
                bee.get_random()
                    .random_range(MIN_FIND_FLOWER_RETRY_COOLDOWN..=MAX_FIND_FLOWER_RETRY_COOLDOWN),
                Relaxed,
            );
            return false;
        };
        bee.set_saved_flower_pos(pos);
        let center = pos.to_centered_f64();
        let living = &bee.mob_entity.living_entity;
        bee.mob_entity.steered().navigator().move_to_coords(
            center.x,
            center.y,
            center.z,
            1.2f32.into(),
            living,
        );
        !bee.is_angry()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        bee(mob).is_some_and(|bee| {
            bee.pollinating.load(Relaxed)
                && bee.flower_pos.load().is_some()
                && !bee.get_entity().world.load().is_raining()
                && (self.successful_ticks <= MIN_POLLINATION_TICKS
                    || bee.get_random().random::<f32>() < 0.2)
                && !bee.is_angry()
        })
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(bee) = bee(mob) else { return };
        self.successful_ticks = 0;
        self.pollinating_ticks = 0;
        self.last_sound_tick = 0;
        bee.pollinating.store(true, Relaxed);
        bee.ticks_without_nectar.store(0, Relaxed);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let Some(bee) = bee(mob) else { return };
        if self.successful_ticks > MIN_POLLINATION_TICKS {
            bee.set_has_nectar(true);
        }
        bee.pollinating.store(false, Relaxed);
        bee.mob_entity.steered().navigator().stop();
        bee.flower_cooldown
            .store(COOLDOWN_BEFORE_LOCATING_NEW_FLOWER, Relaxed);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(bee) = bee(mob) else { return };
        let Some(flower) = bee.flower_pos.load() else {
            return;
        };
        self.pollinating_ticks += 1;
        if self.pollinating_ticks > MAX_POLLINATING_TICKS {
            bee.drop_flower();
            bee.pollinating.store(false, Relaxed);
            bee.flower_cooldown
                .store(COOLDOWN_BEFORE_LOCATING_NEW_FLOWER, Relaxed);
            return;
        }

        let mut flower = flower.to_centered_f64();
        flower.y += f64::from(HOVER_HEIGHT_WITHIN_FLOWER) - 0.5;
        let pos = bee.get_entity().pos.load();
        let steered = bee.mob_entity.steered();
        if pos.squared_distance_to_vec(&flower) > 1.0 {
            self.hover_pos = Some(flower);
            steered.move_control().set_wanted_position(
                flower.x,
                flower.y,
                flower.z,
                f64::from(SPEED_MODIFIER),
            );
            return;
        }

        let hover = self.hover_pos.get_or_insert(flower);
        let mut rng = bee.get_random();
        let arrived = pos.squared_distance_to_vec(hover) <= ARRIVAL_THRESHOLD * ARRIVAL_THRESHOLD;
        let change_pos = arrived && rng.random_range(0..POSITION_CHANGE_CHANCE) == 0;
        if change_pos {
            let mut offset = || f64::from((rng.random::<f32>() * 2.0 - 1.0) * HOVER_POS_OFFSET);
            *hover = Vector3::new(flower.x + offset(), flower.y, flower.z + offset());
            steered.navigator().stop();
        }
        if arrived {
            bee.mob_entity
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .look_at(mob, flower.x, flower.y, flower.z);
        }
        if !arrived || change_pos {
            steered.move_control().set_wanted_position(
                hover.x,
                hover.y,
                hover.z,
                f64::from(SPEED_MODIFIER),
            );
        }
        self.successful_ticks += 1;
        if rng.random::<f32>() < 0.05 && self.successful_ticks > self.last_sound_tick + 60 {
            self.last_sound_tick = self.successful_ticks;
            bee.play_entity_sound(Sound::EntityBeePollinate, 1.0, 1.0);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

struct BeeWanderGoal;

impl Goal for BeeWanderGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        mob.get_mob_entity().steered().navigator().is_done()
            && mob.get_random().random_range(0..10) == 0
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.get_mob_entity().steered().navigator().is_done()
    }

    fn start(&mut self, mob: &dyn Mob) {
        use std::f64::consts::FRAC_PI_2;
        let Some(bee) = bee(mob) else { return };
        let reduction = if bee.hive_pos.load().is_none() && bee.flower_pos.load().is_none() {
            DEFAULT_WANDER_DISTANCE_REDUCTION
        } else {
            RESTRICTED_WANDER_DISTANCE_REDUCTION
        };
        let direction = bee
            .hive_pos
            .load()
            .filter(|pos| {
                !bee.closer_than(*pos, TOO_FAR_DISTANCE - reduction) && bee.hive().is_some()
            })
            .map_or_else(
                || mob.get_looking_vector(),
                |pos| (pos.to_centered_f64() - bee.get_entity().pos.load()).normalize(),
            );
        let target =
            hover_random_pos::get_pos(mob, 8, 7, direction.x, direction.z, FRAC_PI_2, 3, 1)
                .or_else(|| {
                    air_and_water_random_pos::get_pos(
                        mob,
                        8,
                        4,
                        -2,
                        direction.x,
                        direction.z,
                        FRAC_PI_2,
                    )
                });
        if let Some(target) = target {
            let living = &mob.get_mob_entity().living_entity;
            let steered = mob.get_mob_entity().steered();
            let mut navigator = steered.navigator();
            let path = navigator.create_path(living, target, 1);
            navigator.move_to_path(path, 1.0, living);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

struct BeeEnterHiveGoal;

impl Goal for BeeEnterHiveGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(bee) = bee(mob) else { return false };
        let Some(pos) = bee.hive_pos.load() else {
            return false;
        };
        if !bee.wants_to_enter_hive()
            || pos
                .to_centered_f64()
                .squared_distance_to_vec(&bee.get_entity().pos.load())
                >= f64::from(HIVE_CLOSE_ENOUGH_DISTANCE * HIVE_CLOSE_ENOUGH_DISTANCE)
        {
            return false;
        }
        let Some(hive) = bee.hive() else { return false };
        let Some(hive) = hive.as_any().downcast_ref::<BeehiveBlockEntity>() else {
            return false;
        };
        if hive.is_full() {
            bee.hive_pos.store(None);
            return false;
        }
        !bee.is_angry()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(bee) = bee(mob)
            && let Some(hive) = bee.hive()
            && let Some(hive) = hive.as_any().downcast_ref::<BeehiveBlockEntity>()
        {
            hive.add_occupant(&bee.get_entity().world.load(), bee);
        }
    }
}

struct ValidateHiveGoal {
    cooldown: i64,
    last_validate_tick: i64,
}

impl Goal for ValidateHiveGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        bee(mob).is_some_and(|bee| {
            !bee.is_angry()
                && bee.get_entity().world.load().get_world_age()
                    > self.last_validate_tick + self.cooldown
        })
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(bee) = bee(mob) else { return };
        let world = bee.get_entity().world.load();
        if let Some(pos) = bee.hive_pos.load()
            && world.is_loaded(&pos)
            && bee.hive().is_none()
        {
            bee.drop_hive();
        }
        self.last_validate_tick = world.get_world_age();
    }
}

type HiveBlacklist = std::sync::Arc<std::sync::Mutex<VecDeque<BlockPos>>>;

struct BeeLocateHiveGoal(HiveBlacklist);

impl Goal for BeeLocateHiveGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        bee(mob).is_some_and(|bee| {
            bee.hive_cooldown.load(Relaxed) == 0
                && bee.hive_pos.load().is_none()
                && bee.wants_to_enter_hive()
                && !bee.is_angry()
        })
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(bee) = bee(mob) else { return };
        let entity = bee.get_entity();
        let world = entity.world.load();
        bee.hive_cooldown
            .store(COOLDOWN_BEFORE_LOCATING_NEW_HIVE, Relaxed);
        let hives = world
            .portal_poi
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .find_all_closest_first_with_type(
                &|poi| poi.has_tag(&tag::PointOfInterestType::MINECRAFT_BEE_HOME),
                &|_| true,
                entity.block_pos.load(),
                HIVE_SEARCH_DISTANCE,
                Occupancy::Any,
            );
        let mut available = hives.into_iter().map(|(_, pos)| pos).filter(|pos| {
            world.get_block_entity(pos).is_some_and(|entity| {
                entity
                    .as_any()
                    .downcast_ref::<BeehiveBlockEntity>()
                    .is_some_and(|hive| !hive.is_full())
            })
        });
        let Some(first) = available.next() else {
            return;
        };
        let mut blacklist = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let chosen = std::iter::once(first)
            .chain(available)
            .find(|pos| !blacklist.contains(pos));
        if chosen.is_none() {
            blacklist.clear();
        }
        bee.set_hive_pos(chosen.unwrap_or(first));
    }
}

struct BeeGoToHiveGoal {
    blacklist: HiveBlacklist,
    travelling_ticks: i32,
    ticks_stuck: i32,
    last_path: Option<Path>,
}

impl BeeGoToHiveGoal {
    fn drop_and_blacklist(&self, bee: &BeeEntity, pos: BlockPos) {
        let mut blacklist = self
            .blacklist
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if blacklist.len() == MAX_BLACKLISTED_TARGETS {
            blacklist.pop_front();
        }
        blacklist.push_back(pos);
        bee.drop_hive();
    }
}

impl Goal for BeeGoToHiveGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(bee) = bee(mob) else { return false };
        let Some(hive) = bee.hive_pos.load() else {
            return false;
        };
        bee.closer_than(hive, TOO_FAR_DISTANCE)
            && !bee.mob_entity.has_position_target()
            && bee.wants_to_enter_hive()
            && !bee.closer_than(hive, HIVE_CLOSE_ENOUGH_DISTANCE)
            && !bee
                .mob_entity
                .steered()
                .navigator()
                .get_path()
                .is_some_and(|path| path.get_target() == hive && path.can_reach() && path.is_done())
            && bee
                .get_entity()
                .world
                .load()
                .get_block(&hive)
                .has_tag(&tag::Block::MINECRAFT_BEEHIVES)
            && !bee.is_angry()
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.travelling_ticks = 0;
        self.ticks_stuck = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.travelling_ticks = 0;
        self.ticks_stuck = 0;
        stop_travelling(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(bee) = bee(mob) else { return };
        let Some(hive) = bee.hive_pos.load() else {
            return;
        };
        self.travelling_ticks += 1;
        if self.travelling_ticks > self.get_tick_count(MAX_TRAVELLING_TICKS) {
            self.drop_and_blacklist(bee, hive);
            return;
        }
        if !mob.get_mob_entity().steered().navigator().is_done() {
            return;
        }
        if !bee.closer_than(hive, PATHFIND_TO_HIVE_WHEN_CLOSER_THAN) {
            if bee.closer_than(hive, TOO_FAR_DISTANCE) {
                pathfind_randomly_towards(bee, hive);
            } else {
                bee.drop_hive();
            }
            return;
        }
        let living = &bee.mob_entity.living_entity;
        let steered = bee.mob_entity.steered();
        let mut navigator = steered.navigator();
        navigator.set_max_visited_nodes_multiplier(10.0);
        let distance = if bee.closer_than(hive, 3) { 1 } else { 2 };
        let path = navigator.create_path(living, hive.to_f64(), distance);
        navigator.move_to_path(path, 1.0, living);
        match navigator.get_path().filter(|path| path.can_reach()) {
            None => self.drop_and_blacklist(bee, hive),
            Some(path) if path.same_as(self.last_path.as_ref()) => {
                self.ticks_stuck += 1;
                if self.ticks_stuck > TICKS_BEFORE_HIVE_DROP {
                    bee.drop_hive();
                    self.ticks_stuck = 0;
                }
            }
            Some(path) => self.last_path = Some(path.clone()),
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

#[derive(Default)]
struct BeeGoToKnownFlowerGoal {
    travelling_ticks: i32,
}

impl Goal for BeeGoToKnownFlowerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        bee(mob).is_some_and(|bee| {
            bee.flower_pos.load().is_some_and(|pos| {
                !bee.mob_entity.has_position_target()
                    && bee.ticks_without_nectar.load(Relaxed) > TICKS_BEFORE_GOING_TO_KNOWN_FLOWER
                    && !bee.closer_than(pos, HIVE_CLOSE_ENOUGH_DISTANCE)
                    && !bee.is_angry()
            })
        })
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.travelling_ticks = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.travelling_ticks = 0;
        stop_travelling(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(bee) = bee(mob) else { return };
        let Some(flower) = bee.flower_pos.load() else {
            return;
        };
        self.travelling_ticks += 1;
        if self.travelling_ticks > self.get_tick_count(MAX_TRAVELLING_TICKS) {
            bee.drop_flower();
        } else if mob.get_mob_entity().steered().navigator().is_done() {
            if bee.closer_than(flower, TOO_FAR_DISTANCE) {
                pathfind_randomly_towards(bee, flower);
            } else {
                bee.drop_flower();
            }
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

fn stop_travelling(mob: &dyn Mob) {
    let steered = mob.get_mob_entity().steered();
    let mut navigator = steered.navigator();
    navigator.stop();
    navigator.reset_max_visited_nodes_multiplier();
}

fn pathfind_randomly_towards(bee: &BeeEntity, target: BlockPos) {
    let pos = bee.get_entity().block_pos.load();
    let height = match target.0.y - pos.0.y {
        delta if delta > 2 => 4,
        delta if delta < -2 => -4,
        _ => 0,
    };
    let distance = pos.manhattan_distance(target);
    let (xz, y) = if distance < 15 {
        (distance / 2, distance / 2)
    } else {
        (6, 8)
    };
    let target = target.to_f64().add_raw(0.5, 0.0, 0.5);
    if let Some(next) = air_random_pos::get_pos_towards(
        bee,
        xz,
        y,
        height,
        target,
        f64::from(std::f32::consts::PI / 10.0),
    ) {
        let steered = bee.mob_entity.steered();
        let mut navigator = steered.navigator();
        navigator.set_max_visited_nodes_multiplier(0.5);
        navigator.move_to_coords(next.x, next.y, next.z, 1.0, &bee.mob_entity.living_entity);
    }
}

struct BeeGrowCropGoal;

impl Goal for BeeGrowCropGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        bee(mob).is_some_and(|bee| {
            bee.crops_grown_since_pollination.load(Relaxed) < MAX_CROPS_GROWABLE
                && bee.get_random().random::<f32>() >= 0.3
                && bee.has_nectar()
                && bee.hive().is_some()
                && !bee.is_angry()
        })
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(bee) = bee(mob) else { return };
        if bee
            .get_random()
            .random_range(0..self.get_tick_count(GROW_CHANCE))
            != 0
        {
            return;
        }
        let entity = bee.get_entity();
        let world = entity.world.load_full();
        for depth in 1..=2 {
            let pos = entity.block_pos.load().down_height(depth);
            if let Some(grown) = grow_crop(&world, &pos) {
                world.sync_world_event(
                    pumpkin_data::world::WorldEvent::ParticlesBeeGrowth,
                    pos,
                    15,
                );
                world.set_block_state(&pos, grown, BlockFlags::NOTIFY_ALL);
                bee.crops_grown_since_pollination.fetch_add(1, Relaxed);
            }
        }
    }
}

fn grow_crop(world: &std::sync::Arc<World>, pos: &BlockPos) -> Option<BlockStateId> {
    let (block, state) = world.get_block_and_state_id(pos);
    if !block.has_tag(&tag::Block::MINECRAFT_BEE_GROWABLES) || block == &Block::PITCHER_CROP {
        return None;
    }
    if block == &Block::CAVE_VINES || block == &Block::CAVE_VINES_PLANT {
        let behaviour = world.block_registry.get_pumpkin_block(block.id)?;
        let args = || BonemealArgs {
            world,
            block,
            position: pos,
            state_id: state,
        };
        if behaviour.is_valid_bonemeal_target(args()) {
            behaviour.perform_bonemeal(args());
            return Some(world.get_block_state_id(pos));
        }
        return None;
    }
    let mut props = block.properties(state)?.to_props();
    let age = props.iter_mut().find_map(|prop| {
        if let Property::Age(age) = prop {
            Some(age)
        } else {
            None
        }
    })?;
    *age += 1;
    block
        .state_from_properties(&props)
        .map(|state| state.id)
        .or_else(|| {
            // `TorchflowerCropBlock.getStateForAge(2)` turns the crop into the flower.
            (block == &Block::TORCHFLOWER_CROP).then_some(Block::TORCHFLOWER.default_state.id)
        })
}
