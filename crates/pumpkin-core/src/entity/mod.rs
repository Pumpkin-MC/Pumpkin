use crate::block::blocks::bubble_column;
use crate::{
    block::stop_vertical_movement_after_fall,
    entity::{
        ai::brain::memory::codec::{block_pos_from_tag, block_pos_to_tag},
        decoration::leash_knot::LeashKnotEntity,
        item::ItemEntity,
    },
    net::{ClientPlatform, bedrock::BedrockClient, java::JavaClient},
    server::Server,
    world::{
        World,
        portal::{NetherPortal, PortalProcessor, PortalType, SourcePortalInfo},
    },
};
use arc_swap::ArcSwap;
use bytes::BufMut;
use crossbeam::atomic::AtomicCell;
use living::LivingEntity;
use player::Player;
use projectile_deflection::ProjectileDeflection;
use pumpkin_data::BlockState;
use pumpkin_data::biome::Biome;
use pumpkin_data::block_properties::blocks_movement;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::{EntityAttachments, EntityStatus};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_data::{
    block_properties::{Facing, HorizontalFacing},
    damage::DamageType,
    entity::{EntityPose, EntityType, MovementEmission},
    game_event::GameEvent,
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::bedrock::client::{
    CAddActor, CSetActorLink, CSetActorMotion, common::ActorLink,
};
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::{
    PositionFlag,
    bedrock::client::set_actor_data::{
        CSetActorData, MetadataValue, PropertySyncData, SyncedActorDataList, entity_data_flag,
        entity_data_key,
    },
    codec::var_ulong::VarULong,
    codec::{var_int::VarInt, var_uint::VarUInt},
    java::client::play::{
        CEntityPositionSync, CEntityVelocity, CPlayerPosition, CSetEntityMetadata, CSetPassengers,
        CSpawnEntity, Metadata, MetadataSerializer,
    },
    packet::{MultiVersionJavaPacket, Packet},
    ser::NetworkReadExt,
    serial::PacketRead,
};
use pumpkin_util::math::vector3::Axis;
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    get_section_cord, pack_degrees,
    position::BlockPos,
    vector2::Vector2,
    vector3::Vector3,
    wrap_degrees,
};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::version::JavaMinecraftVersion;
use std::collections::{BTreeMap, HashSet};
use std::sync::{
    Arc, Weak,
    atomic::{
        AtomicBool, AtomicI32, AtomicU8, AtomicU32,
        Ordering::{self, Relaxed},
    },
};
use uuid::Uuid;

pub mod ageable;
pub mod ai;
pub mod area_effect_cloud;
pub mod attributes;
pub mod boss;
pub mod breath;
pub mod bucketable;
pub mod crackiness;
pub mod custom_sound;
pub mod decoration;
pub mod dismount_helper;
pub mod effect;
pub mod experience_orb;
pub mod falling;
pub mod hunger;
pub mod interaction;
pub mod item;
pub mod item_steerable;
pub mod lightning;
pub mod living;
pub mod marker;
pub mod mob;
pub mod ominous_item_spawner;
pub mod passive;
pub mod player;
pub mod projectile;
pub mod projectile_deflection;
pub mod shearable;
pub mod spawn_util;
pub mod synched_entity_data;
pub mod tnt;
pub mod r#type;
pub mod vehicle;
pub mod weak_slot;

pub use lightning::LightningBoltEntity;

pub(crate) mod combat;
pub mod predicate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DelayedLeash {
    Entity(Uuid),
    Knot(BlockPos),
}

const LEASH_RESTORE_TIMEOUT: i32 = 100;

/// The maximum number of scoreboard tags an entity can carry, matching Vanilla.
pub const MAX_SCOREBOARD_TAGS: usize = 1024;

/// Push strength per axis in `Entity.push`.
pub(crate) const PUSH_STRENGTH: f64 = 0.05f32 as f64;

const MAX_LOADED_MOTION: f64 = 10.0;
const MAX_HORIZONTAL_POSITION: f64 = 3.000_051_2E7;
const MAX_VERTICAL_POSITION: f64 = 2.0E7;

/// Fluid current speeds of `updateFluidHeightAndDoFluidPushing`.
const WATER_PUSH_SPEED: f64 = 0.014;
const FAST_LAVA_PUSH_SPEED: f64 = 0.007;
const LAVA_PUSH_SPEED: f64 = FAST_LAVA_PUSH_SPEED / 3.0;

/// Speed range of `moveTowardsClosestSpace`.
const PUSH_OUT_MIN_SPEED: f64 = 0.1;
const PUSH_OUT_RANDOM_SPEED: f64 = 0.2;
pub(crate) const MAX_PUSH_OUT_SPEED: f64 = PUSH_OUT_MIN_SPEED + PUSH_OUT_RANDOM_SPEED;

#[must_use]
pub(crate) const fn lava_push_speed(fast_lava: bool) -> f64 {
    if fast_lava {
        FAST_LAVA_PUSH_SPEED
    } else {
        LAVA_PUSH_SPEED
    }
}

/// Speed water and lava currents can add together in one tick.
#[must_use]
pub(crate) const fn max_fluid_push_speed(fast_lava: bool) -> f64 {
    WATER_PUSH_SPEED + lava_push_speed(fast_lava)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GearDamage {
    Armor,
    Helmet,
}

/// Returns the [`EntityStatus`] that should be broadcast when the given
/// equipment slot breaks.
#[must_use]
pub const fn hand_equipment_slot(hand: pumpkin_util::Hand) -> EquipmentSlot {
    match hand {
        pumpkin_util::Hand::Right => EquipmentSlot::MAIN_HAND,
        pumpkin_util::Hand::Left => EquipmentSlot::OFF_HAND,
    }
}

#[must_use]
pub const fn equipment_break_status(slot: &EquipmentSlot) -> EntityStatus {
    match slot {
        EquipmentSlot::MainHand(_) => EntityStatus::MainhandBreak,
        EquipmentSlot::OffHand(_) => EntityStatus::OffhandBreak,
        EquipmentSlot::Head(_) => EntityStatus::HeadBreak,
        EquipmentSlot::Chest(_) => EntityStatus::ChestBreak,
        EquipmentSlot::Legs(_) => EntityStatus::LegsBreak,
        EquipmentSlot::Feet(_) => EntityStatus::FeetBreak,
        EquipmentSlot::Body(_) => EntityStatus::BodyBreak,
        EquipmentSlot::Saddle(_) => EntityStatus::SaddleBreak,
    }
}

impl dyn EntityBase + '_ {
    /// Inherent on the trait object so both sides can be `&dyn EntityBase`.
    #[must_use]
    pub fn is_allied_to(&self, other: &dyn EntityBase) -> bool {
        self.get_entity().entity_id == other.get_entity().entity_id
            || self.considers_entity_as_ally(other)
            || other.considers_entity_as_ally(self)
    }

    /// Vanilla `getPassengerRidingPosition`: where this vehicle seats the passenger.
    #[must_use]
    pub fn passenger_riding_position(&self, passenger: &Entity) -> Option<Vector3<f64>> {
        let (index, rider) = self
            .get_entity()
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .enumerate()
            .find(|(_, rider)| rider.get_entity().entity_id == passenger.entity_id)
            .map(|(index, rider)| (index, rider.clone()))?;
        let seat = self
            .passenger_attachment_point(rider.as_ref(), index)
            .or_else(|| default_passenger_attachment_point(self, index))?;
        Some(self.get_entity().pos.load() + seat)
    }

    #[must_use]
    pub fn do_teams_allow_damage(&self, other: &dyn EntityBase) -> bool {
        let Some(team) = self.get_team_name() else {
            return true;
        };
        other.get_team_name().as_ref() != Some(&team)
            || self
                .get_team()
                .is_some_and(|team| team.is_allow_friendly_fire())
    }
}

/// Maps Java entity types to Bedrock runtime entity identifiers where the naming differs.
#[must_use]
pub const fn entity_type_to_bedrock_identifier(entity_type: &EntityType) -> &'static str {
    if entity_type.id == EntityType::EXPERIENCE_ORB.id {
        "minecraft:xp_orb"
    } else if entity_type.id == EntityType::FISHING_BOBBER.id {
        "minecraft:fishing_hook"
    } else if entity_type.id == EntityType::EVOKER_FANGS.id {
        "minecraft:evocation_fang"
    } else if entity_type.id == EntityType::TRIDENT.id {
        "minecraft:thrown_trident"
    } else if entity_type.id == EntityType::FIREWORK_ROCKET.id {
        "minecraft:fireworks_rocket"
    } else if entity_type.id == EntityType::EYE_OF_ENDER.id {
        "minecraft:eye_of_ender_signal"
    } else if entity_type.id == EntityType::ZOMBIE_VILLAGER.id {
        "minecraft:zombie_villager_v2"
    } else {
        entity_type.resource_name
    }
}

pub trait EntityBase: Send + Sync + std::any::Any {
    fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.get_entity().write_nbt(nbt);
        if let Some(living) = self.get_living_entity() {
            living.write_living_nbt(nbt);
        }
        if let Some(owner) = self.get_projectile_owner() {
            owner.write_nbt(nbt);
        }
        self.write_custom_nbt(nbt);
        self.get_entity().write_passengers(nbt);
    }

    fn write_custom_nbt(&self, _nbt: &mut NbtCompound) {}

    fn get_max_air_supply(&self) -> i32 {
        self.get_mob()
            .map_or(breath::MAX_AIR, mob::Mob::mob_max_air_supply)
    }

    fn increase_air_supply(&self, current: i32) -> i32 {
        self.get_mob().map_or_else(
            || {
                current
                    .wrapping_add(breath::AIR_RECOVERY_RATE)
                    .min(self.get_max_air_supply())
            },
            |mob| mob.mob_increase_air_supply(current),
        )
    }

    fn read_nbt_non_mut(&self, nbt: &NbtCompound) {
        self.get_entity()
            .read_nbt_with_max_air_supply(nbt, self.get_max_air_supply());
        if let Some(living) = self.get_living_entity() {
            living.read_living_nbt_non_mut(nbt);
        }
        if let Some(owner) = self.get_projectile_owner() {
            owner.read_nbt(nbt);
        }
        self.read_custom_nbt(nbt);
    }

    fn read_custom_nbt(&self, _nbt: &NbtCompound) {}

    /// Explicit removal of top-level keys, since a missing key on read keeps the field.
    fn remove_nbt(&self, removed: &[Box<str>]) {
        if let Some(owner) = self.get_projectile_owner() {
            owner.remove_nbt(removed);
        }
        self.remove_custom_nbt(removed);
    }

    fn remove_custom_nbt(&self, _removed: &[Box<str>]) {}
    /// Called every tick for this entity.
    ///
    /// The `caller` parameter is a reference to the entity that initiated the tick.
    /// This can be the same entity the method is being called on (`self`),
    /// but in some scenarios (e.g., interactions or events), it might be a different entity.
    ///
    /// The `server` parameter provides access to the game server instance.
    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        if let Some(living) = self.get_living_entity() {
            living.tick(caller, server);
        } else {
            self.get_entity().tick(caller, server);
        }
    }

    #[must_use]
    fn resolve_dynamic_raw_id(
        &self,
        registry: &str,
        name: &str,
        static_fallback: impl FnOnce(&str) -> Option<u32>,
    ) -> Option<u32>
    where
        Self: Sized,
    {
        self.get_entity()
            .resolve_dynamic_raw_id(registry, name, static_fallback)
    }

    #[must_use]
    fn resolve_dynamic_entry_id(
        &self,
        registry: &str,
        raw_id: u32,
        static_fallback: impl FnOnce(u32) -> String,
    ) -> String
    where
        Self: Sized,
    {
        self.get_entity()
            .resolve_dynamic_entry_id(registry, raw_id, static_fallback)
    }

    fn as_any(&self) -> &dyn std::any::Any
    where
        Self: Sized,
    {
        self
    }

    fn get_item_steerable(&self) -> Option<&dyn crate::entity::item_steerable::ItemSteerable> {
        None
    }

    /// Who shot or threw this, for projectiles.
    fn get_projectile_owner(&self) -> Option<&projectile::owner::ProjectileOwner> {
        None
    }

    /// Vanilla `Projectile.getOwner`.
    fn get_owner_entity(&self) -> Option<Arc<dyn EntityBase>> {
        self.get_projectile_owner()?
            .get(&self.get_entity().world.load())
    }

    fn get_eye_pos(&self) -> Vector3<f64> {
        self.get_entity().get_eye_pos()
    }

    fn get_looking_vector(&self) -> Vector3<f64> {
        let entity = self.get_entity();
        Vector3::from_yaw_pitch(entity.yaw.load(), entity.pitch.load())
    }

    fn get_component(
        &self,
        _component: pumpkin_data::data_component::DataComponent,
    ) -> Option<Box<dyn pumpkin_data::data_component_impl::DataComponentImpl>> {
        None
    }

    fn init_data_tracker(&self) {
        let entity = self.get_entity();

        // If the internal age is negative, it's a baby
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;

        if is_baby {
            let mut bedrock_meta = SyncedActorDataList::new();
            bedrock_meta.set_flag(entity_data_key::FLAGS, entity_data_flag::BABY as u8, true);
            entity.set_synced_data(tracked_data::ageable_mob::DATA_BABY_ID, true);
            entity.send_bedrock_actor_data(&bedrock_meta);
        }
    }
    fn set_variant_name(&self, _name: &str) {}
    fn apply_implicit_component(
        &self,
        _component: pumpkin_data::data_component::DataComponent,
        _value: &str,
    ) {
    }
    fn set_sound_variant_name(&self, _name: &str) {}

    fn teleport(
        &self,
        position: Vector3<f64>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        world: Arc<World>,
    ) {
        self.get_entity().teleport(position, yaw, pitch, &world);
    }

    fn is_pushed_by_fluids(&self) -> bool {
        true
    }

    /// Whether the entity is immune from explosion knockback and damage
    fn is_immune_to_explosion(&self) -> bool {
        false
    }

    fn get_gravity(&self) -> f64 {
        0.0
    }

    /// Air drag used by the shared landing solver for vehicles that use vanilla restitution.
    fn vehicle_bounce_air_drag(&self) -> Option<f64> {
        None
    }

    /// Extra metadata to accompany this entity when spawned for Bedrock clients (e.g. falling block variant, TNT fuse).
    fn entity_bedrock_spawn_metadata(&self) -> Option<SyncedActorDataList> {
        None
    }

    /// Bedrock network Y above the feet. Endstone `getBaseOffset`.
    fn bedrock_y_offset(&self) -> f64 {
        0.0
    }

    fn bedrock_pos(&self) -> Vector3<f64> {
        self.get_entity()
            .pos
            .load()
            .add_raw(0.0, self.bedrock_y_offset(), 0.0)
    }

    /// Feet position of a Bedrock network position
    fn feet_from_bedrock_pos(&self, bedrock_pos: Vector3<f64>) -> Vector3<f64> {
        bedrock_pos.add_raw(0.0, -self.bedrock_y_offset(), 0.0)
    }

    /// What this entity can touch next tick; `None` means a conservative reach.
    fn audited_reach(&self) -> Option<crate::world::entity_islands::AuditedReach> {
        self.get_mob()
            .and_then(|mob| mob.get_mob_entity().audited_reach(mob))
    }

    fn get_mob(&self) -> Option<&dyn mob::Mob> {
        None
    }

    fn has_container_inventory(&self) -> bool {
        false
    }

    fn get_container_inventory(&self) -> Option<Arc<dyn pumpkin_inventory::Inventory>> {
        None
    }

    /// Players are tracked by profile name, every other entity by its UUID.
    fn get_scoreboard_name(&self) -> String {
        self.get_player().map_or_else(
            || self.get_entity().entity_uuid.to_string(),
            |player| player.gameprofile.name.clone(),
        )
    }

    fn get_team(&self) -> Option<crate::world::scoreboard::Team> {
        if let Some(player) = self.get_player() {
            return player.get_team();
        }
        let world = self.get_entity().world.load();
        let scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if scoreboard.get_teams().is_empty() {
            return None;
        }
        scoreboard
            .get_entity_team(&self.get_scoreboard_name())
            .cloned()
    }

    /// The team name alone, which is all the ally checks need. Worth having because
    /// they run once per candidate of every target search, and a `Team` is expensive
    /// to clone.
    fn get_team_name(&self) -> Option<String> {
        if let Some(player) = self.get_player() {
            return player.get_team_name();
        }
        let world = self.get_entity().world.load();
        let scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if scoreboard.get_teams().is_empty() {
            return None;
        }
        scoreboard
            .get_entity_team(&self.get_scoreboard_name())
            .map(|team| team.name.clone())
    }

    fn considers_entity_as_ally(&self, other: &dyn EntityBase) -> bool {
        if let Some(considered) = self
            .get_mob()
            .and_then(|mob| mob.mob_considers_entity_as_ally(other, None))
        {
            return considered;
        }
        if let Some(tamable) = self.get_mob().and_then(mob::Mob::as_tamable)
            && let Some(considered) = tamable.tamable_considers_entity_as_ally(other)
        {
            return considered;
        }
        self.is_on_same_team(other)
    }

    fn is_on_same_team(&self, other: &dyn EntityBase) -> bool {
        let Some(team) = self.get_team_name() else {
            return false;
        };
        other.get_team_name().is_some_and(|other| other == team)
    }

    fn tick_in_void(&self, _dyn_self: &dyn EntityBase) {
        self.get_entity().remove();
    }

    /// Returns if damage was successful or not
    fn damage(&self, caller: &dyn EntityBase, amount: f32, damage_type: DamageType) -> bool {
        caller.damage_with_context(caller, amount, damage_type, None, None, None)
    }

    fn on_lightning_strike(
        &self,
        caller: &dyn EntityBase,
        lightning: &lightning::LightningBoltEntity,
    ) {
        // Vanilla `Entity.thunderHit` hits entities with 5 damage and sets them on fire for 8 seconds.
        self.set_on_fire_for(8.0);
        let cause = lightning.get_cause();
        self.damage_with_context(
            caller,
            5.0,
            DamageType::LIGHTNING_BOLT,
            None,
            Some(lightning),
            cause.as_deref().map(|p| p as &dyn EntityBase),
        );
    }

    /// Leashed by four points when the holder supports it (horses, sniffers).
    fn support_quad_leash(&self) -> bool {
        false
    }

    /// Holds a leash by four points (ghasts).
    fn support_quad_leash_as_holder(&self) -> bool {
        false
    }

    fn leash_snap_distance(&self) -> f64 {
        Entity::LEASH_SNAP_DISTANCE
    }

    fn leash_elastic_distance(&self) -> f64 {
        Entity::LEASH_ELASTIC_DISTANCE
    }

    fn is_spectator(&self) -> bool {
        false
    }

    /// Called on the old holder after an entity leashed to it was unleashed.
    fn notify_leashee_removed(&self) {}

    /// Vanilla `Entity.canInteractWithLevel`, using `LivingEntity.isAlive` for living entities.
    fn can_interact_with_level(&self) -> bool {
        self.get_living_entity()
            .map_or_else(|| self.get_entity().is_alive(), LivingEntity::is_alive)
            && !self.is_spectator()
    }

    fn can_be_seen_by_anyone(&self) -> bool {
        self.get_living_entity()
            .is_some_and(|living| !self.is_spectator() && living.is_alive())
    }

    fn is_visible_and_vulnerable(&self) -> bool {
        self.get_living_entity()
            .is_some_and(|living| !living.entity.invulnerable.load(Ordering::Relaxed))
            && self.can_be_seen_by_anyone()
    }

    fn can_be_seen_as_enemy(&self) -> bool {
        self.is_visible_and_vulnerable()
    }

    fn is_stepping_carefully(&self) -> bool {
        self.get_entity().is_sneaking()
    }

    fn dampens_vibrations(&self) -> bool {
        false
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        false
    }

    fn can_hit(&self) -> bool {
        false
    }

    /// Called while the entity is at the top of a bubble column. Projectiles get the uncapped push.
    fn on_above_bubble_column(&self, drag_down: bool, pos: &BlockPos) {
        let entity = self.get_entity();
        if self.get_projectile_owner().is_some() {
            bubble_column::push_projectile(entity, drag_down, true);
        } else {
            bubble_column::push_entity(entity, drag_down, true);
        }
        bubble_column::send_bubble_column_particles(&entity.world.load(), pos);
    }

    /// Called while the entity is inside a bubble column, below its top.
    fn on_inside_bubble_column(&self, drag_down: bool) {
        let entity = self.get_entity();
        if self.get_projectile_owner().is_some() {
            bubble_column::push_projectile(entity, drag_down, false);
        } else {
            bubble_column::push_entity(entity, drag_down, false);
        }
    }

    /// Whether a projectile bounces back off the world border instead of hitting it, like arrows and bobbers.
    fn should_bounce_on_world_border(&self) -> bool {
        false
    }

    /// Lets the exploding entity change how much a block resists its explosion.
    fn get_block_explosion_resistance(&self, _block: &Block, resistance: f32) -> f32 {
        resistance
    }

    fn can_be_hit_by_projectile(&self) -> bool {
        self.get_entity().is_alive() && self.can_hit()
    }

    /// What a projectile hitting this entity does instead of hitting it.
    fn deflection(&self, _projectile: &dyn EntityBase) -> ProjectileDeflection {
        projectile_deflection::default_deflection(self.get_entity())
    }

    /// Whether a projectile that hit this entity bounces off or stops, `was_hurt` telling if the
    /// hit did damage. Endermen let missed projectiles fly on.
    fn projectile_receives_side_effects_on_hit(&self, _was_hurt: bool) -> bool {
        true
    }

    /// Whether this projectile accepts a deflection right now.
    fn can_be_deflected(&self) -> bool {
        true
    }

    /// Runs after this projectile was deflected, `by_attack` when a player hit it.
    fn on_deflection(&self, _by_attack: bool) {}

    /// The horizontal direction a living entity hit by this projectile is pushed away from.
    /// `None` for anything that isn't a projectile.
    fn hurt_knockback_direction(&self, _hurt: &Entity) -> Option<(f64, f64)> {
        self.get_projectile_owner().map(|_| {
            let velocity = self.get_entity().velocity.load();
            (velocity.x, velocity.z)
        })
    }

    fn is_immobile(&self) -> bool {
        self.get_living_entity()
            .is_some_and(LivingEntity::is_dead_or_dying)
    }

    fn is_effective_ai(&self) -> bool {
        self.get_mob()
            .is_none_or(|mob| !mob.get_mob_entity().is_no_ai())
    }

    fn get_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        None
    }

    /// The seat of `passenger` relative to this entity, when the extracted attachment points
    /// don't describe it.
    fn passenger_attachment_point(
        &self,
        _passenger: &dyn EntityBase,
        _index: usize,
    ) -> Option<Vector3<f64>> {
        None
    }

    fn vehicle_attachment_point(&self, _vehicle: &dyn EntityBase) -> Option<Vector3<f64>> {
        None
    }

    fn on_rider_positioned(&self, _passenger: &dyn EntityBase) {}

    fn hurt_gear_slots(&self, _gear: GearDamage) -> &'static [EquipmentSlot] {
        &[]
    }

    fn can_control_vehicle(&self) -> bool {
        !self
            .get_entity()
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_NON_CONTROLLING_RIDER)
    }

    fn is_flutterer(&self) -> bool {
        false
    }

    fn set_sprinting(&self, is_sprinting: bool) {
        if let Some(living) = self.get_living_entity() {
            living.set_sprinting(is_sprinting);
        } else {
            self.get_entity().set_sprinting(is_sprinting);
        }
    }

    fn get_block_speed_factor(&self) -> f32 {
        self.get_living_entity().map_or_else(
            || self.get_entity().get_block_speed_factor(),
            LivingEntity::get_block_speed_factor,
        )
    }

    /// Vanilla `getSoundSource`.
    fn sound_source(&self) -> SoundCategory {
        self.get_entity().entity_type.sound_source
    }

    /// Vanilla `Entity.playSound`: at the entity, in its sound source, unless it is silent.
    fn play_entity_sound(&self, sound: Sound, volume: f32, pitch: f32) {
        let entity = self.get_entity();
        if !entity.is_silent() {
            entity.world.load().play_sound_fine(
                sound,
                self.sound_source(),
                &entity.pos.load(),
                volume,
                pitch,
            );
        }
    }

    /// Vanilla `getMovementEmission`.
    fn movement_emission(&self) -> MovementEmission {
        self.get_entity().entity_type.movement_emission
    }

    /// Vanilla `nextStep`: the `move_dist` at which the next step sound and event happen.
    fn next_step(&self) -> f32 {
        self.get_entity().default_next_step()
    }

    /// Vanilla `playStepSound`.
    fn play_step_sound(&self, _pos: &BlockPos, state: &BlockState) {
        self.play_block_step_sound(state);
    }

    /// The base `Entity.playStepSound`: the step sound of the block walked on.
    fn play_block_step_sound(&self, state: &BlockState) {
        let sound_type = state.get_sound_type();
        self.play_entity_sound(
            sound_type.step_sound,
            sound_type.volume * 0.15,
            sound_type.pitch,
        );
    }

    /// Vanilla `getSwimSound`.
    fn get_swim_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.swim_sound
    }

    /// Vanilla `playSwimSound`.
    fn play_swim_sound(&self, volume: f32) {
        self.play_default_swim_sound(volume);
    }

    /// The base `Entity.playSwimSound`.
    fn play_default_swim_sound(&self, volume: f32) {
        if let Some(sound) = self.get_swim_sound() {
            let pitch = (rand::random::<f32>() - rand::random::<f32>()).mul_add(0.4, 1.0);
            self.play_entity_sound(sound, volume, pitch);
        }
    }

    /// Vanilla `getSwimSplashSound`.
    fn get_swim_splash_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.swim_splash_sound
    }

    /// Vanilla `getSwimHighSpeedSplashSound`.
    fn get_swim_high_speed_splash_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.swim_high_speed_splash_sound
    }

    /// Vanilla `doWaterSplashEffect`, run when the entity enters water.
    fn do_water_splash_effect(&self, caller: &dyn EntityBase) {
        self.get_entity().water_splash_effect(caller);
    }

    /// Vanilla `LivingEntity.getFallSounds`: the small and big fall damage sounds.
    fn get_fall_sounds(&self) -> (Sound, Sound) {
        let entity_type = self.get_entity().entity_type;
        (
            entity_type
                .small_fall_sound
                .unwrap_or(Sound::EntityGenericSmallFall),
            entity_type
                .big_fall_sound
                .unwrap_or(Sound::EntityGenericBigFall),
        )
    }

    /// Vanilla `isFlapping`: whether moving through the air plays `on_flap` and emits `FLAP`.
    fn is_flapping(&self) -> bool {
        false
    }

    /// Vanilla `onFlap`.
    fn on_flap(&self) {}

    /// Vanilla `isOnRails`: rail-riding entities step even when not on the ground.
    fn is_on_rails(&self) -> bool {
        false
    }

    /// Custom Y-axis velocity drag multiplier applied during `travel_in_air`.
    /// Bats return `Some(0.6)` to match vanilla's `travel()` override.
    fn get_y_velocity_drag(&self) -> Option<f64> {
        None
    }

    fn java_spawn_metadata(&self, version: JavaMinecraftVersion) -> Option<Box<[u8]>> {
        if version < JavaMinecraftVersion::V_1_9 {
            let entity = self.get_entity();
            let shared_flags = entity.flags.load(Ordering::Relaxed);
            return (shared_flags != 0).then(|| {
                // (0 << 5) | 0 = 0 (type: byte, index: 0 flags), value, 127 (terminator)
                Box::<[u8]>::from([0x00u8, shared_flags as u8, 127u8])
            });
        }
        self.get_mob().map_or_else(
            || {
                let entity = self.get_entity();
                let shared_flags = entity.flags.load(Ordering::Relaxed);
                (shared_flags != 0).then(|| {
                    let mut buf = Vec::new();
                    let _ = Metadata::new(
                        pumpkin_data::tracked_data::entity::DATA_SHARED_FLAGS_ID,
                        shared_flags,
                    )
                    .write(&mut buf, &version);
                    buf.put_u8(255);
                    buf.into_boxed_slice()
                })
            },
            |mob| mob.mob_java_spawn_metadata(version),
        )
    }

    /// Direct spawn. Override `send_bedrock_spawn_packets` to customize tracked pairing.
    fn send_bedrock_spawn_packet(&self, client: &BedrockClient) {
        self.send_bedrock_spawn_packets(client, false, &mut |data| client.try_enqueue_packet(data));
    }

    /// Emits Bedrock spawn packets. Tracked pairing sends the current passenger links separately.
    fn send_bedrock_spawn_packets(
        &self,
        client: &BedrockClient,
        tracked: bool,
        send: &mut dyn FnMut(bytes::Bytes),
    ) {
        let entity = self.get_entity();
        let runtime_id = entity.entity_id as u64;
        let identifier = self
            .get_mob()
            .and_then(mob::Mob::mob_bedrock_identifier)
            .unwrap_or_else(|| entity_type_to_bedrock_identifier(entity.entity_type));
        let mut metadata = entity.bedrock_metadata();
        if let Some(extra_meta) = self.entity_bedrock_spawn_metadata() {
            metadata.merge(extra_meta);
        }
        if let Some(mob) = self.get_mob()
            && let Some(mob_metadata) = mob.mob_bedrock_spawn_metadata()
        {
            metadata.merge(mob_metadata);
        }
        let packet = CAddActor {
            target_actor_id: VarLong(runtime_id as i64),
            target_runtime_id: VarULong(runtime_id),
            actor_type: identifier.to_string(),
            position: self.bedrock_pos().to_f32_lossy(),
            velocity: entity.velocity.load().to_f32_lossy(),
            rotation: Vector2::new(entity.pitch.load(), entity.yaw.load()),
            y_head_rotation: entity.head_yaw.load(),
            y_body_rotation: entity.body_yaw.load(),
            attributes_list: Vec::new(),
            actor_data: metadata,
            // TODO: Bedrock `climate_variant` property for pig/cow/chicken (`TemperatureVariantAnimal`)
            // empty -> client picks its own variant.
            synced_properties: PropertySyncData {
                int_entries_list: std::collections::HashMap::new(),
                float_entries_list: std::collections::HashMap::new(),
            },
            actor_links: if tracked {
                Vec::new()
            } else {
                entity.bedrock_actor_links()
            },
            passenger_of_block: None,
        };
        if let Ok(data) = client.serialize_packet(&packet) {
            send(data);
        }
    }

    /// Direct spawn. Override `send_java_spawn_packets` to customize tracked pairing.
    fn send_java_spawn_packet(&self, client: &JavaClient) {
        self.send_java_spawn_packets(client, &mut |data| client.try_enqueue_packet(data));
    }

    /// Emits the Java spawn and follow-up packets through the caller's delivery guard.
    fn send_java_spawn_packets(&self, client: &JavaClient, send: &mut dyn FnMut(bytes::Bytes)) {
        let entity = self.get_entity();
        let metadata = self.java_spawn_metadata(CURRENT_MC_VERSION);
        let spawn_packet = entity.create_spawn_packet();
        if let Ok(data) = client.serialize_packet(&spawn_packet) {
            send(data);
        }
        if let Some(meta) = metadata {
            let meta_packet = CSetEntityMetadata::new(entity.entity_id.into(), meta);
            if let Ok(meta_data) = client.serialize_packet(&meta_packet) {
                send(meta_data);
            }
        }
        let holder = entity
            .get_leash_holder()
            .map(|holder| holder.get_entity().entity_id);
        if let Some(holder_id) = holder {
            let link_packet = pumpkin_protocol::java::client::play::CSetEntityLink::new(
                entity.entity_id,
                holder_id,
                true,
            );
            if let Ok(link_data) = client.serialize_packet(&link_packet) {
                send(link_data);
            }
        }
    }

    fn damage_with_context(
        &self,
        caller: &dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        if let Some(living) = caller.get_living_entity() {
            return living.damage_with_context(
                caller,
                amount,
                damage_type,
                position,
                source,
                cause,
            );
        }
        false
    }

    /// Called when a player right-clicks this entity with an item.
    /// Called when a player right-clicks this entity with an item.
    /// Returns true if the interaction was handled.
    fn interact(&self, _player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        false
    }

    fn interact_with_hand(
        &self,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        hand: pumpkin_util::Hand,
    ) -> Option<bool> {
        interact_with_shears(self, player, item_stack, hand)
    }

    fn set_on_fire_for(&self, seconds: f32) {
        let entity = self.get_entity();
        // Exclude fire-immune entities (ex. certain items) from burn damage
        if !entity.is_fire_immune() {
            self.set_on_fire_for_ticks((seconds * 20.0).floor() as u32);
        }
    }

    fn set_on_fire_for_ticks(&self, ticks: u32) {
        let entity = self.get_entity();
        let mut event = crate::plugin::api::events::entity::entity_combust::EntityCombustEvent::new(
            entity.entity_id,
            ticks as f32 / 20.0,
        );
        if let Some(server) = entity.world.load().server.upgrade() {
            event = server.plugin_manager.fire_blocking(&server, event);
            if event.cancelled {
                return;
            }
        }
        if entity.fire_ticks.load(Ordering::Relaxed) < ticks as i32 {
            entity.fire_ticks.store(ticks as i32, Ordering::Relaxed);
        }
        // TODO: defrost
    }

    /// Called when a player collides with an entity
    fn on_player_collision(&self, _player: &Arc<Player>) {}

    /// When false, world tick skips the player-touch scan for this entity.
    fn receives_player_touch(&self) -> bool {
        false
    }

    fn is_passenger(&self) -> bool {
        self.get_entity().has_vehicle()
    }

    fn can_ride(&self, vehicle: &dyn EntityBase) -> bool {
        let entity = self.get_entity();
        !entity.is_sneaking()
            && entity.riding_cooldown.load(Ordering::Relaxed) <= 0
            && self.get_mob().is_none_or(|mob| mob.mob_can_ride(vehicle))
    }

    fn is_vehicle(&self) -> bool {
        self.get_entity().has_passengers()
    }

    fn has_passenger(&self, other: &dyn EntityBase) -> bool {
        self.get_entity()
            .has_passenger(other.get_entity().entity_id)
    }

    /// Vanilla `Entity.getRootVehicle`: walk up the ride chain.
    fn root_vehicle_id(&self) -> i32 {
        let mut current_id = self.get_entity().entity_id;
        let mut vehicle = self.get_entity().get_vehicle();
        while let Some(ridden) = vehicle {
            current_id = ridden.get_entity().entity_id;
            vehicle = ridden.get_entity().get_vehicle();
        }
        current_id
    }

    /// Vanilla `Entity.isPassengerOfSameVehicle`.
    fn is_passenger_of_same_vehicle(&self, other: &dyn EntityBase) -> bool {
        self.root_vehicle_id() == other.root_vehicle_id()
    }

    fn move_entity(&self, caller: &dyn EntityBase, motion: Vector3<f64>) {
        self.get_entity().move_entity(caller, motion);
    }

    fn is_pushable(&self) -> bool {
        false
    }

    /// Vanilla `LivingEntity.onClimbable`.
    fn on_climbable(&self) -> bool {
        self.get_living_entity()
            .is_some_and(living::LivingEntity::on_climbable)
    }

    fn is_ignoring_block_triggers(&self) -> bool {
        self.get_mob().is_some_and(mob::Mob::ignores_block_triggers)
    }

    fn can_push_contact(&self, _other: &dyn EntityBase) -> bool {
        true
    }

    /// Extra tick reach for a vehicle that applies block contact effects.
    fn vehicle_contact_tick_reach(&self) -> Option<f64> {
        None
    }

    /// Whether vehicle tick behavior can reach beyond the island planner's bounds.
    fn vehicle_requires_serial_tick(&self) -> bool {
        false
    }

    fn client_vehicle_float_threshold(&self) -> Option<f64> {
        None
    }

    fn vehicle_dismount_position(
        &self,
        passenger: &dyn EntityBase,
        still_current: &dyn Fn() -> bool,
    ) -> Option<Vector3<f64>> {
        dismount_helper::default_vehicle_exit(self.get_entity(), passenger, still_current)
    }

    fn push(&self, entity: &dyn EntityBase) {
        // A Java player predicts the other half of a living pusher's own contact.
        self.push_contact(entity, Entity::push_back_by);
    }

    fn push_contact(&self, entity: &dyn EntityBase, push_other: fn(&Entity, Vector3<f64>)) {
        // Vanilla `LivingEntity.push`: sleeping entities neither push nor get pushed here.
        if self.get_living_entity().is_some()
            && self.get_entity().pose.load() == EntityPose::Sleeping
        {
            return;
        }

        let self_entity = self.get_entity();
        let other_entity = entity.get_entity();

        if self_entity.no_physics.load(Ordering::Relaxed)
            || other_entity.no_physics.load(Ordering::Relaxed)
            || !self.can_push_contact(entity)
        {
            return;
        }

        if self.is_passenger_of_same_vehicle(entity) {
            return;
        }

        let mut dx = other_entity.pos.load().x - self_entity.pos.load().x;
        let mut dz = other_entity.pos.load().z - self_entity.pos.load().z;
        let mut d = dx.abs().max(dz.abs());
        if d >= 0.01 {
            d = d.sqrt();
            dx /= d;
            dz /= d;
            let mut d2 = 1.0 / d;
            if d2 > 1.0 {
                d2 = 1.0;
            }
            dx *= d2;
            dz *= d2;
            dx *= PUSH_STRENGTH;
            dz *= PUSH_STRENGTH;

            if !self_entity.has_passengers() && self.is_pushable() {
                self_entity.push_by(Vector3::new(-dx, 0.0, -dz));
            }

            if !other_entity.has_passengers() && entity.is_pushable() {
                push_other(other_entity, Vector3::new(dx, 0.0, dz));
            }
        }
    }

    fn on_hit(&self, _hit: crate::entity::projectile::ProjectileHit) {}

    fn set_paddle_state(&self, _left: bool, _right: bool) {}

    fn is_in_love(&self) -> bool {
        false
    }

    fn is_breeding_ready(&self) -> bool {
        false
    }

    fn reset_love(&self) {}

    fn set_breeding_cooldown(&self, _ticks: i32) {}

    fn is_panicking(&self) -> bool {
        false
    }

    fn get_entity(&self) -> &Entity;

    fn get_living_entity(&self) -> Option<&LivingEntity>;

    /// Whether this entity takes damage from water.
    fn is_sensitive_to_water(&self) -> bool {
        self.get_mob().is_some_and(mob::Mob::is_sensitive_to_water)
    }

    fn cast_any(&self) -> &dyn std::any::Any;

    fn get_item_entity(&self) -> Option<&ItemEntity> {
        None
    }

    fn get_player(&self) -> Option<&Player> {
        None
    }

    /// Should return the name of the entity without click or hover events.
    fn get_name(&self) -> TextComponent {
        let entity = self.get_entity();
        entity
            .custom_name
            .load()
            .as_ref()
            .clone()
            .unwrap_or(TextComponent::translate_cross(
                format!("entity.minecraft.{}", entity.entity_type.resource_name),
                format!("entity.minecraft.{}", entity.entity_type.resource_name),
                [],
            ))
    }

    fn get_display_name(&self) -> TextComponent {
        // TODO: team color
        let entity = self.get_entity();
        let mut name =
            entity
                .custom_name
                .load()
                .as_ref()
                .clone()
                .unwrap_or(TextComponent::translate_cross(
                    format!("entity.minecraft.{}", entity.entity_type.resource_name),
                    format!("entity.minecraft.{}", entity.entity_type.resource_name),
                    [],
                ));
        let name_clone = name.clone();
        name = name.hover_event(HoverEvent::show_entity(
            entity.entity_uuid.to_string(),
            entity.entity_type.resource_name.into(),
            Some(name_clone),
        ));
        name = name.insertion(entity.entity_uuid.to_string());
        name
    }

    /// Kills the Entity.
    fn kill(&self, caller: &dyn EntityBase) {
        if self.get_living_entity().is_some() {
            caller.damage(caller, f32::MAX, DamageType::GENERIC_KILL);
        } else {
            // TODO this should be removed once all entities are implemented
            self.get_entity().remove();
        }
    }

    fn get_experience_reward(&self, _killer: Option<&dyn EntityBase>) -> u32 {
        0
    }

    fn should_drop_loot(&self) -> bool {
        self.get_living_entity()
            .is_none_or(LivingEntity::should_drop_loot)
    }

    fn should_drop_experience(&self) -> bool {
        self.get_living_entity()
            .is_none_or(LivingEntity::should_drop_experience)
    }

    fn get_base_experience_reward(&self) -> u32 {
        0
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum RemovalReason {
    Killed,
    Discarded,
    UnloadedToChunk,
    UnloadedWithPlayer,
    ChangedDimension,
}

impl RemovalReason {
    #[must_use]
    pub const fn should_destroy(&self) -> bool {
        match self {
            Self::Killed | Self::Discarded => true,
            Self::UnloadedToChunk | Self::UnloadedWithPlayer | Self::ChangedDimension => false,
        }
    }

    #[must_use]
    pub const fn should_save(&self) -> bool {
        match self {
            Self::Killed | Self::Discarded | Self::UnloadedWithPlayer | Self::ChangedDimension => {
                false
            }
            Self::UnloadedToChunk => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PassengerRemoval {
    Dismount,
    Teleport,
    VehicleRemoved,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PassengerMount {
    Gameplay,
    Restoration,
}

#[derive(Clone, Copy)]
struct CommittedPassengerMount {
    revision: u64,
    stood: bool,
    displaced_driver: Option<i32>,
    switch_placement_baseline: Option<(Vector3<f64>, i32)>,
}

// IMPORTANT: have that 1 and not 0 because fetch_add returns previous value and 0 would be invalid
static CURRENT_ID: AtomicI32 = AtomicI32::new(1);

/// Position, motion and packed rotation for spawn packets.
struct SpawnState {
    pos: Vector3<f64>,
    velocity: Vector3<f64>,
    pitch: u8,
    yaw: u8,
    head_yaw: u8,
}

const LEASH_AXIS_ELASTICITY: Vector3<f64> = Vector3::new(0.8, 0.2, 0.8);
const LEASH_TORSIONAL_ELASTICITY: f64 = 10.0;
const LEASH_STIFFNESS: f64 = 0.11;
const ENTITY_ATTACHMENT_POINT: Vector3<f64> = Vector3::new(0.0, 0.5, 0.5);
const LEASHER_ATTACHMENT_POINT: Vector3<f64> = Vector3::new(0.0, 0.5, 0.0);
const SHARED_QUAD_ATTACHMENT_POINTS: [Vector3<f64>; 4] = [
    Vector3::new(-0.5, 0.5, 0.5),
    Vector3::new(-0.5, 0.5, -0.5),
    Vector3::new(0.5, 0.5, -0.5),
    Vector3::new(0.5, 0.5, 0.5),
];

const LEASHABLE_SCAN_SIZE: f64 = 32.0;

pub fn can_have_a_leash_attached_to(leashee: &dyn EntityBase, holder: &dyn EntityBase) -> bool {
    let entity = leashee.get_entity();
    let holder = holder.get_entity();
    entity.entity_id != holder.entity_id
        && entity.leash_distance_to(holder) <= leashee.leash_snap_distance()
        && leashee.get_mob().is_none_or(mob::Mob::can_be_leashed)
}

pub fn leashable_in_area(
    world: &World,
    center: Vector3<f64>,
    holder_id: i32,
) -> Vec<Arc<dyn EntityBase>> {
    let half = Vector3::new(
        LEASHABLE_SCAN_SIZE / 2.0,
        LEASHABLE_SCAN_SIZE / 2.0,
        LEASHABLE_SCAN_SIZE / 2.0,
    );
    world
        .get_entities_at_box(&BoundingBox::new(center - half, center + half))
        .into_iter()
        .filter(|other| {
            other
                .get_entity()
                .get_leash_holder()
                .is_some_and(|holder| holder.get_entity().entity_id == holder_id)
        })
        .collect()
}

/// Rotates `vector` around the Y axis.
fn y_rot(vector: Vector3<f64>, angle: f32) -> Vector3<f64> {
    let cos = f64::from(angle.cos());
    let sin = f64::from(angle.sin());
    Vector3::new(
        vector.x * cos + vector.z * sin,
        vector.y,
        vector.z * cos - vector.x * sin,
    )
}

/// Attachment points for the entity's current pose and age, and the scale they grow with.
fn attachments_of(entity: &(impl EntityBase + ?Sized)) -> (&'static EntityAttachments, f64) {
    let base = entity.get_entity();
    let entity_type = base.entity_type;
    let living = entity.get_living_entity();
    let baby = living.is_some_and(LivingEntity::is_baby);
    let attachments = entity_type
        .pose_dimensions(base.pose.load(), baby)
        .map(|pose| &pose.attachments)
        .or_else(|| entity_type.baby_attachments.as_ref().filter(|_| baby))
        .unwrap_or(&entity_type.attachments);
    // The scale the current dimensions were built with, like vanilla's scaled attachments
    (attachments, f64::from(base.scale.load()))
}

pub(crate) fn attachment_point(point: [f64; 3], scale: f64, yaw: f32) -> Vector3<f64> {
    let [x, y, z] = point;
    y_rot(Vector3::new(x, y, z) * scale, -yaw.to_radians())
}

pub(crate) fn average_passenger_attachment(entity: &(impl EntityBase + ?Sized)) -> Vector3<f64> {
    let (attachments, scale) = attachments_of(entity);
    let points = attachments.passenger;
    if points.is_empty() {
        return Vector3::default();
    }
    points.iter().fold(Vector3::default(), |sum, point| {
        sum + Vector3::new(point[0], point[1], point[2])
    }) * (scale / points.len() as f64)
}

pub fn default_passenger_attachment_point(
    vehicle: &dyn EntityBase,
    index: usize,
) -> Option<Vector3<f64>> {
    let (seats, scale) = attachments_of(vehicle);
    let seat = *seats.passenger.get(index).or(seats.passenger.last())?;
    Some(attachment_point(
        seat,
        scale,
        vehicle.get_entity().yaw.load(),
    ))
}

/// Moves `passenger` onto its seat on `vehicle`.
pub fn position_rider(vehicle: &dyn EntityBase, passenger: &dyn EntityBase) {
    if let Some(position) = rider_seat_position(vehicle, passenger) {
        passenger.get_entity().set_pos(position);
        vehicle.on_rider_positioned(passenger);
    }
}

fn rider_seat_position(
    vehicle: &dyn EntityBase,
    passenger: &dyn EntityBase,
) -> Option<Vector3<f64>> {
    let vehicle_entity = vehicle.get_entity();
    let passenger_entity = passenger.get_entity();
    let index = vehicle_entity
        .passengers
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .position(|other| other.get_entity().entity_id == passenger_entity.entity_id)?;
    let seat = vehicle
        .passenger_attachment_point(passenger, index)
        .or_else(|| default_passenger_attachment_point(vehicle, index))?;
    let offset = passenger
        .vehicle_attachment_point(vehicle)
        .unwrap_or_else(|| {
            let (own, own_scale) = attachments_of(passenger);
            attachment_point(own.vehicle, own_scale, passenger_entity.yaw.load())
        });
    Some(vehicle_entity.pos.load() + seat - offset)
}

/// Places every rider of `vehicle`, and their riders, after the vehicle has moved.
pub fn position_riders(vehicle: &dyn EntityBase) {
    let passengers = vehicle
        .get_entity()
        .passengers
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    for passenger in passengers {
        position_rider(vehicle, passenger.as_ref());
        position_riders(passenger.as_ref());
    }
}

/// The end of a ridden entity's own tick: it stays on its seat and builds no fall.
pub fn ride_tick(entity: &dyn EntityBase) {
    let Some(vehicle) = entity.get_entity().get_vehicle() else {
        return;
    };
    let base = entity.get_entity();
    base.velocity.store(Vector3::default());
    if let Some(living) = entity.get_living_entity() {
        living.fall_distance.store(0.0);
    }
    position_rider(vehicle.as_ref(), entity);
    if let Some(mob) = entity.get_mob() {
        mob.mob_ride_tick();
    }
}

/// Vanilla `rideTick` for every rider of a root vehicle, top-down once all entities have
/// ticked, so riders in other tick islands never race their vehicle.
pub fn ride_tick_riders(vehicle: &dyn EntityBase) {
    let passengers = vehicle
        .get_entity()
        .passengers
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    for passenger in passengers {
        if passenger.get_player().is_some() {
            position_rider(vehicle, passenger.as_ref());
        } else {
            ride_tick(passenger.as_ref());
        }
        ride_tick_riders(passenger.as_ref());
    }
}

/// The movement the server knows about: a player's last reported step, otherwise velocity.
fn known_movement(entity: &dyn EntityBase) -> Vector3<f64> {
    let base = entity.get_entity();
    if let Some(controller) = entity.get_controlling_passenger()
        && controller.get_player().is_some()
        && base.is_alive()
    {
        return controller.get_entity().movement.load();
    }
    if entity.get_player().is_some() {
        return base.movement.load();
    }
    base.velocity.load()
}

fn holder_movement(entity: &dyn EntityBase) -> Vector3<f64> {
    if entity
        .get_mob()
        .is_some_and(|mob| mob.get_mob_entity().is_no_ai())
    {
        Vector3::new(0.0, 0.0, 0.0)
    } else {
        known_movement(entity)
    }
}

/// What a leash did this tick, see [`Entity::tick_leash`].
pub enum LeashTick {
    NotLeashed,
    Snapped,
    Elastic(Arc<dyn EntityBase>),
    CloseRange(Arc<dyn EntityBase>),
}

/// Represents a non-living Entity (e.g. Item, Egg, Snowball...)
#[repr(C)]
pub struct Entity {
    /// A unique identifier for the entity
    pub entity_id: i32,
    /// A persistent, unique identifier for the entity
    pub entity_uuid: uuid::Uuid,
    /// The type of entity (e.g., player, zombie, item)
    pub entity_type: &'static EntityType,
    /// The world in which the entity exists.
    /// Uses `ArcSwap` to allow atomic updates when changing dimensions.
    pub world: ArcSwap<World>,
    /// The entity's current position in the world
    pub pos: AtomicCell<Vector3<f64>>,
    /// The last known position of the entity.
    pub last_pos: AtomicCell<Vector3<f64>>,
    /// The last movement vector
    pub movement: AtomicCell<Vector3<f64>>,
    /// The entity's position rounded to the nearest block coordinates
    pub block_pos: AtomicCell<BlockPos>,
    /// The block supporting the entity
    pub supporting_block_pos: AtomicCell<Option<BlockPos>>,
    /// The chunk coordinates of the entity's current position
    pub chunk_pos: AtomicCell<Vector2<i32>>,
    /// Indicates whether the entity is sneaking
    pub sneaking: AtomicBool,
    /// Indicates whether the entity is sprinting
    pub sprinting: AtomicBool,
    /// Indicates whether the entity is swimming
    pub swimming: AtomicBool,
    /// Indicates whether the entity is invisible
    pub invisible: AtomicBool,
    /// Indicates whether the entity is glowing
    pub glowing: AtomicBool,
    pub has_glowing_tag: AtomicBool,
    pub glowing_from_effect: AtomicBool,
    /// Indicates whether the entity is flying due to a fall
    pub fall_flying: AtomicBool,
    /// The entity's current velocity vector, aka knockback
    pub velocity: AtomicCell<Vector3<f64>>,
    /// Tracks a horizontal collision
    pub horizontal_collision: AtomicBool,
    pub vertical_collision: AtomicBool,
    pub(crate) vertical_collision_below: AtomicBool,
    vehicle_movements: Option<std::sync::Mutex<vehicle::movement_history::MovementHistory>>,
    /// Indicates whether the entity is on the ground (may not always be accurate).
    pub on_ground: AtomicBool,
    /// Indicates whether the entity is touching water
    pub touching_water: AtomicBool,
    /// Indicates the fluid height
    pub water_height: AtomicCell<f64>,
    /// Indicates whether the entity is touching lava
    pub touching_lava: AtomicBool,
    /// Indicates the fluid height
    pub lava_height: AtomicCell<f64>,
    /// The entity's yaw rotation (horizontal rotation) ← →
    pub yaw: AtomicCell<f32>,
    /// The entity's head yaw rotation (horizontal rotation of the head)
    pub head_yaw: AtomicCell<f32>,
    /// The entity's body yaw rotation (horizontal rotation of the body)
    pub body_yaw: AtomicCell<f32>,
    /// The entity's pitch rotation (vertical rotation) ↑ ↓
    pub pitch: AtomicCell<f32>,
    /// The entity's current pose (e.g., standing, sitting, swimming).
    pub pose: AtomicCell<EntityPose>,
    /// The bounding box of an entity (hitbox)
    pub bounding_box: EntityBoundingBox,
    ///The size (width and height) of the bounding box
    pub entity_dimension: AtomicCell<EntityDimensions>,
    /// Vanilla `getDefaultDimensions` when it differs from the type size (baby, slime, ...)
    default_dimensions: AtomicCell<Option<EntityDimensions>>,
    /// Vanilla `LivingEntity.getScale` (`Attributes.SCALE`). 1.0 for non-living.
    pub scale: AtomicCell<f32>,
    /// Whether this entity is invulnerable to all damage
    pub invulnerable: AtomicBool,
    /// List of damage types this entity is immune to
    pub damage_immunities: std::sync::Mutex<Vec<DamageType>>,
    // Whether the entity is immune to fire (to disable visual fire and fire damage)
    pub fire_immune: AtomicBool,
    pub fire_ticks: AtomicI32,
    pub has_visual_fire: AtomicBool,
    /// The number of ticks the entity has been frozen (in powder snow)
    /// Max is 140 ticks (7 seconds). Increases by 1/tick in powder snow, decreases by 2/tick outside.
    pub frozen_ticks: AtomicI32,
    /// Set during block-collision processing when the entity is touching powder snow.
    pub is_in_powder_snow: AtomicBool,
    /// True if the entity was in powder snow during the previous tick.
    pub was_in_powder_snow: AtomicBool,
    pub removal_reason: AtomicCell<Option<RemovalReason>>,
    // The passengers that entity has
    pub passengers: std::sync::Mutex<Vec<Arc<dyn EntityBase>>>,
    pub(crate) tracking_pairing_lock: std::sync::Mutex<()>,
    /// The vehicle that entity is in
    pub vehicle: std::sync::Mutex<Option<Weak<dyn EntityBase>>>,
    // Advanced under the vehicle mutex for each new or ended attachment.
    riding_revision: std::sync::atomic::AtomicU64,
    /// The entity this entity is attached/leashed to (if any)
    pub leashed_to: std::sync::Mutex<Option<Weak<dyn EntityBase>>>,
    pub delayed_leash: std::sync::Mutex<Option<DelayedLeash>>,
    /// Spin a taut leash gives the leashed entity, applied to its yaw each tick.
    leash_angular_momentum: AtomicCell<f64>,
    leash_release_cancelled: AtomicBool,
    /// Cooldown before entity can mount again after dismounting
    pub riding_cooldown: AtomicI32,
    /// Vanilla `moveDist`, `flyDist` and `nextStep`, which pace step sounds and movement game events.
    pub move_dist: AtomicCell<f32>,
    pub fly_dist: AtomicCell<f32>,
    pub next_step: AtomicCell<f32>,
    crystal_sound_intensity: AtomicCell<f32>,
    last_crystal_sound_play_tick: AtomicI32,
    /// Vanilla `AgeableMob.age`: negative while a baby, counting up to 0.
    pub age: AtomicI32,
    /// Vanilla `tickCount`: ticks since the entity was created.
    pub tick_count: AtomicI32,

    pub current_biome: ArcSwap<&'static Biome>,
    pub last_biome_update_pos: AtomicCell<BlockPos>,

    pub portal_cooldown: AtomicU32,

    pub portal_manager: std::sync::Mutex<Option<PortalProcessor>>,
    /// Custom name for the entity
    pub custom_name: ArcSwap<Option<TextComponent>>,
    /// Indicates whether the entity's custom name is visible
    pub custom_name_visible: AtomicBool,
    pub blocks_building: bool,
    pub silent: AtomicBool,
    pub air_supply: AtomicI32,
    pub has_no_gravity: AtomicBool,
    /// Scoreboard tags attached to this entity, managed with `/tag`.
    /// Vanilla allows at most [`MAX_SCOREBOARD_TAGS`] tags per entity.
    pub scoreboard_tags: std::sync::Mutex<HashSet<String>>,
    /// The data send in the Entity Spawn packet
    pub data: AtomicI32,
    /// Stores entity boolean flags (on fire, sneaking, invisible, glowing, etc.)
    pub flags: std::sync::atomic::AtomicI8,
    /// Stores Bedrock-specific entity boolean flags (bit 0-63)
    pub bedrock_flags: std::sync::atomic::AtomicI64,
    /// Stores more Bedrock-specific entity boolean flags (bit 0-63)
    pub bedrock_flags_two: std::sync::atomic::AtomicI64,
    bedrock_metadata_enqueue: std::sync::Mutex<()>,
    bedrock_flags_dirty: AtomicBool,
    /// If true, the entity bypasses physics, collisions, and block effects (e.g. spectator, markers, display entities)
    pub no_physics: AtomicBool,
    /// Vanilla `firstTick`: true until the first base tick has run.
    pub first_tick: AtomicBool,
    pub synched_data: synched_entity_data::SynchedEntityData,
    /// Multiplies movement for one tick before being reset
    pub movement_multiplier: AtomicCell<Vector3<f64>>,
    /// Vanilla `needsSync`: tracker resyncs position and velocity of entities
    pub velocity_dirty: AtomicBool,
    /// velocity goes to the own client
    pub sync_velocity: AtomicBool,
    pub(crate) sync_position: AtomicBool,
    /// Unsent entity pushes on a player. Bedrock clients don't predict them, so they stay
    /// pending at full size until `SetActorMotion` can layer them on the last client delta.
    pub push_impulse: AtomicCell<Vector3<f64>>,
    /// Set when an Entity is to be removed but could still be referenced
    pub removed: AtomicBool,
    /// The last sent yaw value (encoded as u8) for change detection
    pub last_sent_yaw: AtomicU8,
    /// The last sent pitch value (encoded as u8) for change detection
    pub last_sent_pitch: AtomicU8,
    /// Delta base of the move packets. Vanilla `ServerEntity.positionCodec`.
    pub last_sent_pos: AtomicCell<Vector3<f64>>,
    /// Cache for the last sent head yaw byte
    pub last_sent_head_yaw: AtomicU8,
    /// Velocity last put on the wire. Vanilla `ServerEntity.lastSentMovement`.
    pub last_sent_velocity: AtomicCell<Vector3<f64>>,
    /// Persistent custom data container for plugins
    pub custom_data: std::sync::Mutex<NbtCompound>,
}

/// Entity bounds that keep the world's spatial queries current when written.
pub struct EntityBoundingBox {
    value: AtomicCell<BoundingBox>,
    index: std::sync::RwLock<std::sync::Weak<crate::world::EntityIndex>>,
    is_player: bool,
    entity_id: i32,
}

impl EntityBoundingBox {
    fn new(world: &World, entity_id: i32, entity_type: &EntityType, bounds: BoundingBox) -> Self {
        Self {
            value: AtomicCell::new(bounds),
            index: std::sync::RwLock::new(Arc::downgrade(&world.entity_index)),
            is_player: entity_type == &EntityType::PLAYER,
            entity_id,
        }
    }

    /// Returns the current bounds.
    #[must_use]
    pub fn load(&self) -> BoundingBox {
        self.value.load()
    }

    /// Publishes bounds and their spatial index membership together.
    pub fn store(&self, bounds: BoundingBox) {
        let index = self
            .index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(index) = index.upgrade() {
            if self.is_player {
                index.set_player_bounds(self.entity_id, &self.value, bounds);
            } else {
                index.set_bounds(self.entity_id, &self.value, bounds);
            }
        } else {
            self.value.store(bounds);
        }
    }

    fn set_index(&self, index: &Arc<crate::world::EntityIndex>) {
        let mut owner = self
            .index
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *owner = Arc::downgrade(index);
        if self.is_player {
            index.set_player_bounds(self.entity_id, &self.value, self.value.load());
        } else {
            index.set_bounds(self.entity_id, &self.value, self.value.load());
        }
    }
}

/// Result of projecting movement before committing position or contact state.
pub(crate) struct Collision {
    /// Movement allowed by the collision shapes.
    pub movement: Vector3<f64>,
    /// Block landed on.
    pub supporting: Option<BlockPos>,
    /// A collision stopped X or Z.
    pub horizontal: bool,
    /// A collision stopped Y.
    pub vertical: bool,
}

impl Collision {
    const fn free(movement: Vector3<f64>) -> Self {
        Self {
            movement,
            supporting: None,
            horizontal: false,
            vertical: false,
        }
    }
}

impl Entity {
    #[must_use]
    pub fn get_server(&self) -> Option<std::sync::Arc<Server>> {
        self.world.load().server.upgrade()
    }

    #[must_use]
    pub fn resolve_dynamic_raw_id(
        &self,
        registry: &str,
        name: &str,
        static_fallback: impl FnOnce(&str) -> Option<u32>,
    ) -> Option<u32> {
        self.get_server()
            .and_then(|s| {
                s.datapack_manager
                    .get_dynamic_registry_raw_id(registry, name)
            })
            .or_else(|| static_fallback(name))
    }

    #[must_use]
    pub fn resolve_dynamic_entry_id(
        &self,
        registry: &str,
        raw_id: u32,
        static_fallback: impl FnOnce(u32) -> String,
    ) -> String {
        self.get_server()
            .and_then(|s| {
                s.datapack_manager
                    .get_dynamic_registry_entry_id(registry, raw_id)
            })
            .unwrap_or_else(|| static_fallback(raw_id))
    }

    pub fn new(
        world: Arc<World>,
        position: Vector3<f64>,
        entity_type: &'static EntityType,
    ) -> Self {
        Self::from_uuid(Uuid::new_v4(), world, position, entity_type)
    }

    pub fn reserve_ids(count: i32) -> i32 {
        CURRENT_ID.fetch_add(count, Relaxed)
    }

    pub fn from_uuid(
        entity_uuid: uuid::Uuid,
        world: Arc<World>,
        position: Vector3<f64>,
        entity_type: &'static EntityType,
    ) -> Self {
        Self::from_uuid_with_id(
            CURRENT_ID.fetch_add(1, Relaxed),
            entity_uuid,
            world,
            position,
            entity_type,
        )
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one line per field of the struct literal"
    )]
    pub fn from_uuid_with_id(
        entity_id: i32,
        entity_uuid: uuid::Uuid,
        world: Arc<World>,
        position: Vector3<f64>,
        entity_type: &'static EntityType,
    ) -> Self {
        let floor_x = position.x.floor() as i32;
        let floor_y = position.y.floor() as i32;
        let floor_z = position.z.floor() as i32;

        let bounding_box_size = Self::type_dimensions(entity_type);

        let current_biome = world
            .level
            .get_rough_biome(&BlockPos::new(floor_x, floor_y, floor_z));

        let bounding_box = EntityBoundingBox::new(
            &world,
            entity_id,
            entity_type,
            Self::make_bounding_box(entity_type, position, &bounding_box_size),
        );

        Self {
            entity_id,
            entity_uuid,
            entity_type,
            on_ground: AtomicBool::new(false),
            touching_water: AtomicBool::new(false),
            water_height: AtomicCell::new(0.0),
            touching_lava: AtomicBool::new(false),
            lava_height: AtomicCell::new(0.0),
            horizontal_collision: AtomicBool::new(false),
            vertical_collision: AtomicBool::new(false),
            vertical_collision_below: AtomicBool::new(false),
            vehicle_movements: (entity_type.boat.is_some()
                || entity_type.has_tag(&tag::EntityType::C_MINECARTS))
            .then(std::sync::Mutex::default),
            pos: AtomicCell::new(position),
            last_pos: AtomicCell::new(position),
            movement: AtomicCell::new(Vector3::default()),
            block_pos: AtomicCell::new(BlockPos(Vector3::new(floor_x, floor_y, floor_z))),
            supporting_block_pos: AtomicCell::new(None),
            chunk_pos: AtomicCell::new(Vector2::new(
                get_section_cord(floor_x),
                get_section_cord(floor_z),
            )),
            sneaking: AtomicBool::new(false),
            swimming: AtomicBool::new(false),
            invisible: AtomicBool::new(false),
            glowing: AtomicBool::new(false),
            has_glowing_tag: AtomicBool::new(false),
            glowing_from_effect: AtomicBool::new(false),
            world: ArcSwap::new(world),
            sprinting: AtomicBool::new(false),
            fall_flying: AtomicBool::new(false),
            yaw: AtomicCell::new(0.0),
            head_yaw: AtomicCell::new(0.0),
            body_yaw: AtomicCell::new(0.0),
            pitch: AtomicCell::new(0.0),
            velocity: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            pose: AtomicCell::new(EntityPose::Standing),
            bounding_box,
            entity_dimension: AtomicCell::new(bounding_box_size),
            default_dimensions: AtomicCell::new(None),
            scale: AtomicCell::new(1.0),
            invulnerable: AtomicBool::new(false),
            damage_immunities: std::sync::Mutex::new(Vec::new()),
            data: AtomicI32::new(0),
            flags: std::sync::atomic::AtomicI8::new(0),
            bedrock_flags: std::sync::atomic::AtomicI64::new(0),
            bedrock_flags_two: std::sync::atomic::AtomicI64::new(0),
            bedrock_metadata_enqueue: std::sync::Mutex::new(()),
            bedrock_flags_dirty: AtomicBool::new(false),
            fire_immune: AtomicBool::new(false),
            fire_ticks: AtomicI32::new(-1),
            has_visual_fire: AtomicBool::new(false),
            frozen_ticks: AtomicI32::new(0),
            is_in_powder_snow: AtomicBool::new(false),
            was_in_powder_snow: AtomicBool::new(false),
            removal_reason: AtomicCell::new(None),
            passengers: std::sync::Mutex::new(Vec::new()),
            tracking_pairing_lock: std::sync::Mutex::new(()),
            vehicle: std::sync::Mutex::new(None),
            riding_revision: std::sync::atomic::AtomicU64::new(0),
            leashed_to: std::sync::Mutex::new(None),
            delayed_leash: std::sync::Mutex::new(None),
            leash_angular_momentum: AtomicCell::new(0.0),
            leash_release_cancelled: AtomicBool::new(false),

            riding_cooldown: AtomicI32::new(0),
            move_dist: AtomicCell::new(0.0),
            fly_dist: AtomicCell::new(0.0),
            next_step: AtomicCell::new(1.0),
            crystal_sound_intensity: AtomicCell::new(0.0),
            last_crystal_sound_play_tick: AtomicI32::new(0),
            age: AtomicI32::new(0),
            tick_count: AtomicI32::new(0),
            current_biome: ArcSwap::new(Arc::new(current_biome)),
            last_biome_update_pos: AtomicCell::new(BlockPos::new(floor_x, floor_y, floor_z)),
            portal_cooldown: AtomicU32::new(0),
            portal_manager: std::sync::Mutex::new(None),
            custom_name: ArcSwap::new(Arc::new(None)),
            custom_name_visible: AtomicBool::new(false),
            blocks_building: false,
            silent: AtomicBool::new(false),
            air_supply: AtomicI32::new(breath::MAX_AIR),
            has_no_gravity: AtomicBool::new(false),
            scoreboard_tags: std::sync::Mutex::new(HashSet::new()),
            no_physics: AtomicBool::new(false),
            first_tick: AtomicBool::new(true),
            synched_data: synched_entity_data::SynchedEntityData::new(),
            movement_multiplier: AtomicCell::new(Vector3::default()),
            velocity_dirty: AtomicBool::new(true),
            sync_velocity: AtomicBool::new(false),
            sync_position: AtomicBool::new(false),
            push_impulse: AtomicCell::new(Vector3::default()),
            removed: AtomicBool::new(false),
            last_sent_yaw: AtomicU8::new(0),
            last_sent_pitch: AtomicU8::new(0),
            last_sent_head_yaw: AtomicU8::new(0),
            last_sent_pos: AtomicCell::new(position),
            last_sent_velocity: AtomicCell::new(Vector3::default()),
            custom_data: std::sync::Mutex::new(NbtCompound::new()),
        }
    }

    pub fn add_velocity(&self, velocity: Vector3<f64>) {
        self.set_velocity(self.velocity.load() + velocity);
    }

    pub fn set_velocity(&self, velocity: Vector3<f64>) {
        self.velocity.store(velocity);
        self.send_velocity();
    }

    /// Updates the world reference for this entity.
    /// Called when the entity changes dimensions (e.g., through a nether portal).
    pub fn set_world(&self, world: Arc<World>) {
        let block_pos = self.block_pos.load();
        let biome = world.level.get_rough_biome(&block_pos);
        self.current_biome.store(Arc::new(biome));
        self.last_biome_update_pos.store(block_pos);
        self.bounding_box.set_index(&world.entity_index);
        self.world.store(world);
    }

    fn init_bedrock_flags(&self) {
        if self.bedrock_flags.load(Ordering::Relaxed) == 0 {
            self.bedrock_flags.fetch_or(
                (1i64 << entity_data_flag::HAS_GRAVITY)
                    | (1i64 << entity_data_flag::CLIMB)
                    | (1i64 << entity_data_flag::HAS_COLLISION)
                    | (1i64 << entity_data_flag::BREATHING),
                Ordering::Relaxed,
            );
        }
    }

    fn bedrock_flag_word(&self) -> i64 {
        self.init_bedrock_flags();
        let mut flags = self.bedrock_flags.load(Relaxed);
        if self.age.load(Relaxed) < 0 {
            flags |= 1i64 << entity_data_flag::BABY;
        }
        if self.silent.load(Relaxed) {
            flags |= 1i64 << entity_data_flag::SILENT;
        }
        if self.has_no_gravity.load(Relaxed) {
            flags &= !(1i64 << entity_data_flag::HAS_GRAVITY);
        }
        if self.custom_name.load().is_some() && self.custom_name_visible.load(Relaxed) {
            flags |= (1i64 << entity_data_flag::SHOW_NAME)
                | (1i64 << entity_data_flag::ALWAYS_SHOW_NAME);
        }
        flags
    }

    pub fn bedrock_metadata(&self) -> SyncedActorDataList {
        let mut metadata = SyncedActorDataList::new();
        let dimension = self.entity_dimension.load();
        metadata.set(
            entity_data_key::WIDTH,
            MetadataValue::Float(dimension.width),
        );
        metadata.set(
            entity_data_key::HEIGHT,
            MetadataValue::Float(dimension.height),
        );
        let is_baby = self.age.load(Relaxed) < 0;
        metadata.set(
            entity_data_key::SCALE,
            MetadataValue::Float(if is_baby { 0.5 } else { 1.0 }),
        );
        metadata.set(
            entity_data_key::FLAGS,
            MetadataValue::Int64(self.bedrock_flag_word()),
        );
        metadata.set(
            entity_data_key::FLAGS_TWO,
            MetadataValue::Int64(self.bedrock_flags_two.load(Ordering::Relaxed)),
        );
        if self.has_vehicle() {
            metadata.set_flag(entity_data_key::FLAGS, entity_data_flag::RIDING as u8, true);
        }
        if let Ok(leash_guard) = self.leashed_to.try_lock()
            && let Some(holder) = leash_guard.as_ref().and_then(Weak::upgrade)
        {
            metadata.set(
                entity_data_key::LEASH_HOLDER,
                MetadataValue::Int64(i64::from(holder.get_entity().entity_id)),
            );
            metadata.set_flag(
                entity_data_key::FLAGS,
                entity_data_flag::LEASHED as u8,
                true,
            );
        }

        if let Some(name) = &**self.custom_name.load() {
            metadata.set(
                entity_data_key::NAME,
                MetadataValue::String(name.clone().get_text()),
            );
        }

        metadata
    }

    #[must_use]
    pub fn bedrock_actor_links(&self) -> Vec<pumpkin_protocol::bedrock::client::common::ActorLink> {
        let Ok(passengers) = self.passengers.try_lock() else {
            return Vec::new();
        };
        passengers
            .iter()
            .enumerate()
            .map(
                |(idx, p)| pumpkin_protocol::bedrock::client::common::ActorLink {
                    ridden_unique_id: VarLong(i64::from(self.entity_id)),
                    rider_unique_id: VarLong(i64::from(p.get_entity().entity_id)),
                    link_type: if idx == 0 { 1 } else { 2 },
                    immediate: true,
                    rider_initiated: false,
                    vehicle_angular_velocity: 0.0,
                },
            )
            .collect()
    }

    /// One block of walking past the current distance.
    #[must_use]
    pub fn default_next_step(&self) -> f32 {
        (self.move_dist.load() as i32 + 1) as f32
    }

    /// The type's baby size for babies, its adult size otherwise.
    #[must_use]
    pub fn age_dimensions(entity_type: &EntityType, baby: bool) -> EntityDimensions {
        entity_type
            .baby_dimensions
            .filter(|_| baby)
            .unwrap_or_else(|| Self::type_dimensions(entity_type))
    }

    /// The type's default size, vanilla `EntityType.getDimensions`.
    #[must_use]
    pub const fn type_dimensions(entity_type: &EntityType) -> EntityDimensions {
        EntityDimensions {
            width: entity_type.dimension[0],
            height: entity_type.dimension[1],
            eye_height: entity_type.eye_height,
        }
    }

    /// Vanilla `makeBoundingBox`: wind charges sit 0.15 below their position.
    #[must_use]
    pub fn make_bounding_box(
        entity_type: &EntityType,
        position: Vector3<f64>,
        dimensions: &EntityDimensions,
    ) -> BoundingBox {
        let offset = if entity_type == &EntityType::WIND_CHARGE
            || entity_type == &EntityType::BREEZE_WIND_CHARGE
        {
            f64::from(0.15f32)
        } else {
            0.0
        };
        BoundingBox::new_from_pos(position.x, position.y - offset, position.z, dimensions)
    }

    /// Sets the entity's growth age. Negative values indicate that the entity is a baby.
    pub fn set_age(&self, age: i32) {
        let old_age = self.age.swap(age, Relaxed);
        if (old_age < 0) != (age < 0) {
            self.refresh_dimensions();
        }
    }

    /// Adds a scoreboard tag to this entity.
    ///
    /// Returns `false` if the entity already has the tag or already carries
    /// [`MAX_SCOREBOARD_TAGS`] tags.
    pub fn add_scoreboard_tag(&self, tag: &str) -> bool {
        let mut tags = self
            .scoreboard_tags
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        tags.len() < MAX_SCOREBOARD_TAGS && tags.insert(tag.to_owned())
    }

    /// Removes a scoreboard tag from this entity.
    ///
    /// Returns `false` if the entity did not have the tag.
    pub fn remove_scoreboard_tag(&self, tag: &str) -> bool {
        self.scoreboard_tags
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(tag)
    }

    /// Sets a custom name for the entity, typically used with nametags
    pub fn set_custom_name(&self, name: TextComponent) {
        self.custom_name.store(Arc::new(Some(name.clone())));
        let mut bedrock_meta = SyncedActorDataList::new();
        bedrock_meta.set(
            entity_data_key::NAME,
            MetadataValue::String(name.clone().get_text()),
        );
        let visible = self.custom_name_visible.load(Ordering::Relaxed);
        bedrock_meta.set_flag(
            entity_data_key::FLAGS,
            entity_data_flag::SHOW_NAME as u8,
            visible,
        );
        bedrock_meta.set_flag(
            entity_data_key::FLAGS,
            entity_data_flag::ALWAYS_SHOW_NAME as u8,
            visible,
        );
        self.set_synced_data(tracked_data::entity::DATA_CUSTOM_NAME, Some(name));
        self.send_bedrock_actor_data(&bedrock_meta);
    }

    pub fn clear_custom_name(&self) {
        if self.custom_name.load().is_none() {
            return;
        }
        self.custom_name.store(Arc::new(None));
        let mut bedrock_meta = SyncedActorDataList::new();
        bedrock_meta.set(entity_data_key::NAME, MetadataValue::String(String::new()));
        self.set_synced_data(
            tracked_data::entity::DATA_CUSTOM_NAME,
            None::<TextComponent>,
        );
        self.send_bedrock_actor_data(&bedrock_meta);
    }

    pub fn set_custom_name_visible(&self, visible: bool) {
        self.custom_name_visible.store(visible, Ordering::Relaxed);
        let mut bedrock_meta = SyncedActorDataList::new();
        if let Some(name) = &**self.custom_name.load() {
            bedrock_meta.set(
                entity_data_key::NAME,
                MetadataValue::String(name.clone().get_text()),
            );
        }
        bedrock_meta.set_flag(
            entity_data_key::FLAGS,
            entity_data_flag::SHOW_NAME as u8,
            visible,
        );
        bedrock_meta.set_flag(
            entity_data_key::FLAGS,
            entity_data_flag::ALWAYS_SHOW_NAME as u8,
            visible,
        );
        self.set_synced_data(tracked_data::entity::DATA_CUSTOM_NAME_VISIBLE, visible);
        self.send_bedrock_actor_data(&bedrock_meta);
    }

    pub fn set_air_supply(&self, mut air: i32) {
        if air == self.air_supply.load(Relaxed) {
            return;
        }
        if let Some(server) = self.world.load().server.upgrade() {
            let mut event =
                crate::plugin::api::events::entity::entity_air_change::EntityAirChangeEvent::new(
                    self.entity_id,
                    air,
                );
            event = server.plugin_manager.fire_blocking(&server, event);
            if event.cancelled {
                return;
            }
            air = event.amount;
        }
        self.air_supply.store(air, Relaxed);
        self.set_synced_data(tracked_data::entity::DATA_AIR_SUPPLY_ID, VarInt(air));
    }

    pub fn is_silent(&self) -> bool {
        self.silent.load(Ordering::Relaxed)
    }

    pub fn set_silent(&self, silent: bool) {
        self.silent.store(silent, Ordering::Relaxed);
        self.set_synced_data(tracked_data::entity::DATA_SILENT, silent);
    }

    pub fn has_no_gravity(&self) -> bool {
        self.has_no_gravity.load(Ordering::Relaxed)
    }

    pub fn set_has_no_gravity(&self, no_gravity: bool) {
        self.has_no_gravity.store(no_gravity, Ordering::Relaxed);
        self.set_synced_data(tracked_data::entity::DATA_NO_GRAVITY, no_gravity);
    }

    /// Vanilla `hurtMarked` path: immediate, to watchers and self. A player's goes through
    /// `PlayerVelocityEvent`.
    pub fn send_velocity(&self) {
        if self.entity_type == &EntityType::PLAYER
            && let Some(player) = self.world.load().get_player_by_id(self.entity_id)
        {
            player.send_hurt_velocity();
            return;
        }
        self.send_velocity_to_watchers();
    }

    /// watchers only: Own client predicts its pushes.
    pub fn send_velocity_to_watchers(&self) {
        let velocity = self.velocity.load();
        self.last_sent_velocity.store(velocity);
        self.world.load().send_to_tracking_players_editioned(
            self,
            &CEntityVelocity::new(self.entity_id.into(), velocity),
            &CSetActorMotion {
                target_runtime_id: VarULong(self.entity_id as u64),
                motion: velocity.to_f32_lossy(),
                tick: VarULong(0),
            },
        );
    }

    /// Vanilla `sendChanges`: motion moved by more than 1e-7 since the last send, or just stopped.
    #[must_use]
    pub fn velocity_changed_since_sent(&self) -> bool {
        let velocity = self.velocity.load();
        let diff = (velocity - self.last_sent_velocity.load()).length_squared();
        diff > 1.0e-7 || (diff > 0.0 && velocity.length_squared() == 0.0)
    }

    /// next fuse velocity send includes self
    pub fn mark_hurt(&self) {
        self.sync_velocity.store(true, Ordering::SeqCst);
    }

    /// Vanilla `LivingEntity.getDimensions`: the pose size from data, else the scaled default.
    #[must_use]
    pub fn get_dimensions(&self, pose: EntityPose) -> EntityDimensions {
        let baby = self.age.load(Relaxed) < 0;
        if let Some(pose_dimensions) = self.entity_type.pose_dimensions(pose, baby) {
            // Vanilla `SLEEPING_DIMENSIONS` is fixed, so it ignores the scale
            if pose == EntityPose::Sleeping {
                return pose_dimensions.dimensions;
            }
            return pose_dimensions.dimensions.scale(self.scale.load());
        }
        let default = self
            .default_dimensions
            .load()
            .unwrap_or_else(|| Self::age_dimensions(self.entity_type, baby));
        // Vanilla `Warden.getDefaultDimensions`: fixed 1 high while in the ground, unscaled
        if self.entity_type == &EntityType::WARDEN
            && matches!(pose, EntityPose::Digging | EntityPose::Emerging)
        {
            return EntityDimensions::new(default.width, 1.0, 0.85);
        }
        default.scale(self.scale.load())
    }

    /// Vanilla `getEyeHeight`: of the current size, which follows the pose
    pub fn get_eye_height(&self) -> f64 {
        f64::from(self.entity_dimension.load().eye_height)
    }

    /// Updates the entity's position, block position, and chunk position.
    ///
    /// This function calculates the new position, block position, and chunk position based on the provided coordinates. If any of these values change, the corresponding fields are updated.
    pub fn set_pos(&self, new_position: Vector3<f64>) {
        let pos = self.pos.load();
        if pos != new_position {
            self.pos.store(new_position);
            self.bounding_box.store(Self::make_bounding_box(
                self.entity_type,
                new_position,
                &self.entity_dimension.load(),
            ));

            let floor_x = new_position.x.floor() as i32;
            let floor_y = new_position.y.floor() as i32;
            let floor_z = new_position.z.floor() as i32;

            let block_pos = self.block_pos.load();
            let block_pos_vec = block_pos.0;
            if floor_x != block_pos_vec.x
                || floor_y != block_pos_vec.y
                || floor_z != block_pos_vec.z
            {
                let new_block_pos = Vector3::new(floor_x, floor_y, floor_z);
                let new_bp = BlockPos(new_block_pos);
                self.block_pos.store(new_bp);

                let world = self.world.load();
                let biome = world.level.get_rough_biome(&new_bp);
                self.current_biome.store(Arc::new(biome));
                self.last_biome_update_pos.store(new_bp);

                let chunk_pos = self.chunk_pos.load();
                if get_section_cord(floor_x) != chunk_pos.x
                    || get_section_cord(floor_z) != chunk_pos.y
                {
                    self.chunk_pos.store(Vector2::new(
                        get_section_cord(new_block_pos.x),
                        get_section_cord(new_block_pos.z),
                    ));
                }
            }
        }
    }

    /// Returns entity rotation as vector
    pub fn rotation(&self) -> Vector3<f32> {
        let pitch_rad = self.pitch.load().to_radians();
        let yaw_rad = -self.yaw.load().to_radians();

        let cos_yaw = yaw_rad.cos();
        let sin_yaw = yaw_rad.sin();
        let cos_pitch = pitch_rad.cos();
        let sin_pitch = pitch_rad.sin();

        Vector3::new(sin_yaw * cos_pitch, -sin_pitch, cos_yaw * cos_pitch)
    }

    /// Changes this entity's pitch and yaw to look at target
    pub fn look_at(&self, target: Vector3<f64>) {
        let position = self.pos.load();
        let delta = target.sub(&position);
        let root = delta.x.hypot(delta.z);
        let pitch = wrap_degrees((-delta.y.atan2(root) as f32).to_degrees());
        let yaw = wrap_degrees((delta.z.atan2(delta.x) as f32).to_degrees() - 90.0);
        self.pitch.store(pitch);
        self.yaw.store(yaw);
    }

    fn default_portal_cooldown(&self) -> u32 {
        if self.entity_type == &EntityType::PLAYER {
            10
        } else {
            300
        }
    }

    #[must_use]
    pub fn root_vehicle_id(&self) -> i32 {
        let mut id = self.entity_id;
        let mut vehicle = self.get_vehicle();
        while let Some(current) = vehicle {
            id = current.get_entity().entity_id;
            vehicle = current.get_entity().get_vehicle();
        }
        id
    }

    /// Returns the block position of the block the (non-player) entity is standing on, if any.
    pub fn get_supporting_block_pos(&self) -> Option<BlockPos> {
        // Check if the entity is on the ground
        if !self.on_ground.load(Ordering::Relaxed) {
            return None;
        }

        self.supporting_block_pos.load()
    }

    fn adjust_movement_for_collisions(
        &self,
        movement: Vector3<f64>,
        caller: &dyn EntityBase,
    ) -> Vector3<f64> {
        self.vertical_collision.store(false, Ordering::SeqCst);
        self.vertical_collision_below.store(false, Ordering::SeqCst);
        if movement.length_squared() == 0.0 {
            return movement;
        }
        let collision = self.project_movement(movement, caller);
        let on_ground = collision.vertical && movement.y < 0.0;
        self.on_ground.store(on_ground, Ordering::SeqCst);
        self.vertical_collision_below
            .store(on_ground, Ordering::SeqCst);
        self.supporting_block_pos.store(collision.supporting);
        self.horizontal_collision
            .store(collision.horizontal, Ordering::SeqCst);
        self.vertical_collision
            .store(collision.vertical, Ordering::SeqCst);
        collision.movement
    }

    /// Client-controlled Java vehicles keep vertical flags on horizontal-only proposals.
    pub(crate) fn apply_client_vehicle_collision(
        &self,
        requested: Vector3<f64>,
        collision: &Collision,
    ) {
        self.horizontal_collision
            .store(collision.horizontal, Relaxed);
        if self.no_physics.load(Relaxed) {
            self.vertical_collision.store(false, Relaxed);
            self.vertical_collision_below.store(false, Relaxed);
        } else if requested.y != 0.0 {
            let on_ground = collision.vertical && requested.y < 0.0;
            self.vertical_collision.store(collision.vertical, Relaxed);
            self.vertical_collision_below.store(on_ground, Relaxed);
            self.on_ground.store(on_ground, Relaxed);
            self.supporting_block_pos.store(collision.supporting);
        }
    }

    pub(crate) fn set_vehicle_packet_ground(
        &self,
        caller: &dyn EntityBase,
        on_ground: bool,
        movement: Vector3<f64>,
    ) {
        self.on_ground.store(on_ground, Relaxed);
        let supporting = on_ground.then(|| {
            let bounds = self.bounding_box.load();
            let range =
                Self::support_test_area(&bounds).expand_towards(-movement.x, 0.0, -movement.z);
            let (colliders, block_positions) =
                self.world.load().get_block_collisions(range, caller);
            let from = self.pos.load() - movement;
            Self::supporting_block_after(
                &colliders,
                &block_positions,
                &bounds.shift(movement * -1.0),
                movement,
                from,
            )
        });
        self.supporting_block_pos.store(supporting.flatten());
    }

    pub(crate) fn record_vehicle_movement(
        &self,
        from: Vector3<f64>,
        movement: Vector3<f64>,
        requested: Vector3<f64>,
    ) -> bool {
        self.vehicle_movements.as_ref().is_none_or(|history| {
            history
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .record(from, movement, requested)
        })
    }

    fn clear_vehicle_movement(&self) {
        if let Some(history) = &self.vehicle_movements {
            let mut history = history
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            history.pending.clear();
        }
    }

    pub(crate) fn vehicle_motion_epoch(&self) -> u64 {
        self.vehicle_movements.as_ref().map_or(0, |history| {
            history
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .teleport_revision
        })
    }

    pub(crate) fn lock_vehicle_motion_if_current(&self, epoch: u64) -> Option<impl Sized + '_> {
        self.vehicle_movements.as_ref().map_or_else(
            || (epoch == 0).then_some(None),
            |history| {
                let history = history
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                (history.teleport_revision == epoch).then_some(Some(history))
            },
        )
    }

    pub(crate) fn vehicle_block_effect_reach(&self) -> f64 {
        self.vehicle_movements.as_ref().map_or(0.0, |history| {
            let history = history
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let current = self.pos.load();
            history.pending.iter().fold(0.0f64, |reach, movement| {
                reach
                    .max((movement.from - current).length())
                    .max((movement.to - current).length())
            })
        })
    }

    pub(crate) fn discard_last_vehicle_movement(&self) {
        if let Some(history) = &self.vehicle_movements {
            history
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .pending
                .pop_back();
        }
    }

    /// Project the full move, including a living entity's step-up, without writing state.
    #[expect(clippy::float_cmp)]
    pub(crate) fn project_movement(
        &self,
        movement: Vector3<f64>,
        caller: &dyn EntityBase,
    ) -> Collision {
        if movement.length_squared() == 0.0 || self.no_physics.load(Relaxed) {
            return Collision::free(movement);
        }
        let was_on_ground = self.on_ground.load(Ordering::SeqCst);
        let bounding_box = self.bounding_box.load();
        let collision = self.collide(movement, caller);
        let movement_step = collision.movement;

        let max_up_step = Self::max_up_step(caller);
        let horizontal_collision = movement.x != movement_step.x || movement.z != movement_step.z;
        let on_ground_after_collision = movement.y != movement_step.y && movement.y < 0.0;
        if max_up_step <= 0.0
            || !(on_ground_after_collision || was_on_ground)
            || !horizontal_collision
        {
            return collision;
        }

        let grounded_box = if on_ground_after_collision {
            bounding_box.shift(Vector3::new(0.0, movement_step.y, 0.0))
        } else {
            bounding_box
        };
        let mut step_up_box =
            grounded_box.expand_towards(movement.x, f64::from(max_up_step), movement.z);
        if !on_ground_after_collision {
            step_up_box = step_up_box.expand_towards(0.0, f64::from(-1.0E-5f32), 0.0);
        }
        let (colliders, _) = self.world.load().get_block_collisions(step_up_box, caller);

        for step_height in Self::candidate_step_up_heights(
            &grounded_box,
            &colliders,
            max_up_step,
            movement_step.y as f32,
        ) {
            let step_from_ground = Self::collide_with_boxes(
                Vector3::new(movement.x, f64::from(step_height), movement.z),
                grounded_box,
                &colliders,
            );
            if step_from_ground.horizontal_length_squared()
                > movement_step.horizontal_length_squared()
            {
                let distance_to_ground = bounding_box.min.y - grounded_box.min.y;
                let stepped = step_from_ground - Vector3::new(0.0, distance_to_ground, 0.0);
                let on_ground = movement.y != stepped.y && movement.y < 0.0;
                // Vanilla checks support after the step, the pre-step support is stale.
                let supporting = on_ground
                    .then(|| {
                        let area = Self::support_test_area(&bounding_box.shift(stepped))
                            .expand_towards(-stepped.x, 0.0, -stepped.z);
                        let (collisions, block_positions) =
                            self.world.load().get_block_collisions(area, caller);
                        Self::supporting_block_after(
                            &collisions,
                            &block_positions,
                            &bounding_box,
                            stepped,
                            self.pos.load(),
                        )
                    })
                    .flatten();
                return Collision {
                    movement: stepped,
                    supporting,
                    horizontal: movement.x != stepped.x || movement.z != stepped.z,
                    vertical: movement.y != stepped.y,
                };
            }
        }

        collision
    }

    /// Only living entities step up, and a player-ridden mob steps at least one block.
    fn max_up_step(caller: &dyn EntityBase) -> f32 {
        if let Some(mob) = caller.get_mob()
            && let Some(step) = mob.mob_max_up_step()
        {
            return step;
        }
        let Some(living) = caller.get_living_entity() else {
            return 0.0;
        };
        let step_height =
            living.get_attribute_value(&pumpkin_data::attributes::Attributes::STEP_HEIGHT) as f32;
        if caller
            .get_controlling_passenger()
            .is_some_and(|passenger| passenger.get_player().is_some())
        {
            step_height.max(1.0)
        } else {
            step_height
        }
    }

    fn candidate_step_up_heights(
        bounding_box: &BoundingBox,
        colliders: &[BoundingBox],
        max_step_height: f32,
        step_height_to_skip: f32,
    ) -> Vec<f32> {
        let mut candidates: Vec<f32> = Vec::with_capacity(4);
        for collider in colliders {
            for coord in [collider.min.y, collider.max.y] {
                let relative_coord = (coord - bounding_box.min.y) as f32;
                if relative_coord < 0.0 || relative_coord == step_height_to_skip {
                    continue;
                }
                if relative_coord > max_step_height {
                    break;
                }
                if !candidates.contains(&relative_coord) {
                    candidates.push(relative_coord);
                }
            }
        }
        candidates.sort_by(f32::total_cmp);
        candidates
    }

    /// Resolves Y first, then the larger horizontal axis last.
    fn collide_with_boxes(
        movement: Vector3<f64>,
        bounding_box: BoundingBox,
        colliders: &[BoundingBox],
    ) -> Vector3<f64> {
        let axes = if movement.x.abs() < movement.z.abs() {
            [Axis::Y, Axis::Z, Axis::X]
        } else {
            [Axis::Y, Axis::X, Axis::Z]
        };
        let mut resolved = Vector3::new(0.0, 0.0, 0.0);
        for axis in axes {
            let mut amount = movement.get_axis(axis);
            if amount == 0.0 {
                continue;
            }
            let moved = bounding_box.shift(resolved);
            for collider in colliders {
                amount = Self::collide_axis(&moved, collider, axis, amount);
            }
            resolved.set_axis(axis, amount);
        }
        resolved
    }

    /// How far `moving` can travel along `axis` before it hits `other`.
    fn collide_axis(moving: &BoundingBox, other: &BoundingBox, axis: Axis, amount: f64) -> f64 {
        const EPSILON: f64 = 1.0E-7;
        let overlaps = [Axis::X, Axis::Y, Axis::Z]
            .into_iter()
            .filter(|&other_axis| other_axis != axis)
            .all(|other_axis| {
                moving.max.get_axis(other_axis) - EPSILON > other.min.get_axis(other_axis)
                    && moving.min.get_axis(other_axis) + EPSILON < other.max.get_axis(other_axis)
            });
        if !overlaps {
            return amount;
        }
        if amount > 0.0 && other.min.get_axis(axis) >= moving.max.get_axis(axis) - EPSILON {
            amount.min(other.min.get_axis(axis) - moving.max.get_axis(axis))
        } else if amount < 0.0 && other.max.get_axis(axis) <= moving.min.get_axis(axis) + EPSILON {
            amount.max(other.max.get_axis(axis) - moving.min.get_axis(axis))
        } else {
            amount
        }
    }

    /// Vanilla `collide` without writing state.
    #[expect(clippy::float_cmp)]
    pub(crate) fn collide(&self, movement: Vector3<f64>, caller: &dyn EntityBase) -> Collision {
        let bounding_box = self.bounding_box.load();

        let world = self.world.load();
        let swept_box = bounding_box.stretch(movement);
        let (mut collisions, block_positions) = world.get_block_collisions(swept_box, caller);
        if self.uses_vehicle_collisions() {
            // Entity and border shapes stop motion, but cannot supply a supporting block.
            // Block indices still refer to the initial span of this list.
            world.add_vehicle_collision_shapes(caller, swept_box, &mut collisions);
        }

        if collisions.is_empty() {
            return Collision::free(movement);
        }

        // Vanilla `collideWithShapes`: Y, then the larger horizontal axis, on the already moved box.
        let order = if movement.x.abs() < movement.z.abs() {
            [Axis::Y, Axis::Z, Axis::X]
        } else {
            [Axis::Y, Axis::X, Axis::Z]
        };
        let mut adjusted_movement = Vector3::new(0.0, 0.0, 0.0);
        for axis in order {
            let wanted = movement.get_axis(axis);
            if wanted == 0.0 {
                continue;
            }
            let (allowed, _) =
                bounding_box
                    .shift(adjusted_movement)
                    .collide_along(axis, &collisions, wanted);
            adjusted_movement.set_axis(axis, allowed);
        }

        // Vanilla `Mth.equal`: sideways cuts under 1e-5 are not a collision.
        let horizontal = (movement.x - adjusted_movement.x).abs() >= 1.0e-5
            || (movement.z - adjusted_movement.z).abs() >= 1.0e-5;
        let vertical = adjusted_movement.y != movement.y;

        let supporting = if vertical && movement.y < 0.0 {
            Self::supporting_block_after(
                &collisions,
                &block_positions,
                &bounding_box,
                adjusted_movement,
                self.pos.load(),
            )
        } else {
            None
        };

        Collision {
            movement: adjusted_movement,
            supporting,
            horizontal,
            vertical,
        }
    }

    pub(crate) fn uses_vehicle_collisions(&self) -> bool {
        self.entity_type.boat.is_some() || self.entity_type.has_tag(&tag::EntityType::C_MINECARTS)
    }

    /// Vanilla `checkSupportingBlock`: a thin box under the final position, else under the start X/Z.
    fn supporting_block_after(
        collisions: &[BoundingBox],
        block_positions: &[(usize, BlockPos)],
        start_box: &BoundingBox,
        movement: Vector3<f64>,
        start_pos: Vector3<f64>,
    ) -> Option<BlockPos> {
        let test_area = Self::support_test_area(&start_box.shift(movement));
        let position = start_pos + movement;
        Self::find_supporting_block(collisions, block_positions, &test_area, &position).or_else(
            || {
                Self::find_supporting_block(
                    collisions,
                    block_positions,
                    &test_area.shift(Vector3::new(-movement.x, 0.0, -movement.z)),
                    &position,
                )
            },
        )
    }

    fn support_test_area(final_box: &BoundingBox) -> BoundingBox {
        BoundingBox::new(
            Vector3::new(final_box.min.x, final_box.min.y - 1.0e-6, final_box.min.z),
            Vector3::new(final_box.max.x, final_box.min.y, final_box.max.z),
        )
    }

    /// Vanilla `CollisionGetter.findSupportingBlock`: the colliding block closest to `position`.
    fn find_supporting_block(
        collisions: &[BoundingBox],
        block_positions: &[(usize, BlockPos)],
        area: &BoundingBox,
        position: &Vector3<f64>,
    ) -> Option<BlockPos> {
        let mut best: Option<(f64, BlockPos)> = None;
        let mut start = 0;
        for &(end, pos) in block_positions {
            if collisions[start..end].iter().any(|s| s.intersects(area)) {
                let distance = pos.to_centered_f64().squared_distance_to_vec(position);
                // Ties go to the larger position in vanilla `Vec3i.compareTo` order (y, z, x).
                let better = best.is_none_or(|(best_distance, best_pos)| {
                    distance < best_distance
                        || (distance == best_distance
                            && (best_pos.0.y, best_pos.0.z, best_pos.0.x)
                                < (pos.0.y, pos.0.z, pos.0.x))
                });
                if better {
                    best = Some((distance, pos));
                }
            }
            start = end;
        }
        best.map(|(_, pos)| pos)
    }

    /// Applies knockback to the entity, following vanilla Minecraft's mechanics.
    /// `LivingEntity.takeKnockback()`
    /// This function calculates the entity's new velocity based on the specified knockback strength and direction.
    ///
    /// Knockback resistance is not applied here, because it is a `LivingEntity`
    /// attribute and this is an `Entity` method. Callers modelling vanilla's
    /// `LivingEntity.knockback` scale `strength` with
    /// `combat::knockback_after_resistance` first; callers modelling vanilla's raw
    /// `Entity.push` (such as the ender dragon) pass `strength` unscaled.
    pub fn apply_knockback(&self, strength: f64, mut x: f64, mut z: f64, attacker: Option<&Self>) {
        if strength <= 0.0 {
            // No vanilla knockback, but plugins can still add one
            if let Some(knockback) =
                self.fire_knockback_events(strength, Vector3::default(), attacker)
                && knockback != Vector3::default()
            {
                self.velocity_dirty.store(true, Ordering::SeqCst);
                self.velocity.store(self.velocity.load() + knockback);
            }
            return;
        }

        // This has some vanilla magic

        while x.mul_add(x, z * z) < 1.0E-5 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;

            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * strength;

        let velocity = self.velocity.load();
        let target = Vector3::new(
            velocity.x / 2.0 - var8.x,
            if self.on_ground.load(Relaxed) {
                (velocity.y / 2.0 + strength).min(0.4)
            } else {
                velocity.y
            },
            velocity.z / 2.0 - var8.z,
        );
        let Some(knockback) = self.fire_knockback_events(strength, target - velocity, attacker)
        else {
            return;
        };
        self.velocity_dirty.store(true, Ordering::SeqCst);
        self.velocity.store(velocity + knockback);
    }

    /// Fires `EntityKnockbackEvent`, then `EntityKnockbackByEntityEvent` with an attacker
    /// -> `None` if cancelled, else the velocity change
    pub fn fire_knockback_events(
        &self,
        strength: f64,
        knockback: Vector3<f64>,
        attacker: Option<&Self>,
    ) -> Option<Vector3<f64>> {
        let Some(server) = self.world.load().server.upgrade() else {
            return Some(knockback);
        };
        let mut event =
            crate::plugin::api::events::entity::entity_knockback::EntityKnockbackEvent {
                entity_id: self.entity_id,
                hit_by_id: attacker.map(|attacker| attacker.entity_id),
                knockback,
                cancelled: false,
            };
        event = server.plugin_manager.fire_blocking(&server, event);
        let mut knockback = event.knockback;
        let mut cancelled = event.cancelled;
        if let Some(attacker) = attacker {
            let mut event = crate::plugin::api::events::entity::entity_knockback_by_entity::EntityKnockbackByEntityEvent::new(
                self.entity_id,
                attacker.entity_id,
                strength,
                knockback.x,
                knockback.y,
                knockback.z,
            );
            event.cancelled = cancelled;
            event = server.plugin_manager.fire_blocking(&server, event);
            knockback = Vector3::new(event.x, event.y, event.z);
            cancelled = event.cancelled;
        }
        (!cancelled).then_some(knockback)
    }

    // Part of LivingEntity.tickMovement() in yarn

    pub fn check_zero_velo(&self) {
        let mut motion = self.velocity.load();

        if self.entity_type == &EntityType::PLAYER {
            if motion.horizontal_length_squared() < 9.0E-6 {
                motion.x = 0.0;

                motion.z = 0.0;
            }
        } else {
            if motion.x.abs() < 0.003 {
                motion.x = 0.0;
            }

            if motion.z.abs() < 0.003 {
                motion.z = 0.0;
            }
        }

        if motion.y.abs() < 0.003 {
            motion.y = 0.0;
        }

        self.velocity.store(motion);
    }

    #[expect(dead_code)]
    const fn tick_block_underneath() {
        // let world = self.world.read();

        // let (pos, block, state) = self.get_block_with_y_offset(0.2);

        // world
        //     .block_registry
        //     .on_stepped_on(&world, caller, pos, block, state)
        //     ;

        // TODO: Add this to on_stepped_on

        /*


        if self.on_ground.load(Ordering::SeqCst) {


            let (_pos, block, state) = self.get_block_with_y_offset(0.2);


            if let Some(live) = living {


                if block == Block::CAMPFIRE


                    || block == Block::SOUL_CAMPFIRE


                        && CampfireLikeProperties::from_state_id(state.id).r#signal_fire


                {


                    let _ = live.damage(1.0, DamageType::CAMPFIRE);


                }





                if block == Block::MAGMA_BLOCK {


                    let _ = live.damage(1.0, DamageType::HOT_FLOOR);


                }


            }


        }


        */
    }

    pub fn tick_block_collisions(&self, caller: &dyn EntityBase) -> bool {
        if !self.is_affected_by_blocks() {
            return false;
        }

        let bounding_box = self.bounding_box.load();
        let aabb = bounding_box.expand(-1.0e-7, -1.0e-7, -1.0e-7);

        let min = aabb.min_block_pos();
        let max = aabb.max_block_pos();

        let eye_height = self.get_eye_height();
        let eye_width = f64::from(self.width()) * 0.8;
        let mut eye_level_box = aabb;
        let shrink_x = (aabb.max.x - aabb.min.x - eye_width) / 2.0;
        let shrink_z = (aabb.max.z - aabb.min.z - eye_width) / 2.0;
        eye_level_box.min.x += shrink_x;
        eye_level_box.max.x -= shrink_x;
        eye_level_box.min.z += shrink_z;
        eye_level_box.max.z -= shrink_z;
        eye_level_box.min.y += eye_height;
        eye_level_box.max.y = eye_level_box.min.y;

        let mut suffocating = false;
        let world = self.world.load();

        for pos in BlockPos::iterate(min, max) {
            let (block, state) = world.get_block_and_state(&pos);
            if state.is_air() {
                continue;
            }

            let check_suffocation = !suffocating && state.is_suffocating();

            World::check_collision(
                &bounding_box,
                pos,
                state,
                check_suffocation,
                |collision_shape: &BoundingBox| {
                    if collision_shape.intersects(&eye_level_box) {
                        suffocating = true;
                    }
                },
            );

            let collision_shape = if block == &Block::POWDER_SNOW {
                crate::block::blocks::powder_snow::inside_collision_shape_for_entity(caller, &pos)
            } else {
                world
                    .block_registry
                    .get_inside_collision_shape(block, &world, state, &pos)
            };

            if bounding_box.intersects(&collision_shape.at_pos(pos)) {
                if block == &Block::POWDER_SNOW {
                    self.is_in_powder_snow.store(true, Relaxed);
                }
                if let Some(server_arc) = world.server.upgrade() {
                    world.block_registry.on_entity_collision(
                        block,
                        &world,
                        caller,
                        &pos,
                        state,
                        &server_arc,
                    );
                }
            }
        }

        suffocating
    }

    pub fn update_last_pos(&self) -> Vector3<f64> {
        let pos = self.pos.load();
        let old = self.last_pos.load();
        // A player's movement is its last client move, set by the move handlers
        if self.entity_type != &EntityType::PLAYER {
            self.movement.store(pos - old);
        }
        self.last_pos.store(pos);
        old
    }

    /// Vanilla `Entity.push(Vec3)`. A player's pushes are also kept apart for Bedrock,
    /// whose client doesn't predict them.
    pub fn push_by(&self, impulse: Vector3<f64>) {
        self.velocity.store(self.velocity.load() + impulse);
        self.velocity_dirty.store(true, Ordering::SeqCst);
        if self.entity_type == &EntityType::PLAYER {
            self.push_impulse.store(self.push_impulse.load() + impulse);
        }
    }

    /// `push_by` on the pusher in its own `pushEntities`, which stays off the Bedrock impulse.
    pub fn push_back_by(&self, impulse: Vector3<f64>) {
        self.velocity.store(self.velocity.load() + impulse);
        self.velocity_dirty.store(true, Ordering::SeqCst);
    }

    /// Vanilla `getKnownMovement`: a player's own client motion, otherwise the velocity.
    pub fn known_movement(&self) -> Vector3<f64> {
        if self.entity_type == &EntityType::PLAYER {
            self.movement.load()
        } else {
            self.velocity.load()
        }
    }

    // updateWaterState() in yarn

    fn update_fluid_state(&self, caller: &dyn EntityBase) {
        let world = self.world.load();
        for fluid in self.update_fluid_interaction(caller) {
            world
                .block_registry
                .on_entity_collision_fluid(fluid, caller);
        }
    }

    /// Vanilla `Entity.updateFluidInteraction`: fluid flags and current push only, without the
    /// fluid's `entityInside` effects. Returns the fluids the entity is in.
    fn update_fluid_interaction(&self, caller: &dyn EntityBase) -> Vec<&'static Fluid> {
        let is_pushed = caller.is_pushed_by_fluids();
        let mut fluids = BTreeMap::new();
        let mut fluid_push = [Vector3::default(); 2];
        let mut fluid_n = [0; 2];
        let mut in_fluid = [false, false];
        let mut fluid_height: [f64; 2] = [0.0; 2];

        let entity_box = self.bounding_box.load();
        let bounding_box = entity_box.expand(-0.001, -0.001, -0.001);
        let boat = self.get_vehicle();
        let bounding_box = if let Some(boat) = boat.as_ref().and_then(|vehicle| {
            vehicle
                .cast_any()
                .downcast_ref::<vehicle::boat::BoatEntity>()
        }) {
            let Some(bounds) = boat.modify_passenger_fluid_interaction_box(bounding_box) else {
                self.water_height.store(0.0);
                self.lava_height.store(0.0);
                self.touching_water.store(false, Relaxed);
                self.touching_lava.store(false, Relaxed);
                return Vec::new();
            };
            bounds
        } else {
            bounding_box
        };

        let min = bounding_box.min_block_pos();

        let max = BlockPos::new(
            bounding_box.max.x.ceil() as i32 - 1,
            bounding_box.max.y.ceil() as i32 - 1,
            bounding_box.max.z.ceil() as i32 - 1,
        );

        let world = self.world.load();

        for x in min.0.x..=max.0.x {
            for y in min.0.y..=max.0.y {
                for z in min.0.z..=max.0.z {
                    let pos = BlockPos::new(x, y, z);

                    let (fluid, state) = world.get_fluid_and_fluid_state(&pos);

                    if fluid.id != Fluid::EMPTY.id {
                        let surface_y =
                            f64::from(world.get_fluid_height(&pos, fluid, &state)) + f64::from(y);

                        if surface_y >= bounding_box.min.y {
                            let marginal_height = surface_y - entity_box.min.y;
                            let i = usize::from(fluid.matches_type(&Fluid::LAVA));

                            fluid_height[i] = fluid_height[i].max(marginal_height);

                            in_fluid[i] = true;
                            let family = fluid.to_flowing();
                            fluids.insert(family.id, family);

                            if !is_pushed {
                                continue;
                            }

                            let mut fluid_velo = world.get_fluid_velocity(pos, fluid, &state);

                            if fluid_height[i] < 0.4 {
                                fluid_velo = fluid_velo * fluid_height[i];
                            }

                            fluid_push[i] += fluid_velo;

                            fluid_n[i] += 1;
                        }
                    }
                }
            }
        }

        let [water_height, lava_height] = fluid_height;
        let [in_water, in_lava] = in_fluid;

        // Java refreshes the fluid depths before notifying the entry splash.
        let previous_water_height = self.water_height.swap(water_height);
        let previous_depths = [previous_water_height, self.lava_height.swap(lava_height)];

        if in_water {
            if let Some(living) = caller.get_living_entity() {
                living.fall_distance.store(0.0);
            }

            if !self.touching_water.load(Ordering::SeqCst)
                && !self.first_tick.load(Ordering::Relaxed)
            {
                let position = self.pos.load();
                let motion_epoch = self.vehicle_motion_epoch();
                caller.do_water_splash_effect(caller);
                if !self.water_contact_is_current(&world, position, motion_epoch) {
                    self.discard_obsolete_fluid_depths(fluid_height, previous_depths);
                    return Vec::new();
                }
            }
        }

        self.touching_water.store(in_water, Ordering::SeqCst);

        if in_lava && let Some(living) = caller.get_living_entity() {
            let halved_fall = living.fall_distance.load() / 2.0;

            if halved_fall != 0.0 {
                living.fall_distance.store(halved_fall);
            }
        }

        self.touching_lava.store(in_lava, Ordering::SeqCst);

        // Java dispatches the entry splash and publishes the water transition before applying currents.
        self.push_by_fluid(WATER_PUSH_SPEED, fluid_push[0], fluid_n[0]);
        self.push_by_fluid(
            lava_push_speed(world.dimension.fast_lava),
            fluid_push[1],
            fluid_n[1],
        );

        // BTreeMap auto-sorts water before lava as in vanilla
        fluids.into_values().collect()
    }

    /// Withdraw the obsolete sample while preserving a fluid update made by the callback.
    fn discard_obsolete_fluid_depths(
        &self,
        [water_height, lava_height]: [f64; 2],
        [previous_water_height, previous_lava_height]: [f64; 2],
    ) {
        if !self.touching_water.load(Relaxed)
            && self.water_height.load().to_bits() == water_height.to_bits()
        {
            self.water_height.store(previous_water_height);
            if self.lava_height.load().to_bits() == lava_height.to_bits() {
                self.lava_height.store(previous_lava_height);
            }
        }
    }

    fn push_by_fluid(&self, speed: f64, mut push: Vector3<f64>, n: usize) {
        const MIN_CURRENT_SPEED: f64 = 0.003;
        const MIN_CURRENT_IMPULSE: f64 = MIN_CURRENT_SPEED * 1.5;
        if n == 0 || push.length_squared() < f64::from(1.0e-5f32) {
            return;
        }
        push = if self.entity_type == &EntityType::PLAYER {
            push * (1.0 / n as f64)
        } else {
            push / push.length()
        };
        push = push * speed;
        let velocity = self.velocity.load();
        let impulse_length = push.length();
        if velocity.x.abs() < MIN_CURRENT_SPEED
            && velocity.z.abs() < MIN_CURRENT_SPEED
            && impulse_length < MIN_CURRENT_IMPULSE
        {
            push = if impulse_length < f64::from(1.0e-5f32) {
                Vector3::default()
            } else {
                push / impulse_length * MIN_CURRENT_IMPULSE
            };
        }
        self.velocity.store(velocity + push);
    }

    fn get_pos_with_y_offset(
        &self,
        offset: f64,
    ) -> (
        BlockPos,
        Option<&'static Block>,
        Option<&'static BlockState>,
    ) {
        if let Some(mut supporting_block) = self.supporting_block_pos.load() {
            if offset > 1.0e-5 {
                let (block, state) = self.world.load().get_block_and_state(&supporting_block);

                // if let Some(props) = block.properties(state.id) {
                //     let name = props.;

                //     if offset <= 0.5
                //         && (name == "OakFenceLikeProperties"
                //             || name == "ResinBrickWallLikeProperties"
                //             || name == "OakFenceGateLikeProperties"
                //                 && OakFenceGateLikeProperties::from_state_id(state.id)
                //                     .r#open)
                //     {
                //         return (supporting_block, Some(block), Some(state));
                //     }
                // }

                supporting_block.0.y = (self.pos.load().y - offset).floor() as i32;

                return (supporting_block, Some(block), Some(state));
            }

            return (supporting_block, None, None);
        }

        let mut block_pos = self.block_pos.load();

        block_pos.0.y = (self.pos.load().y - offset).floor() as i32;

        (block_pos, None, None)
    }

    fn get_block_with_y_offset(
        &self,
        offset: f64,
    ) -> (BlockPos, &'static Block, &'static BlockState) {
        let (pos, block, state) = self.get_pos_with_y_offset(offset);

        if let (Some(b), Some(s)) = (block, state) {
            (pos, b, s)
        } else {
            let (b, s) = self.world.load().get_block_and_state(&pos);

            (pos, b, s)
        }
    }

    // Entity.updateVelocity in yarn

    fn update_velocity_from_input(&self, movement_input: Vector3<f64>, speed: f64) {
        let final_input = self.movement_input_to_velocity(movement_input, speed);

        self.velocity.store(self.velocity.load() + final_input);
    }

    // Entity.movementInputToVelocity in yarn

    fn movement_input_to_velocity(&self, movement_input: Vector3<f64>, speed: f64) -> Vector3<f64> {
        let yaw = f64::from(self.yaw.load()).to_radians();

        let dist = movement_input.length_squared();

        if dist < 1.0e-7 {
            return Vector3::default();
        }

        let input = if dist > 1.0 {
            movement_input.normalize() * speed
        } else {
            movement_input * speed
        };

        let sin = yaw.sin();

        let cos = yaw.cos();

        Vector3::new(
            input.x.mul_add(cos, -(input.z * sin)),
            input.y,
            input.z.mul_add(cos, input.x * sin),
        )
    }

    #[must_use]
    pub fn get_on_pos(&self) -> BlockPos {
        self.get_pos_with_y_offset(1.0e-5).0
    }

    #[must_use]
    pub fn get_block_pos_below_that_affects_my_movement(&self) -> BlockPos {
        self.get_pos_with_y_offset(0.500_001).0
    }

    #[must_use]
    #[expect(clippy::float_cmp)]
    pub fn get_block_speed_factor(&self) -> f32 {
        let world = self.world.load();
        let (block, _state) = world.get_block_and_state(&self.block_pos.load());
        let speed_factor_here = block.get_speed_factor();
        if block != &Block::WATER && block != &Block::BUBBLE_COLUMN {
            if speed_factor_here == 1.0 {
                let below_pos = self.get_block_pos_below_that_affects_my_movement();
                let (below_block, _below_state) = world.get_block_and_state(&below_pos);
                below_block.get_speed_factor()
            } else {
                speed_factor_here
            }
        } else {
            speed_factor_here
        }
    }

    #[expect(clippy::float_cmp)]
    fn get_jump_velocity_multiplier(&self) -> f32 {
        let f = self
            .world
            .load()
            .get_block(&self.block_pos.load())
            .jump_velocity_multiplier;

        let g = self
            .get_block_with_y_offset(0.500_001)
            .1
            .jump_velocity_multiplier;

        if f == 1f32 { g } else { f }
    }

    pub fn move_pos(&self, delta: Vector3<f64>) {
        self.set_pos(self.pos.load() + delta);
    }

    /// Resolves and applies motion without sending a position packet.
    /// Returns false after removal or a callback takes over a vehicle's movement.
    pub fn move_entity(&self, caller: &dyn EntityBase, motion: Vector3<f64>) -> bool {
        if caller.get_player().is_some() {
            return true;
        }

        if self.no_physics.load(Ordering::Relaxed) {
            self.move_pos(motion);
            self.horizontal_collision.store(false, Ordering::Relaxed);
            self.vertical_collision.store(false, Ordering::Relaxed);
            self.vertical_collision_below
                .store(false, Ordering::Relaxed);
            self.on_ground.store(false, Ordering::Relaxed);

            return true;
        }

        let motion = self.apply_stuck_speed_multiplier(motion);
        let final_move = self.adjust_movement_for_collisions(motion, caller);
        let records_vehicle_movement =
            self.vehicle_movements.is_some() && caller.vehicle_contact_tick_reach().is_some();
        if !records_vehicle_movement
            || self.record_vehicle_movement(self.pos.load(), final_move, motion)
        {
            self.move_pos(final_move);
        }
        let vehicle_move = self.vehicle_movements.as_ref().map(|_| {
            (
                self.pos.load(),
                self.world.load_full(),
                self.vehicle_motion_epoch(),
            )
        });
        let still_current = || {
            !self.is_removed()
                && vehicle_move.as_ref().is_none_or(|(pos, world, revision)| {
                    self.pos.load() == *pos
                        && Arc::ptr_eq(world, &self.world.load())
                        && self.vehicle_motion_epoch() == *revision
                })
        };

        let vehicle_air_drag = caller.vehicle_bounce_air_drag();
        if vehicle_air_drag.is_none() {
            let velocity = self.velocity_after_horizontal_clipping(motion, final_move);
            self.apply_movement_speed_factor(caller, velocity);
        }

        if let Some(living) = caller.get_living_entity() {
            let on_ground = self.on_ground.load(Ordering::SeqCst);
            living.fall(caller, final_move.y, on_ground);
        }

        let (effect_pos, effect_block, effect_state) = self.get_block_with_y_offset(0.2);
        let boat = caller
            .cast_any()
            .downcast_ref::<vehicle::boat::BoatEntity>();
        if let Some(boat) = boat {
            boat.update_last_vertical_speed();
        }
        if let Some(air_drag) = vehicle_air_drag {
            self.restitute_vehicle_movement(
                caller,
                effect_block,
                motion,
                final_move,
                air_drag,
                &still_current,
            );
        } else if motion.y != final_move.y {
            if motion.y < 0.0 {
                let world = self.world.load();
                world
                    .block_registry
                    .update_entity_movement_after_fall_on(effect_block, caller);
            } else {
                // Vanilla `restituteMovementAfterCollisions`: block bounciness only applies when landing.
                // TODO vanilla also applies the `BOUNCINESS` attribute (+ sulfur cubes) on
                // walls and ceilings
                stop_vertical_movement_after_fall(caller);
            }
        }

        if still_current() {
            self.emit_movement(caller, final_move, &effect_pos, effect_state);
        }
        if still_current() {
            if vehicle_air_drag.is_some() {
                self.apply_movement_speed_factor(caller, self.velocity.load());
            }
            return true;
        }
        if vehicle_move.is_some_and(|(_, _, revision)| self.vehicle_motion_epoch() == revision) {
            // Raw position changes from a listener do not go through Entity::teleport.
            self.clear_vehicle_movement();
        }
        false
    }

    fn velocity_after_horizontal_clipping(
        &self,
        requested: Vector3<f64>,
        resolved: Vector3<f64>,
    ) -> Vector3<f64> {
        let mut velocity = self.velocity.load();
        if (requested.x - resolved.x).abs() >= 1.0e-5 {
            velocity.x = 0.0;
        }
        if (requested.z - resolved.z).abs() >= 1.0e-5 {
            velocity.z = 0.0;
        }
        velocity
    }

    fn apply_movement_speed_factor(&self, caller: &dyn EntityBase, velocity: Vector3<f64>) {
        let factor = f64::from(caller.get_block_speed_factor());
        self.velocity.store(velocity.multiply(factor, 1.0, factor));
    }

    /// Java publishes the resolved velocity and requests tracking after the bounce callbacks.
    fn restitute_vehicle_movement(
        &self,
        caller: &dyn EntityBase,
        block: &Block,
        motion: Vector3<f64>,
        clipped: Vector3<f64>,
        air_drag: f64,
        still_current: &impl Fn() -> bool,
    ) {
        let mut velocity = self.velocity_after_horizontal_clipping(motion, clipped);
        let mut bounced = false;
        if motion.y != clipped.y {
            let gravity = caller.get_gravity();
            let restitution = if velocity.y < -gravity
                && self.vertical_collision_below.load(Relaxed)
                && !self.is_sneaking()
                && !block.has_tag(&tag::Block::MINECRAFT_SUPPRESSES_BOUNCE)
            {
                f64::from(block.bounce_restitution * 0.8f32)
            } else {
                0.0
            };
            bounced = restitution > 0.0;
            velocity.y = if bounced {
                let fraction = clipped.y / velocity.y;
                let effective_drag = 1.0 + fraction * (air_drag - 1.0);
                (fraction * gravity - velocity.y) * effective_drag * restitution
            } else {
                0.0
            };
        }
        if bounced {
            let support = self.get_block_pos_below_that_affects_my_movement();
            self.world
                .load()
                .block_registry
                .bounce_on(block, caller, &support);
            if still_current() {
                self.emit_own_game_event(caller, GameEvent::Bounce);
            }
        }
        if still_current() {
            self.velocity.store(velocity);
            if bounced {
                self.sync_position.store(true, Ordering::SeqCst);
            }
        }
    }

    /// Vanilla consumes the current inside-block movement modifier once per physical move.
    pub(crate) fn apply_stuck_speed_multiplier(&self, motion: Vector3<f64>) -> Vector3<f64> {
        if self.no_physics.load(Relaxed) {
            return motion;
        }
        let multiplier = self.movement_multiplier.swap(Vector3::default());
        if multiplier.length_squared() <= 1.0e-7 {
            return motion;
        }
        self.velocity.store(Vector3::default());
        motion.multiply(multiplier.x, multiplier.y, multiplier.z)
    }

    /// The emission part of vanilla `Entity.move` for movement the client already resolved,
    /// which is how players move.
    pub fn emit_client_movement(&self, caller: &dyn EntityBase, movement: Vector3<f64>) {
        let (effect_pos, _, effect_state) = self.get_block_with_y_offset(0.2);
        self.emit_movement(caller, movement, &effect_pos, effect_state);
    }

    fn emit_movement(
        &self,
        caller: &dyn EntityBase,
        movement: Vector3<f64>,
        effect_pos: &BlockPos,
        effect_state: &'static BlockState,
    ) {
        let emission = caller.movement_emission();
        if emission.emits_anything() && !caller.is_passenger() {
            self.apply_movement_emission(caller, emission, movement, effect_pos, effect_state);
        }
    }

    /// Vanilla `applyMovementEmissionAndPlaySound`.
    fn apply_movement_emission(
        &self,
        caller: &dyn EntityBase,
        emission: MovementEmission,
        movement: Vector3<f64>,
        effect_pos: &BlockPos,
        effect_state: &'static BlockState,
    ) {
        let moved_distance = (movement.length() * 0.6) as f32;
        let horizontal_moved_distance = (movement.horizontal_length() * 0.6) as f32;
        let supporting_pos = self.get_pos_with_y_offset(1.0e-5).0;
        let (supporting_block, supporting_state) =
            self.world.load().get_block_and_state(&supporting_pos);
        let climbing = Self::is_state_climbable(supporting_block);
        self.move_dist.store(
            self.move_dist.load()
                + if climbing {
                    moved_distance
                } else {
                    horizontal_moved_distance
                },
        );
        self.fly_dist.store(self.fly_dist.load() + moved_distance);

        if self.move_dist.load() > self.next_step.load() && !supporting_state.is_air() {
            let only_effect_state = supporting_pos == *effect_pos;
            let mut produced_side_effects = self.vibration_and_sound_effects_from_block(
                caller,
                effect_pos,
                effect_state,
                emission.emits_sounds(),
                only_effect_state,
                movement,
            );
            if !only_effect_state {
                produced_side_effects |= self.vibration_and_sound_effects_from_block(
                    caller,
                    &supporting_pos,
                    supporting_state,
                    false,
                    emission.emits_events(),
                    movement,
                );
            }

            if produced_side_effects {
                self.next_step.store(caller.next_step());
            } else if self.is_in_water() {
                self.next_step.store(caller.next_step());
                if emission.emits_sounds() {
                    self.water_swim_sound(caller);
                }
                if emission.emits_events() {
                    self.emit_own_game_event(caller, GameEvent::Swim);
                }
            }
        } else if supporting_state.is_air() && caller.is_flapping() {
            // Vanilla `processFlappingMovement`.
            caller.on_flap();
            if emission.emits_events() {
                self.emit_own_game_event(caller, GameEvent::Flap);
            }
        }
    }

    /// Vanilla `isStateClimbable`.
    fn is_state_climbable(block: &Block) -> bool {
        block.has_tag(&tag::Block::MINECRAFT_CLIMBABLE) || block == &Block::POWDER_SNOW
    }

    /// Vanilla `vibrationAndSoundEffectsFromBlock`.
    fn vibration_and_sound_effects_from_block(
        &self,
        caller: &dyn EntityBase,
        pos: &BlockPos,
        state: &'static BlockState,
        should_sound: bool,
        should_vibrate: bool,
        movement: Vector3<f64>,
    ) -> bool {
        if state.is_air() {
            return false;
        }
        let climbable = Self::is_state_climbable(state.id.to_block());
        let crouching = self.pose.load() == EntityPose::Crouching;
        if !(self.on_ground.load(Ordering::Relaxed)
            || climbable
            || crouching && movement.y == 0.0
            || caller.is_on_rails())
            || self.is_swimming()
        {
            return false;
        }
        if should_sound {
            caller.play_step_sound(pos, state);
            if self.should_play_amethyst_step_sound(state) {
                self.play_amethyst_step_sound(caller);
            }
        }
        if should_vibrate {
            self.emit_own_game_event(caller, GameEvent::Step);
        }
        true
    }

    fn should_play_amethyst_step_sound(&self, state: &BlockState) -> bool {
        state
            .id
            .to_block()
            .has_tag(&tag::Block::MINECRAFT_CRYSTAL_SOUND_BLOCKS)
            && self.tick_count.load(Ordering::Relaxed)
                >= self.last_crystal_sound_play_tick.load(Ordering::Relaxed) + 20
    }

    /// Vanilla `playAmethystStepSound`.
    fn play_amethyst_step_sound(&self, caller: &dyn EntityBase) {
        let tick_count = self.tick_count.load(Ordering::Relaxed);
        let ticks_since = tick_count - self.last_crystal_sound_play_tick.load(Ordering::Relaxed);
        let intensity = (self.crystal_sound_intensity.load() * 0.997f64.powi(ticks_since) as f32
            + 0.07)
            .min(1.0);
        self.crystal_sound_intensity.store(intensity);
        let pitch = (intensity * rand::random::<f32>()).mul_add(1.2, 0.5);
        let volume = intensity.mul_add(1.2, 0.1);
        caller.play_entity_sound(Sound::BlockAmethystBlockChime, volume, pitch);
        self.last_crystal_sound_play_tick
            .store(tick_count, Ordering::Relaxed);
    }

    /// Vanilla `getPrimaryStepSoundBlockPos`: the block above when it sounds from inside, like snow.
    pub fn get_primary_step_sound_block_pos(&self, affecting_pos: &BlockPos) -> BlockPos {
        let above_pos = affecting_pos.up();
        let above = self.world.load().get_block_state(&above_pos).id.to_block();
        if above.has_tag(&tag::Block::MINECRAFT_INSIDE_STEP_SOUND_BLOCKS)
            || above.has_tag(&tag::Block::MINECRAFT_COMBINATION_STEP_SOUND_BLOCKS)
        {
            above_pos
        } else {
            *affecting_pos
        }
    }

    /// Vanilla `waterSwimSound`: louder the faster the entity, or its controlling rider, swims.
    pub fn water_swim_sound(&self, caller: &dyn EntityBase) {
        let rider = caller.get_controlling_passenger();
        let (velocity, volume_modifier) = rider.as_ref().map_or_else(
            || (self.velocity.load(), 0.35),
            |rider| (rider.get_entity().velocity.load(), 0.4),
        );
        let speed = (velocity.x * velocity.x * 0.2
            + velocity.y * velocity.y
            + velocity.z * velocity.z * 0.2)
            .sqrt() as f32;
        caller.play_swim_sound((speed * volume_modifier).min(1.0));
    }

    /// The base `Entity.doWaterSplashEffect`. Clients spawn the bubble and splash particles
    /// themselves.
    pub fn water_splash_effect(&self, caller: &dyn EntityBase) {
        let rider = caller.get_controlling_passenger();
        let (velocity, volume_modifier) = rider.as_ref().map_or_else(
            || (self.velocity.load(), 0.2),
            |rider| (rider.get_entity().velocity.load(), 0.9),
        );
        let speed = ((velocity.x * velocity.x * 0.2
            + velocity.y * velocity.y
            + velocity.z * velocity.z * 0.2)
            .sqrt() as f32
            * volume_modifier)
            .min(1.0);
        let sound = if speed < 0.25 {
            caller.get_swim_splash_sound()
        } else {
            caller.get_swim_high_speed_splash_sound()
        };
        if let Some(sound) = sound {
            let world = self.world.load();
            let position = self.pos.load();
            let motion_epoch = self.vehicle_motion_epoch();
            let pitch = (rand::random::<f32>() - rand::random::<f32>()).mul_add(0.4, 1.0);
            caller.play_entity_sound(sound, speed, pitch);
            if !self.water_contact_is_current(&world, position, motion_epoch) {
                return;
            }
        }
        self.emit_own_game_event(caller, GameEvent::Splash);
    }

    /// A splash callback may supersede the sampled contact by relocating or removing the entity.
    fn water_contact_is_current(
        &self,
        world: &Arc<World>,
        position: Vector3<f64>,
        motion_epoch: u64,
    ) -> bool {
        !self.is_removed()
            && self.pos.load() == position
            && Arc::ptr_eq(world, &self.world.load())
            && self.vehicle_motion_epoch() == motion_epoch
    }

    /// Vanilla `gameEvent(event)`: at the entity, with itself as the source.
    pub(crate) fn emit_own_game_event(&self, caller: &dyn EntityBase, event: GameEvent) {
        self.world.load().emit_game_event_with_context(
            event.name(),
            self.pos.load(),
            &crate::world::game_event::GameEventContext {
                source_entity: Some(caller),
            },
        );
    }

    pub fn push_out_of_blocks(&self, center_pos: Vector3<f64>) {
        let block_pos = BlockPos::floored_v(center_pos);

        let delta = center_pos.sub(&block_pos.0.to_f64());

        let mut min_dist = f64::MAX;

        let mut direction = BlockDirection::Up;

        for dir in BlockDirection::all() {
            if dir == BlockDirection::Down {
                continue;
            }

            let offset = dir.to_offset();

            if self
                .world
                .load()
                .get_block_state(&block_pos.offset(offset))
                .is_full_cube()
            {
                continue;
            }

            let component = delta.get_axis(dir.to_axis().into());

            let dist = if dir.positive() {
                1.0 - component
            } else {
                component
            };

            if dist < min_dist {
                min_dist = dist;

                direction = dir;
            }
        }

        // TODO: vanilla rolls from the entity `random`; same odds, not seed-reproducible.
        let amplitude = rand::random::<f64>().mul_add(PUSH_OUT_RANDOM_SPEED, PUSH_OUT_MIN_SPEED);

        let axis = direction.to_axis().into();

        let sign = if direction.positive() { 1.0 } else { -1.0 };

        let mut velo = self.velocity.load();

        velo = velo * 0.75;

        velo.set_axis(axis, sign * amplitude);

        self.velocity.store(velo);
    }

    fn tick_portal(&self, caller: &dyn EntityBase) {
        if self.portal_cooldown.load(Ordering::Relaxed) > 0 {
            self.portal_cooldown.fetch_sub(1, Ordering::Relaxed);
        }
        let Ok(mut manager_guard) = self.portal_manager.try_lock() else {
            return;
        };
        let mut should_remove = false;
        if let Some(portal_processor) = manager_guard.as_mut() {
            if portal_processor.process_portal_teleportation(&self.world.load(), caller, true) {
                self.portal_cooldown
                    .store(self.default_portal_cooldown(), Ordering::Relaxed);

                let world_clone = self.world.load_full();
                let portal_type = portal_processor.portal_type;
                let dest_world_opt = portal_processor.destination_world.clone();
                let src_portal = portal_processor.source_portal.clone();
                let entity_id = self.entity_id;
                let yaw = self.yaw.load();

                if let Some(server) = world_clone.server.upgrade() {
                    server.spawn_task(async move {
                        let Some(entity_arc) = world_clone.get_entity_by_id(entity_id) else {
                            return;
                        };
                        let transition = portal_type
                            .get_portal_destination(
                                &world_clone,
                                dest_world_opt,
                                entity_arc.as_ref(),
                                src_portal.as_ref(),
                            )
                            .await;

                        if let Some(transition) = transition {
                            let dest_world = transition.new_world.clone();
                            let yaw_val = transition.yaw;
                            let pitch = transition.pitch;
                            let teleport_pos = transition.position;

                            // Teleport the main entity
                            entity_arc.teleport(teleport_pos, yaw_val, pitch, dest_world.clone());

                            // Teleport all passengers recursively along with the vehicle
                            let yaw_delta = yaw_val.map(|y| y - yaw);
                            Self::teleport_passengers_recursive(
                                entity_arc.get_entity(),
                                teleport_pos,
                                yaw_delta,
                                &dest_world,
                            );
                        }
                    });
                }
            } else if portal_processor.portal_time == 0 {
                should_remove = true;
            }
        }
        if should_remove {
            *manager_guard = None;
        }
    }

    /// Recursively teleports all passengers (and their passengers) to the destination
    fn teleport_passengers_recursive(
        entity: &Self,
        position: Vector3<f64>,
        yaw_delta: Option<f32>,
        dest_world: &Arc<World>,
    ) {
        let passengers = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for passenger in passengers {
            let passenger_entity = passenger.get_entity();
            let passenger_yaw = yaw_delta.map(|delta| passenger_entity.yaw.load() + delta);
            passenger_entity.portal_cooldown.store(
                passenger_entity.default_portal_cooldown(),
                Ordering::Relaxed,
            );

            // Get nested passengers before teleporting
            let nested_passengers = passenger_entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();

            passenger.teleport(position, passenger_yaw, None, dest_world.clone());

            // Recursively teleport nested passengers
            for nested in nested_passengers {
                let nested_entity = nested.get_entity();
                Self::teleport_passengers_recursive(nested_entity, position, yaw_delta, dest_world);
            }
        }
    }

    pub fn try_use_portal(&self, portal_world: Arc<World>, pos: BlockPos) {
        let mut portal_event =
            crate::plugin::api::events::entity::entity_portal::EntityPortalEvent::new(
                self.entity_id,
                pos,
            );
        if let Some(server) = self.world.load().server.upgrade() {
            portal_event = server.plugin_manager.fire_blocking(&server, portal_event);
        }
        if portal_event.cancelled {
            return;
        }

        // Passengers don't teleport independently - they wait for their vehicle
        if self.has_vehicle() {
            return;
        }

        if self.portal_cooldown.load(Ordering::Relaxed) > 0 {
            self.portal_cooldown
                .store(self.default_portal_cooldown(), Ordering::Relaxed);
            return;
        }

        let Some(server) = portal_world.server.upgrade() else {
            return;
        };

        if (portal_world.dimension == Dimension::THE_NETHER && !server.basic_config.allow_nether)
            || (portal_world.dimension == Dimension::THE_END && !server.basic_config.allow_end)
        {
            return;
        }

        let mut manager = self
            .portal_manager
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let world = self.world.load();
        if manager.is_none() {
            let portal_type = if portal_world.dimension == Dimension::THE_END
                || self.world.load().dimension == Dimension::THE_END
            {
                PortalType::End
            } else {
                PortalType::Nether
            };

            let mut new_manager = PortalProcessor::new(portal_type, pos, portal_world);

            let (block, state) = world.get_block_and_state(&pos);
            let source_axis = (block == &pumpkin_data::Block::NETHER_PORTAL).then(|| {
                let props = <pumpkin_data::block_properties::NetherPortalLikeProperties as pumpkin_data::block_properties::BlockProperties>::from_state_id(state.id, block);
                props.axis
            });

            if let Some(axis) = source_axis
                && let Some(portal) = NetherPortal::get_on_axis(&world, &pos, axis)
                && portal.was_already_valid()
            {
                new_manager.set_source_portal(SourcePortalInfo {
                    lower_corner: portal.lower_corner(),
                    axis: portal.axis(),
                    width: portal.width(),
                    height: portal.height(),
                });
            }

            *manager = Some(new_manager);
        } else if let Some(manager) = manager.as_mut() {
            manager.entry_position = pos;
            manager.inside_portal_this_tick = true;
            if manager.source_portal.is_none() {
                let (block, state) = world.get_block_and_state(&pos);
                if block == &pumpkin_data::Block::NETHER_PORTAL {
                    let props = <pumpkin_data::block_properties::NetherPortalLikeProperties as pumpkin_data::block_properties::BlockProperties>::from_state_id(state.id, block);
                    if let Some(portal) = NetherPortal::get_on_axis(&world, &pos, props.axis)
                        && portal.was_already_valid()
                    {
                        manager.set_source_portal(SourcePortalInfo {
                            lower_corner: portal.lower_corner(),
                            axis: portal.axis(),
                            width: portal.width(),
                            height: portal.height(),
                        });
                    }
                }
            }
        }
    }

    /// Extinguishes this entity.
    pub fn extinguish(&self) {
        self.fire_ticks.store(0, Ordering::Relaxed);
    }

    /// Maximum freeze ticks (7 seconds at 20 tps)
    pub const MAX_FROZEN_TICKS: i32 = 140;

    /// Freeze damage is dealt every 40 ticks when fully frozen
    const FREEZE_DAMAGE_INTERVAL: i32 = 40;

    /// Check if the entity is currently in powder snow.
    ///
    /// The flag is reset at the start of each tick and set while processing
    /// block collisions for the current tick.
    pub fn is_in_powder_snow(&self) -> bool {
        self.is_in_powder_snow.load(Ordering::Relaxed)
    }

    /// Check if this entity type is immune to freezing
    pub fn is_freeze_immune(&self) -> bool {
        self.entity_type
            .has_tag(&tag::EntityType::MINECRAFT_FREEZE_IMMUNE_ENTITY_TYPES)
    }

    /// Mirrors vanilla `LivingEntity#canFreeze`: spectators and entities wearing
    /// freeze-immune wearables (e.g. leather armor) cannot freeze.
    fn can_freeze(&self, caller: &dyn EntityBase) -> bool {
        if caller.get_mob().is_some_and(|mob| !mob.mob_can_freeze()) {
            return false;
        }

        if caller.is_spectator() || self.is_freeze_immune() {
            return false;
        }

        let Some(living) = caller.get_living_entity() else {
            return true;
        };

        if let Ok(equipment) = living.entity_equipment.try_lock() {
            for (slot, stack) in &equipment.equipment {
                if (*slot == EquipmentSlot::HEAD
                    || *slot == EquipmentSlot::CHEST
                    || *slot == EquipmentSlot::LEGS
                    || *slot == EquipmentSlot::FEET)
                    && stack
                        .get_item()
                        .has_tag(&tag::Item::MINECRAFT_FREEZE_IMMUNE_WEARABLES)
                {
                    return false;
                }
            }
        }

        true
    }

    /// Ticks the frozen state of the entity.
    /// In powder snow and freezeable: `frozen_ticks` increases by 1 (up to `MAX_FROZEN_TICKS`)
    /// Otherwise: `frozen_ticks` decreases by 2 (down to 0)
    /// When fully frozen, deals 1 damage every 40 ticks
    pub fn tick_frozen(&self, caller: &dyn EntityBase) {
        let can_freeze = self.can_freeze(caller);
        let in_powder_snow = self.is_in_powder_snow();
        let old_frozen_ticks = self.frozen_ticks.load(Ordering::Relaxed);

        let new_frozen_ticks = if in_powder_snow && can_freeze {
            // Increase frozen ticks when in powder snow
            (old_frozen_ticks + 1).min(Self::MAX_FROZEN_TICKS)
        } else {
            // Vanilla: thaw whenever not in powder snow OR when freezing is prevented
            (old_frozen_ticks - 2).max(0)
        };

        // Only update and send metadata if the value changed
        if new_frozen_ticks != old_frozen_ticks {
            self.frozen_ticks.store(new_frozen_ticks, Ordering::Relaxed);
            let mut bedrock_meta = SyncedActorDataList::new();
            bedrock_meta.set(
                entity_data_key::FREEZING_EFFECT_STRENGTH,
                MetadataValue::Float(new_frozen_ticks as f32),
            );
            self.set_synced_data(
                tracked_data::entity::DATA_TICKS_FROZEN,
                VarInt(new_frozen_ticks),
            );
            self.send_bedrock_actor_data(&bedrock_meta);
        }

        // Vanilla parity: full-freeze damage is tick-phase based.
        if can_freeze
            && new_frozen_ticks >= Self::MAX_FROZEN_TICKS
            && self.tick_count.load(Ordering::Relaxed) % Self::FREEZE_DAMAGE_INTERVAL == 0
        {
            let world = self.world.load_full();
            if world.level_info.load().game_rules.freeze_damage
                && let Some(entity) = world.get_entity_by_id(self.entity_id)
            {
                entity.damage(entity.as_ref(), 1.0, DamageType::FREEZE);
            }
        }
    }

    /// Sets the number of ticks the entity has been frozen.
    pub fn set_frozen_ticks(&self, ticks: i32) {
        let new_frozen_ticks = ticks.clamp(0, Self::MAX_FROZEN_TICKS);
        self.frozen_ticks.store(new_frozen_ticks, Ordering::Relaxed);
        let mut bedrock_meta = SyncedActorDataList::new();
        bedrock_meta.set(
            entity_data_key::FREEZING_EFFECT_STRENGTH,
            MetadataValue::Float(new_frozen_ticks as f32),
        );
        self.set_synced_data(
            tracked_data::entity::DATA_TICKS_FROZEN,
            VarInt(new_frozen_ticks),
        );
        self.send_bedrock_actor_data(&bedrock_meta);
    }

    /// Returns the number of ticks the entity has been frozen.
    pub fn get_frozen_ticks(&self) -> i32 {
        self.frozen_ticks.load(Ordering::Relaxed)
    }

    /// Sets the `Entity` yaw & pitch rotation
    pub fn set_rotation(&self, yaw: f32, pitch: f32) {
        // TODO
        self.yaw.store(yaw);
        // Java players have no separate head-yaw packet from the client- look yaw is also head yaw.
        // Bedrock `MovePlayer` / `MoveActorDelta` need this field, not only `yaw`.
        self.head_yaw.store(yaw);
        self.set_pitch(pitch);
    }

    pub fn set_pitch(&self, pitch: f32) {
        self.pitch.store(pitch.clamp(-90.0, 90.0) % 360.0);
    }

    /// Removes the `Entity` from their current `World`
    pub fn remove(&self) {
        self.world.load().remove_entity(self);
    }

    /// Vanilla `ClientboundAddEntityPacket(entity, serverEntity)`: tracker, not live.
    /// Players live: tracker skips `send_changes` for them.
    fn spawn_state(&self) -> SpawnState {
        if self.entity_type == &EntityType::PLAYER {
            SpawnState {
                pos: self.pos.load(),
                velocity: self.velocity.load(),
                pitch: pack_degrees(self.pitch.load()),
                yaw: pack_degrees(self.yaw.load()),
                head_yaw: pack_degrees(self.head_yaw.load()),
            }
        } else {
            SpawnState {
                pos: self.last_sent_pos.load(),
                velocity: self.last_sent_velocity.load(),
                pitch: self.last_sent_pitch.load(Relaxed),
                yaw: self.last_sent_yaw.load(Relaxed),
                head_yaw: self.last_sent_head_yaw.load(Relaxed),
            }
        }
    }

    pub fn create_spawn_packet(&self) -> CSpawnEntity {
        let spawn = self.spawn_state();
        CSpawnEntity::new_packed(
            VarInt(self.entity_id),
            self.entity_uuid,
            VarInt(i32::from(self.entity_type.id)),
            spawn.pos,
            spawn.pitch,
            spawn.yaw,
            spawn.head_yaw, // todo: head_yaw and yaw are swapped, find out why
            self.data.load(Relaxed).into(),
            spawn.velocity,
        )
    }

    pub fn width(&self) -> f32 {
        self.entity_dimension.load().width
    }

    pub fn height(&self) -> f32 {
        self.entity_dimension.load().height
    }

    pub fn set_sneaking(&self, sneaking: bool) {
        //assert!(self.sneaking.load(Relaxed) != sneaking);
        self.sneaking.store(sneaking, Relaxed);
        self.set_flag(Flag::Sneaking, sneaking);
    }
    pub fn is_sneaking(&self) -> bool {
        self.sneaking.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn is_swimming(&self) -> bool {
        self.swimming.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn is_visually_swimming(&self) -> bool {
        self.pose.load() == EntityPose::Swimming
    }

    #[must_use]
    pub fn is_in_water(&self) -> bool {
        self.touching_water.load(Ordering::Relaxed)
    }

    /// Vanilla `isInLiquid`
    #[must_use]
    pub fn is_in_liquid(&self) -> bool {
        self.is_in_water() || self.touching_lava.load(Ordering::Relaxed)
    }

    pub fn is_in_water_or_rain(&self) -> bool {
        self.is_in_water() || self.is_in_rain()
    }

    fn is_in_rain(&self) -> bool {
        let world = self.world.load();
        let pos = self.block_pos.load();
        world.is_raining_at(&pos)
            || world.is_raining_at(&BlockPos::new(
                pos.0.x,
                self.bounding_box.load().max.y.floor() as i32,
                pos.0.z,
            ))
    }

    #[must_use]
    pub fn is_submerged_in_water(&self) -> bool {
        let pos = self.pos.load();
        let eye_y = pos.y + self.get_eye_height();
        let fluid_box = self.bounding_box.load().contract_all(0.001);
        // Include the block below an exact eye boundary, within vanilla's fluid scan box.
        let min_y = (eye_y - 1.0).ceil().max(fluid_box.min.y.floor()) as i32;
        let max_y = eye_y.floor().min(fluid_box.max.y.ceil() - 1.0) as i32;
        let world = self.world.load();
        for y in min_y..=max_y {
            let eye_pos = BlockPos::floored(pos.x, f64::from(y), pos.z);
            let (fluid, state) = world.get_fluid_and_fluid_state(&eye_pos);
            if fluid.matches_type(&Fluid::WATER)
                && f64::from(y) + f64::from(world.get_fluid_height(&eye_pos, fluid, &state))
                    >= fluid_box.min.y
                && eye_y
                    <= f64::from(y)
                        + f64::from(world.get_fluid_height_for_camera(&eye_pos, fluid, &state))
            {
                return true;
            }
        }
        false
    }

    #[must_use]
    pub fn is_under_water(&self) -> bool {
        self.is_in_water() && self.is_submerged_in_water()
    }

    #[must_use]
    pub fn is_visually_crawling(&self) -> bool {
        self.is_visually_swimming() && !self.is_in_water()
    }

    pub fn set_swimming(&self, swimming: bool) {
        if self.swimming.load(Ordering::Relaxed) != swimming {
            let mut event =
                crate::plugin::api::events::entity::entity_toggle_swim::EntityToggleSwimEvent::new(
                    self.entity_id,
                    swimming,
                );
            if let Some(server) = self.world.load().server.upgrade() {
                event = server.plugin_manager.fire_blocking(&server, event);
            }
            if event.cancelled {
                return;
            }
            self.swimming.store(event.is_swimming, Relaxed);
            self.set_flag(Flag::Swimming, event.is_swimming);
        }
    }

    /// Sets whether the entity is invisible and sends updated metadata.
    pub fn set_invisible(&self, invisible: bool) {
        if self.invisible.load(Ordering::Relaxed) != invisible {
            self.invisible.store(invisible, Relaxed);
            self.set_flag(Flag::Invisible, invisible);
        }
    }

    pub fn set_glowing(&self, glowing: bool) {
        self.has_glowing_tag.store(glowing, Ordering::Relaxed);
        self.update_glowing_flag();
    }

    pub fn set_glowing_from_effect(&self, glowing: bool) {
        self.glowing_from_effect.store(glowing, Ordering::Relaxed);
        self.update_glowing_flag();
    }

    fn update_glowing_flag(&self) {
        let glowing = self.has_glowing_tag.load(Ordering::Relaxed)
            || self.glowing_from_effect.load(Ordering::Relaxed);
        if self.glowing.load(Ordering::Relaxed) != glowing {
            self.glowing.store(glowing, Ordering::Relaxed);
            self.set_flag(Flag::Glowing, glowing);
        }
    }

    /// Sets whether the entity is on fire for visual and damage purposes. This is separate from `fire_ticks` which tracks the damage aspect of being on fire.
    pub fn set_on_fire(&self, on_fire: bool) {
        if self.has_visual_fire.load(Ordering::Relaxed) != on_fire {
            self.has_visual_fire.store(on_fire, Ordering::Relaxed);
            self.set_flag(Flag::OnFire, on_fire);
        }
    }

    #[must_use]
    pub fn get_y(&self, height_fraction: f64) -> f64 {
        f64::from(self.entity_dimension.load().height).mul_add(height_fraction, self.pos.load().y)
    }

    #[must_use]
    pub fn is_on_fire(&self) -> bool {
        self.fire_ticks.load(Ordering::Relaxed) > 0 || self.has_visual_fire.load(Ordering::Relaxed)
    }

    pub fn get_horizontal_facing(&self) -> HorizontalFacing {
        let yaw = self.yaw.load();
        // Use vanilla's formula: floor(angle / 90.0 + 0.5) & 3
        let quarter_turns = ((yaw / 90.0) + 0.5).floor() as i32 & 3;
        match quarter_turns {
            0 => HorizontalFacing::South,
            1 => HorizontalFacing::West,
            2 => HorizontalFacing::North,
            _ => HorizontalFacing::East,
        }
    }

    pub fn get_rotation_16(&self) -> u8 {
        let adjusted_yaw = self.yaw.load().rem_euclid(360.0);

        ((adjusted_yaw / 22.5).round() as u8) % 16
    }

    pub fn get_flipped_rotation_16(&self) -> u8 {
        (self.get_rotation_16() + 8) % 16
    }

    pub fn get_facing(&self) -> Facing {
        let pitch = self.pitch.load().to_radians();
        let yaw = -self.yaw.load().to_radians();

        let (sin_p, cos_p) = pitch.sin_cos();
        let (sin_y, cos_y) = yaw.sin_cos();

        let x = sin_y * cos_p;
        let y = -sin_p;
        let z = cos_y * cos_p;

        let ax = x.abs();
        let ay = y.abs();
        let az = z.abs();

        if ax > ay && ax > az {
            if x > 0.0 { Facing::East } else { Facing::West }
        } else if ay > ax && ay > az {
            if y > 0.0 { Facing::Up } else { Facing::Down }
        } else if z > 0.0 {
            Facing::South
        } else {
            Facing::North
        }
    }

    pub fn get_entity_facing_order(&self) -> [Facing; 6] {
        let pitch = self.pitch.load().to_radians();
        let yaw = -self.yaw.load().to_radians();

        let sin_p = pitch.sin();
        let cos_p = pitch.cos();
        let sin_y = yaw.sin();
        let cos_y = yaw.cos();

        let east_west = if sin_y > 0.0 {
            Facing::East
        } else {
            Facing::West
        };
        let up_down = if sin_p < 0.0 {
            Facing::Up
        } else {
            Facing::Down
        };
        let south_north = if cos_y > 0.0 {
            Facing::South
        } else {
            Facing::North
        };

        let x_axis = sin_y.abs();
        let y_axis = sin_p.abs();
        let z_axis = cos_y.abs();
        let x_weight = x_axis * cos_p;
        let z_weight = z_axis * cos_p;

        let (first, second, third) = if x_axis > z_axis {
            if y_axis > x_weight {
                (up_down, east_west, south_north)
            } else if z_weight > y_axis {
                (east_west, south_north, up_down)
            } else {
                (east_west, up_down, south_north)
            }
        } else if y_axis > z_weight {
            (up_down, south_north, east_west)
        } else if x_weight > y_axis {
            (south_north, east_west, up_down)
        } else {
            (south_north, up_down, east_west)
        };

        [
            first,
            second,
            third,
            third.opposite(),
            second.opposite(),
            first.opposite(),
        ]
    }

    pub fn set_sprinting(&self, sprinting: bool) {
        //assert!(self.sprinting.load(Relaxed) != sprinting);
        self.sprinting.store(sprinting, Relaxed);
        self.set_flag(Flag::Sprinting, sprinting);
    }

    pub fn is_sprinting(&self) -> bool {
        self.sprinting.load(Ordering::Relaxed)
    }
    pub fn set_fall_flying(&self, fall_flying: bool) {
        self.fall_flying.store(fall_flying, Relaxed);
        self.set_flag(Flag::FallFlying, fall_flying);
    }

    /// Vanilla `stopFallFlying`: always resends the flag, so a client that predicted gliding is corrected
    pub fn stop_fall_flying(&self) {
        let mask = 1i8 << Flag::FallFlying as u8;
        self.set_synced_data(
            tracked_data::entity::DATA_SHARED_FLAGS_ID,
            self.flags.load(Relaxed) | mask,
        );
        self.set_fall_flying(false);
    }
    pub fn is_fall_flying(&self) -> bool {
        self.fall_flying.load(Ordering::Relaxed)
    }

    fn set_flag(&self, flag: Flag, value: bool) {
        let index = flag as u8;
        let mask = (1i8).wrapping_shl(index as u32);
        let new_je_flags = if value {
            self.flags.fetch_or(mask, Ordering::Relaxed) | mask
        } else {
            self.flags.fetch_and(!mask, Ordering::Relaxed) & !mask
        };

        self.set_synced_data(tracked_data::entity::DATA_SHARED_FLAGS_ID, new_je_flags);

        if let Some(bedrock_flag) = flag.to_bedrock() {
            self.set_bedrock_flag(bedrock_flag, value);
        }
    }

    /// Sets an actor flag only Bedrock has and sends the flag words to Bedrock viewers.
    pub fn set_bedrock_flag(&self, bedrock_flag: u32, value: bool) {
        self.init_bedrock_flags();
        let (key, index) = if bedrock_flag >= 64 {
            (entity_data_key::FLAGS_TWO, (bedrock_flag - 64) as u8)
        } else {
            (entity_data_key::FLAGS, bedrock_flag as u8)
        };

        if value {
            let mask = 1i64 << index;
            if key == entity_data_key::FLAGS {
                self.bedrock_flags.fetch_or(mask, Ordering::Relaxed);
            } else {
                self.bedrock_flags_two.fetch_or(mask, Ordering::Relaxed);
            }
        } else {
            let mask = !(1i64 << index);
            if key == entity_data_key::FLAGS {
                self.bedrock_flags.fetch_and(mask, Ordering::Relaxed);
            } else {
                self.bedrock_flags_two.fetch_and(mask, Ordering::Relaxed);
            }
        }

        let world = self.world.load();
        let chunk_pos = self.chunk_pos.load();
        let mut metadata = SyncedActorDataList(std::collections::HashMap::new());
        metadata.set(
            entity_data_key::FLAGS,
            MetadataValue::Int64(self.bedrock_flags.load(Ordering::Relaxed)),
        );
        metadata.set(
            entity_data_key::FLAGS_TWO,
            MetadataValue::Int64(self.bedrock_flags_two.load(Ordering::Relaxed)),
        );
        let packet = CSetActorData {
            target_runtime_id: VarULong(self.entity_id as u64),
            actor_data: metadata,
            synced_properties: PropertySyncData {
                int_entries_list: std::collections::HashMap::new(),
                float_entries_list: std::collections::HashMap::new(),
            },
            tick: VarULong(0),
        };
        world.broadcast_to_chunk_bedrock(chunk_pos, &packet);
    }

    /// Plays sound at this entity's position with the entity's sound category
    pub fn play_sound(&self, sound: Sound) {
        self.world
            .load()
            .play_sound(sound, SoundCategory::Neutral, &self.pos.load());
    }

    pub fn set_synced_data<T: MetadataSerializer + Clone + Send + Sync + 'static>(
        &self,
        tracked: pumpkin_data::tracked_data::TrackedData,
        value: T,
    ) -> bool {
        // Sent by the tracker at end of tick, vanilla `sendDirtyEntityData`.
        self.synched_data.set(tracked, value)
    }

    pub fn send_bedrock_actor_data(&self, bedrock_meta: &SyncedActorDataList) {
        let world = self.world.load();

        if let Some(tracked) = world.entity_tracker.get_tracked_entity(self.entity_id) {
            let self_bedrock = tracked
                .entity
                .get_player()
                .and_then(|p| match p.client.as_ref() {
                    ClientPlatform::Bedrock(client) => Some(client),
                    ClientPlatform::Java(_) | ClientPlatform::SimulatedPlayer(_) => None,
                });
            if !tracked.has_bedrock_watchers() && self_bedrock.is_none() {
                return;
            }
            let packet = CSetActorData {
                target_runtime_id: VarULong(self.entity_id as u64),
                actor_data: SyncedActorDataList(bedrock_meta.0.clone()),
                synced_properties: PropertySyncData {
                    int_entries_list: std::collections::HashMap::new(),
                    float_entries_list: std::collections::HashMap::new(),
                },
                tick: VarULong(0),
            };
            tracked.send_to_tracking_players_bedrock(&packet, &world);
            if let Some(client) = self_bedrock
                && let Ok(packet_data) = client.serialize_packet(&packet)
            {
                client.try_enqueue_packet(packet_data);
            }
            return;
        }

        let players = world.players.load();
        let chunk_pos = self.chunk_pos.load();
        let mut bedrock_recipients = Vec::new();
        for player in players.iter() {
            if player
                .watched_section
                .load()
                .is_within_distance(chunk_pos.x, chunk_pos.y)
                && let ClientPlatform::Bedrock(client) = player.client.as_ref()
            {
                bedrock_recipients.push(client);
            }
        }
        if bedrock_recipients.is_empty() {
            return;
        }

        let packet = CSetActorData {
            target_runtime_id: VarULong(self.entity_id as u64),
            actor_data: SyncedActorDataList(bedrock_meta.0.clone()),
            synced_properties: PropertySyncData {
                int_entries_list: std::collections::HashMap::new(),
                float_entries_list: std::collections::HashMap::new(),
            },
            tick: VarULong(0),
        };
        for recipient in bedrock_recipients {
            if let Ok(packet_data) = recipient.serialize_packet(&packet) {
                recipient.try_enqueue_packet(packet_data);
            }
        }
    }

    fn send_bedrock_flags(
        &self,
        world: &Arc<World>,
        revision: u64,
        mob_metadata: impl Fn() -> Option<mob::MobMetadata>,
    ) {
        let flag_words = || {
            let [primary, secondary] =
                mob_metadata().map_or([0; 2], |metadata| metadata.bedrock_flag_words());
            [
                self.bedrock_flag_word() | primary,
                self.bedrock_flags_two.load(Relaxed) | secondary,
            ]
        };
        for _ in 0..2 {
            if self.is_removed() || self.riding_revision() != revision {
                return;
            }
            let recipients = self.bedrock_metadata_recipients(world);
            if recipients.is_empty() {
                return;
            }
            let [primary, secondary] = flag_words();
            let mut actor_data = SyncedActorDataList::new();
            actor_data.set(entity_data_key::FLAGS, MetadataValue::Int64(primary));
            actor_data.set(entity_data_key::FLAGS_TWO, MetadataValue::Int64(secondary));
            let changed = std::cell::Cell::new(false);
            self.send_guarded_bedrock_actor_data_to(world, &recipients, actor_data, || {
                let current = flag_words();
                let vehicle = self
                    .vehicle
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if self.is_removed() || self.riding_revision() != revision {
                    return None;
                }
                if current != [primary, secondary] {
                    changed.set(true);
                    return None;
                }
                Some(vehicle)
            });
            if !changed.get() {
                return;
            }
        }
        // A listener can change flags on every send. Retry on the next tracker tick
        // after the immediate retry, without growing the synchronous callback chain.
        self.bedrock_flags_dirty.store(true, Ordering::Release);
    }

    pub(crate) fn flush_bedrock_flags(&self, rider: &dyn EntityBase) {
        if self.bedrock_flags_dirty.load(Relaxed)
            && self.bedrock_flags_dirty.swap(false, Ordering::AcqRel)
        {
            self.send_bedrock_riding_flags(rider, self.riding_revision());
        }
    }

    fn send_bedrock_riding_flags(&self, rider: &dyn EntityBase, revision: u64) {
        let world = self.world.load_full();
        self.send_bedrock_flags(&world, revision, || {
            rider.get_mob().and_then(mob::Mob::mob_metadata)
        });
    }

    pub(super) fn send_bedrock_age_metadata(
        &self,
        is_baby: bool,
        mob_metadata: impl Fn() -> Option<mob::MobMetadata>,
    ) {
        // Age owns scale here. Other metadata has its own publisher and can change in a callback.
        let scale = || {
            mob_metadata()
                .and_then(|metadata| metadata.bedrock_scale())
                .unwrap_or_else(|| if self.age.load(Relaxed) < 0 { 0.5 } else { 1.0 })
        };
        let sent_scale = scale();
        let mut data = SyncedActorDataList::new();
        data.set(entity_data_key::SCALE, MetadataValue::Float(sent_scale));
        let world = self.world.load_full();
        self.send_guarded_bedrock_actor_data(&world, data, || {
            (!self.is_removed()
                && (self.age.load(Relaxed) < 0) == is_baby
                && scale().to_bits() == sent_scale.to_bits())
            .then_some(())
        });
        if (self.age.load(Relaxed) < 0) == is_baby {
            self.send_bedrock_flags(&world, self.riding_revision(), mob_metadata);
        }
    }

    fn send_bedrock_pose(&self, world: &Arc<World>, pose: EntityPose) {
        // Size can change independently of the pose in an outgoing packet observer.
        self.send_bedrock_dimensions(world, self.entity_dimension.load());
        let mut data = SyncedActorDataList::new();
        data.set(entity_data_key::POSE_INDEX, MetadataValue::Int(pose as i32));
        self.send_guarded_bedrock_actor_data(world, data, || {
            (!self.is_removed() && self.pose.load() == pose).then_some(())
        });
    }

    fn send_bedrock_dimensions(&self, world: &Arc<World>, dimensions: EntityDimensions) {
        let mut data = SyncedActorDataList::new();
        data.set(
            entity_data_key::WIDTH,
            MetadataValue::Float(dimensions.width),
        );
        data.set(
            entity_data_key::HEIGHT,
            MetadataValue::Float(dimensions.height),
        );
        self.send_guarded_bedrock_actor_data(world, data, || {
            (!self.is_removed() && self.entity_dimension.load() == dimensions).then_some(())
        });
    }

    fn send_guarded_bedrock_actor_data<T>(
        &self,
        world: &Arc<World>,
        actor_data: SyncedActorDataList,
        lock_if_current: impl Fn() -> Option<T>,
    ) {
        let recipients = self.bedrock_metadata_recipients(world);
        self.send_guarded_bedrock_actor_data_to(world, &recipients, actor_data, lock_if_current);
    }

    fn bedrock_metadata_recipients(&self, world: &Arc<World>) -> Vec<Arc<Player>> {
        world
            .entity_tracker
            .get_tracked_entity(self.entity_id)
            .map_or_else(
                || {
                    let chunk = self.chunk_pos.load();
                    world
                        .players
                        .load()
                        .iter()
                        .filter(|player| {
                            player.client.bedrock().is_some()
                                && player
                                    .watched_section
                                    .load()
                                    .is_within_distance(chunk.x, chunk.y)
                        })
                        .cloned()
                        .collect()
                },
                |tracked| {
                    let mut recipients = tracked.bedrock_watchers_snapshot();
                    if self.entity_type == &EntityType::PLAYER
                        && let Some(player) = world.get_player_by_id(self.entity_id)
                        && player.client.bedrock().is_some()
                    {
                        recipients.push(player);
                    }
                    recipients
                },
            )
    }

    fn send_guarded_bedrock_actor_data_to<T>(
        &self,
        world: &Arc<World>,
        recipients: &[Arc<Player>],
        actor_data: SyncedActorDataList,
        lock_if_current: impl Fn() -> Option<T>,
    ) {
        if recipients.is_empty() {
            return;
        }
        let packet = CSetActorData {
            target_runtime_id: VarULong(self.entity_id as u64),
            actor_data,
            synced_properties: PropertySyncData {
                int_entries_list: std::collections::HashMap::new(),
                float_entries_list: std::collections::HashMap::new(),
            },
            tick: VarULong(0),
        };
        for recipient in recipients {
            if let Some(client) = recipient.client.bedrock()
                && let Ok(data) = client.serialize_packet(&packet)
            {
                let protocol = client.protocol_version.load(Relaxed);
                let protocol = if protocol == 0 {
                    pumpkin_world::CURRENT_BEDROCK_MC_PROTOCOL
                } else {
                    protocol
                };
                client.try_enqueue_packet_data_if(
                    data,
                    CSetActorData::PACKET_ID,
                    |payload| {
                        packet.actor_data.0.keys().any(|&key| {
                            CSetActorData::updates_key(
                                payload,
                                packet.target_runtime_id,
                                protocol,
                                key,
                            )
                            .unwrap_or(false)
                        })
                    },
                    || {
                        // Observers have returned. Keep the check and queue insertion
                        // ordered with other metadata publishers for this entity.
                        let enqueue = self
                            .bedrock_metadata_enqueue
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        let guard = lock_if_current()?;
                        (Arc::ptr_eq(&self.world.load(), world)
                            && Arc::ptr_eq(&recipient.world(), world))
                        .then_some((enqueue, guard))
                    },
                );
            }
        }
    }

    fn send_java_metadata_packet(&self, world: &World, packet: &CSetEntityMetadata) {
        if let Some(tracked) = world.entity_tracker.get_tracked_entity(self.entity_id) {
            tracked.send_to_tracking_players(packet, world);
            if let Some(player) = tracked.entity.get_player() {
                player.try_send_client_packet(packet);
            } else if self.entity_type == &EntityType::PLAYER
                && let Some(player) = world.get_player_by_id(self.entity_id)
            {
                player.try_send_client_packet(packet);
            }
        } else {
            let players = world.players.load();
            let chunk_pos = self.chunk_pos.load();
            World::broadcast_java_players(
                packet,
                players.iter().filter(|player| {
                    player.entity_id() == self.entity_id
                        || player
                            .watched_section
                            .load()
                            .is_within_distance(chunk_pos.x, chunk_pos.y)
                }),
            );
        }
    }

    /// Sends the given metadata entries to this entity's viewers, and the matching
    /// actor data to the Bedrock ones when `bedrock_meta` is given.
    pub fn send_meta_data<T: MetadataSerializer>(
        &self,
        meta: &[Metadata<T>],
        bedrock_meta: Option<&SyncedActorDataList>,
    ) {
        let world = self.world.load();
        let version = CURRENT_MC_VERSION;
        let mut buf = Vec::new();
        for m in meta {
            let _ = m.write(&mut buf, &version);
        }
        if !buf.is_empty() {
            buf.put_u8(255);
            let packet = CSetEntityMetadata::new(self.entity_id.into(), buf.into());
            self.send_java_metadata_packet(&world, &packet);
        }

        if let Some(bedrock_meta) = bedrock_meta {
            self.send_bedrock_actor_data(bedrock_meta);
        }
    }

    /// Sends the synced values that changed since the last call to this entity's
    /// viewers, and does nothing when none did.
    pub fn send_dirty_entity_data(&self) {
        if !self.synched_data.is_dirty() {
            return;
        }

        let world = self.world.load();

        // Unwatched, skip the player scan.
        if self.entity_type != &EntityType::PLAYER
            && world
                .entity_tracker
                .get_tracked_entity(self.entity_id)
                .is_some_and(|tracked| !tracked.has_viewers())
        {
            self.synched_data.clear_dirty();
            return;
        }

        let version = CURRENT_MC_VERSION;
        if let Some(buf) = self.synched_data.pack_dirty_for_version(&version) {
            let packet = CSetEntityMetadata::new(self.entity_id.into(), buf);
            self.send_java_metadata_packet(&world, &packet);
        }
        self.synched_data.clear_dirty();
    }

    /// Resends every non-default synced value of this entity to its viewers, the entity
    /// itself included when it is a player.
    ///
    /// Clients discard the entity data they cached whenever the entity is re-created on
    /// their side (a respawn or a dimension change re-creates the local player, and a
    /// spawn packet resets the entity for everyone else), so the values have to be pushed
    /// again even though nothing became dirty on our side.
    pub fn refresh_synced_data(&self) {
        let world = self.world.load();
        let version = CURRENT_MC_VERSION;
        if let Some(buf) = self
            .synched_data
            .get_non_default_values_for_version(&version)
        {
            let packet = CSetEntityMetadata::new(self.entity_id.into(), buf);
            self.send_java_metadata_packet(&world, &packet);
        }
        self.synched_data.clear_dirty();
    }

    pub fn set_pose(&self, pose: EntityPose) {
        self.set_pose_if(pose, || true);
    }

    fn set_pose_if(&self, pose: EntityPose, may_apply: impl FnOnce() -> bool) -> bool {
        if self.pose.load() == pose {
            return may_apply();
        }

        if !self.allow_pose_change(pose) || !may_apply() {
            return false;
        }

        let world = self.world.load_full();
        self.pose.store(pose);
        self.sync_pose(pose);
        self.send_bedrock_pose(&world, pose);
        true
    }

    fn allow_pose_change(&self, pose: EntityPose) -> bool {
        let mut pose_event =
            crate::plugin::api::events::entity::entity_pose_change::EntityPoseChangeEvent::new(
                self.entity_id,
                (pose as u8).to_string(),
            );
        if let Some(server) = self.world.load().server.upgrade() {
            pose_event = server.plugin_manager.fire_blocking(&server, pose_event);
        }
        !pose_event.cancelled
    }

    fn sync_pose(&self, pose: EntityPose) {
        self.refresh_dimensions_state();
        self.set_synced_data(tracked_data::entity::DATA_POSE, VarInt(pose as i32));
    }

    /// Vanilla `getDefaultDimensions` changed, e.g. a baby growing up.
    pub fn set_default_dimensions(&self, dimensions: EntityDimensions) {
        self.default_dimensions.store(Some(dimensions));
        self.refresh_dimensions();
    }

    /// Vanilla `LivingEntity.getHitbox`: the bounding box, with the bottom clipped
    /// to the vehicle seat when riding.
    #[must_use]
    pub fn get_hitbox(&self) -> BoundingBox {
        let aabb = self.bounding_box.load();
        let Some(vehicle) = self.get_vehicle() else {
            return aabb;
        };
        vehicle
            .passenger_riding_position(self)
            .map_or(aabb, |seat| aabb.with_min_y(seat.y.max(aabb.min.y)))
    }

    /// Vanilla `refreshDimensions`: size and bounding box for the current pose.
    pub fn refresh_dimensions(&self) {
        let world = self.world.load_full();
        let dimensions = self.refresh_dimensions_state();
        self.send_bedrock_dimensions(&world, dimensions);
    }

    fn refresh_dimensions_state(&self) -> EntityDimensions {
        let dimensions = self.get_dimensions(self.pose.load());
        let old = self.entity_dimension.swap(dimensions);
        // Vanilla `reapplyPosition`: the box is rebuilt even at the same size
        self.bounding_box.store(Self::make_bounding_box(
            self.entity_type,
            self.pos.load(),
            &dimensions,
        ));
        if !self.first_tick.load(Ordering::Relaxed)
            && !self.no_physics.load(Ordering::Relaxed)
            && dimensions.width <= 4.0
            && dimensions.height <= 4.0
            && (dimensions.width > old.width || dimensions.height > old.height)
            && self.entity_type != &EntityType::PLAYER
        {
            self.fudge_position_after_size_change(old);
        }
        dimensions
    }

    /// Vanilla `fudgePositionAfterSizeChange`: a grown entity moves to the closest free spot.
    pub fn fudge_position_after_size_change(&self, previous: EntityDimensions) -> bool {
        // The full entity, so block collisions see its context (powder snow, ...)
        let Some(this) = self.world.load().get_entity_by_id(self.entity_id) else {
            return false;
        };
        self.fudge_position_after_size_change_of(this.as_ref(), previous)
    }

    /// `fudge_position_after_size_change` for a caller that already holds the full entity.
    pub fn fudge_position_after_size_change_of(
        &self,
        this: &dyn EntityBase,
        previous: EntityDimensions,
    ) -> bool {
        let world = self.world.load();
        let new = self.get_dimensions(self.pose.load());
        let previous_height = f64::from(previous.height);
        let (new_width, new_height) = (f64::from(new.width), f64::from(new.height));
        let old_center = self.pos.load().add_raw(0.0, previous_height / 2.0, 0.0);
        let width_delta = f64::from((new.width - previous.width).max(0.0)) + 1.0E-6;
        let height_delta = f64::from((new.height - previous.height).max(0.0)) + 1.0E-6;
        if let Some(free) = world.find_free_position(
            this,
            &BoundingBox::of_size(old_center, width_delta, height_delta, width_delta),
            old_center,
            Vector3::new(new_width, new_height, new_width),
        ) {
            self.set_pos(free.add_raw(0.0, -new_height / 2.0, 0.0));
            return true;
        }
        // Grown both ways: free space for the new width at the old height
        if new.width > previous.width
            && new.height > previous.height
            && let Some(free) = world.find_free_position(
                this,
                &BoundingBox::of_size(old_center, width_delta, 1.0E-6, width_delta),
                old_center,
                Vector3::new(new_width, previous_height, new_width),
            )
        {
            self.set_pos(free.add_raw(0.0, -previous_height / 2.0 + 1.0E-6, 0.0));
            return true;
        }
        false
    }

    /// Vanilla `Entity.fireImmune`: the entity type.
    pub fn is_fire_immune(&self) -> bool {
        self.entity_type.fire_immune || self.fire_immune.load(Ordering::Relaxed)
    }

    /// Vanilla `Entity.isInvulnerableToBase`. Driven by the damage type tags, so datapacks
    /// decide what bypasses invulnerability or counts as fire and fall damage.
    /// `cause` is the attacker (vanilla `DamageSource.getEntity`).
    pub fn is_invulnerable_to(
        &self,
        damage_type: &DamageType,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        let bypasses = damage_type.has_tag(&tag::DamageType::MINECRAFT_BYPASSES_INVULNERABILITY);
        let creative_cause = cause
            .and_then(EntityBase::get_player)
            .is_some_and(Player::is_creative);

        self.removed.load(Ordering::SeqCst)
            || (self.invulnerable.load(Ordering::Relaxed) && !bypasses && !creative_cause)
            || (damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_FIRE) && self.is_fire_immune())
            || (damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_FALL)
                && self
                    .entity_type
                    .has_tag(&tag::EntityType::MINECRAFT_FALL_DAMAGE_IMMUNE))
            // Plugin immunities, which never block damage that bypasses invulnerability.
            || (!bypasses
                && self
                    .damage_immunities
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .contains(damage_type))
    }

    /// Sets if the entity is invulnerable to a specific damage type
    pub fn set_damage_immunity(&self, damage_type: DamageType, immune: bool) {
        let mut immunities = self
            .damage_immunities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if immune {
            if !immunities.contains(&damage_type) {
                immunities.push(damage_type);
            }
        } else {
            // retain is cleaner than finding index and removing
            immunities.retain(|dt| dt != &damage_type);
        }
    }

    /// Sets if the entity is invulnerable to all damage types (except `GENERIC_KILL` and `OUT_OF_WORLD`)
    pub fn set_invulnerable(&self, invulnerable: bool) {
        self.invulnerable.store(invulnerable, Relaxed);
    }

    pub fn check_block_collision(entity: &dyn EntityBase, server: &Server) {
        let aabb = entity.get_entity().bounding_box.load();
        let blockpos = BlockPos::new(
            (aabb.min.x + 0.001).floor() as i32,
            (aabb.min.y + 0.001).floor() as i32,
            (aabb.min.z + 0.001).floor() as i32,
        );
        let blockpos1 = BlockPos::new(
            (aabb.max.x - 0.001).floor() as i32,
            (aabb.max.y - 0.001).floor() as i32,
            (aabb.max.z - 0.001).floor() as i32,
        );
        let world = entity.get_entity().world.load();

        for x in blockpos.0.x..=blockpos1.0.x {
            for y in blockpos.0.y..=blockpos1.0.y {
                for z in blockpos.0.z..=blockpos1.0.z {
                    let pos = BlockPos::new(x, y, z);
                    let (block, state) = world.get_block_and_state(&pos);
                    let block_outlines = state.get_block_outline_shapes_at(&pos);

                    if state.outline_shapes.is_empty() {
                        world
                            .block_registry
                            .on_entity_collision(block, &world, entity, &pos, state, server);
                        let fluid = world.get_fluid(&pos);
                        world
                            .block_registry
                            .on_entity_collision_fluid(fluid, entity);
                        continue;
                    }
                    for outline in block_outlines {
                        let outline_aabb = outline.at_pos(pos);
                        if outline_aabb.intersects(&aabb) {
                            world
                                .block_registry
                                .on_entity_collision(block, &world, entity, &pos, state, server);
                            let fluid = world.get_fluid(&pos);
                            world
                                .block_registry
                                .on_entity_collision_fluid(fluid, entity);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn teleport(
        &self,
        position: Vector3<f64>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        world: &World,
    ) {
        let mut movement_history = self.vehicle_movements.as_ref().map(|history| {
            history
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        let teleport_epoch = movement_history.as_mut().map(|history| {
            history.pending.clear();
            history.teleport_revision = history.teleport_revision.wrapping_add(1);
            self.last_pos.store(position);
            history.teleport_revision
        });
        // Update server-side position and bounding box
        self.set_pos(position);
        if let Some(yaw) = yaw {
            self.yaw.store(yaw);
        }
        if let Some(pitch) = pitch {
            self.set_pitch(pitch);
        }
        // Update cache so we don't send rubberbanding deltas
        self.last_sent_pos.store(position);
        // TODO: use `pumpkin_util::math::pack_degrees`.
        if let Some(yaw) = yaw {
            self.last_sent_yaw
                .store((yaw * 256.0 / 360.0).rem_euclid(256.0) as u8, Relaxed);
            self.last_sent_head_yaw
                .store((yaw * 256.0 / 360.0).rem_euclid(256.0) as u8, Relaxed);
        }
        if let Some(pitch) = pitch {
            self.last_sent_pitch
                .store((pitch * 256.0 / 360.0).rem_euclid(256.0) as u8, Relaxed);
        }
        let chunk_pos = self.chunk_pos.load();
        let packet = CEntityPositionSync::new(
            self.entity_id.into(),
            position,
            self.velocity.load(),
            yaw.unwrap_or(self.yaw.load()),
            pitch.unwrap_or(self.pitch.load()),
            self.on_ground.load(Ordering::SeqCst),
        );
        drop(movement_history);
        if let Some(teleport_epoch) = teleport_epoch {
            self.broadcast_vehicle_teleport(world, chunk_pos, &packet, teleport_epoch);
        } else {
            world.broadcast_to_chunk(chunk_pos, &packet);
        }
    }

    fn broadcast_vehicle_teleport(
        &self,
        world: &World,
        chunk_pos: Vector2<i32>,
        packet: &CEntityPositionSync,
        teleport_epoch: u64,
    ) {
        let lock_if_current = || {
            let history = self.lock_vehicle_motion_if_current(teleport_epoch)?;
            (!self.is_removed()
                && self.pos.load() == packet.position
                && std::ptr::eq(self.world.load().as_ref(), world))
            .then_some(history)
        };
        let is_watching = |recipient: &Player| {
            std::ptr::eq(recipient.world().as_ref(), world)
                && recipient
                    .watched_section
                    .load()
                    .is_within_distance(chunk_pos.x, chunk_pos.y)
        };
        let Ok(data) = JavaClient::serialize_packet_for_version(packet, CURRENT_MC_VERSION) else {
            return;
        };
        for recipient in world.players.load().iter().filter(|p| is_watching(p)) {
            if lock_if_current().is_none() {
                break;
            }
            let Some(client) = recipient.client.java() else {
                continue;
            };
            // A packet observer can teleport the entity again before this enqueue.
            client.try_enqueue_packet_data_if(
                data.clone(),
                CEntityPositionSync::to_id(client.version.load()),
                |mut payload| payload.get_var_int().is_ok_and(|id| id.0 == self.entity_id),
                || {
                    let history = lock_if_current()?;
                    is_watching(recipient).then_some(history)
                },
            );
        }
    }

    pub fn get_eye_pos(&self) -> Vector3<f64> {
        let pos = self.pos.load();
        Vector3::new(pos.x, pos.y + self.get_eye_height(), pos.z)
    }

    /// No visual-blocking block between the two eye positions (matching vanilla `ClipContext.Block.VISUAL`).
    #[must_use]
    pub fn has_line_of_sight(&self, other: &Self) -> bool {
        let from = self.get_eye_pos();
        let to = other.get_eye_pos();
        let distance_sq = from.squared_distance_to_vec(&to);
        if distance_sq.is_nan() || distance_sq > 128.0 * 128.0 {
            return false;
        }
        self.world
            .load_full()
            .clip(&pumpkin_world::clip::ClipContext::new(
                from,
                to,
                pumpkin_world::clip::ClipBlock::Visual,
                pumpkin_world::clip::ClipFluid::None,
            ))
            .is_miss
    }

    pub fn get_eye_y(&self) -> f64 {
        self.pos.load().y + f64::from(self.entity_dimension.load().eye_height)
    }

    pub fn is_removed(&self) -> bool {
        self.removal_reason.load().is_some()
    }

    pub fn is_alive(&self) -> bool {
        !self.is_removed()
    }

    #[must_use]
    pub fn is_affected_by_blocks(&self) -> bool {
        !self.is_removed() && !self.no_physics.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn is_in_wall(&self) -> bool {
        if self.no_physics.load(Ordering::Relaxed) {
            return false;
        }

        let eye_pos = self.get_eye_pos();
        let half_width = (f64::from(self.entity_dimension.load().width) * 0.8) / 2.0;
        let eye_bb = BoundingBox::new(
            Vector3::new(eye_pos.x - half_width, eye_pos.y, eye_pos.z - half_width),
            Vector3::new(
                eye_pos.x + half_width,
                eye_pos.y + 1.0e-6,
                eye_pos.z + half_width,
            ),
        );
        let min = eye_bb.min_block_pos();
        let max = eye_bb.max_block_pos();
        let world = self.world.load();

        for pos in BlockPos::iterate(min, max) {
            let (block, state) = world.get_block_and_state(&pos);
            if state.is_air() {
                continue;
            }

            if blocks_movement(state, block.id) && state.is_full_cube() {
                return true;
            }
        }

        false
    }

    pub const LEASH_SNAP_DISTANCE: f64 = 12.0;
    pub const LEASH_ELASTIC_DISTANCE: f64 = 6.0;

    #[expect(
        clippy::needless_pass_by_value,
        reason = "callers pass owned or cast Arc<dyn EntityBase>"
    )]
    /// Returns false if a plugin cancelled the leash event.
    pub fn leash_to(&self, holder: Arc<dyn EntityBase>) -> bool {
        self.fire_leash_event(&holder) && self.attach_leash(&holder)
    }

    /// Moves the leash to `holder` only while `expected_holder_id` still holds it. The check and
    /// the swap happen under one lock, so concurrent transfers (players' packets are handled in
    /// parallel) cannot both win. Returns whether the leash moved.
    pub fn leash_to_if_held_by(
        &self,
        expected_holder_id: i32,
        holder: &Arc<dyn EntityBase>,
    ) -> bool {
        let held_by = |leashed_to: &Option<Weak<dyn EntityBase>>| {
            leashed_to
                .as_ref()
                .and_then(Weak::upgrade)
                .is_some_and(|current| current.get_entity().entity_id == expected_holder_id)
        };
        // Cheap check first so the plugin event only fires for entities that will move. Each lock
        // is released before the next call: the event and `attach_leash` may lock it again.
        let held = held_by(
            &self
                .leashed_to
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        if !held || !self.fire_leash_event(holder) || !self.leave_vehicle_for_leash() {
            return false;
        }
        let mut leashed_to = self
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !held_by(&leashed_to) {
            return false;
        }
        let old_holder = leashed_to.replace(Arc::downgrade(holder));
        drop(leashed_to);
        self.send_leash_attached(holder);
        if let Some(old_holder) = old_holder.as_ref().and_then(Weak::upgrade) {
            old_holder.notify_leashee_removed();
        }
        true
    }

    pub(crate) fn attach_leash_if_free(&self, holder: &Arc<dyn EntityBase>) -> bool {
        if !self.leave_vehicle_for_leash() {
            return false;
        }
        let mut leashed_to = self
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if leashed_to.is_some() {
            return false;
        }
        *leashed_to = Some(Arc::downgrade(holder));
        drop(leashed_to);
        self.send_leash_attached(holder);
        true
    }

    /// Fires `PlayerLeashEntityEvent` when a player becomes the holder. Returns false if a plugin
    /// cancelled it.
    pub(crate) fn fire_leash_event(&self, holder: &Arc<dyn EntityBase>) -> bool {
        let world = self.world.load();
        if let Some(server) = world.server.upgrade()
            && let Some(player) = holder.get_player()
            && let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id)
        {
            let mut event =
                crate::plugin::api::events::player::player_leash_entity::PlayerLeashEntityEvent {
                    player: player_arc,
                    entity_id: self.entity_id,
                    holder_id: holder.get_entity().entity_id,
                    cancelled: false,
                };
            event = server.plugin_manager.fire_blocking(&server, event);
            return !event.cancelled;
        }
        true
    }

    pub fn attach_leash(&self, holder: &Arc<dyn EntityBase>) -> bool {
        if !self.leave_vehicle_for_leash() {
            return false;
        }
        let old_holder = self
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace(Arc::downgrade(holder));
        self.send_leash_attached(holder);
        if let Some(old_holder) = old_holder.as_ref().and_then(Weak::upgrade)
            && old_holder.get_entity().entity_id != holder.get_entity().entity_id
        {
            old_holder.notify_leashee_removed();
        }
        true
    }

    fn leave_vehicle_for_leash(&self) -> bool {
        if let Some(vehicle) = self.get_vehicle() {
            vehicle.get_entity().remove_passenger(self.entity_id);
        }
        self.get_vehicle().is_none()
    }

    /// Vanilla `Entity.resetAngularLeashMomentum`.
    pub(crate) fn reset_angular_leash_momentum(&self) {
        self.leash_angular_momentum.store(0.0);
    }

    /// Resets the leash motion and tells clients about the new holder. Call it without holding
    /// `leashed_to`: sending the packet runs plugin handlers that may lock it again.
    fn send_leash_attached(&self, holder: &Arc<dyn EntityBase>) {
        let holder_entity_id = holder.get_entity().entity_id;
        self.reset_angular_leash_momentum();
        *self
            .delayed_leash
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;

        let je_packet = pumpkin_protocol::java::client::play::CSetEntityLink::new(
            self.entity_id,
            holder_entity_id,
            true,
        );
        self.bedrock_flags
            .fetch_or(1i64 << entity_data_flag::LEASHED, Ordering::Relaxed);
        let mut bedrock_meta = SyncedActorDataList::new();
        bedrock_meta.set(
            entity_data_key::LEASH_HOLDER,
            MetadataValue::Int64(i64::from(holder_entity_id)),
        );
        bedrock_meta.set(
            entity_data_key::FLAGS,
            MetadataValue::Int64(self.bedrock_flags.load(Ordering::Relaxed)),
        );
        let be_packet = CSetActorData {
            target_runtime_id: VarULong(self.entity_id as u64),
            actor_data: bedrock_meta,
            synced_properties: PropertySyncData::default(),
            tick: VarULong(0),
        };

        self.leash_release_cancelled.store(false, Ordering::Relaxed);
        self.world.load().broadcast_to_chunk_editioned(
            self.chunk_pos.load(),
            &je_packet,
            &be_packet,
        );
    }

    /// Removes the leash unless plugins cancel, returning whether this call removed it.
    pub fn unleash(&self) -> bool {
        self.unleash_held_by(None)
    }

    fn unleash_held_by(&self, held_by: Option<i32>) -> bool {
        let Some(holder) = self.get_leash_holder() else {
            return false;
        };
        let holder_id = holder.get_entity().entity_id;
        if held_by.is_some_and(|expected| expected != holder_id) {
            return false;
        }
        self.fire_unleash_events(Some(&holder)) && self.take_leash(Some(holder_id))
    }

    fn fire_unleash_events(&self, holder: Option<&Arc<dyn EntityBase>>) -> bool {
        let world = self.world.load();
        let Some(server) = world.server.upgrade() else {
            return true;
        };
        let mut event = crate::plugin::api::events::entity::entity_unleash::EntityUnleashEvent::new(
            self.entity_id,
            "unleashed".to_string(),
        );
        event = server.plugin_manager.fire_blocking(&server, event);
        if event.cancelled {
            return false;
        }
        if let Some(player) = holder.and_then(|holder| holder.get_player())
            && let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id)
        {
            let event = crate::plugin::api::events::player::player_unleash_entity::PlayerUnleashEntityEvent {
                player: player_arc,
                entity_id: self.entity_id,
                cancelled: false,
            };
            let event = server.plugin_manager.fire_blocking(&server, event);
            if event.cancelled {
                return false;
            }
        }
        true
    }

    fn release_leash_on_tick(&self, holder: Option<&Arc<dyn EntityBase>>, drop_lead: bool) {
        if self.leash_release_cancelled.load(Ordering::Relaxed) {
            return;
        }
        if !self.fire_unleash_events(holder) {
            self.leash_release_cancelled.store(true, Ordering::Relaxed);
            return;
        }
        let released = holder.map_or_else(
            || self.take_expired_leash(),
            |holder| self.take_leash(Some(holder.get_entity().entity_id)),
        );
        if released && drop_lead {
            self.spawn_at_location(ItemStack::new(1, &pumpkin_data::item::Item::LEAD));
        }
    }

    /// Cuts leash connections, returning `None` when there are none to cut.
    /// `Some(false)` means no cut succeeded; callers must not fall through to shearing.
    pub fn shear_off_all_leash_connections(
        &self,
        caller: &(impl EntityBase + ?Sized),
        player: Option<&Arc<Player>>,
    ) -> Option<bool> {
        let world = self.world.load();
        let area = BoundingBox::of_size(self.bounding_box.load().get_center(), 32.0, 32.0, 32.0);
        let connections = world
            .get_entities_at_box(&area)
            .into_iter()
            .filter(|entity| {
                entity
                    .get_entity()
                    .get_leash_holder()
                    .is_some_and(|holder| holder.get_entity().entity_id == self.entity_id)
            })
            .collect::<Vec<_>>();
        if self.get_leash_holder().is_none() && connections.is_empty() {
            return None;
        }
        let mut dropped = self.drop_leash_from_shearing(player, None);
        for entity in connections {
            dropped |= entity
                .get_entity()
                .drop_leash_from_shearing(player, Some(self.entity_id));
        }
        if dropped {
            world.emit_game_event_with_context(
                GameEvent::Shear.name(),
                self.pos.load(),
                &crate::world::game_event::GameEventContext {
                    source_entity: player.map(|player| player.as_ref() as &dyn EntityBase),
                },
            );
            world.play_sound(
                Sound::ItemShearsSnip,
                player.map_or_else(|| caller.sound_source(), |player| player.sound_source()),
                &self.block_pos.load().to_centered_f64(),
            );
        }
        Some(dropped)
    }

    fn drop_leash_from_shearing(&self, player: Option<&Arc<Player>>, held_by: Option<i32>) -> bool {
        let Some(holder) = self.get_leash_holder() else {
            return false;
        };
        if held_by.is_some_and(|holder_id| holder.get_entity().entity_id != holder_id) {
            return false;
        }
        if let Some(server) = self.world.load().server.upgrade() {
            let event = crate::plugin::api::events::entity::entity_unleash::EntityUnleashEvent::new(
                self.entity_id,
                "unleashed".to_string(),
            );
            let event = server.plugin_manager.fire_blocking(&server, event);
            if event.cancelled {
                return false;
            }
            if let Some(player) = player {
                let event = crate::plugin::api::events::player::player_unleash_entity::PlayerUnleashEntityEvent {
                    player: player.clone(),
                    entity_id: self.entity_id,
                    cancelled: false,
                };
                let event = server.plugin_manager.fire_blocking(&server, event);
                if event.cancelled {
                    return false;
                }
            }
        }
        if !self.take_leash(held_by) {
            return false;
        }
        self.spawn_at_location(ItemStack::new(1, &pumpkin_data::item::Item::LEAD));
        true
    }

    /// Removes the leash, returning whether this call took the connection.
    pub fn remove_leash(&self) -> bool {
        self.take_leash(None)
    }

    fn take_leash(&self, held_by: Option<i32>) -> bool {
        let mut leashed_to = self
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(holder_id) = held_by
            && leashed_to
                .as_ref()
                .and_then(Weak::upgrade)
                .is_none_or(|holder| holder.get_entity().entity_id != holder_id)
        {
            return false;
        }
        let old_holder = leashed_to.take();
        drop(leashed_to);
        if let Some(old_holder) = &old_holder {
            self.send_leash_removed();
            self.leash_release_cancelled.store(false, Ordering::Relaxed);
            if let Some(holder) = old_holder.upgrade() {
                holder.notify_leashee_removed();
            }
        }
        old_holder.is_some()
    }

    pub fn drop_leash(&self) -> bool {
        if !self.has_leash_data() {
            return false;
        }
        let removed = if self.is_leashed() {
            self.unleash()
        } else {
            self.fire_unleash_events(None) && self.take_expired_leash()
        };
        if !removed {
            return false;
        }
        self.spawn_at_location(ItemStack::new(1, &pumpkin_data::item::Item::LEAD));
        true
    }

    fn take_expired_leash(&self) -> bool {
        let mut leashed_to = self
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if leashed_to
            .as_ref()
            .is_none_or(|holder| holder.strong_count() > 0)
        {
            return false;
        }
        *leashed_to = None;
        drop(leashed_to);
        self.send_leash_removed();
        self.leash_release_cancelled.store(false, Ordering::Relaxed);
        true
    }

    fn has_leash_holder_link(&self) -> bool {
        self.leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    pub fn has_leash_data(&self) -> bool {
        self.has_leash_holder_link()
            || self
                .delayed_leash
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
    }

    /// Vanilla `Leashable.leashableLeashedTo`: entities in the 32 block box around this one
    /// that are leashed to it.
    pub fn leashable_leashed_to(&self) -> Vec<Arc<dyn EntityBase>> {
        leashable_in_area(
            &self.world.load(),
            self.bounding_box.load().get_center(),
            self.entity_id,
        )
    }

    pub fn drop_all_leash_connections(&self) -> bool {
        let world = self.world.load();
        let mut dropped = self.drop_leash();
        for other in self.leashable_leashed_to() {
            let other = other.get_entity();
            // Another player may have moved this leash since the scan.
            let still_held = other
                .get_leash_holder()
                .is_some_and(|holder| holder.get_entity().entity_id == self.entity_id);
            if still_held && other.drop_leash() {
                dropped = true;
            }
        }

        if dropped {
            if self.entity_type == &EntityType::LEASH_KNOT {
                self.remove();
            }
            world.emit_game_event(
                pumpkin_data::game_event::GameEvent::Shear.name(),
                self.pos.load(),
            );
        }
        dropped
    }

    fn send_leash_removed(&self) {
        let je_packet =
            pumpkin_protocol::java::client::play::CSetEntityLink::new(self.entity_id, -1, true);
        self.bedrock_flags
            .fetch_and(!(1i64 << entity_data_flag::LEASHED), Ordering::Relaxed);
        let mut bedrock_meta = SyncedActorDataList::new();
        bedrock_meta.set(entity_data_key::LEASH_HOLDER, MetadataValue::Int64(-1));
        bedrock_meta.set(
            entity_data_key::FLAGS,
            MetadataValue::Int64(self.bedrock_flags.load(Ordering::Relaxed)),
        );
        let be_packet = CSetActorData {
            target_runtime_id: VarULong(self.entity_id as u64),
            actor_data: bedrock_meta,
            synced_properties: PropertySyncData::default(),
            tick: VarULong(0),
        };

        self.world.load().broadcast_to_chunk_editioned(
            self.chunk_pos.load(),
            &je_packet,
            &be_packet,
        );
    }

    pub fn write_leash_data(&self, nbt: &mut NbtCompound) {
        let holder = self.get_leash_holder();
        let leash = match holder {
            Some(holder) if holder.get_entity().entity_type == &EntityType::LEASH_KNOT => {
                Some(DelayedLeash::Knot(holder.get_entity().block_pos.load()))
            }
            Some(holder) => Some(DelayedLeash::Entity(holder.get_entity().entity_uuid)),
            None => *self
                .delayed_leash
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        };
        match leash {
            Some(DelayedLeash::Entity(uuid)) => {
                let mut tag = NbtCompound::new();
                tag.put_uuid("UUID", uuid);
                nbt.put_compound("leash", tag);
            }
            Some(DelayedLeash::Knot(pos)) => nbt.put("leash", block_pos_to_tag(pos)),
            None => {}
        }
    }

    pub fn read_leash_data(&self, nbt: &NbtCompound) {
        let leash = match nbt.get("leash") {
            Some(NbtTag::Compound(tag)) => tag.get_uuid("UUID").map(DelayedLeash::Entity),
            Some(pos @ NbtTag::IntArray(_)) => block_pos_from_tag(pos).map(DelayedLeash::Knot),
            _ => None,
        };
        if leash.is_none() {
            self.remove_leash();
        }
        *self
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        *self
            .delayed_leash
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = leash;
    }

    fn restore_leash_from_save(&self, delayed: DelayedLeash) {
        let world = self.world.load();
        let holder = match delayed {
            DelayedLeash::Entity(uuid) => world.get_entity_by_uuid(uuid).or_else(|| {
                world
                    .get_player_by_uuid(uuid)
                    .map(|p| p as Arc<dyn EntityBase>)
            }),
            DelayedLeash::Knot(pos) => Some(LeashKnotEntity::get_or_create(&world, pos)),
        };
        if let Some(holder) = holder {
            self.attach_leash(&holder);
        } else if self.tick_count.load(Relaxed) > LEASH_RESTORE_TIMEOUT {
            self.spawn_at_location(ItemStack::new(1, &pumpkin_data::item::Item::LEAD));
            *self
                .delayed_leash
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        }
    }

    /// Applies leash snapping and the elastic pull. Mobs handle the other outcomes.
    pub fn tick_leash(&self, caller: &dyn EntityBase) -> LeashTick {
        let delayed = *self
            .delayed_leash
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(delayed) = delayed {
            self.restore_leash_from_save(delayed);
        }

        let holder = {
            let Ok(guard) = self.leashed_to.try_lock() else {
                return LeashTick::NotLeashed;
            };
            let Some(holder) = guard.as_ref() else {
                return LeashTick::NotLeashed;
            };
            holder.upgrade()
        };

        let world = self.world.load();
        let stale_holder = holder.clone();
        let Some(holder) = holder
            .filter(|holder| caller.can_interact_with_level() && holder.can_interact_with_level())
        else {
            self.release_leash_on_tick(
                stale_holder.as_ref(),
                world.level_info.load().game_rules.entity_drops,
            );
            return LeashTick::NotLeashed;
        };
        let holder_entity = holder.get_entity();
        if !Arc::ptr_eq(&holder_entity.world.load(), &world) {
            return LeashTick::NotLeashed;
        }

        let distance = self.leash_distance_to(holder_entity);
        if distance > caller.leash_snap_distance() {
            let holder_pos = holder_entity.pos.load();
            self.world
                .load()
                .play_sound(Sound::ItemLeadBreak, SoundCategory::Neutral, &holder_pos);
            // Vanilla `leashTooFarBehaviour`
            self.drop_leash();
            return LeashTick::Snapped;
        }
        let elastic_threshold = caller.leash_elastic_distance()
            - f64::from(holder_entity.width())
            - f64::from(self.width());
        let result = if distance > elastic_threshold
            && self.check_elastic_interactions(caller, holder.as_ref())
        {
            LeashTick::Elastic(holder)
        } else {
            LeashTick::CloseRange(holder)
        };

        let momentum = self.leash_angular_momentum.load();
        self.yaw.store(self.yaw.load() - momentum as f32);
        self.leash_angular_momentum
            .store(momentum * f64::from(self.leash_angular_friction()));
        result
    }

    /// Pulls the entity back with a damped spring on each attachment point beyond the slack.
    fn check_elastic_interactions(&self, caller: &dyn EntityBase, holder: &dyn EntityBase) -> bool {
        let quad_connection = holder.support_quad_leash_as_holder() && caller.support_quad_leash();
        let slack_distance = caller.leash_elastic_distance();
        let (entity_points, holder_points): (&[Vector3<f64>], &[Vector3<f64>]) = if quad_connection
        {
            (
                &SHARED_QUAD_ATTACHMENT_POINTS,
                &SHARED_QUAD_ATTACHMENT_POINTS,
            )
        } else {
            (&[ENTITY_ATTACHMENT_POINT], &[LEASHER_ATTACHMENT_POINT])
        };

        let holder_entity = holder.get_entity();
        let entity_movement = holder_movement(caller);
        let entity_dimensions = Vector3::new(
            f64::from(self.width()),
            f64::from(self.height()),
            f64::from(self.width()),
        );
        let holder_dimensions = Vector3::new(
            f64::from(holder_entity.width()),
            f64::from(holder_entity.height()),
            f64::from(holder_entity.width()),
        );
        let entity_yaw = self.yaw.load().to_radians();
        let holder_yaw = holder_entity.yaw.load().to_radians();
        let entity_pos = self.pos.load();
        let holder_pos = holder_entity.pos.load();

        let mut force = Vector3::new(0.0, 0.0, 0.0);
        let mut torque = 0.0;
        let mut pulled = false;
        for (entity_point, holder_point) in entity_points.iter().zip(holder_points) {
            let lever_arm = y_rot(
                entity_point.multiply(
                    entity_dimensions.x,
                    entity_dimensions.y,
                    entity_dimensions.z,
                ),
                -entity_yaw,
            );
            let entity_attach = entity_pos + lever_arm;
            let holder_attach = holder_pos
                + y_rot(
                    holder_point.multiply(
                        holder_dimensions.x,
                        holder_dimensions.y,
                        holder_dimensions.z,
                    ),
                    -holder_yaw,
                );
            let attach_distance = (holder_attach - entity_attach).length();
            if attach_distance < slack_distance {
                continue;
            }
            let mut displacement =
                (holder_attach - entity_attach).normalize() * (attach_distance - slack_distance);
            torque += lever_arm.z * displacement.x - lever_arm.x * displacement.z;
            if entity_movement.dot(&displacement) >= 0.0 {
                displacement = displacement * f64::from(0.3f32);
            }
            force += displacement;
            pulled = true;
        }
        if !pulled {
            return false;
        }

        let scale = if quad_connection { 0.25 } else { 1.0 };
        self.leash_angular_momentum.store(
            self.leash_angular_momentum.load() + LEASH_TORSIONAL_ELASTICITY * torque * scale,
        );
        let relative_velocity = holder_movement(holder) - known_movement(caller);
        let pull = (force * scale).multiply(
            LEASH_AXIS_ELASTICITY.x,
            LEASH_AXIS_ELASTICITY.y,
            LEASH_AXIS_ELASTICITY.z,
        ) + relative_velocity * LEASH_STIFFNESS;
        self.velocity.store(self.velocity.load() + pull);
        self.velocity_dirty.store(true, Relaxed);
        true
    }

    fn leash_angular_friction(&self) -> f32 {
        if self.on_ground.load(Ordering::Relaxed) {
            let below = self.get_block_pos_below_that_affects_my_movement();
            self.world.load().get_block(&below).slipperiness * 0.91
        } else if self.touching_water.load(Ordering::Relaxed)
            || self.touching_lava.load(Ordering::Relaxed)
        {
            0.8
        } else {
            0.91
        }
    }

    /// Caps a fall a taut leash has caught.
    pub fn check_fall_distance_accumulation(&self, living: &LivingEntity) {
        if self.velocity.load().y > -0.5 && living.fall_distance.load() > 1.0 {
            living.fall_distance.store(1.0);
        }
    }

    fn leash_distance_to(&self, other: &Self) -> f64 {
        let center = |entity: &Self| {
            let bounding_box = entity.bounding_box.load();
            (bounding_box.min + bounding_box.max) * 0.5
        };
        (center(other) - center(self)).length()
    }

    pub fn spawn_at_location(&self, stack: ItemStack) {
        self.spawn_at_location_with_offset(stack, Vector3::default());
    }

    pub fn spawn_at_location_with_offset(&self, stack: ItemStack, offset: Vector3<f64>) {
        if stack.is_empty() {
            return;
        }
        let world = self.world.load();
        let pos = self.pos.load() + offset;
        let entity = Self::new(world.clone(), pos, &EntityType::ITEM);
        if !world.fire_item_spawn_event(&entity, &stack) {
            return;
        }
        let item_entity = ItemEntity::new(entity, stack);
        world.spawn_entity(Arc::new(item_entity));
    }

    pub fn has_passengers(&self) -> bool {
        !self
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
    }

    pub fn has_passenger(&self, id: i32) -> bool {
        self.passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .any(|passenger| passenger.get_entity().entity_id == id)
    }

    pub fn has_vehicle(&self) -> bool {
        self.vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|v| v.strong_count() > 0)
    }

    pub fn get_vehicle(&self) -> Option<Arc<dyn EntityBase>> {
        self.vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(Weak::upgrade)
    }

    pub(crate) fn riding_revision(&self) -> u64 {
        self.riding_revision.load(Ordering::Acquire)
    }

    /// Vanilla `getRootVehicle`, `None` when not riding.
    pub fn get_root_vehicle(&self) -> Option<Arc<dyn EntityBase>> {
        let mut root = self.get_vehicle()?;
        while let Some(vehicle) = root.get_entity().get_vehicle() {
            root = vehicle;
        }
        Some(root)
    }

    /// Vanilla `isPassengerOfSameVehicle`: same root vehicle, or the same entity.
    #[must_use]
    pub fn is_passenger_of_same_vehicle(&self, other: &Self) -> bool {
        self.root_vehicle_id() == other.root_vehicle_id()
    }

    pub fn is_leashed(&self) -> bool {
        self.leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|h| h.strong_count() > 0)
    }

    pub fn get_leash_holder(&self) -> Option<Arc<dyn EntityBase>> {
        self.leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(Weak::upgrade)
    }

    pub fn add_passenger(&self, vehicle: Arc<dyn EntityBase>, passenger: Arc<dyn EntityBase>) {
        self.add_passenger_with_capacity(vehicle, passenger, usize::MAX);
    }

    pub(crate) fn add_passenger_with_capacity(
        &self,
        vehicle: Arc<dyn EntityBase>,
        passenger: Arc<dyn EntityBase>,
        capacity: usize,
    ) -> bool {
        self.add_passenger_if(vehicle, passenger, capacity, || true)
    }

    fn restore_player_passenger(&self, vehicle: Arc<dyn EntityBase>, player: Arc<Player>) {
        self.attach_passenger(
            vehicle,
            player,
            usize::MAX,
            PassengerMount::Restoration,
            None,
            || true,
        );
    }

    pub(crate) fn mount_vehicle_by_interaction(
        &self,
        player: &Arc<Player>,
        capacity: usize,
        vehicle_available: impl Fn() -> bool,
    ) -> bool {
        let world = self.world.load_full();
        let Some(vehicle) = world.get_entity_by_id(self.entity_id) else {
            return false;
        };
        let Some(passenger) = world.get_player_by_id(player.entity_id()) else {
            return false;
        };
        let is_java = player.client.java().is_some();
        let rider = player.get_entity();
        let eligible = || {
            !self.is_removed()
                && !rider.is_removed()
                && Arc::ptr_eq(&world, &self.world.load())
                && Arc::ptr_eq(&world, &rider.world.load())
                && player.can_interact_with_entity(vehicle.as_ref(), 1.0)
                && !rider.is_sneaking()
                && vehicle_available()
        };
        if !Arc::ptr_eq(&passenger, player)
            || !eligible()
            || (is_java && !player.can_ride(vehicle.as_ref()))
        {
            return false;
        }

        let revision = rider.riding_revision();
        let position = rider.pos.load();
        let pose = rider.pose.load();
        let teleport = player.teleport_id_count.load(Ordering::Acquire);
        let previous = rider.get_vehicle();
        let source = previous.as_ref().map(|source| source.get_entity());
        if let Some(source) = source {
            if !is_java || std::ptr::eq(source, self) {
                return false;
            }
            {
                let _commit = world
                    .riding_link_commits
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if !eligible()
                    || !self.can_attach_rider(&world, vehicle.as_ref(), rider)
                    || self
                        .passengers
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .len()
                        >= capacity
                {
                    return false;
                }
            }
            // Later mount vetoes can still abort the switch after the exit callbacks.
            if !source.allow_dismount(rider.entity_id, PassengerRemoval::Dismount) {
                return false;
            }
        }
        self.attach_passenger(
            vehicle.clone(),
            passenger,
            capacity,
            PassengerMount::Gameplay,
            source.map(|source| (source, revision)),
            || {
                eligible()
                    && rider.pos.load() == position
                    && rider.pose.load() == pose
                    && player.teleport_id_count.load(Ordering::Acquire) == teleport
            },
        )
    }

    /// Recheck caller-specific eligibility after the mount and pose callbacks.
    pub(crate) fn add_passenger_if(
        &self,
        vehicle: Arc<dyn EntityBase>,
        passenger: Arc<dyn EntityBase>,
        capacity: usize,
        still_eligible: impl FnOnce() -> bool,
    ) -> bool {
        self.attach_passenger(
            vehicle,
            passenger,
            capacity,
            PassengerMount::Gameplay,
            None,
            still_eligible,
        )
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "riding entry points pass owned Arc handles"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "The riding transaction validates and commits both links under the same guards."
    )]
    fn attach_passenger(
        &self,
        vehicle: Arc<dyn EntityBase>,
        passenger: Arc<dyn EntityBase>,
        capacity: usize,
        mount: PassengerMount,
        switch: Option<(&Self, u64)>,
        still_eligible: impl FnOnce() -> bool,
    ) -> bool {
        let passenger_entity = passenger.get_entity();
        if self.removal_reason.load().is_some()
            || passenger_entity.is_removed()
            || (switch.is_none() && passenger_entity.has_vehicle())
        {
            return false;
        }
        let riding_revision = switch.map_or_else(
            || passenger_entity.riding_revision(),
            |(_, revision)| revision,
        );
        if passenger_entity.riding_revision() != riding_revision {
            return false;
        }
        if !self.allow_mount(passenger_entity.entity_id) {
            return false;
        }
        if !self.uses_vehicle_collisions()
            && passenger.get_mob().is_some()
            && passenger_entity.has_leash_holder_link()
            && !passenger_entity.drop_leash()
        {
            return false;
        }
        let stand = self.uses_vehicle_collisions()
            && passenger_entity.pose.load() != EntityPose::Standing
            && passenger_entity.allow_pose_change(EntityPose::Standing);

        let world = self.world.load_full();
        let riding_links = world
            .riding_link_commits
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !still_eligible() || !self.can_attach_rider(&world, vehicle.as_ref(), passenger_entity) {
            return false;
        }
        // Only a switch holds two carrier lists, and switches share the world commit lock.
        let mut source_passengers = switch.map(|(source, _)| {
            source
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        let mut passengers = self
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // World removal marks the vehicle before taking its passenger snapshot.
        if self.removal_reason.load().is_some() || passengers.len() >= capacity {
            return false;
        }
        let mut rider_vehicle = passenger_entity
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if passenger_entity.is_removed()
            || passenger_entity.riding_revision.load(Relaxed) != riding_revision
            || match (switch, rider_vehicle.as_ref().and_then(Weak::upgrade)) {
                (None, None) => false,
                (Some((source, _)), Some(current)) => {
                    !std::ptr::eq(source, current.get_entity())
                        || source.is_removed()
                        || !Arc::ptr_eq(&world, &source.world.load())
                }
                _ => true,
            }
        {
            return false;
        }
        let source_update = if let Some(source_passengers) = source_passengers.as_mut() {
            let Some(index) = source_passengers
                .iter()
                .position(|current| Arc::ptr_eq(current, &passenger))
            else {
                return false;
            };
            let previous_first = source_passengers
                .first()
                .map(|first| first.get_entity().entity_id);
            source_passengers.remove(index);
            passenger_entity.riding_cooldown.store(60, Relaxed);
            let ids = source_passengers
                .iter()
                .map(|p| VarInt(p.get_entity().entity_id))
                .collect::<Vec<_>>();
            Some((ids, previous_first))
        } else {
            None
        };
        if stand {
            passenger_entity.pose.store(EntityPose::Standing);
        }
        *rider_vehicle = Some(Arc::downgrade(&vehicle));
        passenger_entity
            .bedrock_flags
            .fetch_or(1i64 << entity_data_flag::RIDING, Ordering::Relaxed);
        // A player takes the first vehicle seat when a non-player occupies it.
        let displaced_driver = passengers
            .first()
            .filter(|first| {
                self.uses_vehicle_collisions()
                    && passenger.get_player().is_some()
                    && first.get_player().is_none()
            })
            .map(|first| first.get_entity().entity_id);
        if displaced_driver.is_some() {
            passengers.insert(0, passenger.clone());
        } else {
            passengers.push(passenger.clone());
        }
        let riding_revision = passenger_entity
            .riding_revision
            .fetch_add(1, Ordering::Release)
            .wrapping_add(1);
        let switch_placement_baseline = switch
            .and_then(|_| passenger.get_player())
            .map(Self::java_rider_placement_baseline);
        drop(rider_vehicle);
        drop(passengers);
        drop(source_passengers);
        drop(riding_links);

        if let (Some((source, _)), Some((ids, previous_first))) = (switch, source_update) {
            source.finish_passenger_switch_exit(&world, passenger.as_ref(), &ids, previous_first);
        }

        self.finish_passenger_mount(
            vehicle.as_ref(),
            passenger.as_ref(),
            &world,
            CommittedPassengerMount {
                revision: riding_revision,
                stood: stand,
                displaced_driver,
                switch_placement_baseline,
            },
            mount,
        );
        true
    }

    /// Complete the mount before publishing its passenger list. Callbacks may replace the link.
    fn finish_passenger_mount(
        &self,
        vehicle: &dyn EntityBase,
        passenger: &dyn EntityBase,
        world: &Arc<World>,
        committed: CommittedPassengerMount,
        mount: PassengerMount,
    ) {
        let CommittedPassengerMount {
            revision,
            stood,
            displaced_driver,
            switch_placement_baseline,
        } = committed;
        let rider = passenger.get_entity();
        // The pose was committed before releasing the riding locks. Refresh it even
        // if the mount completion loses ownership of the attachment in the meantime.
        let synced_pose = stood && rider.pose.load() == EntityPose::Standing;
        if synced_pose {
            rider.sync_pose(EntityPose::Standing);
        }
        let publish_pose = || {
            if synced_pose
                && !rider.is_removed()
                && Arc::ptr_eq(world, &rider.world.load())
                && rider.pose.load() == EntityPose::Standing
            {
                rider.send_bedrock_pose(world, EntityPose::Standing);
            }
        };
        let still_boarded = || {
            !self.is_removed()
                && !rider.is_removed()
                && rider.riding_revision() == revision
                && Arc::ptr_eq(world, &self.world.load())
                && Arc::ptr_eq(world, &rider.world.load())
        };
        if !still_boarded() {
            publish_pose();
            return;
        }
        let vehicle_mount = self.uses_vehicle_collisions();
        let java_rider = passenger
            .get_player()
            .filter(|player| vehicle_mount && player.client.java().is_some());
        let java_placement = java_rider.map(|player| {
            let (position, teleport_count) = switch_placement_baseline
                .unwrap_or_else(|| Self::java_rider_placement_baseline(player));
            (player, position, teleport_count)
        });
        // Java restores a player's saved attachment with seat and link updates, while
        // suppressing the mount game event and riding advancement triggers.
        if vehicle_mount && mount == PassengerMount::Gameplay {
            world.emit_game_event_with_context(
                GameEvent::EntityMount.name(),
                self.pos.load(),
                &crate::world::game_event::GameEventContext {
                    source_entity: Some(passenger),
                },
            );
            if !self.is_removed() && Arc::ptr_eq(world, &self.world.load()) {
                self.notify_started_riding(world);
            }
        }
        if vehicle_mount && still_boarded() && passenger.get_mob().is_some() && rider.is_leashed() {
            rider.drop_leash();
        }
        publish_pose();

        if still_boarded()
            && let Some(mob) = passenger.get_mob()
        {
            mob.on_start_riding();
        }
        rider.send_bedrock_riding_flags(passenger, revision);
        if let Some((player, before_mount, teleport_count)) = java_placement {
            Self::place_java_rider(
                vehicle,
                passenger,
                player,
                before_mount,
                teleport_count,
                &still_boarded,
            );
        }
        if !still_boarded() {
            return;
        }
        let passenger_ids: Vec<_> = self
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|passenger| VarInt(passenger.get_entity().entity_id))
            .collect();
        let Some(index) = passenger_ids.iter().position(|id| id.0 == rider.entity_id) else {
            return;
        };
        let be_link =
            self.bedrock_passenger_link(rider.entity_id, if index == 0 { 1 } else { 2 }, true);
        let changed_role = displaced_driver.map(|id| self.bedrock_passenger_link(id, 2, false));
        self.send_passenger_update(
            world,
            java_rider,
            &passenger_ids,
            &[Some(be_link), changed_role],
        );
        if !vehicle_mount && mount == PassengerMount::Gameplay && still_boarded() {
            self.notify_started_riding(world);
        }
    }

    fn java_rider_placement_baseline(player: &Player) -> (Vector3<f64>, i32) {
        (
            player.get_entity().pos.load(),
            player.teleport_id_count.load(Ordering::Acquire),
        )
    }

    fn place_java_rider(
        vehicle: &dyn EntityBase,
        passenger: &dyn EntityBase,
        player: &Player,
        before_mount: Vector3<f64>,
        teleport_count: i32,
        still_boarded: &impl Fn() -> bool,
    ) {
        let Some(client) = player.client.java() else {
            return;
        };
        let rider = passenger.get_entity();
        let mut awaiting = player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !still_boarded() || rider.pos.load() != before_mount {
            return;
        }
        let Some(seat) = rider_seat_position(vehicle, passenger) else {
            return;
        };
        let rider_vehicle = rider
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !still_boarded()
            || player
                .teleport_id_count
                .compare_exchange(
                    teleport_count,
                    teleport_count.wrapping_add(1),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_err()
        {
            return;
        }
        let id = VarInt(teleport_count.wrapping_add(1));
        *awaiting = Some((id, seat));
        rider.set_pos(seat);
        vehicle.on_rider_positioned(passenger);
        rider.set_rotation(rider.yaw.load(), rider.pitch.load());
        rider.velocity.store(Vector3::default());
        rider.movement.store(Vector3::default());
        drop(rider_vehicle);
        drop(awaiting);

        let packet = CPlayerPosition::new(
            id,
            seat,
            Vector3::new(0.0, 0.0, 0.0),
            0.0,
            0.0,
            vec![PositionFlag::YRot, PositionFlag::XRot],
        );
        if !client.try_send_riding_position(&packet, player, before_mount, || {
            still_boarded()
                && rider.pos.load() == seat
                && player.teleport_id_count.load(Ordering::Acquire) == id.0
        }) {
            let mut awaiting = player
                .awaiting_teleport
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if awaiting.as_ref().is_some_and(|(pending, _)| *pending == id) {
                *awaiting = None;
            }
        }
    }

    pub(crate) fn board_vehicle_contact(
        &self,
        vehicle: &Arc<dyn EntityBase>,
        passenger: &Arc<dyn EntityBase>,
        capacity: usize,
        still_current: &dyn Fn() -> bool,
        in_contact: impl FnOnce() -> bool,
    ) {
        self.add_passenger_if(vehicle.clone(), passenger.clone(), capacity, || {
            still_current() && passenger.can_ride(vehicle.as_ref()) && in_contact()
        });
    }

    // Called under the world's commit lock. Only additions can introduce a cycle.
    fn can_attach_rider(&self, world: &Arc<World>, vehicle: &dyn EntityBase, rider: &Self) -> bool {
        if !std::ptr::eq(vehicle.get_entity(), self)
            || std::ptr::eq(self, rider)
            || !Arc::ptr_eq(world, &self.world.load())
            || !Arc::ptr_eq(world, &rider.world.load())
        {
            return false;
        }
        let mut ancestor = self.get_vehicle();
        while let Some(parent) = ancestor {
            if std::ptr::eq(parent.get_entity(), rider) {
                return false;
            }
            ancestor = parent.get_entity().get_vehicle();
        }
        true
    }

    fn allow_mount(&self, passenger_id: i32) -> bool {
        let mut mount_event =
            crate::plugin::api::events::entity::entity_mount::EntityMountEvent::new(
                passenger_id,
                self.entity_id,
            );
        let mut vehicle_enter =
            crate::plugin::api::events::vehicle::vehicle_enter::VehicleEnterEvent::new(
                self.entity_id,
                passenger_id,
            );
        if let Some(server) = self.world.load().server.upgrade() {
            mount_event = server.plugin_manager.fire_blocking(&server, mount_event);
            vehicle_enter = server.plugin_manager.fire_blocking(&server, vehicle_enter);
        }
        !mount_event.cancelled && !vehicle_enter.cancelled
    }

    fn notify_started_riding(&self, world: &Arc<World>) {
        use crate::entity::player::advancement::trigger::AdvancementTrigger;

        fn collect_players(
            vehicle: &Entity,
            recipients: &mut Vec<(Arc<dyn EntityBase>, AdvancementTrigger)>,
        ) {
            let passengers = vehicle
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            for passenger in &passengers {
                let rider = passenger.get_entity();
                if passenger.get_player().is_some() {
                    recipients.push((
                        passenger.clone(),
                        AdvancementTrigger::StartedRiding {
                            vehicle_type: Some(format!(
                                "minecraft:{}",
                                vehicle.entity_type.resource_name
                            )),
                            passenger_type: passengers
                                .iter()
                                .find(|other| !Arc::ptr_eq(other, passenger))
                                .map(|other| {
                                    format!(
                                        "minecraft:{}",
                                        other.get_entity().entity_type.resource_name
                                    )
                                }),
                        },
                    ));
                }
                collect_players(rider, recipients);
            }
        }

        let mut recipients = Vec::new();
        collect_players(self, &mut recipients);
        for (rider, trigger) in recipients {
            let entity = rider.get_entity();
            if !entity.is_removed()
                && Arc::ptr_eq(world, &entity.world.load())
                && let Some(player) = rider.get_player()
            {
                player.trigger_advancement(trigger);
            }
        }
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "callers pass owned Arc for vehicle and passenger"
    )]
    pub fn add_passenger_silently(
        &self,
        vehicle: Arc<dyn EntityBase>,
        passenger: Arc<dyn EntityBase>,
    ) {
        let world = self.world.load_full();
        let riding_links = world
            .riding_link_commits
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let rider = passenger.get_entity();
        if !self.can_attach_rider(&world, vehicle.as_ref(), rider) {
            return;
        }
        let mut passengers = self
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut rider_vehicle = rider
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.is_removed()
            || rider.is_removed()
            || rider_vehicle.as_ref().and_then(Weak::upgrade).is_some()
        {
            return;
        }
        *rider_vehicle = Some(Arc::downgrade(&vehicle));
        rider
            .bedrock_flags
            .fetch_or(1i64 << entity_data_flag::RIDING, Relaxed);
        let riding_revision = rider
            .riding_revision
            .fetch_add(1, Ordering::Release)
            .wrapping_add(1);
        passengers.push(passenger.clone());
        drop(rider_vehicle);
        drop(passengers);
        drop(riding_links);
        if rider.riding_revision() == riding_revision
            && let Some(mob) = passenger.get_mob()
        {
            mob.on_start_riding();
        }
    }

    fn write_passengers(&self, nbt: &mut NbtCompound) {
        let passengers = self
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let passengers: Vec<NbtTag> = passengers
            .iter()
            .filter(|passenger| {
                passenger.get_player().is_none() && !passenger.get_entity().is_removed()
            })
            .map(|passenger| {
                let mut tag = NbtCompound::new();
                passenger.write_nbt(&mut tag);
                NbtTag::Compound(tag)
            })
            .collect();
        if !passengers.is_empty() {
            nbt.put_list("Passengers", passengers);
        }
    }

    pub(crate) fn remove_passenger_on_disconnect(&self, passenger_id: i32) {
        let mut passengers = self
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous_first = passengers.first().map(|p| p.get_entity().entity_id);
        if let Some(index) = passengers
            .iter()
            .position(|passenger| passenger.get_entity().entity_id == passenger_id)
        {
            let passenger = passengers.remove(index);
            let rider = passenger.get_entity();
            let mut vehicle = rider
                .vehicle
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if vehicle
                .as_ref()
                .and_then(Weak::upgrade)
                .is_some_and(|current| std::ptr::eq(current.get_entity(), self))
            {
                *vehicle = None;
                rider
                    .bedrock_flags
                    .fetch_and(!(1i64 << entity_data_flag::RIDING), Relaxed);
                rider.riding_revision.fetch_add(1, Ordering::Release);
            }
        }

        let passenger_ids: Vec<VarInt> = passengers
            .iter()
            .map(|passenger| VarInt(passenger.get_entity().entity_id))
            .collect();
        drop(passengers);

        let links = self.bedrock_links_after_removal(passenger_id, previous_first, &passenger_ids);
        self.send_passenger_update(&self.world.load(), None, &passenger_ids, &links);
    }

    pub(crate) fn bedrock_passenger_link(
        &self,
        passenger_id: i32,
        link_type: u8,
        rider_initiated: bool,
    ) -> CSetActorLink {
        CSetActorLink {
            link: ActorLink {
                ridden_unique_id: VarLong(i64::from(self.entity_id)),
                rider_unique_id: VarLong(i64::from(passenger_id)),
                link_type,
                immediate: true,
                rider_initiated,
                vehicle_angular_velocity: 0.0,
            },
        }
    }

    fn bedrock_links_after_removal(
        &self,
        passenger_id: i32,
        previous_first: Option<i32>,
        ids: &[VarInt],
    ) -> [Option<CSetActorLink>; 2] {
        let unlink = self.bedrock_passenger_link(passenger_id, 0, false);
        let promotion = ids
            .first()
            .filter(|_| previous_first == Some(passenger_id))
            .map(|first| self.bedrock_passenger_link(first.0, 1, false));
        [Some(unlink), promotion]
    }

    pub(crate) fn send_passenger_update_to(
        &self,
        world: &World,
        recipient: &Player,
        ids: &[VarInt],
        links: &[Option<CSetActorLink>],
    ) {
        self.send_passenger_update_to_if(
            world,
            recipient,
            ids,
            ids,
            links,
            |_| {
                Some((
                    (),
                    self.passengers
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner),
                ))
            },
            |_| true,
        );
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Both editions share the pairing snapshot and observer guards."
    )]
    pub(crate) fn send_passenger_update_to_if<'a, T>(
        &'a self,
        world: &World,
        recipient: &Player,
        ids: &[VarInt],
        visible_ids: &[VarInt],
        links: &[Option<CSetActorLink>],
        lock_current: impl Fn(
            Option<i32>,
        )
            -> Option<(T, std::sync::MutexGuard<'a, Vec<Arc<dyn EntityBase>>>)>,
        keep_stale_extra: impl Fn(&[u8]) -> bool,
    ) {
        let in_world = || {
            std::ptr::eq(recipient.world().as_ref(), world)
                && std::ptr::eq(self.world.load().as_ref(), world)
        };
        if let Some(client) = recipient.client.java()
            && let Ok(data) =
                client.serialize_packet(&CSetPassengers::new(VarInt(self.entity_id), visible_ids))
        {
            let protocol = client.version.load();
            client.try_enqueue_packet_data_filtering(
                data,
                || {
                    let (session, current) = lock_current(None)?;
                    (in_world()
                        && current
                            .iter()
                            .map(|p| VarInt(p.get_entity().entity_id))
                            .eq(ids.iter().copied()))
                    .then_some((session, current))
                },
                |data| {
                    let mut affected = ids.to_vec();
                    affected.extend(links.iter().flatten().filter_map(|link| {
                        (link.link.link_type == 0)
                            .then(|| i32::try_from(link.link.rider_unique_id.0).ok())
                            .flatten()
                            .map(VarInt)
                    }));
                    (keep_stale_extra(&data)
                        && !Self::java_passenger_update_overlaps(
                            &data,
                            protocol,
                            self.entity_id,
                            &affected,
                        ))
                    .then_some(data)
                },
            );
        } else if let Some(client) = recipient.client.bedrock() {
            for link in links.iter().flatten() {
                let Ok(data) = client.serialize_packet(link) else {
                    continue;
                };
                let rider = link.link.rider_unique_id.0;
                client.try_enqueue_packet_data_filtering(
                    data,
                    || {
                        let (session, current) = lock_current(i32::try_from(rider).ok())?;
                        let role = current
                            .iter()
                            .position(|passenger| {
                                i64::from(passenger.get_entity().entity_id) == rider
                            })
                            .map_or(0, |index| if index == 0 { 1 } else { 2 });
                        (in_world() && role == link.link.link_type).then_some((session, current))
                    },
                    |data| {
                        let mut payload = data.as_ref();
                        let overlaps = VarUInt::read(&mut payload)
                            .is_ok_and(|id| id.0 == CSetActorLink::PACKET_ID as u32)
                            && VarLong::read(&mut payload).is_ok_and(|carrier| {
                                carrier == link.link.ridden_unique_id
                                    || VarLong::read(&mut payload)
                                        .is_ok_and(|other| other.0 == rider)
                            });
                        (!overlaps && keep_stale_extra(&data)).then_some(data)
                    },
                );
            }
        }
    }

    pub(crate) fn java_passenger_update_overlaps(
        mut payload: &[u8],
        protocol: JavaMinecraftVersion,
        carrier: i32,
        riders: &[VarInt],
    ) -> bool {
        use pumpkin_protocol::{ServerPacket, java::client::play::CSetEntityLink};

        let Ok(packet_id) = payload.get_var_int() else {
            return false;
        };
        if protocol <= JavaMinecraftVersion::V_1_8 && packet_id.0 == CSetEntityLink::to_id(protocol)
        {
            // These versions share the attach packet between riding and leashing.
            return CSetEntityLink::read(&mut payload, &protocol).is_ok_and(|link| {
                !link.leash
                    && (link.holding_entity_id == carrier
                        || riders.contains(&VarInt(link.attached_entity_id)))
            });
        }
        if packet_id.0 != CSetPassengers::to_id(protocol) {
            return false;
        }
        if payload.get_var_int().is_ok_and(|id| id.0 == carrier) {
            return true;
        }
        payload.get_var_int().is_ok_and(|count| {
            (0..count.0)
                .map_while(|_| payload.get_var_int().ok())
                .any(|id| riders.contains(&id))
        })
    }

    fn send_passenger_update(
        &self,
        world: &World,
        direct_recipient: Option<&Player>,
        ids: &[VarInt],
        links: &[Option<CSetActorLink>],
    ) {
        if let Some(player) = direct_recipient {
            self.send_passenger_update_to(world, player, ids, links);
        }
        let chunk = self.chunk_pos.load();
        for player in world.players.load().iter() {
            if direct_recipient
                .is_none_or(|direct| direct.get_entity().entity_id != player.get_entity().entity_id)
                && player
                    .watched_section
                    .load()
                    .is_within_distance(chunk.x, chunk.y)
            {
                self.send_passenger_update_to(world, player, ids, links);
            }
        }
    }

    pub fn remove_passenger_sync(&self, passenger_id: i32) {
        self.remove_passenger_on_disconnect(passenger_id);
    }

    pub fn remove_passenger(&self, passenger_id: i32) {
        self.remove_passenger_internal(passenger_id, PassengerRemoval::Dismount, None);
    }

    pub fn remove_passenger_before_teleport(&self, passenger_id: i32) {
        self.remove_passenger_internal(passenger_id, PassengerRemoval::Teleport, None);
    }

    pub(crate) fn remove_passenger_after_vehicle_removal(&self, passenger_id: i32) {
        self.remove_passenger_internal(passenger_id, PassengerRemoval::VehicleRemoved, None);
    }

    pub(crate) fn eject_vehicle_passengers(&self) -> bool {
        let world = self.world.load_full();
        let position = self.pos.load();
        let epoch = self.vehicle_motion_epoch();
        let still_current = || {
            !self.is_removed()
                && self.pos.load() == position
                && self.vehicle_motion_epoch() == epoch
                && Arc::ptr_eq(&world, &self.world.load())
        };
        let passengers: Vec<_> = self
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .rev()
            .map(|passenger| {
                let rider = passenger.get_entity();
                (rider.entity_id, rider.riding_revision())
            })
            .collect();
        for (id, revision) in passengers {
            if !still_current() {
                return false;
            }
            self.remove_passenger_internal(id, PassengerRemoval::Dismount, Some(revision));
        }
        still_current()
    }

    fn legacy_dismount_position(
        &self,
        world: &World,
        passenger_entity: &Self,
        vehicle_box: BoundingBox,
    ) -> Vector3<f64> {
        let offsets = dismount_helper::offsets_for_yaw(self.yaw.load());

        let target_block_y = vehicle_box.max.y.floor() as i32;
        let below_pos = BlockPos(Vector3::new(
            self.pos.load().x.floor() as i32,
            target_block_y - 1,
            self.pos.load().z.floor() as i32,
        ));

        let below_state_id = world.get_block_state_id(&below_pos);
        // Vanilla: isWater checks specifically for water fluid, not any fluid
        let is_water =
            Fluid::from_state_id(below_state_id).is_some_and(|f| f.matches_type(&Fluid::WATER));

        let fallback_pos = Vector3::new(self.pos.load().x, vehicle_box.max.y, self.pos.load().z);

        if is_water {
            fallback_pos
        } else {
            // Vanilla checks Standing, Crouching, Swimming poses and their respective height checks
            let poses_and_heights = [
                (EntityPose::Standing, vec![0, 1, -1]),
                (EntityPose::Crouching, vec![0, 1, -1]),
                (EntityPose::Swimming, vec![0, 1]),
            ];

            let vehicle_block_pos = self.block_pos.load();
            let mut found = None;

            'search: for (pose, y_offsets) in poses_and_heights {
                let dims = passenger_entity.get_dimensions(pose);

                for y_offset in y_offsets {
                    for offset in offsets {
                        let target_block_x = vehicle_block_pos.0.x + offset.x;
                        let target_block_y = vehicle_block_pos.0.y + y_offset;
                        let target_block_z = vehicle_block_pos.0.z + offset.z;

                        let target_pos =
                            BlockPos(Vector3::new(target_block_x, target_block_y, target_block_z));
                        let height = world.get_dismount_height(&target_pos);

                        if height.is_finite() && height < 1.0 {
                            let location = Vector3::new(
                                f64::from(target_block_x) + 0.5,
                                f64::from(target_block_y) + height,
                                f64::from(target_block_z) + 0.5,
                            );

                            let bbox = BoundingBox::new_from_pos(
                                location.x, location.y, location.z, &dims,
                            );
                            if world.is_space_empty(bbox) {
                                found = Some((location, pose));
                                break 'search;
                            }
                        }
                    }
                }
            }

            if let Some((pos, pose)) = found {
                if pose != EntityPose::Standing {
                    passenger_entity.set_pose(pose);
                }
                pos
            } else {
                // Try dismounting directly on top of the vehicle as fallback
                let mut found_fallback = None;
                let vehicle_top = vehicle_box.max.y;

                let poses = [
                    EntityPose::Standing,
                    EntityPose::Crouching,
                    EntityPose::Swimming,
                ];

                for pose in poses {
                    let dims = passenger_entity.get_dimensions(pose);
                    let bbox = BoundingBox::new_from_pos(
                        self.pos.load().x,
                        vehicle_top,
                        self.pos.load().z,
                        &dims,
                    );
                    if world.is_space_empty(bbox) {
                        found_fallback = Some((
                            Vector3::new(self.pos.load().x, vehicle_top, self.pos.load().z),
                            pose,
                        ));
                        break;
                    }
                }

                if let Some((pos, pose)) = found_fallback {
                    if pose != EntityPose::Standing {
                        passenger_entity.set_pose(pose);
                    }
                    pos
                } else {
                    fallback_pos
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn allow_dismount(&self, passenger_id: i32, reason: PassengerRemoval) -> bool {
        let mut dismount_event =
            crate::plugin::api::events::entity::entity_dismount::EntityDismountEvent::new(
                passenger_id,
                self.entity_id,
            );
        let mut vehicle_exit =
            crate::plugin::api::events::vehicle::vehicle_exit::VehicleExitEvent::new(
                self.entity_id,
                passenger_id,
            );
        if let Some(server) = self.world.load().server.upgrade() {
            dismount_event = server.plugin_manager.fire_blocking(&server, dismount_event);
            vehicle_exit = server.plugin_manager.fire_blocking(&server, vehicle_exit);
        }
        reason == PassengerRemoval::VehicleRemoved
            || !(dismount_event.cancelled || vehicle_exit.cancelled)
    }

    fn finish_passenger_switch_exit(
        &self,
        world: &World,
        rider: &dyn EntityBase,
        ids: &[VarInt],
        previous_first: Option<i32>,
    ) {
        let rider_entity = rider.get_entity();
        if self.uses_vehicle_collisions()
            && std::ptr::eq(world, rider_entity.world.load().as_ref())
            && rider_entity
                .removal_reason
                .load()
                .is_none_or(|reason| reason.should_destroy())
        {
            world.emit_game_event_with_context(
                GameEvent::EntityDismount.name(),
                self.pos.load(),
                &crate::world::game_event::GameEventContext {
                    source_entity: Some(rider),
                },
            );
        }
        // The rider remains mounted throughout the switch; destination mount publishes the riding flags.
        let links =
            self.bedrock_links_after_removal(rider.get_entity().entity_id, previous_first, ids);
        self.send_passenger_update(world, rider.get_player(), ids, &links);
    }

    #[expect(clippy::too_many_lines)]
    fn remove_passenger_internal(
        &self,
        passenger_id: i32,
        reason: PassengerRemoval,
        expected_revision: Option<u64>,
    ) -> bool {
        let previous_attachment = self
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .find(|p| p.get_entity().entity_id == passenger_id)
            .map(|p| {
                (
                    p.clone(),
                    p.get_entity().riding_revision.load(Ordering::Acquire),
                )
            });
        let Some((previous_rider, riding_revision)) = previous_attachment else {
            return false;
        };
        if expected_revision.is_some_and(|expected| expected != riding_revision) {
            return false;
        }
        let previous_entity = previous_rider.get_entity();
        let previous_world = previous_entity.world.load_full();
        let previous_position = previous_entity.pos.load();
        let previous_pose = previous_entity.pose.load();
        let previous_teleport = previous_rider
            .get_player()
            .map(|player| player.teleport_id_count.load(Ordering::Acquire));
        if !self.allow_dismount(passenger_id, reason) {
            return false;
        }
        let reposition = reason != PassengerRemoval::Teleport;

        let (removed_passenger, passenger_ids, previous_first) = {
            let mut passengers = self
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let previous_first = passengers.first().map(|p| p.get_entity().entity_id);
            let detached = passengers
                .iter()
                .position(|p| p.get_entity().entity_id == passenger_id)
                .and_then(|idx| {
                    let passenger = passengers[idx].clone();
                    let rider = passenger.get_entity();
                    let mut vehicle = rider
                        .vehicle
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let Some(carrier) = vehicle
                        .as_ref()
                        .and_then(Weak::upgrade)
                        .filter(|current| std::ptr::eq(current.get_entity(), self))
                    else {
                        passengers.remove(idx);
                        return None;
                    };
                    if rider.riding_revision.load(Relaxed) != riding_revision
                        && reason != PassengerRemoval::VehicleRemoved
                    {
                        return None;
                    }
                    passengers.remove(idx);
                    *vehicle = None;
                    // Vanilla prevents immediate re-mounting and clears the rider flag.
                    rider.riding_cooldown.store(60, Relaxed);
                    rider
                        .bedrock_flags
                        .fetch_and(!(1i64 << entity_data_flag::RIDING), Relaxed);
                    let revision = rider
                        .riding_revision
                        .fetch_add(1, Ordering::Release)
                        .wrapping_add(1);
                    drop(vehicle);
                    Some((passenger, revision, carrier))
                });

            let passenger_ids: Vec<VarInt> = passengers
                .iter()
                .map(|p| VarInt(p.get_entity().entity_id))
                .collect();
            (detached, passenger_ids, previous_first)
        };

        if let Some((passenger, riding_revision, carrier)) = removed_passenger {
            let vehicle_box = self.bounding_box.load();
            let exit_position = self.pos.load();
            let passenger_entity = passenger.get_entity();
            let world = self.world.load_full();
            let rider_position = passenger_entity.pos.load();
            let rider_pose = passenger_entity.pose.load();
            let vehicle_exit = self.uses_vehicle_collisions();
            let guard_placement = vehicle_exit;
            // Exit listeners can move a rider before the old attachment is unlinked.
            let reposition = reposition
                && (!guard_placement
                    || (rider_position == previous_position
                        && rider_pose == previous_pose
                        && Arc::ptr_eq(&previous_world, &passenger_entity.world.load())
                        && previous_teleport.is_none_or(|id| {
                            passenger.get_player().is_some_and(|player| {
                                player.teleport_id_count.load(Ordering::Acquire) == id
                            })
                        })));

            // Pre-allocate teleport ID and block movement packets BEFORE sending
            // CSetPassengers. This prevents a race condition where the client receives
            // the dismount packet, sends stale position packets from the old riding
            // position, and the server processes them before the teleport arrives.
            let teleport_id = if reposition && let Some(player) = passenger.get_player() {
                let id = player
                    .teleport_id_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    + 1;
                // Use fallback position as placeholder — updated below with real position
                let placeholder = Vector3::new(exit_position.x, vehicle_box.max.y, exit_position.z);
                *player
                    .awaiting_teleport
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                    Some((id.into(), placeholder));
                Some(id)
            } else {
                None
            };

            if vehicle_exit
                && Arc::ptr_eq(&world, &passenger_entity.world.load())
                && passenger_entity
                    .removal_reason
                    .load()
                    .is_none_or(|reason| reason.should_destroy())
            {
                world.emit_game_event_with_context(
                    GameEvent::EntityDismount.name(),
                    exit_position,
                    &crate::world::game_event::GameEventContext {
                        source_entity: Some(passenger.as_ref()),
                    },
                );
            }

            passenger_entity.send_bedrock_riding_flags(passenger.as_ref(), riding_revision);

            // Send CSetPassengers directly to the dismounting player before broadcasting it.
            let links =
                self.bedrock_links_after_removal(passenger_id, previous_first, &passenger_ids);
            self.send_passenger_update(&world, passenger.get_player(), &passenger_ids, &links);

            if !reposition {
                return false;
            }

            let still_detached = || {
                !passenger_entity.is_removed()
                    && passenger_entity.riding_revision.load(Ordering::Acquire) == riding_revision
                    && Arc::ptr_eq(&world, &passenger_entity.world.load())
                    && teleport_id.is_none_or(|id| {
                        passenger.get_player().is_some_and(|player| {
                            player.teleport_id_count.load(Ordering::Acquire) == id
                        })
                    })
            };
            let still_owns_placement =
                || still_detached() && passenger_entity.pos.load() == rider_position;
            let dismount_pos = if vehicle_exit
                && (!still_owns_placement() || passenger_entity.pose.load() != rider_pose)
            {
                None
            } else if vehicle_exit {
                carrier.vehicle_dismount_position(passenger.as_ref(), &still_owns_placement)
            } else {
                Some(self.legacy_dismount_position(&world, passenger_entity, vehicle_box))
            };

            let dismount_pos = dismount_pos.filter(|_| !guard_placement || still_owns_placement());

            // Phase 2: Teleport to safety (unblocks movement)
            if let Some(player) = passenger.get_player() {
                if let Some(id) = teleport_id {
                    let mut awaiting = player
                        .awaiting_teleport
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if guard_placement
                        && awaiting
                            .as_ref()
                            .is_none_or(|(pending, _)| *pending != id.into())
                    {
                        return false;
                    }
                    let Some(dismount_pos) = dismount_pos else {
                        *awaiting = None;
                        return false;
                    };
                    *awaiting = Some((id.into(), dismount_pos));
                    drop(awaiting);
                    passenger_entity.set_pos(dismount_pos);
                    // The Java vehicle exit keeps the current look and rotates the
                    // preserved delta with the relative rotation.
                    let mut relatives = vec![
                        PositionFlag::DeltaX,
                        PositionFlag::DeltaY,
                        PositionFlag::DeltaZ,
                        PositionFlag::YRot,
                        PositionFlag::XRot,
                    ];
                    if vehicle_exit {
                        relatives.push(PositionFlag::RotateDelta);
                    }
                    let packet = CPlayerPosition::new(
                        id.into(),
                        dismount_pos,
                        Vector3::new(0.0, 0.0, 0.0),
                        0.0,
                        0.0,
                        relatives,
                    );
                    if guard_placement {
                        let still_current =
                            || still_detached() && passenger_entity.pos.load() == dismount_pos;
                        let queued = match player.client.as_ref() {
                            ClientPlatform::Java(client) => client.try_send_riding_position(
                                &packet,
                                player,
                                rider_position,
                                still_current,
                            ),
                            ClientPlatform::Bedrock(client) => {
                                client.try_send_dismount_position(player, id.into(), still_current)
                            }
                            ClientPlatform::SimulatedPlayer(_) => false,
                        };
                        if !queued {
                            let mut awaiting = player
                                .awaiting_teleport
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            if awaiting
                                .as_ref()
                                .is_some_and(|(pending, _)| *pending == id.into())
                            {
                                *awaiting = None;
                            }
                        }
                    } else {
                        player.try_send_client_packet(&packet);
                        if let Some(client) = player.client.bedrock() {
                            player.send_bedrock_teleport(client);
                        }
                    }
                }

                // Vanilla: setSneaking(false) after dismount via sneak input
                if (!vehicle_exit || still_detached()) && passenger_entity.sneaking.load(Relaxed) {
                    passenger_entity.set_sneaking(false);
                }
            } else if let Some(dismount_pos) = dismount_pos {
                passenger_entity.set_pos(dismount_pos);
            }
            still_detached()
                && dismount_pos.is_some_and(|position| passenger_entity.pos.load() == position)
        } else {
            // No passenger was removed, still need to broadcast the passenger list
            self.send_passenger_update(&self.world.load(), None, &passenger_ids, &[]);
            false
        }
    }

    pub fn check_out_of_world(&self, dyn_self: &dyn EntityBase) {
        if self.pos.load().y < f64::from(self.world.load().dimension.min_y) - 64.0 {
            dyn_self.tick_in_void(dyn_self);
        }
    }

    pub fn reset_state(&self) {
        self.pose.store(EntityPose::Standing);
        self.refresh_dimensions();
        // Through the flag too, or both editions keep showing the glide after respawn
        if self.is_fall_flying() {
            self.set_fall_flying(false);
        }
        self.extinguish();
        self.set_on_fire(false);
    }

    pub fn slow_movement(&self, state: &BlockState, multiplier: Vector3<f64>) {
        match self.entity_type.id {
            v if v == EntityType::PLAYER.id => {
                if let Some(player_entity) = self.get_player()
                    && player_entity.is_flying()
                {
                    return;
                }
            }
            v if (v == EntityType::SPIDER.id || v == EntityType::CAVE_SPIDER.id)
                && Block::from_state_id(state.id).id == Block::COBWEB.id =>
            {
                return;
            }
            v if v == EntityType::WITHER.id => {
                return;
            }
            _ => {}
        }
        if let Some(living) = self.get_living_entity() {
            living.fall_distance.store(0f32);
        }
        self.movement_multiplier.store(multiplier);
    }

    pub fn set_custom_data(&self, namespace: &str, key: &str, value: NbtTag) {
        let mut custom_data = self
            .custom_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let mut namespace_data = custom_data
            .child_tags
            .remove(namespace)
            .and_then(|tag| match tag {
                NbtTag::Compound(compound) => Some(compound),
                _ => None,
            })
            .unwrap_or_default();

        namespace_data.child_tags.insert(key.into(), value);
        custom_data
            .child_tags
            .insert(namespace.into(), NbtTag::Compound(namespace_data));
    }

    pub fn get_custom_data(&self, namespace: &str, key: &str) -> Option<NbtTag> {
        let custom_data = self
            .custom_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        custom_data
            .get(namespace)?
            .extract_compound()?
            .get(key)
            .cloned()
    }

    pub fn remove_custom_data(&self, namespace: &str, key: &str) {
        let mut custom_data = self
            .custom_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let Some(NbtTag::Compound(mut namespace_data)) = custom_data.child_tags.remove(namespace)
        else {
            return;
        };

        namespace_data.child_tags.remove(key);
        if !namespace_data.is_empty() {
            custom_data
                .child_tags
                .insert(namespace.into(), NbtTag::Compound(namespace_data));
        }
    }

    pub fn has_custom_data(&self, namespace: &str, key: &str) -> bool {
        self.get_custom_data(namespace, key).is_some()
    }
}

impl Entity {
    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        let position = self.pos.load();
        nbt.put_string(
            "id",
            format!("minecraft:{}", self.entity_type.resource_name),
        );
        nbt.put_uuid("UUID", self.entity_uuid);
        let (x, z) = self
            .get_vehicle()
            .map_or((position.x, position.z), |vehicle| {
                let vehicle_pos = vehicle.get_entity().pos.load();
                (vehicle_pos.x, vehicle_pos.z)
            });
        nbt.put(
            "Pos",
            NbtTag::List(vec![x.into(), position.y.into(), z.into()]),
        );
        let velocity = self.velocity.load();
        nbt.put(
            "Motion",
            NbtTag::List(vec![
                velocity.x.into(),
                velocity.y.into(),
                velocity.z.into(),
            ]),
        );
        nbt.put(
            "Rotation",
            NbtTag::List(vec![self.yaw.load().into(), self.pitch.load().into()]),
        );
        if self.entity_type != &EntityType::PLAYER {
            nbt.put_short("Air", self.air_supply.load(Relaxed) as i16);
        }
        nbt.put_short("Fire", self.fire_ticks.load(Relaxed) as i16);
        nbt.put_bool("OnGround", self.on_ground.load(Relaxed));
        nbt.put_bool("Invulnerable", self.invulnerable.load(Relaxed));
        nbt.put_int("PortalCooldown", self.portal_cooldown.load(Relaxed) as i32);
        if let Some(custom_name) = &**self.custom_name.load() {
            let mut tag = custom_name
                .to_nbt_tag_for_version(&pumpkin_util::version::JavaMinecraftVersion::V_26_3);
            // A literal string starting with '{' would read back as legacy JSON, so keep it a compound.
            if let NbtTag::String(text) = &tag
                && text.starts_with('{')
            {
                let mut literal = NbtCompound::new();
                literal.put_string("text", text.to_string());
                tag = NbtTag::Compound(literal);
            }
            nbt.put("CustomName", tag);
        }
        if self.custom_name_visible.load(Relaxed) {
            nbt.put_bool("CustomNameVisible", true);
        }
        if self.is_silent() {
            nbt.put_bool("Silent", true);
        }
        if self.has_no_gravity() {
            nbt.put_bool("NoGravity", true);
        }
        if self.has_glowing_tag.load(Relaxed) {
            nbt.put_bool("Glowing", true);
        }
        let frozen_ticks = self.frozen_ticks.load(Relaxed);
        if frozen_ticks > 0 {
            nbt.put_int("TicksFrozen", frozen_ticks);
        }
        if self.has_visual_fire.load(Relaxed) {
            nbt.put_bool("HasVisualFire", true);
        }

        let tags = self
            .scoreboard_tags
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !tags.is_empty() {
            nbt.put(
                "Tags",
                NbtTag::List(
                    tags.iter()
                        .map(|tag| NbtTag::String(tag.as_str().into()))
                        .collect(),
                ),
            );
        }

        let custom_data = self
            .custom_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !custom_data.is_empty() {
            nbt.put_compound("data", custom_data.clone());
        }
    }

    fn read_placement_nbt(&self, nbt: &NbtCompound) {
        if let Some(position) = nbt.get_list("Pos")
            && position.len() >= 3
        {
            let x = position[0].extract_double().unwrap_or(0.0);
            let y = position[1].extract_double().unwrap_or(0.0);
            let z = position[2].extract_double().unwrap_or(0.0);
            let pos = Vector3::new(
                x.clamp(-MAX_HORIZONTAL_POSITION, MAX_HORIZONTAL_POSITION),
                y.clamp(-MAX_VERTICAL_POSITION, MAX_VERTICAL_POSITION),
                z.clamp(-MAX_HORIZONTAL_POSITION, MAX_HORIZONTAL_POSITION),
            );
            if pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite() {
                self.set_pos(pos);
                self.last_sent_pos.store(pos);
            }
        }
        if let Some(velocity) = nbt.get_list("Motion")
            && velocity.len() >= 3
        {
            let [x, y, z] = [0, 1, 2].map(|axis| {
                let motion = velocity[axis].extract_double().unwrap_or(0.0);
                if motion.abs() > MAX_LOADED_MOTION {
                    0.0
                } else {
                    motion
                }
            });
            self.velocity.store(Vector3::new(x, y, z));
        }
        if let Some(rotation) = nbt.get_list("Rotation")
            && rotation.len() >= 2
        {
            let yaw = rotation[0].extract_float().unwrap_or(0.0);
            let pitch = rotation[1].extract_float().unwrap_or(0.0);
            if yaw.is_finite() && pitch.is_finite() {
                self.set_rotation(yaw, pitch);
                // TODO: use `pumpkin_util::math::pack_degrees`.
                let yaw_byte = (yaw * 256.0 / 360.0).rem_euclid(256.0) as u8;
                let pitch_byte = (pitch * 256.0 / 360.0).rem_euclid(256.0) as u8;
                self.last_sent_yaw.store(yaw_byte, Relaxed);
                self.last_sent_pitch.store(pitch_byte, Relaxed);
                self.head_yaw.store(yaw);
                self.last_sent_head_yaw.store(yaw_byte, Relaxed);
            }
        }
    }

    pub fn read_nbt_non_mut(&self, nbt: &NbtCompound) {
        self.read_nbt_with_max_air_supply(nbt, breath::MAX_AIR);
    }

    fn read_nbt_with_max_air_supply(&self, nbt: &NbtCompound, max_air_supply: i32) {
        self.read_placement_nbt(nbt);
        self.fire_ticks.store(
            i32::from(nbt.get_numeric_short("Fire").unwrap_or(0)),
            Relaxed,
        );
        self.on_ground
            .store(nbt.get_bool("OnGround").unwrap_or(false), Relaxed);
        if self.entity_type != &EntityType::PLAYER {
            let air = nbt
                .get_numeric_short("Air")
                .map_or(max_air_supply, i32::from);
            self.air_supply.store(air, Relaxed);
            self.set_synced_data(tracked_data::entity::DATA_AIR_SUPPLY_ID, VarInt(air));
        }
        self.invulnerable
            .store(nbt.get_bool("Invulnerable").unwrap_or(false), Relaxed);
        self.portal_cooldown
            .store(nbt.get_int("PortalCooldown").unwrap_or(0) as u32, Relaxed);
        if let Some(name) = nbt.get("CustomName") {
            // Vanilla stores a text component tag; a string is literal text. Older
            // Pumpkin saves hold a JSON string (same data version, so it can't be told
            // apart by version); the writer never emits a literal starting with '{'.
            let component = match name {
                NbtTag::String(json) if json.starts_with('{') => {
                    pumpkin_util::serde_json::from_str(json)
                        .unwrap_or_else(|_| TextComponent::from_nbt(name))
                }
                _ => TextComponent::from_nbt(name),
            };
            // set_custom_name also updates the synced tracked data
            self.set_custom_name(component);
        } else {
            self.clear_custom_name();
        }
        let custom_name_visible = nbt.get_bool("CustomNameVisible").unwrap_or(false);
        if self.custom_name_visible.load(Relaxed) != custom_name_visible {
            self.set_custom_name_visible(custom_name_visible);
        }
        self.set_silent(nbt.get_bool("Silent").unwrap_or(false));
        self.set_has_no_gravity(nbt.get_bool("NoGravity").unwrap_or(false));
        self.set_glowing(nbt.get_bool("Glowing").unwrap_or(false));
        self.frozen_ticks
            .store(nbt.get_int("TicksFrozen").unwrap_or(0), Relaxed);
        self.has_visual_fire
            .store(nbt.get_bool("HasVisualFire").unwrap_or(false), Relaxed);

        let custom_data = nbt
            .get_compound("data")
            .or_else(|| nbt.get_compound("PumpkinCustomData"))
            .or_else(|| nbt.get_compound("BukkitValues"));
        *self
            .custom_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            custom_data.cloned().unwrap_or_default();

        {
            let mut tags = self
                .scoreboard_tags
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            tags.clear();
            if let Some(tag_list) = nbt.get_list("Tags") {
                tags.extend(
                    tag_list
                        .iter()
                        .filter_map(|tag| tag.extract_string().map(str::to_owned))
                        .take(MAX_SCOREBOARD_TAGS),
                );
            }
        }

        if let Some(team_name) = nbt.get_string("Team") {
            self.join_team(team_name);
        }
    }

    fn join_team(&self, team_name: &str) {
        if self.entity_type == &EntityType::PLAYER {
            return;
        }
        let world = self.world.load();
        let scoreboard_name = self.entity_uuid.to_string();
        let mut scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if scoreboard.get_team(team_name).is_none() {
            tracing::warn!(
                "Unable to add entity to team \"{team_name}\" (that team probably doesn't exist)"
            );
            return;
        }
        if let Some(previous_team) = scoreboard
            .get_entity_team(&scoreboard_name)
            .map(|team| team.name.clone())
        {
            if previous_team == team_name {
                return;
            }
            scoreboard.remove_player_from_team(&*world, &previous_team, &scoreboard_name);
        }
        scoreboard.add_player_to_team(&*world, team_name, scoreboard_name);
    }
}

impl Entity {
    pub(crate) fn base_tick(&self, caller: &dyn EntityBase) {
        // Recomputed during movement/block-collision handling in the same tick.
        let was_in_powder_snow = self.is_in_powder_snow.load(Ordering::Relaxed);
        self.was_in_powder_snow
            .store(was_in_powder_snow, Ordering::Relaxed);
        self.is_in_powder_snow.store(false, Ordering::Relaxed);

        if self.entity_type.boat.is_some() && caller.vehicle_contact_tick_reach().is_some() {
            // The boat's two block-effect passes apply fluid contact through the shared collector.
            // Advancing a portal also requires moving the vehicle and its riders between worlds.
            self.update_fluid_interaction(caller);
        } else {
            self.tick_portal(caller);
            self.update_fluid_state(caller);
        }
        self.check_out_of_world(caller);
        let fire_ticks = self.fire_ticks.load(Ordering::Relaxed);

        // Check for fire immunity (or if the specific entity is)
        let is_immune = self.is_fire_immune();
        if fire_ticks > 0 {
            if is_immune {
                self.fire_ticks.store(fire_ticks - 4, Ordering::Relaxed);
                if self.fire_ticks.load(Ordering::Relaxed) < 0 {
                    self.extinguish();
                }
            } else {
                // lava deals its own damage.
                if fire_ticks % 20 == 0 && !self.touching_lava.load(Ordering::SeqCst) {
                    caller.damage(caller, 1.0, DamageType::ON_FIRE);
                }

                self.fire_ticks.store(fire_ticks - 1, Ordering::Relaxed);
            }
        }

        // Check if visual fire should be sent
        let should_render_fire = self.fire_ticks.load(Ordering::Relaxed) > 0 && !is_immune;
        self.set_on_fire(should_render_fire);

        let riding_cooldown = self.riding_cooldown.load(Ordering::Relaxed);
        if riding_cooldown > 0 {
            self.riding_cooldown
                .store(riding_cooldown - 1, Ordering::Relaxed);
        }
        self.first_tick.store(false, Ordering::Relaxed);
    }
}

impl EntityBase for Entity {
    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.update_last_pos();
        self.base_tick(caller);
    }

    fn get_entity(&self) -> &Entity {
        self
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl<T: EntityBase + ?Sized> NBTStorage for T {
    fn write_nbt(&self, nbt: &mut NbtCompound) {
        EntityBase::write_nbt(self, nbt);
    }

    fn read_nbt_non_mut(&self, nbt: &NbtCompound) {
        EntityBase::read_nbt_non_mut(self, nbt);
    }
}

pub trait NBTStorage: Send + Sync {
    fn write_nbt(&self, _nbt: &mut NbtCompound) {}

    fn read_nbt(&mut self, nbt: &mut NbtCompound) {
        self.read_nbt_non_mut(nbt);
    }

    fn read_nbt_non_mut(&self, _nbt: &NbtCompound) {}
}

pub trait NBTStorageInit: Send + Sync + Sized {
    fn create_from_nbt(_nbt: &mut NbtCompound) -> Option<Self> {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Represents various entity flags that are sent in entity metadata.
///
/// These flags are used by the client to modify the rendering of entities based on their current state.
///
/// **Purpose:**
///
/// This enum provides a more type-safe and readable way to represent entity flags compared to using raw integer values.
pub enum Flag {
    /// Indicates if the entity is on fire.
    OnFire = 0,
    /// Indicates if the entity is sneaking.
    Sneaking = 1,
    /// Indicates if the entity is sprinting.
    Sprinting = 3,
    /// Indicates if the entity is swimming.
    Swimming = 4,
    /// Indicates if the entity is invisible.
    Invisible = 5,
    /// Indicates if the entity is glowing.
    Glowing = 6,
    /// Indicates if the entity is flying due to a fall.
    FallFlying = 7,
}

impl Flag {
    #[must_use]
    pub const fn to_bedrock(&self) -> Option<u32> {
        match self {
            Self::OnFire => Some(entity_data_flag::ON_FIRE),
            Self::Sneaking => Some(entity_data_flag::SNEAKING),
            Self::Sprinting => Some(entity_data_flag::SPRINTING),
            Self::Swimming => Some(entity_data_flag::SWIMMING),
            Self::Invisible => Some(entity_data_flag::INVISIBLE),
            Self::FallFlying => Some(entity_data_flag::GLIDING),
            Self::Glowing => None,
        }
    }
}

fn interact_with_shears(
    target: &(impl EntityBase + ?Sized),
    player: &Arc<Player>,
    stack: &mut ItemStack,
    hand: pumpkin_util::Hand,
) -> Option<bool> {
    if stack.item != &pumpkin_data::item::Item::SHEARS {
        return None;
    }
    if let Some(sheared) = target
        .get_entity()
        .shear_off_all_leash_connections(target, Some(player))
    {
        if sheared {
            let _ = player.damage_held_stack(stack, 1, hand);
        }
        return Some(sheared);
    }
    let mob = target.get_mob()?;
    if mob.can_shear_equipment(player) && !player.get_entity().is_sneaking() {
        mob.attempt_to_shear_equipment(player, stack, hand)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn committed_mount_pose_is_refreshed_after_disconnect()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::entity::ai::test_support::world;

        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let position = Vector3::new(8.5, 64.0, 8.5);
        let boat = Arc::new(Entity::new(world.clone(), position, &EntityType::OAK_BOAT));
        let rider = Arc::new(Entity::new(world.clone(), position, &EntityType::PLAYER));
        rider.set_pose(EntityPose::Crouching);
        let bounds = || {
            let bbox = rider.bounding_box.load();
            (bbox.min, bbox.max)
        };
        let crouching_box = bounds();

        // Model a disconnect after the mount commits its pose and riding links
        // but before it refreshes metadata and publishes the passenger update.
        boat.add_passenger_silently(boat.clone(), rider.clone());
        let revision = rider.riding_revision();
        rider.pose.store(EntityPose::Standing);
        boat.remove_passenger_on_disconnect(rider.entity_id);
        assert!(!rider.has_vehicle());
        assert_ne!(revision, rider.riding_revision());
        assert_eq!(bounds(), crouching_box);

        boat.finish_passenger_mount(
            boat.as_ref(),
            rider.as_ref(),
            &world,
            CommittedPassengerMount {
                revision,
                stood: true,
                displaced_driver: None,
                switch_placement_baseline: None,
            },
            PassengerMount::Gameplay,
        );
        let standing = rider.get_dimensions(EntityPose::Standing);
        let standing_box = Entity::make_bounding_box(&EntityType::PLAYER, position, &standing);
        assert_ne!(bounds(), crouching_box);
        assert_eq!(bounds(), (standing_box.min, standing_box.max));
        assert!(!rider.set_synced_data(
            tracked_data::entity::DATA_POSE,
            VarInt(EntityPose::Standing as i32),
        ));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn vehicle_projection_clips_against_pushable_entities_outside_its_mount_tree()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::entity::{
            ai::test_support::world, passive::pig::PigEntity, vehicle::minecart::MinecartEntity,
        };

        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let chunk = Vector2::new(0, 0);
        world
            .level
            .loaded_chunks
            .insert(chunk, pumpkin_world::chunk::ChunkData::empty_sync(0, 0));
        Arc::make_mut(
            &mut world
                .active_chunks
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
        .insert(chunk);
        let start = Vector3::new(6.5, 64.0, 8.5);
        let cart = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            start,
            &EntityType::HOPPER_MINECART,
        )));
        let pig = PigEntity::new(Entity::new(
            world.clone(),
            Vector3::new(8.5, 64.0, 8.5),
            &EntityType::PIG,
        ));
        world.add_entity_silent(cart.clone());
        world.add_entity_silent(pig.clone());
        assert!(pig.is_pushable());
        assert!(!pig.is_collidable(None));

        let carrier = cart.get_entity();
        let movement = Vector3::new(2.0, 0.0, 0.0);
        let contact_x =
            pig.get_entity().bounding_box.load().min.x - carrier.bounding_box.load().max.x;
        let projected = carrier.project_movement(movement, cart.as_ref());
        assert!((projected.movement.x - contact_x).abs() < 1.0E-7);
        assert!(projected.horizontal);
        assert_eq!(projected.supporting, None);
        assert_eq!(carrier.pos.load(), start);
        assert!(!carrier.horizontal_collision.load(Relaxed));

        cart.move_entity(cart.as_ref(), movement);
        assert!((carrier.pos.load().x - start.x - contact_x).abs() < 1.0E-7);
        assert!(carrier.horizontal_collision.load(Relaxed));

        carrier.set_pos(start);
        carrier.add_passenger(cart.clone(), pig);
        let projected = carrier.project_movement(movement, cart.as_ref());
        assert_eq!(projected.movement, movement);
        assert!(!projected.horizontal);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn concurrent_leash_shearing_drops_one_lead_per_connection()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::entity::{ai::test_support::world, passive::snow_golem::SnowGolemEntity};
        use std::sync::{Barrier, atomic::AtomicUsize};

        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let golem = SnowGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::SNOW_GOLEM,
        ));
        let holder: Arc<dyn EntityBase> = SnowGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(4.5, 64.0, 0.5),
            &EntityType::SNOW_GOLEM,
        ));
        let removals = AtomicUsize::new(0);
        for _ in 0..16 {
            golem.get_entity().attach_leash(&holder);
            let barrier = Barrier::new(8);
            std::thread::scope(|scope| {
                for _ in 0..8 {
                    scope.spawn(|| {
                        barrier.wait();
                        if golem.get_entity().drop_leash_from_shearing(None, None) {
                            removals.fetch_add(1, Ordering::Relaxed);
                        }
                    });
                }
            });
        }
        assert_eq!(removals.load(Ordering::Relaxed), 16);
        let leads: usize = world
            .entities
            .load()
            .iter()
            .filter_map(|entity| entity.cast_any().downcast_ref::<item::ItemEntity>())
            .map(|item| {
                let stack = item
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if stack.item == &pumpkin_data::item::Item::LEAD {
                    usize::from(stack.item_count)
                } else {
                    0
                }
            })
            .sum();
        assert_eq!(leads, 16);
        world.level.shutdown().await;
        Ok(())
    }

    #[test]
    fn equipment_break_status_maps_all_slots() {
        // Status bytes from vanilla EntityEvent: mainhand=47, offhand=48,
        // head=49, chest=50, legs=51, feet=52, body=65, saddle=68.
        let cases: &[(&EquipmentSlot, u8)] = &[
            (&EquipmentSlot::MAIN_HAND, EntityStatus::MainhandBreak as u8),
            (&EquipmentSlot::OFF_HAND, EntityStatus::OffhandBreak as u8),
            (&EquipmentSlot::HEAD, EntityStatus::HeadBreak as u8),
            (&EquipmentSlot::CHEST, EntityStatus::ChestBreak as u8),
            (&EquipmentSlot::LEGS, EntityStatus::LegsBreak as u8),
            (&EquipmentSlot::FEET, EntityStatus::FeetBreak as u8),
            (&EquipmentSlot::BODY, EntityStatus::BodyBreak as u8),
            (&EquipmentSlot::SADDLE, EntityStatus::SaddleBreak as u8),
        ];
        for (i, (slot, expected)) in cases.iter().enumerate() {
            assert_eq!(
                equipment_break_status(slot) as u8,
                *expected,
                "status mismatch at index {i}"
            );
        }
    }

    #[test]
    fn entity_type_bedrock_identifier_mappings() {
        assert_eq!(
            entity_type_to_bedrock_identifier(&EntityType::EXPERIENCE_ORB),
            "minecraft:xp_orb"
        );
        assert_eq!(
            entity_type_to_bedrock_identifier(&EntityType::FISHING_BOBBER),
            "minecraft:fishing_hook"
        );
        assert_eq!(
            entity_type_to_bedrock_identifier(&EntityType::EVOKER_FANGS),
            "minecraft:evocation_fang"
        );
        assert_eq!(
            entity_type_to_bedrock_identifier(&EntityType::TRIDENT),
            "minecraft:thrown_trident"
        );
        assert_eq!(
            entity_type_to_bedrock_identifier(&EntityType::FIREWORK_ROCKET),
            "minecraft:fireworks_rocket"
        );
        assert_eq!(
            entity_type_to_bedrock_identifier(&EntityType::EYE_OF_ENDER),
            "minecraft:eye_of_ender_signal"
        );
        assert_eq!(
            entity_type_to_bedrock_identifier(&EntityType::ZOMBIE_VILLAGER),
            "minecraft:zombie_villager_v2"
        );
        assert_eq!(entity_type_to_bedrock_identifier(&EntityType::COW), "cow");
    }

    #[tokio::test]
    async fn drop_leash_drops_when_the_holder_is_gone() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let pos = Vector3::new(0.0, 64.0, 0.0);
        let cow = Entity::new(world.clone(), pos, &EntityType::COW);
        let knot: Arc<dyn EntityBase> = Arc::new(LeashKnotEntity::new(Entity::new(
            world.clone(),
            pos,
            &EntityType::LEASH_KNOT,
        )));
        cow.attach_leash(&knot);
        drop(knot);
        assert!(cow.drop_leash());
        assert!(!cow.has_leash_data());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn unleash_held_by_keeps_a_leash_moved_to_another_holder()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let pos = Vector3::new(0.0, 64.0, 0.0);
        let knot = |world: &Arc<crate::world::World>| -> Arc<dyn EntityBase> {
            Arc::new(LeashKnotEntity::new(Entity::new(
                world.clone(),
                pos,
                &EntityType::LEASH_KNOT,
            )))
        };
        let old_knot = knot(&world);
        let new_holder = knot(&world);
        let cow = Entity::new(world.clone(), pos, &EntityType::COW);
        cow.attach_leash(&old_knot);
        cow.attach_leash(&new_holder);
        assert!(!cow.unleash_held_by(Some(old_knot.get_entity().entity_id)));
        assert_eq!(
            cow.get_leash_holder()
                .map(|holder| holder.get_entity().entity_id),
            Some(new_holder.get_entity().entity_id)
        );
        world.level.shutdown().await;
        Ok(())
    }
}

#[cfg(test)]
mod dimension_tests {
    use pumpkin_data::entity::{EntityPose, EntityType};
    use pumpkin_util::math::vector3::Vector3;

    use super::{Entity, EntityBase};
    use crate::entity::ai::test_support::world;
    use crate::entity::decoration::armor_stand::ArmorStandEntity;
    use crate::entity::interaction::InteractionEntity;
    use crate::entity::mob::warden::WardenEntity;
    use crate::entity::passive::sniffer::{SnifferEntity, SnifferState};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[tokio::test]
    async fn special_sizes_match_vanilla() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let at =
            |entity_type| Entity::new(world.clone(), Vector3::new(0.0, 64.0, 0.0), entity_type);

        // Interaction keeps its own size when moved
        let interaction = InteractionEntity::new(at(&EntityType::INTERACTION));
        interaction.set_width(2.0);
        interaction.set_height(3.0);
        interaction.entity.set_pos(Vector3::new(10.0, 70.0, 0.0));
        let aabb = interaction.entity.bounding_box.load();
        assert!(close(aabb.min.x, 9.0) && close(aabb.max.y, 73.0));

        // Small stand 0.25 x 0.9875, marker 0 x 0
        let stand = ArmorStandEntity::new(at(&EntityType::ARMOR_STAND));
        stand.set_small(true);
        let size = stand.get_entity().entity_dimension.load();
        assert!(close(f64::from(size.width), 0.25) && close(f64::from(size.height), 0.9875));
        stand.set_marker(true);
        let aabb = stand.get_entity().bounding_box.load();
        assert!(close(aabb.max.y, aabb.min.y) && close(aabb.max.x, aabb.min.x));

        // Warden 1 high while emerging, 2.9 again after
        let warden = WardenEntity::new(at(&EntityType::WARDEN));
        let entity = &warden.mob_entity.living_entity.entity;
        entity.set_pose(EntityPose::Emerging);
        assert!(close(f64::from(entity.entity_dimension.load().height), 1.0));
        entity.set_pose(EntityPose::Standing);
        assert!(close(f64::from(entity.entity_dimension.load().height), 2.9));

        // Sniffer 1.35 high while digging
        let sniffer = SnifferEntity::new(at(&EntityType::SNIFFER));
        sniffer.set_state(SnifferState::Digging);
        let entity = &sniffer.mob_entity.living_entity.entity;
        assert!(close(
            f64::from(entity.entity_dimension.load().height),
            1.35
        ));
        sniffer.set_state(SnifferState::Idling);
        assert!(close(
            f64::from(entity.entity_dimension.load().height),
            1.75
        ));

        // Wind charge box starts 0.15 below its position
        let charge = at(&EntityType::WIND_CHARGE);
        assert!(close(charge.bounding_box.load().min.y, 63.85));

        world.level.shutdown().await;
        Ok(())
    }
}

#[cfg(test)]
mod velocity_sync_tests {
    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;

    use super::Entity;
    use crate::entity::ai::test_support::world;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn player_motion_cutoff_and_send_filter_match_vanilla() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let player = Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::PLAYER,
        );

        // Horizontal speed squared under 9e-6 stops, 0.002 + 0.002 stays
        player.velocity.store(Vector3::new(0.002, 0.0, 0.002));
        player.check_zero_velo();
        assert_eq!(player.velocity.load(), Vector3::default());
        player.velocity.store(Vector3::new(0.0025, 0.0, 0.0025));
        player.check_zero_velo();
        assert_eq!(player.velocity.load().x, 0.0025);

        // Changes up to 1e-7 squared are not sent, stopping always is
        player.last_sent_velocity.store(Vector3::new(0.1, 0.0, 0.0));
        player.velocity.store(Vector3::new(0.1003, 0.0, 0.0));
        assert!(!player.velocity_changed_since_sent());
        player.velocity.store(Vector3::new(0.1004, 0.0, 0.0));
        assert!(player.velocity_changed_since_sent());
        player
            .last_sent_velocity
            .store(Vector3::new(0.0001, 0.0, 0.0));
        player.velocity.store(Vector3::default());
        assert!(player.velocity_changed_since_sent());

        world.level.shutdown().await;
        Ok(())
    }
}
