use super::{Entity, EntityBase, NBTStorage, ai::pathfinder::Navigator, living::LivingEntity};
use crate::entity::EntityBaseFuture;
use crate::entity::ai::brain::Brain;
use crate::entity::ai::control::MoveControlTrait;
use crate::entity::ai::control::look_control::LookControl;
use crate::entity::ai::control::move_control::MoveControl;
use crate::entity::ai::goal::goal_selector::GoalSelector;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::MobCategory;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::java::client::play::{CHeadRot, CUpdateEntityRot, Metadata};
use pumpkin_util::Difficulty;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomGenerator, get_seed};
use rand::RngExt;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering};
use uuid::Uuid;

pub mod bat;
pub mod blaze;
pub mod breeze;
pub mod cave_spider;
pub mod creaking;
pub mod creeper;
pub mod elder_guardian;
pub mod enderman;
pub mod endermite;
pub mod equipment;
pub mod evoker;
pub mod ghast;
pub mod giant;
pub mod guardian;
pub mod hoglin;
pub mod hoglin_gore;
pub mod illusioner;
pub mod magma_cube;
pub mod phantom;
pub mod piglin;
pub mod piglin_brute;
pub mod piglin_shared;
pub mod pillager;
pub mod ravager;
pub mod shulker;
pub mod silverfish;
pub mod skeleton;
pub mod slime;
pub mod spider;
pub mod sulfur_cube;
pub mod vex;
pub mod vindicator;
pub mod warden;
pub mod warden_anger;
pub mod witch;
pub mod zoglin;
pub mod zombie;
pub mod zombified_piglin;

pub struct MobEntity {
    pub living_entity: LivingEntity,
    /// `Mob.brain` -- present only for mobs migrated to the Brain/Memory/Activity system
    /// (`crate::entity::ai::brain`). `None` for every Goal-driven mob, which is still the vast
    /// majority; vanilla likewise holds a `goalSelector` and a `brain` on the same `Mob` and
    /// lets each mob use whichever it needs. A Brain-having mob keeps its `goals_selector`, and
    /// the two are ticked independently and do not know about each other.
    pub brain: Option<Brain>,
    pub goals_selector: std::sync::Mutex<GoalSelector>,
    pub target_selector: std::sync::Mutex<GoalSelector>,
    pub navigator: std::sync::Mutex<Navigator>,
    pub target: tokio::sync::Mutex<Option<Arc<dyn EntityBase>>>,
    pub look_control: std::sync::Mutex<LookControl>,
    pub move_control: std::sync::Mutex<Box<dyn MoveControlTrait>>,
    pub position_target: AtomicCell<BlockPos>,
    pub position_target_range: AtomicI32,
    pub love_ticks: AtomicI32,
    pub breeding_cooldown: AtomicI32,
    /// Vanilla `Mob.noActionTime`, used by the random despawn check.
    pub no_action_time: AtomicI32,
    /// Vanilla `Entity.tickCount`, used by species-specific despawn rules.
    pub tick_count: AtomicI32,
    pub breeder: AtomicCell<Option<Uuid>>,
    pub owner: AtomicCell<Option<Uuid>>,
    pub ordered_to_sit: AtomicBool,
    mob_flags: AtomicU8,
    last_sent_yaw: AtomicU8,
    last_sent_pitch: AtomicU8,
    last_sent_head_yaw: AtomicU8,
}

/// Tick boundaries (both inclusive) when monsters do not burn in sunlight (26.1).
///
/// Sourced from `data/minecraft/timeline/day.json` — `monsters_burn` keyframes:
/// `value=false` at tick 12542 (dusk), `value=true` at tick 23460 (dawn).
///
/// TODO: Replace with `EnvironmentAttributes::MONSTERS_BURN` lookup once the
/// `EnvironmentAttributeSystem` is implemented in `pumpkin-data`.
const NIGHT_START: i64 = 12542;
const NIGHT_END: i64 = 23459;

impl MobEntity {
    const AI_DISABLED_FLAG: u8 = 1;
    const LEFT_HANDED_FLAG: u8 = 2;
    const ATTACKING_FLAG: u8 = 4;
    const CAN_PICK_UP_LOOT_FLAG: u8 = 8;

    #[must_use]
    pub fn new(entity: Entity) -> Self {
        Self {
            living_entity: LivingEntity::new(entity),
            brain: None,
            goals_selector: std::sync::Mutex::new(GoalSelector::default()),
            target_selector: std::sync::Mutex::new(GoalSelector::default()),
            navigator: std::sync::Mutex::new(Navigator::default()),
            target: tokio::sync::Mutex::new(None),
            look_control: std::sync::Mutex::new(LookControl::default()),
            move_control: std::sync::Mutex::new(Box::new(MoveControl::default())),
            position_target: AtomicCell::new(BlockPos::ZERO),
            position_target_range: AtomicI32::new(-1),
            love_ticks: AtomicI32::new(0),
            breeding_cooldown: AtomicI32::new(0),
            no_action_time: AtomicI32::new(0),
            tick_count: AtomicI32::new(0),
            breeder: AtomicCell::new(None),
            owner: AtomicCell::new(None),
            ordered_to_sit: AtomicBool::new(false),
            mob_flags: AtomicU8::new(0),
            last_sent_yaw: AtomicU8::new(0),
            last_sent_pitch: AtomicU8::new(0),
            last_sent_head_yaw: AtomicU8::new(0),
        }
    }

    pub fn is_in_position_target_range(&self) -> bool {
        self.is_in_position_target_range_pos(&self.living_entity.entity.block_pos.load())
    }

    pub fn is_in_position_target_range_pos(&self, block_pos: &BlockPos) -> bool {
        let position_target_range = self.position_target_range.load(Relaxed);
        if position_target_range == -1 {
            true
        } else {
            let target = self.position_target.load();
            let dx = f64::from(target.0.x) - f64::from(block_pos.0.x);
            let dy = f64::from(target.0.y) - f64::from(block_pos.0.y);
            let dz = f64::from(target.0.z) - f64::from(block_pos.0.z);
            // Java evaluates homeRadius * homeRadius as an int before comparing it to the
            // double distance, so preserve its two's-complement overflow behavior.
            let range_squared = position_target_range.wrapping_mul(position_target_range);
            dx.mul_add(dx, dy.mul_add(dy, dz * dz)) < f64::from(range_squared)
        }
    }

    pub fn set_attacking(&self, attacking: bool) {
        self.set_mob_flag(Self::ATTACKING_FLAG, attacking);
    }

    pub fn is_attacking(&self) -> bool {
        (self.mob_flags.load(Relaxed) & Self::ATTACKING_FLAG) != 0
    }

    pub fn set_left_handed(&self, left_handed: bool) {
        self.set_mob_flag(Self::LEFT_HANDED_FLAG, left_handed);
    }

    pub fn can_pick_up_loot(&self) -> bool {
        (self.mob_flags.load(Relaxed) & Self::CAN_PICK_UP_LOOT_FLAG) != 0
    }

    pub fn set_can_pick_up_loot(&self, value: bool) {
        self.set_mob_flag(Self::CAN_PICK_UP_LOOT_FLAG, value);
    }

    pub fn is_left_handed(&self) -> bool {
        (self.mob_flags.load(Relaxed) & Self::LEFT_HANDED_FLAG) != 0
    }

    pub fn set_persistence_required(&self) {
        self.living_entity
            .entity
            .persistence_required
            .store(true, Relaxed);
    }

    pub fn is_persistence_required(&self) -> bool {
        self.living_entity.entity.persistence_required.load(Relaxed)
    }

    pub fn set_no_ai(&self, no_ai: bool) {
        self.set_mob_flag(Self::AI_DISABLED_FLAG, no_ai);
    }

    pub fn is_no_ai(&self) -> bool {
        (self.mob_flags.load(Relaxed) & Self::AI_DISABLED_FLAG) != 0
    }

    pub async fn clear_ai_goals(&self, mob: &dyn Mob) {
        let running_goals = self.goals_selector.lock().unwrap().clear();
        for mut goal in running_goals {
            goal.goal.stop(mob).await;
        }

        let running_target_goals = self.target_selector.lock().unwrap().clear();
        for mut goal in running_target_goals {
            goal.goal.stop(mob).await;
        }
    }

    pub fn add_goal<G: crate::entity::ai::goal::Goal + 'static>(&self, priority: u8, goal: G) {
        self.goals_selector
            .lock()
            .unwrap()
            .add_goal(priority, Box::new(goal));
    }

    pub fn add_target_goal<G: crate::entity::ai::goal::Goal + 'static>(
        &self,
        priority: u8,
        goal: G,
    ) {
        self.target_selector
            .lock()
            .unwrap()
            .add_goal(priority, Box::new(goal));
    }

    pub async fn set_target(&self, target: Option<Arc<dyn EntityBase>>) {
        let mut t = self.target.lock().await;
        *t = target;
    }

    pub async fn get_target(&self) -> Option<Arc<dyn EntityBase>> {
        self.target.lock().await.clone()
    }

    fn set_mob_flag(&self, flag: u8, value: bool) {
        let old_b = self.mob_flags.load(Ordering::Relaxed);

        let new_b = if value { old_b | flag } else { old_b & !flag };

        if new_b != old_b {
            self.mob_flags.store(new_b, Ordering::Relaxed);

            self.living_entity.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::MOB_FLAGS_ID,
                    MetaDataType::BYTE,
                    new_b,
                )],
                None,
            );
        }
    }

    pub fn is_in_love(&self) -> bool {
        self.love_ticks.load(Relaxed) > 0
    }

    pub fn set_love_ticks(&self, ticks: i32, breeder: Option<Uuid>) {
        self.love_ticks.store(ticks, Relaxed);
        self.breeder.store(breeder);
    }

    pub fn reset_love_ticks(&self) {
        self.love_ticks.store(0, Relaxed);
    }

    pub fn try_claim_love(&self) -> bool {
        self.love_ticks
            .fetch_update(Relaxed, Relaxed, |ticks| (ticks > 0).then_some(0))
            .is_ok()
    }

    pub fn is_tamed(&self) -> bool {
        self.owner.load().is_some()
    }

    pub fn set_owner(&self, owner: Uuid) {
        self.owner.store(Some(owner));
    }

    pub fn is_ordered_to_sit(&self) -> bool {
        self.ordered_to_sit.load(Relaxed)
    }

    pub fn set_ordered_to_sit(&self, value: bool) {
        self.ordered_to_sit.store(value, Relaxed);
    }

    pub fn is_breeding_ready(&self) -> bool {
        self.living_entity.entity.age.load(Relaxed) >= 0
            && self.breeding_cooldown.load(Relaxed) <= 0
    }

    pub async fn is_in_attack_range(&self, target: &dyn EntityBase) -> bool {
        const DEFAULT_ATTACK_RANGE: f64 = 0.828_427_12; // sqrt(2.04) - 0.6

        let held_item = self
            .living_entity
            .held_item(&self.living_entity.entity)
            .await;
        let held_item = held_item.lock().await;
        let (max_range, min_range) = held_item
            .get_data_component::<pumpkin_data::data_component_impl::AttackRangeImpl>()
            .map_or((DEFAULT_ATTACK_RANGE, 0.0), |attack_range| {
                (
                    f64::from(attack_range.max_reach * attack_range.mob_factor),
                    f64::from(attack_range.min_reach * attack_range.mob_factor),
                )
            });

        let target_hitbox = target.get_entity().bounding_box.load();

        if !self
            .get_attack_box(max_range)
            .await
            .intersects(&target_hitbox)
        {
            return false;
        }

        min_range <= 0.0
            || !self
                .get_attack_box(min_range)
                .await
                .intersects(&target_hitbox)
    }

    pub fn is_dark_enough_to_spawn(world: &World, pos: &BlockPos, is_thundering: bool) -> bool {
        let sky_light = world.get_sky_light_level(pos);
        if sky_light > rand::random_range(0..32) {
            return false;
        }

        let dimension = &world.dimension;
        let block_light_limit = dimension.monster_spawn_block_light_limit;

        let block_light = world.get_block_light_level(pos).unwrap_or(0);
        if block_light_limit < 15 && block_light > block_light_limit {
            return false;
        }

        let current_brightness = if is_thundering {
            world.get_raw_brightness(pos, 10)
        } else {
            world.get_max_local_raw_brightness(pos)
        };

        // TODO
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        current_brightness <= dimension.monster_spawn_light_level.get(&mut random) as u8
    }

    pub fn check_monster_spawn_rules(world: &World, pos: &BlockPos, is_thundering: bool) -> bool {
        if world.level_info.load().difficulty == Difficulty::Peaceful {
            return false;
        }

        if !Self::is_dark_enough_to_spawn(world, pos, is_thundering) {
            return false;
        }

        //TODO:check_mob_spawn_rules(entity_type, world, spawn_reason, pos).await
        true
    }

    pub fn check_any_light_monster_spawn_rules(_world: &World, _pos: &BlockPos) -> bool {
        // Vanilla delegates this predicate to Mob.checkMobSpawnRules. The
        // natural-spawn caller has already run is_spawn_position_ok, which is
        // Pumpkin's equivalent of that block-state predicate.
        true
    }

    pub async fn try_attack(&self, target: &dyn EntityBase) -> bool {
        if self.living_entity.dead.load(Relaxed) {
            return false;
        }

        let mut attack_damage = self
            .living_entity
            .get_attribute_value(&Attributes::ATTACK_DAMAGE);
        let mut fire_aspect_level = 0u32;
        let mut knockback_level = 0u32;
        let held_item = self
            .living_entity
            .held_item(&self.living_entity.entity)
            .await;
        let held_item = held_item.lock().await;
        if let Some(enchantments) =
            held_item.get_data_component::<pumpkin_data::data_component_impl::EnchantmentsImpl>()
        {
            let target_type = target.get_entity().entity_type;
            for (enchantment, level) in enchantments.enchantment.iter() {
                for effect in crate::enchantment::effects_for(enchantment) {
                    match effect {
                        crate::enchantment::EnchantmentEffect::Damage(condition, value)
                            if condition.applies(target_type) =>
                        {
                            attack_damage += f64::from(value.calculate(*level));
                        }
                        crate::enchantment::EnchantmentEffect::IgniteOnHit(value) => {
                            fire_aspect_level = (value.calculate(*level) * 20.0) as u32 / 80;
                        }
                        crate::enchantment::EnchantmentEffect::Knockback(condition, value)
                            if *condition == crate::enchantment::KnockbackCondition::Always =>
                        {
                            knockback_level = value.calculate(*level).max(0.0) as u32;
                        }
                        _ => {}
                    }
                }
            }
        }
        drop(held_item);

        let caller = self
            .living_entity
            .entity
            .world
            .load()
            .get_entity_by_id(self.living_entity.entity.entity_id);

        let damaged = target
            .damage_with_context(
                target,
                attack_damage as f32,
                DamageType::MOB_ATTACK,
                Some(self.living_entity.entity.pos.load()),
                caller.as_deref(),
                caller.as_deref(),
            )
            .await;

        if damaged {
            if fire_aspect_level != 0 {
                target
                    .get_entity()
                    .set_on_fire_for_ticks(fire_aspect_ticks(fire_aspect_level as i32));
            }
            if knockback_level != 0 {
                let yaw = self.living_entity.entity.yaw.load().to_radians();
                let strength = knockback_enchantment_strength(knockback_level);
                let x = f64::from(yaw.sin());
                let z = f64::from(-yaw.cos());
                if let Some(living) = target.get_living_entity() {
                    living.knockback_with_resistance(strength, x, z);
                } else {
                    target.get_entity().knockback(strength, x, z);
                }
            }
            self.living_entity
                .last_attacking_id
                .store(target.get_entity().entity_id, Relaxed);
            self.living_entity
                .last_attack_time
                .store(self.living_entity.entity.age.load(Relaxed), Relaxed);
        }

        damaged
    }

    async fn get_attack_box(&self, attack_range: f64) -> BoundingBox {
        let vehicle_lock = self.living_entity.entity.vehicle.lock().await;

        let base_box = vehicle_lock.as_ref().map_or_else(
            || self.living_entity.entity.bounding_box.load(),
            |vehicle| {
                let vehicle_box = vehicle.get_entity().bounding_box.load();
                let my_box = self.living_entity.entity.bounding_box.load();

                BoundingBox {
                    min: Vector3::new(
                        my_box.min.x.min(vehicle_box.min.x),
                        my_box.min.y,
                        my_box.min.z.min(vehicle_box.min.z),
                    ),
                    max: Vector3::new(
                        my_box.max.x.max(vehicle_box.max.x),
                        my_box.max.y,
                        my_box.max.z.max(vehicle_box.max.z),
                    ),
                }
            },
        );

        base_box.expand(attack_range, 0.0, attack_range)
    }

    pub async fn tick_sun_burn(&self) {
        if !self
            .living_entity
            .entity
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_BURN_IN_DAYLIGHT)
        {
            return;
        }
        if !self.is_sun_burn_tick().await {
            return;
        }
        self.apply_sun_burn();
    }

    async fn is_sun_burn_tick(&self) -> bool {
        let entity = &self.living_entity.entity;

        let world_arc = entity.world.load();
        let world = world_arc.as_ref();

        // Night boundary from data/minecraft/timeline/day.json — monsters_burn keyframes:
        // value=false at tick 12542 (dusk), value=true at tick 23460 (dawn).
        // TODO: read directly from EnvironmentAttributes::MONSTERS_BURN once implemented.

        let day_time = world.get_time_of_day().await % 24000;
        if (NIGHT_START..=NIGHT_END).contains(&day_time) {
            return false;
        }

        // Vanilla: getLightLevelDependentMagicValue() — sky light at eye pos, scaled 0–1.
        let eye_block_pos = entity.get_eye_pos();
        let brightness = world
            .level
            .light_engine
            .get_sky_light_level(&world.level, &eye_block_pos.to_block_pos())
            as f32
            / 15.0;

        if brightness <= 0.5 {
            return false;
        }

        let is_in_non_burnable = entity.touching_water.load(Relaxed)
            || world.weather.lock().await.raining
            || entity.is_in_powder_snow()
            || entity.was_in_powder_snow.load(Relaxed);

        if is_in_non_burnable {
            return false;
        }

        let pos = entity.pos.load();
        let top_y = world.get_top_block(Vector2::new(pos.x as i32, pos.z as i32));
        if (entity.get_eye_y() as i32) < top_y {
            return false;
        }

        let mut rng = rand::rng();
        rng.random::<f32>() * 30.0 < (brightness - 0.4) * 2.0
    }

    fn apply_sun_burn(&self) {
        let entity = &self.living_entity.entity;
        entity.set_on_fire_for(8.0);
    }

    pub async fn mob_interact(
        &self,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        can_be_leashed: bool,
    ) -> bool {
        let entity = &self.living_entity.entity;

        // If already leashed to player, right-clicking unleashes the mob
        let currently_leashed = {
            let guard = entity.leashed_to.lock().await;
            guard.is_some()
        };

        if currently_leashed {
            entity.unleash().await;
            let lead_item =
                pumpkin_data::item_stack::ItemStack::new(1, &pumpkin_data::item::Item::LEAD);
            entity
                .world
                .load()
                .drop_stack(&entity.block_pos.load(), lead_item)
                .await;
            return true;
        }

        // If holding a lead, leash the mob to the player
        if can_be_leashed
            && (item_stack.item.registry_key == "lead"
                || item_stack.item.registry_key == "minecraft:lead")
        {
            let diff = entity.pos.load() - player.get_entity().pos.load();
            let dist_sq = diff.length_squared();
            if dist_sq <= Entity::LEASH_SNAP_DISTANCE * Entity::LEASH_SNAP_DISTANCE {
                entity.leash_to(player.clone() as Arc<dyn EntityBase>).await;
                if player.gamemode.load() != pumpkin_util::GameMode::Creative {
                    item_stack.decrement(1);
                }
                return true;
            }
        }

        false
    }
}

pub trait Mob: EntityBase + Send + Sync {
    /// Vanilla `Mob` entities are pickable unless a concrete entity overrides it.
    fn is_pickable(&self) -> bool {
        self.get_entity().is_alive()
    }

    fn get_random(&self) -> rand::rngs::ThreadRng {
        rand::rng()
    }

    fn get_max_look_yaw_change(&self) -> f32 {
        10.0
    }

    fn get_max_look_pitch_change(&self) -> f32 {
        40.0
    }

    fn get_max_head_rotation(&self) -> f32 {
        75.0
    }

    fn get_mob_entity(&self) -> &MobEntity;

    /// Vanilla `Mob.setPersistenceRequired`.
    fn set_persistence_required(&self) {
        self.get_mob_entity().set_persistence_required();
    }

    /// Vanilla `Mob.isPersistenceRequired`.
    fn is_persistence_required(&self) -> bool {
        self.get_mob_entity().is_persistence_required()
    }

    /// Vanilla `Mob.removeWhenFarAway`, including the current species
    /// overrides whose state is represented by Pumpkin.
    fn remove_when_far_away(&self, _dist_sqr: f64) -> bool {
        let category = self.get_entity().entity_type.category;
        let mob_entity = self.get_mob_entity();
        match self.get_entity().entity_type.id {
            // Cat.java: untamed cats become removable after 120 seconds.
            id if id == pumpkin_data::entity::EntityType::CAT.id => {
                !mob_entity.is_tamed() && mob_entity.tick_count.load(Relaxed) > 2400
            }
            // Ocelot.java: Pumpkin has no trust state yet, so its spawned ocelots
            // follow the vanilla untamed branch.
            id if id == pumpkin_data::entity::EntityType::OCELOT.id => {
                mob_entity.tick_count.load(Relaxed) > 2400
            }
            // AbstractFish.java: tadpoles are bucketable fish despite their
            // CREATURE category.
            id if id == pumpkin_data::entity::EntityType::TADPOLE.id => true,
            // AbstractFish and Axolotl override Animal's non-despawning default.
            // Bucketed/named variants are made persistent by their interaction/NBT paths.
            id if id == pumpkin_data::entity::EntityType::AXOLOTL.id
                || id == pumpkin_data::entity::EntityType::COD.id
                || id == pumpkin_data::entity::EntityType::NAUTILUS.id
                || id == pumpkin_data::entity::EntityType::PUFFERFISH.id
                || id == pumpkin_data::entity::EntityType::SALMON.id
                || id == pumpkin_data::entity::EntityType::TROPICAL_FISH.id
                || id == pumpkin_data::entity::EntityType::ZOMBIE_HORSE.id =>
            {
                true
            }
            // Animal and non-despawning MISC mob implementations in the
            // generated registry use the persistent far-away behavior.
            _ if category == &MobCategory::CREATURE || category == &MobCategory::MISC => false,
            _ => true,
        }
    }

    /// Vanilla `Mob.requiresCustomPersistence`: passengers and leashed mobs
    /// must not be removed by the normal despawn checks.
    fn requires_custom_persistence(&self) -> EntityBaseFuture<'_, bool> {
        Box::pin(async move {
            let entity = self.get_entity();
            entity.vehicle.lock().await.is_some() || entity.leashed_to.lock().await.is_some()
        })
    }

    /// Vanilla `Mob.checkDespawn`, called by the server entity tick loop.
    fn check_despawn(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let mob_entity = self.get_mob_entity();
            let entity = self.get_entity();
            if entity.is_removed() {
                return;
            }
            let world = entity.world.load();

            if world.level_info.load().difficulty == Difficulty::Peaceful
                && !entity.entity_type.category.is_friendly
            {
                entity.remove().await;
                return;
            }

            if self.is_persistence_required() || self.requires_custom_persistence().await {
                mob_entity.no_action_time.store(0, Relaxed);
                return;
            }

            let position = entity.pos.load();
            let nearest_player_distance = world
                .players
                .load()
                .iter()
                .map(|player| {
                    player
                        .get_entity()
                        .pos
                        .load()
                        .squared_distance_to_vec(&position)
                })
                .min_by(f64::total_cmp);

            let Some(distance_sqr) = nearest_player_distance else {
                return;
            };

            let despawn_distance = f64::from(entity.entity_type.category.despawn_distance);
            if distance_sqr > despawn_distance * despawn_distance
                && self.remove_when_far_away(distance_sqr)
            {
                entity.remove().await;
                return;
            }

            let no_despawn_distance = f64::from(MobCategory::NO_DESPAWN_DISTANCE);
            let no_despawn_distance_sqr = no_despawn_distance * no_despawn_distance;
            let no_action_time = mob_entity.no_action_time.load(Relaxed);
            if no_action_time > 600
                && rand::random_range(0..800) == 0
                && distance_sqr > no_despawn_distance_sqr
                && self.remove_when_far_away(distance_sqr)
            {
                entity.remove().await;
            } else if distance_sqr < no_despawn_distance_sqr {
                mob_entity.no_action_time.store(0, Relaxed);
            }
        })
    }

    /// `Raider.canBeLeader` default (all raiders except `Ravager`, which overrides to `false`).
    fn can_be_raid_leader(&self) -> bool {
        true
    }

    /// `Raider.applyRaidBuffs` default no-op. Raid-participant mobs (Vindicator, Pillager,
    /// Witch, Evoker, Illusioner) override this to enchant gear or grant potion effects;
    /// those overrides are separate work and are not implemented here.
    fn apply_raid_buffs(&self, _wave: i32, _is_captain: bool) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {})
    }

    /// Vanilla `LivingEntity.blockedByItem`: called on the attacker (`self`) when `defender`
    /// successfully shield-blocks one of `self`'s attacks. Default no-op; Ravager overrides this
    /// to sometimes stun itself, Hoglin/Zoglin have their own vanilla overrides not yet ported.
    fn blocked_by_item<'a>(
        &'a self,
        _defender: &'a dyn EntityBase,
        _damage: f32,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {})
    }

    /// Vanilla `CrossbowAttackMob.setChargingCrossbow` (synced data, drives `getArmPose`'s
    /// `CROSSBOW_CHARGE` state client-side). Default no-op; crossbow-wielding mobs (Pillager)
    /// override this to store and broadcast the flag.
    fn set_charging_crossbow(&self, _charging: bool) {}

    fn try_attack<'a>(&'a self, target: &'a dyn EntityBase) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let damaged = self.get_mob_entity().try_attack(target).await;
            if damaged {
                self.on_successful_attack(target).await;
            }
            damaged
        })
    }

    fn on_successful_attack<'a>(&'a self, _target: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {})
    }

    fn get_job_site(&self) -> Option<BlockPos> {
        None
    }

    fn get_home(&self) -> Option<BlockPos> {
        None
    }

    fn get_meeting_point(&self) -> Option<BlockPos> {
        None
    }

    fn get_path_aware_entity(&self) -> Option<&dyn PathAwareEntity> {
        None
    }

    /// Per-mob tick hook called each tick before AI runs. Override for mob-specific logic.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {})
    }

    /// Hook for mobs whose vanilla `travel` implementation replaces the generic living-mob
    /// movement path (for example `Squid.travel`, which moves with its current movement vector).
    /// Returning `true` means the hook has already moved the entity for this tick.
    fn custom_travel<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, bool> {
        Box::pin(async { false })
    }

    /// Vanilla custom-travel goals can publish a movement vector without taking over the
    /// navigator. The default is inert for ordinary mobs.
    fn set_movement_vector(&self, _movement: Vector3<f64>) {}

    #[must_use]
    fn get_movement_vector(&self) -> Option<Vector3<f64>> {
        None
    }

    fn post_tick(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {})
    }

    /// Called before damage is applied. Return `false` to cancel the damage entirely.
    /// Used by endermen to dodge projectiles via teleportation.
    fn pre_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        _source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async { true })
    }

    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        _source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {})
    }

    /// Called on the killed mob once its death is confirmed, with `cause` as the
    /// killer entity (mirrors `LivingEntity::on_death`'s `cause` parameter). Used by
    /// villagers to notify nearby witnesses of a murder.
    fn on_mob_death<'a>(&'a self, _cause: Option<&'a dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {})
    }

    fn on_eating_grass(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {})
    }

    fn modify_incoming_damage(&self, amount: f32, _damage_type: DamageType) -> f32 {
        amount
    }

    fn can_attack_with_owner(&self, _target: &dyn EntityBase, _owner: &dyn EntityBase) -> bool {
        true
    }

    /// Vanilla `Mob.canAttack`, consulted by `TargetingConditions.test`'s combat branch
    /// (`TargetingConditions.java:78`). Defaults to `true`; species with a blanket "never
    /// target this" rule (Iron Golem's player-created and creeper exclusions) override it.
    ///
    /// Consulted at initial acquisition by the active, hostile, witch, ghast, and non-tame target
    /// goals, and at continuation by `RevengeGoal` and `TrackTargetGoal`.
    fn can_attack(&self, _target: &Entity) -> bool {
        true
    }

    /// Exposes this mob's `NeutralMob`-equivalent grudge state (Wolf, `ZombifiedPiglin`),
    /// if it has one, for shared goals (`ActiveTargetGoal`'s angry-at-player predicate).
    fn persistent_anger(&self) -> Option<&crate::entity::persistent_anger::PersistentAnger> {
        None
    }

    fn get_mob_gravity(&self) -> f64 {
        self.get_mob_entity().living_entity.get_gravity()
    }

    fn get_mob_y_velocity_drag(&self) -> Option<f64> {
        None
    }

    /// Vanilla `Entity.isPushedByFluid`: whether currents apply push velocity to this mob.
    /// Turtle overrides this to `false`.
    fn mob_is_pushed_by_fluids(&self) -> bool {
        true
    }

    /// Set or clear the mob's target. Override to add side effects when targeting changes.
    fn set_mob_target(&self, target: Option<Arc<dyn EntityBase>>) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let mut mob_target = self.get_mob_entity().target.lock().await;
            *mob_target = target;
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            self.get_mob_entity()
                .mob_interact(player, item_stack, self.can_be_leashed())
                .await
        })
    }

    /// Vanilla `Mob.canBeLeashed`: whether a lead can be attached to this mob at all.
    /// Defaults to `true`; species that are never leashable (e.g. Turtle) override this.
    fn can_be_leashed(&self) -> bool {
        true
    }

    fn mob_player_collision<'a>(&'a self, _player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {})
    }

    /// Vanilla `Mob.wantsToPickUp` default: delegates to `canHoldItem`, which defaults to
    /// `true`. Whether picking up is ever attempted at all is gated separately by
    /// `can_pick_up_loot`.
    fn wants_to_pick_up_item(&self, _world: &World, _stack: &ItemStack) -> bool {
        true
    }

    /// Vanilla `Mob.canPickUpLoot`: whether this mob is allowed to pick up dropped items at
    /// all. Backed by the mob's `CanPickUpLoot` tracked-data flag, which defaults to `false`
    /// and is set at spawn time for a few mob types (see `equipment.rs`).
    fn can_pick_up_loot(&self) -> bool {
        self.get_mob_entity().can_pick_up_loot()
    }

    /// Vanilla `Mob.onItemPickup`/`equipItemIfPossible`: called once a candidate item stack
    /// has passed `wants_to_pick_up_item`, to actually take it. Returns the number of items
    /// taken from the stack; the caller only shrinks/removes the `ItemEntity` by that count.
    /// Default takes nothing, so no `ItemEntity` is ever touched unless a mob overrides this.
    fn on_item_pickup(&self, _stack: &ItemStack) -> u8 {
        0
    }

    /// Vanilla `Mob.aiStep`'s pickup-loot loop: scans nearby dropped items within pickup
    /// reach and offers each one to `on_item_pickup` if it passes `wants_to_pick_up_item`
    /// and the item entity's own pickup-delay gate. Gated on `can_pick_up_loot` and the
    /// `mobGriefing` gamerule.
    fn mob_try_pick_up_items(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            if !self.can_pick_up_loot() {
                return;
            }

            let mob_entity = self.get_mob_entity();
            let entity = &mob_entity.living_entity.entity;
            if !entity.is_alive() {
                return;
            }

            let world = entity.world.load();
            if !world.level_info.load().game_rules.mob_griefing {
                return;
            }

            let reach = entity.bounding_box.load().expand(1.0, 0.0, 1.0);
            for candidate in world.get_entities_at_box(&reach) {
                let Some(item_entity) = candidate.clone().get_item_entity() else {
                    continue;
                };

                if !item_entity.get_entity().is_alive() || item_entity.has_pickup_delay() {
                    continue;
                }

                let stack_snapshot = { item_entity.get_item_stack().lock().await.clone() };
                if stack_snapshot.is_empty() || !self.wants_to_pick_up_item(&world, &stack_snapshot)
                {
                    continue;
                }

                let taken = self
                    .on_item_pickup(&stack_snapshot)
                    .min(stack_snapshot.item_count);
                if taken == 0 {
                    continue;
                }

                self.set_persistence_required();

                let is_empty = {
                    let mut stack = item_entity.get_item_stack().lock().await;
                    stack.decrement(taken);
                    stack.is_empty()
                };

                if is_empty {
                    item_entity.get_entity().remove().await;
                } else {
                    item_entity.init_data_tracker().await;
                }
            }
        })
    }

    fn get_owner_uuid(&self) -> Option<Uuid> {
        self.get_mob_entity().owner.load()
    }

    fn is_sitting(&self) -> bool {
        self.get_mob_entity().is_ordered_to_sit()
    }

    fn get_base_experience_reward(&self) -> u32 {
        self.get_entity().entity_type.experience_reward
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let is_baby = entity.age.load(std::sync::atomic::Ordering::Relaxed) < 0;
            if is_baby {
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
        })
    }

    fn mob_set_variant_name(&self, _name: &str) {}

    /// Vanilla `Animal.getBreedOffspring`: builds the baby entity to spawn after a successful
    /// breed with `mate`. Override to customize the offspring (e.g. inherited color/variant)
    /// before it enters the world. Returning `None` skips spawning a baby entity entirely,
    /// matching `Sniffer`'s override, which drops a `SNIFFER_EGG` item instead.
    fn create_offspring<'a>(
        &'a self,
        _mate: &'a dyn EntityBase,
        world: &'a Arc<World>,
    ) -> EntityBaseFuture<'a, Option<Arc<dyn EntityBase>>> {
        Box::pin(async move {
            let entity = self.get_entity();
            Some(crate::entity::r#type::from_type(
                entity.entity_type,
                entity.pos.load(),
                world,
                Uuid::new_v4(),
            ))
        })
    }

    /// Called once a breed has been claimed (both parents' love ticks reset) and offspring is
    /// about to be created. Override for side effects vanilla ties to a specific `BreedGoal`
    /// subclass rather than the generic breed path, e.g. `Turtle.TurtleBreedGoal.breed` setting
    /// `hasEgg = true` (`Turtle.java:300-326`).
    fn on_bred(&self, _mate: &dyn EntityBase) {}
}
impl<T: Mob + Send + 'static> EntityBase for T {
    fn get_mob(&self) -> Option<&dyn Mob> {
        Some(self)
    }

    fn check_despawn(&self) -> EntityBaseFuture<'_, ()> {
        Mob::check_despawn(self)
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.mob_init_data_tracker().await;
            let world = self.get_mob_entity().living_entity.entity.world.load();
            crate::entity::mob::equipment::equip_mob_on_spawn(self as &dyn EntityBase, &world)
                .await;

            let entity_name = self.get_entity().entity_type.resource_name;
            if let Some(def) = crate::entity::mob::equipment::EQUIPMENT_REGISTRY.get(entity_name)
                && def.can_pick_up_loot
            {
                let difficulty = crate::entity::mob::equipment::RegionalDifficulty::at(
                    &world,
                    self.get_entity().pos.load(),
                );
                let pickup_chance = 0.55 * difficulty.special_multiplier;
                self.get_mob_entity()
                    .set_can_pick_up_loot(rand::random::<f32>() < pickup_chance);
            }
        })
    }

    fn set_variant_name(&self, name: &str) {
        self.mob_set_variant_name(name);
    }

    #[allow(clippy::too_many_lines)]
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let mob_entity = self.get_mob_entity();
            mob_entity.tick_count.fetch_add(1, Relaxed);
            if !mob_entity.is_no_ai() {
                mob_entity.no_action_time.fetch_add(1, Relaxed);
            }
            mob_entity.living_entity.entity.tick_leash().await;
            mob_entity.tick_sun_burn().await;

            if mob_entity.breeding_cooldown.load(Relaxed) > 0 {
                mob_entity.breeding_cooldown.fetch_sub(1, Relaxed);
            }

            if mob_entity.love_ticks.load(Relaxed) > 0 {
                let ticks = mob_entity.love_ticks.fetch_sub(1, Relaxed);
                if ticks % 10 == 0 {
                    let entity = &mob_entity.living_entity.entity;
                    let pos = entity.pos.load();
                    let world = entity.world.load();
                    world.spawn_particle(
                        pos + Vector3::new(0.0, f64::from(entity.height()) + 0.5, 0.0),
                        Vector3::new(0.5, 0.5, 0.5),
                        1.0,
                        1,
                        pumpkin_data::particle::Particle::Heart,
                    );
                }
            }

            // Vanilla `LivingEntity.isEffectiveAi()` suppresses the complete server AI step
            // for mobs carrying the `NoAI` flag. Physics, leash updates, ageing, living effects,
            // and Mob.aiStep item pickup still run; only goals, brains, navigation, and
            // controllers belong to the skipped AI step.
            if !mob_entity.is_no_ai() {
                self.mob_tick(caller).await;

                // 1. "Take" selectors out of the mutexes
                let mut target_selector = {
                    let mut guard = mob_entity.target_selector.lock().unwrap();
                    std::mem::take(&mut *guard)
                };
                let mut goals_selector = {
                    let mut guard = mob_entity.goals_selector.lock().unwrap();
                    std::mem::take(&mut *guard)
                };

                // 2. Perform AI logic (No locks held, so .await is safe!). Vanilla only
                // reevaluates selector membership on alternating ticks; on the other ticks it
                // advances already-running goals that explicitly require every-tick updates.
                // The phase is entity-specific so a mob swarm does not perform all expensive
                // selector work on the same tick.
                let tick_count = mob_entity.tick_count.load(Relaxed);
                let run_all_goals = tick_count <= 1
                    || (tick_count.wrapping_add(mob_entity.living_entity.entity.entity_id)) % 2
                        == 0;
                if run_all_goals {
                    target_selector.tick(self).await;
                    goals_selector.tick(self).await;
                } else {
                    target_selector.tick_goals(self, false).await;
                    goals_selector.tick_goals(self, false).await;
                }

                // 3. "Put back" selectors
                {
                    *mob_entity.target_selector.lock().unwrap() = target_selector;
                    *mob_entity.goals_selector.lock().unwrap() = goals_selector;
                };

                // 3b. Brain tick, for mobs migrated to the Brain/Memory/Activity system.
                //
                // Placement matters: this must run *after* the goal selectors are back in their
                // mutexes and *before* the navigator is taken out, so that a `WALK_TARGET` written
                // this tick reaches `Navigator::set_progress` and is consumed by the navigator in
                // the same tick, matching vanilla where `Brain.tick` runs inside
                // `Mob.customServerAiStep` ahead of navigation.
                //
                // `Brain::tick` takes only its own runtime out of its mutex; the memory store stays
                // live so damage/game-event writes arriving mid-tick are not dropped.
                if let Some(brain) = &mob_entity.brain {
                    // Vanilla passes `level.getGameTime()`, which is shared by every entity in
                    // the dimension and persists across entity reloads.
                    let game_time = mob_entity
                        .living_entity
                        .entity
                        .world
                        .load_full()
                        .get_world_age()
                        .await;
                    brain.tick(self, game_time).await;
                }

                // 4. Repeat for Navigator
                let mut navigator = {
                    let mut guard = mob_entity.navigator.lock().unwrap();
                    std::mem::take(&mut *guard)
                };

                navigator.tick(&mob_entity.living_entity).await;

                {
                    *mob_entity.navigator.lock().unwrap() = navigator;
                };

                // Controllers are synchronous, so we can just use normal blocks
                {
                    let mut look_control = mob_entity.look_control.lock().unwrap();
                    look_control.tick(self);
                };

                {
                    let mut move_control = mob_entity.move_control.lock().unwrap();
                    // Vanilla PathNavigation never writes movement input directly: after following
                    // its path it calls MoveControl.setWantedPosition. The waypoint was computed
                    // while the navigator was taken out above, so hand it to the controller only
                    // after restoring the navigator lock. This is especially important for
                    // FlyingMoveControl, whose vertical input and no-gravity state otherwise never
                    // receive path targets.
                    let navigation_target =
                        mob_entity.navigator.lock().unwrap().next_movement_target();
                    if let Some((target, speed)) = navigation_target {
                        move_control.set_wanted_position(target.x, target.y, target.z, speed);
                    }
                    move_control.tick(self);
                }
            }

            mob_entity.living_entity.tick(caller, server).await;
            self.mob_try_pick_up_items().await;
            self.post_tick().await;

            // --- Packet logic remains the same ---
            let entity = &mob_entity.living_entity.entity;
            let yaw = (entity.yaw.load() * 256.0 / 360.0).rem_euclid(256.0) as u8;
            let pitch = (entity.pitch.load() * 256.0 / 360.0).rem_euclid(256.0) as u8;
            let head_yaw = (entity.head_yaw.load() * 256.0 / 360.0).rem_euclid(256.0) as u8;

            let last_yaw = mob_entity.last_sent_yaw.load(Relaxed);
            let last_pitch = mob_entity.last_sent_pitch.load(Relaxed);
            let last_head_yaw = mob_entity.last_sent_head_yaw.load(Relaxed);

            let chunk_pos = entity.chunk_pos.load();
            if yaw.abs_diff(last_yaw) >= 1 || pitch.abs_diff(last_pitch) >= 1 {
                let world = entity.world.load();
                world.broadcast_to_chunk(
                    chunk_pos,
                    &CUpdateEntityRot::new(
                        entity.entity_id.into(),
                        yaw,
                        pitch,
                        entity.on_ground.load(Relaxed),
                    ),
                );
                mob_entity.last_sent_yaw.store(yaw, Relaxed);
                mob_entity.last_sent_pitch.store(pitch, Relaxed);
            }

            if head_yaw.abs_diff(last_head_yaw) >= 1 {
                let world = entity.world.load();

                world.broadcast_to_chunk(
                    chunk_pos,
                    &CHeadRot::new(entity.entity_id.into(), head_yaw),
                );
                mob_entity.last_sent_head_yaw.store(head_yaw, Relaxed);
            }
        })
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn can_hit(&self) -> bool {
        true
    }

    fn damage_with_context<'a>(
        &'a self,
        caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            // pre_damage hook: allows mobs to dodge/cancel damage (e.g. enderman projectile dodge)
            if !self.pre_damage(damage_type, source).await {
                return false;
            }
            // Mob-specific damage modifier (e.g. shulker armor when closed).
            let amount = self.modify_incoming_damage(amount, damage_type);
            let damaged = self
                .get_mob_entity()
                .living_entity
                .damage_with_context(caller, amount, damage_type, position, source, cause)
                .await;
            if damaged {
                self.on_damage(damage_type, source).await;
            }
            damaged
        })
    }

    fn interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move { self.mob_interact(player, item_stack).await })
    }

    fn on_player_collision<'a>(&'a self, player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move { self.mob_player_collision(player).await })
    }

    fn get_entity(&self) -> &Entity {
        &self.get_mob_entity().living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.get_mob_entity().living_entity)
    }

    fn is_pickable(&self) -> bool {
        Mob::is_pickable(self)
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_in_love(&self) -> bool {
        self.get_mob_entity().is_in_love()
    }

    fn is_breeding_ready(&self) -> bool {
        self.get_mob_entity().is_breeding_ready()
    }

    fn reset_love(&self) {
        self.get_mob_entity().reset_love_ticks();
    }

    fn try_claim_love(&self) -> bool {
        self.get_mob_entity().try_claim_love()
    }

    fn set_breeding_cooldown(&self, ticks: i32) {
        self.get_mob_entity()
            .breeding_cooldown
            .store(ticks, Relaxed);
    }

    fn is_panicking(&self) -> bool {
        // `PathfinderMob.isPanicking` (`PathfinderMob.java:37-48`) first checks the Brain
        // memory and then the currently running goals. Pumpkin's concrete mobs are represented
        // through `Mob` rather than a separate Java-style PathfinderMob subtype, so query both
        // sources here instead of relying on the currently-unused optional PathAwareEntity
        // adapter.
        if self.get_mob_entity().brain.as_ref().is_some_and(|brain| {
            brain.has_value::<crate::entity::ai::brain::memory::IsPanickingMemory>()
        }) {
            return true;
        }

        self.get_mob_entity()
            .goals_selector
            .lock()
            .unwrap()
            .is_panic_running()
    }

    fn get_job_site_pos(&self) -> Option<pumpkin_util::math::position::BlockPos> {
        <T as Mob>::get_job_site(self)
    }

    fn get_home_pos(&self) -> Option<pumpkin_util::math::position::BlockPos> {
        <T as Mob>::get_home(self)
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn get_gravity(&self) -> f64 {
        self.get_mob_gravity()
    }

    fn get_y_velocity_drag(&self) -> Option<f64> {
        self.get_mob_y_velocity_drag()
    }

    fn is_pushed_by_fluids(&self) -> bool {
        self.mob_is_pushed_by_fluids()
    }

    fn get_experience_reward(&self, _killer: Option<&dyn EntityBase>) -> u32 {
        if self
            .get_entity()
            .age
            .load(std::sync::atomic::Ordering::Relaxed)
            < 0
        {
            return 0;
        }
        // TODO: apply enchantment processing like in vanilla
        Mob::get_base_experience_reward(self)
    }

    fn get_base_experience_reward(&self) -> u32 {
        Mob::get_base_experience_reward(self)
    }
}

#[expect(dead_code)]
const DEFAULT_PATHFINDING_FAVOR: f32 = 0.0;

const fn fire_aspect_ticks(level: i32) -> u32 {
    if level > 0 { level as u32 * 80 } else { 0 }
}

const fn knockback_enchantment_strength(level: u32) -> f64 {
    level as f64 * 0.5
}

pub trait PathAwareEntity: Mob + Send + Sync {
    fn get_pathfinding_favor(&self, _block_pos: BlockPos, _world: Arc<World>) -> f32 {
        0.0
    }

    // TODO: missing SpawnReason attribute
    fn can_spawn(&self, world: Arc<World>) -> bool {
        self.get_pathfinding_favor(
            self.get_mob_entity().living_entity.entity.block_pos.load(),
            world,
        ) >= 0.0
    }

    fn is_navigation<'a>(&'a self) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async {
            let navigator = self.get_mob_entity().navigator.lock().unwrap();
            !navigator.is_idle()
        })
    }

    fn is_panicking(&self) -> bool {
        if self.get_mob_entity().brain.as_ref().is_some_and(|brain| {
            brain.has_value::<crate::entity::ai::brain::memory::IsPanickingMemory>()
        }) {
            return true;
        }

        self.get_mob_entity()
            .goals_selector
            .lock()
            .unwrap()
            .is_panic_running()
    }

    fn should_follow_leash(&self) -> bool {
        true
    }

    fn on_short_leash_tick(&self) {
        // TODO: implement
    }

    fn before_leash_tick(&self) {
        // TODO: implement
    }

    fn get_follow_leash_speed(&self) -> f32 {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::{fire_aspect_ticks, knockback_enchantment_strength};

    #[test]
    fn fire_aspect_uses_eighty_ticks_per_level() {
        assert_eq!(fire_aspect_ticks(1), 80);
        assert_eq!(fire_aspect_ticks(2), 160);
    }

    #[test]
    fn knockback_enchantment_adds_half_strength_per_level() {
        assert_eq!(knockback_enchantment_strength(1), 0.5);
        assert_eq!(knockback_enchantment_strength(2), 1.0);
    }
}
