use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, PoisonError};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data::abstract_cube_mob::ID_SIZE;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::subtract_angles;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::ai::goal::active_target::ActiveTargetGoal;
use crate::entity::ai::goal::{Controls, Goal, to_goal_ticks};
use crate::entity::custom_sound::CustomSound;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};

pub const MIN_SIZE: i32 = 1;
pub const MAX_SIZE: i32 = 127;

/// A sound with a separate variant for tiny cubes.
pub struct SizedSound {
    pub normal: Sound,
    pub small: Sound,
}

impl SizedSound {
    const fn get(&self, tiny: bool) -> Sound {
        if tiny { self.small } else { self.normal }
    }
}

pub struct CubeSounds {
    pub jump: SizedSound,
    pub squish: SizedSound,
    pub hurt: SizedSound,
    pub death: SizedSound,
}

/// Which cubes hurt players they touch, from each `canDealDamage` override.
pub enum ContactDamage {
    Never,
    UnlessTiny,
    Always,
}

/// An attribute and its base value for a given actual size.
pub type SizeAttribute = (Attributes, fn(i32) -> f64);

/// What each vanilla `AbstractCubeMob` subclass overrides.
pub struct CubeKind {
    pub sounds: CubeSounds,
    pub sound_category: SoundCategory,
    /// `setCubeMobHealth`.
    pub health: fn(i32) -> f64,
    /// Attributes the subclass `setSize` derives from the actual size.
    pub size_attributes: &'static [SizeAttribute],
    pub jump_delay_multiplier: i32,
    pub contact_damage: ContactDamage,
    /// `getSplitCount`.
    pub split_count: fn() -> i32,
    /// `setSpawnSize`.
    pub spawn_size: fn() -> i32,
    /// `addBehaviourGoals` and `addTargetingGoals`.
    pub register_goals: fn(&MobEntity),
}

/// Vanilla `AbstractCubeMob.setSpawnSize`, without the difficulty multiplier.
#[must_use]
pub fn random_spawn_size() -> i32 {
    let mut size_scale = rand::random_range(0..3);
    if size_scale < 2 && rand::random_range(0.0..1.0) < 0.5 {
        size_scale += 1;
    }
    1 << size_scale
}

#[must_use]
pub fn random_split_count() -> i32 {
    2 + rand::random_range(0..3)
}

/// Shared `addBehaviourGoals` and `addTargetingGoals` of slimes and magma cubes.
pub fn register_hostile_goals(mob: &MobEntity) {
    mob.goals_selector
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .add_goal(2, Box::new(CubeMobAttackGoal::default()));
    let mut targets = mob
        .target_selector
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    targets.add_goal(
        1,
        ActiveTargetGoal::with_default(mob, &EntityType::PLAYER, true),
    );
    targets.add_goal(
        3,
        ActiveTargetGoal::with_default(mob, &EntityType::IRON_GOLEM, true),
    );
}

/// Vanilla `AbstractCubeMob`: slimes, magma cubes and sulfur cubes.
pub struct CubeMobEntity {
    pub mob_entity: MobEntity,
    pub kind: &'static CubeKind,
    jump_delay: AtomicI32,
    target_yaw: AtomicCell<f32>,
    is_aggressive: AtomicBool,
    was_on_ground: AtomicBool,
    pub squish: AtomicCell<f32>,
    pub target_squish: AtomicCell<f32>,
    pub o_squish: AtomicCell<f32>,
    speed_modifier: AtomicCell<f64>,
    has_split: AtomicBool,
}

impl CubeMobEntity {
    pub fn new(entity: Entity, kind: &'static CubeKind) -> Arc<Self> {
        entity.synched_data.define(ID_SIZE, MIN_SIZE);
        let cube = Arc::new(Self {
            mob_entity: MobEntity::new(entity),
            kind,
            jump_delay: AtomicI32::new(0),
            target_yaw: AtomicCell::new(0.0),
            is_aggressive: AtomicBool::new(false),
            was_on_ground: AtomicBool::new(false),
            squish: AtomicCell::new(0.0),
            target_squish: AtomicCell::new(0.0),
            o_squish: AtomicCell::new(0.0),
            speed_modifier: AtomicCell::new(0.0),
            has_split: AtomicBool::new(false),
        });
        cube.register_goals();
        cube.set_size((kind.spawn_size)(), true);
        cube
    }

    fn register_goals(&self) {
        let mob = &self.mob_entity;
        *mob.move_control
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Box::new(CubeMobMoveControl);
        let mut goals = mob
            .goals_selector
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        goals.add_goal(1, Box::new(CubeMobFloatGoal));
        goals.add_goal(4, Box::new(CubeMobRandomDirectionGoal::default()));
        goals.add_goal(5, Box::new(CubeMobKeepOnJumpingGoal));
        drop(goals);
        (self.kind.register_goals)(mob);
    }

    pub fn set_size(&self, size: i32, update_health: bool) {
        let actual_size = size.clamp(MIN_SIZE, MAX_SIZE);
        let living_entity = &self.mob_entity.living_entity;
        let entity = &living_entity.entity;
        entity.data.store(actual_size, Ordering::Relaxed);
        entity.set_synced_data(ID_SIZE, actual_size);

        let mut attributes = living_entity
            .attributes
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        let mut set_base = |attribute: &Attributes, value: f64| {
            if let Some(instance) = attributes.get_mut(&attribute.id) {
                instance.base_value = value;
                instance.dirty.store(true, Ordering::Relaxed);
            }
        };
        set_base(&Attributes::MAX_HEALTH, (self.kind.health)(actual_size));
        set_base(
            &Attributes::MOVEMENT_SPEED,
            f64::from(0.2 + 0.1 * actual_size as f32),
        );
        for (attribute, value) in self.kind.size_attributes {
            set_base(attribute, value(actual_size));
        }
        drop(attributes);

        if update_health {
            let max_health = living_entity.get_attribute_value(&Attributes::MAX_HEALTH) as f32;
            living_entity.health.store(max_health);
        }

        let scaled_dimensions = EntityDimensions {
            width: entity.entity_type.dimension[0] * actual_size as f32,
            height: entity.entity_type.dimension[1] * actual_size as f32,
            eye_height: entity.entity_type.eye_height * actual_size as f32,
        };
        entity.entity_dimension.store(scaled_dimensions);
        let pos = entity.pos.load();
        entity.bounding_box.store(BoundingBox::new_from_pos(
            pos.x,
            pos.y,
            pos.z,
            &scaled_dimensions,
        ));
    }

    pub fn get_size(&self) -> i32 {
        self.mob_entity
            .living_entity
            .entity
            .data
            .load(Ordering::Relaxed)
    }

    pub fn is_tiny(&self) -> bool {
        self.get_size() <= MIN_SIZE
    }

    fn can_deal_damage(&self) -> bool {
        let allowed = match self.kind.contact_damage {
            ContactDamage::Never => false,
            ContactDamage::UnlessTiny => !self.is_tiny(),
            ContactDamage::Always => true,
        };
        allowed && !self.mob_entity.is_no_ai()
    }

    fn get_jump_delay(&self) -> i32 {
        rand::random_range(10..30) * self.kind.jump_delay_multiplier
    }

    fn get_sound_volume(&self) -> f32 {
        0.4 * self.get_size() as f32
    }

    fn get_sound_pitch(&self) -> f32 {
        let pitch_adjuster = if self.is_tiny() { 1.4 } else { 0.8 };
        ((rand::random_range(0.0..1.0) - rand::random_range(0.0..1.0)) * 0.2 + 1.0) * pitch_adjuster
    }

    fn play_sound(&self, sound: Sound, pitch: f32) {
        let entity = &self.mob_entity.living_entity.entity;
        entity.world.load().play_sound_fine(
            sound,
            self.kind.sound_category,
            &entity.pos.load(),
            self.get_sound_volume(),
            pitch,
        );
    }

    /// Spawns the smaller cubes, like vanilla `AbstractCubeMob.remove`.
    fn split(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        let half_size = self.get_size() / 2;
        let xz_offset = entity.entity_dimension.load().width / 2.0;
        for i in 0..(self.kind.split_count)() {
            let xd = ((i % 2) as f32 - 0.5) * xz_offset;
            let zd = ((i / 2) as f32 - 0.5) * xz_offset;
            let child_entity = Entity::new(
                world.clone(),
                Vector3::new(pos.x + f64::from(xd), pos.y + 0.5, pos.z + f64::from(zd)),
                entity.entity_type,
            );
            child_entity.yaw.store(rand::random_range(0.0..360.0));
            let child = Self::new(child_entity, self.kind);
            child.set_size(half_size, true);
            self.copy_split_state(&child);
            world.spawn_entity_non_save(child);
        }
    }

    /// Part of vanilla `ConversionType.convertCommon` for `SPLIT_ON_DEATH`.
    fn copy_split_state(&self, child: &Self) {
        let from = &self.mob_entity;
        let to = &child.mob_entity;
        let from_entity = &from.living_entity.entity;
        let to_entity = &to.living_entity.entity;
        to.set_no_ai(from.is_no_ai());
        if from.persistence_required.load(Ordering::Relaxed) {
            to.persistence_required.store(true, Ordering::Relaxed);
        }
        if let Some(custom_name) = &**from_entity.custom_name.load() {
            to_entity.set_custom_name(custom_name.clone());
        }
        to_entity.set_custom_name_visible(from_entity.custom_name_visible.load(Ordering::Relaxed));
        to_entity.invulnerable.store(
            from_entity.invulnerable.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
    }
}

impl CustomSound for CubeMobEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(self.kind.sounds.death.get(self.is_tiny()))
    }
    fn hurt_sound(&self) -> Option<Sound> {
        Some(self.kind.sounds.hurt.get(self.is_tiny()))
    }
}

impl Mob for CubeMobEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("Size", self.get_size() - 1);
        nbt.put_bool("wasOnGround", self.was_on_ground.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        // Wraps like Java's int addition, so i32::MAX clamps to the minimum size.
        self.set_size(nbt.get_int("Size").unwrap_or(0).wrapping_add(1), false);
        self.was_on_ground.store(
            nbt.get_bool("wasOnGround").unwrap_or(false),
            Ordering::Relaxed,
        );
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        // Cleared before AI runs so the float goal's jump survives the move control.
        self.mob_entity
            .living_entity
            .jumping
            .store(false, Ordering::SeqCst);
        self.o_squish.store(self.squish.load());
        self.squish
            .store(self.squish.load() + (self.target_squish.load() - self.squish.load()) * 0.5);

        let on_ground = self
            .mob_entity
            .living_entity
            .entity
            .on_ground
            .load(Ordering::Relaxed);
        let was_on_ground = self.was_on_ground.load(Ordering::Relaxed);
        if on_ground && !was_on_ground {
            // Landing particles are client-side: vanilla's Level.addParticle is a no-op on the server.
            self.play_sound(
                self.kind.sounds.squish.get(self.is_tiny()),
                ((rand::random_range(0.0..1.0) - rand::random_range(0.0..1.0)) * 0.2 + 1.0) / 0.8,
            );
            self.target_squish.store(-0.5);
        } else if !on_ground && was_on_ground {
            self.target_squish.store(1.0);
        }
        self.was_on_ground.store(on_ground, Ordering::Relaxed);
        self.target_squish.store(self.target_squish.load() * 0.6);
        self.is_aggressive.store(false, Ordering::Relaxed);
        self.speed_modifier.store(0.0);
    }

    fn post_tick(&self) {
        if self.mob_entity.living_entity.dead.load(Ordering::Relaxed)
            && self.get_size() > MIN_SIZE
            && self
                .has_split
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
        {
            self.split();
        }
    }

    fn mob_player_collision(&self, player: &Arc<Player>) {
        if self.can_deal_damage() {
            self.mob_entity.try_attack(self, &**player);
        }
    }
}

fn as_cube(mob: &dyn Mob) -> Option<&CubeMobEntity> {
    mob.cast_any().downcast_ref()
}

struct CubeMobMoveControl;

impl Control for CubeMobMoveControl {}

impl MoveControlTrait for CubeMobMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let Some(cube) = as_cube(mob) else {
            return;
        };
        let living_entity = &cube.mob_entity.living_entity;
        let entity = &living_entity.entity;
        let new_yaw = self.change_angle(entity.yaw.load(), cube.target_yaw.load(), 90.0);
        entity.yaw.store(new_yaw);
        entity.head_yaw.store(new_yaw);
        entity.body_yaw.store(new_yaw);

        let speed_modifier = cube.speed_modifier.load();
        let mut movement_input = Vector3::new(0.0, 0.0, 0.0);
        if entity.on_ground.load(Ordering::Relaxed) {
            if speed_modifier > 0.0 {
                let current_delay = cube.jump_delay.load(Ordering::Relaxed);
                if current_delay <= 0 {
                    let mut next_delay = cube.get_jump_delay();
                    if cube.is_aggressive.load(Ordering::Relaxed) {
                        next_delay /= 3;
                    }
                    cube.jump_delay.store(next_delay, Ordering::Relaxed);
                    living_entity.jumping.store(true, Ordering::SeqCst);
                    cube.play_sound(
                        cube.kind.sounds.jump.get(cube.is_tiny()),
                        cube.get_sound_pitch(),
                    );
                    movement_input.z = speed_modifier;
                } else {
                    cube.jump_delay.store(current_delay - 1, Ordering::Relaxed);
                }
            }
        } else if speed_modifier > 0.0 {
            movement_input.z = speed_modifier;
        }
        living_entity.movement_input.store(movement_input);
    }
}

struct CubeMobFloatGoal;

impl Goal for CubeMobFloatGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        entity.touching_water.load(Ordering::Relaxed)
            || entity.touching_lava.load(Ordering::Relaxed)
    }
    fn tick(&mut self, mob: &dyn Mob) {
        let Some(cube) = as_cube(mob) else {
            return;
        };
        if rand::random_range(0.0..1.0) < 0.8 {
            cube.mob_entity
                .living_entity
                .jumping
                .store(true, Ordering::SeqCst);
        }
        cube.speed_modifier.store(1.2);
    }
    fn should_run_every_tick(&self) -> bool {
        true
    }
    fn controls(&self) -> Controls {
        Controls::JUMP | Controls::MOVE
    }
}

#[derive(Default)]
struct CubeMobAttackGoal {
    grow_tired_timer: i32,
}

impl Goal for CubeMobAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        mob.get_mob_entity()
            .get_target()
            .is_some_and(|target| mob.can_attack(&*target))
    }
    fn start(&mut self, _mob: &dyn Mob) {
        self.grow_tired_timer = to_goal_ticks(300);
    }
    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        if !self.can_start(mob) {
            return false;
        }
        self.grow_tired_timer -= 1;
        self.grow_tired_timer > 0
    }
    fn tick(&mut self, mob: &dyn Mob) {
        let Some(cube) = as_cube(mob) else {
            return;
        };
        let yaw = cube.mob_entity.living_entity.entity.yaw.load();
        let mut new_yaw = yaw;
        if let Some(target) = cube.mob_entity.get_target() {
            let target_pos = target.get_entity().pos.load();
            let pos = cube.mob_entity.living_entity.entity.pos.load();
            // Vanilla Mob.lookAt(target, 10, 10).
            let look_yaw = ((target_pos.z - pos.z)
                .atan2(target_pos.x - pos.x)
                .to_degrees()
                - 90.0) as f32;
            new_yaw = yaw + subtract_angles(yaw, look_yaw).clamp(-10.0, 10.0);
            cube.mob_entity.living_entity.entity.yaw.store(new_yaw);
        }
        cube.target_yaw.store(new_yaw);
        cube.is_aggressive
            .store(cube.can_deal_damage(), Ordering::Relaxed);
    }
    fn should_run_every_tick(&self) -> bool {
        true
    }
    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}

#[derive(Default)]
struct CubeMobRandomDirectionGoal {
    chosen_degrees: f32,
    next_randomize_time: i32,
}

impl Goal for CubeMobRandomDirectionGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        mob_entity.get_target().is_none()
            && (entity.on_ground.load(Ordering::Relaxed)
                || entity.touching_water.load(Ordering::Relaxed)
                || entity.touching_lava.load(Ordering::Relaxed)
                || mob_entity
                    .living_entity
                    .has_effect(&StatusEffect::LEVITATION))
    }
    fn tick(&mut self, mob: &dyn Mob) {
        let Some(cube) = as_cube(mob) else {
            return;
        };
        self.next_randomize_time -= 1;
        if self.next_randomize_time <= 0 {
            self.next_randomize_time = self.get_tick_count(40 + rand::random_range(0..60));
            self.chosen_degrees = rand::random_range(0..360) as f32;
        }
        cube.target_yaw.store(self.chosen_degrees);
        cube.is_aggressive.store(false, Ordering::Relaxed);
    }
    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}

struct CubeMobKeepOnJumpingGoal;

impl Goal for CubeMobKeepOnJumpingGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        !mob.get_entity().has_vehicle()
    }
    fn tick(&mut self, mob: &dyn Mob) {
        if let Some(cube) = as_cube(mob) {
            cube.speed_modifier.store(1.0);
        }
    }
    fn controls(&self) -> Controls {
        Controls::JUMP | Controls::MOVE
    }
}
