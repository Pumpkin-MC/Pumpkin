pub mod ai;

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

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
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::control::flying_move_control::FlyingMoveControl;
use crate::entity::ai::control::ghast_move_control::GhastMoveControl;
use crate::entity::ai::control::happy_ghast_look_control::HappyGhastLookControl;
use crate::entity::ai::control::look_control::LookControl;
use crate::entity::ai::goal::random_float_around::RandomFloatAroundGoal;
use crate::entity::ai::goal::swim::SwimGoal;
use crate::entity::ai::goal::tempt::TemptGoal;
use crate::entity::ai::goal::{Controls, Goal, GoalReach};
use crate::entity::ai::pathfinder::Navigator;
use crate::entity::custom_sound::CustomSound;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    mob::{Mob, MobEntity, equipment::equip_on_target},
    passive::animal::Animal,
    player::Player,
};

const LEASH_SNAP_DISTANCE: f64 = 16.0;
const LEASH_ELASTIC_DISTANCE: f64 = 10.0;

pub struct HappyGhastEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub server_still_timeout: AtomicI32,
    pub leash_holder_time: AtomicI32,
    pub is_leash_holder: AtomicBool,
    pub stays_still: AtomicBool,
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

    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mob_arc = Arc::new_cyclic(|weak: &std::sync::Weak<Self>| Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            server_still_timeout: AtomicI32::new(0),
            leash_holder_time: AtomicI32::new(0),
            is_leash_holder: AtomicBool::new(false),
            stays_still: AtomicBool::new(false),
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
        self.get_entity()
            .requires_precise_position
            .store(self.is_on_still_timeout(), Ordering::Relaxed);
        if old <= 0 && timeout > 0 {
            let entity = self.get_entity();
            let pos = entity.pos.load();
            entity.last_sent_pos.store(pos);
            let world = entity.world.load();
            world.send_to_tracking_players(
                entity,
                &CEntityPositionSync::new(
                    entity.entity_id.into(),
                    pos,
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
            let root = player
                .get_entity()
                .get_root_vehicle()
                .unwrap_or_else(|| player.clone() as Arc<dyn EntityBase>);
            if root.get_entity().entity_type == &EntityType::HAPPY_GHAST {
                continue;
            }
            let root_pos = root.get_entity().pos.load();
            if detection_box.contains(root_pos) {
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
            let dx = i64::from(home_pos.0.x) - i64::from(current_pos.0.x);
            let dy = i64::from(home_pos.0.y) - i64::from(current_pos.0.y);
            let dz = i64::from(home_pos.0.z) - i64::from(current_pos.0.z);
            let dist_sq = dx * dx + dy * dy + dz * dz;
            let limit = i64::from(radius + Self::RESTRICTION_RADIUS_BUFFER);
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
                let world = entity.world.load();
                let is_fast_healing =
                    entity.is_in_clouds() || world.has_precipitation_at(&entity.block_pos.load());
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
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Box::new(GhastMoveControl::new(
            true,
            Some(|mob| {
                mob.cast_any()
                    .downcast_ref::<Self>()
                    .is_some_and(Self::is_on_still_timeout)
            }),
        ));
        *self
            .mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(HappyGhastLookControl::new());

        let weak = self.self_weak.clone();
        let goal_selector = &self.mob_entity.goals_selector;
        goal_selector.add_goal(3, Box::new(HappyGhastFloatGoal::new(weak)));
        goal_selector.add_goal(
            4,
            Box::new(TemptGoal::for_non_pathfinders(
                1.0,
                is_tempt_item,
                false,
                7.0,
            )),
        );
        goal_selector.add_goal(5, Box::new(RandomFloatAroundGoal::with_distance(16)));

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

        *self
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Navigator::baby_flying();

        *self
            .mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(FlyingMoveControl::new(180, true));

        *self
            .mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Box::new(LookControl::default());
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

    fn mob_on_passenger_added(&self, _passenger: &dyn EntityBase, is_first_passenger: bool) {
        if is_first_passenger {
            self.play_entity_sound(Sound::EntityHappyGhastHarnessGogglesDown, 1.0, 1.0);
        }
        if !self.scan_player_above_ghast() {
            self.set_server_still_timeout(0);
        } else if self.server_still_timeout.load(Ordering::Relaxed) > Self::MAX_STILL_TIMEOUT {
            self.set_server_still_timeout(Self::MAX_STILL_TIMEOUT);
        }
    }

    fn mob_on_passenger_removed(&self, _passenger: &dyn EntityBase, is_last_passenger: bool) {
        self.set_server_still_timeout(Self::MAX_STILL_TIMEOUT);
        if is_last_passenger {
            self.mob_entity.clear_home();
            self.play_entity_sound(Sound::EntityHappyGhastHarnessGogglesUp, 1.0, 1.0);
        }
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

    fn mob_max_passengers(&self) -> usize {
        Self::MAX_PASSENGERS
    }

    fn mob_leash_snap_distance(&self) -> f64 {
        LEASH_SNAP_DISTANCE
    }

    fn mob_leash_elastic_distance(&self) -> f64 {
        LEASH_ELASTIC_DISTANCE
    }

    fn mob_get_dismount_location_for_passenger(
        &self,
        _passenger: &dyn EntityBase,
    ) -> Option<Vector3<f64>> {
        let entity = self.get_entity();
        let pos = entity.pos.load();
        let bb = entity.bounding_box.load();
        Some(Vector3::new(pos.x, bb.max.y, pos.z))
    }

    fn mob_is_collidable(&self, entity: Option<&dyn EntityBase>) -> bool {
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

    fn travel(&self, caller: &dyn EntityBase) -> bool {
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
        let base = crate::entity::passive::animal::DEFAULT_AMBIENT_SOUND_INTERVAL;
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
        self.get_entity()
            .requires_precise_position
            .store(self.is_on_still_timeout(), Ordering::Relaxed);
        let mut leash_time = self.leash_holder_time.load(Ordering::Relaxed);
        if leash_time > 0 {
            leash_time -= 1;
            self.leash_holder_time.store(leash_time, Ordering::Relaxed);
        }
        self.set_leash_holder(leash_time > 0);

        let still_timeout = self.server_still_timeout.load(Ordering::Relaxed);
        if still_timeout > 0 {
            let entity = self.get_entity();
            if entity.tick_count.load(Ordering::Relaxed) > Self::STILL_TIMEOUT_ON_LOAD_GRACE_PERIOD
            {
                self.set_server_still_timeout(still_timeout - 1);
            }
        }

        if self.scan_player_above_ghast() {
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

        if equip_on_target(self, player, item_stack) {
            return true;
        }

        if self.is_wearing_body_armor() && !player.get_entity().is_sneaking() {
            self.do_player_ride(player);
            return true;
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use pumpkin_data::entity::EntityType;
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::control::LookControlTrait;
    use crate::entity::ai::control::happy_ghast_look_control::HappyGhastLookControl;
    use crate::entity::ai::test_support::world;
    use crate::entity::mob::Mob;

    fn test_happy_ghast(folder: &tempfile::TempDir, age: i32) -> Arc<HappyGhastEntity> {
        let w = world(folder);
        let entity = Entity::new(w, Vector3::new(0.0, 70.0, 0.0), &EntityType::HAPPY_GHAST);
        entity.age.store(age, Ordering::Relaxed);
        HappyGhastEntity::new(entity)
    }

    #[tokio::test]
    async fn still_timeout_and_snap_yaw() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);
        assert!(!ghast.is_on_still_timeout());

        ghast.set_server_still_timeout(10);
        assert!(ghast.is_on_still_timeout());
        assert!(ghast.stays_still.load(Ordering::Relaxed));
        assert!(
            ghast
                .get_entity()
                .requires_precise_position
                .load(Ordering::Relaxed)
        );
        assert!(ghast.is_collidable(None));

        let dismount = ghast.get_dismount_location_for_passenger(ghast.as_ref());
        let pos = ghast.get_entity().pos.load();
        let bb = ghast.get_entity().bounding_box.load();
        assert_eq!(dismount, Some(Vector3::new(pos.x, bb.max.y, pos.z)));

        // Yaw snapping: e.g. 85.0 should snap to 90.0, 190.0 should snap to 180.0
        let entity = ghast.get_entity();
        entity.yaw.store(85.0);
        let mut look_control = HappyGhastLookControl::new();
        look_control.tick(ghast.as_ref());
        assert_eq!(entity.yaw.load(), 90.0);
        assert_eq!(entity.head_yaw.load(), 90.0);

        entity.yaw.store(190.0);
        look_control.tick(ghast.as_ref());
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
    async fn passenger_lifecycle_callbacks_and_leash_time() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);
        let rider = test_happy_ghast(&folder, 0);

        ghast.on_passenger_added(rider.as_ref(), true);
        // Since no player is standing above, timeout is reset to 0
        assert_eq!(ghast.server_still_timeout.load(Ordering::Relaxed), 0);

        ghast.mob_entity.set_home_to(BlockPos::new(0, 64, 0), 16);
        assert!(ghast.mob_entity.has_position_target());

        ghast.on_passenger_removed(rider.as_ref(), true);
        // Removing passenger sets still timeout to 10 and clears home when empty
        assert_eq!(ghast.server_still_timeout.load(Ordering::Relaxed), 10);
        assert!(!ghast.mob_entity.has_position_target());

        // Leash holder time 1 becomes 0 and sets leash holder flag to false in tick
        ghast.leash_holder_time.store(1, Ordering::Relaxed);
        ghast.mob_tick(ghast.as_ref());
        assert_eq!(ghast.leash_holder_time.load(Ordering::Relaxed), 0);
        assert!(!ghast.is_leash_holder.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn is_in_clouds_checks_dimension() {
        let folder = tempfile::tempdir().unwrap();
        let ghast = test_happy_ghast(&folder, 0);

        // At y = 64, ghast is not in clouds
        ghast.get_entity().set_pos(Vector3::new(0.0, 64.0, 0.0));
        assert!(!ghast.get_entity().is_in_clouds());

        // At cloud height (192.33 in overworld, ghast height is 4.0, 191.0 + 4.0 = 195.0 >= 192.33 and 191 <= 196.33)
        ghast.get_entity().set_pos(Vector3::new(0.0, 191.0, 0.0));
        assert!(ghast.get_entity().is_in_clouds());
    }
}
