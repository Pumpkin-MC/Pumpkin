use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::RandomImpl;
use pumpkin_util::random::legacy_rand::LegacyRand;
use rand::RngExt;

use crate::entity::ai::brain::activity::ActivityData;
use crate::entity::ai::brain::behavior::one_shot::OneShot;
use crate::entity::ai::brain::behavior::utils::{
    get_nearest_target, is_dead_or_dying,
    is_other_target_much_further_away_than_current_attack_target,
};
use crate::entity::ai::brain::behavior::{self, BehaviorControl};
use crate::entity::ai::brain::memory::{MemoryStatus, types};
use crate::entity::ai::brain::sensing::SensorType;
use crate::entity::ai::brain::sensing::is_entity_attackable_ignoring_line_of_sight;
use crate::entity::ai::brain::{Brain, BrainTick, VisibilityContext};
use crate::entity::ai::brain::{BrainProvider, memory::MemoryModuleId};
use crate::entity::ai::util::land_random_pos;
use crate::entity::item::ItemEntity;
use crate::entity::mob::hoglin::HoglinEntity;
use crate::entity::mob::piglin::PiglinEntity;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::world::World;
use crate::world::loot::LootContextParameters;
use pumpkin_data::environment_attribute::Activity;

pub const REPELLENT_DETECTION_RANGE_HORIZONTAL: i32 = 8;
pub const REPELLENT_DETECTION_RANGE_VERTICAL: i32 = 4;
pub const BARTERING_ITEM: &Item = &Item::GOLD_INGOT;
pub const ANGER_DURATION: i64 = 600;
pub const ADMIRE_DURATION: i32 = 119;
pub const MAX_DISTANCE_TO_WALK_TO_ITEM: i32 = 9;
pub const MAX_TIME_TO_WALK_TO_ITEM: i32 = 200;
pub const HOW_LONG_TIME_TO_DISABLE_ADMIRE_WALKING_IF_CANT_REACH_ITEM: i32 = 200;
pub const CELEBRATION_TIME: i32 = 300;
pub const TIME_BETWEEN_HUNTS: (i32, i32) = (30 * 20, 120 * 20);
pub const BABY_FLEE_DURATION_AFTER_GETTING_HIT: i64 = 100;
pub const HIT_BY_PLAYER_MEMORY_TIMEOUT: i64 = 400;
pub const MAX_WALK_DISTANCE_TO_START_RIDING: i32 = 8;
pub const RIDE_START_INTERVAL: (i32, i32) = (10 * 20, 40 * 20);
pub const RIDE_DURATION: (i32, i32) = (10 * 20, 30 * 20);
pub const RETREAT_DURATION: (i32, i32) = (5 * 20, 20 * 20);
pub const MELEE_ATTACK_COOLDOWN: i32 = 20;
pub const EAT_COOLDOWN: i64 = 200;
pub const DESIRED_DISTANCE_FROM_ENTITY_WHEN_AVOIDING: i32 = 12;
pub const MAX_LOOK_DIST: f32 = 8.0;
pub const MAX_LOOK_DIST_FOR_PLAYER_HOLDING_LOVED_ITEM: f32 = 14.0;
pub const INTERACTION_RANGE: i32 = 8;
pub const MIN_DESIRED_DIST_FROM_TARGET_WHEN_HOLDING_CROSSBOW: i32 = 5;
pub const SPEED_WHEN_STRAFING_BACK_FROM_TARGET: f32 = 0.75;
pub const DESIRED_DISTANCE_FROM_ZOMBIFIED: f64 = 6.0;
pub const AVOID_ZOMBIFIED_DURATION: (i32, i32) = (5 * 20, 7 * 20);
pub const BABY_AVOID_NEMESIS_DURATION: (i32, i32) = (5 * 20, 7 * 20);
pub const PROBABILITY_OF_CELEBRATION_DANCE: f32 = 0.1;
pub const SPEED_MULTIPLIER_WHEN_AVOIDING: f32 = 1.0;
pub const SPEED_MULTIPLIER_WHEN_RETREATING: f32 = 1.0;
pub const SPEED_MULTIPLIER_WHEN_MOUNTING: f32 = 0.8;
pub const SPEED_MULTIPLIER_WHEN_GOING_TO_WANTED_ITEM: f32 = 1.0;
pub const SPEED_MULTIPLIER_WHEN_GOING_TO_CELEBRATE_LOCATION: f32 = 1.0;
pub const SPEED_MULTIPLIER_WHEN_DANCING: f32 = 0.6;
pub const SPEED_MULTIPLIER_WHEN_IDLING: f32 = 0.6;
const BARTERING_LOOT_TABLE: &str = "minecraft:gameplay/piglin_bartering";
pub const THROW_SPEED: f64 = 0.3;
pub const THROW_HAND_Y_DISTANCE_FROM_EYE: f64 = 0.3;

#[must_use]
pub fn sample(range: (i32, i32), rng: &mut impl RngExt) -> i32 {
    range.0 + rng.random_range(0..(range.1 + 1 - range.0).max(1))
}

#[must_use]
pub fn is_zombified(entity: &dyn EntityBase) -> bool {
    let entity_type = entity.get_entity().entity_type;
    entity_type == &EntityType::ZOMBIFIED_PIGLIN || entity_type == &EntityType::ZOGLIN
}

#[must_use]
pub fn is_loved_item(stack: &ItemStack) -> bool {
    stack.item.has_tag(&tag::Item::MINECRAFT_PIGLIN_LOVED)
}

#[must_use]
pub fn is_food(stack: &ItemStack) -> bool {
    stack.item.has_tag(&tag::Item::MINECRAFT_PIGLIN_FOOD)
}

#[must_use]
pub const fn is_barter_currency(stack: &ItemStack) -> bool {
    stack.item.id == BARTERING_ITEM.id
}

#[must_use]
pub fn is_wearing_safe_armor(entity: &dyn EntityBase) -> bool {
    let Some(living) = entity.get_living_entity() else {
        return false;
    };
    let Ok(equipment) = living.entity_equipment.try_lock() else {
        return false;
    };
    [
        EquipmentSlot::HEAD,
        EquipmentSlot::CHEST,
        EquipmentSlot::LEGS,
        EquipmentSlot::FEET,
    ]
    .iter()
    .any(|slot| {
        equipment
            .get(slot)
            .item
            .has_tag(&tag::Item::MINECRAFT_PIGLIN_SAFE_ARMOR)
    })
}

#[must_use]
pub fn is_player_holding_loved_item(entity: &dyn EntityBase) -> bool {
    if entity.get_entity().entity_type != &EntityType::PLAYER {
        return false;
    }
    let Some(living) = entity.get_living_entity() else {
        return false;
    };
    let Ok(equipment) = living.entity_equipment.try_lock() else {
        return false;
    };
    is_loved_item(&equipment.get(&EquipmentSlot::MAIN_HAND))
        || is_loved_item(&equipment.get(&EquipmentSlot::OFF_HAND))
}

#[must_use]
pub fn as_piglin(entity: &dyn EntityBase) -> Option<&PiglinEntity> {
    entity.cast_any().downcast_ref::<PiglinEntity>()
}

#[must_use]
pub fn as_hoglin(entity: &dyn EntityBase) -> Option<&HoglinEntity> {
    entity.cast_any().downcast_ref::<HoglinEntity>()
}

#[must_use]
pub fn is_adult_piglin(entity: &dyn EntityBase) -> bool {
    as_piglin(entity).is_some_and(PiglinEntity::is_adult)
}

#[must_use]
pub fn is_baby_hoglin(entity: &dyn EntityBase) -> bool {
    as_hoglin(entity)
        .is_some_and(|hoglin| hoglin.is_baby.load(std::sync::atomic::Ordering::Relaxed))
}

#[must_use]
pub fn hoglin_can_be_hunted(entity: &dyn EntityBase) -> bool {
    as_hoglin(entity).is_some_and(HoglinEntity::can_be_hunted)
}

/// Vanilla `PiglinAi.findNearbyAdultPiglins`, over `NEAREST_LIVING_ENTITIES`.
#[must_use]
pub fn find_nearby_adult_piglins(brain: &Brain) -> Vec<Arc<dyn EntityBase>> {
    brain
        .get(types::NEAREST_LIVING_ENTITIES)
        .map(|entities| {
            entities
                .iter()
                .filter(|entity| {
                    let entity_type = entity.get_entity().entity_type;
                    entity_type == &EntityType::PIGLIN_BRUTE || is_adult_piglin(entity.as_ref())
                })
                .map(Arc::clone)
                .collect()
        })
        .unwrap_or_default()
}

#[must_use]
pub fn is_admiring_item(brain: &Brain) -> bool {
    brain.has_memory_value(types::ADMIRING_ITEM.id())
}

#[must_use]
pub fn is_admiring_disabled(brain: &Brain) -> bool {
    brain.has_memory_value(types::ADMIRING_DISABLED.id())
}

#[must_use]
pub fn has_eaten_recently(brain: &Brain) -> bool {
    brain.has_memory_value(types::ATE_RECENTLY.id())
}

#[must_use]
pub fn is_near_repellent(brain: &Brain) -> bool {
    brain.has_memory_value(types::NEAREST_REPELLENT.id())
}

#[must_use]
pub fn sees_player_holding_loved_item(brain: &Brain) -> bool {
    brain.has_memory_value(types::NEAREST_PLAYER_HOLDING_WANTED_ITEM.id())
}

#[must_use]
pub fn was_hurt_recently(brain: &Brain) -> bool {
    brain.has_memory_value(types::HURT_BY.id())
}

#[must_use]
pub const fn is_idle(brain: &Brain) -> bool {
    brain.is_active(pumpkin_data::environment_attribute::Activity::Idle)
}

pub fn admire_gold_item(brain: &mut Brain) {
    brain.set_with_expiry(types::ADMIRING_ITEM, true, i64::from(ADMIRE_DURATION));
}

pub fn eat(brain: &mut Brain) {
    brain.set_with_expiry(types::ATE_RECENTLY, true, EAT_COOLDOWN);
}

pub fn init_memories(brain: &mut Brain, rng: &mut impl RngExt) {
    let delay = sample(TIME_BETWEEN_HUNTS, rng);
    brain.set_with_expiry(types::HUNTED_RECENTLY, true, i64::from(delay));
}

pub fn dont_kill_any_more_hoglins_for_a_while(brain: &mut Brain, rng: &mut impl RngExt) {
    let delay = sample(TIME_BETWEEN_HUNTS, rng);
    brain.set_with_expiry(types::HUNTED_RECENTLY, true, i64::from(delay));
}

#[must_use]
pub fn is_near_zombified(ctx: &VisibilityContext<'_>) -> bool {
    let Some(zombified) = ctx.brain.get(types::NEAREST_VISIBLE_ZOMBIFIED) else {
        return false;
    };
    let body_pos = ctx.mob.get_entity().pos.load();
    zombified
        .get_entity()
        .pos
        .load()
        .squared_distance_to_vec(&body_pos)
        < DESIRED_DISTANCE_FROM_ZOMBIFIED * DESIRED_DISTANCE_FROM_ZOMBIFIED
}

#[must_use]
pub fn is_near_avoid_target(ctx: &VisibilityContext<'_>) -> bool {
    let Some(avoid) = ctx.brain.get(types::AVOID_TARGET) else {
        return false;
    };
    let body_pos = ctx.mob.get_entity().pos.load();
    let range = f64::from(DESIRED_DISTANCE_FROM_ENTITY_WHEN_AVOIDING);
    avoid
        .get_entity()
        .pos
        .load()
        .squared_distance_to_vec(&body_pos)
        < range * range
}

#[must_use]
pub fn hoglins_outnumber_piglins(brain: &Brain) -> bool {
    let piglins = brain
        .get(types::VISIBLE_ADULT_PIGLIN_COUNT)
        .copied()
        .unwrap_or(0)
        + 1;
    let hoglins = brain
        .get(types::VISIBLE_ADULT_HOGLIN_COUNT)
        .copied()
        .unwrap_or(0);
    hoglins > piglins
}

#[must_use]
pub fn wants_to_stop_fleeing(brain: &Brain) -> bool {
    let Some(avoided) = brain.get(types::AVOID_TARGET) else {
        return true;
    };
    if avoided.get_entity().entity_type == &EntityType::HOGLIN {
        return !hoglins_outnumber_piglins(brain);
    }
    is_zombified(avoided.as_ref())
        && !brain.is_entity_value(types::NEAREST_VISIBLE_ZOMBIFIED, avoided.as_ref())
}

#[must_use]
pub fn get_anger_target(brain: &Brain, world: &World) -> Option<Arc<dyn EntityBase>> {
    crate::entity::ai::brain::behavior::utils::get_living_entity_from_uuid_memory(
        brain,
        world,
        types::ANGRY_AT,
    )
}

#[must_use]
pub fn get_nearest_visible_targetable_player(brain: &Brain) -> Option<Arc<Player>> {
    brain.get(types::NEAREST_VISIBLE_ATTACKABLE_PLAYER).cloned()
}

/// Vanilla `PiglinAi.findNearestValidAttackTarget`, in the same order.
#[must_use]
pub fn find_nearest_valid_attack_target(
    ctx: &VisibilityContext<'_>,
) -> Option<Arc<dyn EntityBase>> {
    if is_near_zombified(ctx) {
        return None;
    }
    if let Some(angry_at) = get_anger_target(ctx.brain, ctx.world)
        && is_entity_attackable_ignoring_line_of_sight(ctx, angry_at.as_ref())
    {
        return Some(angry_at);
    }
    if ctx.brain.has_memory_value(types::UNIVERSAL_ANGER.id())
        && let Some(player) = ctx.brain.get(types::NEAREST_VISIBLE_ATTACKABLE_PLAYER)
    {
        return Some(player.clone() as Arc<dyn EntityBase>);
    }
    if let Some(nemesis) = ctx.brain.get(types::NEAREST_VISIBLE_NEMESIS) {
        return Some(Arc::clone(nemesis));
    }
    let player = ctx
        .brain
        .get(types::NEAREST_TARGETABLE_PLAYER_NOT_WEARING_GOLD)?;
    let player = Arc::clone(player) as Arc<dyn EntityBase>;
    crate::entity::ai::brain::sensing::is_entity_attackable(ctx, player.as_ref()).then_some(player)
}

#[must_use]
pub fn is_nearest_valid_attack_target(
    ctx: &VisibilityContext<'_>,
    target: &Arc<dyn EntityBase>,
) -> bool {
    find_nearest_valid_attack_target(ctx)
        .is_some_and(|nearest| nearest.get_entity().entity_id == target.get_entity().entity_id)
}

#[must_use]
pub fn wants_to_dance(world: &World, killed_target: &Arc<dyn EntityBase>) -> bool {
    if killed_target.get_entity().entity_type != &EntityType::HOGLIN {
        return false;
    }
    // Game time seed, so the whole group rolls the same
    let mut rng = LegacyRand::from_seed(world.get_world_age() as u64);
    rng.next_f32() < PROBABILITY_OF_CELEBRATION_DANCE
}

/// Vanilla `setAngerTarget`; the caller must already hold the brain.
pub fn set_anger_target(
    brain: &mut Brain,
    ctx_world: &Arc<World>,
    mob: &dyn Mob,
    target: &Arc<dyn EntityBase>,
    rng: &mut impl RngExt,
) {
    let attackable = {
        let ctx = VisibilityContext {
            brain,
            world: ctx_world,
            mob,
            follow_range: mob
                .get_mob_entity()
                .living_entity
                .get_attribute_value(&pumpkin_data::attributes::Attributes::FOLLOW_RANGE),
        };
        is_entity_attackable_ignoring_line_of_sight(&ctx, target.as_ref())
    };
    if !attackable {
        return;
    }
    brain.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
    brain.set_with_expiry(
        types::ANGRY_AT,
        target.get_entity().entity_uuid,
        ANGER_DURATION,
    );
    let target_type = target.get_entity().entity_type;
    if target_type == &EntityType::HOGLIN
        && mob
            .cast_any()
            .downcast_ref::<PiglinEntity>()
            .is_some_and(PiglinEntity::can_hunt)
    {
        dont_kill_any_more_hoglins_for_a_while(brain, rng);
    }
    if target_type == &EntityType::PLAYER && ctx_world.level_info.load().game_rules.universal_anger
    {
        brain.set_with_expiry(types::UNIVERSAL_ANGER, true, ANGER_DURATION);
    }
}

pub fn set_avoid_target_and_dont_hunt_for_a_while(
    brain: &mut Brain,
    target: Arc<dyn EntityBase>,
    rng: &mut impl RngExt,
) {
    brain.erase(types::ANGRY_AT.id());
    brain.erase(types::ATTACK_TARGET.id());
    brain.erase(types::WALK_TARGET.id());
    let duration = sample(RETREAT_DURATION, rng);
    brain.set_with_expiry(types::AVOID_TARGET, target, i64::from(duration));
    dont_kill_any_more_hoglins_for_a_while(brain, rng);
}

/// Queues `set_anger_target` on every nearby adult piglin; vanilla writes them inline.
pub fn broadcast_anger_target(tick: &mut BrainTick<'_>, target: &Arc<dyn EntityBase>) {
    let adults = tick
        .brain
        .get(types::NEARBY_ADULT_PIGLINS)
        .cloned()
        .unwrap_or_default();
    let target_is_hoglin = target.get_entity().entity_type == &EntityType::HOGLIN;
    let target_can_be_hunted = hoglin_can_be_hunted(target.as_ref());

    for other in adults {
        let Some(mob_entity) = other.as_mob_entity() else {
            continue;
        };
        let hunter = as_piglin(other.as_ref()).is_some_and(PiglinEntity::can_hunt);
        if target_is_hoglin && !(target_can_be_hunted && hunter) {
            continue;
        }
        let target = Arc::clone(target);
        let body = Arc::clone(&other);
        mob_entity.post_to_brain(Box::new(move |tick| {
            set_anger_target_if_closer_than_current(tick, body.as_ref(), &target);
        }));
    }
}

/// The distance comparison vanilla does before overwriting an existing anger target.
fn set_anger_target_if_closer_than_current(
    tick: &mut BrainTick<'_>,
    body: &dyn EntityBase,
    new_target: &Arc<dyn EntityBase>,
) {
    let current = get_anger_target(tick.brain, tick.world);
    let nearest = get_nearest_target(body, current.clone(), Arc::clone(new_target));
    if current
        .is_some_and(|current| current.get_entity().entity_uuid == nearest.get_entity().entity_uuid)
    {
        return;
    }
    let world = Arc::clone(tick.world);
    let mut rng = tick.mob.get_random();
    set_anger_target(tick.brain, &world, tick.mob, &nearest, &mut rng);
}

pub fn broadcast_universal_anger(tick: &mut BrainTick<'_>) {
    let adults = tick
        .brain
        .get(types::NEARBY_ADULT_PIGLINS)
        .cloned()
        .unwrap_or_default();
    for other in adults {
        let Some(mob_entity) = other.as_mob_entity() else {
            continue;
        };
        mob_entity.post_to_brain(Box::new(|tick| {
            let Some(player) = get_nearest_visible_targetable_player(tick.brain) else {
                return;
            };
            let player = player as Arc<dyn EntityBase>;
            let world = Arc::clone(tick.world);
            let mut rng = tick.mob.get_random();
            set_anger_target(tick.brain, &world, tick.mob, &player, &mut rng);
        }));
    }
}

pub fn broadcast_retreat(tick: &mut BrainTick<'_>, target: &Arc<dyn EntityBase>) {
    let visible = tick
        .brain
        .get(types::NEAREST_VISIBLE_ADULT_PIGLINS)
        .cloned()
        .unwrap_or_default();
    let seed = tick.time;
    for other in visible {
        if as_piglin(other.as_ref()).is_none() {
            continue;
        }
        let Some(mob_entity) = other.as_mob_entity() else {
            continue;
        };
        let target = Arc::clone(target);
        let body = Arc::clone(&other);
        mob_entity.post_to_brain(Box::new(move |tick| {
            retreat_from_nearest_target(tick.brain, body.as_ref(), target, seed);
        }));
    }
}

fn retreat_from_nearest_target(
    brain: &mut Brain,
    body: &dyn EntityBase,
    new_avoid_target: Arc<dyn EntityBase>,
    seed: i64,
) {
    let mut nearest = new_avoid_target;
    nearest = get_nearest_target(body, brain.get(types::AVOID_TARGET).cloned(), nearest);
    nearest = get_nearest_target(body, brain.get(types::ATTACK_TARGET).cloned(), nearest);
    let _ = seed;
    let duration =
        RETREAT_DURATION.0 + rand::random_range(0..RETREAT_DURATION.1 + 1 - RETREAT_DURATION.0);
    brain.erase(types::ANGRY_AT.id());
    brain.erase(types::ATTACK_TARGET.id());
    brain.erase(types::WALK_TARGET.id());
    brain.set_with_expiry(types::AVOID_TARGET, nearest, i64::from(duration));
    let hunt_delay = TIME_BETWEEN_HUNTS.0
        + rand::random_range(0..TIME_BETWEEN_HUNTS.1 + 1 - TIME_BETWEEN_HUNTS.0);
    brain.set_with_expiry(types::HUNTED_RECENTLY, true, i64::from(hunt_delay));
}

#[must_use]
pub fn get_sound_for_current_activity(
    ctx: &VisibilityContext<'_>,
    is_converting: bool,
) -> Option<Sound> {
    use pumpkin_data::environment_attribute::Activity;

    let activity = ctx.brain.get_active_non_core_activity()?;
    Some(if activity == Activity::Fight {
        Sound::EntityPiglinAngry
    } else if is_converting || (activity == Activity::Avoid && is_near_avoid_target(ctx)) {
        Sound::EntityPiglinRetreat
    } else if activity == Activity::AdmireItem {
        Sound::EntityPiglinAdmiringItem
    } else if activity == Activity::Celebrate {
        Sound::EntityPiglinCelebrate
    } else if sees_player_holding_loved_item(ctx.brain) {
        Sound::EntityPiglinJealous
    } else if is_near_repellent(ctx.brain) {
        Sound::EntityPiglinRetreat
    } else {
        Sound::EntityPiglinAmbient
    })
}

#[must_use]
pub fn get_random_nearby_pos(mob: &dyn Mob) -> Vector3<f64> {
    land_random_pos::get_pos(mob, 4, 2).unwrap_or_else(|| mob.get_entity().pos.load())
}

pub fn stop_walking(brain: &mut Brain, mob_entity: &MobEntity) {
    brain.erase(types::WALK_TARGET.id());
    mob_entity
        .navigator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .stop();
}

#[must_use]
pub fn start_admiring_item_if_seen(admire_duration: i32) -> OneShot {
    OneShot::new(
        "StartAdmiringItemIfSeen",
        vec![
            (
                types::NEAREST_VISIBLE_WANTED_ITEM.id(),
                MemoryStatus::ValuePresent,
            ),
            (types::ADMIRING_ITEM.id(), MemoryStatus::ValueAbsent),
            (types::ADMIRING_DISABLED.id(), MemoryStatus::ValueAbsent),
            (
                types::DISABLE_WALK_TO_ADMIRE_ITEM.id(),
                MemoryStatus::ValueAbsent,
            ),
        ],
        move |tick| {
            let Some(item) = tick.brain.get(types::NEAREST_VISIBLE_WANTED_ITEM).cloned() else {
                return false;
            };
            let Some(item_entity) = item.get_item_entity() else {
                return false;
            };
            let loved = {
                let stack = item_entity
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                is_loved_item(&stack)
            };
            if !loved {
                return false;
            }
            tick.brain
                .set_with_expiry(types::ADMIRING_ITEM, true, i64::from(admire_duration));
            true
        },
    )
}

#[must_use]
pub fn stop_admiring_if_item_too_far_away(max_distance_to_item: i32) -> OneShot {
    let max_distance_squared = f64::from(max_distance_to_item * max_distance_to_item);
    OneShot::with_required(
        "StopAdmiringIfItemTooFarAway",
        vec![(types::ADMIRING_ITEM.id(), MemoryStatus::ValuePresent)],
        vec![types::NEAREST_VISIBLE_WANTED_ITEM.id()],
        move |tick| {
            if holds_something_in_off_hand(tick.mob) {
                return false;
            }
            let body_pos = tick.mob.get_entity().pos.load();
            let still_close = tick
                .brain
                .get(types::NEAREST_VISIBLE_WANTED_ITEM)
                .is_some_and(|item| {
                    item.get_entity()
                        .pos
                        .load()
                        .squared_distance_to_vec(&body_pos)
                        < max_distance_squared
                });
            if still_close {
                return false;
            }
            tick.brain.erase(types::ADMIRING_ITEM.id());
            true
        },
    )
}

#[must_use]
pub fn stop_admiring_if_tired_of_trying_to_reach_item(
    max_time_to_reach_item: i32,
    disable_time: i32,
) -> OneShot {
    OneShot::with_required(
        "StopAdmiringIfTiredOfTryingToReachItem",
        vec![
            (types::ADMIRING_ITEM.id(), MemoryStatus::ValuePresent),
            (
                types::NEAREST_VISIBLE_WANTED_ITEM.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        vec![
            types::TIME_TRYING_TO_REACH_ADMIRE_ITEM.id(),
            types::DISABLE_WALK_TO_ADMIRE_ITEM.id(),
        ],
        move |tick| {
            if holds_something_in_off_hand(tick.mob) {
                return false;
            }
            let Some(time_trying) = tick
                .brain
                .get(types::TIME_TRYING_TO_REACH_ADMIRE_ITEM)
                .copied()
            else {
                tick.brain.set(types::TIME_TRYING_TO_REACH_ADMIRE_ITEM, 0);
                return true;
            };
            if time_trying > max_time_to_reach_item {
                tick.brain.erase(types::ADMIRING_ITEM.id());
                tick.brain
                    .erase(types::TIME_TRYING_TO_REACH_ADMIRE_ITEM.id());
                tick.brain.set_with_expiry(
                    types::DISABLE_WALK_TO_ADMIRE_ITEM,
                    true,
                    i64::from(disable_time),
                );
            } else {
                tick.brain
                    .set(types::TIME_TRYING_TO_REACH_ADMIRE_ITEM, time_trying + 1);
            }
            true
        },
    )
}

#[must_use]
pub fn stop_holding_item_if_no_longer_admiring() -> OneShot {
    OneShot::new(
        "StopHoldingItemIfNoLongerAdmiring",
        vec![(types::ADMIRING_ITEM.id(), MemoryStatus::ValueAbsent)],
        |tick| {
            let Some(piglin) = tick.mob.cast_any().downcast_ref::<PiglinEntity>() else {
                return false;
            };
            let off_hand = piglin.off_hand_item();
            if off_hand.is_empty() || blocks_attacks(&off_hand) {
                return false;
            }
            stop_holding_off_hand_item(tick, piglin, true);
            true
        },
    )
}

#[must_use]
pub fn start_hunting_hoglin() -> OneShot {
    OneShot::with_required(
        "StartHuntingHoglin",
        vec![
            (
                types::NEAREST_VISIBLE_HUNTABLE_HOGLIN.id(),
                MemoryStatus::ValuePresent,
            ),
            (types::ANGRY_AT.id(), MemoryStatus::ValueAbsent),
            (types::HUNTED_RECENTLY.id(), MemoryStatus::ValueAbsent),
        ],
        vec![types::NEAREST_VISIBLE_ADULT_PIGLINS.id()],
        |tick| {
            let is_baby = tick
                .mob
                .cast_any()
                .downcast_ref::<PiglinEntity>()
                .is_some_and(PiglinEntity::is_baby);
            if is_baby {
                return false;
            }
            let nearby = tick
                .brain
                .get(types::NEAREST_VISIBLE_ADULT_PIGLINS)
                .cloned()
                .unwrap_or_default();
            if nearby.iter().any(other_has_hunted_recently) {
                return false;
            }
            let Some(target) = tick
                .brain
                .get(types::NEAREST_VISIBLE_HUNTABLE_HOGLIN)
                .cloned()
            else {
                return false;
            };

            let world = Arc::clone(tick.world);
            let mut rng = tick.mob.get_random();
            set_anger_target(tick.brain, &world, tick.mob, &target, &mut rng);
            dont_kill_any_more_hoglins_for_a_while(tick.brain, &mut rng);
            broadcast_anger_target(tick, &target);
            for other in nearby {
                let Some(mob_entity) = other.as_mob_entity() else {
                    continue;
                };
                let delay = sample(TIME_BETWEEN_HUNTS, &mut rng);
                mob_entity.post_to_brain(Box::new(move |tick| {
                    tick.brain
                        .set_with_expiry(types::HUNTED_RECENTLY, true, i64::from(delay));
                }));
            }
            true
        },
    )
}

/// A contended lock reads as "has not hunted".
fn other_has_hunted_recently(other: &Arc<dyn EntityBase>) -> bool {
    let Some(mob_entity) = other.as_mob_entity() else {
        return false;
    };
    // Never block on another mob's brain from inside a brain tick
    let Ok(brain) = mob_entity.brain.try_lock() else {
        return false;
    };
    brain.has_memory_value(types::HUNTED_RECENTLY.id())
}

#[must_use]
pub fn remember_if_hoglin_was_killed() -> OneShot {
    OneShot::with_required(
        "RememberIfHoglinWasKilled",
        vec![(types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent)],
        vec![types::HUNTED_RECENTLY.id()],
        |tick| {
            let killed_hoglin = tick.brain.get(types::ATTACK_TARGET).is_some_and(|target| {
                target.get_entity().entity_type == &EntityType::HOGLIN
                    && is_dead_or_dying(target.as_ref())
            });
            if killed_hoglin {
                let delay = sample(TIME_BETWEEN_HUNTS, &mut tick.mob.get_random());
                tick.brain
                    .set_with_expiry(types::HUNTED_RECENTLY, true, i64::from(delay));
            }
            true
        },
    )
}

fn holds_something_in_off_hand(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<PiglinEntity>()
        .is_some_and(|piglin| !piglin.off_hand_item().is_empty())
}

fn blocks_attacks(stack: &ItemStack) -> bool {
    stack
        .get_data_component::<pumpkin_data::data_component_impl::BlocksAttacksImpl>()
        .is_some()
}

#[must_use]
pub fn is_not_holding_loved_item_in_off_hand(piglin: &PiglinEntity) -> bool {
    let off_hand = piglin.off_hand_item();
    off_hand.is_empty() || !is_loved_item(&off_hand)
}

#[must_use]
pub fn can_admire(brain: &Brain, piglin: &PiglinEntity, stack: &ItemStack) -> bool {
    !is_admiring_disabled(brain)
        && !is_admiring_item(brain)
        && piglin.is_adult()
        && is_barter_currency(stack)
}

/// Vanilla `PiglinAi.wantsToPickup`, in the same order of rejections.
#[must_use]
pub fn wants_to_pickup(brain: &Brain, piglin: &PiglinEntity, stack: &ItemStack) -> bool {
    if piglin.is_baby()
        && stack
            .item
            .has_tag(&tag::Item::MINECRAFT_IGNORED_BY_PIGLIN_BABIES)
    {
        return false;
    }
    if stack.item.has_tag(&tag::Item::MINECRAFT_PIGLIN_REPELLENTS) {
        return false;
    }
    if is_admiring_disabled(brain) && brain.has_memory_value(types::ATTACK_TARGET.id()) {
        return false;
    }
    if is_barter_currency(stack) {
        return is_not_holding_loved_item_in_off_hand(piglin);
    }
    let has_space = piglin.can_add_to_inventory(stack);
    if stack.item.id == Item::GOLD_NUGGET.id {
        return has_space;
    }
    if is_food(stack) {
        return !has_eaten_recently(brain) && has_space;
    }
    if is_loved_item(stack) {
        return is_not_holding_loved_item_in_off_hand(piglin) && has_space;
    }
    piglin.can_replace_current_item_for(stack)
}

fn take_off_hand_item(piglin: &PiglinEntity) -> ItemStack {
    piglin
        .get_mob_entity()
        .set_item_slot(&EquipmentSlot::OFF_HAND, ItemStack::EMPTY.clone())
}

fn equip_if_possible(piglin: &PiglinEntity, stack: &ItemStack) -> bool {
    !crate::entity::mob::equipment::equip_item_if_possible(piglin, stack.clone()).is_empty()
}

/// Vanilla `PiglinAi.pickUpItem`.
pub fn pick_up_item(tick: &mut BrainTick<'_>, piglin: &PiglinEntity, taken: ItemStack) {
    let mob_entity = tick.mob.get_mob_entity();
    stop_walking(tick.brain, mob_entity);

    if is_loved_item(&taken) {
        tick.brain
            .erase(types::TIME_TRYING_TO_REACH_ADMIRE_ITEM.id());
        piglin.hold_in_off_hand(taken);
        admire_gold_item(tick.brain);
        return;
    }
    if is_food(&taken) && !has_eaten_recently(tick.brain) {
        eat(tick.brain);
        return;
    }
    if !equip_if_possible(piglin, &taken) {
        put_in_inventory(tick, piglin, taken);
    }
}

fn put_in_inventory(tick: &BrainTick<'_>, piglin: &PiglinEntity, stack: ItemStack) {
    if let Some(left_over) = piglin.add_to_inventory(stack) {
        throw_items_toward_random_pos(tick, piglin, vec![left_over]);
    }
}

/// Vanilla `PiglinAi.stopHoldingOffHandItem`.
pub fn stop_holding_off_hand_item(
    tick: &mut BrainTick<'_>,
    piglin: &PiglinEntity,
    bartering_enabled: bool,
) {
    let stack = take_off_hand_item(piglin);

    if piglin.is_adult() {
        let barter_currency = is_barter_currency(&stack);
        if bartering_enabled && barter_currency {
            let response = get_barter_response_items(tick);
            throw_items(tick, piglin, response);
        } else if !barter_currency && !equip_if_possible(piglin, &stack) {
            put_in_inventory(tick, piglin, stack);
        }
        return;
    }

    if equip_if_possible(piglin, &stack) {
        return;
    }
    let main_hand = piglin.main_hand_item();
    if is_loved_item(&main_hand) {
        put_in_inventory(tick, piglin, main_hand);
    } else {
        throw_items(tick, piglin, vec![main_hand]);
    }
    piglin.hold_in_main_hand(stack);
}

pub fn cancel_admiring(tick: &BrainTick<'_>, piglin: &PiglinEntity) {
    if is_admiring_item(tick.brain) && !piglin.off_hand_item().is_empty() {
        let stack = take_off_hand_item(piglin);
        piglin.get_mob_entity().spawn_at_location(stack);
    }
}

fn throw_items(tick: &BrainTick<'_>, piglin: &PiglinEntity, stacks: Vec<ItemStack>) {
    match tick.brain.get(types::NEAREST_VISIBLE_PLAYER).cloned() {
        Some(player) => {
            let target = player.get_entity().pos.load();
            throw_items_toward_pos(piglin, stacks, target);
        }
        None => throw_items_toward_random_pos(tick, piglin, stacks),
    }
}

fn throw_items_toward_random_pos(
    tick: &BrainTick<'_>,
    piglin: &PiglinEntity,
    stacks: Vec<ItemStack>,
) {
    let target = get_random_nearby_pos(tick.mob);
    throw_items_toward_pos(piglin, stacks, target);
}

fn throw_items_toward_pos(piglin: &PiglinEntity, stacks: Vec<ItemStack>, target: Vector3<f64>) {
    if stacks.is_empty() {
        return;
    }
    let living = &piglin.get_mob_entity().living_entity;
    living.swing_off_hand();

    let entity = &living.entity;
    let world = entity.world.load();
    let pos = entity.pos.load();
    let hand_pos = Vector3::new(
        pos.x,
        entity.get_eye_y() - THROW_HAND_Y_DISTANCE_FROM_EYE,
        pos.z,
    );
    let direction =
        Vector3::new(target.x - pos.x, target.y + 1.0 - pos.y, target.z - pos.z).normalize();
    let velocity = Vector3::new(
        direction.x * THROW_SPEED,
        direction.y * THROW_SPEED,
        direction.z * THROW_SPEED,
    );

    for stack in stacks {
        if stack.is_empty() {
            continue;
        }
        let item_entity = ItemEntity::new_with_velocity(
            Entity::new(world.clone(), hand_pos, &EntityType::ITEM),
            stack,
            velocity,
            ItemEntity::DEFAULT_PICKUP_DELAY,
        );
        world.spawn_entity(Arc::new(item_entity));
    }
}

/// The half of vanilla `PiglinAi.mobInteract` that runs once the caller has taken the item.
pub fn start_admiring(tick: &mut BrainTick<'_>, piglin: &PiglinEntity, taken: ItemStack) {
    piglin.hold_in_off_hand(taken);
    admire_gold_item(tick.brain);
    let mob_entity = tick.mob.get_mob_entity();
    stop_walking(tick.brain, mob_entity);
}

/// Vanilla `PiglinAi.wasHurtBy`.
pub fn was_hurt_by(
    tick: &mut BrainTick<'_>,
    piglin: &PiglinEntity,
    attacker: &Arc<dyn EntityBase>,
) {
    if as_piglin(attacker.as_ref()).is_some() {
        return;
    }
    if piglin.is_holding_item_in_off_hand() {
        stop_holding_off_hand_item(tick, piglin, false);
    }
    tick.brain.erase(types::CELEBRATE_LOCATION.id());
    tick.brain.erase(types::DANCING.id());
    tick.brain.erase(types::ADMIRING_ITEM.id());

    let attacker_type = attacker.get_entity().entity_type;
    if attacker_type == &EntityType::PLAYER {
        tick.brain
            .set_with_expiry(types::ADMIRING_DISABLED, true, HIT_BY_PLAYER_MEMORY_TIMEOUT);
    }
    let avoid_is_other_type = tick
        .brain
        .get(types::AVOID_TARGET)
        .is_some_and(|avoid| avoid.get_entity().entity_type != attacker_type);
    if avoid_is_other_type {
        tick.brain.erase(types::AVOID_TARGET.id());
    }

    let mut rng = tick.mob.get_random();
    if piglin.is_baby() {
        tick.brain.set_with_expiry(
            types::AVOID_TARGET,
            Arc::clone(attacker),
            BABY_FLEE_DURATION_AFTER_GETTING_HIT,
        );
        let attackable = {
            let ctx = tick.visibility();
            is_entity_attackable_ignoring_line_of_sight(&ctx, attacker.as_ref())
        };
        if attackable {
            broadcast_anger_target(tick, attacker);
        }
        return;
    }
    if attacker_type == &EntityType::HOGLIN && hoglins_outnumber_piglins(tick.brain) {
        set_avoid_target_and_dont_hunt_for_a_while(tick.brain, Arc::clone(attacker), &mut rng);
        broadcast_retreat(tick, attacker);
        return;
    }
    maybe_retaliate(tick, attacker);
}

/// Vanilla `PiglinAi.maybeRetaliate`, shared with the brute.
pub fn maybe_retaliate(tick: &mut BrainTick<'_>, attacker: &Arc<dyn EntityBase>) {
    if tick
        .brain
        .is_active(pumpkin_data::environment_attribute::Activity::Avoid)
    {
        return;
    }
    let (attackable, much_further) = {
        let ctx = tick.visibility();
        (
            is_entity_attackable_ignoring_line_of_sight(&ctx, attacker.as_ref()),
            is_other_target_much_further_away_than_current_attack_target(
                ctx.brain,
                ctx.mob,
                attacker.as_ref(),
                4.0,
            ),
        )
    };
    if !attackable || much_further {
        return;
    }

    let world = Arc::clone(tick.world);
    let mut rng = tick.mob.get_random();
    if attacker.get_entity().entity_type == &EntityType::PLAYER
        && world.level_info.load().game_rules.universal_anger
    {
        match get_nearest_visible_targetable_player(tick.brain) {
            Some(player) => {
                let player = player as Arc<dyn EntityBase>;
                set_anger_target(tick.brain, &world, tick.mob, &player, &mut rng);
            }
            None => set_anger_target(tick.brain, &world, tick.mob, attacker, &mut rng),
        }
        broadcast_universal_anger(tick);
    } else {
        set_anger_target(tick.brain, &world, tick.mob, attacker, &mut rng);
        broadcast_anger_target(tick, attacker);
    }
}

/// Vanilla `PiglinAi.getBarterResponseItems`.
#[must_use]
pub fn get_barter_response_items(tick: &BrainTick<'_>) -> Vec<ItemStack> {
    let Some(table) = tick.world.get_loot_table(BARTERING_LOOT_TABLE) else {
        return Vec::new();
    };
    let params = LootContextParameters {
        this_entity: Some(tick.mob.get_entity().entity_type),
        ..Default::default()
    };
    crate::world::loot::generate_loot_from_handle(&table, rand::random(), &params)
}

fn look_behaviors() -> Vec<(Box<dyn BehaviorControl>, i32)> {
    vec![
        (
            Box::new(behavior::set_entity_look_target_of_type(
                &EntityType::PLAYER,
                MAX_LOOK_DIST,
            )),
            1,
        ),
        (
            Box::new(behavior::set_entity_look_target_of_type(
                &EntityType::PIGLIN,
                MAX_LOOK_DIST,
            )),
            1,
        ),
        (
            Box::new(behavior::set_entity_look_target_any(MAX_LOOK_DIST)),
            1,
        ),
    ]
}

fn idle_look_behaviors() -> Box<dyn BehaviorControl> {
    let mut weighted = look_behaviors();
    weighted.push((Box::new(behavior::DoNothing::new(30, 60)), 1));
    Box::new(behavior::run_one(weighted))
}

fn idle_movement_behaviors() -> Box<dyn BehaviorControl> {
    Box::new(behavior::run_one(vec![
        (
            Box::new(behavior::stroll(SPEED_MULTIPLIER_WHEN_IDLING, true)),
            2,
        ),
        (
            Box::new(behavior::interact_with(
                &EntityType::PIGLIN,
                INTERACTION_RANGE,
                |_| true,
                |_| true,
                types::INTERACTION_TARGET,
                SPEED_MULTIPLIER_WHEN_IDLING,
                2,
            )),
            2,
        ),
        (
            Box::new(behavior::set_walk_target_from_look_target(
                |tick| !sees_player_holding_loved_item(tick.brain),
                SPEED_MULTIPLIER_WHEN_IDLING,
                3,
            )),
            2,
        ),
        (Box::new(behavior::DoNothing::new(30, 60)), 1),
    ]))
}

fn avoid_repellent() -> Box<dyn BehaviorControl> {
    Box::new(behavior::set_walk_target_away_from_pos(
        types::NEAREST_REPELLENT,
        SPEED_MULTIPLIER_WHEN_AVOIDING,
        REPELLENT_DETECTION_RANGE_HORIZONTAL,
        false,
    ))
}

fn baby_avoid_nemesis() -> Box<dyn BehaviorControl> {
    Box::new(behavior::copy_memory_with_expiry(
        "BabyAvoidNemesis",
        |tick| {
            tick.mob
                .cast_any()
                .downcast_ref::<PiglinEntity>()
                .is_some_and(PiglinEntity::is_baby)
        },
        types::NEAREST_VISIBLE_NEMESIS,
        types::AVOID_TARGET,
        BABY_AVOID_NEMESIS_DURATION.0,
        BABY_AVOID_NEMESIS_DURATION.1,
    ))
}

fn avoid_zombified() -> Box<dyn BehaviorControl> {
    Box::new(behavior::copy_memory_with_expiry(
        "AvoidZombified",
        |tick| is_near_zombified(&tick.visibility()),
        types::NEAREST_VISIBLE_ZOMBIFIED,
        types::AVOID_TARGET,
        AVOID_ZOMBIFIED_DURATION.0,
        AVOID_ZOMBIFIED_DURATION.1,
    ))
}

fn baby_sometimes_ride_baby_hoglin() -> Box<dyn BehaviorControl> {
    let mut ticker = behavior::Ticker::new(RIDE_START_INTERVAL.0, RIDE_START_INTERVAL.1);
    Box::new(behavior::copy_memory_with_expiry(
        "BabySometimesRideBabyHoglin",
        move |tick| {
            let is_baby = tick
                .mob
                .cast_any()
                .downcast_ref::<PiglinEntity>()
                .is_some_and(PiglinEntity::is_baby);
            is_baby && ticker.tick_down_and_check(&mut tick.mob.get_random())
        },
        types::NEAREST_VISIBLE_BABY_HOGLIN,
        types::RIDE_TARGET,
        RIDE_DURATION.0,
        RIDE_DURATION.1,
    ))
}

fn init_core_activity() -> ActivityData {
    ActivityData::with_priority_start(
        Activity::Core,
        0,
        vec![
            Box::new(behavior::Timed::new(behavior::LookAtTargetSink::new(
                45, 90,
            ))),
            Box::new(behavior::Timed::new(behavior::MoveToTargetSink::default())),
            Box::new(behavior::interact_with_door()),
            baby_avoid_nemesis(),
            avoid_zombified(),
            Box::new(stop_holding_item_if_no_longer_admiring()),
            Box::new(start_admiring_item_if_seen(ADMIRE_DURATION)),
            Box::new(behavior::start_celebrating_if_target_dead(
                CELEBRATION_TIME,
                |tick, target| wants_to_dance(tick.world, target),
            )),
            Box::new(behavior::stop_being_angry_if_target_dead()),
        ],
    )
}

fn init_idle_activity() -> ActivityData {
    ActivityData::with_priority_start(
        Activity::Idle,
        10,
        vec![
            Box::new(behavior::set_entity_look_target(
                |entity| is_player_holding_loved_item(entity.as_ref()),
                MAX_LOOK_DIST_FOR_PLAYER_HOLDING_LOVED_ITEM,
            )),
            Box::new(behavior::start_attacking(
                |tick| {
                    tick.mob
                        .cast_any()
                        .downcast_ref::<PiglinEntity>()
                        .is_some_and(PiglinEntity::is_adult)
                },
                |tick| find_nearest_valid_attack_target(&tick.visibility()),
            )),
            Box::new(behavior::one_shot::sequence(
                "TriggerIfCanHunt",
                |tick| {
                    tick.mob
                        .cast_any()
                        .downcast_ref::<PiglinEntity>()
                        .is_some_and(PiglinEntity::can_hunt)
                },
                start_hunting_hoglin(),
            )),
            avoid_repellent(),
            baby_sometimes_ride_baby_hoglin(),
            idle_look_behaviors(),
            idle_movement_behaviors(),
            Box::new(behavior::set_look_and_interact(&EntityType::PLAYER, 4)),
        ],
    )
}

fn init_admire_item_activity() -> ActivityData {
    ActivityData::with_gate_memory(
        Activity::AdmireItem,
        10,
        vec![
            Box::new(behavior::go_to_wanted_item(
                |tick| {
                    tick.mob
                        .cast_any()
                        .downcast_ref::<PiglinEntity>()
                        .is_some_and(is_not_holding_loved_item_in_off_hand)
                },
                SPEED_MULTIPLIER_WHEN_GOING_TO_WANTED_ITEM,
                true,
                MAX_DISTANCE_TO_WALK_TO_ITEM,
            )),
            Box::new(stop_admiring_if_item_too_far_away(
                MAX_DISTANCE_TO_WALK_TO_ITEM,
            )),
            Box::new(stop_admiring_if_tired_of_trying_to_reach_item(
                MAX_TIME_TO_WALK_TO_ITEM,
                HOW_LONG_TIME_TO_DISABLE_ADMIRE_WALKING_IF_CANT_REACH_ITEM,
            )),
        ],
        types::ADMIRING_ITEM.id(),
    )
}

fn init_fight_activity() -> ActivityData {
    ActivityData::with_gate_memory(
        Activity::Fight,
        10,
        vec![
            Box::new(behavior::stop_attacking_if_target_invalid(
                |tick, target| !is_nearest_valid_attack_target(&tick.visibility(), target),
                |_, _| {},
                true,
            )),
            Box::new(behavior::one_shot::sequence(
                "TriggerIfHasCrossbow",
                |tick| has_crossbow(tick.mob),
                behavior::back_up_if_too_close(
                    MIN_DESIRED_DIST_FROM_TARGET_WHEN_HOLDING_CROSSBOW,
                    SPEED_WHEN_STRAFING_BACK_FROM_TARGET,
                ),
            )),
            Box::new(behavior::set_walk_target_from_attack_target_if_target_out_of_reach(1.0)),
            Box::new(behavior::melee_attack(|_| true, MELEE_ATTACK_COOLDOWN)),
            Box::new(behavior::Timed::new(behavior::CrossbowAttack::new())),
            Box::new(remember_if_hoglin_was_killed()),
            Box::new(behavior::erase_memory_if(
                "EraseAttackTargetIfNearZombified",
                |tick| is_near_zombified(&tick.visibility()),
                types::ATTACK_TARGET.id(),
            )),
        ],
        types::ATTACK_TARGET.id(),
    )
}

fn init_celebrate_activity() -> ActivityData {
    ActivityData::with_gate_memory(
        Activity::Celebrate,
        10,
        vec![
            avoid_repellent(),
            Box::new(behavior::set_entity_look_target(
                |entity| is_player_holding_loved_item(entity.as_ref()),
                MAX_LOOK_DIST_FOR_PLAYER_HOLDING_LOVED_ITEM,
            )),
            Box::new(behavior::start_attacking(
                |tick| {
                    tick.mob
                        .cast_any()
                        .downcast_ref::<PiglinEntity>()
                        .is_some_and(PiglinEntity::is_adult)
                },
                |tick| find_nearest_valid_attack_target(&tick.visibility()),
            )),
            Box::new(behavior::one_shot::sequence(
                "TriggerIfNotDancing",
                |tick| !is_dancing(tick.mob),
                behavior::go_to_target_location(
                    types::CELEBRATE_LOCATION,
                    2,
                    SPEED_MULTIPLIER_WHEN_GOING_TO_CELEBRATE_LOCATION,
                ),
            )),
            Box::new(behavior::one_shot::sequence(
                "TriggerIfDancing",
                |tick| is_dancing(tick.mob),
                behavior::go_to_target_location(
                    types::CELEBRATE_LOCATION,
                    4,
                    SPEED_MULTIPLIER_WHEN_DANCING,
                ),
            )),
            Box::new(behavior::run_one(vec![
                (
                    Box::new(behavior::set_entity_look_target_of_type(
                        &EntityType::PIGLIN,
                        MAX_LOOK_DIST,
                    )),
                    1,
                ),
                (
                    Box::new(behavior::stroll_with_range(
                        SPEED_MULTIPLIER_WHEN_IDLING,
                        2,
                        1,
                        true,
                    )),
                    1,
                ),
                (Box::new(behavior::DoNothing::new(10, 20)), 1),
            ])),
        ],
        types::CELEBRATE_LOCATION.id(),
    )
}

fn init_retreat_activity() -> ActivityData {
    ActivityData::with_gate_memory(
        Activity::Avoid,
        10,
        vec![
            Box::new(behavior::set_walk_target_away_from_entity(
                types::AVOID_TARGET,
                SPEED_MULTIPLIER_WHEN_RETREATING,
                DESIRED_DISTANCE_FROM_ENTITY_WHEN_AVOIDING,
                true,
            )),
            idle_look_behaviors(),
            idle_movement_behaviors(),
            Box::new(behavior::erase_memory_if(
                "EraseAvoidTargetIfDoneFleeing",
                |tick| wants_to_stop_fleeing(tick.brain),
                types::AVOID_TARGET.id(),
            )),
        ],
        types::AVOID_TARGET.id(),
    )
}

fn init_ride_hoglin_activity() -> ActivityData {
    ActivityData::with_gate_memory(
        Activity::Ride,
        10,
        vec![
            Box::new(behavior::mount(SPEED_MULTIPLIER_WHEN_MOUNTING)),
            Box::new(behavior::set_entity_look_target(
                |entity| is_player_holding_loved_item(entity.as_ref()),
                MAX_LOOK_DIST,
            )),
            Box::new(behavior::one_shot::sequence(
                "LookAroundWhileRiding",
                |tick| tick.mob.is_passenger(),
                behavior::one_shot::trigger_one_shuffled("RideLookAround", ride_look_triggers()),
            )),
            Box::new(behavior::dismount_or_skip_mounting(
                MAX_WALK_DISTANCE_TO_START_RIDING,
                wants_to_stop_riding,
            )),
        ],
        types::RIDE_TARGET.id(),
    )
}

fn ride_look_triggers() -> Vec<(behavior::Trigger, i32)> {
    let mut triggers: Vec<(behavior::Trigger, i32)> = Vec::new();
    for entity_type in [&EntityType::PLAYER, &EntityType::PIGLIN] {
        let mut look = behavior::set_entity_look_target_of_type(entity_type, MAX_LOOK_DIST);
        triggers.push((
            Box::new(move |tick| behavior::BehaviorControl::try_start(&mut look, tick)),
            1,
        ));
    }
    let mut look_any = behavior::set_entity_look_target_any(MAX_LOOK_DIST);
    triggers.push((
        Box::new(move |tick| behavior::BehaviorControl::try_start(&mut look_any, tick)),
        1,
    ));
    triggers.push((Box::new(|_| true), 1));
    triggers
}

#[must_use]
fn has_crossbow(mob: &dyn Mob) -> bool {
    let Ok(equipment) = mob
        .get_mob_entity()
        .living_entity
        .entity_equipment
        .try_lock()
    else {
        return false;
    };
    equipment.get(&EquipmentSlot::MAIN_HAND).item.id == Item::CROSSBOW.id
        || equipment.get(&EquipmentSlot::OFF_HAND).item.id == Item::CROSSBOW.id
}

#[must_use]
fn is_dancing(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<PiglinEntity>()
        .is_some_and(PiglinEntity::is_dancing)
}

/// Vanilla `PiglinAi.wantsToStopRiding`.
#[must_use]
fn wants_to_stop_riding(tick: &BrainTick<'_>, vehicle: &Arc<dyn EntityBase>) -> bool {
    let Some(mob_entity) = vehicle.as_mob_entity() else {
        return false;
    };
    let vehicle_is_baby = is_baby_hoglin(vehicle.as_ref())
        || as_piglin(vehicle.as_ref()).is_some_and(PiglinEntity::is_baby);
    if !vehicle_is_baby || !crate::entity::ai::brain::behavior::utils::is_alive(vehicle.as_ref()) {
        return true;
    }
    if was_hurt_recently(tick.brain) {
        return true;
    }
    // Never block on another mob's brain from inside a brain tick
    let vehicle_was_hurt = mob_entity
        .brain
        .try_lock()
        .is_ok_and(|brain| brain.has_memory_value(types::HURT_BY.id()));
    if vehicle_was_hurt {
        return true;
    }
    as_piglin(vehicle.as_ref()).is_some() && vehicle.get_entity().get_vehicle().is_none()
}

#[must_use]
fn is_baby_riding_baby(mob: &dyn Mob) -> bool {
    let Some(piglin) = mob.cast_any().downcast_ref::<PiglinEntity>() else {
        return false;
    };
    if !piglin.is_baby() {
        return false;
    }
    let Some(vehicle) = mob.get_entity().get_vehicle() else {
        return false;
    };
    is_baby_hoglin(vehicle.as_ref())
        || as_piglin(vehicle.as_ref()).is_some_and(PiglinEntity::is_baby)
}

/// Vanilla `PiglinAi.updateActivity`, run right after the brain tick.
pub fn update_activity(tick: &mut BrainTick<'_>) {
    let old_activity = tick.brain.get_active_non_core_activity();
    tick.brain.set_active_activity_to_first_valid(&[
        Activity::AdmireItem,
        Activity::Fight,
        Activity::Avoid,
        Activity::Celebrate,
        Activity::Ride,
        Activity::Idle,
    ]);
    let new_activity = tick.brain.get_active_non_core_activity();

    if old_activity != new_activity {
        let is_converting = tick
            .mob
            .cast_any()
            .downcast_ref::<PiglinEntity>()
            .is_some_and(|piglin| piglin.is_converting(tick.world));
        let sound = {
            let ctx = tick.visibility();
            get_sound_for_current_activity(&ctx, is_converting)
        };
        if let Some(sound) = sound
            && let Some(piglin) = tick.mob.cast_any().downcast_ref::<PiglinEntity>()
        {
            piglin.make_sound(sound);
        }
    }

    let has_attack_target = tick.brain.has_memory_value(types::ATTACK_TARGET.id());
    tick.mob.get_mob_entity().set_attacking(has_attack_target);

    if !tick.brain.has_memory_value(types::RIDE_TARGET.id())
        && is_baby_riding_baby(tick.mob)
        && let Some(vehicle) = tick.mob.get_entity().get_vehicle()
    {
        vehicle
            .get_entity()
            .remove_passenger(tick.mob.get_entity().entity_id);
    }
    if !tick.brain.has_memory_value(types::CELEBRATE_LOCATION.id()) {
        tick.brain.erase(types::DANCING.id());
    }
    let dancing = tick.brain.has_memory_value(types::DANCING.id());
    if let Some(piglin) = tick.mob.cast_any().downcast_ref::<PiglinEntity>() {
        piglin.set_dancing(dancing);
    }
}

pub const MEMORY_TYPES: &[MemoryModuleId] = &[
    types::UNIVERSAL_ANGER.id(),
    types::ATE_RECENTLY.id(),
    types::SPEAR_FLEEING_TIME.id(),
    types::SPEAR_FLEEING_POSITION.id(),
    types::SPEAR_CHARGE_POSITION.id(),
    types::SPEAR_ENGAGE_TIME.id(),
];

pub const SENSOR_TYPES: &[SensorType] = &[
    SensorType::NearestLivingEntities,
    SensorType::NearestPlayers,
    SensorType::NearestItems,
    SensorType::HurtBy,
    SensorType::PiglinSpecific,
];

pub static PIGLIN_PROVIDER: BrainProvider = BrainProvider {
    memory_types: MEMORY_TYPES,
    sensor_types: SENSOR_TYPES,
    activities: |_| {
        vec![
            init_core_activity(),
            init_idle_activity(),
            init_admire_item_activity(),
            init_fight_activity(),
            init_celebrate_activity(),
            init_retreat_activity(),
            init_ride_hoglin_activity(),
        ]
    },
};
