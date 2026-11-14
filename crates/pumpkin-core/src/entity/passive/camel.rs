mod ai;

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::wrap_degrees;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::brain::{Brain, memory::PackedMemories},
    mob::spawn::{EntitySpawnReason, SpawnGroupData},
    mob::{Mob, MobEntity, equipment::equip_on_target},
    passive::abstract_horse::{
        AbstractHorse, AbstractHorseData, BEDROCK_SADDLE_INVENTORY_SIZE, spawn_age_up_particle,
        stand_on_hurt,
    },
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;
use pumpkin_data::entity::EntityPose;
use pumpkin_util::math::vector3::Vector3;

const CAMEL_AGE_UP_SECONDS: i32 = 10;
const MAX_PASSENGERS: usize = 2;
const MAX_HEAD_Y_ROT: f32 = 30.0;
const DASH_COOLDOWN_TICKS: i32 = 55;
const DASH_MINIMUM_DURATION_TICKS: i32 = 5;
const SITDOWN_DURATION_TICKS: i64 = 40;
const STANDUP_DURATION_TICKS: i64 = 52;
const SITTING_HEIGHT_DIFFERENCE: f32 = 1.43;
const BABY_SCALE: f32 = 0.6;
const HUSK_STEP_VOLUME: f32 = 0.4;
const DRIVER_SEAT_OFFSET: f32 = 0.5;
const BACK_SEAT_OFFSET: f32 = -0.7;
const ANIMAL_SEAT_SHIFT: f32 = 0.2;
const REMOVED_SEAT_HEIGHT: f64 = 0.01;

pub struct CamelEntity {
    husk: bool,
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    horse_data: AbstractHorseData,
    pub dashing: AtomicBool,
    dash_cooldown: AtomicI32,
    last_pose_change_tick: AtomicI64,
}

impl CamelEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let husk = entity.entity_type == &EntityType::CAMEL_HUSK;
        let mut mob_entity = MobEntity::new(entity);
        let navigator = mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_can_float(true);
        navigator.set_can_walk_over_fences(true);
        let camel = Self {
            husk,
            mob_entity,
            ageable_data: AgeableData::default(),
            horse_data: AbstractHorseData::default(),
            dashing: AtomicBool::new(false),
            dash_cooldown: AtomicI32::new(0),
            last_pose_change_tick: AtomicI64::new(0),
        };
        Arc::new(camel)
    }

    #[must_use]
    pub fn is_dashing(&self) -> bool {
        self.dashing.load(Ordering::Relaxed)
    }

    pub fn set_dashing(&self, dashing: bool) {
        let changed = self.dashing.swap(dashing, Ordering::Relaxed) != dashing;
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::camel::DASH, dashing);
        // Any change of the synced flag after the first tick starts the cooldown if it is idle.
        if changed && entity.tick_count.load(Ordering::Relaxed) > 0 {
            let _ = self.dash_cooldown.compare_exchange(
                0,
                DASH_COOLDOWN_TICKS,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
        }
    }

    fn tick_dash(&self) {
        let entity = self.get_entity();
        let cooldown = self.dash_cooldown.load(Ordering::Relaxed);
        if self.is_dashing()
            && cooldown < DASH_COOLDOWN_TICKS - DASH_MINIMUM_DURATION_TICKS
            && (entity.on_ground.load(Ordering::Relaxed)
                || entity.is_in_water()
                || entity.touching_lava.load(Ordering::Relaxed)
                || entity.has_vehicle())
        {
            self.set_dashing(false);
        }
        let cooldown = self.dash_cooldown.load(Ordering::Relaxed);
        if cooldown > 0 {
            self.dash_cooldown.store(cooldown - 1, Ordering::Relaxed);
            if cooldown == 1
                && let Some(sound) = entity.entity_type.dash_ready_sound
            {
                entity.world.load().play_sound_fine(
                    sound,
                    SoundCategory::Neutral,
                    &entity.block_pos.load().to_centered_f64(),
                    1.0,
                    1.0,
                );
            }
        }
    }

    #[must_use]
    pub const fn is_husk(&self) -> bool {
        self.husk
    }

    fn make_type_sound(&self, sound: Option<Sound>) {
        if let Some(sound) = sound {
            self.make_sound(sound);
        }
    }

    /// A negative tick means the camel sat down at that tick.
    #[must_use]
    pub fn is_camel_sitting(&self) -> bool {
        self.last_pose_change_tick.load(Ordering::Relaxed) < 0
    }

    #[must_use]
    pub fn get_pose_time(&self) -> i64 {
        self.get_entity().world.load().get_world_age()
            - self.last_pose_change_tick.load(Ordering::Relaxed).abs()
    }

    #[must_use]
    pub fn is_in_pose_transition(&self) -> bool {
        let duration = if self.is_camel_sitting() {
            SITDOWN_DURATION_TICKS
        } else {
            STANDUP_DURATION_TICKS
        };
        self.get_pose_time() < duration
    }

    #[must_use]
    pub fn refuse_to_move(&self) -> bool {
        self.is_camel_sitting() || self.is_in_pose_transition()
    }

    #[must_use]
    pub fn can_camel_change_pose(&self) -> bool {
        let entity = self.get_entity();
        let dimensions = self.dimensions_for(!self.is_camel_sitting());
        let pos = entity.pos.load();
        let target = BoundingBox::new_from_pos(pos.x, pos.y, pos.z, &dimensions);
        entity
            .world
            .load()
            .is_space_empty(target.contract_all(1.0E-7))
    }

    pub fn sit_down(&self) {
        if self.is_camel_sitting() {
            return;
        }
        self.make_type_sound(self.get_entity().entity_type.sit_sound);
        self.change_pose(true);
        self.reset_last_pose_change_tick(-self.get_entity().world.load().get_world_age());
    }

    pub fn stand_up(&self) {
        if !self.is_camel_sitting() {
            return;
        }
        self.make_type_sound(self.get_entity().entity_type.stand_sound);
        self.change_pose(false);
        self.reset_last_pose_change_tick(self.get_entity().world.load().get_world_age());
    }

    pub fn stand_up_instantly(&self) {
        self.change_pose(false);
        self.reset_last_pose_change_tick_to_full_stand(
            self.get_entity().world.load().get_world_age(),
        );
    }

    fn change_pose(&self, sitting: bool) {
        let entity = self.get_entity();
        entity.set_default_dimensions(self.dimensions_for(sitting));
        entity.set_pose(if sitting {
            EntityPose::Sitting
        } else {
            EntityPose::Standing
        });
        entity.emit_own_game_event(self, GameEvent::EntityAction);
    }

    fn reset_last_pose_change_tick(&self, tick: i64) {
        self.last_pose_change_tick.store(tick, Ordering::Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::camel::LAST_POSE_CHANGE_TICK,
            tick,
        );
    }

    fn reset_last_pose_change_tick_to_full_stand(&self, game_time: i64) {
        self.reset_last_pose_change_tick((game_time - STANDUP_DURATION_TICKS - 1).max(0));
    }

    fn dimensions_for(&self, sitting: bool) -> EntityDimensions {
        let entity_type = self.get_entity().entity_type;
        let baby = self.is_baby();
        sitting
            .then(|| entity_type.pose_dimensions(EntityPose::Sitting, baby))
            .flatten()
            .map_or_else(
                || Entity::age_dimensions(entity_type, baby),
                |pose| pose.dimensions,
            )
    }

    fn age_scale(&self) -> f32 {
        if self.is_baby() { BABY_SCALE } else { 1.0 }
    }

    /// Height of the saddle above the feet, following the sit and stand animations.
    fn body_anchor_y_offset(&self, front: bool, height: f32, scale: f32) -> f64 {
        let age_sit_y_offset = if self.is_baby() { 0.093_75 } else { 0.375 };
        let mut offset = f64::from(height) - age_sit_y_offset;
        let sitting_height_difference = scale * SITTING_HEIGHT_DIFFERENCE;
        let vertical_drop = sitting_height_difference - scale * 0.2;
        let bottom_point = sitting_height_difference - vertical_drop;
        let sitting = self.is_camel_sitting();
        let in_transition = self.is_in_pose_transition();
        if in_transition {
            let (duration, half_point, flex_point_offset) = match (sitting, front) {
                (true, true) => (SITDOWN_DURATION_TICKS, 28, 0.5),
                (true, false) => (SITDOWN_DURATION_TICKS, 28, 0.1),
                (false, true) => (STANDUP_DURATION_TICKS, 24, 0.6),
                (false, false) => (STANDUP_DURATION_TICKS, 32, 0.35),
            };
            let pose_time = (self.get_pose_time() as f32).clamp(0.0, duration as f32);
            let half_point = half_point as f32;
            let first_part = pose_time < half_point;
            let part = if first_part {
                pose_time / half_point
            } else {
                (pose_time - half_point) / (duration as f32 - half_point)
            };
            let flex_point = sitting_height_difference - flex_point_offset * vertical_drop;
            let (from, to) = match (sitting, first_part) {
                (true, true) => (sitting_height_difference, flex_point),
                (true, false) => (flex_point, bottom_point),
                (false, true) => (
                    bottom_point - sitting_height_difference,
                    bottom_point - flex_point,
                ),
                (false, false) => (bottom_point - flex_point, 0.0),
            };
            offset += f64::from(part.mul_add(to - from, from));
        }
        if sitting && !in_transition {
            offset += f64::from(bottom_point);
        }
        offset
    }

    fn clamp_head_rotation_to_body(&self) {
        let entity = self.get_entity();
        let head = entity.head_yaw.load();
        let diff = wrap_degrees(entity.body_yaw.load() - head);
        let clamped = diff.clamp(-MAX_HEAD_Y_ROT, MAX_HEAD_Y_ROT);
        entity.head_yaw.store(head + diff - clamped);
    }
}

crate::impl_abstract_horse!(
    CamelEntity,
    {
        fn is_mob_controlled(&self) -> bool {
            self.is_husk()
                && self
                    .mob_entity
                    .first_passenger()
                    .is_some_and(|passenger| passenger.get_mob().is_some())
        }

        fn can_perform_rearing(&self) -> bool {
            false
        }

        fn refresh_age_dimensions(&self) {
            self.get_entity()
                .set_default_dimensions(self.dimensions_for(self.is_camel_sitting()));
        }

        fn finalize_horse_spawn(
            &self,
            world: &Arc<World>,
            _reason: EntitySpawnReason,
            group_data: Option<SpawnGroupData>,
        ) -> Option<SpawnGroupData> {
            self.reset_last_pose_change_tick_to_full_stand(world.get_world_age());
            self.randomize_attributes();
            self.finalize_spawn_super(group_data)
        }

        fn subclass_ai_step(&self) {
            self.tick_dash();
            if self.refuse_to_move() {
                self.clamp_head_rotation_to_body();
            }
            if self.is_camel_sitting() && self.get_entity().is_in_water() {
                self.stand_up_instantly();
            }
        }

        fn on_hurt(&self) {
            self.stand_up_instantly();
            stand_on_hurt(self);
        }

        fn on_leash_pull(&self) {
            if self.is_eating() {
                self.set_eating(false);
            }
            if self.is_camel_sitting()
                && !self.is_in_pose_transition()
                && self.can_camel_change_pose()
            {
                self.stand_up();
            }
        }

        fn can_jump(&self) -> bool {
            !self.refuse_to_move() && self.is_saddled()
        }

        fn handle_start_jump(&self, _jump_scale: i32) {
            self.make_type_sound(self.get_entity().entity_type.dash_sound);
            self.get_entity()
                .emit_own_game_event(self, GameEvent::EntityAction);
            self.set_dashing(true);
        }

        fn bedrock_inventory_size(&self) -> i32 {
            BEDROCK_SADDLE_INVENTORY_SIZE
        }

        fn handle_eating(&self, player: &Arc<Player>, stack: &ItemStack) -> bool {
            if !self.is_food(stack) {
                return false;
            }
            let living = &self.mob_entity.living_entity;
            let could_heal = living.health.load() < living.get_max_health();
            if could_heal {
                living.heal(2.0);
            }
            let could_set_in_love =
                self.is_tamed() && self.get_age() == 0 && !self.mob_entity.is_in_love();
            if could_set_in_love {
                self.set_in_love(player);
            }
            let could_age_up = self.can_age_up();
            if could_age_up {
                spawn_age_up_particle(self.get_entity(), &mut self.get_random());
                self.age_up(CAMEL_AGE_UP_SECONDS, false);
            }
            if !could_heal && !could_set_in_love && !could_age_up {
                return false;
            }
            if let Some(sound) = self.get_eating_sound() {
                self.play_varied_sound(sound);
            }
            let entity = self.get_entity();
            entity
                .world
                .load()
                .emit_game_event(GameEvent::Eat.name(), entity.pos.load());
            true
        }
    },
    {
        fn can_be_a_baby(&self) -> bool {
            !self.husk
        }
    }
);

impl Animal for CamelEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(if self.is_husk() {
            &tag::Item::MINECRAFT_CAMEL_HUSK_FOOD
        } else {
            &tag::Item::MINECRAFT_CAMEL_FOOD
        })
    }

    fn can_mate(&self, partner: &dyn EntityBase) -> bool {
        !self.is_husk() && self.can_mate_with(partner, &[&EntityType::CAMEL])
    }
}

impl Mob for CamelEntity {
    crate::abstract_horse_mob_hooks!(always_tamed);

    fn charge_speed_modifier(&self) -> f32 {
        if self.get_entity().entity_type == &EntityType::CAMEL_HUSK {
            4.0
        } else {
            1.0
        }
    }

    fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        crate::entity::passive::abstract_horse::can_use_slot(self, slot)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, state: &BlockState) {
        let sand = state
            .id
            .to_block()
            .has_tag(&tag::Block::MINECRAFT_CAMEL_SAND_STEP_SOUND_BLOCKS);
        let (sound, volume) = match (self.husk, sand) {
            (false, true) => (Sound::EntityCamelStepSand, 1.0),
            (false, false) => (Sound::EntityCamelStep, 1.0),
            (true, true) => (Sound::EntityCamelHuskStepSand, HUSK_STEP_VOLUME),
            (true, false) => (Sound::EntityCamelHuskStep, HUSK_STEP_VOLUME),
        };
        self.play_entity_sound(sound, volume, 1.0);
    }

    fn can_be_leashed(&self) -> bool {
        !self.is_mob_controlled()
    }

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        self.is_husk()
    }

    fn mob_interact_with_hand(
        &self,
        _player: &Arc<Player>,
        _item_stack: &mut ItemStack,
        _hand: pumpkin_util::Hand,
    ) -> Option<bool> {
        if self.is_husk() {
            self.mob_entity
                .persistence_required
                .store(true, Ordering::Relaxed);
        }
        None
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_horse_nbt(nbt);
        nbt.put_long(
            "LastPoseTick",
            self.last_pose_change_tick.load(Ordering::Relaxed),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_horse_nbt(nbt);
        self.mob_entity
            .read_legacy_saddle(nbt.get_bool("Saddle") == Some(true));
        let pose_tick = nbt.get_long("LastPoseTick").unwrap_or(0);
        if pose_tick < 0 {
            self.get_entity()
                .set_default_dimensions(self.dimensions_for(true));
            self.get_entity().set_pose(EntityPose::Sitting);
        }
        self.reset_last_pose_change_tick(pose_tick);
    }

    fn mob_init_data_tracker(&self) {
        self.init_horse_data_tracker();
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::camel::DASH, self.is_dashing());
        entity.set_synced_data(
            pumpkin_data::tracked_data::camel::LAST_POSE_CHANGE_TICK,
            self.last_pose_change_tick.load(Ordering::Relaxed),
        );
    }

    fn travel(&self, _caller: &dyn EntityBase) -> bool {
        let entity = self.get_entity();
        if self.refuse_to_move() && entity.on_ground.load(Ordering::Relaxed) {
            let velocity = entity.velocity.load();
            entity.velocity.store(Vector3::new(0.0, velocity.y, 0.0));
            let living = &self.mob_entity.living_entity;
            let input = living.movement_input.load();
            living.movement_input.store(Vector3::new(0.0, input.y, 0.0));
        }
        false
    }

    fn mob_passenger_attachment_point(
        &self,
        passenger: &dyn EntityBase,
        index: usize,
    ) -> Option<Vector3<f64>> {
        let entity = self.get_entity();
        let driver = index == 0;
        let scale_attribute = self
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::SCALE) as f32;
        let scale = scale_attribute * self.age_scale();
        let height = self.dimensions_for(self.is_camel_sitting()).height * scale_attribute;
        let seat_height = if entity.is_removed() {
            REMOVED_SEAT_HEIGHT
        } else {
            self.body_anchor_y_offset(driver, height, scale)
        };
        let mut offset = DRIVER_SEAT_OFFSET;
        let passenger_count = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        if passenger_count > 1 {
            if !driver {
                offset = BACK_SEAT_OFFSET;
            }
            if passenger
                .get_mob()
                .and_then(|mob| mob.as_animal())
                .is_some()
            {
                offset += ANIMAL_SEAT_SHIFT;
            }
        }
        let yaw = entity.yaw.load().to_radians();
        let forward = f64::from(offset * scale);
        Some(Vector3::new(
            -forward * f64::from(yaw.sin()),
            seat_height,
            forward * f64::from(yaw.cos()),
        ))
    }

    fn get_max_head_rotation(&self) -> f32 {
        MAX_HEAD_Y_ROT
    }

    fn ticks_look_control(&self) -> bool {
        self.get_controlling_passenger().is_none()
    }

    fn ticks_body_rotation(&self) -> bool {
        !self.refuse_to_move()
    }

    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.mob_entity.tick_brain(self);
        self.mob_entity.with_brain(self, |brain| {
            brain.set_active_activity_to_first_valid(&[
                pumpkin_data::environment_attribute::Activity::Idle,
            ]);
        });
        let wants_to_move = self
            .mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .has_wanted();
        if wants_to_move
            && !self.get_entity().is_leashed()
            && self.is_camel_sitting()
            && !self.is_in_pose_transition()
            && self.can_camel_change_pose()
        {
            self.stand_up();
        }
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if let Some(result) = self.handle_important_interactions(player, item_stack) {
            return result;
        }
        if player.get_entity().is_sneaking() && !self.is_baby() {
            self.open_custom_inventory_screen(player);
            return true;
        }
        if equip_on_target(self, player, item_stack) {
            return true;
        }
        if self.is_food(item_stack) {
            return self.fed_food(player, item_stack);
        }
        let entity = self.get_entity();
        let passengers = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        if passengers < MAX_PASSENGERS && !self.is_baby() {
            self.do_player_ride(player);
            return true;
        }
        self.is_baby()
            && item_stack.item == &Item::GOLDEN_DANDELION
            && self.animal_interact(player, item_stack)
    }
}
