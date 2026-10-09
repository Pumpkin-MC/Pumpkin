use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, AtomicU8, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    CaveVinesLikeProperties, CaveVinesPlantLikeProperties, NetherWartLikeProperties,
};
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::ageable::AgeableMob;
use crate::entity::ai::goal::active_target::ActiveTargetGoal;
use crate::entity::ai::goal::avoid_entity::AvoidEntityGoal;
use crate::entity::ai::goal::breed::BreedGoal;
use crate::entity::ai::goal::escape_danger::EscapeDangerGoal;
use crate::entity::ai::goal::follow_parent::FollowParentGoal;
use crate::entity::ai::goal::leap_at_target::LeapAtTargetGoal;
use crate::entity::ai::goal::look_at_entity::LookAtEntityGoal;
use crate::entity::ai::goal::water_avoiding_random_stroll::WaterAvoidingRandomStrollGoal;
use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::ai::util::default_random_pos;
use crate::entity::custom_sound::CustomSound;
use crate::entity::living::LivingEntity;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::passive::animal::Animal;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoxVariant {
    Red = 0,
    Snow = 1,
}

impl FoxVariant {
    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Snow,
            _ => Self::Red,
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "snow" => Self::Snow,
            _ => Self::Red,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Snow => "snow",
        }
    }

    #[must_use]
    pub fn select_for_biome(biome_name: &str) -> Self {
        if biome_name.contains("snow")
            || biome_name.contains("frozen")
            || biome_name.contains("ice")
            || biome_name.contains("grove")
            || biome_name.contains("jagged_peaks")
        {
            Self::Snow
        } else {
            Self::Red
        }
    }
}

pub mod flags {
    pub const SITTING: u8 = 1;
    pub const CROUCHING: u8 = 4;
    pub const INTERESTED: u8 = 8;
    pub const POUNCING: u8 = 16;
    pub const SLEEPING: u8 = 32;
    pub const FACEPLANTED: u8 = 64;
    pub const DEFENDING: u8 = 128;
}

pub struct FoxEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: crate::entity::ageable::AgeableData,
    variant: AtomicU8,
    flags: AtomicU8,
    trusted_0: AtomicCell<Option<Uuid>>,
    trusted_1: AtomicCell<Option<Uuid>>,
    ticks_since_eaten: AtomicI32,
    crouch_amount: AtomicCell<f32>,
    interested_angle: AtomicCell<f32>,
    ambient_sound_time: AtomicI32,
}

impl FoxEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mob_arc = Arc::new(Self {
            mob_entity,
            ageable_data: crate::entity::ageable::AgeableData::default(),
            variant: AtomicU8::new(FoxVariant::Red.id() as u8),
            flags: AtomicU8::new(0),
            trusted_0: AtomicCell::new(None),
            trusted_1: AtomicCell::new(None),
            ticks_since_eaten: AtomicI32::new(0),
            crouch_amount: AtomicCell::new(0.0),
            interested_angle: AtomicCell::new(0.0),
            ambient_sound_time: AtomicI32::new(-80),
        });

        let fox_weak = Arc::downgrade(&mob_arc);
        let dyn_mob_weak: Weak<dyn Mob> = {
            let dyn_mob: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&dyn_mob)
        };

        mob_arc.init_goals(fox_weak, dyn_mob_weak);
        mob_arc.set_target_goals();
        mob_arc.populate_default_equipment();

        mob_arc
    }

    fn init_goals(&self, fox_weak: Weak<Self>, dyn_mob_weak: Weak<dyn Mob>) {
        let mut nav = self
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        nav.set_can_float(true);
        nav.set_required_path_length(32.0);
        nav.set_pathfinding_malus(PathType::WaterBorder, 0.0);
        nav.set_pathfinding_malus(PathType::DangerOther, 0.0);
        nav.set_pathfinding_malus(PathType::DamageOther, 0.0);
        drop(nav);

        let mut goal_selector = self
            .mob_entity
            .goals_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        goal_selector.add_goal(0, Box::new(FoxFloatGoal::new(fox_weak.clone())));
        goal_selector.add_goal(
            0,
            Box::new(ClimbOnTopOfPowderSnowGoal::new(fox_weak.clone())),
        );
        goal_selector.add_goal(1, Box::new(FaceplantGoal::new(fox_weak.clone())));
        goal_selector.add_goal(2, Box::new(FoxPanicGoal::new(fox_weak.clone(), 2.2)));
        goal_selector.add_goal(3, Box::new(FoxBreedGoal::new(fox_weak.clone(), 1.0)));
        goal_selector.add_goal(4, Box::new(FoxAvoidPlayerGoal::new(fox_weak.clone())));
        goal_selector.add_goal(
            4,
            Box::new(FoxAvoidEntityGoal::new(
                fox_weak.clone(),
                &EntityType::WOLF,
                8.0,
                1.6,
                1.4,
            )),
        );
        goal_selector.add_goal(
            4,
            Box::new(FoxAvoidEntityGoal::new(
                fox_weak.clone(),
                &EntityType::POLAR_BEAR,
                8.0,
                1.6,
                1.4,
            )),
        );
        goal_selector.add_goal(5, Box::new(StalkPreyGoal::new(fox_weak.clone())));
        goal_selector.add_goal(6, Box::new(FoxPounceGoal::new(fox_weak.clone())));
        goal_selector.add_goal(6, Box::new(FoxSeekShelterGoal::new(fox_weak.clone(), 1.25)));
        goal_selector.add_goal(7, Box::new(FoxMeleeAttackGoal::new(fox_weak.clone(), 1.2)));
        goal_selector.add_goal(7, Box::new(SleepGoal::new(fox_weak.clone())));
        goal_selector.add_goal(
            8,
            Box::new(FoxFollowParentGoal::new(fox_weak.clone(), 1.25)),
        );
        goal_selector.add_goal(
            9,
            Box::new(FoxStrollThroughVillageGoal::new(fox_weak.clone(), 32, 200)),
        );
        goal_selector.add_goal(10, Box::new(FoxEatBerriesGoal::new(fox_weak.clone(), 1.2)));
        goal_selector.add_goal(10, Box::new(LeapAtTargetGoal::new(0.4)));
        goal_selector.add_goal(11, Box::new(WaterAvoidingRandomStrollGoal::new(1.0)));
        goal_selector.add_goal(11, Box::new(FoxSearchForItemsGoal::new(fox_weak.clone())));
        goal_selector.add_goal(
            12,
            Box::new(FoxLookAtPlayerGoal::new(
                fox_weak.clone(),
                dyn_mob_weak,
                24.0,
            )),
        );
        goal_selector.add_goal(13, Box::new(PerchAndSearchGoal::new(fox_weak.clone())));
        drop(goal_selector);

        let mut target_selector = self
            .mob_entity
            .target_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        target_selector.add_goal(3, Box::new(DefendTrustedTargetGoal::new(fox_weak)));
    }

    fn set_target_goals(&self) {
        let mut target_selector = self
            .mob_entity
            .target_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        target_selector.remove_goals::<ActiveTargetGoal>();

        let (land_priority, fish_priority) = match self.get_variant() {
            FoxVariant::Red => (4, 6),
            FoxVariant::Snow => (6, 4),
        };

        target_selector.add_goal(
            land_priority,
            ActiveTargetGoal::predicated(
                &self.mob_entity,
                10,
                false,
                |target: &LivingEntity, _world: &World| {
                    target.entity.entity_type == &EntityType::CHICKEN
                        || target.entity.entity_type == &EntityType::RABBIT
                },
            ),
        );
        target_selector.add_goal(
            land_priority,
            ActiveTargetGoal::predicated(
                &self.mob_entity,
                10,
                false,
                |target: &LivingEntity, _world: &World| {
                    target.entity.entity_type == &EntityType::TURTLE
                        && target.entity.age.load(Ordering::Relaxed) < 0
                        && !target.entity.is_submerged_in_water()
                },
            ),
        );
        target_selector.add_goal(
            fish_priority,
            ActiveTargetGoal::predicated(
                &self.mob_entity,
                20,
                false,
                |target: &LivingEntity, _world: &World| {
                    target.entity.entity_type == &EntityType::COD
                        || target.entity.entity_type == &EntityType::SALMON
                },
            ),
        );
    }

    fn get_flag(&self, flag: u8) -> bool {
        (self.flags.load(Ordering::Relaxed) & flag) != 0
    }

    fn set_flag(&self, flag: u8, val: bool) {
        let old = self.flags.load(Ordering::Relaxed);
        let new_val = if val { old | flag } else { old & !flag };
        self.flags.store(new_val, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(pumpkin_data::tracked_data::fox::DATA_FLAGS_ID, new_val);
    }

    #[must_use]
    pub fn get_variant(&self) -> FoxVariant {
        FoxVariant::from_id(i32::from(self.variant.load(Ordering::Relaxed)))
    }

    pub fn set_variant(&self, variant: FoxVariant) {
        self.variant.store(variant.id() as u8, Ordering::Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_TYPE_ID,
            VarInt(variant.id()),
        );
        self.set_target_goals();
    }

    pub fn is_sitting(&self) -> bool {
        self.get_flag(flags::SITTING)
    }

    pub fn set_sitting(&self, val: bool) {
        self.set_flag(flags::SITTING, val);
    }

    pub fn is_crouching(&self) -> bool {
        self.get_flag(flags::CROUCHING)
    }

    pub fn set_crouching(&self, val: bool) {
        self.set_flag(flags::CROUCHING, val);
        if !val {
            self.crouch_amount.store(0.0);
        }
    }

    pub fn is_fully_crouched(&self) -> bool {
        self.crouch_amount.load() >= 5.0
    }

    pub fn is_interested(&self) -> bool {
        self.get_flag(flags::INTERESTED)
    }

    pub fn set_interested(&self, val: bool) {
        self.set_flag(flags::INTERESTED, val);
    }

    pub fn is_pouncing(&self) -> bool {
        self.get_flag(flags::POUNCING)
    }

    pub fn set_pouncing(&self, val: bool) {
        self.set_flag(flags::POUNCING, val);
    }

    pub fn is_sleeping(&self) -> bool {
        self.get_flag(flags::SLEEPING)
    }

    pub fn set_sleeping(&self, val: bool) {
        self.set_flag(flags::SLEEPING, val);
    }

    pub fn is_faceplanted(&self) -> bool {
        self.get_flag(flags::FACEPLANTED)
    }

    pub fn set_faceplanted(&self, val: bool) {
        self.set_flag(flags::FACEPLANTED, val);
    }

    pub fn is_defending(&self) -> bool {
        self.get_flag(flags::DEFENDING)
    }

    pub fn set_defending(&self, val: bool) {
        self.set_flag(flags::DEFENDING, val);
    }

    pub fn clear_states(&self) {
        self.set_interested(false);
        self.set_crouching(false);
        self.set_sitting(false);
        self.set_sleeping(false);
        self.set_defending(false);
        self.set_faceplanted(false);
    }

    #[must_use]
    pub fn can_move(&self) -> bool {
        !self.is_sleeping() && !self.is_sitting() && !self.is_faceplanted()
    }

    pub fn wake_up(&self) {
        self.set_sleeping(false);
    }

    pub fn trusts(&self, uuid: &Uuid) -> bool {
        self.trusted_0.load().as_ref() == Some(uuid) || self.trusted_1.load().as_ref() == Some(uuid)
    }

    pub fn add_trusted(&self, uuid: Uuid) {
        if self.trusted_0.load().is_none() {
            self.trusted_0.store(Some(uuid));
            self.get_entity().set_synced_data(
                pumpkin_data::tracked_data::fox::DATA_TRUSTED_ID_0,
                Some(uuid),
            );
        } else if self.trusted_1.load().is_none() && self.trusted_0.load() != Some(uuid) {
            self.trusted_1.store(Some(uuid));
            self.get_entity().set_synced_data(
                pumpkin_data::tracked_data::fox::DATA_TRUSTED_ID_1,
                Some(uuid),
            );
        }
    }

    pub fn clear_trusted(&self) {
        self.trusted_0.store(None);
        self.trusted_1.store(None);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_TRUSTED_ID_0,
            None::<Uuid>,
        );
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_TRUSTED_ID_1,
            None::<Uuid>,
        );
    }

    #[must_use]
    pub fn get_mouth_item(&self) -> ItemStack {
        let equipment = self
            .mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        equipment
            .equipment
            .get(&EquipmentSlot::MAIN_HAND)
            .cloned()
            .unwrap_or_else(|| ItemStack::EMPTY.clone())
    }

    pub fn set_mouth_item(&self, item: ItemStack) {
        let mut equipment = self
            .mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        equipment
            .equipment
            .insert(EquipmentSlot::MAIN_HAND, item.clone());
        drop(equipment);
        self.mob_entity
            .living_entity
            .send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, item)]);
    }

    fn can_eat(&self, item: &ItemStack) -> bool {
        !item.is_empty()
            && item.has_data_component(pumpkin_data::data_component::DataComponent::Food)
            && self.mob_entity.get_target().is_none()
            && self.get_entity().on_ground.load(Ordering::Relaxed)
            && !self.is_sleeping()
    }

    pub fn can_hold_item(&self, new_item: &ItemStack) -> bool {
        let held = self.get_mouth_item();
        if held.is_empty() {
            return true;
        }
        let is_new_food =
            new_item.has_data_component(pumpkin_data::data_component::DataComponent::Food);
        let is_held_food =
            held.has_data_component(pumpkin_data::data_component::DataComponent::Food);
        self.ticks_since_eaten.load(Ordering::Relaxed) > 0 && is_new_food && !is_held_food
    }

    pub fn spit_out_item(&self, item: &ItemStack) {
        if item.is_empty() {
            return;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        world.play_sound(Sound::EntityFoxSpit, SoundCategory::Neutral, &pos);
        let look = entity.rotation();
        let spit_pos = Vector3::new(
            pos.x + f64::from(look.x),
            pos.y + 1.0,
            pos.z + f64::from(look.z),
        );
        let item_entity = crate::entity::item::ItemEntity::new(
            Entity::new(world.clone(), spit_pos, &EntityType::ITEM),
            item.clone(),
        );
        item_entity.set_pickup_delay(40);
        world.spawn_entity(Arc::new(item_entity));
    }

    fn play_ambient_sound(&self, world: &World) {
        let pos = self.get_entity().pos.load();
        if self.is_sleeping() {
            world.play_sound(Sound::EntityFoxSleep, SoundCategory::Neutral, &pos);
        } else {
            if !world.is_bright_outside() && rand::rng().random_range(0..10) == 0 {
                let player_nearby = world
                    .get_nearest_player(pos, 16.0, |p| {
                        let gm = p.gamemode.load();
                        gm != GameMode::Spectator
                    })
                    .is_some();
                if !player_nearby {
                    world.play_sound_fine(
                        Sound::EntityFoxScreech,
                        SoundCategory::Neutral,
                        &pos,
                        2.0,
                        1.0,
                    );
                    return;
                }
            }
            world.play_sound(Sound::EntityFoxAmbient, SoundCategory::Neutral, &pos);
        }
    }

    fn eat_food_in_mouth(&self, world: &World, item: &ItemStack) {
        world.play_sound(
            Sound::EntityFoxEat,
            SoundCategory::Neutral,
            &self.get_entity().pos.load(),
        );
        let mut next_item = item.clone();
        next_item.item_count = next_item.item_count.saturating_sub(1);
        if next_item.item_count == 0 {
            self.set_mouth_item(ItemStack::EMPTY.clone());
        } else {
            self.set_mouth_item(next_item);
        }
    }

    pub fn populate_default_equipment(&self) {
        if rand::random::<f32>() < 0.2 {
            let roll = rand::random::<f32>();
            let item = if roll < 0.05 {
                &Item::EMERALD
            } else if roll < 0.2 {
                &Item::EGG
            } else if roll < 0.4 {
                if rand::random::<bool>() {
                    &Item::RABBIT_FOOT
                } else {
                    &Item::RABBIT_HIDE
                }
            } else if roll < 0.6 {
                &Item::WHEAT
            } else if roll < 0.8 {
                &Item::LEATHER
            } else {
                &Item::FEATHER
            };
            self.set_mouth_item(ItemStack::new(1, item));
        }
    }

    fn fox_tick(&self) {
        let entity = self.get_entity();
        if !entity.is_alive() {
            return;
        }

        let world = entity.world.load();
        let in_water = entity.is_in_water();
        let target = self.mob_entity.get_target();

        if in_water || target.is_some() || world.is_thundering() {
            self.wake_up();
        }

        if in_water || self.is_sleeping() {
            self.set_sitting(false);
        }

        if self.is_sleeping() || self.is_crouching() {
            let living = &self.mob_entity.living_entity;
            living.jumping.store(false, Ordering::SeqCst);
            living.movement_input.store(Vector3::new(0.0, 0.0, 0.0));
            living.set_speed(0.0);
        }

        let mouth_item = self.get_mouth_item();
        if self.can_eat(&mouth_item) {
            let ticks = self.ticks_since_eaten.fetch_add(1, Ordering::Relaxed) + 1;
            if ticks > 600 {
                self.eat_food_in_mouth(&world, &mouth_item);
                self.ticks_since_eaten.store(0, Ordering::Relaxed);
            } else if ticks > 560 && rand::rng().random_range(0..10) == 0 {
                world.play_sound(
                    Sound::EntityFoxEat,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                );
                world.send_entity_status(entity, pumpkin_data::entity::EntityStatus::FoxEat, None);
            }
        } else {
            self.ticks_since_eaten.store(0, Ordering::Relaxed);
        }

        if entity.is_alive() {
            let sound_time = self.ambient_sound_time.fetch_add(1, Ordering::Relaxed);
            if sound_time > 0 && rand::rng().random_range(0..1000) < sound_time {
                self.ambient_sound_time.store(-80, Ordering::Relaxed);
                self.play_ambient_sound(&world);
            }
        }

        let target = self.mob_entity.get_target();
        if target.as_ref().is_none_or(|t| !t.is_alive()) {
            self.set_crouching(false);
            self.set_interested(false);
        }

        if self.is_crouching() {
            let amount = self.crouch_amount.load();
            let next = (amount + 0.2).min(5.0);
            self.crouch_amount.store(next);
        } else {
            self.crouch_amount.store(0.0);
        }

        let curr_angle = self.interested_angle.load();
        let target_angle = if self.is_interested() { 1.0 } else { 0.0 };
        self.interested_angle
            .store(curr_angle + (target_angle - curr_angle) * 0.4);

        if self.is_sleeping() {
            let vel = entity.velocity.load();
            entity.set_velocity(Vector3::new(0.0, vel.y.min(0.0), 0.0));
        }

        if self.is_faceplanted() && rand::random::<f32>() < 0.2 {
            let pos = entity.block_pos.load();
            let state_id = world.get_block_state_id(&pos);
            world.sync_world_event(
                WorldEvent::ParticlesAndSoundDestroyBlock,
                pos,
                state_id.as_u16() as i32,
            );
        }

        if self.is_defending() && rand::random::<f32>() < 0.05 {
            world.play_sound(
                Sound::EntityFoxAggro,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
        }
    }
}

impl AgeableMob for FoxEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }
}

impl Animal for FoxEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        let item = item_stack.get_item();
        item.has_tag(&tag::Item::MINECRAFT_FOX_FOOD)
    }
}

impl CustomSound for FoxEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityFoxDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityFoxHurt)
    }
}

impl Mob for FoxEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_max_look_yaw_change(&self) -> f32 {
        10.0
    }

    fn get_max_head_rotation(&self) -> f32 {
        75.0
    }

    fn mob_set_variant_name(&self, name: &str) {
        self.set_variant(FoxVariant::from_name(name));
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.ageable_ai_step();
    }

    fn post_tick(&self) {
        self.fox_tick();
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        if entity.age.load(Ordering::Relaxed) < 0 {
            entity.set_synced_data(pumpkin_data::tracked_data::fox::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_TYPE_ID,
            VarInt(self.get_variant().id()),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_FLAGS_ID,
            self.flags.load(Ordering::Relaxed),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_TRUSTED_ID_0,
            self.trusted_0.load(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::fox::DATA_TRUSTED_ID_1,
            self.trusted_1.load(),
        );
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("Type", self.get_variant().name().to_string());
        nbt.put_bool("Sleeping", self.is_sleeping());
        nbt.put_bool("Sitting", self.is_sitting());
        nbt.put_bool("Crouching", self.is_crouching());

        let mut trusted_list = Vec::new();
        if let Some(uuid) = self.trusted_0.load() {
            trusted_list.push(pumpkin_nbt::tag::NbtTag::String(uuid.to_string().into()));
        }
        if let Some(uuid) = self.trusted_1.load() {
            trusted_list.push(pumpkin_nbt::tag::NbtTag::String(uuid.to_string().into()));
        }
        if !trusted_list.is_empty() {
            nbt.put_list("Trusted", trusted_list);
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(variant_name) = nbt.get_string("Type") {
            self.set_variant(FoxVariant::from_name(variant_name));
        }
        self.set_sleeping(nbt.get_bool("Sleeping").unwrap_or(false));
        self.set_sitting(nbt.get_bool("Sitting").unwrap_or(false));
        self.set_crouching(nbt.get_bool("Crouching").unwrap_or(false));

        self.clear_trusted();
        if let Some(list) = nbt.get_list("Trusted") {
            for entry in list {
                if let pumpkin_nbt::tag::NbtTag::String(s) = entry
                    && let Ok(uuid) = Uuid::parse_str(s)
                {
                    self.add_trusted(uuid);
                }
            }
        }
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack, Sound::EntityFoxAmbient)
    }
}

pub fn is_path_clear(fox_pos: Vector3<f64>, target_pos: Vector3<f64>, world: &World) -> bool {
    let zdiff = target_pos.z - fox_pos.z;
    let xdiff = target_pos.x - fox_pos.x;

    for i in 0..6 {
        let factor = f64::from(i) / 6.0;
        let z = zdiff * factor;
        let x = xdiff * factor;

        for j in 1..4 {
            let check_pos = BlockPos::new(
                (fox_pos.x + x).floor() as i32,
                (fox_pos.y + f64::from(j)).floor() as i32,
                (fox_pos.z + z).floor() as i32,
            );
            let state = world.get_block_state(&check_pos);
            if !state.is_air()
                && !state.replaceable()
                && !state.is_liquid()
                && !state.collision_shapes.is_empty()
            {
                return false;
            }
        }
    }
    true
}

pub fn is_alertable(fox: &FoxEntity, world: &World) -> bool {
    let pos = fox.get_entity().pos.load();
    let entities = world.entities.load();
    for candidate in entities.iter() {
        if candidate.get_entity().entity_uuid == fox.get_entity().entity_uuid {
            continue;
        }
        let c_entity = candidate.get_entity();
        let dist_sq = pos.squared_distance_to_vec(&c_entity.pos.load());

        if (*c_entity.entity_type == EntityType::CHICKEN
            || *c_entity.entity_type == EntityType::RABBIT)
            && dist_sq <= 144.0
        {
            return true;
        }
        if *c_entity.entity_type == EntityType::POLAR_BEAR && dist_sq <= 256.0 {
            return true;
        }
        if *c_entity.entity_type == EntityType::WOLF && dist_sq <= 196.0 {
            return true;
        }
        if *c_entity.entity_type == EntityType::PLAYER
            && dist_sq <= 256.0
            && let Some(player) = candidate.get_player()
        {
            let gm = player.gamemode.load();
            if gm == GameMode::Creative || gm == GameMode::Spectator {
                continue;
            }
            if !fox.trusts(&player.gameprofile.id)
                && !player.living_entity.entity.sneaking.load(Ordering::Relaxed)
            {
                return true;
            }
        }
    }
    false
}

pub struct FoxFloatGoal {
    fox: Weak<FoxEntity>,
}

impl FoxFloatGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self { fox }
    }

    fn is_in_fluid(mob: &dyn Mob) -> bool {
        let living = &mob.get_mob_entity().living_entity;
        let entity = &living.entity;
        entity.touching_water.load(Ordering::Relaxed)
            || entity.touching_lava.load(Ordering::Relaxed)
    }
}

impl Goal for FoxFloatGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_can_float(true);
        Self::is_in_fluid(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        Self::is_in_fluid(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_can_float(true);
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade()
            && (fox.is_crouching() || fox.is_interested())
        {
            fox.set_crouching(false);
            fox.set_interested(false);
        }
        if mob.get_random().random::<f32>() < 0.8 {
            mob.get_mob_entity()
                .living_entity
                .jumping
                .store(true, Ordering::SeqCst);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::JUMP
    }
}

pub struct ClimbOnTopOfPowderSnowGoal {
    _fox: Weak<FoxEntity>,
}

impl ClimbOnTopOfPowderSnowGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self { _fox: fox }
    }
}

impl Goal for ClimbOnTopOfPowderSnowGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let pos = entity.block_pos.load();
        let block = world.get_block_state(&pos).id.to_block();
        if block.id == Block::POWDER_SNOW.id {
            let above = pos.up();
            let above_state = world.get_block_state(&above);
            let above_block = above_state.id.to_block();
            above_block.id == Block::POWDER_SNOW.id || above_state.collision_shapes.is_empty()
        } else {
            false
        }
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob)
    }

    fn tick(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity()
            .living_entity
            .jumping
            .store(true, Ordering::SeqCst);
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::JUMP
    }
}

pub struct FaceplantGoal {
    fox: Weak<FoxEntity>,
    countdown: i32,
}

impl FaceplantGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self { fox, countdown: 0 }
    }
}

impl Goal for FaceplantGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.fox.upgrade().is_some_and(|f| f.is_faceplanted())
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.countdown > 0
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.countdown = 40;
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_faceplanted(false);
        }
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.countdown -= 1;
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP
    }
}

pub struct FoxPanicGoal {
    fox: Weak<FoxEntity>,
    inner: EscapeDangerGoal,
}

impl FoxPanicGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>, speed: f64) -> Self {
        Self {
            fox,
            inner: *EscapeDangerGoal::new(speed),
        }
    }
}

impl Goal for FoxPanicGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_defending() {
            return false;
        }
        self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_defending() {
            return false;
        }
        self.inner.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.inner.tick(mob);
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

pub struct FoxBreedGoal {
    fox: Weak<FoxEntity>,
    inner: BreedGoal,
}

impl FoxBreedGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>, speed: f64) -> Self {
        Self {
            fox,
            inner: *BreedGoal::new(speed),
        }
    }
}

impl Goal for FoxBreedGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.inner.can_start(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
        self.inner.start(mob);
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.inner.should_continue(mob)
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.inner.tick(mob);
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

pub struct FoxAvoidPlayerGoal {
    fox: Weak<FoxEntity>,
    target_player: Option<Arc<Player>>,
    flee_pos: Option<Vector3<f64>>,
}

impl FoxAvoidPlayerGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            target_player: None,
            flee_pos: None,
        }
    }
}

impl Goal for FoxAvoidPlayerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_defending() {
            return false;
        }

        let pos = fox.get_entity().pos.load();
        let world = fox.get_entity().world.load();

        let threat = world.get_nearest_player(pos, 16.0, |player| {
            let gm = player.gamemode.load();
            gm != GameMode::Creative
                && gm != GameMode::Spectator
                && !player.living_entity.entity.sneaking.load(Ordering::Relaxed)
                && !fox.trusts(&player.gameprofile.id)
        });

        let Some(player) = threat else {
            return false;
        };
        let threat_pos = player.get_entity().pos.load();
        let Some(flee_pos) = default_random_pos::get_pos_away(mob, 16, 7, threat_pos) else {
            return false;
        };

        if threat_pos.squared_distance_to_vec(&flee_pos) < threat_pos.squared_distance_to_vec(&pos)
        {
            return false;
        }

        self.target_player = Some(player);
        self.flee_pos = Some(flee_pos);
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.is_navigator_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
        if let Some(flee_pos) = self.flee_pos {
            let mob_pos = mob.get_entity().pos.load();
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(mob_pos, flee_pos, 1.6));
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if let Some(player) = &self.target_player {
            let mob_pos = mob.get_entity().pos.load();
            let threat_pos = player.get_entity().pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&threat_pos);
            let speed = if dist_sq < 49.0 { 1.4 } else { 1.6 };

            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_speed(speed);
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target_player = None;
        self.flee_pos = None;
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

pub struct FoxAvoidEntityGoal {
    fox: Weak<FoxEntity>,
    inner: AvoidEntityGoal,
}

impl FoxAvoidEntityGoal {
    #[must_use]
    pub fn new(
        fox: Weak<FoxEntity>,
        target_type: &'static EntityType,
        distance: f64,
        slow_speed: f64,
        fast_speed: f64,
    ) -> Self {
        Self {
            fox,
            inner: AvoidEntityGoal::new(target_type, distance, slow_speed, fast_speed),
        }
    }
}

impl Goal for FoxAvoidEntityGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_defending() {
            return false;
        }
        self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_defending() {
            return false;
        }
        self.inner.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.inner.tick(mob);
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

pub struct StalkPreyGoal {
    fox: Weak<FoxEntity>,
    update_countdown_ticks: i32,
}

impl StalkPreyGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            update_countdown_ticks: 0,
        }
    }
}

impl Goal for StalkPreyGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_sleeping() || fox.is_crouching() || fox.is_interested() || fox.is_pouncing() {
            return false;
        }
        let Some(target) = mob.get_mob_entity().get_target() else {
            return false;
        };
        if !target.is_alive() {
            return false;
        }
        let target_entity = target.get_entity();
        let target_type = target_entity.entity_type;
        if target_type != &EntityType::CHICKEN && target_type != &EntityType::RABBIT {
            return false;
        }
        let fox_pos = fox.get_entity().pos.load();
        let target_pos = target_entity.pos.load();
        fox_pos.squared_distance_to_vec(&target_pos) > 36.0
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.update_countdown_ticks = 0;
        if let Some(fox) = self.fox.upgrade() {
            fox.set_sitting(false);
            fox.set_faceplanted(false);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let Some(fox) = self.fox.upgrade() else {
            return;
        };
        let target = mob.get_mob_entity().get_target();
        if let Some(target) = target {
            let world = fox.get_entity().world.load();
            let fox_pos = fox.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            if is_path_clear(fox_pos, target_pos, &world) {
                fox.set_interested(true);
                fox.set_crouching(true);
                let living = &mob.get_mob_entity().living_entity;
                living.set_speed(0.0);
                living.movement_input.store(Vector3::default());
                let mut vel = living.entity.velocity.load();
                vel.x = 0.0;
                vel.z = 0.0;
                living.entity.velocity.store(vel);
                let mut nav = mob
                    .get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                nav.stop();
                let mut look = mob
                    .get_mob_entity()
                    .look_control
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                look.look_at_entity(mob, &target);
                return;
            }
        }
        fox.set_interested(false);
        fox.set_crouching(false);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(fox) = self.fox.upgrade() else {
            return;
        };
        let Some(target) = mob.get_mob_entity().get_target() else {
            return;
        };

        let mut look = mob
            .get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        look.look_at_entity(mob, &target);

        let fox_pos = fox.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let dist_sq = fox_pos.squared_distance_to_vec(&target_pos);

        if dist_sq <= 36.0 {
            fox.set_interested(true);
            fox.set_crouching(true);
            let living = &mob.get_mob_entity().living_entity;
            living.set_speed(0.0);
            living.movement_input.store(Vector3::default());
            let mut vel = living.entity.velocity.load();
            vel.x = 0.0;
            vel.z = 0.0;
            living.entity.velocity.store(vel);
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.stop();
        } else {
            self.update_countdown_ticks = (self.update_countdown_ticks - 1).max(0);
            if self.update_countdown_ticks <= 0 {
                self.update_countdown_ticks = 4 + mob.get_random().random_range(0..7);
                let mut nav = mob
                    .get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                nav.set_progress(NavigatorGoal::new(fox_pos, target_pos, 1.5));
            }
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

pub struct FoxPounceGoal {
    fox: Weak<FoxEntity>,
    pounce_ticks: i32,
}

impl FoxPounceGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            pounce_ticks: 0,
        }
    }
}

impl Goal for FoxPounceGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if !fox.is_fully_crouched() {
            return false;
        }

        let Some(target) = mob.get_mob_entity().get_target() else {
            fox.set_crouching(false);
            fox.set_interested(false);
            return false;
        };
        if !target.is_alive() {
            fox.set_crouching(false);
            fox.set_interested(false);
            return false;
        }

        let world = fox.get_entity().world.load();
        let fox_pos = fox.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let has_clear_path = is_path_clear(fox_pos, target_pos, &world);
        if !has_clear_path {
            fox.set_crouching(false);
            fox.set_interested(false);
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(fox_pos, target_pos, 1.5));
            return false;
        }
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        let Some(target) = mob.get_mob_entity().get_target() else {
            return false;
        };
        let entity = fox.get_entity();
        if !target.is_alive() || fox.is_faceplanted() {
            return false;
        }
        let vel = entity.velocity.load();
        let on_ground = entity.on_ground.load(Ordering::Relaxed);
        let pitch = entity.pitch.load();
        (vel.y * vel.y >= 0.05 || pitch.abs() >= 15.0 || !on_ground) && !fox.is_faceplanted()
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.pounce_ticks = 0;
        let Some(fox) = self.fox.upgrade() else {
            return;
        };
        let living = &fox.mob_entity.living_entity;
        living.jumping.store(true, Ordering::SeqCst);
        living.set_speed(0.0);
        living.movement_input.store(Vector3::default());
        fox.set_pouncing(true);
        fox.set_interested(false);

        let entity = fox.get_entity();

        if let Some(target) = mob.get_mob_entity().get_target() {
            let fox_pos = entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            let diff = target_pos.sub(&fox_pos);
            let dist_3d = (diff.x * diff.x + diff.y * diff.y + diff.z * diff.z).sqrt();
            let (uv_x, uv_z) = if dist_3d > 1e-5 {
                (diff.x / dist_3d, diff.z / dist_3d)
            } else {
                (0.0, 0.0)
            };

            let jump_yaw = (uv_z.atan2(uv_x).to_degrees() as f32) - 90.0;
            entity.yaw.store(jump_yaw);
            entity.head_yaw.store(jump_yaw);
            entity.body_yaw.store(jump_yaw);

            entity.set_velocity(Vector3::new(uv_x * 0.8, 0.9, uv_z * 0.8));
        }

        let mut nav = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        nav.stop();
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.pounce_ticks = 0;
        if let Some(fox) = self.fox.upgrade() {
            fox.set_crouching(false);
            fox.crouch_amount.store(0.0);
            fox.set_interested(false);
            fox.set_pouncing(false);
            fox.mob_entity
                .living_entity
                .jumping
                .store(false, Ordering::SeqCst);
            fox.mob_entity
                .living_entity
                .movement_input
                .store(Vector3::default());
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.pounce_ticks += 1;
        let Some(fox) = self.fox.upgrade() else {
            return;
        };
        let entity = fox.get_entity();
        let living = &fox.mob_entity.living_entity;
        living.set_speed(0.0);
        living.movement_input.store(Vector3::default());

        let Some(target) = mob.get_mob_entity().get_target() else {
            return;
        };

        let mut look = mob
            .get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        look.look_at_entity(mob, &target);

        let vel = entity.velocity.load();
        let on_ground = entity.on_ground.load(Ordering::Relaxed);
        if !fox.is_faceplanted() {
            let cur_pitch = entity.pitch.load();
            if (vel.y * vel.y < 0.03 || on_ground) && cur_pitch != 0.0 {
                let new_pitch = cur_pitch + (0.0 - cur_pitch) * 0.2;
                entity.pitch.store(new_pitch);
            } else {
                let h_dist = vel.x.hypot(vel.z);
                let upwards_bias = if living.jumping.load(Ordering::Relaxed) && vel.y > 0.0 {
                    6.5
                } else {
                    1.0
                };
                let biased_y = vel.y * upwards_bias;
                let len = h_dist.hypot(biased_y);
                if len > 1e-5 {
                    let rotation =
                        (-biased_y).signum() * (h_dist / len).clamp(-1.0, 1.0).acos().to_degrees();
                    entity.pitch.store(rotation as f32);
                }
            }
        }

        let fox_pos = entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        let dx = target_pos.x - fox_pos.x;
        let dy = target_pos.y - fox_pos.y;
        let dz = target_pos.z - fox_pos.z;
        let dist_3d_sq = dx * dx + dy * dy + dz * dz;
        if dist_3d_sq <= 4.0 {
            mob.get_mob_entity()
                .try_attack(mob.get_entity(), target.as_ref());
            let mut cur_vel = entity.velocity.load();
            cur_vel.x *= 0.6;
            cur_vel.z *= 0.6;
            entity.velocity.store(cur_vel);
            let world = entity.world.load();
            world.play_sound(
                pumpkin_data::sound::Sound::EntityFoxBite,
                pumpkin_data::sound::SoundCategory::Neutral,
                &entity.pos.load(),
            );
            if on_ground {
                entity.pitch.store(0.0);
            }
        } else if entity.pitch.load() > 0.0
            && entity.on_ground.load(Ordering::Relaxed)
            && vel.y != 0.0
        {
            let world = entity.world.load();
            let block = world
                .get_block_state(&entity.block_pos.load())
                .id
                .to_block();
            if block.id == Block::SNOW.id {
                entity.pitch.store(60.0);
                fox.set_faceplanted(true);
                mob.get_mob_entity().set_target(None);
            }
        }
    }

    fn can_stop(&self) -> bool {
        false
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::JUMP
    }
}

pub struct FoxSeekShelterGoal {
    fox: Weak<FoxEntity>,
    speed: f64,
    interval: i32,
    shelter_pos: Option<Vector3<f64>>,
}

impl FoxSeekShelterGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>, speed: f64) -> Self {
        Self {
            fox,
            speed,
            interval: 100,
            shelter_pos: None,
        }
    }

    fn find_shelter_pos(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let mob_pos = mob.get_entity().block_pos.load();
        let world = mob.get_entity().world.load();
        let mut rng = mob.get_random();

        for _ in 0..10 {
            let offset_x = rng.random_range(-10..=10);
            let offset_y = rng.random_range(-3..=3);
            let offset_z = rng.random_range(-10..=10);
            let check_pos = BlockPos::new(
                mob_pos.0.x + offset_x,
                mob_pos.0.y + offset_y,
                mob_pos.0.z + offset_z,
            );

            let block_at = world.get_block_state(&check_pos);
            let block_above = world.get_block_state(&check_pos.up());
            let block_below = world.get_block_state(&check_pos.down());

            if !block_at.is_solid()
                && !block_above.is_solid()
                && block_below.is_solid()
                && !world.can_see_sky(&check_pos)
            {
                return Some(Vector3::new(
                    f64::from(check_pos.0.x) + 0.5,
                    f64::from(check_pos.0.y),
                    f64::from(check_pos.0.z) + 0.5,
                ));
            }
        }
        None
    }
}

impl Goal for FoxSeekShelterGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_sleeping() || mob.get_mob_entity().get_target().is_some() {
            return false;
        }

        let entity = mob.get_entity();
        let world = entity.world.load();
        let block_pos = entity.block_pos.load();

        if world.is_thundering()
            && world.can_see_sky(&block_pos)
            && let Some(pos) = Self::find_shelter_pos(mob)
        {
            self.shelter_pos = Some(pos);
            return true;
        }

        if self.interval > 0 {
            self.interval -= 1;
            return false;
        }
        self.interval = 100;

        if world.is_bright_outside()
            && world.can_see_sky(&block_pos)
            && let Some(pos) = Self::find_shelter_pos(mob)
        {
            self.shelter_pos = Some(pos);
            return true;
        }

        false
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.is_navigator_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
        if let Some(pos) = self.shelter_pos {
            let mob_pos = mob.get_entity().pos.load();
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(mob_pos, pos, self.speed));
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.shelter_pos = None;
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

pub struct FoxMeleeAttackGoal {
    fox: Weak<FoxEntity>,
    speed: f64,
    cooldown: i32,
    update_countdown_ticks: i32,
    last_target_pos: Option<Vector3<f64>>,
}

impl FoxMeleeAttackGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>, speed: f64) -> Self {
        Self {
            fox,
            speed,
            cooldown: 0,
            update_countdown_ticks: 0,
            last_target_pos: None,
        }
    }
}

impl Goal for FoxMeleeAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_sitting() || fox.is_sleeping() || fox.is_crouching() || fox.is_faceplanted() {
            return false;
        }
        mob.get_mob_entity()
            .get_target()
            .is_some_and(|t| t.is_alive())
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(target) = mob.get_mob_entity().get_target() else {
            return false;
        };
        target.is_alive()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_interested(false);
        }
        self.update_countdown_ticks = 4 + rand::rng().random_range(0..7);
        self.cooldown = 0;
        let Some(target) = mob.get_mob_entity().get_target() else {
            return;
        };
        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let mut nav = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        nav.set_progress(NavigatorGoal::new(mob_pos, target_pos, self.speed));
        self.last_target_pos = Some(target_pos);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.last_target_pos = None;
        let living = &mob.get_mob_entity().living_entity;
        living.set_speed(0.0);
        living.movement_input.store(Vector3::default());
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(target) = mob.get_mob_entity().get_target() else {
            return;
        };
        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();

        let mut look = mob
            .get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        look.look_at_entity(mob, &target);

        self.update_countdown_ticks = (self.update_countdown_ticks - 1).max(0);
        let in_range = mob.get_mob_entity().is_in_attack_range(target.as_ref());

        let should_update = self.update_countdown_ticks <= 0
            && (self
                .last_target_pos
                .is_none_or(|last| target_pos.squared_distance_to_vec(&last) >= 1.0)
                || rand::random::<f32>() < 0.05);

        if should_update {
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(mob_pos, target_pos, self.speed));
            self.last_target_pos = Some(target_pos);
            self.update_countdown_ticks = 4 + rand::rng().random_range(0..7);
            let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);
            if dist_sq > 1024.0 {
                self.update_countdown_ticks += 10;
            } else if dist_sq > 256.0 {
                self.update_countdown_ticks += 5;
            }
        }

        self.cooldown = (self.cooldown - 1).max(0);
        if self.cooldown <= 0 && in_range {
            self.cooldown = 20;
            mob.get_mob_entity()
                .try_attack(mob.get_entity(), target.as_ref());
            let world = mob.get_entity().world.load();
            world.play_sound(Sound::EntityFoxBite, SoundCategory::Neutral, &mob_pos);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

pub struct SleepGoal {
    fox: Weak<FoxEntity>,
    countdown: i32,
    alertable_timer: i32,
}

impl SleepGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            countdown: rand::rng().random_range(0..140),
            alertable_timer: 0,
        }
    }

    fn can_sleep(&mut self, fox: &FoxEntity, world: &World) -> bool {
        if self.countdown > 0 {
            self.countdown -= 1;
            return false;
        }
        let pos = fox.get_entity().block_pos.load();
        if !world.is_bright_outside() || world.can_see_sky(&pos) {
            return false;
        }
        let block_here = world.get_block_state(&pos).id.to_block();
        if block_here.id == Block::POWDER_SNOW.id {
            return false;
        }
        self.alertable_timer -= 1;
        if self.alertable_timer <= 0 {
            self.alertable_timer = 20;
            if is_alertable(fox, world) {
                return false;
            }
        }
        true
    }
}

impl Goal for SleepGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        let entity = fox.get_entity();
        if entity.is_in_water() {
            return false;
        }
        let world = entity.world.load();

        let input = fox.mob_entity.living_entity.movement_input.load();
        if input.x.abs() > 1e-4 || input.z.abs() > 1e-4 {
            return false;
        }

        self.can_sleep(&fox, &world) || fox.is_sleeping()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.get_entity().is_in_water() {
            return false;
        }
        let world = fox.get_entity().world.load();
        self.can_sleep(&fox, &world)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_sitting(false);
            fox.set_crouching(false);
            fox.set_interested(false);
            fox.mob_entity
                .living_entity
                .jumping
                .store(false, Ordering::SeqCst);
            fox.set_sleeping(true);
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.stop();
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.countdown = rand::rng().random_range(0..140);
        self.alertable_timer = 0;
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP
    }
}

pub struct FoxFollowParentGoal {
    fox: Weak<FoxEntity>,
    inner: FollowParentGoal,
}

impl FoxFollowParentGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>, speed: f64) -> Self {
        Self {
            fox,
            inner: FollowParentGoal::new(speed),
        }
    }
}

impl Goal for FoxFollowParentGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_defending() {
            return false;
        }
        self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_defending() {
            return false;
        }
        self.inner.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.inner.tick(mob);
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

pub struct FoxEatBerriesGoal {
    fox: Weak<FoxEntity>,
    speed: f64,
    target_block: Option<BlockPos>,
    ticks_waited: i32,
    cooldown: i32,
}

impl FoxEatBerriesGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>, speed: f64) -> Self {
        Self {
            fox,
            speed,
            target_block: None,
            ticks_waited: 0,
            cooldown: rand::rng().random_range(100..200),
        }
    }

    fn find_berry_bush(fox: &FoxEntity, world: &World) -> Option<BlockPos> {
        let pos = fox.get_entity().block_pos.load();
        for dy in -1..=1 {
            for dx in -12..=12 {
                for dz in -12..=12 {
                    let check_pos = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
                    let state = world.get_block_state(&check_pos);
                    let block_id = state.id.to_block().id;
                    if block_id == Block::SWEET_BERRY_BUSH.id {
                        let props = NetherWartLikeProperties::from_state_id(
                            world.get_block_state_id(&check_pos),
                        );
                        if props.age >= 2 {
                            return Some(check_pos);
                        }
                    } else if block_id == Block::CAVE_VINES.id {
                        let props = CaveVinesLikeProperties::from_state_id(
                            world.get_block_state_id(&check_pos),
                        );
                        if props.berries {
                            return Some(check_pos);
                        }
                    } else if block_id == Block::CAVE_VINES_PLANT.id {
                        let props = CaveVinesPlantLikeProperties::from_state_id(
                            world.get_block_state_id(&check_pos),
                        );
                        if props.berries {
                            return Some(check_pos);
                        }
                    }
                }
            }
        }
        None
    }

    fn harvest_berries(
        fox: &FoxEntity,
        world: &Arc<World>,
        pos: &BlockPos,
        target_vec: &Vector3<f64>,
    ) {
        let state_id = world.get_block_state_id(pos);
        let block = Block::from_state_id(state_id);
        if block.id == Block::SWEET_BERRY_BUSH.id {
            let mut props = NetherWartLikeProperties::from_state_id(state_id);
            if props.age >= 2 {
                let count: u8 = 1 + rand::rng().random_range(0..=1) + u8::from(props.age == 3);
                let mut remaining: u8 = count;

                if fox.get_mouth_item().is_empty() {
                    fox.set_mouth_item(ItemStack::new(1, &Item::SWEET_BERRIES));
                    fox.ticks_since_eaten.store(0, Ordering::Relaxed);
                    remaining = remaining.saturating_sub(1);
                }

                if remaining > 0 {
                    world.drop_stack(pos, ItemStack::new(remaining, &Item::SWEET_BERRIES));
                }

                props.age = 1;
                world.set_block_state(
                    pos,
                    props.to_state_id(&Block::SWEET_BERRY_BUSH),
                    BlockFlags::NOTIFY_ALL,
                );
            }

            world.play_sound(
                Sound::BlockSweetBerryBushPickBerries,
                SoundCategory::Blocks,
                target_vec,
            );
        } else if block.id == Block::CAVE_VINES.id {
            let mut props = CaveVinesLikeProperties::from_state_id(state_id);
            if props.berries {
                props.berries = false;
                world.set_block_state(
                    pos,
                    props.to_state_id(&Block::CAVE_VINES),
                    BlockFlags::NOTIFY_ALL,
                );
                if fox.get_mouth_item().is_empty() {
                    fox.set_mouth_item(ItemStack::new(1, &Item::GLOW_BERRIES));
                    fox.ticks_since_eaten.store(0, Ordering::Relaxed);
                } else {
                    world.drop_stack(pos, ItemStack::new(1, &Item::GLOW_BERRIES));
                }
            }

            world.play_sound(
                Sound::BlockCaveVinesPickBerries,
                SoundCategory::Blocks,
                target_vec,
            );
        } else if block.id == Block::CAVE_VINES_PLANT.id {
            let mut props = CaveVinesPlantLikeProperties::from_state_id(state_id);
            if props.berries {
                props.berries = false;
                world.set_block_state(
                    pos,
                    props.to_state_id(&Block::CAVE_VINES_PLANT),
                    BlockFlags::NOTIFY_ALL,
                );
                if fox.get_mouth_item().is_empty() {
                    fox.set_mouth_item(ItemStack::new(1, &Item::GLOW_BERRIES));
                    fox.ticks_since_eaten.store(0, Ordering::Relaxed);
                } else {
                    world.drop_stack(pos, ItemStack::new(1, &Item::GLOW_BERRIES));
                }
            }

            world.play_sound(
                Sound::BlockCaveVinesPickBerries,
                SoundCategory::Blocks,
                target_vec,
            );
        }
    }
}

impl Goal for FoxEatBerriesGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_sleeping() {
            return false;
        }
        if self.cooldown > 0 {
            self.cooldown -= 1;
            return false;
        }
        let world = fox.get_entity().world.load();
        if let Some(pos) = Self::find_berry_bush(&fox, &world) {
            self.target_block = Some(pos);
            true
        } else {
            self.cooldown = 100;
            false
        }
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.target_block.is_some() && self.ticks_waited <= 120
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
            fox.set_sitting(false);
        }
        self.ticks_waited = 0;
        if let Some(pos) = self.target_block {
            let fox_pos = mob.get_entity().pos.load();
            let target_vec = Vector3::new(
                f64::from(pos.0.x) + 0.5,
                f64::from(pos.0.y),
                f64::from(pos.0.z) + 0.5,
            );
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(fox_pos, target_vec, self.speed));
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target_block = None;
        self.ticks_waited = 0;
        self.cooldown = rand::rng().random_range(100..200);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(fox) = self.fox.upgrade() else {
            return;
        };
        let Some(pos) = self.target_block else {
            return;
        };

        let fox_pos = mob.get_entity().pos.load();
        let target_vec = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y),
            f64::from(pos.0.z) + 0.5,
        );
        let dist = fox_pos.squared_distance_to_vec(&target_vec).sqrt();

        if dist <= 2.0 {
            self.ticks_waited += 1;
            if self.ticks_waited >= 40 {
                let world = mob.get_entity().world.load_full();
                if world.level_info.load().game_rules.mob_griefing {
                    Self::harvest_berries(&fox, &world, &pos, &target_vec);
                }
                self.target_block = None;
            }
        } else if rand::random::<f32>() < 0.05 {
            let world = mob.get_entity().world.load();
            world.play_sound(Sound::EntityFoxSniff, SoundCategory::Neutral, &fox_pos);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

pub struct FoxSearchForItemsGoal {
    fox: Weak<FoxEntity>,
    target_item: Option<Arc<dyn EntityBase>>,
    update_countdown_ticks: i32,
}

impl FoxSearchForItemsGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            target_item: None,
            update_countdown_ticks: 0,
        }
    }
}

impl Goal for FoxSearchForItemsGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if !fox.get_mouth_item().is_empty() {
            return false;
        }
        if fox.mob_entity.get_target().is_some() || !fox.can_move() {
            return false;
        }
        if rand::rng().random_range(0..10) != 0 {
            return false;
        }

        let entity = fox.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        let entities = world.entities.load();
        let mut closest: Option<(f64, Arc<dyn EntityBase>)> = None;

        for candidate in entities.iter() {
            let Some(item) = candidate.get_item_entity() else {
                continue;
            };
            if item.get_pickup_delay() > 0 || !item.get_entity().is_alive() {
                continue;
            }
            let stack = item
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            if !fox.can_hold_item(&stack) {
                continue;
            }
            let dist_sq = pos.squared_distance_to_vec(&item.get_entity().pos.load());
            if dist_sq <= 64.0 {
                match &closest {
                    Some((best, _)) if dist_sq >= *best => {}
                    _ => closest = Some((dist_sq, candidate.clone())),
                }
            }
        }

        if let Some((_, item)) = closest {
            self.target_item = Some(item);
            true
        } else {
            false
        }
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if !fox.get_mouth_item().is_empty() {
            return false;
        }
        if !fox.can_move() {
            return false;
        }
        self.target_item.as_ref().is_some_and(|item| {
            if !item.get_entity().is_alive() {
                return false;
            }
            item.get_item_entity().is_some_and(|item_ent| {
                let stack = item_ent
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                fox.can_hold_item(&stack)
            })
        })
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.update_countdown_ticks = 0;
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
        if let Some(target) = &self.target_item {
            let fox_pos = mob.get_entity().pos.load();
            let item_pos = target.get_entity().pos.load();
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(fox_pos, item_pos, 1.2));
            self.update_countdown_ticks = 10;
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(fox) = self.fox.upgrade() else {
            return;
        };
        let Some(target) = &self.target_item else {
            return;
        };

        let fox_pos = mob.get_entity().pos.load();
        let item_pos = target.get_entity().pos.load();
        let dist = fox_pos.squared_distance_to_vec(&item_pos).sqrt();

        if dist <= 1.5 {
            if let Some(item_ent) = target.get_item_entity() {
                let stack = item_ent
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                if !stack.is_empty() && fox.can_hold_item(&stack) {
                    let world = mob.get_entity().world.load();
                    let prev_item = fox.get_mouth_item();
                    if !prev_item.is_empty() {
                        fox.spit_out_item(&prev_item);
                    }
                    if stack.item_count > 1 {
                        let mut remainder = stack.clone();
                        remainder.item_count = remainder.item_count.saturating_sub(1);
                        let item_entity = crate::entity::item::ItemEntity::new(
                            Entity::new(world.clone(), fox_pos, &EntityType::ITEM),
                            remainder,
                        );
                        world.spawn_entity(Arc::new(item_entity));
                    }
                    let mut single = stack;
                    single.item_count = 1;
                    fox.set_mouth_item(single);
                    fox.ticks_since_eaten.store(0, Ordering::Relaxed);
                    target.get_entity().remove();
                }
            }
            self.target_item = None;
        } else {
            self.update_countdown_ticks = (self.update_countdown_ticks - 1).max(0);
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if nav.is_idle() || self.update_countdown_ticks <= 0 {
                self.update_countdown_ticks = 10;
                nav.set_progress(NavigatorGoal::new(fox_pos, item_pos, 1.2));
            }
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target_item = None;
        self.update_countdown_ticks = 0;
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

pub struct FoxLookAtPlayerGoal {
    fox: Weak<FoxEntity>,
    inner: LookAtEntityGoal,
}

impl FoxLookAtPlayerGoal {
    #[must_use]
    pub fn new(fox_weak: Weak<FoxEntity>, mob_weak: Weak<dyn Mob>, range: f32) -> Self {
        Self {
            fox: fox_weak,
            inner: LookAtEntityGoal::new(mob_weak, &EntityType::PLAYER, range, 0.02, false),
        }
    }
}

impl Goal for FoxLookAtPlayerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_faceplanted() || fox.is_interested() {
            return false;
        }
        self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_faceplanted() || fox.is_interested() {
            return false;
        }
        self.inner.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.inner.tick(mob);
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

pub struct PerchAndSearchGoal {
    fox: Weak<FoxEntity>,
    rel_x: f64,
    rel_z: f64,
    look_time: i32,
    looks_remaining: i32,
}

impl PerchAndSearchGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            rel_x: 0.0,
            rel_z: 0.0,
            look_time: 0,
            looks_remaining: 0,
        }
    }

    fn reset_look(&mut self) {
        let rnd = 2.0 * std::f64::consts::PI * rand::rng().random::<f64>();
        self.rel_x = rnd.cos();
        self.rel_z = rnd.sin();
        self.look_time = 80 + rand::rng().random_range(0..20);
    }
}

impl Goal for PerchAndSearchGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_sleeping()
            || fox.is_pouncing()
            || fox.is_crouching()
            || fox.get_entity().is_in_water()
            || mob.get_mob_entity().get_target().is_some()
            || mob
                .get_mob_entity()
                .living_entity
                .last_attacker_id
                .load(Ordering::Relaxed)
                != 0
            || !mob.is_navigator_idle()
            || rand::random::<f32>() >= 0.02
        {
            return false;
        }

        let world = fox.get_entity().world.load();
        !is_alertable(&fox, &world)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        self.looks_remaining > 0
            && !fox.get_entity().is_in_water()
            && !fox.is_sleeping()
            && mob.get_mob_entity().get_target().is_none()
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.reset_look();
        self.looks_remaining = 2 + rand::rng().random_range(0..3);
        if let Some(fox) = self.fox.upgrade() {
            fox.set_sitting(true);
        }
        let mut nav = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        nav.stop();
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_sitting(false);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.look_time -= 1;
        if self.look_time <= 0 {
            self.looks_remaining -= 1;
            self.reset_look();
        }

        let entity = mob.get_entity();
        let pos = entity.pos.load();
        let eye_y = entity.get_eye_pos().y;
        let look_target = Vector3::new(pos.x + self.rel_x * 5.0, eye_y, pos.z + self.rel_z * 5.0);

        let mut look = mob
            .get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        look.look_at_position(mob, look_target);
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

pub struct DefendTrustedTargetGoal {
    fox: Weak<FoxEntity>,
    target_attacker: Option<Arc<dyn EntityBase>>,
}

impl DefendTrustedTargetGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            target_attacker: None,
        }
    }
}

impl Goal for DefendTrustedTargetGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        let world = fox.get_entity().world.load();

        let trusted_uuids = [fox.trusted_0.load(), fox.trusted_1.load()];
        for opt_uuid in trusted_uuids {
            let Some(uuid) = opt_uuid else {
                continue;
            };
            if let Some(player) = world.get_player_by_uuid(uuid) {
                let attacker_id = player
                    .living_entity
                    .last_attacker_id
                    .load(Ordering::Relaxed);
                if attacker_id != 0
                    && let Some(attacker_ent) = world.get_entity_by_id(attacker_id)
                {
                    let att = attacker_ent.get_entity();
                    if attacker_ent.is_alive() && !fox.trusts(&att.entity_uuid) {
                        self.target_attacker = Some(attacker_ent);
                        return true;
                    }
                }
            } else {
                let entities = world.entities.load();
                for candidate in entities.iter() {
                    let c_ent = candidate.get_entity();
                    if c_ent.entity_uuid == uuid {
                        if let Some(living) = candidate.get_living_entity() {
                            let attacker_id = living.last_attacker_id.load(Ordering::Relaxed);
                            if attacker_id != 0
                                && let Some(attacker_ent) = world.get_entity_by_id(attacker_id)
                            {
                                let att = attacker_ent.get_entity();
                                if attacker_ent.is_alive() && !fox.trusts(&att.entity_uuid) {
                                    self.target_attacker = Some(attacker_ent);
                                    return true;
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
        false
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.target_attacker.as_ref().is_some_and(|t| t.is_alive())
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = &self.target_attacker {
            mob.get_mob_entity().set_target(Some(target.clone()));
            if let Some(fox) = self.fox.upgrade() {
                fox.set_defending(true);
                fox.wake_up();
                let pos = mob.get_entity().pos.load();
                mob.get_entity().world.load().play_sound(
                    Sound::EntityFoxAggro,
                    SoundCategory::Neutral,
                    &pos,
                );
            }
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_defending(false);
        }
        self.target_attacker = None;
    }

    fn controls(&self) -> Controls {
        Controls::TARGET
    }
}

pub struct FoxStrollThroughVillageGoal {
    fox: Weak<FoxEntity>,
    #[expect(dead_code)]
    search_radius: i32,
    interval: i32,
    target_pos: Option<Vector3<f64>>,
}

impl FoxStrollThroughVillageGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>, search_radius: i32, interval: i32) -> Self {
        Self {
            fox,
            search_radius,
            interval,
            target_pos: None,
        }
    }

    fn can_fox_move(fox: &FoxEntity) -> bool {
        !fox.is_sleeping()
            && !fox.is_sitting()
            && !fox.is_defending()
            && fox.mob_entity.get_target().is_none()
            && !fox.get_entity().is_in_water()
    }
}

impl Goal for FoxStrollThroughVillageGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if !Self::can_fox_move(&fox) {
            return false;
        }

        let entity = fox.get_entity();
        let world = entity.world.load();
        let is_day = (world.get_time_of_day() % 24000) < 12000;
        if is_day {
            return false;
        }

        if rand::rng().random_range(0..self.interval) != 0 {
            return false;
        }

        let fox_pos = entity.pos.load();
        let entities = world.entities.load();
        let near_village = entities.iter().any(|e| {
            e.get_entity().entity_type == &EntityType::VILLAGER
                && fox_pos.squared_distance_to_vec(&e.get_entity().pos.load()) <= (32.0 * 32.0)
        });

        if !near_village {
            return false;
        }

        if let Some(target) = default_random_pos::get_pos(mob, 15, 7) {
            self.target_pos = Some(target);
            true
        } else {
            false
        }
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        Self::can_fox_move(&fox) && !mob.is_navigator_idle() && self.target_pos.is_some()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
        if let Some(target) = self.target_pos {
            let mob_pos = mob.get_entity().pos.load();
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(mob_pos, target, 1.0));
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target_pos = None;
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
