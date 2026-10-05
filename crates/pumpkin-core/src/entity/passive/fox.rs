use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, AtomicU8, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::Block;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
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
use crate::entity::ai::goal::swim::SwimGoal;
use crate::entity::ai::goal::water_avoiding_random_stroll::WaterAvoidingRandomStrollGoal;
use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
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
    crouch_ticks: AtomicI32,
    interested_angle: AtomicCell<f32>,
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
            crouch_ticks: AtomicI32::new(0),
            interested_angle: AtomicCell::new(0.0),
        });

        let fox_weak = Arc::downgrade(&mob_arc);
        let dyn_mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
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
                    1.4,
                    1.6,
                )),
            );
            goal_selector.add_goal(
                4,
                Box::new(FoxAvoidEntityGoal::new(
                    fox_weak.clone(),
                    &EntityType::POLAR_BEAR,
                    8.0,
                    1.4,
                    1.6,
                )),
            );
            goal_selector.add_goal(5, Box::new(StalkPreyGoal::new(fox_weak.clone())));
            goal_selector.add_goal(6, Box::new(FoxPounceGoal::new(fox_weak.clone())));
            goal_selector.add_goal(6, Box::new(FoxSeekShelterGoal::new(fox_weak.clone(), 1.25)));
            goal_selector.add_goal(7, Box::new(FoxMeleeAttackGoal::new(fox_weak.clone(), 1.2)));
            goal_selector.add_goal(7, Box::new(SleepGoal::new(fox_weak.clone())));
            goal_selector.add_goal(8, Box::new(FollowParentGoal::new(1.25)));
            goal_selector.add_goal(10, Box::new(FoxEatBerriesGoal::new(fox_weak.clone(), 1.2)));
            goal_selector.add_goal(10, Box::new(LeapAtTargetGoal::new(0.4)));
            goal_selector.add_goal(11, Box::new(FoxStrollGoal::new(fox_weak.clone(), 1.0)));
            goal_selector.add_goal(11, Box::new(FoxSearchForItemsGoal::new(fox_weak.clone())));
            goal_selector.add_goal(
                12,
                Box::new(FoxLookAtPlayerGoal::new(fox_weak.clone(), dyn_mob_weak, 16.0)),
            );
            goal_selector.add_goal(13, Box::new(PerchAndSearchGoal::new(fox_weak.clone())));
        }

        {
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            target_selector.add_goal(3, Box::new(DefendTrustedTargetGoal::new(fox_weak)));
            target_selector.add_goal(
                4,
                ActiveTargetGoal::predicated(
                    &mob_arc.mob_entity,
                    10,
                    false,
                    |target: &LivingEntity, _world: &World| {
                        target.entity.entity_type == &EntityType::CHICKEN
                            || target.entity.entity_type == &EntityType::RABBIT
                    },
                ),
            );
            target_selector.add_goal(
                4,
                ActiveTargetGoal::predicated(
                    &mob_arc.mob_entity,
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
                6,
                ActiveTargetGoal::predicated(
                    &mob_arc.mob_entity,
                    20,
                    false,
                    |target: &LivingEntity, _world: &World| {
                        target.entity.entity_type == &EntityType::COD
                            || target.entity.entity_type == &EntityType::SALMON
                    },
                ),
            );
        }

        mob_arc
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
            self.crouch_ticks.store(0, Ordering::Relaxed);
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
        self.trusted_0.load().as_ref() == Some(uuid)
            || self.trusted_1.load().as_ref() == Some(uuid)
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
            .insert(EquipmentSlot::MAIN_HAND, item);
    }

    fn can_eat(&self, item: &ItemStack) -> bool {
        !item.is_empty()
            && item.has_data_component(pumpkin_data::data_component::DataComponent::Food)
            && self.mob_entity.get_target().is_none()
            && self.get_entity().on_ground.load(Ordering::Relaxed)
            && !self.is_sleeping()
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

        // 1:1 Vanilla Fox.java:565-573
        if in_water || target.is_some() || world.is_thundering() {
            self.wake_up();
        }

        if in_water || self.is_sleeping() {
            self.set_sitting(false);
        }

        if self.is_sleeping() {
            let living = &self.mob_entity.living_entity;
            living.jumping.store(false, Ordering::SeqCst);
            living.movement_input.store(Vector3::new(0.0, 0.0, 0.0));
        }

        let ticks = self.ticks_since_eaten.fetch_add(1, Ordering::Relaxed) + 1;

        let mouth_item = self.get_mouth_item();
        if self.can_eat(&mouth_item) {
            if ticks > 600 {
                self.eat_food_in_mouth(&world, &mouth_item);
                self.ticks_since_eaten.store(0, Ordering::Relaxed);
            } else if ticks > 560 && rand::random::<f32>() < 0.1 {
                world.play_sound(
                    Sound::EntityFoxEat,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                );
                world.send_entity_status(entity, pumpkin_data::entity::EntityStatus::FoxEat, None);
            }
        }

        let target = self.mob_entity.get_target();
        if entity.is_in_water()
            || target
                .as_ref()
                .is_none_or(|t| !t.get_entity().is_alive() || t.get_entity().is_in_water())
        {
            self.set_crouching(false);
            self.set_interested(false);
        }

        // Aiming (crouching) animation logic with safety timeout against infinite stalling
        if self.is_crouching() {
            let amount = self.crouch_amount.load();
            let next = (amount + 0.2).min(5.0);
            self.crouch_amount.store(next);

            let c_ticks = self.crouch_ticks.fetch_add(1, Ordering::Relaxed) + 1;
            // Safety timeout: If crouched for > 35 ticks (~1.75s) and pounce didn't start, reset!
            if c_ticks > 35 {
                self.set_crouching(false);
                self.set_interested(false);
                self.crouch_amount.store(0.0);
                self.crouch_ticks.store(0, Ordering::Relaxed);
            } else if let Some(ref t) = target {
                let mut look = self
                    .mob_entity
                    .look_control
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                look.look_at_entity(self, t);
            }
        } else {
            self.crouch_amount.store(0.0);
            self.crouch_ticks.store(0, Ordering::Relaxed);
        }

        let curr_angle = self.interested_angle.load();
        let target_angle = if self.is_interested() { 1.0 } else { 0.0 };
        self.interested_angle
            .store(curr_angle + (target_angle - curr_angle) * 0.4);

        if self.is_sleeping() {
            let vel = entity.velocity.load();
            entity.set_velocity(Vector3::new(0.0, vel.y.min(0.0), 0.0));
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
                if let pumpkin_nbt::tag::NbtTag::String(s) = entry {
                    if let Ok(uuid) = Uuid::parse_str(s) {
                        self.add_trusted(uuid);
                    }
                }
            }
        }
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack, Sound::EntityFoxAmbient)
    }
}

pub fn is_path_clear(fox_pos: Vector3<f64>, target_pos: Vector3<f64>, world: &World) -> bool {
    let diff = target_pos.sub(&fox_pos);
    let dist_sq = diff.x * diff.x + diff.z * diff.z;
    if dist_sq <= 9.0 {
        return true;
    }

    let xdiff = diff.x;
    let zdiff = diff.z;

    for i in 0..6 {
        let factor = f64::from(i) / 6.0;
        let x = xdiff * factor;
        let z = zdiff * factor;

        for j in 1..4 {
            let check_pos = BlockPos::new(
                (fox_pos.x + x).floor() as i32,
                (fox_pos.y + f64::from(j)).floor() as i32,
                (fox_pos.z + z).floor() as i32,
            );
            let state = world.get_block_state(&check_pos);
            if !state.is_air() && !state.replaceable() && !state.is_liquid() {
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
        if candidate.get_entity().entity_uuid == fox.get_entity().entity_uuid { continue; }
        let c_entity = candidate.get_entity();
        let dist_sq = pos.squared_distance_to_vec(&c_entity.pos.load());

        if *c_entity.entity_type == EntityType::POLAR_BEAR && dist_sq <= 256.0 { return true; }
        if *c_entity.entity_type == EntityType::WOLF && dist_sq <= 196.0 { return true; }
        if *c_entity.entity_type == EntityType::PLAYER && dist_sq <= 256.0 {
            if let Some(player) = candidate.get_player() {
                let gm = player.gamemode.load();
                if gm == GameMode::Creative || gm == GameMode::Spectator { continue; }
                if !fox.trusts(&player.gameprofile.id) && !player.living_entity.entity.sneaking.load(Ordering::Relaxed) {
                    return true;
                }
            }
        }
    }
    false
}

pub struct FaceplantGoal {
    fox: Weak<FoxEntity>,
    countdown: i32,
}

impl FaceplantGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            countdown: 0,
        }
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
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.is_defending() { return false; }
        self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.is_defending() { return false; }
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
    pub fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            target_player: None,
            flee_pos: None,
        }
    }
}

impl Goal for FoxAvoidPlayerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.is_defending() { return false; }

        let pos = fox.get_entity().pos.load();
        let world = fox.get_entity().world.load();

        let threat = world.get_nearest_player(pos, 16.0, |player| {
            let gm = player.gamemode.load();
            gm != GameMode::Creative
                && gm != GameMode::Spectator
                && !player.living_entity.entity.sneaking.load(Ordering::Relaxed)
                && !fox.trusts(&player.gameprofile.id)
        });

        let Some(player) = threat else { return false; };
        let threat_pos = player.get_entity().pos.load();
        let Some(flee_pos) = default_random_pos::get_pos_away(mob, 16, 7, threat_pos) else {
            return false;
        };

        if threat_pos.squared_distance_to_vec(&flee_pos) < threat_pos.squared_distance_to_vec(&pos) {
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
        if let Some(flee_pos) = self.flee_pos {
            let mob_pos = mob.get_entity().pos.load();
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(mob_pos, flee_pos, 1.4));
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if let Some(player) = &self.target_player {
            let mob_pos = mob.get_entity().pos.load();
            let threat_pos = player.get_entity().pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&threat_pos);
            let speed = if dist_sq < 49.0 { 1.6 } else { 1.4 };

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
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.is_defending() { return false; }
        self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.is_defending() { return false; }
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

pub struct StalkPreyGoal {
    fox: Weak<FoxEntity>,
}

impl StalkPreyGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>) -> Self {
        Self { fox }
    }
}

impl Goal for StalkPreyGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.is_sleeping() || fox.is_crouching() || fox.is_interested() || fox.get_entity().is_in_water() {
            return false;
        }
        let Some(target) = mob.get_mob_entity().get_target() else { return false; };
        let target_entity = target.get_entity();
        if !target_entity.is_alive() || target_entity.is_in_water() { return false; }
        let target_type = target_entity.entity_type;
        if target_type != &EntityType::CHICKEN && target_type != &EntityType::RABBIT { return false; }
        fox.get_entity().pos.load().squared_distance_to_vec(&target_entity.pos.load()) > 36.0
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_sitting(false);
            fox.set_faceplanted(false);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let Some(fox) = self.fox.upgrade() else { return; };
        let target = mob.get_mob_entity().get_target();
        if let Some(target) = target {
            let world = fox.get_entity().world.load();
            let fox_pos = fox.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            if !fox.get_entity().is_in_water()
                && !target.get_entity().is_in_water()
                && is_path_clear(fox_pos, target_pos, &world)
            {
                fox.set_interested(true);
                fox.set_crouching(true);
                let mut nav = mob.get_mob_entity().navigator.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                nav.stop();
                let mut look = mob.get_mob_entity().look_control.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                look.look_at_entity(mob, &target);
                return;
            }
        }
        fox.set_interested(false);
        fox.set_crouching(false);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(fox) = self.fox.upgrade() else { return; };
        let Some(target) = mob.get_mob_entity().get_target() else { return; };

        let mut look = mob.get_mob_entity().look_control.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        look.look_at_entity(mob, &target);

        let fox_pos = fox.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let dist_sq = fox_pos.squared_distance_to_vec(&target_pos);

        if dist_sq <= 36.0 && !fox.get_entity().is_in_water() && !target.get_entity().is_in_water() {
            fox.set_interested(true);
            fox.set_crouching(true);
            let mut nav = mob.get_mob_entity().navigator.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.stop();
        } else {
            let mut nav = mob.get_mob_entity().navigator.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(fox_pos, target_pos, 1.5));
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

pub struct FoxPounceGoal {
    fox: Weak<FoxEntity>,
}

impl FoxPounceGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>) -> Self {
        Self { fox }
    }
}

impl Goal for FoxPounceGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        if !fox.is_fully_crouched() || fox.get_entity().is_in_water() {
            if fox.get_entity().is_in_water() {
                fox.set_crouching(false);
                fox.set_interested(false);
            }
            return false;
        }

        let Some(target) = mob.get_mob_entity().get_target() else {
            fox.set_crouching(false);
            fox.set_interested(false);
            return false;
        };
        if !target.get_entity().is_alive() || target.get_entity().is_in_water() {
            fox.set_crouching(false);
            fox.set_interested(false);
            return false;
        }

        let world = fox.get_entity().world.load();
        let fox_pos = fox.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        if fox_pos.squared_distance_to_vec(&target_pos) > 49.0 || !is_path_clear(fox_pos, target_pos, &world) {
            fox.set_crouching(false);
            fox.set_interested(false);
            return false;
        }
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        let Some(target) = mob.get_mob_entity().get_target() else { return false; };
        let entity = fox.get_entity();
        if !target.get_entity().is_alive() || fox.is_faceplanted() || entity.is_in_water() || entity.on_ground.load(Ordering::Relaxed) {
            return false;
        }
        let vel = entity.velocity.load();
        vel.y * vel.y >= 0.05
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(fox) = self.fox.upgrade() else { return };
        // The crouch is now converted into the leap! Clear aiming stance immediately:
        fox.set_crouching(false);
        fox.crouch_amount.store(0.0);
        fox.crouch_ticks.store(0, Ordering::Relaxed);
        fox.set_pouncing(true);
        fox.set_interested(false);

        let entity = fox.get_entity();
        entity.on_ground.store(false, Ordering::Relaxed);

        if let Some(target) = mob.get_mob_entity().get_target() {
            let fox_pos = entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            let diff = target_pos.sub(&fox_pos);
            let dist_horiz = (diff.x * diff.x + diff.z * diff.z).sqrt();
            let (uv_x, uv_z) = if dist_horiz > 1e-5 {
                (diff.x / dist_horiz, diff.z / dist_horiz)
            } else {
                (0.0, 0.0)
            };

            // Align entity yaw with the leap trajectory so it never snaps abruptly
            let jump_yaw = (uv_z.atan2(uv_x).to_degrees() as f32) - 90.0;
            entity.yaw.store(jump_yaw);
            entity.head_yaw.store(jump_yaw);

            entity.set_velocity(Vector3::new(
                uv_x * 0.85,
                0.9,
                uv_z * 0.85,
            ));
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
            fox.set_crouching(false);
            fox.crouch_amount.store(0.0);
            fox.crouch_ticks.store(0, Ordering::Relaxed);
            fox.set_interested(false);
            fox.set_pouncing(false);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(fox) = self.fox.upgrade() else { return };
        let entity = fox.get_entity();
        let Some(target) = mob.get_mob_entity().get_target() else { return };

        let mut look = mob
            .get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        look.look_at_entity(mob, &target);

        let fox_pos = entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        let dx = target_pos.x - fox_pos.x;
        let dy = target_pos.y - fox_pos.y;
        let dz = target_pos.z - fox_pos.z;
        let dist_3d_sq = dx * dx + dy * dy + dz * dz;
        let dist_h_sq = dx * dx + dz * dz;

        // Mid-air bite hit check (matches vanilla, robust against height offset during apex)
        if dist_3d_sq <= 4.84 || (dist_h_sq <= 4.0 && dy.abs() <= 2.2) {
            mob.get_mob_entity()
                .try_attack(mob.get_entity(), target.as_ref());
            let world = entity.world.load();
            world.play_sound(
                Sound::EntityFoxBite,
                SoundCategory::Neutral,
                &fox_pos,
            );
        } else if entity.on_ground.load(Ordering::Relaxed) {
            let world = entity.world.load();
            let block = world.get_block_state(&entity.block_pos.load()).id.to_block();
            if block.id == Block::SNOW.id || block.id == Block::SNOW_BLOCK.id {
                fox.set_faceplanted(true);
                mob.get_mob_entity().set_target(None);
            }
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::JUMP | Controls::LOOK
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
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.is_sleeping() || mob.get_mob_entity().get_target().is_some() {
            return false;
        }

        let entity = mob.get_entity();
        let world = entity.world.load();
        let block_pos = entity.block_pos.load();

        if world.is_thundering() && world.can_see_sky(&block_pos) {
            if let Some(pos) = Self::find_shelter_pos(mob) {
                self.shelter_pos = Some(pos);
                return true;
            }
        }

        if self.interval > 0 {
            self.interval -= 1;
            return false;
        }
        self.interval = 100;

        if world.is_bright_outside() && world.can_see_sky(&block_pos) {
            if let Some(pos) = Self::find_shelter_pos(mob) {
                self.shelter_pos = Some(pos);
                return true;
            }
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
    pub fn new(fox: Weak<FoxEntity>, speed: f64) -> Self {
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
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.is_sitting() || fox.is_sleeping() || fox.is_crouching() || fox.is_faceplanted() {
            return false;
        }
        mob.get_mob_entity().get_target().is_some_and(|t| t.get_entity().is_alive())
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_interested(false);
        }
        self.update_countdown_ticks = 0;
        self.last_target_pos = None;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.last_target_pos = None;
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(target) = mob.get_mob_entity().get_target() else { return; };
        let entity = mob.get_entity();
        let mob_pos = entity.pos.load();
        let target_pos = target.get_entity().pos.load();

        let mut look = mob
            .get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        look.look_at_entity(mob, &target);

        // Path update throttling to prevent resetting navigation on every tick
        self.update_countdown_ticks -= 1;
        let should_update = self.update_countdown_ticks <= 0
            && (self.last_target_pos.is_none_or(|last| {
                target_pos.squared_distance_to_vec(&last) >= 1.0
            }) || rand::random::<f32>() < 0.05);

        if should_update {
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(mob_pos, target_pos, self.speed));
            self.last_target_pos = Some(target_pos);
            self.update_countdown_ticks = 4 + rand::rng().random_range(0..7);
        }

        if self.cooldown > 0 {
            self.cooldown -= 1;
        } else if mob.get_mob_entity().is_in_attack_range(target.as_ref()) {
            self.cooldown = 20;
            mob.get_mob_entity()
                .try_attack(mob.get_entity(), target.as_ref());
            let world = mob.get_entity().world.load();
            world.play_sound(
                Sound::EntityFoxBite,
                SoundCategory::Neutral,
                &mob_pos,
            );
        }
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
        let Some(fox) = self.fox.upgrade() else { return false; };
        let entity = fox.get_entity();
        if entity.is_in_water() { return false; }
        let world = entity.world.load();

        let vel = entity.velocity.load();
        if vel.x * vel.x + vel.z * vel.z >= 0.001 { return false; }

        self.can_sleep(&fox, &world) || fox.is_sleeping()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.get_entity().is_in_water() { return false; }
        let world = fox.get_entity().world.load();
        self.can_sleep(&fox, &world)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_sitting(false);
            fox.set_crouching(false);
            fox.set_interested(false);
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

    fn find_berry_bush(&self, fox: &FoxEntity, world: &World) -> Option<BlockPos> {
        let pos = fox.get_entity().block_pos.load();
        for dy in -1..=1 {
            for dx in -8..=8 {
                for dz in -8..=8 {
                    let check_pos = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
                    let state = world.get_block_state(&check_pos);
                    if state.id.to_block().id == Block::SWEET_BERRY_BUSH.id {
                        return Some(check_pos);
                    }
                }
            }
        }
        None
    }
}

impl Goal for FoxEatBerriesGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.is_sleeping() || fox.is_sitting() || fox.is_crouching() || fox.is_faceplanted() || fox.mob_entity.get_target().is_some() {
            return false;
        }
        if self.cooldown > 0 {
            self.cooldown -= 1;
            return false;
        }
        let world = fox.get_entity().world.load();
        if let Some(pos) = self.find_berry_bush(&fox, &world) {
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
        let Some(fox) = self.fox.upgrade() else { return; };
        let Some(pos) = self.target_block else { return; };

        let fox_pos = mob.get_entity().pos.load();
        let target_vec = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y),
            f64::from(pos.0.z) + 0.5,
        );
        let dist = fox_pos.squared_distance_to_vec(&target_vec).sqrt();

        if dist <= 1.5 {
            self.ticks_waited += 1;
            let world = mob.get_entity().world.load();

            if self.ticks_waited == 20 {
                world.play_sound(
                    Sound::BlockSweetBerryBushPickBerries,
                    SoundCategory::Blocks,
                    &target_vec,
                );
                if fox.get_mouth_item().is_empty() {
                    fox.set_mouth_item(ItemStack::new(1, &Item::SWEET_BERRIES));
                }
            } else if self.ticks_waited > 20 && self.ticks_waited % 10 == 0 {
                world.play_sound(
                    Sound::EntityFoxEat,
                    SoundCategory::Neutral,
                    &fox_pos,
                );
            }

            if self.ticks_waited >= 40 {
                self.target_block = None;
            }
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

pub struct FoxStrollGoal {
    fox: Weak<FoxEntity>,
    stroll_goal: WaterAvoidingRandomStrollGoal,
}

impl FoxStrollGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>, speed: f64) -> Self {
        Self {
            fox,
            stroll_goal: WaterAvoidingRandomStrollGoal::new(speed),
        }
    }
}

impl Goal for FoxStrollGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        if !fox.can_move()
            || fox.is_crouching()
            || fox.is_pouncing()
            || mob.get_mob_entity().get_target().is_some()
        {
            return false;
        }
        self.stroll_goal.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        if !fox.can_move()
            || fox.is_crouching()
            || fox.is_pouncing()
            || mob.get_mob_entity().get_target().is_some()
        {
            return false;
        }
        self.stroll_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.stroll_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.stroll_goal.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.stroll_goal.controls()
    }
}

pub struct FoxSearchForItemsGoal {
    fox: Weak<FoxEntity>,
    target_item: Option<Arc<dyn EntityBase>>,
}

impl FoxSearchForItemsGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            target_item: None,
        }
    }
}

impl Goal for FoxSearchForItemsGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        if !fox.get_mouth_item().is_empty() || fox.mob_entity.get_target().is_some() || !fox.can_move() {
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
            let Some(item) = candidate.get_item_entity() else { continue; };
            if item.get_pickup_delay() > 0 || !item.get_entity().is_alive() {
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
        let Some(fox) = self.fox.upgrade() else { return false; };
        if !fox.get_mouth_item().is_empty() {
            return false;
        }
        self.target_item.as_ref().is_some_and(|item| item.get_entity().is_alive())
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = &self.target_item {
            let fox_pos = mob.get_entity().pos.load();
            let item_pos = target.get_entity().pos.load();
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(fox_pos, item_pos, 1.2));
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(fox) = self.fox.upgrade() else { return; };
        let Some(target) = &self.target_item else { return; };

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
                if !stack.is_empty() {
                    fox.set_mouth_item(stack);
                    target.get_entity().remove();
                }
            }
            self.target_item = None;
        } else {
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(fox_pos, item_pos, 1.2));
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target_item = None;
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
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.is_faceplanted() || fox.is_interested() || fox.is_crouching() || fox.is_sleeping() {
            return false;
        }
        self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.is_faceplanted() || fox.is_interested() || fox.is_crouching() || fox.is_sleeping() {
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
    pub fn new(fox: Weak<FoxEntity>) -> Self {
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
        self.look_time = 40 + rand::rng().random_range(0..10);
    }
}

impl Goal for PerchAndSearchGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        if fox.is_sleeping()
            || fox.is_pouncing()
            || fox.is_crouching()
            || fox.get_entity().is_in_water()
            || mob.get_mob_entity().get_target().is_some()
            || !mob.is_navigator_idle()
            || rand::random::<f32>() >= 0.02
        {
            return false;
        }

        let world = fox.get_entity().world.load();
        !is_alertable(&fox, &world)
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else { return false; };
        self.looks_remaining > 0 && !fox.get_entity().is_in_water() && !fox.is_sleeping()
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
        Controls::MOVE
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
        let Some(fox) = self.fox.upgrade() else { return false; };
        let world = fox.get_entity().world.load();
        let entities = world.entities.load();

        for candidate in entities.iter() {
            let Some(player) = candidate.get_player() else { continue; };
            if !fox.trusts(&player.gameprofile.id) { continue; }

            let attacker_id = player.living_entity.last_attacker_id.load(Ordering::Relaxed);
            if attacker_id == 0 { continue; }
            if let Some(attacker_ent) = world.get_entity_by_id(attacker_id) {
                if attacker_ent.get_entity().is_alive() && !fox.trusts(&attacker_ent.get_entity().entity_uuid) {
                    self.target_attacker = Some(attacker_ent);
                    return true;
                }
            }
        }
        false
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.target_attacker.as_ref().is_some_and(|t| t.get_entity().is_alive())
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = &self.target_attacker {
            mob.get_mob_entity().set_target(Some(target.clone()));
            if let Some(fox) = self.fox.upgrade() {
                fox.set_defending(true);
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
