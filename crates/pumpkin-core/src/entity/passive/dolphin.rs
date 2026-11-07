use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::ageable_water_creature::AgeableWaterCreature;
use crate::entity::ageable::{AgeableData, AgeableMob};
use crate::entity::custom_sound::CustomSound;
use crate::world::World;

use crate::entity::ai::behavior::fleeing::{self, Flee};
use crate::entity::{
    Entity, EntityBase,
    ai::control::{
        smooth_swimming_look_control::SmoothSwimmingLookControl,
        smooth_swimming_move_control::SmoothSwimmingMoveControl,
    },
    ai::goal::{breath_air::BreathAirGoal, revenge::RevengeGoal, try_find_water::TryFindWaterGoal},
    ai::pathfinder::{Navigator, node::PathType},
    ai::target_match::TargetMatch,
    mob::{Mob, MobEntity},
    player::Player,
};

mod goals;
const TOTAL_AIR_SUPPLY: i32 = 4800;
const TOTAL_MOISTNESS_LEVEL: i32 = 2400;
const IN_WATER_SPEED_MODIFIER: f32 = 0.02;
const OUTSIDE_WATER_SPEED_MODIFIER: f32 = 0.1;
const SWIM_WITH_PLAYER_SPEED: f64 = 4.0;
const RIDDEN_FOLLOW_GOAL_COUNT: f64 = 2.0;
// Air seeking, both ridden followers and normal travel can each move once.
const MAX_MOVES_PER_TICK: f64 = 4.0;

const DOLPHIN_FLEES: &[Flee] = &[Flee::new(9, TargetMatch::GUARDIAN_LIKE, 8.0, 1.0, 1.0)];

pub struct DolphinEntity {
    pub mob_entity: MobEntity,
    pub got_fish: AtomicBool,
    pub moistness_level: AtomicI32,
    ageable_data: AgeableData,
    swimming_player: Arc<AtomicCell<Option<i32>>>,
    boat_player: Arc<AtomicCell<Option<i32>>>,
    nautilus_player: Arc<AtomicCell<Option<i32>>>,
}

impl DolphinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        entity.air_supply.store(TOTAL_AIR_SUPPLY, Ordering::Relaxed);
        let mut mob_entity = MobEntity::new(entity);
        *mob_entity
            .move_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(SmoothSwimmingMoveControl::new(
                85,
                10,
                IN_WATER_SPEED_MODIFIER,
                OUTSIDE_WATER_SPEED_MODIFIER,
                true,
            ));
        *mob_entity
            .look_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(SmoothSwimmingLookControl::new(10));
        *mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Navigator::water_bound(true);
        mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_pathfinding_malus(PathType::Water, 0.0);
        mob_entity.set_can_pick_up_loot(true);
        let dolphin = Self {
            mob_entity,
            got_fish: AtomicBool::new(false),
            moistness_level: AtomicI32::new(TOTAL_MOISTNESS_LEVEL),
            ageable_data: AgeableData::default(),
            swimming_player: Arc::new(AtomicCell::new(None)),
            boat_player: Arc::new(AtomicCell::new(None)),
            nautilus_player: Arc::new(AtomicCell::new(None)),
        };
        let mob_arc = Arc::new(dolphin);
        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(BreathAirGoal));
            goal_selector.add_goal(
                0,
                Box::new(TryFindWaterGoal::with_fluid_tag(
                    &tag::Fluid::MINECRAFT_DOLPHIN_TRIES_TO_FIND,
                )),
            );
            goals::register(&mob_arc);
            fleeing::apply(goal_selector, DOLPHIN_FLEES);
        };

        {
            let target_selector = &mob_arc.mob_entity.target_selector;

            target_selector.add_goal(
                1,
                Box::new(
                    RevengeGoal::new(true)
                        .ignoring(|entity_type| TargetMatch::GUARDIAN_LIKE.matches(entity_type))
                        .alerting_others(),
                ),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn got_fish(&self) -> bool {
        self.got_fish.load(Ordering::Relaxed)
    }

    pub fn set_got_fish(&self, val: bool) {
        self.got_fish.store(val, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::dolphin::GOT_FISH, val);
    }

    fn set_moistness_level(&self, level: i32) {
        self.moistness_level.store(level, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::dolphin::MOISTNESS_LEVEL, VarInt(level));
    }
}

impl AgeableMob for DolphinEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn default_baby_spawn_chance(&self) -> f32 {
        0.1
    }
}

impl AgeableWaterCreature for DolphinEntity {}

impl CustomSound for DolphinEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityDolphinDeath)
    }
    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityDolphinHurt)
    }
    fn sound_category(&self) -> SoundCategory {
        SoundCategory::Neutral
    }
}

impl Mob for DolphinEntity {
    fn has_global_tick_reach(&self) -> bool {
        let entity = self.get_entity();
        entity.has_vehicle()
            || entity.has_passengers()
            || entity
                .leashed_to
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
    }

    fn additional_tick_movement(&self) -> f64 {
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        let velocity = entity.velocity.load().length();
        let multiplier = entity.movement_multiplier.load();
        let multiplier = multiplier
            .x
            .abs()
            .max(multiplier.y.abs())
            .max(multiplier.z.abs())
            .max(1.0);
        let navigation_speed = self
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_speed_modifier()
            .abs()
            .max(SWIM_WITH_PLAYER_SPEED);
        let speed = (navigation_speed
            * living
                .get_attribute_value(&Attributes::MOVEMENT_SPEED)
                .abs()) as f32;
        let control_speed = f64::from((speed * OUTSIDE_WATER_SPEED_MODIFIER).abs())
            .max(f64::from(living.get_speed().abs()));
        let travel_acceleration = crate::entity::living::LivingEntity::friction_influenced_speed(
            control_speed,
            crate::entity::living::LivingEntity::FRICTION_SPEED_THRESHOLD.next_up(),
        )
        .max(control_speed)
        .max(crate::entity::living::LivingEntity::DEFAULT_OFF_GROUND_SPEED);
        let max_velocity = velocity
            + goals::JUMP_HORIZONTAL_SPEED.hypot(goals::JUMP_VERTICAL_SPEED)
            + f64::from(BreathAirGoal::MOVEMENT_SPEED)
            + RIDDEN_FOLLOW_GOAL_COUNT * f64::from(goals::FOLLOW_ENTITY_MOVEMENT_SPEED);
        (MAX_MOVES_PER_TICK * max_velocity
            + travel_acceleration
            + SmoothSwimmingMoveControl::BUOYANCY)
            * multiplier
            - velocity
    }

    fn player_tick_range(&self) -> f64 {
        10.0 * f64::from(pumpkin_data::data_component_impl::MobVisibilityImpl::MAX_VISIBILITY)
    }

    fn tick_links(&self) -> Vec<i32> {
        [
            self.swimming_player.load(),
            self.boat_player.load(),
            self.nautilus_player.load(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
    fn pick_up_item(&self, item: &crate::entity::item::ItemEntity) {
        let living = &self.mob_entity.living_entity;
        if !living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND)
            .is_empty()
        {
            return;
        }
        let count = item
            .get_item_stack()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .item_count;
        if !living.pickup(item.get_entity(), u32::from(count)) {
            return;
        }
        living.on_item_pickup(item);
        let taken = std::mem::replace(
            &mut *item
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            ItemStack::EMPTY.clone(),
        );
        self.mob_entity
            .set_item_slot_and_drop_when_killed(&EquipmentSlot::MAIN_HAND, taken);
        item.get_entity().remove();
    }

    fn on_attack(&self, _target: &dyn EntityBase) {
        self.play_entity_sound(Sound::EntityDolphinAttack, 1.0, 1.0);
    }
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn check_spawn_obstruction(&self, world: &World) -> bool {
        self.water_creature_spawn_obstruction(world)
    }

    fn get_base_experience_reward(&self) -> u32 {
        self.water_creature_experience_reward()
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        super::water_animal::AMBIENT_SOUND_INTERVAL
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(if self.get_entity().is_in_water() {
            Sound::EntityDolphinAmbientWater
        } else {
            Sound::EntityDolphinAmbient
        })
    }

    fn mob_is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn get_max_look_pitch_change(&self) -> f32 {
        1.0
    }

    fn get_max_head_rotation(&self) -> f32 {
        1.0
    }

    fn can_attack(&self, target: &dyn EntityBase) -> bool {
        !AgeableMob::is_baby(self) && self.can_attack_base(target)
    }

    fn can_dispenser_equip_into_slot(&self, slot: &EquipmentSlot) -> bool {
        *slot == EquipmentSlot::MAIN_HAND && self.mob_entity.can_pick_up_loot()
    }

    fn finalize_spawn(
        &self,
        _world: &Arc<World>,
        _reason: crate::entity::mob::spawn::EntitySpawnReason,
        group_data: Option<crate::entity::mob::spawn::SpawnGroupData>,
    ) -> Option<crate::entity::mob::spawn::SpawnGroupData> {
        self.get_entity().set_air_supply(TOTAL_AIR_SUPPLY);
        self.get_entity().set_pitch(0.0);
        self.finalize_ageable_spawn(group_data)
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(
            tracked_data::ageable_mob::DATA_BABY_ID,
            AgeableMob::is_baby(self),
        );
        entity.set_synced_data(tracked_data::ageable_mob::AGE_LOCKED, self.is_age_locked());
        entity.set_synced_data(tracked_data::dolphin::GOT_FISH, self.got_fish());
        entity.set_synced_data(
            tracked_data::dolphin::MOISTNESS_LEVEL,
            VarInt(self.moistness_level.load(Ordering::Relaxed)),
        );
    }

    fn mob_max_air_supply(&self) -> i32 {
        TOTAL_AIR_SUPPLY
    }

    fn mob_increase_air_supply(&self, _current: i32) -> i32 {
        TOTAL_AIR_SUPPLY
    }

    fn base_tick(&self) {
        self.mob_entity.living_entity.tick_air_supply(self);
    }

    fn post_tick(&self) {
        let entity = self.get_entity();
        if self.mob_entity.is_no_ai() {
            entity.set_air_supply(TOTAL_AIR_SUPPLY);
        } else if entity.is_in_water_or_rain() {
            self.set_moistness_level(TOTAL_MOISTNESS_LEVEL);
        } else {
            let moistness = self.moistness_level.load(Ordering::Relaxed).wrapping_sub(1);
            self.set_moistness_level(moistness);
            if moistness <= 0 {
                self.damage(self, 1.0, DamageType::DRY_OUT);
            }
            if entity.on_ground.load(Ordering::Relaxed) {
                let mut rng = self.get_random();
                entity.velocity.store(
                    entity.velocity.load()
                        + Vector3::new(
                            f64::from((rng.random::<f32>() * 2.0 - 1.0) * 0.2),
                            0.5,
                            f64::from((rng.random::<f32>() * 2.0 - 1.0) * 0.2),
                        ),
                );
                entity.yaw.store(rng.random::<f32>() * 360.0);
                entity.on_ground.store(false, Ordering::Relaxed);
                entity.velocity_dirty.store(true, Ordering::Relaxed);
            }
        }
    }

    fn travel_in_water(&self, caller: &dyn EntityBase, _gravity: f64, _falling: bool) {
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        entity.update_velocity_from_input(
            living.movement_input.load(),
            f64::from(living.get_speed()),
        );
        living.make_move(caller);
        let mut velocity = entity.velocity.load() * 0.9;
        if self
            .mob_entity
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_none()
        {
            velocity.y -= 0.005;
        }
        entity.velocity.store(velocity);
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("GotFish", self.got_fish());
        nbt.put_int("Moistness", self.moistness_level.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_got_fish(nbt.get_bool("GotFish").unwrap_or(false));
        self.set_moistness_level(nbt.get_int("Moistness").unwrap_or(TOTAL_MOISTNESS_LEVEL));
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let item = item_stack.get_item();
        if !item_stack.is_empty() && item.has_tag(&tag::Item::MINECRAFT_FISHES) {
            self.play_entity_sound(Sound::EntityDolphinEat, 1.0, 1.0);
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            if self.can_age_up() {
                self.age_up(
                    crate::entity::ageable::get_speed_up_seconds_when_feeding(
                        self.get_age().wrapping_neg(),
                    ),
                    true,
                );
            } else {
                self.set_got_fish(true);
            }
            return true;
        }

        self.mob_entity
            .mob_interact(player, item_stack, self.can_be_leashed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::goal::Goal;
    use crate::entity::ai::test_support::world;
    use pumpkin_data::effect::StatusEffect;
    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector2::Vector2;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn conservative_body(
        dolphin: &DolphinEntity,
    ) -> Result<pumpkin_util::math::boundingbox::BoundingBox, Box<dyn std::error::Error>> {
        let crate::world::entity_islands::EntityReach::Conservative(reach) =
            crate::world::entity_islands::EntityReach::of(
                dolphin,
                &pumpkin_config::world::EntityIslandsConfig::default(),
            )
        else {
            return Err("dolphin unexpectedly has audited reach".into());
        };
        Ok(reach.body)
    }

    #[tokio::test]
    async fn air_seeking_extra_move_stays_inside_reported_body_at_high_velocity() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let dolphin = DolphinEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::DOLPHIN,
        ));
        let entity = dolphin.get_entity();
        entity.velocity.store(Vector3::new(96.0, 0.0, 0.0));
        entity.air_supply.store(139, Ordering::Relaxed);
        let body = conservative_body(dolphin.as_ref())?;
        BreathAirGoal.tick(dolphin.as_ref());
        dolphin.travel_in_water(dolphin.as_ref(), 0.08, false);
        let actual = entity.bounding_box.load();
        assert!(entity.pos.load().x > 190.0);
        assert!(actual.min.x >= body.min.x && actual.max.x <= body.max.x);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn fast_swim_control_and_movement_multiplier_stay_inside_reported_body() -> TestResult {
        use crate::entity::ai::pathfinder::{node::Node, path::Path};
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let dolphin = DolphinEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::DOLPHIN,
        ));
        let living = &dolphin.mob_entity.living_entity;
        let entity = &living.entity;
        living.set_attribute_base(&Attributes::MOVEMENT_SPEED, 1024.0);
        entity.touching_water.store(true, Ordering::Relaxed);
        entity
            .movement_multiplier
            .store(Vector3::new(2.0, 2.0, 2.0));
        let target = pumpkin_util::math::position::BlockPos::new(0, 64, 8);
        dolphin
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .move_to_path(
                Some(Path::new(vec![Node::new(target)], target, true)),
                4.0,
                living,
            );
        dolphin
            .mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_wanted_position(0.5, 64.0, 8.5, 4.0);
        let body = conservative_body(dolphin.as_ref())?;
        dolphin
            .mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .tick(dolphin.as_ref());
        dolphin.travel_in_water(dolphin.as_ref(), 0.08, false);
        let actual = entity.bounding_box.load();
        assert!(entity.pos.load().z > 160.0);
        assert!(actual.min.z >= body.min.z && actual.max.z <= body.max.z);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn feeding_growth_wraps_like_java_for_the_smallest_saved_age() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let dolphin = DolphinEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::DOLPHIN,
        ));
        dolphin.set_age(i32::MIN);
        let seconds = crate::entity::ageable::get_speed_up_seconds_when_feeding(
            dolphin.get_age().wrapping_neg(),
        );
        dolphin.age_up(seconds, true);
        assert_eq!(dolphin.get_age(), i32::MIN);
        assert_eq!(
            dolphin.ageable_data.forced_age.load(Ordering::Relaxed),
            i32::MIN
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn loading_dolphin_nbt_resets_missing_fields_and_preserves_saved_air() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let dolphin = DolphinEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::DOLPHIN,
        ));
        let mut saved = NbtCompound::new();
        saved.put_bool("GotFish", true);
        saved.put_int("Moistness", 17);
        saved.put_int("Age", -100);
        saved.put_short("Air", 123);
        EntityBase::read_nbt_non_mut(dolphin.as_ref(), &saved);
        let mut written = NbtCompound::new();
        dolphin.mob_write_nbt(&mut written);
        assert_eq!(written.get_int("Moistness"), Some(17));
        assert_eq!(written.get_bool("GotFish"), Some(true));
        assert_eq!(dolphin.get_entity().air_supply.load(Ordering::Relaxed), 123);
        assert!(AgeableMob::is_baby(dolphin.as_ref()));
        EntityBase::read_nbt_non_mut(dolphin.as_ref(), &NbtCompound::new());
        assert!(!dolphin.got_fish());
        assert_eq!(dolphin.moistness_level.load(Ordering::Relaxed), 2400);
        assert_eq!(
            dolphin.get_entity().air_supply.load(Ordering::Relaxed),
            4800
        );
        assert!(!AgeableMob::is_baby(dolphin.as_ref()));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn dolphin_air_distinguishes_nautilus_breathing_and_no_ai() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let chunk = pumpkin_world::chunk::ChunkData::empty_sync(0, 0);
        chunk.set_block_absolute_y(0, 64, 0, pumpkin_data::Block::WATER.default_state.id);
        world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
        let dolphin = DolphinEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::DOLPHIN,
        ));
        let living = &dolphin.mob_entity.living_entity;
        dolphin.base_tick();
        assert_eq!(
            dolphin.get_entity().air_supply.load(Ordering::Relaxed),
            4799
        );
        living.add_effect(pumpkin_data::potion::Effect {
            effect_type: &StatusEffect::BREATH_OF_THE_NAUTILUS,
            duration: 100,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        });
        dolphin.base_tick();
        assert_eq!(
            dolphin.get_entity().air_supply.load(Ordering::Relaxed),
            4799
        );
        living.add_effect(pumpkin_data::potion::Effect {
            effect_type: &StatusEffect::WATER_BREATHING,
            duration: 100,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        });
        dolphin.base_tick();
        assert_eq!(
            dolphin.get_entity().air_supply.load(Ordering::Relaxed),
            4800
        );
        dolphin
            .get_entity()
            .air_supply
            .store(100, Ordering::Relaxed);
        dolphin.moistness_level.store(17, Ordering::Relaxed);
        dolphin.mob_entity.set_no_ai(true);
        dolphin.post_tick();
        assert_eq!(
            dolphin.get_entity().air_supply.load(Ordering::Relaxed),
            4800
        );
        assert_eq!(dolphin.moistness_level.load(Ordering::Relaxed), 17);
        world.level.shutdown().await;
        Ok(())
    }
}
