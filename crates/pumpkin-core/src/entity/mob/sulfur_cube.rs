use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex, Weak};

use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    BucketEntityDataImpl, DataComponentImpl, EquipmentSlot, Operation, SulfurCubeContentImpl,
};
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::sulfur_cube_archetype::{
    SulfurCubeArchetype, SulfurCubeContactDamage, SulfurCubeExplosion,
    SulfurCubeKnockbackModifiers, SulfurCubeSoundSettings,
};
use pumpkin_data::tag::Taggable;
use pumpkin_data::tracked_data;
use pumpkin_data::translation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::EntityDimensions;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{self, clamped_map, wrap_degrees_90};
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomGenerator, get_seed};
use pumpkin_util::text::TextComponent;
use rand::RngExt;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::entity::ageable::{AgeableData, AgeableMob, get_speed_up_seconds_when_feeding};
use crate::entity::ai::control::look_control::LookControl;
use crate::entity::ai::control::{Control, LookControlTrait, MoveControlTrait};
use crate::entity::ai::goal::tempt::TemptGoal;
use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::attributes::{Modifier, ModifierOperation};
use crate::entity::bucketable::Bucketable;
use crate::entity::custom_sound::CustomSound;
use crate::entity::item::ItemEntity;
use crate::entity::mob::cube_mob::{
    CubeMob, CubeMobData, CubeMobFloatGoal, CubeMobKeepOnJumpingGoal, CubeMobMoveControl,
    CubeMobRandomDirectionGoal, tick_cube_mob,
};
use crate::entity::mob::spawn::{EntitySpawnReason, SpawnGroupData};
use crate::entity::mob::{Mob, MobEntity, conversion};
use crate::entity::player::advancement::trigger::AdvancementTrigger;
use crate::entity::projectile::arrow::ArrowEntity;
use crate::entity::tnt::TNTEntity;
use crate::entity::{Entity, EntityBase};
use crate::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent;
use crate::world::ExplosionInteraction;
use crate::world::World;

const PASSENGER_SINK_PER_SIZE: f64 = 0.015_625;
const PICKUP_TIMER_DURATION: i32 = 100;
const PUSH_DISTANCE_THRESHOLD: f64 = 1.3f32 as f64;
const MAX_PLAYER_PUSH_SPEED: f64 = 0.5;
const PLAYER_PUSH_SPEED_SCALE_MULTIPLIER: f32 = 0.3;
const VEHICLE_PUSH_SPEED_SCALE_MULTIPLIER: f32 = 0.16;
const VERTICAL_PUSH_MULTIPLIER: f32 = 0.3;
const HORIZONTAL_HIT_ANGLE_SCALE: f32 = 1.6;
const VERTICAL_HIT_ANGLE_SCALE: f32 = 0.5;
const VERTICAL_POSITION_ANGLE_SCALE: f32 = 0.8;
const EXTRA_KNOCKBACK_DAMPENING: f32 = 0.25;

const DEFAULT_KNOCKBACK_MODIFIERS: SulfurCubeKnockbackModifiers = SulfurCubeKnockbackModifiers {
    horizontal_power: 0.33,
    vertical_power: 0.06,
};

const DEFAULT_SOUND_SETTINGS: SulfurCubeSoundSettings = SulfurCubeSoundSettings {
    hit_sound: Sound::EntitySulfurCubeRegularHit,
    push_sound: Sound::EntitySulfurCubeRegularPush,
    push_sound_impulse_threshold: 0.2,
    push_sound_cooldown: 0.5,
};

pub const SPLIT_COUNT: i32 = 2;
pub const MAX_SIZE: i32 = 2;
pub const MIN_SIZE: i32 = 1;

struct ArchetypeState {
    floats_in_liquids: bool,
    explosion: Option<&'static SulfurCubeExplosion>,
    contact_damages: Vec<&'static SulfurCubeContactDamage>,
    knockback_modifiers: SulfurCubeKnockbackModifiers,
    sound_settings: SulfurCubeSoundSettings,
}

impl Default for ArchetypeState {
    fn default() -> Self {
        Self {
            floats_in_liquids: false,
            explosion: None,
            contact_damages: Vec::new(),
            knockback_modifiers: DEFAULT_KNOCKBACK_MODIFIERS,
            sound_settings: DEFAULT_SOUND_SETTINGS,
        }
    }
}

fn matching_archetypes(stack: &ItemStack) -> impl Iterator<Item = &'static SulfurCubeArchetype> {
    SulfurCubeArchetype::all()
        .iter()
        .copied()
        .filter(|archetype| !stack.is_empty() && stack.item.has_tag(archetype.items))
}

fn rotate((x, y): (f32, f32), angle: f32) -> (f32, f32) {
    let (sine, cosine) = (math::sin(angle), math::cos(angle));
    (x * cosine - y * sine, y * cosine + x * sine)
}

fn apply_horizontal_hit_angle_scale(
    horizontal_angle_scale: f32,
    original_angle: (f32, f32),
    attacker_position: Vector3<f64>,
    attacker_aim_direction: Vector3<f64>,
    target_center: Vector3<f64>,
) -> (f32, f32) {
    let attacker_to_target = (target_center - attacker_position).normalize();
    let angle_diff = (attacker_aim_direction.x * attacker_to_target.z
        - attacker_aim_direction.z * attacker_to_target.x)
        .atan2(
            attacker_aim_direction.x * attacker_to_target.x
                + attacker_aim_direction.z * attacker_to_target.z,
        ) as f32;
    rotate(original_angle, angle_diff * horizontal_angle_scale)
}

fn apply_vertical_hit_angle_power_transfer(
    vertical_hit_angle_scale: f32,
    horizontal_power: f32,
    vertical_power: f32,
    attacker_position: Vector3<f64>,
    attacker_aim_direction: Vector3<f64>,
    target_center: Vector3<f64>,
    target_height: f32,
) -> (f32, f32) {
    let half_height = f64::from(0.5 * target_height);
    let to_top = (target_center.add_raw(0.0, half_height, 0.0) - attacker_position).normalize();
    let to_bottom = (target_center.add_raw(0.0, -half_height, 0.0) - attacker_position).normalize();
    let factor = clamped_map(attacker_aim_direction.y, to_top.y, to_bottom.y, -1.0, 1.0) as f32;
    let mut transferred = (factor * vertical_hit_angle_scale).abs();
    if factor < 0.0 {
        transferred = -transferred;
    }
    (
        horizontal_power * (1.0 - transferred),
        vertical_power * (1.0 + transferred),
    )
}

fn apply_vertical_position_angle_power_rotation(
    vertical_position_angle_scale: f32,
    horizontal_power: f32,
    vertical_power: f32,
    original_horizontal_power: f32,
    original_vertical_power: f32,
    attacker_feet: Vector3<f64>,
    target_feet: Vector3<f64>,
) -> (f32, f32) {
    let delta = target_feet - attacker_feet;
    let angle = (-delta.y).atan2(delta.horizontal_length()) as f32;
    let (x, y) = rotate(
        (horizontal_power, vertical_power),
        -angle * vertical_position_angle_scale,
    );
    let horizontal_ratio = if original_horizontal_power > 0.0 {
        x.abs() / original_horizontal_power
    } else {
        0.0
    };
    let vertical_ratio = if original_vertical_power > 0.0 {
        y.abs() / original_vertical_power
    } else {
        0.0
    };
    let max_ratio = horizontal_ratio.max(vertical_ratio);
    if max_ratio > 1.0 {
        (x / max_ratio, y / max_ratio)
    } else {
        (x, y)
    }
}

fn is_swallowable_item(stack: &ItemStack) -> bool {
    stack
        .item
        .has_tag(&pumpkin_data::tag::Item::MINECRAFT_SULFUR_CUBE_SWALLOWABLE)
}

fn is_food(stack: &ItemStack) -> bool {
    stack
        .item
        .has_tag(&pumpkin_data::tag::Item::MINECRAFT_SULFUR_CUBE_FOOD)
}

const fn modifier_operation(operation: Operation) -> ModifierOperation {
    match operation {
        Operation::AddValue => ModifierOperation::Add,
        Operation::AddMultipliedBase => ModifierOperation::MultiplyBase,
        Operation::AddMultipliedTotal => ModifierOperation::MultiplyTotal,
    }
}

pub struct SulfurCubeEntity {
    pub mob_entity: MobEntity,
    cube: CubeMobData,
    ageable_data: AgeableData,
    pickup_timer: AtomicI32,
    push_sound_cooldown: AtomicI32,
    from_bucket: AtomicBool,
    fuse: AtomicI32,
    archetype: Mutex<ArchetypeState>,
    last_body_item: Mutex<ItemStack>,
    this: Weak<Self>,
}

impl SulfurCubeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let sulfur_cube = Arc::new_cyclic(|this| Self {
            mob_entity,
            cube: CubeMobData::default(),
            ageable_data: AgeableData::default(),
            pickup_timer: AtomicI32::new(0),
            push_sound_cooldown: AtomicI32::new(0),
            from_bucket: AtomicBool::new(false),
            fuse: AtomicI32::new(-1),
            archetype: Mutex::new(ArchetypeState::default()),
            last_body_item: Mutex::new(ItemStack::EMPTY.clone()),
            this: this.clone(),
        });

        {
            let mut move_control = sulfur_cube
                .mob_entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *move_control = Box::new(SulfurCubeMobMoveControl::new(sulfur_cube.this.clone()));
        };

        {
            let mut look_control = sulfur_cube
                .mob_entity
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *look_control = Box::new(SulfurCubeLookControl::new(sulfur_cube.this.clone()));
        };

        sulfur_cube.register_goals();
        sulfur_cube.set_size(1, false);

        sulfur_cube
    }

    fn register_goals(&self) {
        let cube: Weak<dyn CubeMob> = self.this.clone();
        let goal_selector = &self.mob_entity.goals_selector;

        goal_selector.add_goal(1, Box::new(CubeMobFloatGoal::new(cube.clone())));
        goal_selector.add_goal(
            2,
            Box::new(TemptGoal::for_cube_mob(
                1.0,
                |mob, stack| {
                    if mob.as_ageable().is_some_and(AgeableMob::is_baby) {
                        is_food(stack)
                    } else {
                        is_swallowable_item(stack)
                    }
                },
                false,
                1.0,
                cube.clone(),
            )),
        );
        goal_selector.add_goal(
            3,
            Box::new(SulfurCubeSearchForItemsGoal {
                cube: self.this.clone(),
                target_item: None,
            }),
        );
        goal_selector.add_goal(4, Box::new(CubeMobRandomDirectionGoal::new(cube.clone())));
        goal_selector.add_goal(5, Box::new(CubeMobKeepOnJumpingGoal::new(cube)));
    }

    pub fn set_size(&self, size: i32, update_health: bool) {
        let actual_size = size.clamp(1, 127);
        let entity = &self.mob_entity.living_entity.entity;
        entity.data.store(actual_size, Ordering::Relaxed);

        {
            let mut attributes = self
                .mob_entity
                .living_entity
                .attributes
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(health) = attributes.get_mut(&Attributes::MAX_HEALTH.id) {
                health.base_value = (4 * actual_size) as f64;
                health.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(speed) = attributes.get_mut(&Attributes::MOVEMENT_SPEED.id) {
                speed.base_value = (0.2 + 0.1 * actual_size as f32) as f64;
                speed.dirty.store(true, Ordering::Relaxed);
            }
        }

        if update_health {
            let max_health = self
                .mob_entity
                .living_entity
                .get_attribute_value(&Attributes::MAX_HEALTH) as f32;
            self.mob_entity.living_entity.health.store(max_health);
        }

        let scaled_dimensions = EntityDimensions {
            width: entity.entity_type.dimension[0] * actual_size as f32,
            height: entity.entity_type.dimension[1] * actual_size as f32,
            eye_height: entity.entity_type.eye_height * actual_size as f32,
        };
        entity.set_default_dimensions(scaled_dimensions);

        entity.set_synced_data(tracked_data::sulfur_cube::ID_SIZE, actual_size);
        if let Some(meta) = self.mob_metadata() {
            entity.send_bedrock_actor_data(&meta.to_bedrock_actor_data());
        }

        if update_health && size == 1 && !AgeableMob::is_baby(self) {
            AgeableMob::set_baby(self, true);
        }
    }

    pub fn is_primed(&self) -> bool {
        self.fuse.load(Ordering::Relaxed) >= 0
    }

    pub fn has_body_item(&self) -> bool {
        !self.body_item().is_empty()
    }

    fn body_item(&self) -> ItemStack {
        self.mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::BODY)
    }

    fn play_sound(&self, sound: Sound) {
        self.mob_entity
            .living_entity
            .make_sound(sound, SoundCategory::Neutral, 1.0, 1.0);
    }

    fn spawn_at_attachment(&self, stack: ItemStack) {
        if stack.is_empty() {
            return;
        }
        let entity = self.get_entity();
        let offset = crate::entity::default_passenger_attachment_point(self, 0)
            .unwrap_or_else(|| Vector3::new(0.0, 0.0, 0.0));
        let world = entity.world.load();
        let item_entity = ItemEntity::new(
            Entity::new(world.clone(), entity.pos.load() + offset, &EntityType::ITEM),
            stack,
        );
        world.spawn_entity(Arc::new(item_entity));
    }

    fn detect_body_item_change(&self) {
        let current = self.body_item();
        let previous = {
            let mut last = self
                .last_body_item
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if last.are_items_and_components_equal(&current) {
                return;
            }
            std::mem::replace(&mut *last, current.clone())
        };
        self.on_body_item_changed(&previous, &current);
    }

    fn on_body_item_changed(&self, previous: &ItemStack, current: &ItemStack) {
        if current.is_empty() {
            self.register_goals();
        } else {
            self.mob_entity.goals_selector.clear(self);
            self.mob_entity.set_speed(0.0);
        }

        let mut touched = Vec::new();
        for archetype in matching_archetypes(previous) {
            for modifier in archetype.attribute_modifiers {
                self.mob_entity
                    .living_entity
                    .update_attribute(modifier.attribute, |instance| {
                        instance.remove_modifier(modifier.id);
                    });
                if !touched
                    .iter()
                    .any(|attr: &Attributes| attr.id == modifier.attribute.id)
                {
                    touched.push(modifier.attribute.clone());
                }
            }
        }

        {
            let mut state = self
                .archetype
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *state = ArchetypeState::default();
            for archetype in matching_archetypes(current) {
                if archetype.buoyant {
                    state.floats_in_liquids = true;
                }
                if let Some(explosion) = archetype.explosion.as_ref() {
                    state.explosion = Some(explosion);
                }
                if let Some(contact_damage) = archetype.contact_damage.as_ref() {
                    state.contact_damages.push(contact_damage);
                }
                state.knockback_modifiers = archetype.knockback_modifiers;
                state.sound_settings = archetype.sound_settings;
            }
        }

        for archetype in matching_archetypes(current) {
            for modifier in archetype.attribute_modifiers {
                self.mob_entity
                    .living_entity
                    .update_attribute(modifier.attribute, |instance| {
                        instance.add_or_replace_modifier(Modifier {
                            id: modifier.id.to_string(),
                            amount: modifier.amount,
                            operation: modifier_operation(modifier.operation),
                        });
                    });
                if !touched
                    .iter()
                    .any(|attr: &Attributes| attr.id == modifier.attribute.id)
                {
                    touched.push(modifier.attribute.clone());
                }
            }
        }

        if !touched.is_empty() {
            crate::entity::attributes::send_attribute_updates_for_living(
                &self.mob_entity.living_entity,
                touched,
            );
        }
    }

    fn can_explode(&self) -> bool {
        let has_explosion = self
            .archetype
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .explosion
            .is_some();
        has_explosion && self.mob_entity.living_entity.is_alive() && !self.is_primed()
    }

    fn tick_fuse(&self) {
        let fuse = self.fuse.load(Ordering::Relaxed);
        if fuse > 0 {
            self.fuse.store(fuse - 1, Ordering::Relaxed);
        }

        let explosion = self
            .archetype
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .explosion;
        let Some(explosion) = explosion else {
            return;
        };
        if self.fuse.load(Ordering::Relaxed) != 0 {
            return;
        }

        let entity = self.get_entity();
        entity.remove_leash();
        self.mob_entity
            .living_entity
            .dead
            .store(true, Ordering::Relaxed);

        let world = entity.world.load();
        if world.level_info.load().game_rules.tnt_explodes {
            let interaction = if world.level_info.load().game_rules.mob_griefing {
                ExplosionInteraction::Tnt
            } else {
                ExplosionInteraction::None
            };
            let pos = entity.pos.load();
            world.explode(
                Vector3::new(pos.x, entity.get_y(0.0625), pos.z),
                explosion.power as f32,
                interaction,
            );
        }

        entity.remove();
    }

    fn prime_when_on_powered_position(&self) {
        if !self.can_explode() {
            return;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.block_pos.load();
        if block_receives_redstone_power(&world, &pos) {
            self.prime_time(false);
        }
    }

    fn prime_time(&self, imminent: bool) -> bool {
        let explosion = self
            .archetype
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .explosion;
        let Some(explosion) = explosion else {
            return false;
        };
        if !self.mob_entity.living_entity.is_alive() {
            return false;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        if !world.level_info.load().game_rules.tnt_explodes || self.is_primed() {
            return false;
        }

        let fuse = if imminent {
            TNTEntity::random_short_fuse(explosion.fuse as u32) as i32
        } else {
            explosion.fuse
        };
        entity.set_invulnerable(true);
        self.fuse.store(fuse, Ordering::Relaxed);
        entity.set_synced_data(tracked_data::sulfur_cube::MAX_FUSE, fuse);
        self.play_sound(Sound::EntityTntPrimed);
        world.emit_game_event(
            GameEvent::PrimeFuse.name(),
            entity.block_pos.load().to_centered_f64(),
        );
        true
    }

    fn equip_item(&self, held_item: &ItemStack) -> bool {
        if AgeableMob::is_baby(self) {
            return false;
        }
        if self.has_body_item() {
            let swallowed_item = self.body_item();
            if held_item.item.id == swallowed_item.item.id {
                return false;
            }
            self.spawn_at_attachment(swallowed_item);
        }

        self.mob_entity
            .set_item_slot_and_drop_when_killed(&EquipmentSlot::BODY, held_item.copy_with_count(1));
        self.play_sound(Sound::EntitySulfurCubeAbsorb);
        true
    }

    fn shear(&self) {
        let body_item = self.body_item();
        self.mob_entity
            .set_item_slot(&EquipmentSlot::BODY, ItemStack::EMPTY.clone());
        self.spawn_at_attachment(body_item);
        self.play_sound(Sound::EntitySulfurCubeEject);
        self.pickup_timer
            .store(PICKUP_TIMER_DURATION, Ordering::Relaxed);
    }

    fn apply_contact_damage(&self, target: &dyn EntityBase) {
        let contact_damages = self
            .archetype
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contact_damages
            .clone();
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        for damage in contact_damages {
            let attacker: Option<&dyn EntityBase> =
                (damage.attribute_to_source || target.get_player().is_some()).then_some(self);
            target.damage_with_context(
                target,
                damage.amount.get(&mut random),
                *damage.damage_type,
                None,
                attacker,
                attacker,
            );
        }
    }

    fn player_push(&self, player: &crate::entity::player::Player) {
        if !self.has_body_item() {
            return;
        }
        let player_entity = player.get_entity();
        let root_vehicle = player_entity.get_root_vehicle();
        let pusher = root_vehicle
            .as_ref()
            .map_or(player_entity, |vehicle| vehicle.get_entity());
        let entity = self.get_entity();
        let cube_to_pusher = entity.pos.load() - pusher.pos.load();
        let pusher_feet = pusher.pos.load().y;
        let cube_bottom = entity.pos.load().y;
        let cube_top = cube_bottom + f64::from(entity.height());
        let pusher_top = pusher_feet + f64::from(pusher.height());
        if cube_to_pusher.horizontal_length() >= PUSH_DISTANCE_THRESHOLD
            || pusher_feet > cube_top
            || pusher_top <= cube_bottom
        {
            return;
        }
        let knockback = (1.0
            - self
                .mob_entity
                .living_entity
                .get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE))
        .max(0.0);
        let direction =
            Vector3::new(cube_to_pusher.x, 0.0, cube_to_pusher.z).normalize() * knockback;
        let speed_scale = if player_entity.has_vehicle() {
            VEHICLE_PUSH_SPEED_SCALE_MULTIPLIER
        } else {
            PLAYER_PUSH_SPEED_SCALE_MULTIPLIER
        };
        let player_speed = (player_entity.known_movement().length() * 2.0 * f64::from(speed_scale))
            .clamp(0.0, MAX_PLAYER_PUSH_SPEED);
        let vertical = if entity.on_ground.load(Ordering::Relaxed) {
            knockback * f64::from(VERTICAL_PUSH_MULTIPLIER)
        } else {
            0.0
        };
        let push = Vector3::new(direction.x, vertical, direction.z) * player_speed;
        entity.velocity_dirty.store(true, Ordering::SeqCst);
        let sound_settings = self
            .archetype
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .sound_settings;
        let threshold = f64::from(sound_settings.push_sound_impulse_threshold);
        if push.length_squared() > threshold * threshold
            && self.push_sound_cooldown.load(Ordering::Relaxed) <= 0
        {
            self.push_sound_cooldown.store(
                (sound_settings.push_sound_cooldown * 20.0) as i32,
                Ordering::Relaxed,
            );
            self.play_sound(sound_settings.push_sound);
        }
        entity.velocity.store(entity.velocity.load() + push);
        self.apply_contact_damage(player);
    }

    fn play_eating_sound(&self) {
        self.play_sound(Sound::EntitySmallSulfurCubeEat);
    }
}

impl CubeMob for SulfurCubeEntity {
    fn cube_data(&self) -> &CubeMobData {
        &self.cube
    }

    fn get_jump_sound(&self) -> Sound {
        if CubeMob::is_tiny(self) {
            Sound::EntitySmallSulfurCubeJump
        } else {
            Sound::EntitySulfurCubeJump
        }
    }

    fn get_squish_sound(&self) -> Sound {
        if CubeMob::is_tiny(self) {
            Sound::EntitySmallSulfurCubeSquish
        } else if self.has_body_item() {
            Sound::EntitySulfurCubeBounce
        } else {
            Sound::EntitySulfurCubeSquish
        }
    }

    fn cube_sound_category(&self) -> SoundCategory {
        SoundCategory::Neutral
    }
}

impl CustomSound for SulfurCubeEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(if CubeMob::is_tiny(self) {
            Sound::EntitySmallSulfurCubeDeath
        } else {
            Sound::EntitySulfurCubeDeath
        })
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(if CubeMob::is_tiny(self) {
            Sound::EntitySmallSulfurCubeHurt
        } else {
            Sound::EntitySulfurCubeHurt
        })
    }
}

impl AgeableMob for SulfurCubeEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn age_boundary_reached(&self) {
        if !AgeableMob::is_baby(self) {
            self.set_size(MAX_SIZE, true);
        }
    }
}

impl Bucketable for SulfurCubeEntity {
    fn is_from_bucket(&self) -> bool {
        self.from_bucket.load(Ordering::Relaxed)
    }

    fn set_from_bucket(&self, from_bucket: bool) {
        self.from_bucket.store(from_bucket, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::sulfur_cube::FROM_BUCKET, from_bucket);
    }

    fn bucket_item(&self) -> &'static Item {
        &Item::SULFUR_CUBE_BUCKET
    }

    fn pickup_sound(&self) -> Sound {
        Sound::ItemBucketFillSulfurCube
    }

    fn save_to_bucket_tag(&self, bucket: &mut ItemStack) {
        self.save_default_data_to_bucket_tag(bucket);
        let mut nbt = bucket
            .get_data_component::<BucketEntityDataImpl>()
            .and_then(|data| data.nbt.clone())
            .unwrap_or_default();
        nbt.put_int("age", self.get_age());
        nbt.put_bool("age_locked", self.is_age_locked());
        bucket.set_data_component(BucketEntityDataImpl { nbt: Some(nbt) });
    }

    fn load_from_bucket_tag(&self, nbt: &NbtCompound) {
        self.load_default_data_from_bucket_tag(nbt);
        self.set_age(nbt.get_int("age").unwrap_or(0));
        self.set_age_locked(nbt.get_bool("age_locked").unwrap_or(false));
    }
}

impl Mob for SulfurCubeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_bucketable(&self) -> Option<&dyn Bucketable> {
        Some(self)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.detect_body_item_change();
        self.tick_fuse();
        self.prime_when_on_powered_position();
        tick_cube_mob(self);
    }

    fn mob_travel_in_fluid(&self, _caller: &dyn EntityBase) {
        if !self.has_body_item() {
            return;
        }
        let floats_in_liquids = self
            .archetype
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .floats_in_liquids;
        if !floats_in_liquids {
            return;
        }

        let entity = self.get_entity();
        let tick_count = entity.tick_count.load(Ordering::Relaxed);
        let vibe = 0.2 * (tick_count as f32 * 0.4).sin();
        let fluid_height = if entity.is_in_water() {
            entity.water_height.load()
        } else {
            entity.lava_height.load()
        };
        let immersion =
            fluid_height - self.mob_entity.living_entity.get_swim_height() + f64::from(vibe);
        if immersion > 0.0 {
            let mut velocity = entity.velocity.load();
            velocity.y += immersion.min(1.0) * 0.04;
            entity.velocity.store(velocity);
        }
    }

    fn omnidirectional_air_mover(&self) -> bool {
        self.has_body_item()
    }

    fn mob_max_up_step(&self) -> Option<f32> {
        self.has_body_item().then_some(0.0)
    }

    fn mob_can_freeze(&self) -> bool {
        !self.has_body_item()
    }

    fn can_breathe_underwater(&self) -> bool {
        self.has_body_item()
            || self
                .get_entity()
                .entity_type
                .has_tag(&pumpkin_data::tag::EntityType::MINECRAFT_CAN_BREATHE_UNDER_WATER)
    }

    fn can_pick_up_loot(&self, _brain: Option<&crate::entity::ai::brain::BrainState>) -> bool {
        !self.has_body_item()
    }

    fn can_hold_item(&self, stack: &ItemStack) -> bool {
        self.body_item().is_empty() && is_swallowable_item(stack) && !AgeableMob::is_baby(self)
    }

    fn pick_up_item(&self, item: &ItemEntity) {
        let stack = item
            .get_item_stack()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if !self.can_hold_item(&stack) || self.pickup_timer.load(Ordering::Relaxed) > 0 {
            return;
        }
        let living = &self.mob_entity.living_entity;
        if !living.pickup(item.get_entity(), 1) {
            return;
        }
        living.on_item_pickup(item);
        let taken = item
            .get_item_stack()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .split(1);
        self.mob_entity
            .set_item_slot_and_drop_when_killed(&EquipmentSlot::BODY, taken);
        self.play_sound(Sound::EntitySulfurCubeAbsorb);

        let empty = item
            .get_item_stack()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty();
        if empty {
            item.get_entity().remove();
        } else {
            item.init_data_tracker();
        }
    }

    fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        if slot == &EquipmentSlot::BODY {
            self.get_entity().is_alive() && !AgeableMob::is_baby(self)
        } else {
            true
        }
    }

    fn is_equippable_in_slot(&self, stack: &ItemStack, slot: &EquipmentSlot) -> bool {
        if slot == &EquipmentSlot::BODY {
            is_swallowable_item(stack)
        } else {
            crate::entity::mob::equipment::is_equippable_in_slot(
                self.get_entity().entity_type,
                stack,
                slot,
            ) && self.can_use_slot(slot)
        }
    }

    fn can_dispenser_equip_into_slot(&self, slot: &EquipmentSlot) -> bool {
        slot == &EquipmentSlot::BODY
    }

    fn mob_get_component(&self, component: DataComponent) -> Option<Box<dyn DataComponentImpl>> {
        if component == DataComponent::SulfurCubeContent {
            let body_item = self.body_item();
            if !body_item.is_empty() {
                return Some(Box::new(SulfurCubeContentImpl { item: body_item }));
            }
        }
        None
    }

    fn mob_hurt_server(
        &self,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> Option<bool> {
        if !self.has_body_item() {
            return None;
        }
        if self.can_explode() {
            let burning_arrow = source
                .and_then(|source| source.cast_any().downcast_ref::<ArrowEntity>())
                .is_some_and(ArrowEntity::is_on_fire);
            if damage_type.has_tag(&pumpkin_data::tag::DamageType::MINECRAFT_IS_FIRE)
                || burning_arrow
            {
                self.prime_time(false);
            } else if damage_type.has_tag(&pumpkin_data::tag::DamageType::MINECRAFT_IS_EXPLOSION) {
                self.prime_time(true);
            }
        }
        if !damage_type
            .has_tag(&pumpkin_data::tag::DamageType::MINECRAFT_SULFUR_CUBE_WITH_BLOCK_IMMUNE_TO)
        {
            return None;
        }
        if !damage_type.has_tag(&pumpkin_data::tag::DamageType::MINECRAFT_NO_KNOCKBACK) {
            self.mob_entity
                .living_entity
                .deal_default_knockback(self, position, source, cause, amount);
        }
        Some(true)
    }

    fn mob_knockback(
        &self,
        power: f64,
        x: f64,
        z: f64,
        source: Option<&Entity>,
        damage: f32,
        comes_from_effect: bool,
    ) -> bool {
        let Some(source) = source else {
            return false;
        };
        if !self.has_body_item() {
            return false;
        }
        let (knockback, hit_sound) = {
            let state = self
                .archetype
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (state.knockback_modifiers, state.sound_settings.hit_sound)
        };
        let entity = self.get_entity();
        let height = entity.height();
        let bounding_box = entity.bounding_box.load();
        let center = (bounding_box.min + bounding_box.max) * 0.5;
        let eye = source.get_eye_pos();
        let look = Vector3::from_yaw_pitch(source.yaw.load(), source.pitch.load()).normalize();

        let (xd, zd) = apply_horizontal_hit_angle_scale(
            HORIZONTAL_HIT_ANGLE_SCALE,
            (x as f32, z as f32),
            eye,
            look,
            center,
        );
        let (horizontal, vertical) = apply_vertical_hit_angle_power_transfer(
            VERTICAL_HIT_ANGLE_SCALE,
            knockback.horizontal_power,
            knockback.vertical_power,
            eye,
            look,
            center,
            height,
        );
        let (mut horizontal, mut vertical) = apply_vertical_position_angle_power_rotation(
            VERTICAL_POSITION_ANGLE_SCALE,
            horizontal,
            vertical,
            knockback.horizontal_power,
            knockback.vertical_power,
            source.pos.load(),
            entity.pos.load(),
        );

        let power_multiplier = damage.sqrt()
            * if comes_from_effect {
                power as f32 * EXTRA_KNOCKBACK_DAMPENING
            } else {
                1.0
            };
        horizontal *= power_multiplier;
        vertical *= power_multiplier;
        let resistance = self
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE);
        horizontal *= (1.0 - resistance) as f32;
        vertical *= (1.0 - resistance) as f32;
        horizontal *= 0.4;
        let horizontal = horizontal.clamp(-128.0, 128.0);
        let vertical = vertical.clamp(-128.0, 128.0);
        let push =
            Vector3::new(f64::from(xd), 0.0, f64::from(zd)).normalize() * f64::from(horizontal);
        let velocity = entity.velocity.load();
        entity.velocity.store(Vector3::new(
            velocity.x - push.x,
            velocity.y + f64::from(vertical) * 1.2,
            velocity.z - push.z,
        ));
        entity.velocity_dirty.store(true, Ordering::SeqCst);
        self.play_sound(hit_sound);
        true
    }

    fn can_be_picked_from_inside(&self) -> bool {
        !self.has_body_item()
    }

    fn is_invulnerable_to_piercing_weapon(&self) -> bool {
        self.get_entity().invulnerable.load(Ordering::Relaxed) && !self.is_primed()
    }

    fn spawn_as_baby(&self) -> bool {
        self.set_size(MIN_SIZE, true);
        AgeableMob::set_baby(self, true);
        true
    }

    fn mob_receives_player_touch(&self) -> bool {
        true
    }

    fn mob_player_collision(&self, player: &Arc<crate::entity::player::Player>) {
        self.player_push(player);
    }

    fn do_push(&self, entity: &Arc<dyn EntityBase>) {
        self.apply_contact_damage(entity.as_ref());
    }

    fn mob_interact(
        &self,
        player: &Arc<crate::entity::player::Player>,
        item_stack: &mut ItemStack,
    ) -> bool {
        let item = item_stack.get_item();

        if AgeableMob::is_baby(self) {
            if is_food(item_stack) && self.can_age_up() {
                let age = self.get_age();
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                self.age_up(get_speed_up_seconds_when_feeding(-age), true);
                self.play_eating_sound();
                return true;
            }
            return self
                .mob_entity
                .mob_interact(player, item_stack, self.can_be_leashed());
        }

        if self.is_primed() {
            return false;
        }

        if !self.can_explode()
            || (item.id != Item::FLINT_AND_STEEL.id && item.id != Item::FIRE_CHARGE.id)
        {
            if item.id == Item::SHEARS.id && self.has_body_item() {
                let entity = self.get_entity();
                let world = entity.world.load();
                if let Some(server) = world.server.upgrade() {
                    let mut event = PlayerShearEntityEvent {
                        player: player.clone(),
                        entity_id: entity.entity_id,
                        hand: 0,
                        cancelled: false,
                    };
                    server.plugin_manager.fire_blocking(&server, &mut event);
                    if event.cancelled {
                        return false;
                    }
                }

                let sheared_item = self.body_item();
                self.shear();
                world.emit_game_event(GameEvent::Shear.name(), entity.pos.load());
                let _ = player.damage_item_stack(item_stack, 1);
                player.trigger_advancement(AdvancementTrigger::PlayerShearedEquipment {
                    entity_type: "minecraft:sulfur_cube".to_string(),
                    item_id: format!("minecraft:{}", sheared_item.item.registry_key),
                });
                return true;
            }

            if is_swallowable_item(item_stack) {
                let equipped = self.equip_item(item_stack);
                if equipped {
                    item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                    self.get_entity().world.load().emit_game_event(
                        GameEvent::EntityInteract.name(),
                        self.get_entity().block_pos.load().to_centered_f64(),
                    );
                }
                return equipped;
            }

            return self
                .mob_entity
                .mob_interact(player, item_stack, self.can_be_leashed());
        }

        let world = self.get_entity().world.load();
        if !world.level_info.load().game_rules.tnt_explodes {
            player.send_system_message_raw(
                &TextComponent::translate(translation::java::BLOCK_MINECRAFT_TNT_DISABLED, []),
                true,
            );
            return false;
        }

        self.prime_time(false);
        if item.id == Item::FLINT_AND_STEEL.id {
            let _ = player.damage_item_stack(item_stack, 1);
        } else {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
        }
        player.increment_stat(StatisticCategory::Used, i32::from(item.id), 1);
        true
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        let pickup_timer = self.pickup_timer.load(Ordering::Relaxed);
        if pickup_timer > 0 {
            self.pickup_timer.store(pickup_timer - 1, Ordering::Relaxed);
        }

        let push_sound_cooldown = self.push_sound_cooldown.load(Ordering::Relaxed);
        if push_sound_cooldown > 0 {
            self.push_sound_cooldown
                .store(push_sound_cooldown - 1, Ordering::Relaxed);
        }
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, state: &BlockState) {
        if !self.has_body_item() {
            self.play_block_step_sound(state);
        }
    }

    fn requires_custom_persistence(&self) -> bool {
        self.has_body_item() || self.is_from_bucket()
    }

    fn can_be_leashed(&self) -> bool {
        self.has_body_item()
    }

    fn get_base_experience_reward(&self) -> u32 {
        if AgeableMob::is_baby(self) {
            0
        } else {
            1 + self.get_random().random_range(0..2u32)
        }
    }

    fn finalize_spawn(
        &self,
        _world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let group_data = self.finalize_spawn_super(group_data);
        if AgeableMob::is_baby(self) {
            self.set_size(MIN_SIZE, true);
        } else {
            self.set_size(MAX_SIZE, true);
        }
        group_data
    }

    fn mob_passenger_attachment_point(
        &self,
        _passenger: &dyn EntityBase,
        _index: usize,
    ) -> Option<Vector3<f64>> {
        let living = &self.mob_entity.living_entity;
        let scale = living.get_attribute_value(&Attributes::SCALE);
        Some(Vector3::new(
            0.0,
            f64::from(living.entity.height())
                - PASSENGER_SINK_PER_SIZE * f64::from(CubeMob::get_size(self)) * scale,
            0.0,
        ))
    }

    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        Some(crate::entity::mob::MobMetadata::new().slime_size(CubeMob::get_size(self)))
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("Size", CubeMob::get_size(self) - 1);
        nbt.put_bool(
            "wasOnGround",
            self.cube.was_on_ground.load(Ordering::Relaxed),
        );
        nbt.put_int("pickup_timer", self.pickup_timer.load(Ordering::Relaxed));
        nbt.put_bool("from_bucket", self.is_from_bucket());
        nbt.put_int("fuse", self.fuse.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.pickup_timer
            .store(nbt.get_int("pickup_timer").unwrap_or(0), Ordering::Relaxed);
        self.set_from_bucket(nbt.get_bool("from_bucket").unwrap_or(false));
        let fuse = nbt.get_int("fuse").unwrap_or(-1);
        self.fuse.store(fuse, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::sulfur_cube::MAX_FUSE, fuse);
        self.set_size(nbt.get_int("Size").unwrap_or(0) + 1, false);
        self.cube.was_on_ground.store(
            nbt.get_bool("wasOnGround").unwrap_or(false),
            Ordering::Relaxed,
        );
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        if entity.age.load(Ordering::Relaxed) < 0 {
            entity.set_synced_data(tracked_data::ageable_mob::DATA_BABY_ID, true);
        }
        self.set_from_bucket(self.is_from_bucket());
        entity.set_synced_data(
            tracked_data::sulfur_cube::MAX_FUSE,
            self.fuse.load(Ordering::Relaxed),
        );
    }

    fn on_removed(&self) {
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        let size = CubeMob::get_size(self);
        if size <= 1 || living.health.load() > 0.0 {
            return;
        }
        let pos = entity.pos.load();
        let half_size = size / 2;
        let count = if self.is_primed() { 0 } else { SPLIT_COUNT };
        let xz_offset = entity.entity_dimension.load().width / 2.0;
        for i in 0..count {
            let xd = ((i % 2) as f32 - 0.5) * xz_offset;
            let zd = ((i / 2) as f32 - 0.5) * xz_offset;
            let yaw = self.get_random().random_range(0.0..360.0);
            conversion::split_on_death(self, entity.entity_type, |child| {
                let Some(child) = child.cast_any().downcast_ref::<Self>() else {
                    return;
                };
                child.set_size(half_size, true);
                let child_entity = child.get_entity();
                child_entity.set_pos(Vector3::new(
                    pos.x + f64::from(xd),
                    pos.y + 0.5,
                    pos.z + f64::from(zd),
                ));
                child_entity.set_rotation(yaw, 0.0);
                AgeableMob::set_baby(child, true);
            });
        }
    }
}

struct SulfurCubeMobMoveControl {
    inner: CubeMobMoveControl,
    cube: Weak<SulfurCubeEntity>,
}

impl SulfurCubeMobMoveControl {
    fn new(cube: Weak<SulfurCubeEntity>) -> Self {
        let cube_mob: Weak<dyn CubeMob> = cube.clone();
        Self {
            inner: CubeMobMoveControl::new(cube_mob),
            cube,
        }
    }
}

impl Control for SulfurCubeMobMoveControl {}

impl MoveControlTrait for SulfurCubeMobMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        if self.cube.upgrade().is_some_and(|cube| cube.has_body_item()) {
            return;
        }
        self.inner.tick(mob);
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.inner.set_wanted_position(x, y, z, speed_modifier);
    }

    fn strafe(&mut self, forward: f32, right: f32) {
        self.inner.strafe(forward, right);
    }

    fn has_wanted(&self) -> bool {
        self.inner.has_wanted()
    }

    fn get_speed_modifier(&self) -> f64 {
        self.inner.get_speed_modifier()
    }

    fn get_wanted_position(&self) -> Vector3<f64> {
        self.inner.get_wanted_position()
    }
}

struct SulfurCubeLookControl {
    inner: LookControl,
    cube: Weak<SulfurCubeEntity>,
}

impl SulfurCubeLookControl {
    fn new(cube: Weak<SulfurCubeEntity>) -> Self {
        Self {
            inner: LookControl::default(),
            cube,
        }
    }
}

impl Control for SulfurCubeLookControl {}

impl LookControlTrait for SulfurCubeLookControl {
    fn wanted_position(&self) -> Option<Vector3<f64>> {
        self.inner.wanted_position()
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if self.cube.upgrade().is_some_and(|cube| cube.has_body_item()) {
            let entity = mob.get_entity();
            let yaw = entity.yaw.load();
            let close_angle = wrap_degrees_90(yaw);
            entity.yaw.store(yaw - close_angle);
            entity.head_yaw.store(entity.yaw.load());
        } else {
            self.inner.tick(mob);
        }
    }

    fn look_at_with_range(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        max_yaw_change: f32,
        max_pitch_change: f32,
    ) {
        self.inner
            .look_at_with_range(x, y, z, max_yaw_change, max_pitch_change);
    }
}

struct SulfurCubeSearchForItemsGoal {
    cube: Weak<SulfurCubeEntity>,
    target_item: Option<Arc<dyn EntityBase>>,
}

impl Goal for SulfurCubeSearchForItemsGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(cube) = self.cube.upgrade() else {
            return false;
        };
        if AgeableMob::is_baby(cube.as_ref()) || cube.pickup_timer.load(Ordering::Relaxed) > 0 {
            return false;
        }

        let entity = cube.get_entity();
        let world = entity.world.load();
        let area = entity.bounding_box.load().expand(8.0, 8.0, 8.0);
        let pos = entity.pos.load();
        self.target_item = world
            .get_entities_at_box(&area)
            .into_iter()
            .filter(|candidate| {
                let Some(item) = candidate.get_item_entity() else {
                    return false;
                };
                if item.get_pickup_delay() != 0 || !candidate.get_entity().is_alive() {
                    return false;
                }
                let stack = item
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                is_swallowable_item(&stack)
            })
            .min_by(|a, b| {
                a.get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&pos)
                    .total_cmp(&b.get_entity().pos.load().squared_distance_to_vec(&pos))
            });
        self.target_item.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob)
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(target) = &self.target_item else {
            return;
        };
        mob.get_mob_entity()
            .look_at_entity(target.as_ref(), 10.0, 10.0);
        if let Some(cube) = self.cube.upgrade() {
            let data = cube.cube_data();
            data.target_yaw.store(cube.get_entity().yaw.load());
            data.is_aggressive.store(true, Ordering::Relaxed);
        }
    }

    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}
