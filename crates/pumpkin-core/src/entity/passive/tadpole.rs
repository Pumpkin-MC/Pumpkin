mod ai;

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering::Relaxed},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;

use super::abstract_fish::{self, AbstractFish, FishData};
use crate::entity::{
    Entity, EntityBase,
    ageable::{
        AGE_LOCK_COOLDOWN_TICKS, can_use_golden_dandelion, get_speed_up_seconds_when_feeding,
        set_age_locked,
    },
    ai::{
        brain::{Brain, memory::PackedMemories},
        control::{
            smooth_swimming_look_control::SmoothSwimmingLookControl,
            smooth_swimming_move_control::SmoothSwimmingMoveControl,
        },
    },
    bucketable::Bucketable,
    mob::{
        Mob, MobEntity,
        conversion::{ConversionParams, convert_to},
        spawn::EntitySpawnReason,
    },
    player::Player,
};
use pumpkin_data::data_component_impl::BucketEntityDataImpl;
use pumpkin_data::environment_attribute::Activity;

const TICKS_TO_BE_FROG: i32 = 24000;
const GROW_UP_SOUND_VOLUME: f32 = 0.15;
const MAX_TURN_X: i32 = 85;
const MAX_TURN_Y: i32 = 10;
const IN_WATER_SPEED_MODIFIER: f32 = 0.02;
const OUTSIDE_WATER_SPEED_MODIFIER: f32 = 0.1;
const MAX_HEAD_Y_ROT_FROM_CENTER: i32 = 10;

pub struct TadpoleEntity {
    pub mob_entity: MobEntity,
    fish_data: FishData,
    age: AtomicI32,
    age_locked: AtomicBool,
    age_lock_particle_timer: AtomicI32,
}

impl TadpoleEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        abstract_fish::initialize_fish_movement(&mut mob_entity);
        *mob_entity
            .move_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(SmoothSwimmingMoveControl::new(
                MAX_TURN_X,
                MAX_TURN_Y,
                IN_WATER_SPEED_MODIFIER,
                OUTSIDE_WATER_SPEED_MODIFIER,
                true,
            ));
        *mob_entity
            .look_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(SmoothSwimmingLookControl::new(MAX_HEAD_Y_ROT_FROM_CENTER));
        let tadpole = Arc::new(Self {
            mob_entity,
            fish_data: FishData::default(),
            age: AtomicI32::new(0),
            age_locked: AtomicBool::new(false),
            age_lock_particle_timer: AtomicI32::new(0),
        });
        abstract_fish::register_fish_goals(&tadpole.mob_entity);
        tadpole
    }

    fn is_age_locked(&self) -> bool {
        self.age_locked.load(Relaxed)
    }

    fn set_age_locked(&self, locked: bool) {
        self.age_locked.store(locked, Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::tadpole::AGE_LOCKED, locked);
    }

    fn set_age_locked_data(&self) {
        self.set_age_locked(!self.is_age_locked());
        self.set_age(0);
        self.age_lock_particle_timer
            .store(AGE_LOCK_COOLDOWN_TICKS, Relaxed);
    }

    fn set_age(&self, new_age: i32) {
        self.age.store(new_age, Relaxed);
        if new_age >= TICKS_TO_BE_FROG {
            self.age_up();
        }
    }

    fn age_up_by(&self, ticks_to_age_up: i32) {
        self.set_age(self.age.load(Relaxed) + ticks_to_age_up * 20);
    }

    fn get_ticks_left_until_adult(&self) -> i32 {
        (TICKS_TO_BE_FROG - self.age.load(Relaxed)).max(0)
    }

    fn age_up(&self) {
        let entity = self.get_entity();
        if entity
            .world
            .load()
            .get_entity_by_id(entity.entity_id)
            .is_none()
        {
            return;
        }
        let dimensions = entity.get_dimensions(entity.pose.load());
        convert_to(
            self,
            &EntityType::FROG,
            ConversionParams::single(false, false),
            |frog| {
                let world = entity.world.load_full();
                frog.finalize_spawn(&world, EntitySpawnReason::Conversion, None);
                frog.get_mob_entity()
                    .persistence_required
                    .store(true, Relaxed);
                frog.get_entity()
                    .fudge_position_after_size_change_of(frog, dimensions);
                self.play_entity_sound(Sound::EntityTadpoleGrowUp, GROW_UP_SOUND_VOLUME, 1.0);
            },
        );
    }

    fn is_food(item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_FROG_FOOD)
    }

    fn feed(&self, player: &Player, item_stack: &mut ItemStack) {
        item_stack.decrement_unless_creative(player.gamemode.load(), 1);
        self.age_up_by(get_speed_up_seconds_when_feeding(
            self.get_ticks_left_until_adult(),
        ));
    }
}

impl Mob for TadpoleEntity {
    crate::fish_mob_hooks!();

    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.mob_entity.tick_brain(self);
        self.mob_entity.with_brain(self, |brain| {
            brain.set_active_activity_to_first_valid(&[Activity::Idle]);
        });
    }

    fn mob_ai_step(&self) {
        if !self.is_age_locked() {
            self.set_age(self.age.load(Relaxed) + 1);
        }
        let timer = self.age_lock_particle_timer.load(Relaxed);
        if timer > 0 {
            self.age_lock_particle_timer.store(timer - 1, Relaxed);
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.fish_write_nbt(nbt);
        nbt.put_int("Age", self.age.load(Relaxed));
        nbt.put_bool("AgeLocked", self.is_age_locked());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.fish_read_nbt(nbt);
        self.set_age(nbt.get_int("Age").unwrap_or(0));
        self.set_age_locked(nbt.get_bool("AgeLocked").unwrap_or(false));
    }

    fn should_drop_experience(&self) -> bool {
        false
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if Self::is_food(item_stack) && !self.is_age_locked() {
            self.feed(player, item_stack);
            return true;
        }
        if can_use_golden_dandelion(
            item_stack,
            true,
            self.age_lock_particle_timer.load(Relaxed),
            self,
        ) {
            set_age_locked(
                self,
                || self.is_age_locked(),
                player,
                item_stack,
                || self.set_age_locked_data(),
            );
            return true;
        }
        self.get_mob_entity()
            .mob_interact(player, item_stack, self.can_be_leashed())
    }
}

impl crate::entity::passive::water_animal::WaterAnimal for TadpoleEntity {}

impl AbstractFish for TadpoleEntity {
    fn flop_sound(&self) -> Sound {
        Sound::EntityTadpoleFlop
    }

    fn fish_data(&self) -> &FishData {
        &self.fish_data
    }

    fn ambient_sound(&self) -> Option<Sound> {
        None
    }
}

impl crate::entity::custom_sound::CustomSound for TadpoleEntity {
    fn death_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.death_sound
    }

    fn hurt_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.hurt_sound
    }

    fn sound_category(&self) -> pumpkin_data::sound::SoundCategory {
        pumpkin_data::sound::SoundCategory::Neutral
    }
}

impl Bucketable for TadpoleEntity {
    fn is_from_bucket(&self) -> bool {
        true
    }

    fn set_from_bucket(&self, _from_bucket: bool) {}

    fn bucket_item(&self) -> &'static Item {
        &Item::TADPOLE_BUCKET
    }

    fn pickup_sound(&self) -> Sound {
        Sound::ItemBucketFillTadpole
    }

    fn save_to_bucket_tag(&self, bucket: &mut ItemStack) {
        self.save_default_data_to_bucket_tag(bucket);
        let mut nbt = bucket
            .get_data_component::<BucketEntityDataImpl>()
            .and_then(|data| data.nbt.clone())
            .unwrap_or_default();
        nbt.put_int("Age", self.age.load(Relaxed));
        nbt.put_bool("AgeLocked", self.is_age_locked());
        bucket.set_data_component(BucketEntityDataImpl { nbt: Some(nbt) });
    }

    fn load_from_bucket_tag(&self, nbt: &NbtCompound) {
        self.load_default_data_from_bucket_tag(nbt);
        if let Some(age) = nbt.get_int("Age") {
            self.set_age(age);
        }
        self.set_age_locked(nbt.get_bool("AgeLocked").unwrap_or(false));
    }
}
