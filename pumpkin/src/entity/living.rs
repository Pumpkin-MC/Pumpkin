use pumpkin_data::potion::Effect;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU8, Ordering::Relaxed};
use std::{collections::HashMap, sync::atomic::AtomicI32};

use super::EntityBase;
use super::{Entity, NBTStorage};
use crate::server::Server;
use crate::world::loot::{LootContextParameters, LootTableExt};
use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use pumpkin_config::advanced_config;
use pumpkin_data::Block;
use pumpkin_data::damage::DeathMessageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::{EntityPose, EntityStatus, EntityType};
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::{damage::DamageType, sound::Sound};
use pumpkin_inventory::entity_equipment::EntityEquipment;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{CHurtAnimation, CTakeItemEntity};
use pumpkin_protocol::{
    codec::item_stack_seralizer::ItemStackSerializer,
    java::client::play::{CDamageEvent, CSetEquipment, MetaDataType, Metadata},
};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

/// Represents a living entity within the game world.
///
/// This struct encapsulates the core properties and behaviors of living entities, including players, mobs, and other creatures.
pub struct LivingEntity {
    /// The underlying entity object, providing basic entity information and functionality.
    pub entity: Entity,
    /// The last known position of the entity.
    pub last_pos: AtomicCell<Vector3<f64>>,
    /// Tracks the remaining time until the entity can regenerate health.
    pub hurt_cooldown: AtomicI32,
    /// Stores the amount of damage the entity last received.
    pub last_damage_taken: AtomicCell<f32>,
    /// The current health level of the entity.
    pub health: AtomicCell<f32>,
    pub death_time: AtomicU8,
    /// Indicates whether the entity is dead. (`on_death` called)
    pub dead: AtomicBool,
    /// The distance the entity has been falling.
    pub fall_distance: AtomicCell<f32>,
    pub active_effects: Mutex<HashMap<&'static StatusEffect, Effect>>,
    pub entity_equipment: Arc<Mutex<EntityEquipment>>,
}

#[async_trait]
pub trait LivingEntityTrait: EntityBase {
    async fn on_actually_hurt(&self, _amount: f32, _damage_type: DamageType) {
        //TODO: wolves, etc...
    }
}

impl LivingEntity {
    pub fn new(entity: Entity) -> Self {
        let pos = entity.pos.load();
        Self {
            entity,
            last_pos: AtomicCell::new(pos),
            hurt_cooldown: AtomicI32::new(0),
            last_damage_taken: AtomicCell::new(0.0),
            health: AtomicCell::new(20.0),
            fall_distance: AtomicCell::new(0.0),
            death_time: AtomicU8::new(0),
            dead: AtomicBool::new(false),
            active_effects: Mutex::new(HashMap::new()),
            entity_equipment: Arc::new(Mutex::new(EntityEquipment::new())),
        }
    }

    pub async fn send_equipment_changes(&self, equipment: &[(EquipmentSlot, ItemStack)]) {
        let equipment: Vec<(i8, ItemStackSerializer)> = equipment
            .iter()
            .map(|(slot, stack)| {
                (
                    slot.discriminant(),
                    ItemStackSerializer::from(stack.clone()),
                )
            })
            .collect();
        self.entity
            .world
            .read()
            .await
            .broadcast_packet_except(
                &[self.entity.entity_uuid],
                &CSetEquipment::new(self.entity_id().into(), equipment),
            )
            .await;
    }

    /// Picks up and Item entity or XP Orb
    pub async fn pickup(&self, item: &Entity, stack_amount: u32) {
        // TODO: Only nearby
        self.entity
            .world
            .read()
            .await
            .broadcast_packet_all(&CTakeItemEntity::new(
                item.entity_id.into(),
                self.entity.entity_id.into(),
                stack_amount.try_into().unwrap(),
            ))
            .await;
    }

    pub fn set_pos(&self, position: Vector3<f64>) {
        self.last_pos.store(self.entity.pos.load());
        self.entity.set_pos(position);
    }

    pub async fn heal(&self, additional_health: f32) {
        assert!(additional_health > 0.0);
        self.set_health(self.health.load() + additional_health)
            .await;
    }

    pub async fn set_health(&self, health: f32) {
        self.health.store(health.max(0.0));
        // tell everyone entities health changed
        self.entity
            .send_meta_data(&[Metadata::new(9, MetaDataType::Float, health)])
            .await;
    }

    pub const fn entity_id(&self) -> i32 {
        self.entity.entity_id
    }

    pub async fn add_effect(&self, effect: Effect) {
        let mut effects = self.active_effects.lock().await;
        effects.insert(effect.effect_type, effect);
        // TODO broadcast metadata
    }

    pub async fn remove_effect(&self, effect_type: &'static StatusEffect) {
        let mut effects = self.active_effects.lock().await;
        effects.remove(&effect_type);
        self.entity
            .world
            .read()
            .await
            .send_remove_mob_effect(&self.entity, effect_type)
            .await;
    }

    pub async fn has_effect(&self, effect: &'static StatusEffect) -> bool {
        let effects = self.active_effects.lock().await;
        effects.contains_key(&effect)
    }

    pub async fn get_effect(&self, effect: &'static StatusEffect) -> Option<Effect> {
        let effects = self.active_effects.lock().await;
        effects.get(&effect).cloned()
    }

    // Check if the entity is in water
    pub async fn is_in_water(&self) -> bool {
        let world = self.entity.world.read().await;
        let block_pos = self.entity.block_pos.load();
        world.get_block(&block_pos).await == &Block::WATER
    }

    // Check if the entity is in powder snow
    pub async fn is_in_powder_snow(&self) -> bool {
        let world = self.entity.world.read().await;
        let block_pos = self.entity.block_pos.load();
        world.get_block(&block_pos).await == &Block::POWDER_SNOW
    }

    pub async fn update_fall_distance(
        &self,
        height_difference: f64,
        ground: bool,
        dont_damage: bool,
    ) {
        if ground {
            let fall_distance = self.fall_distance.swap(0.0);
            if fall_distance <= 0.0
                || dont_damage
                || self.is_in_water().await
                || self.is_in_powder_snow().await
            {
                return;
            }

            let safe_fall_distance = 3.0;
            let mut damage = fall_distance - safe_fall_distance;
            damage = damage.ceil();

            // TODO: Play block fall sound
            if damage > 0.0 {
                let check_damage = self.damage(damage, DamageType::FALL).await; // Fall
                if check_damage {
                    self.entity
                        .play_sound(Self::get_fall_sound(fall_distance as i32))
                        .await;
                }
            }
        } else if height_difference < 0.0 {
            let new_fall_distance = if !self.is_in_water().await && !self.is_in_powder_snow().await
            {
                let distance = self.fall_distance.load();
                distance - (height_difference as f32)
            } else {
                0f32
            };

            // Reset fall distance if is in water or powder_snow
            self.fall_distance.store(new_fall_distance);
        }
    }

    fn get_fall_sound(distance: i32) -> Sound {
        if distance > 4 {
            Sound::EntityGenericBigFall
        } else {
            Sound::EntityGenericSmallFall
        }
    }

    /// Kills the Entity
    pub async fn kill(&self) {
        self.damage(f32::MAX, DamageType::GENERIC_KILL).await;
    }

    pub async fn get_death_message(
        dyn_self: &dyn EntityBase,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> TextComponent {
        match damage_type.death_message_type {
            DeathMessageType::Default => {
                if cause.is_some() && source.is_some() {
                    TextComponent::translate(
                        format!("death.attack.{}.player", damage_type.message_id),
                        [
                            dyn_self.get_display_name().await,
                            cause.unwrap().get_display_name().await,
                        ],
                    )
                } else {
                    TextComponent::translate(
                        format!("death.attack.{}", damage_type.message_id),
                        [dyn_self.get_display_name().await],
                    )
                }
            }
            DeathMessageType::FallVariants => {
                //TODO
                TextComponent::translate(
                    "death.fell.accident.generic",
                    [dyn_self.get_display_name().await],
                )
            }
            DeathMessageType::IntentionalGameDesign => TextComponent::text("[")
                .add_child(TextComponent::translate(
                    format!("death.attack.{}.message", damage_type.message_id),
                    [dyn_self.get_display_name().await],
                ))
                .add_child(TextComponent::text("]")),
        }
    }

    pub async fn on_death(
        &self,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) {
        let world = self.entity.world.read().await;
        let dyn_self = world
            .get_entity_by_id(self.entity.entity_id)
            .await
            .expect("Entity not found in world");
        if self
            .dead
            .compare_exchange(false, true, Relaxed, Relaxed)
            .is_ok()
        {
            // Plays the death sound
            world
                .send_entity_status(
                    &self.entity,
                    EntityStatus::PlayDeathSoundOrAddProjectileHitParticles,
                )
                .await;
            let params = LootContextParameters {
                killed_by_player: cause.map(|c| c.get_entity().entity_type == &EntityType::PLAYER),
                ..Default::default()
            };

            self.drop_loot(params).await;
            self.entity.pose.store(EntityPose::Dying);

            let level_info = world.level_info.read().await;
            let game_rules = &level_info.game_rules;
            if self.entity.entity_type == &EntityType::PLAYER && game_rules.show_death_messages {
                //TODO: KillCredit
                let death_message =
                    Self::get_death_message(&*dyn_self, damage_type, source, cause).await;
                if let Some(server) = world.server.upgrade() {
                    for player in server.get_all_players().await {
                        player.send_system_message(&death_message).await;
                    }
                }
            }
        }
    }

    async fn drop_loot(&self, params: LootContextParameters) {
        if let Some(loot_table) = &self.get_entity().entity_type.loot_table {
            let world = self.entity.world.read().await;
            let pos = self.entity.block_pos.load();
            for stack in loot_table.get_loot(params) {
                world.drop_stack(&pos, stack).await;
            }
        }
    }

    async fn tick_move(&self, entity: &dyn EntityBase, server: &Server) {
        let velo = self.entity.velocity.load();
        let pos = self.entity.pos.load();
        self.entity
            .pos
            .store(Vector3::new(pos.x + velo.x, pos.y + velo.y, pos.z + velo.z));
        let multiplier = f64::from(Entity::velocity_multiplier(pos));
        self.entity
            .velocity
            .store(velo.multiply(multiplier, 1.0, multiplier));
        Entity::check_block_collision(entity, server).await;
    }

    async fn tick_effects(&self) {
        let mut effects_to_remove = Vec::new();

        {
            let mut effects = self.active_effects.lock().await;
            for effect in effects.values_mut() {
                if effect.duration == 0 {
                    effects_to_remove.push(effect.effect_type);
                }
                effect.duration -= 1;
            }
        }

        for effect_type in effects_to_remove {
            self.remove_effect(effect_type).await;
        }
    }

    pub fn is_part_of_game(&self) -> bool {
        self.is_spectator() && self.entity.is_alive()
    }

    pub async fn reset_state(&self) {
        self.entity.reset_state().await;
        self.hurt_cooldown.store(0, Relaxed);
        self.last_damage_taken.store(0f32);
        self.entity.portal_cooldown.store(0, Relaxed);
        *self.entity.portal_manager.lock().await = None;
        self.fall_distance.store(0f32);
        self.dead.store(false, Relaxed);
    }
}

impl LivingEntityTrait for LivingEntity {}

#[async_trait]
impl EntityBase for LivingEntity {
    async fn damage_with_context(
        &self,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        // Check invulnerability before applying damage
        if self.entity.is_invulnerable_to(&damage_type) {
            return false;
        }

        if self.health.load() <= 0.0 || self.dead.load(Relaxed) {
            return false; // Dying or dead
        }

        if amount < 0.0 {
            return false;
        }

        if (damage_type == DamageType::IN_FIRE || damage_type == DamageType::ON_FIRE)
            && self.has_effect(&StatusEffect::FIRE_RESISTANCE).await
        {
            return false; // Fire resistance
        }

        let world = self.entity.world.read().await;

        let last_damage = self.last_damage_taken.load();
        let play_sound;
        let mut damage_amount = if self.hurt_cooldown.load(Relaxed) > 10 {
            if amount <= last_damage {
                return false;
            }
            play_sound = false;
            amount - self.last_damage_taken.load()
        } else {
            self.hurt_cooldown.store(20, Relaxed);
            play_sound = true;
            amount
        };
        self.last_damage_taken.store(amount);
        damage_amount = damage_amount.max(0.0);

        let config = &advanced_config().pvp;

        if config.hurt_animation {
            let entity_id = VarInt(self.entity.entity_id);
            world
                .broadcast_packet_all(&CHurtAnimation::new(entity_id, self.entity.yaw.load()))
                .await;
        }

        self.entity
            .world
            .read()
            .await
            .broadcast_packet_all(&CDamageEvent::new(
                self.entity.entity_id.into(),
                damage_type.id.into(),
                source.map(|e| e.get_entity().entity_id.into()),
                cause.map(|e| e.get_entity().entity_id.into()),
                position,
            ))
            .await;

        if play_sound {
            self.entity
                .world
                .read()
                .await
                .play_sound(
                    // Sound::EntityPlayerHurt,
                    Sound::EntityGenericHurt,
                    SoundCategory::Players,
                    &self.entity.pos.load(),
                )
                .await;
            // todo: calculate knockback
        }

        let new_health = self.health.load() - damage_amount;
        if damage_amount > 0.0 {
            self.on_actually_hurt(damage_amount, damage_type).await;
            self.set_health(new_health).await;
        }

        if new_health <= 0.0 {
            self.on_death(damage_type, source, cause).await;
        }

        true
    }

    async fn tick(&self, caller: Arc<dyn EntityBase>, server: &Server) {
        self.entity.tick(caller.clone(), server).await;
        self.tick_move(caller.as_ref(), server).await;
        self.tick_effects().await;
        if self.hurt_cooldown.load(Relaxed) > 0 {
            self.hurt_cooldown.fetch_sub(1, Relaxed);
        }
        if self.health.load() <= 0.0 {
            let time = self.death_time.fetch_add(1, Relaxed);
            if time == 20 {
                // Spawn Death particles
                self.entity
                    .world
                    .read()
                    .await
                    .send_entity_status(&self.entity, EntityStatus::AddDeathParticles)
                    .await;
                self.entity.remove().await;
            }
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(self)
    }

    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        self.entity.write_nbt(nbt).await;
        nbt.put("Health", NbtTag::Float(self.health.load()));
        nbt.put("fall_distance", NbtTag::Float(self.fall_distance.load()));
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
        //TODO: write equipment
        // todo more...
    }

    async fn read_nbt(&self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        self.entity.read_nbt(nbt).await;
        self.health.store(nbt.get_float("Health").unwrap_or(0.0));
        self.fall_distance
            .store(nbt.get_float("fall_distance").unwrap_or(0.0));
        {
            let mut active_effects = self.active_effects.lock().await;
            let nbt_effects = nbt.get_list("active_effects");
            if let Some(nbt_effects) = nbt_effects {
                for effect in nbt_effects {
                    if let NbtTag::Compound(effect_nbt) = effect {
                        let effect = Effect::create_from_nbt(&mut effect_nbt.clone()).await;
                        if effect.is_none() {
                            log::warn!("Unable to read effect from nbt");
                            continue;
                        }
                        let mut effect = effect.unwrap();
                        effect.blend = true; // TODO: change, is taken from effect give command
                        active_effects.insert(effect.effect_type, effect);
                    }
                }
            }
        }
        // todo more...
    }
}
