use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI8, AtomicI32, Ordering::Relaxed},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::block_state::SoundType;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::data_component_impl::EquippableImpl;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag;
use pumpkin_data::tracked_data;
use pumpkin_data::{Block, BlockState};
use pumpkin_inventory::Inventory;
use pumpkin_inventory::inventory::{SimpleInventory, sync_read_items_from_nbt};
use pumpkin_inventory::mount_screen_handler::MountScreenHandler;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::bedrock::client::set_actor_data::{
    MetadataValue, SyncedActorDataList, entity_data_flag, entity_data_key,
};
use pumpkin_protocol::bedrock::client::update_equip::HORSE_CONTAINER_TYPE;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::ageable::AgeableMob;
use crate::entity::ai::goal::{
    breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
    look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
    random_stand::RandomStandGoal, run_around_like_crazy::RunAroundLikeCrazyGoal, swim::SwimGoal,
    tempt::TemptGoal, wander_around::WanderAroundGoal,
};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::prevents_equipment_drop;
use crate::entity::mob::Mob;
use crate::entity::mob::equipment::EquipmentSlotContainer;
use crate::entity::mob::equipment::equip_on_target;
use crate::entity::mob::spawn::{EntitySpawnReason, SpawnGroupData};
use crate::entity::passive::animal::Animal;
use crate::entity::player::Player;
use crate::entity::player::advancement::trigger::AdvancementTrigger;
use crate::entity::r#type::from_type;
use crate::entity::{Entity, EntityBase};
use crate::world::World;

pub const FLAG_TAME: i8 = 2;
pub const FLAG_BRED: i8 = 8;
pub const FLAG_EATING: i8 = 16;
pub const FLAG_STANDING: i8 = 32;
pub const FLAG_OPEN_MOUTH: i8 = 64;
pub const SOUND_VOLUME: f32 = 0.8;
pub const AMBIENT_SOUND_INTERVAL: i32 = 400;
pub const CHESTED_INVENTORY_COLUMNS: i32 = 5;
pub const BEDROCK_HORSE_INVENTORY_SIZE: i32 = 2;
pub const BEDROCK_SADDLE_INVENTORY_SIZE: i32 = 1;
pub const BEDROCK_CHESTED_INVENTORY_SIZE: i32 = 16;
const STAND_TICKS: i32 = 20;
const BABY_AGE_SCALE: f64 = 0.5;
const STEP_VOLUME_SCALE: f32 = 0.15;
pub const JUMP_SOUND_VOLUME: f32 = 0.4;
const GALLOP_START_STEPS: i32 = 5;
const GALLOP_STEP_INTERVAL: i32 = 3;
const MAX_JUMP_AMOUNT: i32 = 90;
const HEAL_CHANCE: i32 = 900;
const GRASS_EAT_CHANCE: i32 = 300;
const GRASS_EAT_TICKS: i32 = 50;
const STAND_ON_HURT_CHANCE: i32 = 3;
const MOUTH_OPEN_TICKS: i32 = 30;
const MOMMY_RANGE: f64 = 16.0;
const MOMMY_MIN_DISTANCE_SQ: f64 = 4.0;

pub enum RiddenStep {
    Walk,
    Gallop,
    Quiet,
}

pub struct AbstractHorseData {
    flags: AtomicI8,
    temper: AtomicI32,
    owner: AtomicCell<Option<Uuid>>,
    has_chest: AtomicBool,
    inventory: Mutex<Arc<SimpleInventory>>,
    mouth_counter: AtomicI32,
    stand_counter: AtomicI32,
    eating_counter: AtomicI32,
    gallop_sound_counter: AtomicI32,
    stand_anim: AtomicCell<f32>,
    stand_anim_o: AtomicCell<f32>,
}

impl Default for AbstractHorseData {
    fn default() -> Self {
        Self {
            flags: AtomicI8::default(),
            temper: AtomicI32::default(),
            owner: AtomicCell::default(),
            has_chest: AtomicBool::default(),
            inventory: Mutex::new(Arc::new(SimpleInventory::new(0))),
            mouth_counter: AtomicI32::default(),
            stand_counter: AtomicI32::default(),
            eating_counter: AtomicI32::default(),
            gallop_sound_counter: AtomicI32::default(),
            stand_anim: AtomicCell::default(),
            stand_anim_o: AtomicCell::default(),
        }
    }
}

pub trait AbstractHorse: Animal + AgeableMob {
    fn horse_data(&self) -> &AbstractHorseData;

    fn get_max_temper(&self) -> i32 {
        100
    }

    fn get_angry_sound(&self) -> Option<Sound> {
        let entity_type = self.get_entity().entity_type;
        if self.is_baby() {
            entity_type.baby_angry_sound
        } else {
            entity_type.angry_sound
        }
    }

    fn get_eating_sound(&self) -> Option<Sound> {
        let entity_type = self.get_entity().entity_type;
        if self.is_baby() {
            entity_type.baby_eating_sound
        } else {
            entity_type.eating_sound
        }
    }

    fn ambient_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.ambient_sound
    }

    /// `None` falls back to the entity type's sound.
    fn get_hurt_sound(&self) -> Option<Sound> {
        None
    }

    /// `None` falls back to the entity type's sound.
    fn get_death_sound(&self) -> Option<Sound> {
        None
    }

    fn can_eat_grass(&self) -> bool {
        true
    }

    fn on_hurt(&self)
    where
        Self: Sized,
    {
        stand_on_hurt(self);
    }

    fn on_leash_pull(&self) {
        if self.is_eating() {
            self.set_eating(false);
        }
    }

    fn get_chest_equip_sound(&self) -> Sound {
        Sound::EntityDonkeyChest
    }

    /// Whether a chest can be strapped on, as on donkeys, mules and llamas.
    fn can_carry_chest(&self) -> bool {
        false
    }

    fn can_perform_rearing(&self) -> bool {
        true
    }

    fn can_gallop(&self) -> bool {
        !self.can_carry_chest()
    }

    fn play_gallop_sound(&self, sound_type: &SoundType) {
        play_horse_gallop_sound(self, sound_type);
    }

    fn is_mob_controlled(&self) -> bool {
        false
    }

    fn can_jump(&self) -> bool {
        self.is_saddled()
    }

    fn handle_start_jump(&self, _jump_scale: i32) {
        self.stand_if_possible();
        self.play_jump_sound();
    }

    fn play_jump_sound(&self) {
        self.play_entity_sound(Sound::EntityHorseJump, JUMP_SOUND_VOLUME, 1.0);
    }

    fn randomize_attributes(&self) {}

    /// Re-pins a size override after growing up; the age size itself comes from data.
    fn refresh_age_dimensions(&self) {}

    fn finalize_horse_spawn(
        &self,
        _world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        self.randomize_attributes();
        self.finalize_spawn_super(group_data)
    }

    fn bedrock_variant_data(&self) -> SyncedActorDataList {
        SyncedActorDataList::new()
    }

    fn can_parent(&self) -> bool {
        let entity = self.get_entity();
        let living = &self.get_mob_entity().living_entity;
        !entity.has_passengers()
            && !entity.has_vehicle()
            && self.is_tamed()
            && !self.is_baby()
            && living.health.load() >= living.get_max_health()
            && self.get_mob_entity().is_in_love()
    }

    /// Both parents must be one of `partner_types` and ready to breed.
    fn can_mate_with(&self, partner: &dyn EntityBase, partner_types: &[&EntityType]) -> bool {
        partner.get_entity().entity_id != self.get_entity().entity_id
            && partner_types.contains(&partner.get_entity().entity_type)
            && self.can_parent()
            && partner
                .get_mob()
                .and_then(Mob::as_abstract_horse)
                .is_some_and(AbstractHorse::can_parent)
    }

    /// Spawns a `baby_type` foal whose health, jump and speed come from both parents.
    fn breed_offspring(
        &self,
        partner: &dyn EntityBase,
        baby_type: &'static EntityType,
    ) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = from_type(
            baby_type,
            entity.pos.load(),
            &entity.world.load(),
            Uuid::new_v4(),
        );
        self.set_offspring_attributes(partner, baby.as_ref());
        baby
    }

    fn set_offspring_attributes(&self, partner: &dyn EntityBase, baby: &dyn EntityBase) {
        let (Some(partner), Some(baby)) = (partner.get_living_entity(), baby.get_living_entity())
        else {
            return;
        };
        let living = &self.get_mob_entity().living_entity;
        let mut rng = self.get_random();
        let mut offspring = |attribute: &Attributes, min: f64, max: f64| {
            create_offspring_attribute(
                living.get_attribute_base(attribute),
                partner.get_attribute_base(attribute),
                min,
                max,
                &mut rng,
            )
        };
        let health = offspring(
            &Attributes::MAX_HEALTH,
            f64::from(generate_max_health(|_| 0)),
            f64::from(generate_max_health(|bound| bound - 1)),
        );
        let jump = offspring(
            &Attributes::JUMP_STRENGTH,
            f64::from(generate_jump_strength(|| 0.0) as f32),
            f64::from(generate_jump_strength(|| 1.0) as f32),
        );
        let speed = offspring(
            &Attributes::MOVEMENT_SPEED,
            f64::from(generate_speed(|| 0.0) as f32),
            f64::from(generate_speed(|| 1.0) as f32),
        );
        baby.set_max_health(health as f32);
        baby.set_attribute_base(&Attributes::JUMP_STRENGTH, jump);
        baby.set_attribute_base(&Attributes::MOVEMENT_SPEED, speed);
    }

    fn follow_mommy(&self)
    where
        Self: Sized,
    {
        follow_mommy(self);
    }

    fn horse_ai_step(&self)
    where
        Self: Sized,
    {
        let living = &self.get_mob_entity().living_entity;
        if !living.is_alive() {
            return;
        }
        if self.get_random().random_range(0..HEAL_CHANCE) == 0
            && living.death_time.load(Relaxed) == 0
        {
            living.heal(1.0);
        }
        if self.can_eat_grass() {
            self.tick_grass_eating();
        }
        self.follow_mommy();
    }

    fn subclass_ai_step(&self) {}

    fn tick_grass_eating(&self) {
        let entity = self.get_entity();
        if !self.is_eating()
            && !entity.has_passengers()
            && self.get_random().random_range(0..GRASS_EAT_CHANCE) == 0
            && entity
                .world
                .load()
                .get_block_state(&entity.block_pos.load().down())
                .id
                .to_block()
                == &Block::GRASS_BLOCK
        {
            self.set_eating(true);
        }
        let counter = &self.horse_data().eating_counter;
        if self.is_eating() && counter.fetch_add(1, Relaxed) + 1 > GRASS_EAT_TICKS {
            counter.store(0, Relaxed);
            self.set_eating(false);
        }
    }

    fn make_mad(&self) {
        if self.is_standing() {
            return;
        }
        self.stand_if_possible();
        if let Some(sound) = self.get_angry_sound() {
            self.make_sound(sound);
        }
    }

    fn stand_if_possible(&self) {
        if self.can_perform_rearing() {
            self.set_standing(STAND_TICKS);
        }
    }

    fn set_standing(&self, ticks: i32) {
        self.set_eating(false);
        self.set_flag(FLAG_STANDING, true);
        self.horse_data().stand_counter.store(ticks, Relaxed);
    }

    fn clear_standing(&self) {
        self.set_flag(FLAG_STANDING, false);
        self.horse_data().stand_counter.store(0, Relaxed);
    }

    fn next_ridden_step(&self) -> RiddenStep {
        let steps = self.horse_data().gallop_sound_counter.fetch_add(1, Relaxed) + 1;
        if steps <= GALLOP_START_STEPS {
            RiddenStep::Walk
        } else if steps % GALLOP_STEP_INTERVAL == 0 {
            RiddenStep::Gallop
        } else {
            RiddenStep::Quiet
        }
    }

    fn open_mouth(&self) {
        self.horse_data().mouth_counter.store(1, Relaxed);
        self.set_flag(FLAG_OPEN_MOUTH, true);
    }

    /// Closes the mouth and ends rearing once their counters run out.
    fn tick_horse_counters(&self) {
        let data = self.horse_data();
        let mouth = data.mouth_counter.load(Relaxed);
        if mouth > 0 {
            if mouth + 1 > MOUTH_OPEN_TICKS {
                data.mouth_counter.store(0, Relaxed);
                self.set_flag(FLAG_OPEN_MOUTH, false);
            } else {
                data.mouth_counter.store(mouth + 1, Relaxed);
            }
        }
        let stand = data.stand_counter.load(Relaxed);
        if stand > 0 {
            data.stand_counter.store(stand - 1, Relaxed);
            if stand - 1 <= 0 {
                self.clear_standing();
            }
        }
        let stand_anim = data.stand_anim.load();
        data.stand_anim_o.store(stand_anim);
        data.stand_anim.store(if self.is_standing() {
            (stand_anim + (1.0 - stand_anim) * 0.4 + 0.05).min(1.0)
        } else {
            (stand_anim + (0.8 * stand_anim * stand_anim * stand_anim - stand_anim) * 0.6 - 0.05)
                .max(0.0)
        });
    }

    fn eating(&self) {
        self.open_mouth();
        if let Some(sound) = self.get_eating_sound() {
            self.play_varied_sound(sound);
        }
    }

    /// Plays at full volume with a pitch of 1 +/- 0.2.
    fn play_varied_sound(&self, sound: Sound) {
        if self.get_entity().is_silent() {
            return;
        }
        let mut rng = self.get_random();
        let pitch = 1.0 + (rng.random::<f32>() - rng.random::<f32>()) * 0.2;
        self.play_entity_sound(sound, 1.0, pitch);
    }

    fn handle_eating(&self, player: &Arc<Player>, stack: &ItemStack) -> bool {
        let item = stack.item;
        let (heal, age_up, temper, breeds) = if item == &Item::WHEAT {
            (2.0, 20, 3, false)
        } else if item == &Item::SUGAR {
            (1.0, 30, 3, false)
        } else if item == &Item::HAY_BLOCK {
            (20.0, 180, 0, false)
        } else if item == &Item::APPLE || item == &Item::CARROT {
            (3.0, 60, 3, false)
        } else if item == &Item::RED_MUSHROOM {
            (3.0, 0, 3, false)
        } else if item == &Item::GOLDEN_CARROT {
            (4.0, 60, 5, true)
        } else if item == &Item::GOLDEN_APPLE || item == &Item::ENCHANTED_GOLDEN_APPLE {
            (10.0, 240, 10, true)
        } else {
            (0.0, 0, 0, false)
        };
        let mut item_used = false;
        if breeds && self.is_tamed() && self.get_age() == 0 && !self.get_mob_entity().is_in_love() {
            item_used = true;
            self.set_in_love(player);
        }
        let living = &self.get_mob_entity().living_entity;
        if living.health.load() < living.get_max_health() && heal > 0.0 {
            living.heal(heal);
            item_used = true;
        }
        if self.is_baby() && age_up > 0 && !self.is_age_locked() {
            spawn_age_up_particle(self.get_entity(), &mut self.get_random());
            self.age_up(age_up, false);
            item_used = true;
        }
        if temper > 0
            && (item_used || !self.is_tamed())
            && self.get_temper() < self.get_max_temper()
        {
            self.modify_temper(temper);
            item_used = true;
        }
        if item_used {
            self.eating();
            let entity = self.get_entity();
            entity
                .world
                .load()
                .emit_game_event(GameEvent::Eat.name(), entity.pos.load());
        }
        item_used
    }

    fn fed_food(&self, player: &Arc<Player>, stack: &mut ItemStack) -> bool {
        let ate = self.handle_eating(player, stack);
        if ate {
            stack.decrement_unless_creative(player.gamemode.load(), 1);
        }
        ate
    }

    fn equip_chest(&self, player: &Arc<Player>, stack: &mut ItemStack) {
        self.set_chest(true);
        self.play_varied_sound(self.get_chest_equip_sound());
        stack.decrement_unless_creative(player.gamemode.load(), 1);
        self.create_inventory();
    }

    fn equip_body_armor(&self, player: &Arc<Player>, stack: &mut ItemStack) {
        let mut equipped = stack.clone();
        equipped.item_count = 1;
        stack.decrement_unless_creative(player.gamemode.load(), 1);
        let equip_sound = equipped
            .get_data_component::<EquippableImpl>()
            .map(|equippable| equippable.equip_sound.clone());
        self.get_mob_entity()
            .set_item_slot_and_drop_when_killed(&EquipmentSlot::BODY, equipped);
        let entity = self.get_entity();
        let world = entity.world.load();
        if let Some(sound) = equip_sound
            && !entity.is_silent()
        {
            world.play_sound_event(&sound, self.sound_source(), &entity.pos.load());
        }
        world.emit_game_event(GameEvent::Equip.name(), entity.pos.load());
    }

    fn do_player_ride(&self, player: &Arc<Player>) {
        start_riding(self, player);
    }

    /// Leads, name tags and spawn eggs, which come before any horse-specific handling.
    fn handle_important_interactions(
        &self,
        player: &Arc<Player>,
        stack: &mut ItemStack,
    ) -> Option<bool> {
        if self
            .get_mob_entity()
            .mob_interact(player, stack, self.can_be_leashed())
        {
            return Some(true);
        }
        (stack.item == &Item::NAME_TAG || stack.item.registry_key.ends_with("_spawn_egg"))
            .then_some(false)
    }

    /// The feeding, anger and chest checks horses, zombie horses and chested horses run before
    /// the shared interaction.
    fn interact_with_held_item(
        &self,
        player: &Arc<Player>,
        stack: &mut ItemStack,
        blocks_golden_dandelion: bool,
    ) -> Option<bool> {
        let opens_inventory =
            !self.is_baby() && self.is_tamed() && player.get_entity().is_sneaking();
        if self.get_entity().has_passengers()
            || opens_inventory
            || stack.is_empty()
            || (blocks_golden_dandelion
                && self.is_baby()
                && is_holding(player, &Item::GOLDEN_DANDELION))
        {
            return None;
        }
        if self.is_food(stack) {
            return Some(self.fed_food(player, stack));
        }
        if !self.is_tamed() {
            self.make_mad();
            return Some(true);
        }
        if self.can_carry_chest() && !self.has_chest() && stack.item == &Item::CHEST {
            self.equip_chest(player, stack);
            return Some(true);
        }
        None
    }

    fn horse_mob_interact(&self, player: &Arc<Player>, stack: &mut ItemStack) -> bool
    where
        Self: Sized,
    {
        if self.get_entity().has_passengers() || self.is_baby() {
            return self.animal_interact(player, stack);
        }
        if self.is_tamed() && player.get_entity().is_sneaking() {
            self.open_custom_inventory_screen(player);
            return true;
        }
        if !stack.is_empty() {
            if equip_on_target(self, player, stack) {
                return true;
            }
            if self.is_equippable_in_slot(stack, &EquipmentSlot::BODY)
                && self.get_mob_entity().is_slot_empty(&EquipmentSlot::BODY)
            {
                self.equip_body_armor(player, stack);
                return true;
            }
        }
        self.do_player_ride(player);
        true
    }

    fn tame_with_name(&self, player: &Arc<Player>) {
        self.set_owner(Some(player.gameprofile.id));
        self.set_tamed(true);
        let entity = self.get_entity();
        player.trigger_advancement(AdvancementTrigger::TameAnimal {
            entity_type: format!("minecraft:{}", entity.entity_type.resource_name),
        });
        entity.world.load().send_entity_status(
            entity,
            EntityStatus::TamingSucceeded,
            ActorEventID::TamingSucceeded,
        );
    }

    fn get_flag(&self, flag: i8) -> bool {
        self.horse_data().flags.load(Relaxed) & flag != 0
    }

    fn set_flag(&self, flag: i8, value: bool) {
        let flags = &self.horse_data().flags;
        let new_flags = if value {
            flags.fetch_or(flag, Relaxed) | flag
        } else {
            flags.fetch_and(!flag, Relaxed) & !flag
        };
        self.get_entity()
            .set_synced_data(tracked_data::abstract_horse::DATA_ID_FLAGS, new_flags);
    }

    fn set_tamed(&self, tamed: bool) {
        self.set_flag(FLAG_TAME, tamed);
        let entity = self.get_entity();
        entity.set_bedrock_flag(entity_data_flag::TAMED, tamed);
        entity.send_bedrock_actor_data(&self.bedrock_container_data());
    }

    fn is_bred(&self) -> bool {
        self.get_flag(FLAG_BRED)
    }

    fn set_bred(&self, bred: bool) {
        self.set_flag(FLAG_BRED, bred);
    }

    fn is_eating(&self) -> bool {
        self.get_flag(FLAG_EATING)
    }

    fn set_eating(&self, eating: bool) {
        self.set_flag(FLAG_EATING, eating);
    }

    fn is_standing(&self) -> bool {
        self.get_flag(FLAG_STANDING)
    }

    fn get_temper(&self) -> i32 {
        self.horse_data().temper.load(Relaxed)
    }

    fn set_temper(&self, temper: i32) {
        self.horse_data().temper.store(temper, Relaxed);
    }

    fn modify_temper(&self, amount: i32) -> i32 {
        let temper = (self.get_temper() + amount).clamp(0, self.get_max_temper());
        self.set_temper(temper);
        temper
    }

    fn get_owner(&self) -> Option<Uuid> {
        self.horse_data().owner.load()
    }

    fn set_owner(&self, owner: Option<Uuid>) {
        self.horse_data().owner.store(owner);
    }

    fn has_chest(&self) -> bool {
        self.horse_data().has_chest.load(Relaxed)
    }

    fn set_chest(&self, has_chest: bool) {
        self.horse_data().has_chest.store(has_chest, Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            tracked_data::abstract_chested_horse::DATA_ID_CHEST,
            has_chest,
        );
        entity.set_bedrock_flag(entity_data_flag::CHESTED, has_chest);
        entity.send_bedrock_actor_data(&self.bedrock_container_data());
    }

    fn get_inventory_columns(&self) -> i32 {
        0
    }

    /// The Java mount screen slot Bedrock shows as its one equipment slot.
    fn bedrock_equip_slot(&self) -> usize {
        MountScreenHandler::SLOT_SADDLE
    }

    /// The `UpdateEquip` slot list Bedrock uses to draw the equipment slot.
    fn bedrock_equip_slots(&self) -> NbtCompound {
        bedrock_equip_slot_nbt("minecraft:saddle", 0)
    }

    fn bedrock_inventory_size(&self) -> i32 {
        BEDROCK_HORSE_INVENTORY_SIZE
    }

    /// The actor data Bedrock reads to size and open the mount inventory.
    fn bedrock_container_data(&self) -> SyncedActorDataList {
        let mut data = SyncedActorDataList::new();
        data.set(
            entity_data_key::CONTAINER_TYPE,
            MetadataValue::Byte(self.bedrock_container_type()),
        );
        data.set(
            entity_data_key::CONTAINER_SIZE,
            MetadataValue::Int(self.bedrock_inventory_size()),
        );
        data
    }

    fn bedrock_container_type(&self) -> i8 {
        if self.is_tamed() {
            HORSE_CONTAINER_TYPE as i8
        } else {
            0
        }
    }

    fn inventory(&self) -> Arc<SimpleInventory> {
        self.horse_data()
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Resizes the inventory for the current columns, keeping what fits.
    fn create_inventory(&self) {
        let size = MountScreenHandler::get_inventory_size(
            usize::try_from(self.get_inventory_columns()).unwrap_or(0),
        );
        let inventory = SimpleInventory::new(size);
        let mut current = self
            .horse_data()
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for slot in 0..size.min(current.size()) {
            inventory.set_stack(slot, current.get_stack(slot));
        }
        *current = Arc::new(inventory);
    }

    fn open_custom_inventory_screen(&self, player: &Arc<Player>) {
        let entity = self.get_entity();
        if (entity.has_passengers() && !entity.has_passenger(player.entity_id()))
            || !self.is_tamed()
        {
            return;
        }
        let Some(mount) = entity.world.load().get_entity_by_id(entity.entity_id) else {
            return;
        };
        let mount = Arc::downgrade(&mount);
        let inventory = self.inventory();
        let columns = self.get_inventory_columns();
        let bedrock_equip_slot = self.bedrock_equip_slot();
        player.open_mount_screen(
            columns,
            entity.entity_id,
            bedrock_equip_slot,
            self.bedrock_equip_slots(),
            |sync_id, player_inventory| {
                let mut handler = MountScreenHandler::new(
                    sync_id,
                    player_inventory,
                    inventory.clone(),
                    Arc::new(EquipmentSlotContainer::new(
                        mount.clone(),
                        EquipmentSlot::SADDLE,
                    )),
                    Arc::new(EquipmentSlotContainer::new(
                        mount.clone(),
                        EquipmentSlot::BODY,
                    )),
                    usize::try_from(columns).unwrap_or(0),
                );
                handler.bedrock_equip_slot = Some(bedrock_equip_slot);
                handler.validity_check = Some(Box::new(move |player| {
                    let Some(player) = player.as_any().downcast_ref::<Player>() else {
                        return false;
                    };
                    mount.upgrade().is_some_and(|mount| {
                        mount.get_entity().is_alive()
                            && mount
                                .get_mob()
                                .and_then(|mob| mob.as_abstract_horse())
                                .is_some_and(|horse| Arc::ptr_eq(&horse.inventory(), &inventory))
                            && player.can_interact_with_entity(mount.as_ref(), 4.0)
                    })
                }));
                Arc::new(Mutex::new(handler))
            },
        );
    }

    fn drop_horse_inventory(&self) {
        let entity = self.get_entity();
        let inventory = self.inventory();
        for slot in 0..inventory.size() {
            let stack = inventory.get_stack(slot);
            if !stack.is_empty() && !prevents_equipment_drop(&stack) {
                entity.spawn_at_location(stack);
            }
        }
        if self.has_chest() {
            entity.spawn_at_location(ItemStack::new(1, &Item::CHEST));
            self.set_chest(false);
        }
    }

    fn init_horse_data_tracker(&self) {
        let entity = self.get_entity();
        if self.is_baby() {
            entity.set_synced_data(tracked_data::abstract_horse::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            tracked_data::abstract_horse::DATA_ID_FLAGS,
            self.horse_data().flags.load(Relaxed),
        );
    }

    fn init_chested_horse_data_tracker(&self) {
        self.get_entity().set_synced_data(
            tracked_data::abstract_chested_horse::DATA_ID_CHEST,
            self.has_chest(),
        );
    }

    fn write_horse_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("EatingHaystack", self.is_eating());
        nbt.put_bool("Bred", self.is_bred());
        nbt.put_int("Temper", self.get_temper());
        nbt.put_bool("Tame", self.is_tamed());
        if let Some(owner) = self.get_owner() {
            nbt.put_uuid("Owner", owner);
        }
    }

    fn read_horse_nbt(&self, nbt: &NbtCompound) {
        self.set_eating(nbt.get_bool("EatingHaystack").unwrap_or(false));
        self.set_bred(nbt.get_bool("Bred").unwrap_or(false));
        self.set_temper(nbt.get_int("Temper").unwrap_or(0));
        self.set_tamed(nbt.get_bool("Tame").unwrap_or(false));
        self.set_owner(nbt.get_uuid("Owner"));
    }

    fn write_chested_horse_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("ChestedHorse", self.has_chest());
        if self.has_chest() {
            self.inventory().write_inventory_nbt(nbt, true);
        }
    }

    fn read_chested_horse_nbt(&self, nbt: &NbtCompound) {
        self.set_chest(nbt.get_bool("ChestedHorse").unwrap_or(false));
        self.create_inventory();
        if self.has_chest() {
            let inventory = self.inventory();
            sync_read_items_from_nbt(
                nbt,
                &mut inventory
                    .stacks
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
        }
    }
}

pub fn create_offspring_attribute(
    parent_a: f64,
    parent_b: f64,
    min: f64,
    max: f64,
    rng: &mut impl rand::Rng,
) -> f64 {
    let parent_a = parent_a.clamp(min, max);
    let parent_b = parent_b.clamp(min, max);
    let margin = 0.15 * (max - min);
    let range = (parent_a - parent_b).abs() + margin * 2.0;
    let average = f64::midpoint(parent_a, parent_b);
    let baby_quality =
        (rng.random::<f64>() + rng.random::<f64>() + rng.random::<f64>()) / 3.0 - 0.5;
    let value = average + range * baby_quality;
    if value > max {
        max - (value - max)
    } else if value < min {
        min + (min - value)
    } else {
        value
    }
}

pub fn generate_max_health(mut next_int: impl FnMut(i32) -> i32) -> f32 {
    15.0 + next_int(8) as f32 + next_int(9) as f32
}

pub fn generate_jump_strength(mut probability: impl FnMut() -> f64) -> f64 {
    f64::from(0.4f32) + probability() * 0.2 + probability() * 0.2 + probability() * 0.2
}

pub fn generate_speed(mut probability: impl FnMut() -> f64) -> f64 {
    (f64::from(0.45f32) + probability() * 0.3 + probability() * 0.3 + probability() * 0.3) * 0.25
}

/// Fall damage for horse-like mobs: a landing sound and no hurt sound.
pub fn cause_fall_damage(
    mob: &dyn Mob,
    caller: &dyn EntityBase,
    fall_distance: f32,
    damage_modifier: f32,
) -> bool {
    if fall_distance > 1.0 {
        let sound = if mob.as_ageable().is_some_and(AgeableMob::is_baby) {
            Sound::EntityBabyHorseLand
        } else {
            Sound::EntityHorseLand
        };
        caller.play_entity_sound(sound, 0.4, 1.0);
    }
    let damage = mob.calculate_fall_damage(fall_distance, damage_modifier);
    if damage <= 0 {
        return false;
    }
    let living = &mob.get_mob_entity().living_entity;
    living.damage(caller, damage as f32, DamageType::FALL);
    living.propagate_fall_to_passengers(fall_distance, damage_modifier);
    living.play_block_fall_sound(caller);
    true
}

/// One Bedrock `UpdateEquip` slot that takes any data value of `item`.
#[must_use]
pub fn bedrock_equip_slot_nbt(item: &str, slot_number: i32) -> NbtCompound {
    let slot_item = || {
        let mut compound = NbtCompound::new();
        compound.put_short("Aux", i16::MAX);
        compound.put_string("Name", item.to_string());
        compound
    };
    let mut accepted = NbtCompound::new();
    accepted.put_compound("slotItem", slot_item());
    let mut slot = NbtCompound::new();
    slot.put_list("acceptedItems", vec![NbtTag::Compound(accepted)]);
    slot.put_compound("item", slot_item());
    slot.put_int("slotNumber", slot_number);
    let mut data = NbtCompound::new();
    data.put_list("slots", vec![NbtTag::Compound(slot)]);
    data
}

/// The goals every horse registers before its behaviour goals.
pub fn register_horse_goals(horse: &Arc<dyn Mob>) {
    let goals = &horse.get_mob_entity().goals_selector;
    goals.add_goal(1, Box::new(RunAroundLikeCrazyGoal::new(1.2)));
    goals.add_goal(
        2,
        BreedGoal::with_partners(1.0, TargetMatch::ABSTRACT_HORSE),
    );
    goals.add_goal(4, Box::new(FollowParentGoal::new(1.0)));
    goals.add_goal(6, Box::new(WanderAroundGoal::water_avoiding(0.7)));
    goals.add_goal(
        7,
        LookAtEntityGoal::with_default(Arc::downgrade(horse), &EntityType::PLAYER, 6.0),
    );
    goals.add_goal(8, Box::new(RandomLookAroundGoal::default()));
    if horse
        .as_abstract_horse()
        .is_some_and(AbstractHorse::can_perform_rearing)
    {
        goals.add_goal(
            9,
            Box::new(RandomStandGoal::new(horse.get_ambient_sound_interval())),
        );
    }
}

/// The behaviour goals most horses add: floating, panicking and following tempting food.
pub fn add_horse_behaviour_goals(horse: &dyn Mob) {
    let goals = &horse.get_mob_entity().goals_selector;
    goals.add_goal(0, Box::new(SwimGoal::default()));
    goals.add_goal(1, EscapeDangerGoal::mount(1.2));
    goals.add_goal(
        3,
        Box::new(TemptGoal::with_tag(
            1.25,
            &tag::Item::MINECRAFT_HORSE_TEMPT_ITEMS,
            false,
        )),
    );
}

pub fn start_riding<H: AbstractHorse + ?Sized>(horse: &H, player: &Arc<Player>) {
    horse.set_eating(false);
    horse.clear_standing();
    let entity = horse.get_entity();
    player
        .get_entity()
        .set_rotation(entity.yaw.load(), entity.pitch.load());
    if let Some(vehicle) = entity.world.load().get_entity_by_id(entity.entity_id) {
        entity.add_passenger(vehicle, player.clone());
    }
}

fn is_holding(player: &Player, item: &Item) -> bool {
    let inventory = player.inventory();
    inventory.held_item().item == item || inventory.off_hand_item().item == item
}

pub fn spawn_age_up_particle(entity: &Entity, rng: &mut impl rand::Rng) {
    let width = f64::from(entity.width());
    let height = f64::from(entity.height());
    let pos = entity.pos.load()
        + Vector3::new(
            (rng.random::<f64>() * 2.0 - 1.0) * width,
            rng.random::<f64>() * height + 0.5,
            (rng.random::<f64>() * 2.0 - 1.0) * width,
        );
    entity.world.load().spawn_particle(
        pos,
        Vector3::new(0.0, 0.0, 0.0),
        0.0,
        1,
        Particle::HappyVillager,
    );
}

/// Step sound for every horse that doesn't pick its own.
pub fn play_horse_step_sound(horse: &dyn AbstractHorse, pos: &BlockPos, state: &BlockState) {
    if state.is_liquid() {
        return;
    }
    let entity = horse.get_entity();
    let above = entity.world.load().get_block_state(&pos.up());
    let sound_type = if above.id.to_block() == &Block::SNOW {
        above.get_sound_type()
    } else {
        state.get_sound_type()
    };
    let volume = sound_type.volume * STEP_VOLUME_SCALE;
    if entity.has_passengers() && horse.can_gallop() {
        // Only the rider's client resets this counter, so the server keeps galloping once
        // the first steps are past.
        match horse.next_ridden_step() {
            RiddenStep::Walk => {
                horse.play_entity_sound(Sound::EntityHorseStepWood, volume, sound_type.pitch);
            }
            RiddenStep::Gallop => horse.play_gallop_sound(sound_type),
            RiddenStep::Quiet => {}
        }
        return;
    }
    let sound = if is_wood_sound_type(sound_type) {
        Sound::EntityHorseStepWood
    } else if entity.age.load(Relaxed) < 0 {
        Sound::EntityBabyHorseStep
    } else {
        Sound::EntityHorseStep
    };
    horse.play_entity_sound(sound, volume, sound_type.pitch);
}

pub fn play_horse_gallop_sound<H: EntityBase + ?Sized>(horse: &H, sound_type: &SoundType) {
    horse.play_entity_sound(
        Sound::EntityHorseGallop,
        sound_type.volume * STEP_VOLUME_SCALE,
        sound_type.pitch,
    );
}

#[must_use]
pub fn player_jump_pending_scale(jump_amount: i32) -> f32 {
    if jump_amount >= MAX_JUMP_AMOUNT {
        1.0
    } else {
        0.4 + 0.4 * jump_amount as f32 / MAX_JUMP_AMOUNT as f32
    }
}

fn is_wood_sound_type(sound_type: &SoundType) -> bool {
    matches!(
        sound_type.name,
        "WOOD" | "NETHER_WOOD" | "STEM" | "CHERRY_WOOD" | "BAMBOO_WOOD"
    )
}

/// The path is dropped on purpose; only the navigator target is kept, so the foal heads for
/// its parent on the next path recompute.
pub fn follow_mommy<H: AbstractHorse>(horse: &H) {
    if !horse.is_bred() || !horse.is_baby() || horse.is_eating() {
        return;
    }
    let entity = horse.get_entity();
    let world = entity.world.load();
    let pos = entity.pos.load();
    let mommy_targeting = TargetPredicate::create_non_attackable()
        .set_base_max_distance(MOMMY_RANGE)
        .ignore_visibility();
    let area = entity
        .bounding_box
        .load()
        .expand(MOMMY_RANGE, MOMMY_RANGE, MOMMY_RANGE);
    let mommy = world
        .get_entities_at_box(&area)
        .into_iter()
        .filter(|candidate| {
            candidate
                .get_mob()
                .and_then(Mob::as_abstract_horse)
                .is_some_and(AbstractHorse::is_bred)
                && mommy_targeting.test(&world, Some(horse), candidate.as_ref())
        })
        .min_by(|a, b| {
            let a = a.get_entity().pos.load().squared_distance_to_vec(&pos);
            let b = b.get_entity().pos.load().squared_distance_to_vec(&pos);
            a.total_cmp(&b)
        });
    if let Some(mommy) = mommy {
        let mommy_pos = mommy.get_entity().pos.load();
        if pos.squared_distance_to_vec(&mommy_pos) > MOMMY_MIN_DISTANCE_SQ {
            let mob_entity = horse.get_mob_entity();
            mob_entity
                .steered()
                .navigator()
                .create_path(&mob_entity.living_entity, mommy_pos, 0);
        }
    }
}

/// Only a living, tamed adult takes a saddle.
pub fn can_use_slot<H: AbstractHorse>(horse: &H, slot: &EquipmentSlot) -> bool {
    *slot != EquipmentSlot::SADDLE
        || (horse.get_entity().is_alive() && !horse.is_baby() && horse.is_tamed())
}

pub fn is_immobile<H: AbstractHorse>(horse: &H) -> bool {
    let entity = horse.get_entity();
    (horse.get_mob_entity().living_entity.is_dead_or_dying()
        && entity.has_passengers()
        && horse.is_saddled())
        || horse.is_eating()
        || horse.is_standing()
}

pub fn stand_on_hurt<H: AbstractHorse>(horse: &H) {
    if horse.get_random().random_range(0..STAND_ON_HURT_CHANCE) == 0 {
        horse.stand_if_possible();
    }
}

pub fn can_dispenser_equip_into_slot(mob: &dyn Mob, slot: &EquipmentSlot) -> bool {
    ((*slot == EquipmentSlot::BODY || *slot == EquipmentSlot::SADDLE) && mob.is_tamed())
        || mob.get_mob_entity().can_pick_up_loot()
}

/// Generates the `AgeableMob` and `AbstractHorse` impls.
///
/// The type needs `ageable_data` and `horse_data` fields. Extra `AbstractHorse` methods go in
/// the first braces, extra `AgeableMob` methods in the optional second ones.
#[macro_export]
macro_rules! impl_abstract_horse {
    ($horse:ty) => {
        $crate::impl_abstract_horse!($horse, {});
    };
    ($horse:ty, { $($extra:tt)* }) => {
        $crate::impl_abstract_horse!($horse, { $($extra)* }, {});
    };
    ($horse:ty, { $($extra:tt)* }, { $($ageable:tt)* }) => {
        impl $crate::entity::ageable::AgeableMob for $horse {
            fn get_ageable_data(&self) -> &$crate::entity::ageable::AgeableData {
                &self.ageable_data
            }

            fn default_baby_spawn_chance(&self) -> f32 {
                $crate::entity::ageable::ABSTRACT_HORSE_BABY_SPAWN_CHANCE
            }

            fn age_boundary_reached(&self) {
                $crate::entity::passive::abstract_horse::AbstractHorse::refresh_age_dimensions(self);
            }

            $($ageable)*
        }

        impl $crate::entity::passive::abstract_horse::AbstractHorse for $horse {
            fn horse_data(&self) -> &$crate::entity::passive::abstract_horse::AbstractHorseData {
                &self.horse_data
            }

            $($extra)*
        }

        impl $crate::entity::custom_sound::CustomSound for $horse {
            fn death_sound(&self) -> Option<pumpkin_data::sound::Sound> {
                $crate::entity::passive::abstract_horse::AbstractHorse::get_death_sound(self)
            }

            fn hurt_sound(&self) -> Option<pumpkin_data::sound::Sound> {
                $crate::entity::passive::abstract_horse::AbstractHorse::get_hurt_sound(self)
            }

            fn sound_volume(&self) -> f32 {
                $crate::entity::passive::abstract_horse::SOUND_VOLUME
            }

            fn sound_category(&self) -> pumpkin_data::sound::SoundCategory {
                $crate::entity::EntityBase::sound_source(self)
            }
        }
    };
}

/// Generates the `Mob` methods every horse shares. Use inside `impl Mob for ...`; camels pass
/// `always_tamed`.
pub fn stand_passenger_attachment_point<H: AbstractHorse>(
    horse: &H,
    index: usize,
) -> Option<Vector3<f64>> {
    let seat = crate::entity::default_passenger_attachment_point(horse, index)?;
    let stand = f64::from(horse.horse_data().stand_anim_o.load());
    let age_scale = if horse.is_baby() { BABY_AGE_SCALE } else { 1.0 };
    let scale = horse
        .get_mob_entity()
        .living_entity
        .get_attribute_value(&Attributes::SCALE)
        * age_scale;
    Some(
        seat + crate::entity::attachment_point(
            [0.0, 0.15 * stand, -0.7 * stand],
            scale,
            horse.get_entity().yaw.load(),
        ),
    )
}

#[macro_export]
macro_rules! abstract_horse_mob_hooks {
    () => {
        fn is_tamed(&self) -> bool {
            $crate::entity::passive::abstract_horse::AbstractHorse::get_flag(
                self,
                $crate::entity::passive::abstract_horse::FLAG_TAME,
            )
        }

        fn mob_passenger_attachment_point(
            &self,
            _passenger: &dyn $crate::entity::EntityBase,
            index: usize,
        ) -> Option<pumpkin_util::math::vector3::Vector3<f64>> {
            $crate::entity::passive::abstract_horse::stand_passenger_attachment_point(self, index)
        }

        $crate::abstract_horse_mob_hooks!(@shared);
    };
    (always_tamed) => {
        fn is_tamed(&self) -> bool {
            true
        }

        $crate::abstract_horse_mob_hooks!(@shared);
    };
    (@shared) => {
        fn get_mob_entity(&self) -> &$crate::entity::mob::MobEntity {
            &self.mob_entity
        }

        fn on_climbable(&self) -> bool {
            false
        }

        fn as_ageable(&self) -> Option<&dyn $crate::entity::ageable::AgeableMob> {
            Some(self)
        }

        fn as_animal(&self) -> Option<&dyn $crate::entity::passive::animal::Animal> {
            Some(self)
        }

        fn as_abstract_horse(
            &self,
        ) -> Option<&dyn $crate::entity::passive::abstract_horse::AbstractHorse> {
            Some(self)
        }

        fn get_owner_uuid(&self) -> Option<uuid::Uuid> {
            $crate::entity::passive::abstract_horse::AbstractHorse::get_owner(self)
        }

        fn mob_support_quad_leash(&self) -> bool {
            true
        }

        fn mob_is_pushable(&self) -> bool {
            !$crate::entity::EntityBase::get_entity(self).has_passengers()
        }

        fn mob_controlling_passenger(
            &self,
        ) -> Option<std::sync::Arc<dyn $crate::entity::EntityBase>> {
            self.saddled_controlling_passenger(None)
        }

        fn get_equip_sound(
            &self,
            slot: &pumpkin_data::data_component_impl::EquipmentSlot,
        ) -> Option<pumpkin_data::sound::Sound> {
            (*slot == pumpkin_data::data_component_impl::EquipmentSlot::SADDLE)
                .then(|| $crate::entity::EntityBase::get_entity(self).entity_type.saddle_sound)
                .flatten()
        }

        fn can_dispenser_equip_into_slot(
            &self,
            slot: &pumpkin_data::data_component_impl::EquipmentSlot,
        ) -> bool {
            $crate::entity::passive::abstract_horse::can_dispenser_equip_into_slot(self, slot)
        }

        fn cause_fall_damage(
            &self,
            caller: &dyn $crate::entity::EntityBase,
            fall_distance: f32,
            damage_modifier: f32,
        ) -> bool {
            $crate::entity::passive::abstract_horse::cause_fall_damage(
                self,
                caller,
                fall_distance,
                damage_modifier,
            )
        }

        fn get_ambient_sound_interval(&self) -> i32 {
            $crate::entity::passive::abstract_horse::AMBIENT_SOUND_INTERVAL
        }

        fn get_ambient_sound(&self) -> Option<pumpkin_data::sound::Sound> {
            $crate::entity::passive::abstract_horse::AbstractHorse::ambient_sound(self)
        }

        fn as_custom_sound(&self) -> Option<&dyn $crate::entity::custom_sound::CustomSound> {
            Some(self)
        }

        fn is_mob_immobile(&self) -> bool {
            $crate::entity::passive::abstract_horse::is_immobile(self)
        }

        fn on_damage(
            &self,
            _damage_type: pumpkin_data::damage::DamageType,
            _source: Option<&dyn $crate::entity::EntityBase>,
        ) {
            $crate::entity::passive::abstract_horse::AbstractHorse::on_hurt(self);
        }

        fn on_elastic_leash_pull(&self) {
            $crate::entity::passive::abstract_horse::AbstractHorse::on_leash_pull(self);
        }

        fn drop_inventory(&self) {
            $crate::entity::passive::abstract_horse::AbstractHorse::drop_horse_inventory(self);
        }

        fn mob_ai_step(&self) {
            $crate::entity::passive::abstract_horse::AbstractHorse::horse_ai_step(self);
            $crate::entity::passive::abstract_horse::AbstractHorse::subclass_ai_step(self);
        }

        fn finalize_spawn(
            &self,
            world: &std::sync::Arc<$crate::world::World>,
            reason: $crate::entity::mob::spawn::EntitySpawnReason,
            group_data: Option<$crate::entity::mob::spawn::SpawnGroupData>,
        ) -> Option<$crate::entity::mob::spawn::SpawnGroupData> {
            $crate::entity::passive::abstract_horse::AbstractHorse::finalize_horse_spawn(
                self, world, reason, group_data,
            )
        }

        fn mob_tick(&self, _caller: &dyn $crate::entity::EntityBase) {
            $crate::entity::passive::abstract_horse::AbstractHorse::tick_horse_counters(self);
        }

        fn mob_bedrock_spawn_metadata(
            &self,
        ) -> Option<pumpkin_protocol::bedrock::client::set_actor_data::SyncedActorDataList> {
            let mut data = self
                .mob_metadata()
                .map(|meta| meta.to_bedrock_actor_data())
                .unwrap_or_default();
            data.0.extend(
                $crate::entity::passive::abstract_horse::AbstractHorse::bedrock_container_data(
                    self,
                )
                .0,
            );
            data.0.extend(
                $crate::entity::passive::abstract_horse::AbstractHorse::bedrock_variant_data(self)
                    .0,
            );
            Some(data)
        }
    };
}
