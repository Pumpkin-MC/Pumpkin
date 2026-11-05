use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering::Relaxed},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{DataComponentImpl, EquipmentSlot, LlamaVariantImpl};
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_variant::LlamaVariant;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{BlockState, tracked_data};
use pumpkin_inventory::mount_screen_handler::MountScreenHandler;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::client::set_actor_data::{
    MetadataValue, SyncedActorDataList, entity_data_key,
};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::goal::llama_follow_caravan::LlamaFollowCaravanGoal;
use crate::entity::ai::goal::run_around_like_crazy::RunAroundLikeCrazyGoal;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::custom_sound::CustomSound;
use crate::entity::mob::spawn::SpawnGroupData;
use crate::entity::passive::abstract_horse::{
    AbstractHorse, AbstractHorseData, BEDROCK_CHESTED_INVENTORY_SIZE, FLAG_TAME,
    bedrock_equip_slot_nbt, follow_mommy, generate_max_health, spawn_age_up_particle, start_riding,
};
use crate::entity::passive::tamable::is_untamed;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob, DEFAULT_BABY_SPAWN_CHANCE},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        ranged_attack::RangedAttackGoal, revenge::RevengeGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity, RangedAttackMob},
    passive::{animal::Animal, trader_llama::TraderLlamaEntity},
    player::Player,
    projectile::llama_spit::LlamaSpitEntity,
};
use crate::world::World;

const MAX_STRENGTH: i32 = 5;
const MAX_TEMPER: i32 = 30;
const FOLLOW_CARAVAN_SPEED: f64 = 2.1;
const FOLLOW_LEASH_SPEED: f64 = 2.0;
const LLAMA_SAFE_FALL_DISTANCE: f32 = 6.0;
const BEDROCK_CONTAINER_STRENGTH_MODIFIER: i32 = 3;

/// Vanilla `Wolf.avoidLlama`: the stronger the llama, the likelier a wolf flees.
pub fn scares_wolves(threat: &dyn EntityBase) -> bool {
    as_llama(threat).is_some_and(|llama| llama.get_strength() >= rand::rng().random_range(0..5))
}

/// Vanilla `LlamaAttackWolfGoal`: untamed wolves within a quarter of the follow range.
pub const LLAMA_HUNTS: &[Hunt] = &[Hunt::new(2, TargetMatch::Type(&EntityType::WOLF), false)
    .every(16)
    .only_mobs(is_untamed)
    .range_scale(0.25)];

#[must_use]
pub fn as_llama(entity: &dyn EntityBase) -> Option<&LlamaEntity> {
    let any = entity.cast_any();
    any.downcast_ref::<LlamaEntity>().or_else(|| {
        any.downcast_ref::<TraderLlamaEntity>()
            .map(|trader| &*trader.llama)
    })
}

pub struct LlamaEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    horse_data: AbstractHorseData,
    strength: AtomicI32,
    variant: AtomicCell<LlamaVariant>,
    did_spit: AtomicBool,
    caravan_head: AtomicCell<Option<i32>>,
    caravan_tail: AtomicCell<Option<i32>>,
}

impl LlamaEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_required_path_length(40.0);
        let llama = Arc::new(Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            horse_data: AbstractHorseData::default(),
            strength: AtomicI32::new(0),
            variant: AtomicCell::new(LlamaVariant::DEFAULT),
            did_spit: AtomicBool::new(false),
            caravan_head: AtomicCell::new(None),
            caravan_tail: AtomicCell::new(None),
        });
        llama.register_goals();
        llama
    }

    fn register_goals(self: &Arc<Self>) {
        let mob_weak: Weak<dyn Mob> = {
            let mob: Arc<dyn Mob> = self.clone();
            Arc::downgrade(&mob)
        };
        let ranged_weak: Weak<dyn RangedAttackMob> = {
            let ranged: Arc<dyn RangedAttackMob> = self.clone();
            Arc::downgrade(&ranged)
        };
        let goal_selector = &self.mob_entity.goals_selector;
        let target_selector = &self.mob_entity.target_selector;

        goal_selector.add_goal(0, Box::new(SwimGoal::default()));
        goal_selector.add_goal(1, Box::new(RunAroundLikeCrazyGoal::new(1.2)));
        goal_selector.add_goal(
            2,
            Box::new(LlamaFollowCaravanGoal::new(FOLLOW_CARAVAN_SPEED)),
        );
        goal_selector.add_goal(
            3,
            Box::new(RangedAttackGoal::new(ranged_weak, 1.25, 40, 20.0)),
        );
        goal_selector.add_goal(3, EscapeDangerGoal::new(1.2));
        let breed = if self.is_trader_llama() {
            BreedGoal::new(1.0)
        } else {
            BreedGoal::with_partners(1.0, TargetMatch::LLAMA_LIKE)
        };
        goal_selector.add_goal(4, breed);
        goal_selector.add_goal(
            5,
            Box::new(TemptGoal::with_tag(
                1.25,
                &tag::Item::MINECRAFT_LLAMA_TEMPT_ITEMS,
                false,
            )),
        );
        goal_selector.add_goal(6, Box::new(FollowParentGoal::new(1.0)));
        goal_selector.add_goal(7, Box::new(WanderAroundGoal::water_avoiding(0.7)));
        goal_selector.add_goal(
            8,
            LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
        );
        goal_selector.add_goal(9, Box::new(RandomLookAroundGoal::default()));

        // A spit drops the revenge target once.
        target_selector.add_goal(
            1,
            Box::new(RevengeGoal::new(true).continuing_while(|mob| {
                as_llama(mob).is_none_or(|llama| !llama.did_spit.swap(false, Relaxed))
            })),
        );
        hunting::apply(target_selector, &self.mob_entity, LLAMA_HUNTS);
    }

    fn is_leashed_to_wandering_trader(&self) -> bool {
        self.get_entity()
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|holder| holder.get_entity().entity_type == &EntityType::WANDERING_TRADER)
    }

    #[must_use]
    pub fn is_trader_llama(&self) -> bool {
        self.get_entity().entity_type == &EntityType::TRADER_LLAMA
    }

    #[must_use]
    pub fn get_strength(&self) -> i32 {
        self.strength.load(Relaxed)
    }

    fn set_strength(&self, strength: i32) {
        let strength = strength.clamp(1, MAX_STRENGTH);
        self.strength.store(strength, Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::llama::DATA_STRENGTH_ID, VarInt(strength));
    }

    fn set_random_strength(&self) {
        let mut rng = self.get_random();
        let max_strength = if rng.random::<f32>() < 0.04 {
            MAX_STRENGTH
        } else {
            3
        };
        self.set_strength(1 + rng.random_range(0..max_strength));
    }

    #[must_use]
    pub fn get_variant(&self) -> LlamaVariant {
        self.variant.load()
    }

    pub fn set_variant(&self, variant: LlamaVariant) {
        self.variant.store(variant);
        let entity = self.get_entity();
        entity.set_synced_data(tracked_data::llama::DATA_VARIANT_ID, VarInt(variant.id()));
        entity.send_bedrock_actor_data(&self.bedrock_variant_data());
    }

    fn bedrock_variant_data(&self) -> SyncedActorDataList {
        let mut data = SyncedActorDataList::new();
        data.set(
            entity_data_key::VARIANT,
            MetadataValue::Int(self.get_variant().id()),
        );
        data
    }

    #[must_use]
    pub fn in_caravan(&self) -> bool {
        self.caravan_head.load().is_some()
    }

    #[must_use]
    pub fn has_caravan_tail(&self) -> bool {
        self.caravan_tail.load().is_some()
    }

    #[must_use]
    pub fn get_caravan_head(&self) -> Option<Arc<dyn EntityBase>> {
        let head = self.caravan_head.load()?;
        self.get_entity().world.load().get_entity_by_id(head)
    }

    pub fn join_caravan(&self, head: &Self) {
        self.caravan_head.store(Some(head.get_entity().entity_id));
        head.caravan_tail.store(Some(self.get_entity().entity_id));
    }

    pub fn leave_caravan(&self) {
        if let Some(head) = self.get_caravan_head()
            && let Some(head) = as_llama(head.as_ref())
        {
            head.caravan_tail.store(None);
        }
        self.caravan_head.store(None);
    }

    fn spit(&self, target: &dyn EntityBase) {
        let entity = self.get_entity();
        let world = entity.world.load_full();
        let spit = LlamaSpitEntity::new_shot(
            Entity::new(world.clone(), entity.pos.load(), &EntityType::LLAMA_SPIT),
            entity,
        );

        let pos = entity.pos.load();
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();
        let dx = target_pos.x - pos.x;
        let dy = target_pos.y + f64::from(target_entity.height()) * 0.333_333_333_333_333_3
            - spit.get_entity().pos.load().y;
        let dz = target_pos.z - pos.z;
        let yo = dx.hypot(dz) * f64::from(0.2f32);
        spit.thrown.set_velocity(dx, dy + yo, dz, 1.5, 10.0);
        world.spawn_entity(Arc::new(spit));

        let mut rng = self.get_random();
        self.play_entity_sound(
            Sound::EntityLlamaSpit,
            1.0,
            1.0 + (rng.random::<f32>() - rng.random::<f32>()) * 0.2,
        );
        self.did_spit.store(true, Relaxed);
    }

    fn make_new_llama(&self) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            entity.entity_type,
        );
        if self.is_trader_llama() {
            let trader = TraderLlamaEntity::new(baby);
            trader
                .get_mob_entity()
                .persistence_required
                .store(true, Relaxed);
            trader
        } else {
            Self::new(baby)
        }
    }
}

impl AgeableMob for LlamaEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for LlamaEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_LLAMA_FOOD)
    }

    fn can_mate(&self, partner: &dyn EntityBase) -> bool {
        as_llama(partner).is_some_and(|partner| self.can_parent() && partner.can_parent())
    }

    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let baby = self.make_new_llama();
        if let Some(baby_llama) = as_llama(baby.as_ref())
            && let Some(other) = as_llama(partner)
        {
            self.set_offspring_attributes(partner, baby.as_ref());
            let mut rng = self.get_random();
            let mut strength =
                rng.random_range(0..self.get_strength().max(other.get_strength())) + 1;
            if rng.random::<f32>() < 0.03 {
                strength += 1;
            }
            baby_llama.set_strength(strength);
            baby_llama.set_variant(if rng.random::<bool>() {
                self.get_variant()
            } else {
                other.get_variant()
            });
        }
        baby
    }
}

impl AbstractHorse for LlamaEntity {
    fn horse_data(&self) -> &AbstractHorseData {
        &self.horse_data
    }

    fn get_max_temper(&self) -> i32 {
        MAX_TEMPER
    }

    fn get_chest_equip_sound(&self) -> Sound {
        Sound::EntityLlamaChest
    }

    fn can_carry_chest(&self) -> bool {
        true
    }

    fn can_perform_rearing(&self) -> bool {
        false
    }

    fn can_eat_grass(&self) -> bool {
        false
    }

    fn randomize_attributes(&self) {
        let mut rng = self.get_random();
        let max_health = generate_max_health(|bound| rng.random_range(0..bound));
        self.mob_entity.living_entity.set_max_health(max_health);
    }

    fn follow_mommy(&self) {
        if !self.in_caravan() && self.is_baby() {
            follow_mommy(self);
        }
    }

    fn handle_eating(&self, player: &Arc<Player>, stack: &ItemStack) -> bool {
        let (age_up, temper, heal) = if stack.item == &Item::WHEAT {
            (10, 3, 2.0)
        } else if stack.item == &Item::HAY_BLOCK {
            (90, 6, 10.0)
        } else {
            (0, 0, 0.0)
        };
        let mut item_used = false;
        if stack.item == &Item::HAY_BLOCK
            && self.is_tamed()
            && self.get_age() == 0
            && !self.mob_entity.is_in_love()
        {
            item_used = true;
            self.set_in_love(player);
        }

        let living = &self.mob_entity.living_entity;
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
            self.play_varied_sound(Sound::EntityLlamaEat);
        }
        item_used
    }

    fn do_player_ride(&self, player: &Arc<Player>) {
        if !(self.is_trader_llama() && self.is_leashed_to_wandering_trader()) {
            start_riding(self, player);
        }
    }

    fn bedrock_equip_slot(&self) -> usize {
        MountScreenHandler::SLOT_BODY_ARMOR
    }

    fn bedrock_equip_slots(&self) -> NbtCompound {
        bedrock_equip_slot_nbt("minecraft:carpet", 1)
    }

    fn bedrock_inventory_size(&self) -> i32 {
        BEDROCK_CHESTED_INVENTORY_SIZE
    }

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
        data.set(
            entity_data_key::CONTAINER_STRENGTH_MODIFIER,
            MetadataValue::Int(BEDROCK_CONTAINER_STRENGTH_MODIFIER),
        );
        data.set(
            entity_data_key::STRENGTH,
            MetadataValue::Int(self.get_strength()),
        );
        data
    }

    fn get_inventory_columns(&self) -> i32 {
        if self.has_chest() {
            self.get_strength()
        } else {
            0
        }
    }
}

impl CustomSound for LlamaEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityLlamaDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityLlamaHurt)
    }

    fn sound_volume(&self) -> f32 {
        crate::entity::passive::abstract_horse::SOUND_VOLUME
    }

    fn sound_category(&self) -> SoundCategory {
        self.sound_source()
    }
}

impl Mob for LlamaEntity {
    fn on_climbable(&self) -> bool {
        false
    }

    fn on_elastic_leash_pull(&self) {
        if self.is_eating() {
            self.set_eating(false);
        }
    }

    fn cause_fall_damage(
        &self,
        caller: &dyn EntityBase,
        fall_distance: f32,
        damage_modifier: f32,
    ) -> bool {
        let damage = self.calculate_fall_damage(fall_distance, damage_modifier);
        if damage <= 0 {
            return false;
        }
        let living = &self.mob_entity.living_entity;
        if fall_distance >= LLAMA_SAFE_FALL_DISTANCE {
            living.damage(caller, damage as f32, DamageType::FALL);
            living.propagate_fall_to_passengers(fall_distance, damage_modifier);
        }
        living.play_block_fall_sound(caller);
        true
    }

    fn mob_is_pushable(&self) -> bool {
        !self.get_entity().has_passengers()
    }

    fn is_tamed(&self) -> bool {
        self.get_flag(FLAG_TAME)
    }

    fn get_owner_uuid(&self) -> Option<uuid::Uuid> {
        self.get_owner()
    }

    fn follow_leash_speed(&self) -> f64 {
        FOLLOW_LEASH_SPEED
    }

    fn can_dispenser_equip_into_slot(&self, slot: &EquipmentSlot) -> bool {
        crate::entity::passive::abstract_horse::can_dispenser_equip_into_slot(self, slot)
    }

    fn is_mob_immobile(&self) -> bool {
        self.mob_entity.living_entity.is_dead_or_dying() || self.is_eating()
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(Sound::EntityLlamaAmbient)
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        crate::entity::passive::abstract_horse::AMBIENT_SOUND_INTERVAL
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityLlamaStep, 0.15, 1.0);
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

    fn as_abstract_horse(&self) -> Option<&dyn AbstractHorse> {
        Some(self)
    }

    fn drop_inventory(&self) {
        self.drop_horse_inventory();
    }

    fn tick_links(&self) -> Vec<i32> {
        self.caravan_head.load().into_iter().collect()
    }

    fn mob_ai_step(&self) {
        self.horse_ai_step();
    }

    fn finalize_spawn(
        &self,
        _world: &Arc<World>,
        _reason: crate::entity::mob::spawn::EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        self.set_random_strength();
        let (variant, group_size) = if let Some(SpawnGroupData::Llama {
            variant,
            group_size,
        }) = group_data
        {
            (variant, group_size)
        } else {
            let variants = LlamaVariant::ALL;
            (
                variants[self.get_random().random_range(0..variants.len())],
                0,
            )
        };
        self.set_variant(variant);
        self.randomize_attributes();
        let group_size = match self.finalize_ageable_spawn(Some(SpawnGroupData::Ageable {
            group_size,
            should_spawn_baby: true,
            baby_spawn_chance: DEFAULT_BABY_SPAWN_CHANCE,
        })) {
            Some(SpawnGroupData::Ageable { group_size, .. }) => group_size,
            _ => group_size + 1,
        };
        Some(SpawnGroupData::Llama {
            variant,
            group_size,
        })
    }

    fn mob_get_component(&self, component: DataComponent) -> Option<Box<dyn DataComponentImpl>> {
        match component {
            DataComponent::LlamaVariant => Some(Box::new(LlamaVariantImpl {
                value: self.get_variant().name().into(),
            })),
            _ => None,
        }
    }

    fn mob_set_variant_name(&self, name: &str) {
        if let Some(variant) = LlamaVariant::from_name(name) {
            self.set_variant(variant);
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_horse_nbt(nbt);
        self.write_chested_horse_nbt(nbt);
        nbt.put_int("Variant", self.get_variant().id());
        nbt.put_int("Strength", self.get_strength());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_strength(nbt.get_int("Strength").unwrap_or(0));
        self.read_horse_nbt(nbt);
        self.read_chested_horse_nbt(nbt);
        self.set_variant(
            nbt.get_int("Variant")
                .map_or(LlamaVariant::DEFAULT, LlamaVariant::by_id),
        );
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_bedrock_spawn_metadata(&self) -> Option<SyncedActorDataList> {
        let mut data = self
            .mob_metadata()
            .map(|meta| meta.to_bedrock_actor_data())
            .unwrap_or_default();
        data.0.extend(self.bedrock_container_data().0);
        data.0.extend(self.bedrock_variant_data().0);
        Some(data)
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        self.init_horse_data_tracker();
        self.init_chested_horse_data_tracker();
        entity.set_synced_data(
            tracked_data::llama::DATA_STRENGTH_ID,
            VarInt(self.get_strength()),
        );
        entity.set_synced_data(
            tracked_data::llama::DATA_VARIANT_ID,
            VarInt(self.get_variant().id()),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if let Some(result) = self.handle_important_interactions(player, item_stack) {
            return result;
        }
        if let Some(result) = self.interact_with_held_item(player, item_stack, true) {
            return result;
        }
        self.horse_mob_interact(player, item_stack)
    }
}

impl RangedAttackMob for LlamaEntity {
    fn perform_ranged_attack(&self, target: &Arc<dyn EntityBase>, _power: f32) {
        self.spit(target.as_ref());
    }
}
