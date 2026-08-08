use pumpkin_data::item::Item;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::potion::Effect;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::{TrackedData, TrackedId};
use pumpkin_inventory::build_equipment_slots;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_protocol::bedrock::client::take_item_actor::CTakeItemActor;
use pumpkin_protocol::bedrock::server::actor_event::{ActorEventType, SActorEvent};
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_util::GameMode;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::atomic::{
    AtomicBool, AtomicU8,
    Ordering::{Relaxed, SeqCst},
};
use std::{collections::HashMap, sync::atomic::AtomicI32};
use tracing::warn;

use super::experience_orb::ExperienceOrbEntity;
use super::{Entity, EntityBase, NBTStorage, NBTStorageInit};
use crate::block::OnLandedUponArgs;
use crate::entity::attributes::AttributeInstance;
use crate::entity::attributes::Modifier;
use crate::entity::attributes::ModifierOperation;
use crate::entity::combat::{breach_armor_fraction, knockback_after_resistance};
use crate::entity::mob::equipment::DEFAULT_EQUIPMENT_DROP_CHANCE;
use crate::entity::mob::slime::SlimeEntity;
use crate::entity::mob::sulfur_cube::SulfurCubeEntity;
use crate::entity::player::statistics::{CustomStatistic, StatisticCategory};
use crate::entity::{EntityBaseFuture, NbtFuture};
use crate::server::Server;
use crate::world::loot::{LootContextParameters, LootTableExt};
use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DeathMessageType;
use pumpkin_data::data_component_impl::Operation;
use pumpkin_data::data_component_impl::food::{
    ConsumableImpl, ConsumeAnimation, ConsumeEffect, UseRemainderImpl,
};
use pumpkin_data::data_component_impl::{
    AttributeModifiersImpl, BlocksAttacksImpl, DamageResistantImpl, DamageResistantType,
    DeathProtectionImpl, EnchantmentsImpl, EquipmentSlot, EquippableImpl, FoodImpl,
    OminousBottleAmplifierImpl,
};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::{EntityPose, EntityStatus, EntityType};
use pumpkin_data::item_stack::{DamageResult, ItemStack};
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::{Block, Enchantment, translation};
use pumpkin_data::{damage::DamageType, sound::Sound};
use pumpkin_inventory::entity_equipment::EntityEquipment;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{
    CEntityStatus, CHurtAnimation, CSetPlayerInventory, CTakeItemEntity, CUpdateMobEffect,
};
use pumpkin_protocol::{
    codec::item_stack_seralizer::ItemStackSerializer,
    java::client::play::{CDamageEvent, CSetEquipment, Metadata},
};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use rand::RngExt;
use std::sync::RwLock;
use tokio::sync::Mutex;
use uuid::Uuid;

fn knockback_strength_with_resistance(strength: f64, resistance: f64) -> f64 {
    strength * (1.0 - resistance.clamp(0.0, 1.0))
}

fn armor_resists_damage(stack: &ItemStack, damage_type: &DamageType) -> bool {
    let Some(resistant) = stack.get_data_component::<DamageResistantImpl>() else {
        return false;
    };

    match resistant.res_type {
        DamageResistantType::Fire => damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_FIRE),
        DamageResistantType::Explosion => {
            damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_EXPLOSION)
        }
        DamageResistantType::Fall => damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_FALL),
        DamageResistantType::Freezing => {
            damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_FREEZING)
        }
        DamageResistantType::Lightning => {
            damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_LIGHTNING)
        }
        DamageResistantType::Drowning => {
            damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_DROWNING)
        }
        DamageResistantType::Projectile => {
            damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_PROJECTILE)
        }
        DamageResistantType::PlayerAttack => {
            damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_PLAYER_ATTACK)
        }
        DamageResistantType::MaceSmash => {
            damage_type.has_tag(&tag::DamageType::MINECRAFT_MACE_SMASH)
        }
        _ => false,
    }
}

/// A raider's membership in an active `Raid`.
///
/// Mirrors `Raider.wave`/`raid`/`isPatrolLeader` (`Raider.java`). Kept on `LivingEntity` rather
/// than `MobEntity` so the wave-clear/death hook in `on_death` below can read it directly.
#[derive(Clone, Copy)]
pub struct RaidMembership {
    pub raid_id: i32,
    pub wave: i32,
    pub is_patrol_leader: bool,
}

/// Represents a living entity within the game world.
///
/// This struct encapsulates the core properties and behaviors of living entities, including players, mobs, and other creatures.
pub struct LivingEntity {
    /// The underlying entity object, providing basic entity information and functionality.
    pub entity: Entity,
    /// Tracks the remaining time until the entity can regenerate health.
    pub hurt_cooldown: AtomicI32,
    /// Stores the amount of damage the entity last received.
    pub last_damage_taken: AtomicCell<f32>,
    /// The current health level of the entity.
    pub health: AtomicCell<f32>,
    /// The current absorption (yellow hearts) on the entity.
    pub absorption: AtomicCell<f32>,
    pub item_use_time: AtomicI32,
    pub item_in_use: Mutex<Option<ItemStack>>,
    pub active_hand: Mutex<Option<Hand>>,
    pub death_time: AtomicU8,
    /// Indicates whether the entity is dead. (`on_death` called)
    pub dead: AtomicBool,
    /// The distance the entity has been falling.
    pub fall_distance: AtomicCell<f32>,
    pub active_effects: Mutex<HashMap<&'static StatusEffect, Effect>>,
    pub entity_equipment: Arc<Mutex<EntityEquipment>>,
    pub equipment_drop_chances: Arc<Mutex<HashMap<EquipmentSlot, f32>>>,
    pub movement_input: AtomicCell<Vector3<f64>>,
    /// `LivingEntity.speed` in vanilla: the per-tick movement factor consumed by
    /// `travel`/`getFrictionInfluencedSpeed`. For players this is the raw
    /// `MOVEMENT_SPEED` attribute (`Player.aiStep`), for mobs it is
    /// `speedModifier * MOVEMENT_SPEED` as written by `MoveControl` via `Mob.setSpeed`.
    pub speed: AtomicCell<f64>,
    pub equipment_slots: Arc<HashMap<usize, EquipmentSlot>>,

    pub jumping: AtomicBool,

    pub jumping_cooldown: AtomicU8,

    pub climbing: AtomicBool,

    /// The position where the entity was last climbing, used for death messages
    pub climbing_pos: AtomicCell<Option<BlockPos>>,

    /// The entity ID of the entity that last attacked this living entity.
    pub last_attacker_id: AtomicI32,
    /// The tick at which this entity was last attacked (entity age).
    pub last_attacked_time: AtomicI32,

    /// The entity ID of the entity this living entity last attacked.
    pub last_attacking_id: AtomicI32,
    /// The tick at which this entity last attacked something (entity age).
    pub last_attack_time: AtomicI32,

    /// Vanilla `LivingEntity.canBeSeenAsEnemy()` override hook (e.g. `Axolotl.canBeSeenAsEnemy`
    /// while playing dead): when `true`, target-selection AI (`TargetPredicate`) treats this
    /// entity as unattackable, independent of `can_take_damage` -- players and existing attackers
    /// can still damage it normally, it just won't be picked as a *new* AI target.
    pub not_targetable_as_enemy: AtomicBool,

    water_movement_speed_multiplier: f32,
    livings_flags: AtomicU8,

    /// The attributes of the entity
    pub attributes: RwLock<HashMap<u8, AttributeInstance>>,

    /// `Raider.raid`/`Raider.wave`/`Raider.isPatrolLeader` (`Raider.java`).
    pub raid_membership: AtomicCell<Option<RaidMembership>>,
    /// `Raider.canJoinRaid` (`Raider.java`).
    pub can_join_raid: AtomicBool,
    /// `Raider.ticksOutsideRaid` (`Raider.java`).
    pub ticks_outside_raid: AtomicI32,
}

impl LivingEntity {
    pub fn knockback_with_resistance(&self, strength: f64, x: f64, z: f64) {
        let resistance = self.get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE);
        self.entity.knockback(
            knockback_strength_with_resistance(strength, resistance),
            x,
            z,
        );
    }

    const USING_ITEM_FLAG: u8 = 1;
    const OFF_HAND_ACTIVE_FLAG: u8 = 2;
    #[expect(dead_code)]
    const USING_RIPTIDE_FLAG: u8 = 4;

    const PREVENT_AREA_FALL_DAMAGE_BLOCKS: [&'static Block; 4] = [
        &Block::COBWEB,
        &Block::LADDER,
        &Block::POWDER_SNOW,
        &Block::SLIME_BLOCK,
    ];

    fn hurt_sound_for_entity(entity_type: &'static EntityType) -> Sound {
        entity_type.hurt_sound.unwrap_or(Sound::EntityGenericHurt)
    }

    #[inline]
    pub fn is_undead(&self) -> bool {
        self.entity
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_INVERTED_HEALING_AND_HARM)
    }

    pub(crate) const fn instant_effect_is_damage(
        effect_type: &'static StatusEffect,
        inverted: bool,
    ) -> bool {
        (effect_type.id == StatusEffect::INSTANT_HEALTH.id && inverted)
            || (effect_type.id == StatusEffect::INSTANT_DAMAGE.id && !inverted)
    }

    pub fn new(entity: Entity) -> Self {
        let water_movement_speed_multiplier = if entity.entity_type == &EntityType::POLAR_BEAR {
            0.98
        } else if entity.entity_type == &EntityType::SKELETON_HORSE {
            0.96
        } else {
            0.8
        };
        let mut max_health: f32 = 20.0; // Overridden by attribute base below
        let mut base_movement_speed = Attributes::MOVEMENT_SPEED.default_value;
        Self {
            // Populate local attribute instances from the default registry and get initial vars
            attributes: {
                let mut m = std::collections::HashMap::new();

                for (attr, base) in entity.entity_type.attributes {
                    if attr.id == Attributes::MAX_HEALTH.id {
                        max_health = *base as f32;
                    }
                    if attr.id == Attributes::MOVEMENT_SPEED.id {
                        base_movement_speed = *base;
                    }
                    m.insert(attr.id, AttributeInstance::new(*base));
                }
                std::sync::RwLock::new(m)
            },
            health: AtomicCell::new(max_health), // Initial health value from attributes
            entity,
            hurt_cooldown: AtomicI32::new(0),
            last_damage_taken: AtomicCell::new(0.0),
            absorption: AtomicCell::new(0.0),
            fall_distance: AtomicCell::new(0.0),
            death_time: AtomicU8::new(0),
            dead: AtomicBool::new(false),
            item_use_time: AtomicI32::new(0),
            item_in_use: Mutex::new(None),
            active_hand: Mutex::new(None),
            livings_flags: AtomicU8::new(0),
            active_effects: Mutex::new(HashMap::new()),
            entity_equipment: Arc::new(Mutex::new(EntityEquipment::new())),
            equipment_drop_chances: Arc::new(Mutex::new(HashMap::new())),
            equipment_slots: Arc::new(build_equipment_slots()),
            jumping: AtomicBool::new(false),
            jumping_cooldown: AtomicU8::new(0),
            climbing: AtomicBool::new(false),
            climbing_pos: AtomicCell::new(None),
            last_attacker_id: AtomicI32::new(0),
            last_attacked_time: AtomicI32::new(0),
            last_attacking_id: AtomicI32::new(0),
            last_attack_time: AtomicI32::new(0),
            not_targetable_as_enemy: AtomicBool::new(false),
            movement_input: AtomicCell::new(Vector3::default()),
            speed: AtomicCell::new(base_movement_speed),
            water_movement_speed_multiplier,
            raid_membership: AtomicCell::new(None),
            can_join_raid: AtomicBool::new(false),
            ticks_outside_raid: AtomicI32::new(0),
        }
    }

    pub fn send_equipment_changes(&self, equipment: &[(EquipmentSlot, ItemStack)]) {
        if equipment.is_empty() {
            return;
        }
        let equipment: Vec<(i8, ItemStackSerializer)> = equipment
            .iter()
            .map(|(slot, stack)| {
                (
                    slot.discriminant(),
                    ItemStackSerializer::from(stack.clone()),
                )
            })
            .collect();
        self.entity.world.load().broadcast_packet_except(
            &[self.entity.entity_uuid],
            &CSetEquipment::new(self.entity_id().into(), equipment),
        );
    }

    /// Picks up and Item entity or XP Orb
    pub fn pickup(&self, item: &Entity, stack_amount: u32) {
        let chunk_pos = self.entity.chunk_pos.load();
        self.entity.world.load().broadcast_to_chunk_editioned_sync(
            chunk_pos,
            &CTakeItemEntity::new(
                item.entity_id.into(),
                self.entity.entity_id.into(),
                VarInt(stack_amount as i32),
            ),
            &CTakeItemActor::new(
                VarULong(item.entity_id as u64),
                VarULong(self.entity.entity_id as u64),
            ),
        );
    }

    /// Sends the Hand animation to all others, used when Eating for example
    pub async fn set_active_hand(&self, hand: Hand, stack: ItemStack, duration: i32) {
        self.item_use_time.store(duration, Ordering::Relaxed);
        *self.item_in_use.lock().await = Some(stack);
        *self.active_hand.lock().await = Some(hand);
        self.set_living_flag(Self::USING_ITEM_FLAG, true);
        self.set_living_flag(Self::OFF_HAND_ACTIVE_FLAG, hand == Hand::Left);
    }

    fn set_living_flag(&self, flag: u8, value: bool) {
        let index = flag;
        let mut b = self.livings_flags.load(Ordering::Relaxed);
        if value {
            b |= index;
        } else {
            b &= !index;
        }
        self.livings_flags.store(b, Ordering::Relaxed);

        let bedrock_meta = (flag == Self::USING_ITEM_FLAG).then(|| {
            use pumpkin_protocol::bedrock::client::set_actor_data::{
                EntityMetadata, MetadataValue, entity_data_flag, entity_data_key,
            };
            let mut meta = EntityMetadata::new();
            // The Bedrock FLAGS field is a full bitfield: a SetActorData that
            // carries it replaces the client's entire flag set. Building this
            // from an empty metadata sent FLAGS with only USING_ITEM, which
            // cleared HAS_GRAVITY (and HAS_COLLISION, CLIMB, BREATHING) that were
            // set on spawn, so the moment a player used an item the client
            // stopped applying gravity and floated. Seed the field with the
            // entity's accumulated flags before toggling USING_ITEM so gravity
            // is preserved.
            meta.set(
                entity_data_key::FLAGS,
                MetadataValue::Long(self.entity.bedrock_flags.load(Ordering::Relaxed)),
            );
            meta.set_flag(
                entity_data_key::FLAGS,
                entity_data_flag::USING_ITEM as u8,
                value,
            );
            meta
        });

        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::LIVING_ENTITY_FLAGS,
                MetaDataType::BYTE,
                b,
            )],
            bedrock_meta.as_ref(),
        );
    }

    pub async fn clear_active_hand(&self) {
        *self.item_in_use.lock().await = None;
        *self.active_hand.lock().await = None;
        self.item_use_time.store(0, Ordering::Relaxed);

        self.set_living_flag(Self::USING_ITEM_FLAG, false);
    }

    /// Vanilla: `Raider::hasActiveRaid` (`getCurrentRaid() != null && raid.isActive()`).
    ///
    /// Approximation: reads the cached `RaidMembership` captured at spawn time rather than
    /// re-querying `RaidManager`; membership is cleared on death (see `on_death` below), so
    /// `is_some()` is a reasonable proxy for a live raider's `hasActiveRaid` for the raid-gated
    /// goals that consume this (Vindicator's door goals, Witch's heal-raiders goal).
    #[must_use]
    pub fn has_active_raid(&self) -> bool {
        self.raid_membership.load().is_some()
    }

    /// Vanilla: `Raider::getWave` (via `getCurrentRaid()`/`Raid.getGroupsSpawned() + 1`).
    #[must_use]
    pub fn raid_wave(&self) -> Option<i32> {
        self.raid_membership.load().map(|m| m.wave)
    }

    pub async fn is_blocking(&self) -> bool {
        let item_in_use = self.item_in_use.lock().await;
        if let Some(item) = item_in_use.as_ref()
            && item.get_data_component::<BlocksAttacksImpl>().is_some()
        {
            let use_time = self.item_use_time.load(Ordering::Relaxed);
            return item.get_max_use_time() - use_time >= 5;
        }
        false
    }

    pub fn heal(&self, additional_health: f32) {
        assert!(additional_health > 0.0);
        self.set_health(self.health.load() + additional_health);
    }

    pub fn set_health(&self, health: f32) {
        // Clamp to [0, max_health]
        let max_health = self.get_max_health();
        let clamped = health.max(0.0).min(max_health);
        self.health.store(clamped);
        // tell everyone entities health changed
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::HEALTH_ID,
                MetaDataType::FLOAT,
                clamped,
            )],
            None,
        );
    }

    /// Returns the current maximum health for this entity
    pub fn get_max_health(&self) -> f32 {
        self.get_attribute_value(&Attributes::MAX_HEALTH) as f32
    }

    /// Sets the maximum health for this entity
    pub async fn set_max_health(&self, max_health: f32) {
        // Update base attribute
        self.set_attribute_base(&Attributes::MAX_HEALTH, max_health as f64);

        // Broadcast the attribute change
        crate::entity::attributes::send_attribute_updates_for_living(
            self,
            vec![Attributes::MAX_HEALTH],
        )
        .await;

        // Clamp current health to new max if needed and send metadata update
        let current_health = self.health.load();
        if current_health > max_health {
            self.set_health(max_health);
        }
    }

    /// Returns the current absorption amount for this entity (yellow hearts)
    pub fn get_absorption(&self) -> f32 {
        self.absorption.load()
    }

    /// Sets the current absorption amount for this entity (yellow hearts)
    pub async fn set_absorption(&self, new_abs: f32) {
        // Must be at least 0
        let new_abs = new_abs.max(0.0);

        // Set local state
        self.absorption.store(new_abs);

        // Broadcast attribute update for max_absorption so clients receive
        // the updated absorption value via the attribute packet.
        crate::entity::attributes::send_attribute_updates_for_living(
            self,
            vec![Attributes::MAX_ABSORPTION],
        )
        .await;

        // Send absorption metadata for players (visual yellow hearts)
        if let Some(tracked_id) = self.player_absorption_id() {
            self.entity.send_meta_data(
                &[Metadata::new(tracked_id, MetaDataType::FLOAT, new_abs)],
                None,
            );
        }
    }

    /// Returns the absorption ID for this (player) entity
    /// TODO: don't hardcode these here?
    fn player_absorption_id(&self) -> Option<TrackedId> {
        (self.entity.entity_type == &EntityType::PLAYER).then_some(TrackedId {
            v1_21: 17u8,
            v1_21_2: 17u8,
            v1_21_4: 17u8,
            v1_21_5: 17u8,
            v1_21_6: 17u8,
            v1_21_7: 17u8,
            v1_21_9: 17u8,
            v1_21_11: 17u8,
            v26_1: 17u8, // ?
            v26_2: 17u8,
        })
    }

    /// Convenience helper to mutate an attribute instance. Automatically inserts
    /// a new instance populated from the registry base if needed.
    pub fn update_attribute<F: FnOnce(&mut AttributeInstance)>(
        &self,
        attribute: &Attributes,
        f: F,
    ) {
        let mut map = self.attributes.write().unwrap();

        let inst = map.entry(attribute.id).or_insert_with(|| {
            let base = self
                .entity
                .entity_type
                .attributes
                .iter()
                .find(|a| a.0.id == attribute.id)
                .map_or_else(
                    || {
                        tracing::warn!(
                            "Entity type {:?} has no base value for attribute {:?}; falling back to default {}",
                            self.entity.entity_type,
                            attribute.id,
                            attribute.default_value,
                        );
                        attribute.default_value
                    },
                    |a| a.1,
                );
            AttributeInstance::new(base)
        });

        f(inst);
        inst.dirty.store(true, Ordering::Relaxed);
    }

    /// Returns the computed value for `attribute` using the local instance, falling back
    /// to `attribute.default_value` if no local instance exists.
    pub fn get_attribute_value(&self, attribute: &Attributes) -> f64 {
        let map = self.attributes.read().unwrap();
        map.get(&attribute.id)
            .map_or(attribute.default_value, AttributeInstance::value)
    }

    /// `Mob.setSpeed`: stores the movement factor and mirrors it into the forward
    /// movement input (`setZza`).
    pub fn set_speed(&self, speed: f64) {
        self.speed.store(speed);
        let mut input = self.movement_input.load();
        input.z = speed;
        self.movement_input.store(input);
    }

    /// `speedModifier * MOVEMENT_SPEED`, the value `MoveControl` passes to `Mob.setSpeed`.
    #[must_use]
    pub fn speed_for_modifier(&self, speed_modifier: f64) -> f64 {
        speed_modifier * self.get_attribute_value(&Attributes::MOVEMENT_SPEED)
    }

    /// Returns the base attribute value for `attribute` for this entity's type.
    pub fn get_attribute_base(&self, attribute: &Attributes) -> f64 {
        // Check the local base value first (could be modified)
        let map = self.attributes.read().unwrap();
        if let Some(instance) = map.get(&attribute.id) {
            return instance.base_value;
        }

        // Fall back to registry base value if no local instance exists
        self.entity
            .entity_type
            .attributes
            .iter()
            .find(|a| a.0.id == attribute.id)
            .unwrap()
            .1
    }

    /// Update or insert the base value for an attribute on this entity.
    /// If the attribute doesn't exist locally yet, it will be inserted.
    pub fn set_attribute_base(&self, attribute: &Attributes, new_base: f64) {
        let mut map = self.attributes.write().unwrap();
        if let Some(inst) = map.get_mut(&attribute.id) {
            inst.base_value = new_base;
            inst.dirty.store(true, Ordering::Relaxed);
        } else {
            let ai = AttributeInstance::new(new_base);
            ai.dirty.store(true, Ordering::Relaxed);
            map.insert(attribute.id, ai);
        }
    }

    pub async fn reset_effects_and_attributes(&self) {
        // Clear active effects and reset modified attributes
        let effects_to_remove: Vec<_> = {
            let lock = self.active_effects.lock().await;
            lock.keys().copied().collect()
        };

        for effect_type in effects_to_remove {
            self.remove_effect(effect_type).await;
        }
    }

    pub const fn entity_id(&self) -> i32 {
        self.entity.entity_id
    }

    /// `LivingEntity#canBeAffected`. Only `Parched#canBeAffected` (Parched.java), which makes
    /// Parched immune to the Weakness its own arrows apply, is modelled here. The base
    /// class's `IMMUNE_TO_INFESTED` / `IMMUNE_TO_OOZING` / `IGNORES_POISON_AND_REGEN`
    /// entity-type-tag checks and the `Spider`, `WitherSkeleton`, `WitherBoss` and
    /// `AbstractNautilus` overrides are still unported.
    #[must_use]
    pub fn can_be_affected(&self, effect_type: &StatusEffect) -> bool {
        !(self.entity.entity_type == &EntityType::PARCHED
            && effect_type.id == StatusEffect::WEAKNESS.id)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "effect application also synchronizes attributes"
    )]
    pub async fn add_effect(&self, effect: Effect) {
        if !self.can_be_affected(effect.effect_type) {
            return;
        }
        let inverted = self.is_undead();
        let is_instant = effect.effect_type.id == StatusEffect::INSTANT_HEALTH.id
            || effect.effect_type.id == StatusEffect::INSTANT_DAMAGE.id;
        if !Self::instant_effect_is_damage(effect.effect_type, inverted) && is_instant {
            let heal_amount = 4.0 * (1 << effect.amplifier) as f32;
            self.heal(heal_amount);
            return;
        } else if is_instant {
            let damage_amount = 6.0 * (1 << effect.amplifier) as f32;
            if let Some(dyn_self) = self
                .entity
                .world
                .load()
                .get_entity_by_id(self.entity.entity_id)
            {
                dyn_self
                    .damage(&*dyn_self, damage_amount, DamageType::MAGIC)
                    .await;
            }
        } else {
            let did_apply = {
                let mut active_effects = self.active_effects.lock().await;
                let should_replace = active_effects
                    .get(effect.effect_type)
                    .is_none_or(|current| should_replace_effect(current, &effect));
                if should_replace {
                    active_effects.insert(effect.effect_type, effect.clone());
                }
                should_replace
            };

            if !did_apply {
                return;
            }

            // Effects that modify attributes (ex. speed) should also update the
            // entity's attribute instances (server-side) and then notify clients.
            if !effect.effect_type.attribute_modifiers.is_empty() {
                // Apply each attribute modifier into the local AttributeInstance
                for m in effect.effect_type.attribute_modifiers {
                    let id = m.id.to_string();
                    let op = match m.operation {
                        Operation::AddValue => ModifierOperation::Add,
                        Operation::AddMultipliedBase => ModifierOperation::MultiplyBase,
                        Operation::AddMultipliedTotal => ModifierOperation::MultiplyTotal,
                    };
                    let scaled_amount = m.base_value * (f64::from(effect.amplifier) + 1.);
                    let mod_inst = Modifier {
                        id,
                        amount: scaled_amount,
                        operation: op,
                    };

                    self.update_attribute(m.attribute, |inst| {
                        inst.add_or_replace_modifier(mod_inst.clone());
                    });
                }

                // Recompute packet modifiers from active effects for each affected attribute
                let mut touched_attrs: Vec<pumpkin_data::attributes::Attributes> = Vec::new();
                for m in effect.effect_type.attribute_modifiers {
                    if !touched_attrs.iter().any(|a| a.id == m.attribute.id) {
                        touched_attrs.push(m.attribute.clone());
                    }
                }

                if !touched_attrs.is_empty() {
                    crate::entity::attributes::send_attribute_updates_for_living(
                        self,
                        touched_attrs,
                    )
                    .await;
                }
            }

            // Apply absorption effect (+4 absorption per level)
            if effect.effect_type == &StatusEffect::ABSORPTION {
                let added = 4.0 * (effect.amplifier as f32 + 1.0);
                let max_abs = self.get_attribute_value(&Attributes::MAX_ABSORPTION) as f32;
                let new_abs = (self.absorption.load() + added).min(max_abs);
                self.set_absorption(new_abs).await;
            }

            // Apply invisible effect
            if effect.effect_type == &StatusEffect::INVISIBILITY {
                self.entity.set_invisible(true).await;
            }

            // Apply glowing effect
            if effect.effect_type == &StatusEffect::GLOWING {
                self.entity.set_glowing(true).await;
            }
        }

        // Broadcast effect to nearby players
        let mut flag: i8 = 0;
        if effect.ambient {
            flag |= 1;
        }
        if effect.show_particles {
            flag |= 2;
        }
        if effect.show_icon {
            flag |= 4;
        }
        if effect.blend {
            flag |= 8;
        }

        let packet = CUpdateMobEffect::new(
            self.entity.entity_id.into(),
            VarInt(i32::from(effect.effect_type.id)),
            effect.amplifier.into(),
            effect.duration.into(),
            flag,
        );

        self.entity.world.load().broadcast_packet_all(&packet);
    }

    pub async fn remove_all_effects(&self) -> bool {
        let effect_list: Vec<&'static StatusEffect> =
            self.active_effects.lock().await.keys().copied().collect();

        let mut succeeded = false;
        for effect_type in effect_list {
            succeeded |= self.remove_effect(effect_type).await;
        }
        succeeded
    }

    pub async fn remove_effect(&self, effect_type: &'static StatusEffect) -> bool {
        // Remove the effect
        let succeeded = self
            .active_effects
            .lock()
            .await
            .remove(&effect_type)
            .is_some();

        if !succeeded {
            return false;
        }

        // Broadcast effect removal
        self.entity
            .world
            .load()
            .send_remove_mob_effect(&self.entity, effect_type);

        // Remove attribute modifiers, if any
        if !effect_type.attribute_modifiers.is_empty() {
            let mut touched_attrs = Vec::new();

            for m in effect_type.attribute_modifiers {
                let id = m.id.to_string();

                // Clean local server state
                self.update_attribute(m.attribute, |inst| {
                    inst.remove_modifier(&id);
                });

                // Track unique attributes for the packet update
                if !touched_attrs
                    .iter()
                    .any(|a: &Attributes| a.id == m.attribute.id)
                {
                    touched_attrs.push(m.attribute.clone());
                }
            }

            // Sync the clean state to the client
            if !touched_attrs.is_empty() {
                crate::entity::attributes::send_attribute_updates_for_living(self, touched_attrs)
                    .await;
            }
        }

        // If absorption effect removed, clear current absorption amount and notify clients
        if effect_type == &StatusEffect::ABSORPTION {
            self.set_absorption(0.0).await;
        }

        // If health boost effect removed, clamp current health to new max and notify clients
        if effect_type == &StatusEffect::HEALTH_BOOST {
            let new_max = self.get_max_health();
            if self.health.load() > new_max {
                // Update local health and send both health and absorption metadata together
                self.set_health(new_max.max(0.0));
            }
        }

        // If invisible effect removed, disable invisibility
        if effect_type == &StatusEffect::INVISIBILITY {
            self.entity.set_invisible(false).await;
        }

        // If glowing effect removed, disable glowing
        if effect_type == &StatusEffect::GLOWING {
            self.entity.set_glowing(false).await;
        }

        succeeded
    }

    pub async fn has_effect(&self, effect: &'static StatusEffect) -> bool {
        let effects = self.active_effects.lock().await;
        effects.contains_key(&effect)
    }

    pub async fn get_effect(&self, effect: &'static StatusEffect) -> Option<Effect> {
        let effects = self.active_effects.lock().await;
        effects.get(&effect).cloned()
    }

    pub fn is_in_fall_damage_resetting(&self) -> (bool, &Block) {
        let block_pos = self.entity.block_pos.load();
        let block = self.entity.world.load().get_block(&block_pos);
        (
            block.has_tag(&tag::Block::MINECRAFT_FALL_DAMAGE_RESETTING),
            block,
        )
    }

    // Check if the entity is in water
    pub fn is_in_water(&self) -> bool {
        let block_pos = self.entity.block_pos.load();
        self.entity.world.load().get_block(&block_pos) == &Block::WATER
    }

    // Check if the entity is in powder snow
    pub fn is_in_powder_snow(&self) -> bool {
        let block_pos = self.entity.block_pos.load();
        self.entity.world.load().get_block(&block_pos) == &Block::POWDER_SNOW
    }

    pub fn should_prevent_fall_damage(&self) -> bool {
        let (prevents, block) = self.is_in_fall_damage_resetting();

        if block == &Block::SCAFFOLDING && !self.entity.is_sneaking() {
            return false;
        }

        if block == &Block::WATER {
            return true;
        }

        if self.entity.entity_type == &EntityType::PLAYER {
            if block == &Block::END_GATEWAY || block == &Block::END_PORTAL {
                return true;
            }

            if block == &Block::NETHER_PORTAL {
                let world = self.entity.world.load();
                let level_info = world.level_info.load();

                return level_info.game_rules.players_nether_portal_default_delay == 0;
            }
        }

        prevents
    }

    pub fn should_prevent_fall_damage_in_area(&self) -> bool {
        let world = self.entity.world.load();
        let block_pos = self.entity.block_pos.load().down();
        let entity_pos = self.entity.pos.load();

        let min = BlockPos(Vector3::new(
            block_pos.0.x - 1,
            block_pos.0.y,
            block_pos.0.z - 1,
        ));
        let max = BlockPos(Vector3::new(
            block_pos.0.x + 1,
            block_pos.0.y,
            block_pos.0.z + 1,
        ));
        let pos_iter = BlockPos::iterate(min, max);

        // FIXME: it seems the java server checks all blocks around with a raycast and check if miss or hit,
        // then added to a collision checker to handle in the tick handler
        for pos in pos_iter {
            let block = world.get_block(&pos);

            if Self::PREVENT_AREA_FALL_DAMAGE_BLOCKS.contains(&block) {
                let block_center = Vector3::new(
                    f64::from(pos.0.x) + 0.5,
                    f64::from(pos.0.y) + 0.5,
                    f64::from(pos.0.z) + 0.5,
                );
                let distance = entity_pos.squared_distance_to_vec(&block_center);

                // Fetch safe fall distance from attribute
                let safe_distance = self.get_attribute_value(&Attributes::SAFE_FALL_DISTANCE);
                return distance.sqrt() <= safe_distance * safe_distance;
            }
        }

        false
    }

    pub fn is_immune_to_fall_damage(&self) -> bool {
        self.entity
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_FALL_DAMAGE_IMMUNE)
    }

    async fn get_effective_gravity(&self, caller: &Arc<dyn EntityBase>) -> f64 {
        let final_gravity = caller.get_gravity();

        if self.entity.velocity.load().y <= 0.0
            && self.has_effect(&StatusEffect::SLOW_FALLING).await
        {
            final_gravity.min(0.01)
        } else {
            final_gravity
        }
    }

    pub async fn swing_hand(&self) {
        let world = self.entity.world.load();
        let entity_id = self.entity_id();

        let je_packet = pumpkin_protocol::java::client::play::CEntityAnimation::new(
            entity_id.into(),
            pumpkin_protocol::java::client::play::Animation::SwingMainArm,
        );
        let be_packet = pumpkin_protocol::bedrock::server::animate::SAnimate {
            action: pumpkin_protocol::bedrock::server::animate::AnimateAction::SwingArm,
            runtime_entity_id: pumpkin_protocol::codec::var_ulong::VarULong(entity_id as u64),
            data: 0.0,
            swing_source: None,
        };

        world.broadcast_editioned(&je_packet, &be_packet).await;
    }

    async fn tick_movement<'a>(&'a self, server: &'a Server, caller: &'a Arc<dyn EntityBase>) {
        if self.jumping_cooldown.load(Relaxed) != 0 {
            self.jumping_cooldown.fetch_sub(1, Relaxed);
        }

        let should_swim_in_fluids = if let Some(player) = caller.get_player() {
            !player.is_flying().await
        } else {
            true
        };

        self.entity.check_zero_velo();

        // Player.aiStep: players refresh `speed` from the attribute every tick. Mobs get
        // theirs from MoveControl/navigation as `speedModifier * MOVEMENT_SPEED`.
        if caller.get_player().is_some() {
            self.speed
                .store(self.get_attribute_value(&Attributes::MOVEMENT_SPEED));
        }

        let mut movement_input = self.movement_input.load();

        movement_input.x *= 0.98;

        movement_input.z *= 0.98;

        self.movement_input.store(movement_input);

        // TODO: Tick AI

        if self.jumping.load(SeqCst) && should_swim_in_fluids {
            let in_lava = self.entity.touching_lava.load(SeqCst);

            let in_water = self.entity.touching_water.load(SeqCst);

            let fluid_height = if in_lava {
                self.entity.lava_height.load()
            } else {
                self.entity.water_height.load()
            };

            let swim_height = self.get_swim_height();

            let on_ground = self.entity.on_ground.load(SeqCst);

            if (in_water || in_lava) && (!on_ground || fluid_height > swim_height) {
                // Swim upward

                let mut velo = self.entity.velocity.load();

                velo.y += 0.04;

                self.entity.velocity.store(velo);
            } else if (on_ground || in_water && fluid_height <= swim_height)
                && self.jumping_cooldown.load(SeqCst) == 0
            {
                self.jump().await;

                self.jumping_cooldown.store(10, SeqCst);
            }
        } else {
            self.jumping_cooldown.store(0, SeqCst);
        }

        if self.has_effect(&StatusEffect::SLOW_FALLING).await
            || self.has_effect(&StatusEffect::LEVITATION).await
        {
            self.fall_distance.store(0.0);
        }

        let custom_travel = if let Some(mob) = caller.get_mob() {
            mob.custom_travel(caller).await
        } else {
            false
        };

        if !custom_travel {
            let touching_water = self.entity.touching_water.load(SeqCst);

            // Strider is the only entity that has canWalkOnFluid = false

            if (touching_water || self.entity.touching_lava.load(SeqCst))
                && should_swim_in_fluids
                && self.entity.entity_type != &EntityType::STRIDER
            {
                self.travel_in_fluid(caller, touching_water).await;
            } else {
                // TODO: Gliding

                self.travel_in_air(caller).await;
            }
        }

        // TODO: Apply Soul Speed boot durability when tick_block_underneath is implemented.
        //self.entity.tick_block_underneath(&caller);

        let suffocating = self.entity.tick_block_collisions(caller, server).await;

        if suffocating {
            self.damage(&**caller, 1.0, DamageType::IN_WALL).await;
        }
    }

    async fn travel_in_air<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) {
        // applyMovementInput

        // LivingEntity.getFrictionInfluencedSpeed uses `getSpeed()`, not the raw attribute.
        let effective_speed = self.speed.load();

        let (speed, friction) = if self.entity.on_ground.load(SeqCst) {
            // getVelocityAffectingPos

            let slipperiness = f64::from(
                self.entity
                    .get_block_with_y_offset(0.500_001)
                    .1
                    .slipperiness,
            );

            (
                friction_influenced_speed(effective_speed, slipperiness),
                slipperiness * 0.91,
            )
        } else {
            let speed = if let Some(player) = caller.get_player() {
                player.get_off_ground_speed().await
            } else {
                // TODO: If the passenger is a player, ogs = movement_speed * 0.1

                0.02
            };

            (speed, 0.91)
        };

        self.entity
            .update_velocity_from_input(self.movement_input.load(), speed);

        self.apply_climbing_speed();

        self.make_move(caller).await;

        let mut velo = self.entity.velocity.load();

        let can_powder_snow_climb = if self.entity.was_in_powder_snow.load(Relaxed) {
            crate::block::blocks::powder_snow::can_entity_walk_on_powder_snow(caller.as_ref()).await
        } else {
            false
        };

        if (self.entity.horizontal_collision.load(SeqCst) || self.jumping.load(SeqCst))
            && (self.climbing.load(Relaxed) || can_powder_snow_climb)
        {
            velo.y = 0.2;
        }

        let levitation = self.get_effect(&StatusEffect::LEVITATION).await;

        if let Some(lev) = levitation {
            velo.y += 0.05f64.mul_add(f64::from(lev.amplifier + 1), -velo.y) * 0.2;
        } else {
            velo.y -= self.get_effective_gravity(caller).await;

            // TODO: If world is not loaded: replace effective gravity with:

            // if below world's bottom y then -0.1, else 0.0
        }

        // If entity has no drag: store velo and return

        velo.x *= friction;

        velo.z *= friction;

        velo.y *= caller.get_y_velocity_drag().unwrap_or_else(|| {
            if caller.is_flutterer() {
                friction
            } else {
                0.98
            }
        });

        self.entity.velocity.store(velo);
    }

    async fn travel_in_fluid<'a>(&'a self, caller: &'a Arc<dyn EntityBase>, water: bool) {
        let movement_input = self.movement_input.load();

        let old_y = self.entity.pos.load().y;
        let falling = self.entity.velocity.load().y <= 0.0;
        let gravity = self.get_effective_gravity(caller).await;
        // LivingEntity.travelInFluid also blends toward `getSpeed()`, not the raw attribute.
        let effective_speed = self.speed.load();

        if water {
            let mut friction = if self.entity.sprinting.load(Relaxed) {
                0.9
            } else {
                f64::from(self.water_movement_speed_multiplier)
            };

            let mut speed = 0.02;

            // Apply water movement efficiency attribute
            let mut water_movement_efficiency =
                self.get_attribute_value(&Attributes::WATER_MOVEMENT_EFFICIENCY);

            if water_movement_efficiency > 0.0 {
                if !self.entity.on_ground.load(SeqCst) {
                    water_movement_efficiency *= 0.5;
                }

                friction += (0.546_000_06 - friction) * water_movement_efficiency;
                speed += (effective_speed - speed) * water_movement_efficiency;
            }

            if self.has_effect(&StatusEffect::DOLPHINS_GRACE).await {
                friction = 0.96;
            }

            self.entity
                .update_velocity_from_input(movement_input, speed);

            self.make_move(caller).await;

            let mut velo = self.entity.velocity.load();
            if self.entity.horizontal_collision.load(SeqCst) && self.climbing.load(Relaxed) {
                velo.y = 0.2;
            }

            velo = velo.multiply(friction, 0.8, friction);

            self.apply_fluid_moving_speed(&mut velo.y, gravity, falling);
            self.entity.velocity.store(velo);
        } else {
            self.entity.update_velocity_from_input(movement_input, 0.02);

            self.make_move(caller).await;

            let mut velo = self.entity.velocity.load();

            if self.entity.lava_height.load() <= self.get_swim_height() {
                velo.x *= 0.5;
                velo.z *= 0.5;
                velo.y *= 0.8;

                self.apply_fluid_moving_speed(&mut velo.y, gravity, falling);
            } else {
                velo = velo * 0.5;
            }

            if gravity != 0.0 {
                velo.y -= gravity / 4.0; // Negative gravity = buoyancy
            }

            self.entity.velocity.store(velo);
        }

        let mut velo = self.entity.velocity.load();

        // Vanilla `LivingEntity.jumpOutOfFluid`: nudges the entity upward when it's swum
        // into a wall, but only if the space it would be nudged into is actually free
        // (`Entity.isFree` = no block collision *and* no liquid there, Entity.java:668-670).
        // This previously probed a box shifted by the raw velocity and checked only for
        // the absence of liquid, so pushing against solid ground or a wall while
        // horizontally colliding (e.g. a Drowned pathing along an uneven seafloor) would
        // repeatedly apply the 0.3 upward boost even though the probed space was inside a
        // solid block, launching the entity above the water surface.
        //
        // Vanilla probes `(movement.x, movement.y + 0.6 - getY() + oldY, movement.z)`: a
        // roughly fixed ~0.6-block step-up check, adjusted for how much of the intended Y
        // movement collision actually consumed this tick. `getY() - oldY` is the actual
        // net Y movement already applied by `make_move` above; the `+ 0.6` alone is used
        // when movement wasn't vertically obstructed.
        let world = self.entity.world.load();
        let actual_dy = self.entity.pos.load().y - old_y;
        let probe_box = self.entity.bounding_box.load().shift(Vector3::new(
            velo.x,
            velo.y + 0.6 - actual_dy,
            velo.z,
        ));
        if self.entity.horizontal_collision.load(SeqCst)
            && !world.check_fluid_collision(probe_box)
            && world.is_space_empty(probe_box)
        {
            velo.y = 0.3;

            self.entity.velocity.store(velo);
        }
    }

    fn apply_fluid_moving_speed(&self, dy: &mut f64, gravity: f64, falling: bool) {
        if gravity != 0.0 && !self.entity.sprinting.load(Relaxed) {
            if falling && (*dy - 0.005).abs() >= 0.003 && (*dy - gravity / 16.0).abs() < 0.003 {
                *dy = -0.003;
            } else {
                *dy -= gravity / 16.0;
            }
        }
    }

    async fn make_move<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) {
        self.entity
            .move_entity(caller, self.entity.velocity.load())
            .await;

        self.check_climbing();
    }

    fn check_climbing(&self) {
        if let Some(climbing) = spider_climbing_state(
            self.entity.entity_type,
            self.entity.horizontal_collision.load(SeqCst),
        ) {
            let was_climbing = self.climbing.swap(climbing, Relaxed);

            if climbing != was_climbing {
                self.entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::SPIDER_FLAGS,
                        MetaDataType::BYTE,
                        u8::from(climbing),
                    )],
                    None,
                );
            }
            return;
        }

        let pos = self.entity.block_pos.load();
        let world = self.entity.world.load();
        if world
            .get_block(&pos)
            .has_tag(&tag::Block::MINECRAFT_CLIMBABLE)
        {
            self.climbing.store(true, Relaxed);
            self.climbing_pos.store(Some(pos));
            return;
        }

        self.climbing.store(false, Relaxed);

        if self.entity.on_ground.load(SeqCst) {
            self.climbing_pos.store(None);
        }
    }

    fn apply_climbing_speed(&self) {
        if self.climbing.load(Relaxed) {
            self.fall_distance.store(0.0);

            let mut velo = self.entity.velocity.load();

            let pos = 0.15;

            let neg = -0.15;

            if velo.x < neg {
                velo.x = neg;
            } else if velo.x > pos {
                velo.x = pos;
            }

            if velo.z < neg {
                velo.z = neg;
            } else if velo.z > pos {
                velo.z = pos;
            }

            velo.y = velo.y.max(neg);

            // TODO
            // if velo.y < 0.0
            //     && self.entity.entity_type == &EntityType::PLAYER
            //     && self.entity.sneaking.load(Relaxed)
            // {
            //     let block = self
            //         .entity
            //         .world
            //         .read()
            //         .await
            //         .get_block(&self.entity.block_pos.load())
            //         .await;

            //     if let Some(props) = block.properties(block.default_state.id) {
            //         if props.name() == "ScaffoldingLikeProperties" {
            //             velo.y = 0.0;
            //         }
            //     }
            // }

            self.entity.velocity.store(velo);
        }
    }

    pub fn get_swim_height(&self) -> f64 {
        let eye_height = self.entity.get_eye_height();

        if self.entity.entity_type == &EntityType::BREEZE {
            eye_height
        } else if eye_height < 0.4 {
            0.0
        } else {
            0.4
        }
    }

    async fn jump(&self) {
        let jump = self.get_jump_velocity(1.0).await;

        if jump <= 1.0e-5 {
            return;
        }

        let mut velo = self.entity.velocity.load();

        velo.y = jump.max(velo.y);

        if self.entity.sprinting.load(Relaxed) {
            let yaw = f64::from(self.entity.yaw.load()).to_radians();

            velo.x -= yaw.sin() * 0.2;
            velo.z += yaw.cos() * 0.2;
        }

        self.entity.velocity.store(velo);

        self.entity.velocity_dirty.store(true, SeqCst);
    }

    async fn get_jump_velocity(&self, mut strength: f64) -> f64 {
        strength *= self.get_attribute_value(&Attributes::JUMP_STRENGTH);
        strength *= f64::from(self.entity.get_jump_velocity_multiplier());
        if let Some(effect) = self.get_effect(&StatusEffect::JUMP_BOOST).await {
            strength += 0.1 * f64::from(effect.amplifier + 1);
        }
        strength
    }

    pub async fn fall(
        &self,
        caller: Arc<dyn EntityBase>,
        height_difference: f64,
        ground: bool,
        dont_damage: bool,
    ) {
        if ground {
            let fall_distance = self.fall_distance.swap(0.0);
            if fall_distance <= 0.0
                || dont_damage
                || self.should_prevent_fall_damage()
                || self.should_prevent_fall_damage_in_area()
                || self.is_immune_to_fall_damage()
            {
                return;
            }
            let world = self.entity.world.load();
            let landed_pos = self.entity.get_pos_with_y_offset(0.2).0;
            let block = world.get_block(&landed_pos);
            let pumpkin_block = world.block_registry.get_pumpkin_block(block.id);
            if let Some(pumpkin_block) = pumpkin_block {
                pumpkin_block
                    .on_landed_upon(OnLandedUponArgs {
                        world: &world,
                        position: &landed_pos,
                        fall_distance,
                        entity: caller.as_ref(),
                    })
                    .await;
            } else {
                self.handle_fall_damage(&*caller, fall_distance, 1.0).await;
            }
        } else if height_difference < 0.0 {
            let new_fall_distance = if !self.should_prevent_fall_damage()
                && !self.should_prevent_fall_damage_in_area()
            {
                let distance = self.fall_distance.load();
                distance - (height_difference as f32)
            } else {
                0f32
            };
            self.fall_distance.store(new_fall_distance);
        }
    }

    pub async fn handle_fall_damage(
        &self,
        caller: &dyn EntityBase,
        fall_distance: f32,
        damage_per_distance: f32,
    ) {
        if self.is_immune_to_fall_damage() {
            return;
        }

        // Fetches the safe fall distance attribute
        let safe_fall_distance = self.get_attribute_value(&Attributes::SAFE_FALL_DISTANCE) as f32;
        let unsafe_fall_distance = fall_distance + 1.0E-6 - safe_fall_distance;

        let damage = (unsafe_fall_distance * damage_per_distance).floor();
        if damage > 0.0 {
            let check_damage = self.damage(caller, damage, DamageType::FALL).await; // Fall
            if check_damage {
                self.entity
                    .play_sound(Self::get_fall_sound(fall_distance as i32));
            }
        }
    }

    const fn get_fall_sound(distance: i32) -> Sound {
        if distance > 4 {
            Sound::EntityGenericBigFall
        } else {
            Sound::EntityGenericSmallFall
        }
    }

    pub async fn get_death_message(
        dyn_self: &dyn EntityBase,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> TextComponent {
        match damage_type.death_message_type {
            DeathMessageType::Default => {
                if let Some(cause) = cause
                    && source.is_some()
                {
                    TextComponent::translate_cross(
                        format!("death.attack.{}.player", damage_type.message_id),
                        format!("death.attack.{}.player", damage_type.message_id),
                        [
                            dyn_self.get_display_name().await,
                            cause.get_display_name().await,
                        ],
                    )
                } else {
                    TextComponent::translate_cross(
                        format!("death.attack.{}", damage_type.message_id),
                        format!("death.attack.{}", damage_type.message_id),
                        [dyn_self.get_display_name().await],
                    )
                }
            }
            DeathMessageType::FallVariants => {
                //TODO
                TextComponent::translate_cross(
                    translation::java::DEATH_FELL_ACCIDENT_GENERIC,
                    translation::bedrock::DEATH_FELL_ACCIDENT_GENERIC,
                    [dyn_self.get_display_name().await],
                )
            }
            DeathMessageType::IntentionalGameDesign => TextComponent::text("[")
                .add_child(TextComponent::translate_cross(
                    format!("death.attack.{}.message", damage_type.message_id),
                    format!("death.attack.{}.message", damage_type.message_id),
                    [dyn_self.get_display_name().await],
                ))
                .add_child(TextComponent::text("]")),
        }
    }

    /// Vanilla `Villager.releaseAllPois` (called from `Villager.die`):
    /// release any claimed POI ticket (bed, job site) on death so it isn't
    /// held forever by an entity that no longer exists. Uses the generic
    /// `EntityBase::get_home_pos`/`get_job_site_pos` hooks (default `None`)
    /// rather than downcasting to `VillagerEntity`, so any future
    /// POI-claiming mob gets this for free.
    async fn release_claimed_pois(world: &crate::world::World, dyn_self: &dyn EntityBase) {
        if let Some(home) = dyn_self.get_home_pos() {
            world.release_poi(home).await;
        }
        if let Some(job_site) = dyn_self.get_job_site_pos() {
            world.release_poi(job_site).await;
        }
    }

    pub async fn on_death(
        &self,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) {
        let world = self.entity.world.load();
        if self
            .dead
            .compare_exchange(false, true, Relaxed, Relaxed)
            .is_ok()
        {
            self.movement_input.store(Vector3::default());
            self.jumping.store(false, Relaxed);

            let Some(dyn_self) = world.get_entity_by_id(self.entity.entity_id) else {
                // Damage can finish after the entity has been removed by another task.
                // Vanilla's entity remains addressable during die(); do not bring down
                // the server when Pumpkin observes the equivalent race.
                return;
            };

            Self::release_claimed_pois(&world, &*dyn_self).await;

            // Statistics updates
            self.update_death_stats(&*dyn_self, cause).await;

            // Raider.die (Raider.java): report the removal to the owning raid, hand out
            // Hero of the Village credit, and clear the wave-leader slot if this was the banner
            // carrier.
            if let Some(membership) = self.raid_membership.swap(None) {
                let mut raids = world.raids.lock().await;
                if let Some(raid) = raids.raid_mut(membership.raid_id) {
                    if membership.is_patrol_leader {
                        raid.remove_leader(membership.wave);
                    }
                    if let Some(killer) = cause
                        && killer.get_entity().entity_type.id == EntityType::PLAYER.id
                    {
                        raid.add_hero_of_the_village(killer.get_entity().entity_uuid);
                    }
                    raid.remove_from_raid(membership.wave, self.entity.entity_uuid, 0.0, false);
                }
            }

            // Plays the death sound
            world.send_entity_status(&self.entity, EntityStatus::Death);
            let looting_level;
            let tool = if let Some(cause_ent) = cause {
                if let Some(player) = cause_ent
                    .cast_any()
                    .downcast_ref::<crate::entity::player::Player>()
                {
                    let hand_stack = player
                        .inventory
                        .get_stack_in_hand(pumpkin_util::Hand::Right)
                        .await;
                    let stack_guard = hand_stack.lock().await;
                    looting_level = stack_guard
                        .get_enchantment_level(&Enchantment::LOOTING)
                        .max(0) as u32;
                    (stack_guard.item_count > 0).then(|| stack_guard.clone())
                } else {
                    looting_level = 0;
                    None
                }
            } else {
                looting_level = 0;
                None
            };

            let is_raining = world.is_raining().await;
            let is_thundering = world.is_thundering().await;

            let params = LootContextParameters {
                killed_by_player: cause.map(|c| c.get_entity().entity_type == &EntityType::PLAYER),
                this_entity: Some(self.entity.entity_type),
                killer_entity: cause.map(|c| c.get_entity().entity_type),
                direct_killer_entity: source.map(|s| s.get_entity().entity_type),
                position: Some(self.entity.pos.load()),
                world_time: world.level_info.load().day_time as u64,
                damage_type: Some(damage_type),
                tool,
                is_raining: Some(is_raining),
                is_thundering: Some(is_thundering),
                is_on_fire: Some(
                    self.entity
                        .fire_ticks
                        .load(std::sync::atomic::Ordering::Relaxed)
                        > 0,
                ),
                ..Default::default()
            };

            if let Some(mob) = dyn_self.get_mob() {
                mob.on_mob_death(cause).await;
            }

            // LivingEntity.die, line 1472: this.gameEvent(GameEvent.ENTITY_DIE), fired
            // right before dropAllDeathLoot. Entity::gameEvent(event) (Entity.java:1431)
            // defaults the source entity to `this`.
            crate::world::game_event::emit_game_event(
                &world,
                pumpkin_data::game_event::GameEvent::EntityDie,
                self.entity.pos.load(),
                crate::world::game_event::GameEventContext::of_entity(dyn_self.clone()),
            )
            .await;

            // Drop loot
            self.drop_loot(params.clone()).await;

            // Award experience
            if params.killed_by_player.unwrap_or(false)
                && world.level_info.load().game_rules.mob_drops
            {
                let amount = dyn_self.get_experience_reward(cause);
                if amount > 0 {
                    ExperienceOrbEntity::spawn(&world, self.entity.pos.load(), amount).await;
                }
            }
            self.entity.pose.store(EntityPose::Dying);

            self.drop_equipment(looting_level).await;

            // Broadcast death message if it's a player and the gamerule is enabled
            self.broadcast_death_message(&*dyn_self, damage_type, source, cause)
                .await;

            self.reset_effects_and_attributes().await;
        }
    }

    async fn drop_equipment(&self, looting_level: u32) {
        let world = self.entity.world.load();
        let block_pos = self.entity.block_pos.load();

        let drop_chances = self.equipment_drop_chances.lock().await;

        let slots_to_drop: Vec<EquipmentSlot> = {
            let mut slots: Vec<_> = self.equipment_slots.values().cloned().collect();
            slots.push(EquipmentSlot::MAIN_HAND);
            slots
        };

        for slot in &slots_to_drop {
            let mut chance = drop_chances
                .get(slot)
                .copied()
                .unwrap_or(DEFAULT_EQUIPMENT_DROP_CHANCE);
            // Vanilla approximation: EnchantmentHelper.processEquipmentDropChance
            // adds lootingLevel * 0.01 to the per-slot equipment drop chance.
            chance += looting_level as f32 * 0.01;
            chance = chance.min(1.0);
            if rand::random::<f32>() >= chance {
                continue;
            }
            let mut item = {
                let q = self.entity_equipment.lock().await;
                let item_arc = q.get(slot);
                let mut item_lock = item_arc.lock().await;
                mem::replace(&mut *item_lock, ItemStack::EMPTY.clone())
            };
            if item.is_empty() {
                continue;
            }
            // Vanilla approximation: Mob.dropCustomDeathLoot applies random
            // damage to dropped equipment using two chained random calls:
            // setDamageValue(maxDamage - random.nextInt(1 + random.nextInt(max(maxDamage - 3, 1))))
            if let Some(max_damage) = item.get_max_damage() {
                let mut rng = rand::rng();
                let inner = rng.random_range(0..(max_damage - 3).max(1));
                let outer = rng.random_range(0..=inner);
                item.set_damage((max_damage - outer).max(0));
            }
            world.drop_stack(&block_pos, item).await;
        }
    }

    async fn broadcast_death_message(
        &self,
        dyn_self: &dyn EntityBase,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) {
        let world = self.entity.world.load();
        let show_death_messages = { world.level_info.load().game_rules.show_death_messages };
        if self.entity.entity_type == &EntityType::PLAYER && show_death_messages {
            //TODO: KillCredit
            let death_message = Self::get_death_message(dyn_self, damage_type, source, cause).await;
            if let Some(server) = world.server.upgrade() {
                for player in server.get_all_players() {
                    player.send_system_message(&death_message).await;
                }
            }
        }
    }

    async fn update_death_stats(&self, dyn_self: &dyn EntityBase, cause: Option<&dyn EntityBase>) {
        if let Some(victim_player) = dyn_self.get_player() {
            victim_player
                .increment_stat(StatisticCategory::Custom, CustomStatistic::Deaths as i32, 1)
                .await;
            victim_player
                .set_stat(
                    StatisticCategory::Custom,
                    CustomStatistic::TimeSinceDeath as i32,
                    0,
                )
                .await;
            if let Some(killer_entity) = cause.map(EntityBase::get_entity) {
                victim_player
                    .increment_stat(
                        StatisticCategory::KilledBy,
                        killer_entity.entity_type.id as i32,
                        1,
                    )
                    .await;
            }
        }

        if let Some(killer_player) = cause.and_then(|c| c.get_player()) {
            if dyn_self.get_player().is_some() {
                killer_player
                    .increment_stat(
                        StatisticCategory::Custom,
                        CustomStatistic::PlayerKills as i32,
                        1,
                    )
                    .await;
            } else {
                killer_player
                    .increment_stat(
                        StatisticCategory::Custom,
                        CustomStatistic::MobKills as i32,
                        1,
                    )
                    .await;

                let resource_name = self.entity.entity_type.resource_name;
                let criterion_key = format!("minecraft:{resource_name}");
                killer_player
                    .trigger_advancement(
                        crate::entity::player::advancement::trigger::AdvancementTrigger::PlayerKilledEntity {
                            entity_type_resource: criterion_key,
                        }
                    )
                    .await;

                if resource_name == "skeleton" {
                    let distance_sq = killer_player
                        .position()
                        .squared_distance_to_vec(&self.entity.pos.load());
                    if distance_sq >= 2500.0 {
                        killer_player.trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::SniperDuel).await;
                    }
                }

                if resource_name == "phantom" {
                    killer_player.trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::TwoBirdsOneArrow).await;
                }

                let held_item = killer_player.inventory().held_item();
                let is_crossbow = {
                    let lock = held_item.lock().await;
                    lock.item.registry_key == "crossbow"
                };
                if is_crossbow {
                    killer_player.trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::Arbalistic).await;
                }
            }
            killer_player
                .increment_stat(
                    StatisticCategory::Killed,
                    self.entity.entity_type.id as i32,
                    1,
                )
                .await;
        }
    }

    async fn drop_loot(&self, params: LootContextParameters) {
        if let Some(loot_table) = &self.get_entity().entity_type.loot_table {
            let pos = self.entity.block_pos.load();
            for stack in loot_table.get_loot(params) {
                self.entity.world.load().drop_stack(&pos, stack).await;
            }
        }
    }

    async fn tick_effects(&self) {
        let mut effects_to_remove = Vec::new();
        let mut effects_to_apply = Vec::new();

        {
            let mut effects = self.active_effects.lock().await;
            let entity_age = self.entity.age.load(Relaxed);
            for effect in effects.values_mut() {
                if effect.duration == 0 {
                    effects_to_remove.push(effect.effect_type);
                    continue;
                }

                let tick_duration = if effect.duration == -1 {
                    entity_age
                } else {
                    effect.duration
                };

                if Self::should_apply_effect_tick(effect, tick_duration) {
                    effects_to_apply.push((effect.effect_type, effect.amplifier));
                }

                if effect.duration != -1 {
                    effect.duration -= 1;
                }
            }
        }

        // Call the central removal function for each expired effect
        // This will now trigger your logs and absorption resets!
        for effect_type in effects_to_remove {
            self.remove_effect(effect_type).await;
        }

        for (effect_type, amplifier) in effects_to_apply {
            self.apply_effect_tick(effect_type, amplifier).await;
        }
    }

    /// Determines if an effect should apply its tick effect this frame
    /// Based on vanilla Minecraft's effect tick frequencies
    ///
    /// TODO: villager, beacon, and other effects.
    fn should_apply_effect_tick(effect: &pumpkin_data::potion::Effect, duration: i32) -> bool {
        let effect_type = effect.effect_type;

        if effect_type == &StatusEffect::REGENERATION {
            if duration <= 0 {
                return false;
            }
            let tick_rate = 50 >> effect.amplifier.min(4);
            duration % tick_rate == 0
        } else if effect_type == &StatusEffect::POISON {
            if duration <= 0 {
                return false;
            }
            let tick_rate = 25 >> effect.amplifier.min(4);
            duration % tick_rate == 0
        } else if effect_type == &StatusEffect::WITHER {
            if duration <= 0 {
                return false;
            }
            let tick_rate = 40 >> effect.amplifier.min(4);
            duration % tick_rate == 0
        } else if effect_type == &StatusEffect::HUNGER {
            // Hunger every 20 ticks
            duration % 20 == 0
        } else if effect_type == &StatusEffect::SATURATION {
            // Saturation every tick
            true
        } else if effect_type == &StatusEffect::BAD_OMEN {
            // BadOmenMobEffect.shouldApplyEffectTickThisTick: every tick.
            true
        } else if effect_type == &StatusEffect::RAID_OMEN {
            // RaidOmenMobEffect.shouldApplyEffectTickThisTick: only on the last tick before
            // natural expiry.
            duration == 1
        } else {
            // Other effects that don't tick
            false
        }
    }

    /// Applies the actual effect to the entity
    /// This is called by `tick_effects` when an effect should trigger this tick
    async fn apply_effect_tick(&self, effect_type: &'static StatusEffect, amplifier: u8) {
        if effect_type == &StatusEffect::REGENERATION {
            let current_health = self.health.load();
            let max_health = self.get_max_health();
            if current_health < max_health && current_health > 0.0 {
                self.heal(1.0);
            }
        } else if effect_type == &StatusEffect::POISON {
            let current_health = self.health.load();
            if current_health > 1.0
                && let Some(dyn_self) = self
                    .entity
                    .world
                    .load()
                    .get_entity_by_id(self.entity.entity_id)
            {
                let damage_amount = (current_health - 1.0).min(1.0);
                if damage_amount > 0.0 {
                    dyn_self
                        .damage(&*dyn_self, damage_amount, DamageType::MAGIC)
                        .await;
                }
            }
        } else if effect_type == &StatusEffect::WITHER {
            let damage_amount = 1.0;
            if let Some(dyn_self) = self
                .entity
                .world
                .load()
                .get_entity_by_id(self.entity.entity_id)
            {
                dyn_self
                    .damage(&*dyn_self, damage_amount, DamageType::WITHER)
                    .await;
            }
        } else if effect_type == &StatusEffect::HUNGER {
            let world = self.entity.world.load();
            if let Some(entity) = world.get_entity_by_id(self.entity.entity_id)
                && let Some(player) = entity.get_player()
            {
                // Add exhaustion to trigger hunger decrease
                let exhaustion = 0.1 * (amplifier as f32 + 1.0);
                player.hunger_manager.add_exhaustion(exhaustion);
            }
            drop(world);
        } else if effect_type == &StatusEffect::SATURATION {
            let world = self.entity.world.load();
            if let Some(entity) = world.get_entity_by_id(self.entity.entity_id)
                && let Some(player) = entity.get_player()
            {
                // Add hunger and saturation
                let hunger = amplifier + 1;
                player.hunger_manager.add_hunger(hunger);
                player.hunger_manager.add_saturation(hunger as f32 * 2.0);
            }
        } else if effect_type == &StatusEffect::BAD_OMEN {
            // BadOmenMobEffect.applyEffectTick: on entering a real village (with room left
            // to raise the raid's omen level), convert BAD_OMEN into RAID_OMEN and remember
            // where, so RaidOmenMobEffect can trigger the raid when RAID_OMEN expires.
            let world = self.entity.world.load();
            if let Some(entity) = world.get_entity_by_id(self.entity.entity_id)
                && let Some(player) = entity.get_player()
                && player.gamemode.load() != pumpkin_util::GameMode::Spectator
                && world.level_info.load().difficulty != pumpkin_util::Difficulty::Peaceful
            {
                let pos = BlockPos::floored_v(self.entity.pos.load());
                if world.is_close_to_village(pos, 1).await {
                    let raids = world.raids.lock().await;
                    let raid_ok = raids.raid_omen_level_near(pos).is_none_or(|level| {
                        level < crate::world::raid::DEFAULT_MAX_RAID_OMEN_LEVEL
                    });
                    drop(raids);
                    if raid_ok {
                        self.remove_effect(&StatusEffect::BAD_OMEN).await;
                        self.add_effect(Effect {
                            effect_type: &StatusEffect::RAID_OMEN,
                            duration: 600,
                            amplifier,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                            blend: false,
                        })
                        .await;
                        player.raid_omen_position.store(Some(pos));
                    }
                }
            }
        } else if effect_type == &StatusEffect::RAID_OMEN {
            // RaidOmenMobEffect.applyEffectTick: on expiry, trigger/extend the raid at the
            // stored position, then let the effect's own duration reach 0 as normal.
            let world = self.entity.world.load();
            if let Some(entity) = world.get_entity_by_id(self.entity.entity_id)
                && let Some(player) = entity.get_player()
                && let Some(pos) = player.raid_omen_position.swap(None)
            {
                let mut raids = world.raids.lock().await;
                raids.create_or_extend_raid(&world, player, pos).await;
            }
        }
    }

    /// Tries to use a totem of undying from the entity's hands. If successful, applies the totem effects and returns true.
    async fn try_use_death_protector(&self, caller: &dyn EntityBase) -> bool {
        for hand in Hand::all() {
            let stack = self.get_stack_in_hand(caller, hand).await;
            let mut stack = stack.lock().await;

            // Clear the stack and use the totem of undying
            if stack.get_data_component::<DeathProtectionImpl>().is_some() {
                stack.clear();
                self.set_health(1.0);
                self.entity
                    .world
                    .load()
                    .send_entity_status(&self.entity, EntityStatus::ProtectedFromDeath);

                self.remove_all_effects().await;

                // Set Absorption, Regeneration, and Fire Resistance effects
                self.add_effect(Effect {
                    effect_type: &StatusEffect::ABSORPTION,
                    duration: 100,
                    amplifier: 1,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                })
                .await;
                self.add_effect(Effect {
                    effect_type: &StatusEffect::REGENERATION,
                    duration: 900,
                    amplifier: 1,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                })
                .await;
                self.add_effect(Effect {
                    effect_type: &StatusEffect::FIRE_RESISTANCE,
                    duration: 800,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                })
                .await;

                return true;
            }
        }

        false
    }

    async fn damage_armor_items(
        &self,
        caller: &dyn EntityBase,
        damage_amount: f32,
        damage_type: &DamageType,
    ) {
        // Formula: armor loses floor(incoming_damage / 4) durability, minimum 1.
        let armor_damage = (damage_amount / 4.0).floor().max(1.0) as i32;
        let mut equipment_updates = Vec::new();

        // TODO: Implement DAMAGE_RESISTANT component checks (e.g. netherite vs fire).

        let helmet_only = damage_type.id == DamageType::FALLING_ANVIL.id
            || damage_type.id == DamageType::FALLING_BLOCK.id
            || damage_type.id == DamageType::FALLING_STALACTITE.id;

        let armor_slots: Vec<(usize, Arc<Mutex<ItemStack>>, EquipmentSlot)> = {
            let equipment_lock = self.entity_equipment.lock().await;
            self.equipment_slots
                .iter()
                .filter(|(_, slot)| {
                    slot.is_armor_slot() && (!helmet_only || **slot == EquipmentSlot::HEAD)
                })
                .map(|(index, slot)| (*index, equipment_lock.get(slot), slot.clone()))
                .collect()
        };

        for (slot_index, equipment, slot) in armor_slots {
            let (slot_result, updated_stack_opt) = {
                let mut stack = equipment.lock().await;
                if stack.is_empty() || armor_resists_damage(&stack, damage_type) {
                    (pumpkin_data::item_stack::DamageResult::Untouched, None)
                } else {
                    // Items without `EquippableImpl` component take damage freely.
                    // Items with `damage_on_hurt: false` (e.g. elytra) are exempt from armor hit durability.
                    // PERF: Component lookup runs O(1) per armor slot (max 4 per hit). Caching
                    // at the item type level could optimize, but belongs in a broader caching pass.
                    let takes_damage = stack
                        .get_data_component::<EquippableImpl>()
                        .is_none_or(|equippable| equippable.damage_on_hurt);

                    if takes_damage {
                        // Base armor durability damage.
                        let result = stack.damage_item(armor_damage);
                        let changed = result != pumpkin_data::item_stack::DamageResult::Untouched;
                        (result, changed.then_some(stack.clone()))
                    } else {
                        // Equippable items can opt out of on-hurt durability loss (e.g. elytra).
                        (pumpkin_data::item_stack::DamageResult::Untouched, None)
                    }
                }
            };

            if let Some(updated_stack) = updated_stack_opt {
                // Broadcast break status before clearing the slot.
                if slot_result == pumpkin_data::item_stack::DamageResult::Broken {
                    let world = self.entity.world.load();
                    world.send_entity_status(&self.entity, super::equipment_break_status(&slot));
                }
                equipment_updates.push((slot.clone(), updated_stack.clone()));
                if let Some(player) = caller.get_player() {
                    player
                        .enqueue_slot_set_packet(&CSetPlayerInventory::new(
                            (slot_index as i32).into(),
                            &ItemStackSerializer::from(updated_stack),
                        ))
                        .await;
                }
            }
        }

        if !equipment_updates.is_empty() {
            self.send_equipment_changes(&equipment_updates);
        }
    }

    pub async fn held_item(&self, caller: &dyn EntityBase) -> Arc<Mutex<ItemStack>> {
        if let Some(player) = caller.get_player() {
            return player.inventory.held_item();
        }
        self.entity_equipment
            .lock()
            .await
            .get(&EquipmentSlot::MAIN_HAND)
    }

    pub async fn get_stack_in_hand(
        &self,
        caller: &dyn EntityBase,
        hand: Hand,
    ) -> Arc<Mutex<ItemStack>> {
        match hand {
            Hand::Left => self.off_hand_item().await,
            Hand::Right => self.held_item(caller).await,
        }
    }

    /// getOffHandStack in source
    pub async fn off_hand_item(&self) -> Arc<Mutex<ItemStack>> {
        let slot = self
            .equipment_slots
            .get(&PlayerInventory::OFF_HAND_SLOT)
            .unwrap();
        self.entity_equipment.lock().await.get(slot)
    }

    pub fn can_take_damage(&self) -> bool {
        !self.entity.invulnerable.load(Ordering::Relaxed) && self.is_part_of_game()
    }

    pub fn is_part_of_game(&self) -> bool {
        !self.is_spectator() && self.entity.is_alive()
    }

    pub async fn reset_state(&self) {
        self.entity.reset_state().await;

        // Restore to maximum health for this entity type
        let max_health = self.get_max_health();
        self.set_health(max_health);
        // Clear any absorption
        self.absorption.store(0.0);
        // Send health metadata
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::HEALTH_ID,
                MetaDataType::FLOAT,
                max_health,
            )],
            None,
        );

        self.reset_effects_and_attributes().await;

        // Give a short grace period of invulnerability after respawn
        self.hurt_cooldown.store(20, Relaxed);
        self.last_damage_taken.store(0f32);

        self.entity.portal_cooldown.store(0, Relaxed);
        *self.entity.portal_manager.lock().await = None;

        // Clear fall/fire state
        self.fall_distance.store(0f32);
        self.death_time.store(0, Relaxed);
        self.entity.extinguish();
        self.entity.fire_ticks.store(0, Relaxed);

        // Clear velocity and movement input to remove persisted momentum
        self.entity.velocity.store(Vector3::default());
        self.entity.velocity_dirty.store(true, SeqCst);
        self.movement_input.store(Vector3::default());
        self.jumping.store(false, Relaxed);

        // If this LivingEntity corresponds to a Player, reset their hunger manager
        let world = self.entity.world.load();
        if let Some(player) = world.get_player_by_id(self.entity.entity_id) {
            player.hunger_manager.restart();
        }

        self.dead.store(false, Relaxed);
    }

    /// Try to spawn silverfish when this entity is infested and hurt.
    async fn try_spawn_infested_silverfish(&self) {
        if !self.has_effect(&StatusEffect::INFESTED).await {
            return;
        }

        // Wither, ender dragon and silverfish are immune
        if self.entity.entity_type == &EntityType::WITHER
            || self.entity.entity_type == &EntityType::ENDER_DRAGON
            || self.entity.entity_type == &EntityType::SILVERFISH
        {
            return;
        }

        let world = self.entity.world.load();

        // 10% chance
        if rand::rng().random::<f32>() <= 0.1 {
            let count = rand::rng().random_range(1..3);
            for _ in 0..count {
                // Spawn at center of entity
                let bbox = self.entity.bounding_box.load();
                let center = Vector3::new(
                    f64::midpoint(bbox.min.x, bbox.max.x),
                    f64::midpoint(bbox.min.y, bbox.max.y),
                    f64::midpoint(bbox.min.z, bbox.max.z),
                );

                // Random direction
                let yaw_rad = self.entity.yaw.load().to_radians() as f64;
                let random_angle = rand::rng().random::<f64>() * std::f64::consts::PI
                    - std::f64::consts::FRAC_PI_2;
                let angle = yaw_rad + random_angle;
                let speed = 0.3f64;
                let dx = -angle.sin() * speed;
                let dz = angle.cos() * speed;
                let dy = 0.1f64;

                // Spawn
                let silver = crate::entity::r#type::from_type(
                    &EntityType::SILVERFISH,
                    center,
                    &world,
                    Uuid::new_v4(),
                );

                silver.get_entity().set_pos(center);
                silver.get_entity().velocity.store(Vector3::new(dx, dy, dz));

                world.spawn_entity(silver).await;

                // Play sound
                world.play_sound(Sound::EntitySilverfishHurt, SoundCategory::Players, &center);
            }
        }
    }

    pub fn is_player(&self) -> bool {
        let world = self.entity.world.load();
        world.get_player_by_id(self.entity.entity_id).is_some()
    }

    pub fn get_movement(&self) -> Vector3<f64> {
        self.entity.movement.load()
    }

    fn hurt_sound(&self) -> Sound {
        if self.entity.entity_type == &EntityType::SLIME {
            SlimeEntity::hurt_sound_for_size(self.entity.data.load(Relaxed))
        } else if self.entity.entity_type == &EntityType::MAGMA_CUBE {
            SlimeEntity::magma_cube_hurt_sound_for_size(self.entity.data.load(Relaxed))
        } else if self.entity.entity_type == &EntityType::SULFUR_CUBE {
            SulfurCubeEntity::hurt_sound_for_size(self.entity.data.load(Relaxed))
        } else {
            Self::hurt_sound_for_entity(self.entity.entity_type)
        }
    }
}

impl NBTStorage for LivingEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.write_nbt(nbt).await;
            nbt.put("Health", NbtTag::Float(self.health.load()));
            // Avoid persisting a lethal fall distance when the entity is dead to prevent death loops
            let fall_distance = if self.dead.load(Relaxed) {
                0.0
            } else {
                self.fall_distance.load()
            };
            // Persist current absorption amount
            nbt.put("AbsorptionAmount", NbtTag::Float(self.absorption.load()));
            nbt.put("fall_distance", NbtTag::Float(fall_distance));
            {
                let effects = self.active_effects.lock().await;
                if !effects.is_empty() {
                    // Iterate effects and create Box<[NbtTag]>
                    let mut effects_list = Vec::with_capacity(effects.len());
                    for effect in effects.values() {
                        let mut effect_nbt = pumpkin_nbt::compound::NbtCompound::new();
                        effect.write_nbt(&mut effect_nbt).await;
                        effects_list.push(NbtTag::Compound(effect_nbt));
                    }
                    nbt.put("active_effects", NbtTag::List(effects_list));
                }
            }
            let equipment = self.entity_equipment.lock().await;
            let mut hand_items = Vec::with_capacity(2);
            for slot in [EquipmentSlot::MAIN_HAND, EquipmentSlot::OFF_HAND] {
                let stack_arc = equipment.get(&slot);
                let stack = stack_arc.lock().await;
                let mut item_nbt = NbtCompound::new();
                if !stack.is_empty() {
                    stack.write_item_stack(&mut item_nbt);
                }
                hand_items.push(NbtTag::Compound(item_nbt));
            }
            nbt.put("HandItems", NbtTag::List(hand_items));

            let mut armor_items = Vec::with_capacity(4);
            for slot in [
                EquipmentSlot::FEET,
                EquipmentSlot::LEGS,
                EquipmentSlot::CHEST,
                EquipmentSlot::HEAD,
            ] {
                let stack_arc = equipment.get(&slot);
                let stack = stack_arc.lock().await;
                let mut item_nbt = NbtCompound::new();
                if !stack.is_empty() {
                    stack.write_item_stack(&mut item_nbt);
                }
                armor_items.push(NbtTag::Compound(item_nbt));
            }
            nbt.put("ArmorItems", NbtTag::List(armor_items));
            // todo more...
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.entity.read_nbt_non_mut(nbt).await;
            self.health.store(nbt.get_float("Health").unwrap_or(0.0));

            // Clamp any persisted absorption to the entity's configured max
            let raw_abs = nbt.get_float("AbsorptionAmount").unwrap_or(0.0);
            let max_abs = self.get_attribute_value(&Attributes::MAX_ABSORPTION) as f32;
            let clamped_abs = raw_abs.max(0.0).min(max_abs);
            self.absorption.store(clamped_abs);

            // Load fall distance, but if this entity is currently marked dead ensure we don't restore
            // a lethal fall distance that would immediately re-kill on spawn.
            let fd = nbt.get_float("fall_distance").unwrap_or(0.0);
            if self.dead.load(Relaxed) {
                self.fall_distance.store(0.0);
            } else {
                self.fall_distance.store(fd);
            }
            {
                let mut active_effects = self.active_effects.lock().await;
                let nbt_effects = nbt.get_list("active_effects");
                if let Some(nbt_effects) = nbt_effects {
                    for effect in nbt_effects {
                        if let NbtTag::Compound(effect_nbt) = effect {
                            let effect = Effect::create_from_nbt(&mut effect_nbt.clone()).await;
                            if effect.is_none() {
                                warn!("Unable to read effect from nbt");
                                continue;
                            }
                            let mut effect = effect.unwrap();
                            effect.blend = true; // TODO: change, is taken from effect give command
                            active_effects.insert(effect.effect_type, effect);
                        }
                    }
                }
            }
            let mut equipment = self.entity_equipment.lock().await;
            for (key, slot) in [
                (
                    "HandItems",
                    [EquipmentSlot::MAIN_HAND, EquipmentSlot::OFF_HAND].as_slice(),
                ),
                (
                    "ArmorItems",
                    [
                        EquipmentSlot::FEET,
                        EquipmentSlot::LEGS,
                        EquipmentSlot::CHEST,
                        EquipmentSlot::HEAD,
                    ]
                    .as_slice(),
                ),
            ] {
                let Some(items) = nbt.get_list(key) else {
                    continue;
                };
                for (index, slot) in slot.iter().enumerate() {
                    let Some(compound) = items.get(index).and_then(NbtTag::extract_compound) else {
                        continue;
                    };
                    let Some(stack) = ItemStack::read_item_stack(compound) else {
                        continue;
                    };
                    *equipment.get_or_insert(slot).lock().await = stack;
                }
            }
        })
        // todo more...
    }
}

impl EntityBase for LivingEntity {
    #[allow(clippy::too_many_lines)]
    fn damage_with_context<'a>(
        &'a self,
        caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let mut amount = amount;

            // Check invulnerability before applying damage
            if self.entity.is_invulnerable_to(&damage_type).await {
                return false;
            }

            if self.health.load() <= 0.0 || self.dead.load(Relaxed) {
                return false; // Dying or dead
            }

            if amount < 0.0 {
                return false;
            }

            // Brain `HURT_BY`. Vanilla routes this through `HurtBySensor`
            // (`ai/sensing/HurtBySensor.java:18-28`), which each tick mirrors
            // `LivingEntity.getLastDamageSource()` -- a field that self-clears 40 ticks after
            // the last hit (`LivingEntity.java:1419-1425`). Pumpkin has no `lastDamageSource`
            // field, so the write happens here instead, with a 40-tick memory expiry standing
            // in for that self-clearing.
            //
            // This is the write path the split-lock design exists for: this method is called
            // from projectiles, blocks and fluids, i.e. from outside the damaged mob's own AI
            // tick. It works only because `MemoryStore` is never taken out of its mutex.
            // `HURT_BY_ENTITY` is not written -- no behavior in the current port reads it.
            if let Some(mob) = caller.get_mob()
                && let Some(brain) = mob.get_mob_entity().brain.as_ref()
            {
                brain.set_with_expiry::<crate::entity::ai::brain::memory::HurtByMemory>(
                    damage_type,
                    40,
                );
            }

            let world = self.entity.world.load();
            let is_fire_damage = damage_type == DamageType::IN_FIRE
                || damage_type == DamageType::ON_FIRE
                || damage_type == DamageType::LAVA
                || damage_type == DamageType::HOT_FLOOR;

            // Like fire and drowning damage, these are gated for players only;
            // mobs still take them.
            if self.entity.entity_type == &EntityType::PLAYER {
                let game_rules = &world.level_info.load().game_rules;
                if damage_type == DamageType::FALL && !game_rules.fall_damage {
                    return false;
                }
                if damage_type == DamageType::FREEZE && !game_rules.freeze_damage {
                    return false;
                }
            }

            // Fire damage can be prevented by either game rules or fire resistance
            if is_fire_damage {
                // Check game rule for fire damage (only for players)
                if self.entity.entity_type == &EntityType::PLAYER
                    && !world.level_info.load().game_rules.fire_damage
                {
                    return false;
                }

                // Check for fire resistance effect
                if self.has_effect(&StatusEffect::FIRE_RESISTANCE).await {
                    return false;
                }
            }

            // Check for shield blocking. Vanilla resolves blocking in `applyItemBlocking`
            // (hurtServer:1200) before the freeze multiplier (1203-1205) and the helmet
            // multiplier/hurtHelmet call (1207-1210), so this must run on the raw `amount`
            // before either multiplier is applied.
            let shield_source_position = position.or_else(|| {
                source
                    .or(cause)
                    .map(|entity| entity.get_entity().pos.load())
            });
            if self.is_blocking().await
                && !bypasses_shield(&damage_type)
                && let Some(pos) = shield_source_position
            {
                let player_pos = self.entity.pos.load();
                let look_vec = Vector3::rotation_vector(0.0, self.entity.yaw.load() as f64);
                let mut source_to_player = (player_pos - pos).normalize();
                source_to_player.y = 0.0;

                if source_to_player.dot(&look_vec) < 0.0 {
                    world.play_sound(Sound::ItemShieldBlock, SoundCategory::Players, &player_pos);

                    // Vanilla: `LivingEntity.blockUsingShield` -> `attacker.blockedByItem(this,
                    // source, damage)`. Called on the attacker, not the defender.
                    if let Some(attacker_mob) = cause.and_then(EntityBase::get_mob) {
                        attacker_mob.blocked_by_item(caller, amount).await;
                    }

                    if let Some(player) = caller.get_player() {
                        player
                            .increment_stat(
                                StatisticCategory::Custom,
                                CustomStatistic::DamageBlockedByShield as i32,
                                (amount * 10.0) as i32,
                            )
                            .await;

                        player.trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::DeflectedDamage).await;
                    }

                    if let Some(attacker_player) = cause.and_then(|c| c.get_player()) {
                        let held_item = attacker_player.inventory().held_item();
                        let is_axe = held_item.lock().await.is_axe();
                        if is_axe {
                            let mut disable_chance = 0.25;
                            let is_sprinting = attacker_player
                                .living_entity
                                .entity
                                .sprinting
                                .load(Ordering::Relaxed);
                            if is_sprinting {
                                disable_chance = 1.0;
                            }

                            if rand::random::<f32>() < disable_chance
                                && let Some(victim_player) = caller.get_player()
                            {
                                victim_player
                                    .start_cooldown("minecraft:shield".to_string(), 100)
                                    .await;
                                self.clear_active_hand().await;

                                world.broadcast_packet_all(&CEntityStatus::new(
                                    self.entity.entity_id,
                                    30,
                                ));
                            }
                        }
                    }

                    let active_hand = self.active_hand.lock().await;
                    if let Some(hand) = *active_hand
                        && amount >= 3.0
                    {
                        let slot = equipment_slot_for_hand(hand);

                        let equipment_lock = self.entity_equipment.lock().await;
                        let stack_arc = equipment_lock.get(&slot);
                        let mut stack = stack_arc.lock().await;

                        let durability_damage = amount.floor() as i32 + 1;
                        if stack.damage_item(durability_damage) == DamageResult::Broken {
                            if let Some(player) = caller.get_player() {
                                player
                                    .increment_stat(
                                        StatisticCategory::Broken,
                                        stack.item.id as i32,
                                        1,
                                    )
                                    .await;
                            }
                            world.send_entity_status(
                                &self.entity,
                                crate::entity::equipment_break_status(&slot),
                            );
                            *stack = ItemStack::EMPTY.clone();
                            let broken_stack = stack.clone();
                            drop(stack);
                            drop(stack_arc);
                            drop(equipment_lock);

                            self.send_equipment_changes(&[(slot, broken_stack)]);
                            self.clear_active_hand().await;
                        }
                    }

                    return false;
                }
            }

            // Vanilla parity: entities in FREEZE_HURTS_EXTRA_TYPES take 5x freezing damage.
            if damage_type == DamageType::FREEZE
                && self
                    .entity
                    .entity_type
                    .has_tag(&tag::EntityType::MINECRAFT_FREEZE_HURTS_EXTRA_TYPES)
            {
                amount *= 5.0;
            }

            if damages_helmet(&damage_type) {
                amount *= 0.75;
            }

            // These damage types bypass the hurt cooldown and death protection
            let bypasses_cooldown_protection =
                damage_type == DamageType::GENERIC_KILL || damage_type == DamageType::OUT_OF_WORLD;

            // Apply hurt cooldown logic. Vanilla compares and stores `damage` in the same
            // pre-armor/enchantment/resistance domain (LivingEntity.hurtServer:1217-1230):
            // only the freeze/helmet multipliers and shield blocking happen before this.
            let last_damage = self.last_damage_taken.load();
            let (raw_increment, play_sound) =
                if self.hurt_cooldown.load(Relaxed) > 10 && !bypasses_cooldown_protection {
                    if amount <= last_damage {
                        return false;
                    }
                    (amount - last_damage, false)
                } else {
                    self.hurt_cooldown.store(20, Relaxed);
                    (amount, true)
                };
            self.last_damage_taken.store(amount);

            // Armor, enchantment protection and resistance reduce only the incremental
            // damage actually dealt this hit, not the full raw amount (actuallyHurt calls
            // getDamageAfterArmorAbsorb/getDamageAfterMagicAbsorb on `damage - lastHurt`,
            // LivingEntity.java:1222,1953-1956).
            let mut damage_after_armor = raw_increment;
            if !bypasses_armor_durability(&damage_type) {
                let mut armor = 0.0f32;
                let mut toughness = 0.0f32;
                {
                    let equipment_lock = self.entity_equipment.lock().await;
                    for slot in [
                        EquipmentSlot::HEAD,
                        EquipmentSlot::CHEST,
                        EquipmentSlot::LEGS,
                        EquipmentSlot::FEET,
                    ] {
                        let stack_arc = equipment_lock.get(&slot);
                        let stack = stack_arc.lock().await;
                        if !stack.is_empty()
                            && let Some(modifiers) =
                                stack.get_data_component::<AttributeModifiersImpl>()
                        {
                            for modifier in modifiers.attribute_modifiers.iter() {
                                if modifier.r#type == &Attributes::ARMOR {
                                    armor += modifier.amount as f32;
                                } else if modifier.r#type == &Attributes::ARMOR_TOUGHNESS {
                                    toughness += modifier.amount as f32;
                                }
                            }
                        }
                    }
                }
                let value = 2.0f32 + toughness / 4.0;
                let clamped_armor = (armor - damage_after_armor / value)
                    .max(armor / 5.0)
                    .min(20.0);
                let mut armor_fraction = clamped_armor / 25.0;

                if let Some(attacker) = source
                    && let Some(player) = attacker
                        .cast_any()
                        .downcast_ref::<crate::entity::player::Player>()
                {
                    let held = player.inventory().held_item();
                    let breach_level = held
                        .lock()
                        .await
                        .get_enchantment_level(&Enchantment::BREACH);
                    if breach_level > 0 {
                        armor_fraction = breach_armor_fraction(armor_fraction, breach_level);
                    }
                }

                damage_after_armor *= 1.0 - armor_fraction;
            }

            // Apply Resistance unless the damage source bypasses effects or resistance.
            // Vanilla applies resistance before enchantment protection
            // (getDamageAfterMagicAbsorb, LivingEntity.java:1913-1926 resistance,
            // 1935-1948 enchantments).
            let resistance_reduction = if !damage_type
                .has_tag(&tag::DamageType::MINECRAFT_BYPASSES_EFFECTS)
                && !damage_type.has_tag(&tag::DamageType::MINECRAFT_BYPASSES_RESISTANCE)
            {
                self.get_effect(&StatusEffect::RESISTANCE)
                    .await
                    .map_or(0.0, |e| (0.2 * (e.amplifier + 1) as f32).min(1.0))
            } else {
                0.0
            };

            let damage_after_resistance = damage_after_armor * (1.0 - resistance_reduction);

            if resistance_reduction > 0.0 {
                let resisted = damage_after_armor * resistance_reduction;
                if let Some(player) = caller.get_player() {
                    player
                        .increment_stat(
                            StatisticCategory::Custom,
                            CustomStatistic::DamageResisted as i32,
                            (resisted * 10.0) as i32,
                        )
                        .await;
                }
                if let Some(attacker_player) = cause.and_then(|c| c.get_player()) {
                    attacker_player
                        .increment_stat(
                            StatisticCategory::Custom,
                            CustomStatistic::DamageDealtResisted as i32,
                            (resisted * 10.0) as i32,
                        )
                        .await;
                }
            }

            let mut damage_after_enchantments = damage_after_resistance;
            if !bypasses_enchantments(&damage_type) {
                let mut epf = 0i32;
                {
                    let equipment_lock = self.entity_equipment.lock().await;
                    for slot in [
                        EquipmentSlot::HEAD,
                        EquipmentSlot::CHEST,
                        EquipmentSlot::LEGS,
                        EquipmentSlot::FEET,
                    ] {
                        let stack_arc = equipment_lock.get(&slot);
                        let stack = stack_arc.lock().await;
                        if !stack.is_empty()
                            && let Some(enchantments) =
                                stack.get_data_component::<EnchantmentsImpl>()
                        {
                            for (enchantment, level) in enchantments.enchantment.iter() {
                                for effect in crate::enchantment::effects_for(enchantment) {
                                    let crate::enchantment::EnchantmentEffect::DamageProtection(
                                        condition,
                                        value,
                                    ) = effect
                                    else {
                                        continue;
                                    };
                                    let applies = match condition {
                                        crate::enchantment::ProtectionCondition::Always => {
                                            damage_type != DamageType::DROWN
                                                && damage_type != DamageType::STARVE
                                                && damage_type != DamageType::GENERIC_KILL
                                        }
                                        crate::enchantment::ProtectionCondition::IsFire => {
                                            is_fire_damage
                                        }
                                        crate::enchantment::ProtectionCondition::IsExplosion => {
                                            damage_type == DamageType::EXPLOSION
                                                || damage_type == DamageType::PLAYER_EXPLOSION
                                        }
                                        crate::enchantment::ProtectionCondition::IsProjectile => {
                                            damage_type == DamageType::ARROW
                                                || damage_type == DamageType::MOB_PROJECTILE
                                                || damage_type == DamageType::THROWN
                                        }
                                        crate::enchantment::ProtectionCondition::IsFall => {
                                            damage_type == DamageType::FALL
                                        }
                                    };
                                    if applies {
                                        epf += value.calculate(*level) as i32;
                                    }
                                }
                            }
                        }
                    }
                }
                epf = epf.min(20);
                if epf > 0 {
                    damage_after_enchantments *= 1.0 - (epf as f32 * 0.04);
                }
            }

            // Finalize state: damage actually applied this hit, after armor/resistance/enchant.
            let damage_amount = damage_after_enchantments.max(0.0);

            let config = &world.server.upgrade().unwrap().advanced_config.pvp;

            if config.hurt_animation {
                let entity_id = self.entity.entity_id;
                let hurt_yaw = source.map_or(0.0, |source| {
                    let src = source.get_entity().pos.load();
                    let tgt = self.entity.pos.load();
                    (src.z - tgt.z).atan2(src.x - tgt.x).to_degrees() as f32
                        - self.entity.yaw.load()
                });
                let hurt_event = SActorEvent {
                    entity_runtime_id: VarLong(entity_id as i64),
                    event_type: ActorEventType::Hurt,
                    event_data: VarInt(0),
                    fire_at_position: None,
                };
                world
                    .broadcast_editioned(
                        &CHurtAnimation::new(VarInt(entity_id), hurt_yaw),
                        &hurt_event,
                    )
                    .await;
            }

            world.broadcast_packet_all(&CDamageEvent::new(
                self.entity.entity_id.into(),
                damage_type.id.into(),
                source.map(|e| e.get_entity().entity_id.into()),
                cause.map(|e| e.get_entity().entity_id.into()),
                position,
            ));

            // Try to spawn infested silverfish
            self.try_spawn_infested_silverfish().await;

            if play_sound {
                world.play_sound(
                    self.hurt_sound(),
                    SoundCategory::Players,
                    &self.entity.pos.load(),
                );

                if let Some(source) = source {
                    let source_pos = source.get_entity().pos.load();
                    let target_pos = self.entity.pos.load();
                    let dx = source_pos.x - target_pos.x;
                    let dz = source_pos.z - target_pos.z;
                    let resistance = self.get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE);
                    self.entity.apply_knockback(
                        knockback_after_resistance(0.4, resistance),
                        dx,
                        dz,
                    );
                    self.entity.send_velocity();
                }
            }

            // Consume absorption first, then apply remaining damage to health
            let mut remaining = damage_amount;
            let current_abs = self.absorption.load();
            if current_abs > 0.0 {
                let absorbed = current_abs.min(remaining);
                if let Some(player) = caller.get_player() {
                    player
                        .increment_stat(
                            StatisticCategory::Custom,
                            CustomStatistic::DamageAbsorbed as i32,
                            (absorbed * 10.0) as i32,
                        )
                        .await;
                }

                if let Some(attacker_player) = cause.and_then(|c| c.get_player()) {
                    attacker_player
                        .increment_stat(
                            StatisticCategory::Custom,
                            CustomStatistic::DamageDealtAbsorbed as i32,
                            (absorbed * 10.0) as i32,
                        )
                        .await;
                }

                if current_abs >= remaining {
                    let new_abs = current_abs - remaining;
                    self.set_absorption(new_abs).await;
                    remaining = 0.0;
                } else {
                    remaining -= current_abs;
                    self.set_absorption(0.0).await;
                }

                // Track attacker for RevengeGoal (only after confirming damage)
                if let Some(attacker) = cause.or(source) {
                    self.last_attacker_id
                        .store(attacker.get_entity().entity_id, Relaxed);
                    self.last_attacked_time
                        .store(self.entity.age.load(Relaxed), Relaxed);
                }
            }

            // Apply remaining damage to health (clamped)
            let max_h = self.get_max_health();
            let new_health = self.health.load() - remaining;
            let clamped_health = new_health.max(0.0).min(max_h);
            if remaining > 0.0 {
                self.set_health(clamped_health);

                // Statistics updates
                if let Some(player) = caller.get_player() {
                    player
                        .increment_stat(
                            StatisticCategory::Custom,
                            CustomStatistic::DamageTaken as i32,
                            (remaining * 10.0) as i32,
                        )
                        .await;
                }

                if let Some(attacker_player) = cause.and_then(|c| c.get_player()) {
                    attacker_player
                        .increment_stat(
                            StatisticCategory::Custom,
                            CustomStatistic::DamageDealt as i32,
                            (remaining * 10.0) as i32,
                        )
                        .await;
                }

                // Track attacker for RevengeGoal (only after confirming damage)
                if let Some(attacker) = cause.or(source) {
                    self.last_attacker_id
                        .store(attacker.get_entity().entity_id, Relaxed);
                    self.last_attacked_time
                        .store(self.entity.age.load(Relaxed), Relaxed);
                }

                // Thorns (`thorns.json`, `minecraft:post_attack`): each worn armor piece
                // with Thorns rolls independently, chance = 0.15 * level. On success it
                // damages the attacker (DamageType::THORNS, uniform 1.0-5.0) and wears
                // 2 durability off that armor piece.
                if let Some(attacker) = cause {
                    let equipment_lock = self.entity_equipment.lock().await;
                    for slot in [
                        EquipmentSlot::HEAD,
                        EquipmentSlot::CHEST,
                        EquipmentSlot::LEGS,
                        EquipmentSlot::FEET,
                    ] {
                        let stack_arc = equipment_lock.get(&slot);
                        let mut stack = stack_arc.lock().await;
                        if stack.is_empty() {
                            continue;
                        }
                        let level = stack.get_enchantment_level(&Enchantment::THORNS);
                        if level <= 0 {
                            continue;
                        }
                        if rand::random::<f32>() >= 0.15 * level as f32 {
                            continue;
                        }
                        let thorns_damage = 1.0 + rand::random::<f32>() * 4.0;
                        if stack.damage_item(2) == DamageResult::Broken {
                            if let Some(player) = self.get_player() {
                                player
                                    .increment_stat(
                                        StatisticCategory::Broken,
                                        stack.item.id as i32,
                                        1,
                                    )
                                    .await;
                            }
                            world.send_entity_status(
                                &self.entity,
                                crate::entity::equipment_break_status(&slot),
                            );
                            *stack = ItemStack::EMPTY.clone();
                            let broken_stack = stack.clone();
                            drop(stack);
                            drop(stack_arc);
                            self.send_equipment_changes(&[(slot, broken_stack)]);
                        } else {
                            drop(stack);
                            drop(stack_arc);
                        }
                        attacker
                            .damage(attacker, thorns_damage, DamageType::THORNS)
                            .await;
                    }
                }
            }

            // Check if the entity died and isn't protected by a death protection mechanic (ex. totem of undying)
            if clamped_health <= 0.0
                && (bypasses_cooldown_protection || !self.try_use_death_protector(caller).await)
            {
                self.on_death(damage_type, source, cause).await;
            }

            // Armor durability wear uses the pre-armor-absorb increment, matching vanilla's
            // `hurtArmor(damageSource, damage)` call inside `getDamageAfterArmorAbsorb`
            // (LivingEntity.java:1903), entered from `actuallyHurt` with the same `dmg` that
            // `getDamageAfterAbsorb` reduces at line 1904 -- i.e. `raw_increment`, not the
            // post-reduction `damage_amount`.
            if raw_increment > 0.0 && !bypasses_armor_durability(&damage_type) {
                self.damage_armor_items(caller, raw_increment, &damage_type)
                    .await;
            }

            true
        })
    }

    fn tick_in_void<'a>(&'a self, dyn_self: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            dyn_self
                .damage(dyn_self, 4.0, DamageType::OUT_OF_WORLD)
                .await;
        })
    }

    fn get_gravity(&self) -> f64 {
        self.get_attribute_value(&Attributes::GRAVITY)
    }

    #[allow(clippy::too_many_lines)]
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.entity.tick(caller, server).await;

            // Only tick movement if the entity is alive. This prevents a dead "corpse"
            // from continuing to be simulated (accumulating fall_distance/velocity).
            // We allow movement during death animation (20 ticks) so knockback is applied.
            let is_alive = !self.dead.load(Relaxed) && self.health.load() > 0.0;
            let in_death_animation =
                self.health.load() <= 0.0 && self.death_time.load(Relaxed) < 20;
            if is_alive || (in_death_animation && self.entity.entity_type != &EntityType::PLAYER) {
                self.tick_movement(server, caller).await;
                // Vanilla-like order: freeze logic runs after movement/collisions.
                self.entity.tick_frozen(caller.as_ref()).await;
            }

            // TODO
            let player = caller.get_player();
            let is_player = player.is_some();

            if !is_player {
                self.entity.send_pos_rot();
            }

            // Fetch supporting blocks for players or other entities
            let supporting_pos = caller.get_player().map_or_else(
                || self.entity.get_supporting_block_pos(),
                super::player::Player::get_supporting_block_pos,
            );

            // Notify the block under the entity each tick if a supporting block position is found
            if let Some(supporting) = supporting_pos {
                let world = self.entity.world.load();
                let (block, state) = world.get_block_and_state(&supporting);

                world
                    .block_registry
                    .on_entity_step(
                        block,
                        &world,
                        caller.as_ref() as &dyn EntityBase,
                        &supporting,
                        state,
                        false,
                    )
                    .await;

                // Check slightly below supporting_pos for additional supporting blocks (blocks under carpets and the like)
                if !block.is_solid() {
                    let below_supporting = supporting.down();
                    let (below_block, below_state) = world.get_block_and_state(&below_supporting);

                    // If block is not air, notify it as well
                    world
                        .block_registry
                        .on_entity_step(
                            below_block,
                            &world,
                            caller.as_ref() as &dyn EntityBase,
                            &below_supporting,
                            below_state,
                            true, // below supporting block
                        )
                        .await;
                }
            }

            self.tick_effects().await;

            // Current active item
            {
                let item_in_use = self.item_in_use.lock().await.clone();
                let remaining_use_ticks = item_in_use
                    .is_some()
                    .then(|| self.item_use_time.fetch_sub(1, Ordering::Relaxed));

                if let Some(item) = item_in_use.as_ref()
                    && let Some(player) = caller.get_player()
                {
                    server
                        .item_registry
                        .on_use_tick(item, player, remaining_use_ticks.unwrap_or(0))
                        .await;
                }

                if let Some(item) = item_in_use.as_ref()
                    && remaining_use_ticks.is_some_and(|r| r <= 0)
                {
                    // Consume item
                    if let Some(food) = item.get_data_component::<FoodImpl>()
                        && let Some(player) = caller.get_player()
                    {
                        player
                            .hunger_manager
                            .eat(player, food.nutrition as u8, food.saturation)
                            .await;
                    }

                    self.apply_consumable_effects(item).await;

                    if let Some(consumable) = item.get_data_component::<ConsumableImpl>() {
                        let world = self.entity.world.load();
                        world.play_sound_event(
                            &consumable.sound_event,
                            SoundCategory::Players,
                            &self.entity.pos.load(),
                        );

                        // Consumable.onConsume, line 90: `user.gameEvent(this.animation ==
                        // ItemUseAnimation.DRINK ? GameEvent.DRINK : GameEvent.EAT)`. Fired
                        // unconditionally for any item with a Consumable component, regardless
                        // of animation, via Item.finishUsingItem (line 216).
                        let game_event = if consumable.animation == ConsumeAnimation::Drink {
                            pumpkin_data::game_event::GameEvent::Drink
                        } else {
                            pumpkin_data::game_event::GameEvent::Eat
                        };
                        crate::world::game_event::emit_game_event(
                            &world,
                            game_event,
                            self.entity.pos.load(),
                            crate::world::game_event::GameEventContext::of_entity(caller.clone()),
                        )
                        .await;
                    }

                    // Handle potion consumption
                    if item.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>().is_some() {
                        let effects = crate::item::potion::PotionContents::read_potion_effects(item);
                        crate::item::potion::PotionContents::apply_effects_to(self, effects, 1.0, crate::item::potion::PotionApplicationSource::Normal).await;
                    }

                    // SuspiciousStewEffects.onConsume: every entry is applied as a plain
                    // `MobEffectInstance(effect, duration)`, so amplifier 0 and default
                    // visibility, with no duration scaling.
                    if let Some(stew) = item
                        .get_data_component::<pumpkin_data::data_component_impl::SuspiciousStewEffectsImpl>()
                    {
                        for entry in stew.effects.iter() {
                            let Some(effect_type) = StatusEffect::from_minecraft_name(&entry.effect_id)
                            else {
                                continue;
                            };
                            self.add_effect(pumpkin_data::potion::Effect {
                                effect_type,
                                duration: entry.duration,
                                amplifier: 0,
                                ambient: false,
                                show_particles: true,
                                show_icon: true,
                                blend: false,
                            })
                            .await;
                        }
                    }

                    // OminousBottleAmplifier.onConsume (ConsumableListener, applied outside
                    // the onConsumeEffects list via `stack.getAllOfType(ConsumableListener.class)`
                    // in Consumable.onConsume): always non-ambient, hidden particles/icon shown.
                    if let Some(amplifier) = item.get_data_component::<OminousBottleAmplifierImpl>()
                        && let Ok(amplifier) = u8::try_from(amplifier.amplifier)
                    {
                        self.add_effect(Effect {
                            effect_type: &StatusEffect::BAD_OMEN,
                            duration: 120_000,
                            amplifier,
                            ambient: false,
                            show_particles: false,
                            show_icon: true,
                            blend: false,
                        })
                        .await;
                    }

                    if consumable_clears_all_effects(item) {
                        if let Some(player) = caller.get_player() {
                            // This sends one removal packet per active effect before the
                            // living entity broadcasts the removal to nearby players.
                            player.remove_all_effects().await;
                        } else {
                            let effects: Vec<_> =
                                self.active_effects.lock().await.keys().copied().collect();
                            for effect in effects {
                                self.remove_effect(effect).await;
                            }
                        }
                    }

                    if let Some(player) = caller.get_player() {
                        player
                            .trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::ConsumeItem {
                                item_id: format!("minecraft:{}", item.item.registry_key),
                            })
                            .await;

                        // Vanilla `Item#finishUsingItem` (default, non-food/non-consumable)
                        // returns the stack unchanged: no decrement happens on natural
                        // use-duration completion unless the item actually has FoodProperties
                        // or a Consumable component. Without this guard, any item with a
                        // finite get_use_duration() that a player holds to completion (e.g.
                        // SpyglassItem at 1200 ticks) gets silently consumed here.
                        if is_consumed_on_finish(item) {
                            // Prefer modifying the exact stack that matches the consumed item:
                            // 1) selected hotbar (held_item)
                            // 2) off-hand
                            // 3) fallback to active_hand if the above didn't match
                            let mut handled = false;
                            let mut slot_to_sync = None;

                            // Check main hand (hotbar selected)
                            let held_arc = player.inventory.held_item();
                            {
                                let mut held_lock = held_arc.lock().await;
                                if held_lock.are_items_and_components_equal(item) {
                                    if let Some(remainder) = consumable_remainder(item) {
                                        if player.gamemode.load() != GameMode::Creative {
                                            held_lock.decrement(1);
                                            if held_lock.is_empty() {
                                                *held_lock = ItemStack::new(1, remainder);
                                            }
                                        }
                                    } else {
                                        held_lock
                                            .decrement_unless_creative(player.gamemode.load(), 1);
                                    }
                                    handled = true;
                                    slot_to_sync =
                                        Some(player.inventory.get_selected_slot() as usize);
                                }
                            }

                            if !handled {
                                // Check off-hand
                                let off_arc = player.inventory.off_hand_item().await;
                                let mut off_lock = off_arc.lock().await;
                                if off_lock.are_items_and_components_equal(item) {
                                    if let Some(remainder) = consumable_remainder(item) {
                                        if player.gamemode.load() != GameMode::Creative {
                                            off_lock.decrement(1);
                                            if off_lock.is_empty() {
                                                *off_lock = ItemStack::new(1, remainder);
                                            }
                                        }
                                    } else {
                                        off_lock
                                            .decrement_unless_creative(player.gamemode.load(), 1);
                                    }

                                    handled = true;
                                    slot_to_sync = Some(PlayerInventory::OFF_HAND_SLOT);
                                }
                            }

                            if !handled {
                                // Use stored active_hand (as a fallback)
                                let active_hand = *self.active_hand.lock().await;
                                let hand_to_modify = active_hand.unwrap_or(Hand::Right);
                                let item_stack = self
                                    .get_stack_in_hand(caller.as_ref(), hand_to_modify)
                                    .await;
                                let mut item_lock = item_stack.lock().await;

                                if let Some(remainder) = consumable_remainder(item) {
                                    if player.gamemode.load() != GameMode::Creative {
                                        item_lock.decrement(1);
                                        if item_lock.is_empty() {
                                            *item_lock = ItemStack::new(1, remainder);
                                        }
                                    }
                                } else {
                                    item_lock.decrement_unless_creative(player.gamemode.load(), 1);
                                }
                                slot_to_sync = Some(match hand_to_modify {
                                    Hand::Left => PlayerInventory::OFF_HAND_SLOT,
                                    Hand::Right => player.inventory.get_selected_slot() as usize,
                                });
                            }

                            if let Some(slot) = slot_to_sync {
                                let updated = if slot == PlayerInventory::OFF_HAND_SLOT {
                                    player.inventory.off_hand_item().await.lock().await.clone()
                                } else {
                                    player.inventory.held_item().lock().await.clone()
                                };
                                player.sync_hand_slot(slot, updated).await;
                            }
                        }

                        if let Some(cooldown) = item.get_use_cooldown() {
                            let group = cooldown
                                .cooldown_group
                                .clone()
                                .unwrap_or_else(|| item.item.registry_key.to_string());
                            player
                                .start_cooldown(group, (cooldown.seconds * 20.0) as i32)
                                .await;
                        }
                    }

                    self.clear_active_hand().await;
                }
            }

            if self.hurt_cooldown.load(Relaxed) > 0 {
                self.hurt_cooldown.fetch_sub(1, Relaxed);
            }
            if self.health.load() <= 0.0 {
                let time = self.death_time.fetch_add(1, Relaxed);
                // Only send death particles once (on the exact tick death_time reaches 20)
                // and then remove the entity, preventing entity_event spam.
                if time == 20 && !self.entity.removed.swap(true, Ordering::Relaxed) {
                    self.entity
                        .world
                        .load()
                        .send_entity_status(&self.entity, EntityStatus::Death);
                    self.entity.remove().await;
                }
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(self)
    }

    fn is_pushable(&self) -> bool {
        self.health.load() > 0.0 && !self.dead.load(Relaxed)
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }
}

const fn equipment_slot_for_hand(hand: Hand) -> EquipmentSlot {
    match hand {
        Hand::Left => EquipmentSlot::OFF_HAND,
        Hand::Right => EquipmentSlot::MAIN_HAND,
    }
}

impl LivingEntity {
    /// Applies data-driven `apply_effects` consume effects after an item completes use.
    /// Vanilla: `Consumable.onConsume` invokes every configured effect server-side.
    async fn apply_consumable_effects(&self, item: &ItemStack) {
        let Some(consumable) = item.get_data_component::<ConsumableImpl>() else {
            return;
        };

        for consume_effect in consumable.effects.iter() {
            match consume_effect {
                ConsumeEffect::ApplyEffects((effects, probability)) => {
                    if !consume_effect_probability_applies(*probability, rand::random()) {
                        continue;
                    }

                    for effect in effects.iter() {
                        let Some(effect_type) =
                            StatusEffect::from_minecraft_name(&effect.effect_id)
                        else {
                            continue;
                        };
                        let Ok(amplifier) = u8::try_from(effect.amplifier) else {
                            continue;
                        };

                        self.add_effect(Effect {
                            effect_type,
                            duration: effect.duration,
                            amplifier,
                            ambient: effect.ambient,
                            show_particles: effect.show_particles,
                            show_icon: effect.show_icon,
                            blend: false,
                        })
                        .await;
                    }
                }
                ConsumeEffect::RemoveEffects(effects) => {
                    if let pumpkin_data::data_component_impl::IDSet::IDs(ids) = effects {
                        for effect_type in ids.iter() {
                            self.remove_effect(effect_type).await;
                        }
                    }
                }
                ConsumeEffect::PlaySound(sound) => {
                    let world = self.entity.world.load();
                    world.play_sound_event(sound, SoundCategory::Players, &self.entity.pos.load());
                }
                ConsumeEffect::TeleportRandomly(diameter) => {
                    self.teleport_randomly_on_consume(*diameter).await;
                }
                ConsumeEffect::ClearAllEffects => {}
            }
        }
    }

    /// `TeleportRandomlyConsumeEffect.apply` (26.2 decompile,
    /// world/item/consume_effects/TeleportRandomlyConsumeEffect.java:38-72): tries up to 16
    /// random offsets within `diameter` and stops at the first successful landing.
    async fn teleport_randomly_on_consume(&self, diameter: f32) {
        let pos = self.entity.pos.load();

        let (min_y, max_y) = {
            let world = self.entity.world.load();
            (
                f64::from(world.dimension.min_y),
                f64::from(world.dimension.min_y + world.dimension.logical_height - 1),
            )
        };

        for _ in 0..16 {
            let (dx, dy, dz) = {
                let mut rng = rand::rng();
                (
                    (rng.random_range(0.0..1.0) - 0.5) * f64::from(diameter),
                    (rng.random_range(0.0..1.0) - 0.5) * f64::from(diameter),
                    (rng.random_range(0.0..1.0) - 0.5) * f64::from(diameter),
                )
            };
            // Clamp to the dimension's playable Y range before randomTeleport searches for ground.
            let target_y = (pos.y + dy).clamp(min_y, max_y);

            // Clone out of the lock first: holding the guard as the `if let` scrutinee would
            // keep it alive for the whole block, across the `.await` below.
            let vehicle = self.entity.vehicle.lock().await.clone();
            if let Some(vehicle) = vehicle {
                vehicle
                    .get_entity()
                    .remove_passenger(self.entity.entity_id)
                    .await;
            }

            if self.random_teleport(pos.x + dx, target_y, pos.z + dz, true) {
                let world = self.entity.world.load();
                let is_fox = self.entity.entity_type == &EntityType::FOX;
                let (sound, category) = if is_fox {
                    (Sound::EntityFoxTeleport, SoundCategory::Neutral)
                } else {
                    (Sound::ItemChorusFruitTeleport, SoundCategory::Players)
                };
                world.play_sound(sound, category, &self.entity.pos.load());
                self.fall_distance.store(0.0);
                break;
            }
        }
    }

    /// `LivingEntity.randomTeleport` (26.2 decompile,
    /// world/entity/LivingEntity.java:3665-3709): walks down from `(x, y, z)` to the first
    /// block that blocks motion, then teleports there if the destination is free of block
    /// collision and liquid, reverting otherwise.
    fn random_teleport(&self, x: f64, y: f64, z: f64, show_particles: bool) -> bool {
        let origin = self.entity.pos.load();
        let world = self.entity.world.load();
        let dimension = self.entity.entity_dimension.load();

        let target_block = BlockPos::new(x.floor() as i32, y.floor() as i32, z.floor() as i32);
        if !world.is_loaded(&target_block) {
            return false;
        }

        let mut target_y = y;
        let mut pos_y = target_block.0.y;
        let mut landed = false;
        while !landed && pos_y > world.dimension.min_y {
            let below = BlockPos::new(target_block.0.x, pos_y - 1, target_block.0.z);
            if world.get_block_state(&below).is_solid() {
                landed = true;
            } else {
                target_y -= 1.0;
                pos_y -= 1;
            }
        }

        if !landed {
            return false;
        }

        self.entity
            .teleport(Vector3::new(x, target_y, z), None, None, world.clone());

        let bb = BoundingBox::new_from_pos(x, target_y, z, &dimension);
        let space_free = world.is_space_empty(bb);
        let liquid_free = !BlockPos::iterate(bb.min_block_pos(), bb.max_block_pos())
            .any(|pos| world.get_fluid(&pos) != &pumpkin_data::fluid::Fluid::EMPTY);

        if !space_free || !liquid_free {
            self.entity.teleport(origin, None, None, world.clone());
            return false;
        }

        if show_particles {
            world.send_entity_status(&self.entity, EntityStatus::Teleport);
        }

        true
    }
}

const fn spider_climbing_state(
    entity_type: &EntityType,
    horizontal_collision: bool,
) -> Option<bool> {
    if entity_type.id == EntityType::SPIDER.id || entity_type.id == EntityType::CAVE_SPIDER.id {
        Some(horizontal_collision)
    } else {
        None
    }
}

/// Mirrors vanilla's strict `random < probability` consume-effect gate.
const fn consume_effect_probability_applies(probability: f32, random: f32) -> bool {
    random < probability
}

/// Vanilla `Item#finishUsingItem`: the default implementation returns the
/// stack unchanged. Only the food/consumable override
/// (`ItemUtils#finishUsingItem`) decrements the stack on natural use-duration
/// completion.
fn is_consumed_on_finish(item: &ItemStack) -> bool {
    item.get_data_component::<FoodImpl>().is_some()
        || item.get_data_component::<ConsumableImpl>().is_some()
}

/// Returns whether this consumable has vanilla's `clear_all_effects` consume effect.
fn consumable_clears_all_effects(item: &ItemStack) -> bool {
    item.get_data_component::<ConsumableImpl>()
        .is_some_and(|consumable| {
            consumable
                .effects
                .iter()
                .any(|effect| matches!(effect, ConsumeEffect::ClearAllEffects))
        })
}

/// Returns the item that replaces a consumed single-item container.
///
/// `use_remainder` is a unit data component in the vanilla item data, so the
/// concrete remainder remains an item-specific rule. Keep this mapping here at
/// the shared consumable completion point so main hand, off-hand, and packet
/// inventory synchronization all take the same path.
fn consumable_remainder(item: &ItemStack) -> Option<&'static Item> {
    item.get_data_component::<UseRemainderImpl>()?;

    match item.item.id {
        id if id == Item::MILK_BUCKET.id => Some(&Item::BUCKET),
        id if id == Item::POTION.id => Some(&Item::GLASS_BOTTLE),
        _ => None,
    }
}

const fn should_replace_effect(current: &Effect, candidate: &Effect) -> bool {
    // Vanilla MobEffectInstance#isShorterDurationThan / #update: -1 is infinite duration, always
    // considered longer than any finite duration. A raw `duration > duration` comparison treats
    // -1 as shorter than everything, so an equal-amplifier infinite effect would never replace a
    // finite one.
    const fn is_longer(candidate_duration: i32, current_duration: i32) -> bool {
        if current_duration == -1 {
            false
        } else if candidate_duration == -1 {
            true
        } else {
            candidate_duration > current_duration
        }
    }

    candidate.amplifier > current.amplifier
        || (candidate.amplifier == current.amplifier
            && is_longer(candidate.duration, current.duration))
}

#[cfg(test)]
mod milk_bucket_tests {
    use super::{
        consumable_clears_all_effects, consumable_remainder, consume_effect_probability_applies,
        is_consumed_on_finish,
    };
    use pumpkin_data::{item::Item, item_stack::ItemStack};

    #[test]
    fn milk_bucket_clears_all_effects_when_consumed() {
        let milk = ItemStack::new(1, &Item::MILK_BUCKET);

        assert!(consumable_clears_all_effects(&milk));
    }

    #[test]
    fn milk_bucket_returns_an_empty_bucket() {
        let milk = ItemStack::new(1, &Item::MILK_BUCKET);

        assert_eq!(
            consumable_remainder(&milk).map(|item| item.id),
            Some(Item::BUCKET.id)
        );
    }

    #[test]
    fn consumable_effect_probability_matches_vanilla_strict_threshold() {
        assert!(!consume_effect_probability_applies(0.0, 0.0));
        assert!(consume_effect_probability_applies(1.0, 0.999));
        assert!(consume_effect_probability_applies(0.5, 0.499));
        assert!(!consume_effect_probability_applies(0.5, 0.5));
    }

    #[test]
    fn non_consumable_items_are_not_consumed_on_finish() {
        let spyglass = ItemStack::new(1, &Item::SPYGLASS);
        assert!(!is_consumed_on_finish(&spyglass));
    }

    #[test]
    fn food_and_consumable_items_are_consumed_on_finish() {
        let milk = ItemStack::new(1, &Item::MILK_BUCKET);
        let apple = ItemStack::new(1, &Item::APPLE);
        assert!(is_consumed_on_finish(&milk));
        assert!(is_consumed_on_finish(&apple));
    }
}

/// Returns `true` if `damage_type` is in `#minecraft:bypasses_armor` (1.21.11).
/// These sources bypass armor entirely (fall, drown, freeze, etc.).
pub(crate) const fn bypasses_armor_durability(damage_type: &DamageType) -> bool {
    // Bitmask lookup: O(1) with two instructions (shift + AND), no array scan.
    // DamageType IDs can exceed 31; use u64 for sufficient range.
    // TODO: Make data-driven once the data pack system can handle it without performance regressions.
    // Compile-time assertions: ensure all bypassing types fit in u64 bitmask.
    const _: () = assert!(
        DamageType::FALL.id < 64
            && DamageType::FLY_INTO_WALL.id < 64
            && DamageType::ON_FIRE.id < 64
            && DamageType::IN_WALL.id < 64
            && DamageType::CRAMMING.id < 64
            && DamageType::DROWN.id < 64
            && DamageType::GENERIC.id < 64
            && DamageType::WITHER.id < 64
            && DamageType::DRAGON_BREATH.id < 64
            && DamageType::STARVE.id < 64
            && DamageType::ENDER_PEARL.id < 64
            && DamageType::FREEZE.id < 64
            && DamageType::STALAGMITE.id < 64
            && DamageType::MAGIC.id < 64
            && DamageType::INDIRECT_MAGIC.id < 64
            && DamageType::OUT_OF_WORLD.id < 64
            && DamageType::GENERIC_KILL.id < 64
            && DamageType::SONIC_BOOM.id < 64
            && DamageType::OUTSIDE_BORDER.id < 64,
        "One or more bypass DamageType IDs exceed u64 bitmask width (>= 64)"
    );
    const BYPASS_MASK: u64 = (1u64 << DamageType::FALL.id)
        | (1u64 << DamageType::FLY_INTO_WALL.id)
        | (1u64 << DamageType::ON_FIRE.id)
        | (1u64 << DamageType::IN_WALL.id)
        | (1u64 << DamageType::CRAMMING.id)
        | (1u64 << DamageType::DROWN.id)
        | (1u64 << DamageType::GENERIC.id)
        | (1u64 << DamageType::WITHER.id)
        | (1u64 << DamageType::DRAGON_BREATH.id)
        | (1u64 << DamageType::STARVE.id)
        | (1u64 << DamageType::ENDER_PEARL.id)
        | (1u64 << DamageType::FREEZE.id)
        | (1u64 << DamageType::STALAGMITE.id)
        | (1u64 << DamageType::MAGIC.id)
        | (1u64 << DamageType::INDIRECT_MAGIC.id)
        | (1u64 << DamageType::OUT_OF_WORLD.id)
        | (1u64 << DamageType::GENERIC_KILL.id)
        | (1u64 << DamageType::SONIC_BOOM.id)
        | (1u64 << DamageType::OUTSIDE_BORDER.id);
    (damage_type.id < 64) && ((BYPASS_MASK >> damage_type.id) & 1 == 1)
}

/// Returns whether the damage source bypasses protection enchantments.
/// The 26.2 tag contains only sonic boom.
pub(crate) const fn bypasses_enchantments(damage_type: &DamageType) -> bool {
    damage_type.id == DamageType::SONIC_BOOM.id
}

/// Returns whether vanilla routes this damage through helmet protection.
pub(crate) const fn damages_helmet(damage_type: &DamageType) -> bool {
    damage_type.id == DamageType::FALLING_ANVIL.id
        || damage_type.id == DamageType::FALLING_BLOCK.id
        || damage_type.id == DamageType::FALLING_STALACTITE.id
}

/// Returns whether vanilla damage tags prevent shield blocking.
pub(crate) const fn bypasses_shield(damage_type: &DamageType) -> bool {
    matches!(
        damage_type.id,
        id if id == DamageType::ON_FIRE.id
            || id == DamageType::IN_WALL.id
            || id == DamageType::CRAMMING.id
            || id == DamageType::DROWN.id
            || id == DamageType::FLY_INTO_WALL.id
            || id == DamageType::GENERIC.id
            || id == DamageType::WITHER.id
            || id == DamageType::DRAGON_BREATH.id
            || id == DamageType::STARVE.id
            || id == DamageType::FALL.id
            || id == DamageType::ENDER_PEARL.id
            || id == DamageType::FREEZE.id
            || id == DamageType::STALAGMITE.id
            || id == DamageType::MAGIC.id
            || id == DamageType::INDIRECT_MAGIC.id
            || id == DamageType::OUT_OF_WORLD.id
            || id == DamageType::GENERIC_KILL.id
            || id == DamageType::SONIC_BOOM.id
            || id == DamageType::OUTSIDE_BORDER.id
            || id == DamageType::CACTUS.id
            || id == DamageType::CAMPFIRE.id
            || id == DamageType::DRY_OUT.id
            || id == DamageType::FALLING_ANVIL.id
            || id == DamageType::FALLING_STALACTITE.id
            || id == DamageType::HOT_FLOOR.id
            || id == DamageType::SULFUR_CUBE_HOT.id
            || id == DamageType::IN_FIRE.id
            || id == DamageType::LAVA.id
            || id == DamageType::LIGHTNING_BOLT.id
            || id == DamageType::SWEET_BERRY_BUSH.id
    )
}

/// `LivingEntity.getFrictionInfluencedSpeed`: the grounded per-tick factor
/// `moveRelative` is called with.
fn friction_influenced_speed(speed: f64, slipperiness: f64) -> f64 {
    speed * 0.216_000_02 / (slipperiness * slipperiness * slipperiness)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Terminal horizontal speed, in blocks per second, of `velocity += input * factor`
    /// followed by `velocity *= slipperiness * 0.91` as `travel_in_air` applies it on a
    /// normal block (slipperiness 0.6). `tick_movement` decays the stored input by 0.98
    /// each tick before travelling, so a continuously re-applied input settles at 0.98.
    fn terminal_blocks_per_second(input: f64, speed: f64) -> f64 {
        let per_tick = 0.98 * input * friction_influenced_speed(speed, 0.6);
        per_tick / (1.0 - 0.6 * 0.91) * 20.0
    }

    /// A player's forward input is 1.0 and its `speed` is the raw `MOVEMENT_SPEED`
    /// attribute (0.1), which reproduces the documented 4.317 blocks/s walk speed. This
    /// pins the consumer side, so the mob case below can only be fixed on the producer side.
    #[test]
    fn player_walk_speed_matches_the_documented_value() {
        let walk = terminal_blocks_per_second(1.0, 0.1);
        assert!((walk - 4.317).abs() < 0.01, "player {walk} blocks/s");
    }

    /// `Mob.setSpeed` stores `speedModifier * MOVEMENT_SPEED` and mirrors it into `zza`,
    /// so the attribute enters the per-tick velocity twice for mobs. Feeding the bare
    /// speed modifier into the input instead (the bug) made a chasing zombie 10.1 and a
    /// spider 12.9 blocks/s, both faster than a walking player.
    #[test]
    fn mob_input_and_speed_both_carry_the_movement_speed_attribute() {
        for (attribute, expected) in [(0.23, 2.284), (0.3, 3.886)] {
            let speed = 1.0 * attribute; // LivingEntity::speed_for_modifier
            let got = terminal_blocks_per_second(speed, speed);
            assert!((got - expected).abs() < 0.01, "{attribute} -> {got}");
            assert!(got < 4.317);
            // The old behaviour: input was the raw modifier.
            assert!(terminal_blocks_per_second(1.0, speed) > 4.317);
        }
    }

    #[test]
    fn active_hand_maps_to_the_matching_equipment_slot() {
        assert!(equipment_slot_for_hand(Hand::Left) == EquipmentSlot::OFF_HAND);
        assert!(equipment_slot_for_hand(Hand::Right) == EquipmentSlot::MAIN_HAND);
    }

    #[test]
    fn damage_resistant_armor_matches_damage_tags() {
        let netherite_boots = ItemStack::new(1, &Item::NETHERITE_BOOTS);
        assert!(armor_resists_damage(&netherite_boots, &DamageType::ON_FIRE));
        assert!(!armor_resists_damage(
            &netherite_boots,
            &DamageType::PLAYER_ATTACK
        ));
    }

    #[test]
    fn knockback_resistance_scales_damage_knockback() {
        assert_eq!(knockback_strength_with_resistance(0.4, 0.0), 0.4);
        assert!((knockback_strength_with_resistance(0.4, 0.25) - 0.3).abs() < f64::EPSILON);
        assert_eq!(knockback_strength_with_resistance(0.4, 1.0), 0.0);
    }

    #[test]
    fn spiders_climb_only_during_horizontal_collisions() {
        assert_eq!(spider_climbing_state(&EntityType::SPIDER, true), Some(true));
        assert_eq!(
            spider_climbing_state(&EntityType::SPIDER, false),
            Some(false)
        );
        assert_eq!(
            spider_climbing_state(&EntityType::CAVE_SPIDER, true),
            Some(true)
        );
        assert_eq!(spider_climbing_state(&EntityType::ZOMBIE, true), None);
        assert_eq!(TrackedData::SPIDER_FLAGS.v26_1, 16);
        assert_eq!(TrackedData::SPIDER_FLAGS.v26_2, 16);
    }

    fn effect(amplifier: u8, duration: i32) -> Effect {
        Effect {
            effect_type: &StatusEffect::SPEED,
            duration,
            amplifier,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }
    }

    #[test]
    fn stronger_effect_replaces_weaker_effect_even_when_shorter() {
        assert!(should_replace_effect(&effect(0, 1_200), &effect(1, 100)));
    }

    #[test]
    fn instant_effects_invert_for_tagged_entities() {
        assert!(EntityType::ZOMBIE.has_tag(&tag::EntityType::MINECRAFT_INVERTED_HEALING_AND_HARM));
        assert!(EntityType::WITHER.has_tag(&tag::EntityType::MINECRAFT_INVERTED_HEALING_AND_HARM));
        assert!(!EntityType::PLAYER.has_tag(&tag::EntityType::MINECRAFT_INVERTED_HEALING_AND_HARM));
        assert!(!LivingEntity::instant_effect_is_damage(
            &StatusEffect::INSTANT_HEALTH,
            false
        ));
        assert!(LivingEntity::instant_effect_is_damage(
            &StatusEffect::INSTANT_HEALTH,
            true
        ));
        assert!(LivingEntity::instant_effect_is_damage(
            &StatusEffect::INSTANT_DAMAGE,
            false
        ));
        assert!(!LivingEntity::instant_effect_is_damage(
            &StatusEffect::INSTANT_DAMAGE,
            true
        ));
    }

    #[test]
    fn longer_effect_replaces_effect_with_same_amplifier() {
        assert!(should_replace_effect(&effect(1, 100), &effect(1, 101)));
    }

    #[test]
    fn weaker_or_shorter_effect_does_not_replace_active_effect() {
        assert!(!should_replace_effect(&effect(1, 100), &effect(0, 1_200)));
        assert!(!should_replace_effect(&effect(1, 100), &effect(1, 99)));
        assert!(!should_replace_effect(&effect(1, 100), &effect(1, 100)));
    }

    // ── bypasses_armor_durability ─────────────────────────────────────

    /// Every member of `minecraft:bypasses_armor` (1.21.11) must return `true`.
    #[test]
    fn bypasses_armor_durability_returns_true_for_tag_members() {
        // Exact contents of the minecraft:bypasses_armor tag in 1.21.11.
        let bypassing: &[DamageType] = &[
            DamageType::ON_FIRE,
            DamageType::IN_WALL,
            DamageType::CRAMMING,
            DamageType::DROWN,
            DamageType::FLY_INTO_WALL,
            DamageType::GENERIC,
            DamageType::WITHER,
            DamageType::DRAGON_BREATH,
            DamageType::STARVE,
            DamageType::FALL,
            DamageType::ENDER_PEARL,
            DamageType::FREEZE,
            DamageType::STALAGMITE,
            DamageType::MAGIC,
            DamageType::INDIRECT_MAGIC,
            DamageType::OUT_OF_WORLD,
            DamageType::GENERIC_KILL,
            DamageType::SONIC_BOOM,
            DamageType::OUTSIDE_BORDER,
        ];
        for dt in bypassing {
            assert!(
                bypasses_armor_durability(dt),
                "{} should bypass armor durability",
                dt.message_id
            );
        }
    }

    /// Physical/combat damage types must NOT bypass armor durability.
    #[test]
    fn bypasses_armor_durability_returns_false_for_physical_sources() {
        let physical: &[DamageType] = &[
            DamageType::MOB_ATTACK,
            DamageType::PLAYER_ATTACK,
            DamageType::ARROW,
            DamageType::CACTUS,
            DamageType::SWEET_BERRY_BUSH,
            DamageType::LAVA,
            DamageType::EXPLOSION,
            DamageType::PLAYER_EXPLOSION,
            DamageType::LIGHTNING_BOLT,
            DamageType::FIREBALL,
            DamageType::THORNS,
            DamageType::TRIDENT,
        ];
        for dt in physical {
            assert!(
                !bypasses_armor_durability(dt),
                "{} should NOT bypass armor durability",
                dt.message_id
            );
        }
    }

    #[test]
    fn hurt_sound_for_entity_uses_zombie_family_sounds() {
        let cases = [
            (&EntityType::ZOMBIE, Sound::EntityZombieHurt),
            (&EntityType::DROWNED, Sound::EntityDrownedHurt),
            (&EntityType::HUSK, Sound::EntityHuskHurt),
            (
                &EntityType::ZOMBIE_VILLAGER,
                Sound::EntityZombieVillagerHurt,
            ),
        ];

        for (entity_type, expected) in cases {
            assert_eq!(LivingEntity::hurt_sound_for_entity(entity_type), expected);
        }
    }

    #[test]
    fn hurt_sound_for_entity_uses_enderman_hurt_sound() {
        assert_eq!(
            LivingEntity::hurt_sound_for_entity(&EntityType::ENDERMAN),
            Sound::EntityEndermanHurt
        );
    }

    #[test]
    fn hurt_sound_for_entity_uses_skeleton_family_sounds() {
        let cases = [
            (&EntityType::SKELETON, Sound::EntitySkeletonHurt),
            (&EntityType::BOGGED, Sound::EntityBoggedHurt),
            (&EntityType::PARCHED, Sound::EntityParchedHurt),
            (
                &EntityType::WITHER_SKELETON,
                Sound::EntityWitherSkeletonHurt,
            ),
            (&EntityType::STRAY, Sound::EntityStrayHurt),
        ];

        for (entity_type, expected) in cases {
            assert_eq!(LivingEntity::hurt_sound_for_entity(entity_type), expected);
        }
    }

    #[test]
    fn hurt_sound_for_entity_defaults_to_generic_hurt() {
        assert_eq!(
            LivingEntity::hurt_sound_for_entity(&EntityType::CREEPER),
            Sound::EntityGenericHurt
        );
    }

    #[test]
    fn sonic_boom_bypasses_armor_enchantments() {
        assert!(bypasses_enchantments(&DamageType::SONIC_BOOM));
        assert!(!bypasses_enchantments(&DamageType::MAGIC));
    }

    #[test]
    fn falling_damage_sources_use_helmet_protection() {
        assert!(damages_helmet(&DamageType::FALLING_ANVIL));
        assert!(damages_helmet(&DamageType::FALLING_BLOCK));
        assert!(damages_helmet(&DamageType::FALLING_STALACTITE));
        assert!(!damages_helmet(&DamageType::FALL));
    }

    #[test]
    fn shield_bypass_sources_cannot_be_blocked() {
        assert!(bypasses_shield(&DamageType::SONIC_BOOM));
        assert!(bypasses_shield(&DamageType::CAMPFIRE));
        assert!(!bypasses_shield(&DamageType::MOB_ATTACK));
    }
}
