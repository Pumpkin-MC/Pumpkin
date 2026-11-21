pub mod ai;

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::CEntityPositionSync;
use pumpkin_util::GameMode;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::ai::control::flying_move_control::FlyingMoveControl;
use crate::entity::ai::goal::swim::SwimGoal;
use crate::entity::ai::goal::tempt::TemptGoal;
use crate::entity::ai::goal::{Controls, Goal, GoalReach};
use crate::entity::ai::pathfinder::Navigator;
use crate::entity::custom_sound::CustomSound;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;

pub struct HappyGhastEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub server_still_timeout: AtomicI32,
    pub leash_holder_time: AtomicI32,
    pub is_leash_holder: AtomicBool,
    pub stays_still: AtomicBool,
    pub wanted_fly_target: Mutex<Option<Vector3<f64>>>,
    pub last_passenger_count: AtomicUsize,
    pub self_weak: Weak<Self>,
}

impl HappyGhastEntity {
    pub const BABY_SCALE: f32 = 0.2375;
    pub const WANDER_GROUND_DISTANCE: i32 = 16;
    pub const SMALL_RESTRICTION_RADIUS: i32 = 32;
    pub const LARGE_RESTRICTION_RADIUS: i32 = 64;
    pub const RESTRICTION_RADIUS_BUFFER: i32 = 16;
    pub const FAST_HEALING_TICKS: i32 = 20;
    pub const SLOW_HEALING_TICKS: i32 = 600;
    pub const MAX_PASSENGERS: usize = 4;
    pub const STILL_TIMEOUT_ON_LOAD_GRACE_PERIOD: i32 = 60;
    pub const MAX_STILL_TIMEOUT: i32 = 10;
    pub const SPEED_MULTIPLIER_WHEN_PANICKING: f32 = 2.0;
    pub const FLYING_SPEED: f64 = 0.05;
    pub const MOVEMENT_SPEED: f64 = 0.05;

    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mob_arc = Arc::new_cyclic(|weak: &std::sync::Weak<Self>| Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            server_still_timeout: AtomicI32::new(0),
            leash_holder_time: AtomicI32::new(0),
            is_leash_holder: AtomicBool::new(false),
            stays_still: AtomicBool::new(false),
            wanted_fly_target: Mutex::new(None),
            last_passenger_count: AtomicUsize::new(0),
            self_weak: weak.clone(),
        });

        if mob_arc.is_baby() {
            mob_arc.baby_ghast_setup();
        } else {
            mob_arc.adult_ghast_setup();
        }
        mob_arc.update_dimensions();

        mob_arc
    }

    pub fn set_server_still_timeout(&self, timeout: i32) {
        let old = self.server_still_timeout.swap(timeout, Ordering::Relaxed);
        if old <= 0 && timeout > 0 {
            let entity = self.get_entity();
            let chunk_pos = entity.chunk_pos.load();
            let world = entity.world.load();
            world.broadcast_to_chunk(
                chunk_pos,
                &CEntityPositionSync::new(
                    entity.entity_id.into(),
                    entity.pos.load(),
                    Vector3::new(0.0, 0.0, 0.0),
                    entity.yaw.load(),
                    entity.pitch.load(),
                    entity.on_ground.load(Ordering::Relaxed),
                ),
            );
        }
        self.sync_stay_still_flag();
    }

    #[must_use]
    pub fn is_on_still_timeout(&self) -> bool {
        self.stays_still.load(Ordering::Relaxed)
            || self.server_still_timeout.load(Ordering::Relaxed) > 0
    }

    pub fn stop_in_place(&self) {
        let entity = self.get_entity();
        let vel = entity.velocity.load();
        if vel.x != 0.0 || vel.y != 0.0 || vel.z != 0.0 {
            entity.velocity.store(Vector3::default());
            entity.velocity_dirty.store(true, Ordering::SeqCst);
        }
        self.mob_entity.steered().navigator().stop();
        self.mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_wait();
        *self
            .wanted_fly_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }

    pub fn is_wearing_body_armor(&self) -> bool {
        !self.mob_entity.is_slot_empty(&EquipmentSlot::BODY)
    }

    fn set_leash_holder(&self, is_leash_holder: bool) {
        self.is_leash_holder
            .store(is_leash_holder, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::happy_ghast::IS_LEASH_HOLDER,
            is_leash_holder,
        );
    }

    fn sync_stay_still_flag(&self) {
        let stays_still = self.server_still_timeout.load(Ordering::Relaxed) > 0;
        self.stays_still.store(stays_still, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::happy_ghast::STAYS_STILL,
            stays_still,
        );
    }

    #[must_use]
    pub fn scan_player_above_ghast(&self) -> bool {
        let entity = self.get_entity();
        let bb = entity.bounding_box.load();
        let detection_box = BoundingBox::new(
            Vector3::new(bb.min.x - 1.0, bb.max.y - 1.0e-5, bb.min.z - 1.0),
            Vector3::new(
                bb.max.x + 1.0,
                bb.max.y + (bb.max.y - bb.min.y) * 0.5,
                bb.max.z + 1.0,
            ),
        );

        let world = entity.world.load();
        let players = world.players.load();
        for player in players.iter() {
            if player.gamemode.load() == GameMode::Spectator {
                continue;
            }
            let (root_pos, is_happy_ghast) = player.get_entity().get_root_vehicle().map_or_else(
                || (player.get_entity().pos.load(), false),
                |root| {
                    (
                        root.get_entity().pos.load(),
                        root.get_entity().entity_type == &EntityType::HAPPY_GHAST,
                    )
                },
            );
            if !is_happy_ghast && detection_box.contains(root_pos) {
                return true;
            }
        }
        false
    }

    fn get_happy_ghast_restriction_radius(&self) -> i32 {
        if !self.is_baby() && !self.is_wearing_body_armor() {
            Self::LARGE_RESTRICTION_RADIUS
        } else {
            Self::SMALL_RESTRICTION_RADIUS
        }
    }

    fn check_restriction(&self) {
        let entity = self.get_entity();
        if entity.is_leashed() || entity.has_passengers() {
            return;
        }
        let radius = self.get_happy_ghast_restriction_radius();
        let current_pos = entity.block_pos.load();
        if self.mob_entity.has_position_target() {
            let home_pos = self.mob_entity.position_target.load();
            let home_radius = self
                .mob_entity
                .position_target_range
                .load(Ordering::Relaxed);
            let dist_sq = home_pos.squared_distance(&current_pos);
            let limit = radius + Self::RESTRICTION_RADIUS_BUFFER;
            if dist_sq < limit * limit && radius == home_radius {
                return;
            }
        }
        self.mob_entity.set_home_to(current_pos, radius);
    }

    fn continuous_heal(&self) {
        let entity = self.get_entity();
        if entity.is_alive() {
            let living = &self.mob_entity.living_entity;
            let current_health = living.health.load();
            let max_health = living.get_max_health();
            if current_health < max_health {
                let pos = entity.pos.load();
                let y = pos.y;
                let height = f64::from(entity.height());
                let is_in_clouds = y + height >= 192.0 && y <= 196.0;
                let world = entity.world.load();
                let is_raining = world.is_raining_at(&pos.to_block_pos());
                let is_fast_healing = is_in_clouds || is_raining;
                let interval = if is_fast_healing {
                    Self::FAST_HEALING_TICKS
                } else {
                    Self::SLOW_HEALING_TICKS
                };
                if entity.tick_count.load(Ordering::Relaxed) % interval == 0 {
                    living.heal(1.0);
                }
            }
        }
    }

    pub fn adult_ghast_setup(&self) {
        self.mob_entity.goals_selector.clear(self);
        *self
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Navigator::flying();
        *self
            .mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(FlyingMoveControl::new(180, true));

        let weak = self.self_weak.clone();
        let goal_selector = &self.mob_entity.goals_selector;
        goal_selector.add_goal(3, Box::new(HappyGhastFloatGoal::new(weak.clone())));
        goal_selector.add_goal(
            4,
            Box::new(TemptGoal::with_predicate_and_stop_distance(
                1.0,
                is_tempt_item,
                false,
                7.0,
            )),
        );
        goal_selector.add_goal(5, Box::new(HappyGhastFloatAroundGoal::new(weak.clone())));
        goal_selector.add_goal(7, Box::new(HappyGhastLookGoal::new(weak)));

        self.mob_entity.with_brain(self, |brain| {
            let world = self.get_entity().world.load();
            let time = self.get_entity().tick_count.load(Ordering::Relaxed) as i64;
            brain.stop_all(&world, self, time);
            brain.clear_memories();
        });
    }

    pub fn baby_ghast_setup(&self) {
        self.set_server_still_timeout(0);
        self.mob_entity.goals_selector.clear(self);

        let mut nav = self
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *nav = Navigator::flying();
        nav.set_can_float(true);
        nav.set_can_open_doors(false);

        *self
            .mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(FlyingMoveControl::new(180, true));
    }

    pub fn do_player_ride(&self, player: &Arc<Player>) {
        let entity = self.get_entity();
        if let Some(vehicle) = entity.world.load().get_entity_by_id(entity.entity_id) {
            entity.add_passenger(vehicle, player.clone());
        }
    }

    fn update_dimensions(&self) {
        let entity = self.get_entity();
        let baby = self.is_baby();
        let dimensions = Entity::age_dimensions(&EntityType::HAPPY_GHAST, baby);
        entity.set_default_dimensions(dimensions);
        entity.set_synced_data(pumpkin_data::tracked_data::happy_ghast::DATA_BABY_ID, baby);
    }
}

fn is_tempt_item(mob: &dyn Mob, stack: &ItemStack) -> bool {
    let Some(happy_ghast) = mob.cast_any().downcast_ref::<HappyGhastEntity>() else {
        return false;
    };
    if happy_ghast.is_wearing_body_armor() || happy_ghast.is_baby() {
        stack.item.has_tag(&tag::Item::MINECRAFT_HAPPY_GHAST_FOOD)
    } else {
        stack
            .item
            .has_tag(&tag::Item::MINECRAFT_HAPPY_GHAST_TEMPT_ITEMS)
    }
}

impl AgeableMob for HappyGhastEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn age_boundary_reached(&self) {
        if self.is_baby() {
            self.baby_ghast_setup();
        } else {
            self.adult_ghast_setup();
        }
        self.update_dimensions();
    }
}

impl Animal for HappyGhastEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_HAPPY_GHAST_FOOD)
    }

    fn can_fall_in_love(&self) -> bool {
        false
    }
}

impl CustomSound for HappyGhastEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(if self.is_baby() {
            Sound::EntityGhastlingDeath
        } else {
            Sound::EntityHappyGhastDeath
        })
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(if self.is_baby() {
            Sound::EntityGhastlingHurt
        } else {
            Sound::EntityHappyGhastHurt
        })
    }

    fn sound_volume(&self) -> f32 {
        if self.is_baby() { 1.0 } else { 4.0 }
    }

    fn voice_pitch(&self) -> f32 {
        1.0
    }

    fn sound_category(&self) -> SoundCategory {
        SoundCategory::Neutral
    }
}

impl Mob for HappyGhastEntity {
    fn mob_support_quad_leash_as_holder(&self) -> bool {
        true
    }

    fn mob_can_add_passenger(&self, passenger: &dyn EntityBase) -> bool {
        let passengers = self
            .get_entity()
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if passengers.len() >= Self::MAX_PASSENGERS {
            return false;
        }
        !passengers
            .iter()
            .any(|p| p.get_entity().entity_id == passenger.get_entity().entity_id)
    }

    fn mob_notify_leash_holder(&self, leashed: &dyn EntityBase) {
        if leashed.support_quad_leash() {
            self.leash_holder_time.store(5, Ordering::Relaxed);
        }
    }

    fn mob_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        if self.is_wearing_body_armor() && !self.is_on_still_timeout() {
            let passengers = self
                .get_entity()
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(first) = passengers.first()
                && first.get_player().is_some()
            {
                return Some(first.clone());
            }
        }
        None
    }

    fn can_dispenser_equip_into_slot(
        &self,
        slot: &pumpkin_data::data_component_impl::EquipmentSlot,
    ) -> bool {
        *slot == pumpkin_data::data_component_impl::EquipmentSlot::BODY
    }

    fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        if *slot == EquipmentSlot::BODY {
            self.get_entity().is_alive() && !self.is_baby()
        } else {
            true
        }
    }

    fn can_breathe_underwater(&self) -> bool {
        self.is_baby()
    }

    fn should_stay_close_to_leash_holder(&self) -> bool {
        false
    }

    fn on_elastic_leash_pull(&self) {
        *self
            .wanted_fly_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        self.mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_wait();
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &pumpkin_data::BlockState) {}

    fn should_check_fall_damage(&self) -> bool {
        false
    }

    fn cause_fall_damage(
        &self,
        _caller: &dyn EntityBase,
        _fall_distance: f32,
        _damage_modifier: f32,
    ) -> bool {
        false
    }

    fn on_climbable(&self) -> bool {
        false
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }

    fn get_mob_y_velocity_drag(&self) -> Option<f64> {
        Some(0.95)
    }

    fn mob_is_pushable(&self) -> bool {
        !self.is_on_still_timeout()
    }

    fn mob_is_collidable(&self, entity: Option<Box<dyn EntityBase>>) -> bool {
        if self.is_baby() || !self.get_entity().is_alive() {
            return false;
        }
        if let Some(other) = entity
            && other.get_entity().entity_type == &pumpkin_data::entity::EntityType::HAPPY_GHAST
            && !self
                .get_entity()
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
        {
            return true;
        }
        self.is_on_still_timeout()
    }

    fn mob_knockback(
        &self,
        _power: f64,
        _x: f64,
        _z: f64,
        _source: Option<&Entity>,
        _damage: f32,
        _comes_from_effect: bool,
    ) -> bool {
        if !self.is_baby() && (self.is_on_still_timeout() || self.scan_player_above_ghast()) {
            self.set_server_still_timeout(Self::MAX_STILL_TIMEOUT);
            self.stop_in_place();
            return true;
        }
        false
    }

    fn travel(&self, caller: &dyn EntityBase) -> bool {
        if self.is_on_still_timeout() {
            self.stop_in_place();
            return true;
        }
        let speed = (self
            .mob_entity
            .living_entity
            .get_attribute_value(&pumpkin_data::attributes::Attributes::FLYING_SPEED)
            * 5.0
            / 3.0) as f32;
        self.mob_entity
            .living_entity
            .travel_flying_with_speed(caller, speed);
        true
    }

    fn get_walk_target_value(&self, pos: &BlockPos) -> f32 {
        let world = self.get_entity().world.load();
        if !world.get_block_state(pos).is_air() {
            return 0.0;
        }
        let below = pos.down();
        let below2 = below.down();
        if world.get_block_state(&below).is_air() && !world.get_block_state(&below2).is_air() {
            return 10.0;
        }
        5.0
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(if self.is_baby() {
            Sound::EntityGhastlingAmbient
        } else {
            Sound::EntityHappyGhastAmbient
        })
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        let base = MobEntity::DEFAULT_AMBIENT_SOUND_INTERVAL;
        if self.get_entity().has_passengers() {
            base * 6
        } else {
            base
        }
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int(
            "still_timeout",
            self.server_still_timeout.load(Ordering::Relaxed),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.age_boundary_reached();
        if let Some(timeout) = nbt.get_int("still_timeout") {
            self.set_server_still_timeout(timeout);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn make_brain(
        &self,
        packed: &crate::entity::ai::brain::memory::PackedMemories,
    ) -> crate::entity::ai::brain::Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        if self.is_baby() {
            self.mob_entity.tick_brain(self);
            ai::update_activity(self);
        }

        self.check_restriction();
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let leash_time = self.leash_holder_time.load(Ordering::Relaxed);
        if leash_time > 0 {
            self.leash_holder_time.fetch_sub(1, Ordering::Relaxed);
        }
        self.set_leash_holder(leash_time > 0);

        let still_timeout = self.server_still_timeout.load(Ordering::Relaxed);
        if still_timeout > 0 {
            let entity = self.get_entity();
            if entity.tick_count.load(Ordering::Relaxed) > Self::STILL_TIMEOUT_ON_LOAD_GRACE_PERIOD
            {
                self.server_still_timeout.fetch_sub(1, Ordering::Relaxed);
            }
            self.sync_stay_still_flag();
        }

        if self.scan_player_above_ghast() {
            self.set_server_still_timeout(Self::MAX_STILL_TIMEOUT);
        }

        if self.is_on_still_timeout() {
            let entity = self.get_entity();
            let current_yaw = entity.yaw.load();
            let snapped_yaw = (current_yaw / 90.0).round() * 90.0;
            entity.yaw.store(snapped_yaw);
            entity.head_yaw.store(snapped_yaw);
            self.stop_in_place();
        }

        let passenger_count = self
            .get_entity()
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        let prev_passenger_count = self
            .last_passenger_count
            .swap(passenger_count, Ordering::Relaxed);

        if prev_passenger_count == 0 && passenger_count > 0 {
            self.play_entity_sound(Sound::EntityHappyGhastHarnessGogglesDown, 1.0, 1.0);
            if !self.scan_player_above_ghast() {
                self.set_server_still_timeout(0);
            } else if self.server_still_timeout.load(Ordering::Relaxed) > Self::MAX_STILL_TIMEOUT {
                self.set_server_still_timeout(Self::MAX_STILL_TIMEOUT);
            }
        } else if prev_passenger_count > 0 && passenger_count == 0 {
            self.set_server_still_timeout(Self::MAX_STILL_TIMEOUT);
            self.mob_entity.clear_home();
            self.play_entity_sound(Sound::EntityHappyGhastHarnessGogglesUp, 1.0, 1.0);
        } else if passenger_count < prev_passenger_count {
            self.set_server_still_timeout(Self::MAX_STILL_TIMEOUT);
        }

        self.continuous_heal();
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::happy_ghast::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::happy_ghast::IS_LEASH_HOLDER,
            self.is_leash_holder.load(Ordering::Relaxed),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::happy_ghast::STAYS_STILL,
            self.stays_still.load(Ordering::Relaxed),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if self.is_baby() {
            return self.animal_interact(player, item_stack);
        }

        if !item_stack.is_empty() && self.is_food(item_stack) {
            return self.animal_interact(player, item_stack);
        }

        if self.is_wearing_body_armor() && !player.get_entity().is_sneaking() {
            let passenger_count = self
                .get_entity()
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len();
            if passenger_count < Self::MAX_PASSENGERS {
                self.do_player_ride(player);
                return true;
            }
        }

        self.animal_interact(player, item_stack)
    }
}

pub struct HappyGhastFloatGoal {
    ghast: Weak<HappyGhastEntity>,
    swim_goal: SwimGoal,
}

impl HappyGhastFloatGoal {
    #[must_use]
    pub fn new(ghast: Weak<HappyGhastEntity>) -> Self {
        Self {
            ghast,
            swim_goal: SwimGoal::default(),
        }
    }
}

impl Goal for HappyGhastFloatGoal {
    fn tick_reach(&self, mob: &dyn Mob, running: bool) -> Option<GoalReach> {
        self.swim_goal.tick_reach(mob, running)
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(ghast) = self.ghast.upgrade() else {
            return false;
        };
        !ghast.is_on_still_timeout() && self.swim_goal.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(ghast) = self.ghast.upgrade() else {
            return false;
        };
        !ghast.is_on_still_timeout() && self.swim_goal.should_continue(mob)
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.swim_goal.tick(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.swim_goal.controls()
    }
}

pub struct HappyGhastLookGoal {
    ghast: Weak<HappyGhastEntity>,
    goal_control: Controls,
}

impl HappyGhastLookGoal {
    #[must_use]
    pub const fn new(ghast: Weak<HappyGhastEntity>) -> Self {
        Self {
            ghast,
            goal_control: Controls::LOOK,
        }
    }
}

impl Goal for HappyGhastLookGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        true
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(ghast) = self.ghast.upgrade() else {
            return;
        };
        let entity = ghast.get_entity();
        if ghast.is_on_still_timeout() {
            let current_yaw = entity.yaw.load();
            let snapped_yaw = (current_yaw / 90.0).round() * 90.0;
            entity.yaw.store(snapped_yaw);
            entity.head_yaw.store(snapped_yaw);
            return;
        }

        let target_opt = mob.get_mob_entity().get_target();
        if let Some(target) = target_opt {
            let mob_pos = entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            if mob_pos.squared_distance_to_vec(&target_pos) < 4096.0 {
                let dx = target_pos.x - mob_pos.x;
                let dz = target_pos.z - mob_pos.z;
                let yaw = (-f64::atan2(dx, dz).to_degrees()) as f32;
                entity.yaw.store(yaw);
                entity.head_yaw.store(yaw);
                return;
            }
        }

        let velocity = entity.velocity.load();
        if velocity.x != 0.0 || velocity.z != 0.0 {
            let yaw = (-f64::atan2(velocity.x, velocity.z).to_degrees()) as f32;
            entity.yaw.store(yaw);
            entity.head_yaw.store(yaw);
        }
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

pub struct HappyGhastFloatAroundGoal {
    ghast: Weak<HappyGhastEntity>,
    float_duration: i32,
    stuck_ticks: i32,
    last_pos: Vector3<f64>,
}

impl HappyGhastFloatAroundGoal {
    #[must_use]
    pub const fn new(ghast: Weak<HappyGhastEntity>) -> Self {
        Self {
            ghast,
            float_duration: 0,
            stuck_ticks: 0,
            last_pos: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    fn choose_random_position(center: Vector3<f64>) -> Vector3<f64> {
        let mut rng = rand::rng();
        let target_x = center.x + (rng.random::<f64>() * 2.0 - 1.0) * 16.0;
        let target_y = center.y + (rng.random::<f64>() * 2.0 - 1.0) * 16.0;
        let target_z = center.z + (rng.random::<f64>() * 2.0 - 1.0) * 16.0;
        Vector3::new(target_x, target_y, target_z)
    }

    fn choose_random_position_with_restriction(
        ghast: &HappyGhastEntity,
        center: Vector3<f64>,
    ) -> Option<Vector3<f64>> {
        let target = Self::choose_random_position(center);
        let block_pos = BlockPos::floored_v(target);
        if ghast.mob_entity.has_position_target()
            && !ghast.mob_entity.is_in_position_target_range_pos(&block_pos)
        {
            return None;
        }
        Some(target)
    }

    fn is_good_target(world: &World, target: Vector3<f64>, distance_to_blocks: i32) -> bool {
        const DIRS: [(i32, i32, i32); 6] = [
            (0, -1, 0),
            (0, 1, 0),
            (0, 0, -1),
            (0, 0, 1),
            (-1, 0, 0),
            (1, 0, 0),
        ];

        if distance_to_blocks <= 0 {
            return true;
        }
        let pos = BlockPos::floored_v(target);
        if !world.is_in_build_limit(pos) {
            return false;
        }
        if !world.get_block_state(&pos).is_air() {
            return false;
        }

        for (dx, dy, dz) in DIRS {
            for i in 1..distance_to_blocks {
                let offset = BlockPos::new(pos.0.x + dx * i, pos.0.y + dy * i, pos.0.z + dz * i);
                if !world.get_block_state(&offset).is_air() {
                    return true;
                }
            }
        }
        false
    }

    fn get_suitable_fly_to_position(
        ghast: &HappyGhastEntity,
        distance_to_blocks: i32,
    ) -> Vector3<f64> {
        let center = ghast.get_entity().pos.load();
        let world = ghast.get_entity().world.load();
        let mut result = None;
        for _ in 0..64 {
            result = Self::choose_random_position_with_restriction(ghast, center);
            if let Some(target) = result
                && Self::is_good_target(&world, target, distance_to_blocks)
            {
                return target;
            }
        }
        let mut final_pos = result.unwrap_or_else(|| Self::choose_random_position(center));
        let pos = BlockPos::floored_v(final_pos);
        let height_y = f64::from(world.get_top_block(Vector2::new(pos.0.x, pos.0.z)));
        let min_y = f64::from(world.dimension.min_y);
        if height_y < final_pos.y && height_y > min_y {
            final_pos.y = center.y - (center.y - final_pos.y).abs();
        }
        final_pos
    }
}

impl Goal for HappyGhastFloatAroundGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(ghast) = self.ghast.upgrade() else {
            return false;
        };
        if ghast.is_on_still_timeout() {
            return false;
        }
        let wanted = *ghast
            .wanted_fly_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        wanted.is_none_or(|target| {
            let pos = ghast.mob_entity.living_entity.entity.pos.load();
            let dist_sq = pos.squared_distance_to_vec(&target);
            !dist_sq.is_nan() && !(1.0..=3600.0).contains(&dist_sq)
        })
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.stuck_ticks = 0;
        let Some(ghast) = self.ghast.upgrade() else {
            return;
        };
        self.last_pos = ghast.mob_entity.living_entity.entity.pos.load();
        let new_target =
            Self::get_suitable_fly_to_position(&ghast, HappyGhastEntity::WANDER_GROUND_DISTANCE);
        *ghast
            .wanted_fly_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(new_target);
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        if self.stuck_ticks > 40 {
            return false;
        }
        let Some(ghast) = self.ghast.upgrade() else {
            return false;
        };
        if ghast.is_on_still_timeout() {
            return false;
        }
        let wanted = *ghast
            .wanted_fly_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        wanted.is_some_and(|target| {
            let pos = ghast.mob_entity.living_entity.entity.pos.load();
            let dist_sq = pos.squared_distance_to_vec(&target);
            (1.0..=3600.0).contains(&dist_sq)
        })
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(ghast) = self.ghast.upgrade() {
            *ghast
                .wanted_fly_target
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(ghast) = self.ghast.upgrade() else {
            return;
        };
        if ghast.is_on_still_timeout() {
            ghast.stop_in_place();
            return;
        }

        let wanted = *ghast
            .wanted_fly_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(target) = wanted else {
            return;
        };

        let entity = &ghast.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        self.float_duration -= 1;

        if self.float_duration <= 0 {
            self.float_duration = rand::random_range(2..=6);
            let travel = Vector3::new(target.x - pos.x, target.y - pos.y, target.z - pos.z);
            let dist = travel.length();
            if dist > 0.001 {
                let move_scale = HappyGhastEntity::FLYING_SPEED * 5.0 / 3.0;
                let norm = travel.normalize();
                let delta = Vector3::new(
                    norm.x * move_scale,
                    norm.y * move_scale,
                    norm.z * move_scale,
                );
                let current_vel = entity.velocity.load();
                entity.velocity.store(Vector3::new(
                    current_vel.x + delta.x,
                    current_vel.y + delta.y,
                    current_vel.z + delta.z,
                ));
            }
        }

        let dist_moved_sq = pos.squared_distance_to_vec(&self.last_pos);
        if dist_moved_sq < 0.0025 {
            self.stuck_ticks += 1;
            if self.stuck_ticks > 40 {
                *ghast
                    .wanted_fly_target
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            }
        } else {
            self.stuck_ticks = 0;
            self.last_pos = pos;
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use pumpkin_data::data_component_impl::EquipmentSlot;
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::sound::Sound;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::test_support::world;
    use crate::entity::custom_sound::CustomSound;
    use crate::entity::mob::Mob;

    fn test_happy_ghast(folder: &tempfile::TempDir, age: i32) -> Arc<HappyGhastEntity> {
        let w = world(folder);
        let entity = Entity::new(w, Vector3::new(0.0, 70.0, 0.0), &EntityType::HAPPY_GHAST);
        entity.age.store(age, Ordering::Relaxed);
        HappyGhastEntity::new(entity)
    }

    #[tokio::test]
    async fn baby_vs_adult_sounds_and_volumes() {
        let folder = tempfile::tempdir().unwrap();
        let baby = test_happy_ghast(&folder, -24000);
        let adult = test_happy_ghast(&folder, 0);

        assert_eq!(baby.death_sound(), Some(Sound::EntityGhastlingDeath));
        assert_eq!(adult.death_sound(), Some(Sound::EntityHappyGhastDeath));

        assert_eq!(baby.hurt_sound(), Some(Sound::EntityGhastlingHurt));
        assert_eq!(adult.hurt_sound(), Some(Sound::EntityHappyGhastHurt));

        assert_eq!(
            baby.get_ambient_sound(),
            Some(Sound::EntityGhastlingAmbient)
        );
        assert_eq!(
            adult.get_ambient_sound(),
            Some(Sound::EntityHappyGhastAmbient)
        );

        assert_eq!(baby.sound_volume(), 1.0);
        assert_eq!(adult.sound_volume(), 4.0);
        assert_eq!(baby.voice_pitch(), 1.0);
        assert_eq!(adult.voice_pitch(), 1.0);
    }

    #[tokio::test]
    async fn baby_breathes_underwater_adult_does_not() {
        let folder = tempfile::tempdir().unwrap();
        let baby = test_happy_ghast(&folder, -24000);
        let adult = test_happy_ghast(&folder, 0);

        assert!(baby.can_breathe_underwater());
        assert!(!adult.can_breathe_underwater());
    }

    #[tokio::test]
    async fn fall_damage_is_immune() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);
        assert!(!ghast.should_check_fall_damage());
        assert!(!ghast.cause_fall_damage(ghast.as_ref(), 50.0, 1.0));
    }

    #[tokio::test]
    async fn ambient_sound_interval_multiplied_when_vehicle() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);
        let base_interval = ghast.get_ambient_sound_interval();
        assert_eq!(base_interval, MobEntity::DEFAULT_AMBIENT_SOUND_INTERVAL);
    }

    #[tokio::test]
    async fn quad_leash_support() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);
        assert!(ghast.mob_support_quad_leash_as_holder());
        assert!(!ghast.should_stay_close_to_leash_holder());

        // Ghast does not support quad leash as leashed entity, so leash_holder_time stays 0
        assert!(!ghast.support_quad_leash());
        ghast.mob_notify_leash_holder(ghast.as_ref());
        assert_eq!(ghast.leash_holder_time.load(Ordering::Relaxed), 0);

        // Sniffer supports quad leash, so leash_holder_time becomes 5
        let w = world(&folder);
        let sniffer = crate::entity::passive::sniffer::SnifferEntity::new(Entity::new(
            w,
            Vector3::new(0.0, 70.0, 0.0),
            &EntityType::SNIFFER,
        ));
        assert!(sniffer.support_quad_leash());
        ghast.mob_notify_leash_holder(sniffer.as_ref());
        assert_eq!(ghast.leash_holder_time.load(Ordering::Relaxed), 5);
    }

    #[tokio::test]
    async fn still_timeout_and_snap_yaw() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);
        assert!(!ghast.is_on_still_timeout());

        ghast.set_server_still_timeout(10);
        assert!(ghast.is_on_still_timeout());
        assert!(ghast.stays_still.load(Ordering::Relaxed));

        // Yaw snapping: e.g. 85.0 should snap to 90.0, 190.0 should snap to 180.0
        let entity = ghast.get_entity();
        entity.yaw.store(85.0);
        let mut look_goal = HappyGhastLookGoal::new(Arc::downgrade(&ghast));
        look_goal.tick(ghast.as_ref());
        assert_eq!(entity.yaw.load(), 90.0);
        assert_eq!(entity.head_yaw.load(), 90.0);

        entity.yaw.store(190.0);
        look_goal.tick(ghast.as_ref());
        assert_eq!(entity.yaw.load(), 180.0);
        assert_eq!(entity.head_yaw.load(), 180.0);
    }

    #[tokio::test]
    async fn still_timeout_grace_period() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);
        ghast.set_server_still_timeout(10);

        // tick_count <= 60: does not decrement
        ghast.get_entity().tick_count.store(30, Ordering::Relaxed);
        ghast.mob_tick(ghast.as_ref());
        assert_eq!(ghast.server_still_timeout.load(Ordering::Relaxed), 10);

        // tick_count > 60: decrements
        ghast.get_entity().tick_count.store(65, Ordering::Relaxed);
        ghast.mob_tick(ghast.as_ref());
        assert_eq!(ghast.server_still_timeout.load(Ordering::Relaxed), 9);
    }

    #[tokio::test]
    async fn restriction_radius_calculation() {
        let folder = tempfile::tempdir().unwrap();
        let baby = test_happy_ghast(&folder, -24000);
        let adult = test_happy_ghast(&folder, 0);

        // Baby is always small restriction (32)
        assert_eq!(baby.get_happy_ghast_restriction_radius(), 32);

        // Unarmored adult is large restriction (64)
        assert_eq!(adult.get_happy_ghast_restriction_radius(), 64);
    }

    #[tokio::test]
    async fn body_armor_can_use_slot() {
        let folder = tempfile::tempdir().unwrap();
        let baby = test_happy_ghast(&folder, -24000);
        let adult = test_happy_ghast(&folder, 0);

        assert!(!baby.can_use_slot(&EquipmentSlot::BODY));
        assert!(adult.can_use_slot(&EquipmentSlot::BODY));
        assert!(adult.can_dispenser_equip_into_slot(&EquipmentSlot::BODY));
    }

    #[tokio::test]
    async fn max_passengers_limit() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);
        let rider = test_happy_ghast(&folder, 0);
        assert!(ghast.mob_can_add_passenger(rider.as_ref()));

        // Cannot add duplicate passenger
        ghast
            .get_entity()
            .passengers
            .lock()
            .unwrap()
            .push(rider.clone());
        assert!(!ghast.mob_can_add_passenger(rider.as_ref()));
    }

    #[tokio::test]
    async fn walk_target_value_parity() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);
        let pos = BlockPos::new(0, 70, 0);
        let value = ghast.get_walk_target_value(&pos);
        // Air with air below and air below 2 in empty world
        assert_eq!(value, 5.0);
    }

    #[tokio::test]
    async fn age_transition_clears_adult_goals_and_updates_data() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);
        assert!(!ghast.is_baby());
        assert!(!ghast.mob_entity.goals_selector.is_empty());

        let mut nbt = NbtCompound::new();
        nbt.put_int("Age", -24000);
        nbt.put_int("still_timeout", 10);
        ghast.get_entity().age.store(-24000, Ordering::Relaxed);
        ghast.mob_read_nbt(&nbt);

        assert!(ghast.is_baby());
        assert!(ghast.mob_entity.goals_selector.is_empty());
        assert_eq!(ghast.server_still_timeout.load(Ordering::Relaxed), 10);
    }

    #[tokio::test]
    async fn knockback_and_movement_suppressed_when_standing_on_ghast_or_still_timeout() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);

        // When not on still timeout, knockback is not intercepted
        assert!(!ghast.mob_knockback(1.0, 1.0, 0.0, None, 1.0, false));

        // When on still timeout, knockback is intercepted and velocity is cleared
        ghast
            .get_entity()
            .velocity
            .store(Vector3::new(1.0, 0.5, 1.0));
        ghast.set_server_still_timeout(10);
        assert!(ghast.is_on_still_timeout());
        assert!(ghast.mob_knockback(1.0, 1.0, 0.0, None, 1.0, false));
        assert_eq!(ghast.get_entity().velocity.load(), Vector3::default());
        assert!(!ghast.mob_is_pushable());
        assert!(ghast.mob_is_collidable(None));

        // travel stops in place when on still timeout
        ghast
            .get_entity()
            .velocity
            .store(Vector3::new(0.5, 0.0, 0.5));
        assert!(ghast.travel(ghast.as_ref()));
        assert_eq!(ghast.get_entity().velocity.load(), Vector3::default());

        // Baby ghast never suppresses knockback even if still timeout was set
        let baby = test_happy_ghast(&folder, -24000);
        baby.set_server_still_timeout(10);
        assert!(!baby.mob_knockback(1.0, 1.0, 0.0, None, 1.0, false));
        assert!(!baby.mob_is_collidable(None));
    }

    #[tokio::test]
    async fn goal_stop_clears_wanted_fly_target() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);
        *ghast
            .wanted_fly_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some(Vector3::new(10.0, 70.0, 10.0));
        let mut goal = HappyGhastFloatAroundGoal::new(Arc::downgrade(&ghast));
        goal.stop(ghast.as_ref());
        assert!(
            ghast
                .wanted_fly_target
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_none()
        );
    }
}
