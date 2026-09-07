use crate::{
    entity::item::ItemEntity,
    net::{ClientPlatform, bedrock::BedrockClient, java::JavaClient},
    server::Server,
    world::{
        World,
        chunker::is_within_view_distance,
        portal::{NetherPortal, PortalProcessor, PortalType, SourcePortalInfo},
    },
};
use arc_swap::ArcSwap;
use bytes::BufMut;
use crossbeam::atomic::AtomicCell;
use living::LivingEntity;
use player::Player;
use pumpkin_data::BlockState;
use pumpkin_data::biome::Biome;
use pumpkin_data::block_properties::blocks_movement;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_data::{
    block_properties::{Facing, HorizontalFacing},
    damage::DamageType,
    entity::{EntityPose, EntityType},
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::bedrock::client::{CAddActor, CSetActorMotion};
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::java::client::play::{CUpdateEntityPos, CUpdateEntityPosRot};
use pumpkin_protocol::{
    PositionFlag,
    bedrock::client::{
        move_actor_delta::{
            CMoveActorDelta, MOVE_ACTOR_DELTA_FLAG_HAS_HEAD_YAW, MOVE_ACTOR_DELTA_FLAG_HAS_PITCH,
            MOVE_ACTOR_DELTA_FLAG_HAS_X, MOVE_ACTOR_DELTA_FLAG_HAS_Y,
            MOVE_ACTOR_DELTA_FLAG_HAS_YAW, MOVE_ACTOR_DELTA_FLAG_HAS_Z,
            MOVE_ACTOR_DELTA_FLAG_ON_GROUND,
        },
        move_player::CMovePlayer,
        set_actor_data::{
            CSetActorData, MetadataValue, PropertySyncData, SyncedActorDataList, entity_data_flag,
            entity_data_key,
        },
    },
    codec::var_int::VarInt,
    codec::var_ulong::VarULong,
    java::client::play::{
        CEntityPositionSync, CEntityVelocity, CHeadRot, CPlayerPosition, CSetEntityMetadata,
        CSetPassengers, CSpawnEntity, CSpawnLivingEntity, CUpdateEntityRot, Metadata,
        MetadataSerializer,
    },
};
use pumpkin_util::math::vector3::Axis;
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    get_section_cord,
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
    Arc,
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
pub mod decoration;
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
pub mod passive;
pub mod player;
pub mod projectile;
pub mod projectile_deflection;
pub mod synched_entity_data;
pub mod tnt;
pub mod r#type;
pub mod vehicle;

pub use lightning::LightningBoltEntity;

pub(crate) mod combat;
pub mod predicate;

/// The maximum number of scoreboard tags an entity can carry, matching Vanilla.
pub const MAX_SCOREBOARD_TAGS: usize = 1024;

/// Returns the [`EntityStatus`] that should be broadcast when the given
/// equipment slot breaks.
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

pub trait EntityBase: Send + Sync + std::any::Any {
    fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.get_entity().write_nbt(nbt);
        if let Some(living) = self.get_living_entity() {
            living.write_living_nbt(nbt);
        }
        self.write_custom_nbt(nbt);
    }

    fn write_custom_nbt(&self, _nbt: &mut NbtCompound) {}

    fn read_nbt_non_mut(&self, nbt: &NbtCompound) {
        self.get_entity().read_nbt_non_mut(nbt);
        if let Some(living) = self.get_living_entity() {
            living.read_living_nbt_non_mut(nbt);
        }
        self.read_custom_nbt(nbt);
    }

    fn read_custom_nbt(&self, _nbt: &NbtCompound) {}
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

    fn get_job_site_pos(&self) -> Option<pumpkin_util::math::position::BlockPos> {
        None
    }

    fn get_home_pos(&self) -> Option<pumpkin_util::math::position::BlockPos> {
        None
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

    fn get_owner_id(&self) -> Option<i32> {
        None
    }

    fn get_eye_pos(&self) -> Vector3<f64> {
        self.get_entity().get_eye_pos()
    }

    fn get_looking_vector(&self) -> Vector3<f64> {
        let entity = self.get_entity();
        Vector3::from_yaw_pitch(entity.yaw.load(), entity.pitch.load())
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

    fn get_mob(&self) -> Option<&dyn mob::Mob> {
        None
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
        if self.get_living_entity().is_some() {
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
    }

    fn is_spectator(&self) -> bool {
        false
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        false
    }

    fn can_hit(&self) -> bool {
        false
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

    fn send_bedrock_spawn_packet(&self, client: &BedrockClient) {
        if let Some(player) = self.get_player() {
            // Tracking can spawn a player again after they leave and re-enter view.
            // Bedrock needs their skin registration followed by AddPlayer each time.
            let (player_list, add_player) = World::bedrock_player_spawn_packets(player);
            if let (Ok(player_list), Ok(add_player)) = (
                client.serialize_packet(&player_list),
                client.serialize_packet(&add_player),
            ) {
                client.try_enqueue_packet(player_list);
                client.try_enqueue_packet(add_player);
                let held_item = player.inventory.held_item();
                client.try_enqueue_client_packet(
                    &pumpkin_protocol::bedrock::client::CMobEquipment {
                        target_runtime_id: (player.entity_id() as u64).into(),
                        item: (&held_item).into(),
                        slot: 0,
                        selected_slot: 0,
                        container_id: 0,
                    },
                );
            }
            return;
        }

        let entity = self.get_entity();
        let runtime_id = entity.entity_id as u64;
        let identifier = self
            .get_mob()
            .and_then(mob::Mob::mob_bedrock_identifier)
            .unwrap_or(entity.entity_type.resource_name);
        let mut metadata = entity.bedrock_metadata();
        if let Some(mob) = self.get_mob()
            && let Some(mob_metadata) = mob.mob_bedrock_spawn_metadata()
        {
            metadata.0.extend(mob_metadata.0);
        }
        let packet = CAddActor {
            target_actor_id: VarLong(runtime_id as i64),
            target_runtime_id: VarULong(runtime_id),
            actor_type: identifier.to_string(),
            position: entity.pos.load().to_f32_lossy(),
            velocity: entity.velocity.load().to_f32_lossy(),
            rotation: Vector2::new(entity.pitch.load(), entity.yaw.load()),
            y_head_rotation: entity.head_yaw.load(),
            y_body_rotation: entity.body_yaw.load(),
            attributes_list: Vec::new(),
            actor_data: metadata,
            synced_properties: PropertySyncData {
                int_entries_list: std::collections::HashMap::new(),
                float_entries_list: std::collections::HashMap::new(),
            },
            actor_links: Vec::new(),
        };
        if let Ok(data) = client.serialize_packet(&packet) {
            client.try_enqueue_packet(data);
        }
    }

    fn send_java_spawn_packet(&self, client: &JavaClient) {
        let entity = self.get_entity();
        if let Some(player) = self.get_player()
            && matches!(player.client.as_ref(), ClientPlatform::Bedrock(_))
        {
            World::send_java_player_spawn(
                client,
                player,
                entity.pos.load(),
                entity.pitch.load(),
                entity.yaw.load(),
                entity.head_yaw.load(),
                entity.velocity.load(),
            );
            return;
        }
        let version = client.version.load();
        let is_mob = entity.entity_type.mob || self.get_mob().is_some();
        let metadata = self.java_spawn_metadata(version);
        if version < JavaMinecraftVersion::V_1_19 && is_mob {
            let spawn_packet = entity.create_spawn_living_packet(metadata.clone());
            if let Ok(data) = client.serialize_packet(&spawn_packet) {
                client.try_enqueue_packet(data);
            }
            if version >= JavaMinecraftVersion::V_1_15
                && let Some(meta) = metadata
            {
                let meta_packet = CSetEntityMetadata::new(entity.entity_id.into(), meta);
                if let Ok(meta_data) = client.serialize_packet(&meta_packet) {
                    client.try_enqueue_packet(meta_data);
                }
            }
        } else {
            let spawn_packet = entity.create_spawn_packet();
            if let Ok(data) = client.serialize_packet(&spawn_packet) {
                client.try_enqueue_packet(data);
            }
            if let Some(meta) = metadata
                && (version >= JavaMinecraftVersion::V_1_9 || meta.last().copied() == Some(127))
            {
                let meta_packet = CSetEntityMetadata::new(entity.entity_id.into(), meta);
                if let Ok(meta_data) = client.serialize_packet(&meta_packet) {
                    client.try_enqueue_packet(meta_data);
                }
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

    fn set_on_fire_for(&self, seconds: f32) {
        let entity = self.get_entity();
        // Exclude fire-immune entities (ex. certain items) from burn damage
        if !entity.fire_immune.load(Ordering::Relaxed) {
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
            server.plugin_manager.fire_blocking(&server, &mut event);
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

    fn is_passenger(&self) -> bool {
        self.get_entity().has_vehicle()
    }

    fn is_vehicle(&self) -> bool {
        self.get_entity().has_passengers()
    }

    fn has_passenger(&self, other: &dyn EntityBase) -> bool {
        self.get_entity()
            .has_passenger(other.get_entity().entity_id)
    }

    fn move_entity(&self, caller: &dyn EntityBase, motion: Vector3<f64>) {
        self.get_entity().move_entity(caller, motion);
    }

    fn is_pushable(&self) -> bool {
        false
    }

    fn push(&self, entity: &dyn EntityBase) {
        let self_entity = self.get_entity();
        let other_entity = entity.get_entity();

        if self_entity.no_physics.load(Ordering::Relaxed)
            || other_entity.no_physics.load(Ordering::Relaxed)
        {
            return;
        }

        if self_entity.has_passenger(other_entity.entity_id)
            || other_entity.has_passenger(self_entity.entity_id)
        {
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
            dx *= 0.05;
            dz *= 0.05;

            if !self_entity.has_passengers() && self.is_pushable() {
                let mut vel = self_entity.velocity.load();
                vel.x -= dx;
                vel.z -= dz;
                self_entity.velocity.store(vel);
                self_entity.send_velocity();
            }

            if !other_entity.has_passengers() && entity.is_pushable() {
                let mut vel = other_entity.velocity.load();
                vel.x += dx;
                vel.z += dz;
                other_entity.velocity.store(vel);
                other_entity.send_velocity();
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn push_entities(&self, dyn_self: &dyn EntityBase) -> bool {
        let mut picked_up = false;
        let mut pushed = false;
        let self_entity = self.get_entity();
        let entity_bb = self_entity.bounding_box.load();

        if !self.is_pushable() {
            return false;
        }

        let world = self_entity.world.load();

        let is_rideable_minecart = self_entity.entity_type.id == EntityType::MINECART.id;
        let is_abstract_minecart = is_rideable_minecart
            || self_entity.entity_type.id == EntityType::CHEST_MINECART.id
            || self_entity.entity_type.id == EntityType::COMMAND_BLOCK_MINECART.id
            || self_entity.entity_type.id == EntityType::FURNACE_MINECART.id
            || self_entity.entity_type.id == EntityType::HOPPER_MINECART.id
            || self_entity.entity_type.id == EntityType::SPAWNER_MINECART.id
            || self_entity.entity_type.id == EntityType::TNT_MINECART.id;

        let is_minecart_fn = |id| -> bool {
            id == EntityType::MINECART.id
                || id == EntityType::CHEST_MINECART.id
                || id == EntityType::COMMAND_BLOCK_MINECART.id
                || id == EntityType::FURNACE_MINECART.id
                || id == EntityType::HOPPER_MINECART.id
                || id == EntityType::SPAWNER_MINECART.id
                || id == EntityType::TNT_MINECART.id
        };

        if is_abstract_minecart {
            let is_vehicle = self.is_vehicle();

            if is_rideable_minecart && !is_vehicle {
                let pickup_bb = entity_bb.expand(0.2, 0.0, 0.2);
                let other_entities = world.get_entities_at_box(&pickup_bb);

                for other in other_entities {
                    if other.get_entity().entity_id != self_entity.entity_id {
                        let other_type = other.get_entity().entity_type.id;
                        let is_iron_golem = other_type == EntityType::IRON_GOLEM.id;
                        let is_other_minecart = is_minecart_fn(other_type);

                        if !is_iron_golem
                            && !is_other_minecart
                            && !other.is_passenger()
                            && other.is_pushable()
                            && other.get_entity().riding_cooldown.load(Relaxed) == 0
                            && let Some(self_arc) = world.get_entity_by_id(self_entity.entity_id)
                        {
                            self_entity.add_passenger(self_arc, other.clone());
                            picked_up = true;
                            break;
                        }
                    }
                }
            }

            let push_bb = entity_bb.expand(1.0e-7, 1.0e-7, 1.0e-7);

            let other_entities = world.get_entities_at_box(&push_bb);
            for other in other_entities {
                if other.get_entity().entity_id != self_entity.entity_id {
                    let other_type = other.get_entity().entity_type.id;
                    let is_other_minecart = is_minecart_fn(other_type);
                    let is_iron_golem = other_type == EntityType::IRON_GOLEM.id;

                    if is_rideable_minecart {
                        if (is_iron_golem
                            || is_other_minecart
                            || is_vehicle
                            || !other.get_entity().has_vehicle())
                            && other.is_pushable()
                        {
                            dyn_self.push(other.as_ref());
                            pushed = true;
                        }
                    } else if !self.has_passenger(other.as_ref())
                        && other.is_pushable()
                        && is_other_minecart
                    {
                        dyn_self.push(other.as_ref());
                        pushed = true;
                    }
                }
            }

            let players = world.get_players_at_box(&push_bb);
            for player in players {
                if player.get_entity().entity_id != self_entity.entity_id && is_rideable_minecart {
                    dyn_self.push(player.as_ref());
                    pushed = true;
                    // Non-rideable minecarts (hoppers, chests) do not push players in vanilla.
                }
            }
        } else {
            let other_entities = world.get_entities_at_box(&entity_bb);
            for other in other_entities {
                if other.get_entity().entity_id != self_entity.entity_id {
                    dyn_self.push(other.as_ref());
                    pushed = true;
                }
            }

            let players = world.get_players_at_box(&entity_bb);
            for player in players {
                if player.get_entity().entity_id != self_entity.entity_id {
                    dyn_self.push(player.as_ref());
                    pushed = true;
                }
            }
        }

        picked_up && !pushed
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

// IMPORTANT: have that 1 and not 0 because fetch_add returns previous value and 0 would be invalid
static CURRENT_ID: AtomicI32 = AtomicI32::new(1);

/// Represents a non-living Entity (e.g. Item, Egg, Snowball...)
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
    /// Indicates whether the entity is flying due to a fall
    pub fall_flying: AtomicBool,
    /// The entity's current velocity vector, aka knockback
    pub velocity: AtomicCell<Vector3<f64>>,
    /// Tracks a horizontal collision
    pub horizontal_collision: AtomicBool,
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
    pub bounding_box: AtomicCell<BoundingBox>,
    ///The size (width and height) of the bounding box
    pub entity_dimension: AtomicCell<EntityDimensions>,
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
    /// The vehicle that entity is in
    pub vehicle: std::sync::Mutex<Option<Arc<dyn EntityBase>>>,
    /// The entity this entity is attached/leashed to (if any)
    pub leashed_to: std::sync::Mutex<Option<Arc<dyn EntityBase>>>,
    /// Cooldown before entity can mount again after dismounting
    pub riding_cooldown: AtomicI32,
    /// The age of the entity in ticks. Negative values indicate a baby.
    pub age: AtomicI32,

    pub current_biome: ArcSwap<&'static Biome>,
    pub last_biome_update_pos: AtomicCell<BlockPos>,

    pub portal_cooldown: AtomicU32,

    pub portal_manager: std::sync::Mutex<Option<PortalProcessor>>,
    /// Custom name for the entity
    pub custom_name: ArcSwap<Option<TextComponent>>,
    /// Indicates whether the entity's custom name is visible
    pub custom_name_visible: AtomicBool,
    pub silent: AtomicBool,
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
    /// If true, the entity bypasses physics, collisions, and block effects (e.g. spectator, markers, display entities)
    pub no_physics: AtomicBool,
    pub synched_data: synched_entity_data::SynchedEntityData,
    /// Multiplies movement for one tick before being reset
    pub movement_multiplier: AtomicCell<Vector3<f64>>,
    /// Determines whether the entity's velocity needs to be sent
    pub velocity_dirty: AtomicBool,
    /// Set when an Entity is to be removed but could still be referenced
    pub removed: AtomicBool,
    /// The last sent yaw value (encoded as u8) for change detection
    pub last_sent_yaw: AtomicU8,
    /// The last sent pitch value (encoded as u8) for change detection
    pub last_sent_pitch: AtomicU8,
    /// Cache for the last sent position to optimize Entity Pos update packets
    pub last_sent_pos: AtomicCell<Vector3<f64>>,
    /// Cache for the last sent head yaw byte
    pub last_sent_head_yaw: AtomicU8,
    /// Persistent custom data container for plugins (matching Bukkit's `PersistentDataHolder`)
    pub custom_data: std::sync::Mutex<NbtCompound>,
}

impl Entity {
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

        let bounding_box_size = EntityDimensions {
            width: entity_type.dimension[0],
            height: entity_type.dimension[1],
            eye_height: entity_type.eye_height,
        };

        let current_biome = world
            .level
            .get_rough_biome(&BlockPos::new(floor_x, floor_y, floor_z));

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
            world: ArcSwap::new(world),
            sprinting: AtomicBool::new(false),
            fall_flying: AtomicBool::new(false),
            yaw: AtomicCell::new(0.0),
            head_yaw: AtomicCell::new(0.0),
            body_yaw: AtomicCell::new(0.0),
            pitch: AtomicCell::new(0.0),
            velocity: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            pose: AtomicCell::new(EntityPose::Standing),
            bounding_box: AtomicCell::new(BoundingBox::new_from_pos(
                position.x,
                position.y,
                position.z,
                &bounding_box_size,
            )),
            entity_dimension: AtomicCell::new(bounding_box_size),
            invulnerable: AtomicBool::new(false),
            damage_immunities: std::sync::Mutex::new(Vec::new()),
            data: AtomicI32::new(0),
            flags: std::sync::atomic::AtomicI8::new(0),
            bedrock_flags: std::sync::atomic::AtomicI64::new(0),
            bedrock_flags_two: std::sync::atomic::AtomicI64::new(0),
            fire_immune: AtomicBool::new(false),
            fire_ticks: AtomicI32::new(-1),
            has_visual_fire: AtomicBool::new(false),
            frozen_ticks: AtomicI32::new(0),
            is_in_powder_snow: AtomicBool::new(false),
            was_in_powder_snow: AtomicBool::new(false),
            removal_reason: AtomicCell::new(None),
            passengers: std::sync::Mutex::new(Vec::new()),
            vehicle: std::sync::Mutex::new(None),
            leashed_to: std::sync::Mutex::new(None),

            riding_cooldown: AtomicI32::new(0),
            age: AtomicI32::new(0),
            current_biome: ArcSwap::new(Arc::new(current_biome)),
            last_biome_update_pos: AtomicCell::new(BlockPos::new(floor_x, floor_y, floor_z)),
            portal_cooldown: AtomicU32::new(0),
            portal_manager: std::sync::Mutex::new(None),
            custom_name: ArcSwap::new(Arc::new(None)),
            custom_name_visible: AtomicBool::new(false),
            silent: AtomicBool::new(false),
            has_no_gravity: AtomicBool::new(false),
            scoreboard_tags: std::sync::Mutex::new(HashSet::new()),
            no_physics: AtomicBool::new(false),
            synched_data: synched_entity_data::SynchedEntityData::new(),
            movement_multiplier: AtomicCell::new(Vector3::default()),
            velocity_dirty: AtomicBool::new(true),
            removed: AtomicBool::new(false),
            last_sent_yaw: AtomicU8::new(0),
            last_sent_pitch: AtomicU8::new(0),
            last_sent_head_yaw: AtomicU8::new(0),
            last_sent_pos: AtomicCell::new(position),
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
        self.world.store(world);
    }

    pub fn bedrock_metadata(&self) -> SyncedActorDataList {
        if self.bedrock_flags.load(Ordering::Relaxed) == 0 {
            self.bedrock_flags.fetch_or(
                (1i64 << entity_data_flag::HAS_GRAVITY)
                    | (1i64 << entity_data_flag::CLIMB)
                    | (1i64 << entity_data_flag::HAS_COLLISION)
                    | (1i64 << entity_data_flag::BREATHING),
                Ordering::Relaxed,
            );
        }

        let mut metadata = SyncedActorDataList::new();
        metadata.set(
            entity_data_key::WIDTH,
            MetadataValue::Float(self.entity_type.dimension[0]),
        );
        metadata.set(
            entity_data_key::HEIGHT,
            MetadataValue::Float(self.entity_type.dimension[1]),
        );
        metadata.set(entity_data_key::SCALE, MetadataValue::Float(1.0));
        metadata.set(
            entity_data_key::FLAGS,
            MetadataValue::Int64(self.bedrock_flags.load(Ordering::Relaxed)),
        );
        metadata.set(
            entity_data_key::FLAGS_TWO,
            MetadataValue::Int64(self.bedrock_flags_two.load(Ordering::Relaxed)),
        );

        if let Some(name) = &**self.custom_name.load() {
            metadata.set(
                entity_data_key::NAME,
                MetadataValue::String(name.clone().get_text()),
            );
            if self.custom_name_visible.load(Ordering::Relaxed) {
                metadata.set_flag(
                    entity_data_key::FLAGS,
                    entity_data_flag::SHOW_NAME as u8,
                    true,
                );
                metadata.set_flag(
                    entity_data_key::FLAGS,
                    entity_data_flag::ALWAYS_SHOW_NAME as u8,
                    true,
                );
            }
        }

        metadata
    }

    /// Sets the entity's age in ticks.
    /// Negative values indicate that the entity is a baby.
    pub fn set_age(&self, age: i32) {
        self.age.store(age, Relaxed);
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

    pub fn send_velocity(&self) {
        let velocity = self.velocity.load();
        let chunk_pos = self.chunk_pos.load();
        self.world.load().broadcast_to_chunk_editioned(
            chunk_pos,
            &CEntityVelocity::new(self.entity_id.into(), velocity),
            &CSetActorMotion {
                target_runtime_id: VarULong(self.entity_id as u64),
                motion: Vector3::new(velocity.x as f32, velocity.y as f32, velocity.z as f32),
                tick: VarULong(0),
            },
        );
    }

    #[must_use]
    pub const fn get_entity_dimensions(pose: EntityPose) -> EntityDimensions {
        match pose {
            EntityPose::Sleeping => EntityDimensions::new(0.2, 0.2, 0.2),
            EntityPose::FallFlying | EntityPose::Swimming | EntityPose::SpinAttack => {
                EntityDimensions::new(0.6, 0.6, 0.4)
            }
            EntityPose::Crouching => EntityDimensions::new(0.6, 1.5, 1.27),
            EntityPose::Dying => EntityDimensions::new(0.2, 0.2, 1.62),
            _ => EntityDimensions::new(0.6, 1.8, 1.62),
        }
    }

    pub fn get_eye_height(&self) -> f64 {
        f64::from(Self::get_entity_dimensions(self.pose.load()).eye_height)
    }

    /// Updates the entity's position, block position, and chunk position.
    ///
    /// This function calculates the new position, block position, and chunk position based on the provided coordinates. If any of these values change, the corresponding fields are updated.
    pub fn set_pos(&self, new_position: Vector3<f64>) {
        let pos = self.pos.load();
        if pos != new_position {
            self.pos.store(new_position);
            self.bounding_box.store(BoundingBox::new_from_pos(
                new_position.x,
                new_position.y,
                new_position.z,
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

    pub fn send_rotation(&self) {
        let yaw = self.yaw.load();
        let pitch = self.pitch.load();
        let chunk_pos = self.chunk_pos.load();

        // Broadcast the update packet.

        let yaw = (yaw * 256.0 / 360.0).rem_euclid(256.0) as u8;
        let pitch = (pitch * 256.0 / 360.0).rem_euclid(256.0) as u8;

        if yaw == self.last_sent_yaw.load(Relaxed) && pitch == self.last_sent_pitch.load(Relaxed) {
            return;
        }

        self.last_sent_yaw.store(yaw, Relaxed);
        self.last_sent_pitch.store(pitch, Relaxed);

        self.world.load().broadcast_to_chunk(
            chunk_pos,
            &CUpdateEntityRot::new(
                self.entity_id.into(),
                yaw,
                pitch,
                self.on_ground.load(Relaxed),
            ),
        );

        self.send_head_rot(yaw);
    }

    pub fn send_head_rot(&self, head_yaw: u8) {
        let chunk_pos = self.chunk_pos.load();
        if head_yaw == self.last_sent_head_yaw.load(Relaxed) {
            return;
        }
        self.last_sent_head_yaw.store(head_yaw, Relaxed);

        self.world
            .load()
            .broadcast_to_chunk(chunk_pos, &CHeadRot::new(self.entity_id.into(), head_yaw));
    }

    fn default_portal_cooldown(&self) -> u32 {
        if self.entity_type == &EntityType::PLAYER {
            10
        } else {
            300
        }
    }

    /// Returns the block position of the block the (non-player) entity is standing on, if any.
    pub fn get_supporting_block_pos(&self) -> Option<BlockPos> {
        // Check if the entity is on the ground
        if !self.on_ground.load(Ordering::Relaxed) {
            return None;
        }

        self.supporting_block_pos.load()
    }

    #[expect(clippy::float_cmp)]
    fn adjust_movement_for_collisions(
        &self,
        movement: Vector3<f64>,
        caller: &dyn EntityBase,
    ) -> Vector3<f64> {
        if movement.length_squared() == 0.0 {
            return movement;
        }

        self.on_ground.store(false, Ordering::SeqCst);
        self.supporting_block_pos.store(None);
        self.horizontal_collision.store(false, Ordering::SeqCst);

        let bounding_box = self.bounding_box.load();

        let (collisions, block_positions) = self
            .world
            .load()
            .get_block_collisions(bounding_box.stretch(movement), caller);

        if collisions.is_empty() {
            return movement;
        }

        let mut adjusted_movement = movement;

        // Y-Axis adjustment
        if movement.get_axis(Axis::Y) != 0.0 {
            let mut max_time = 1.0;
            let mut positions = block_positions.into_iter();
            if let Some((mut collisions_len, mut position)) = positions.next() {
                let mut supporting_block_pos = None;

                for (i, inert_box) in collisions.iter().enumerate() {
                    if i == collisions_len {
                        let Some((next_len, next_pos)) = positions.next() else {
                            break;
                        };
                        collisions_len = next_len;
                        position = next_pos;
                    }

                    if let Some(collision_time) = bounding_box.calculate_collision_time(
                        inert_box,
                        adjusted_movement,
                        Axis::Y,
                        max_time,
                    ) {
                        max_time = collision_time;

                        // If the entity is moving downwards and collides, set the supporting block position
                        if movement.get_axis(Axis::Y) < 0.0 {
                            supporting_block_pos = Some(position);
                        }
                    }
                }

                if max_time != 1.0 {
                    let changed_component = adjusted_movement.get_axis(Axis::Y) * max_time;
                    adjusted_movement.set_axis(Axis::Y, changed_component);
                }

                self.on_ground
                    .store(supporting_block_pos.is_some(), Ordering::SeqCst);
                self.supporting_block_pos.store(supporting_block_pos);
            }
        }

        let mut horizontal_collision = false;

        for axis in Axis::horizontal() {
            if movement.get_axis(axis) == 0.0 {
                continue;
            }

            let mut max_time = 1.0;

            for inert_box in &collisions {
                if let Some(collision_time) = bounding_box.calculate_collision_time(
                    inert_box,
                    adjusted_movement,
                    axis,
                    max_time,
                ) {
                    max_time = collision_time;
                }
            }

            if max_time != 1.0 {
                let changed_component = adjusted_movement.get_axis(axis) * max_time;
                adjusted_movement.set_axis(axis, changed_component);
                horizontal_collision = true;
            }
        }

        self.horizontal_collision
            .store(horizontal_collision, Ordering::SeqCst);

        adjusted_movement
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
    pub fn apply_knockback(&self, strength: f64, mut x: f64, mut z: f64) {
        if strength <= 0.0 {
            return;
        }

        self.velocity_dirty.store(true, Ordering::SeqCst);

        // This has some vanilla magic

        while x.mul_add(x, z * z) < 1.0E-5 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;

            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * strength;

        let velocity = self.velocity.load();

        self.velocity.store(Vector3::new(
            velocity.x / 2.0 - var8.x,
            if self.on_ground.load(Relaxed) {
                (velocity.y / 2.0 + strength).min(0.4)
            } else {
                velocity.y
            },
            velocity.z / 2.0 - var8.z,
        ));
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

            // TODO: this is default predicate, vanilla overwrites it for some blocks,
            // see .suffocates(...) in Blocks.java
            let check_suffocation =
                !suffocating && blocks_movement(state, block.id) && state.is_full_cube();

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

    #[expect(clippy::too_many_lines)]
    pub fn send_pos_rot(&self) {
        let old = self.last_sent_pos.load();
        let new = self.pos.load();
        let chunk_pos = self.chunk_pos.load();

        let converted = Vector3::new(
            new.x.mul_add(4096.0, -(old.x * 4096.0)) as i16,
            new.y.mul_add(4096.0, -(old.y * 4096.0)) as i16,
            new.z.mul_add(4096.0, -(old.z * 4096.0)) as i16,
        );

        let yaw = self.yaw.load();

        let pitch = self.pitch.load();
        let yaw = (yaw * 256.0 / 360.0).rem_euclid(256.0) as u8;
        let pitch = (pitch * 256.0 / 360.0).rem_euclid(256.0) as u8;

        // Only broadcast when position or rotation has actually changed.
        let pos_changed = converted.x != 0 || converted.y != 0 || converted.z != 0;
        let rot_changed =
            yaw != self.last_sent_yaw.load(Relaxed) || pitch != self.last_sent_pitch.load(Relaxed);

        if !pos_changed && !rot_changed {
            return;
        }

        self.last_sent_pos.store(new);
        self.last_sent_yaw.store(yaw, Relaxed);
        self.last_sent_pitch.store(pitch, Relaxed);

        // Dynamically pick the most efficient packet
        if pos_changed && rot_changed {
            let je_packet = CUpdateEntityPosRot::new(
                self.entity_id.into(),
                Vector3::new(converted.x, converted.y, converted.z),
                yaw,
                pitch,
                self.on_ground.load(Relaxed),
            );
            if self.entity_type == &EntityType::PLAYER {
                self.world.load().broadcast_to_chunk_editioned(
                    chunk_pos,
                    &je_packet,
                    &CMovePlayer::new(
                        VarULong(self.entity_id as u64),
                        Vector3::new(new.x as f32, new.y as f32, new.z as f32),
                        self.pitch.load(),
                        self.yaw.load(),
                        self.yaw.load(),
                        CMovePlayer::MODE_NORMAL,
                        self.on_ground.load(Relaxed),
                        VarULong(0),
                        0,
                        0,
                        VarULong(0),
                    ),
                );
            } else {
                let mut flags = MOVE_ACTOR_DELTA_FLAG_HAS_X
                    | MOVE_ACTOR_DELTA_FLAG_HAS_Y
                    | MOVE_ACTOR_DELTA_FLAG_HAS_Z
                    | MOVE_ACTOR_DELTA_FLAG_HAS_PITCH
                    | MOVE_ACTOR_DELTA_FLAG_HAS_YAW
                    | MOVE_ACTOR_DELTA_FLAG_HAS_HEAD_YAW;
                if self.on_ground.load(Relaxed) {
                    flags |= MOVE_ACTOR_DELTA_FLAG_ON_GROUND;
                }
                self.world.load().broadcast_to_chunk_editioned(
                    chunk_pos,
                    &je_packet,
                    &CMoveActorDelta::new(
                        VarULong(self.entity_id as u64),
                        flags,
                        new.x as f32,
                        new.y as f32,
                        new.z as f32,
                        pitch,
                        yaw,
                        yaw,
                    ),
                );
            }
        } else if pos_changed {
            let je_packet = CUpdateEntityPos::new(
                self.entity_id.into(),
                Vector3::new(converted.x, converted.y, converted.z),
                self.on_ground.load(Relaxed),
            );
            if self.entity_type == &EntityType::PLAYER {
                self.world.load().broadcast_to_chunk_editioned(
                    chunk_pos,
                    &je_packet,
                    &CMovePlayer::new(
                        VarULong(self.entity_id as u64),
                        Vector3::new(new.x as f32, new.y as f32, new.z as f32),
                        self.pitch.load(),
                        self.yaw.load(),
                        self.yaw.load(),
                        CMovePlayer::MODE_NORMAL,
                        self.on_ground.load(Relaxed),
                        VarULong(0),
                        0,
                        0,
                        VarULong(0),
                    ),
                );
            } else {
                let mut flags = MOVE_ACTOR_DELTA_FLAG_HAS_X
                    | MOVE_ACTOR_DELTA_FLAG_HAS_Y
                    | MOVE_ACTOR_DELTA_FLAG_HAS_Z;
                if self.on_ground.load(Relaxed) {
                    flags |= MOVE_ACTOR_DELTA_FLAG_ON_GROUND;
                }

                self.world.load().broadcast_to_chunk_editioned(
                    chunk_pos,
                    &je_packet,
                    &CMoveActorDelta::new(
                        VarULong(self.entity_id as u64),
                        flags,
                        new.x as f32,
                        new.y as f32,
                        new.z as f32,
                        0,
                        0,
                        0,
                    ),
                );
            }
        } else if rot_changed {
            let je_packet = CUpdateEntityRot::new(
                self.entity_id.into(),
                yaw,
                pitch,
                self.on_ground.load(Relaxed),
            );
            if self.entity_type == &EntityType::PLAYER {
                self.world.load().broadcast_to_chunk_editioned(
                    chunk_pos,
                    &je_packet,
                    &CMovePlayer::new(
                        VarULong(self.entity_id as u64),
                        Vector3::new(new.x as f32, new.y as f32, new.z as f32),
                        self.pitch.load(),
                        self.yaw.load(),
                        self.yaw.load(),
                        CMovePlayer::MODE_ROTATION,
                        self.on_ground.load(Relaxed),
                        VarULong(0),
                        0,
                        0,
                        VarULong(0),
                    ),
                );
            } else {
                let mut flags = MOVE_ACTOR_DELTA_FLAG_HAS_PITCH
                    | MOVE_ACTOR_DELTA_FLAG_HAS_YAW
                    | MOVE_ACTOR_DELTA_FLAG_HAS_HEAD_YAW;
                if self.on_ground.load(Relaxed) {
                    flags |= MOVE_ACTOR_DELTA_FLAG_ON_GROUND;
                }
                self.world.load().broadcast_to_chunk_editioned(
                    chunk_pos,
                    &je_packet,
                    &CMoveActorDelta::new(
                        VarULong(self.entity_id as u64),
                        flags,
                        new.x as f32,
                        new.y as f32,
                        new.z as f32,
                        pitch,
                        yaw,
                        yaw,
                    ),
                );
            }
        }
        self.send_head_rot(yaw);
    }

    pub fn send_bedrock_pos(&self) {
        let position = self.pos.load();
        let chunk_pos = self.chunk_pos.load();
        let mut flags =
            MOVE_ACTOR_DELTA_FLAG_HAS_X | MOVE_ACTOR_DELTA_FLAG_HAS_Y | MOVE_ACTOR_DELTA_FLAG_HAS_Z;
        if self.on_ground.load(Relaxed) {
            flags |= MOVE_ACTOR_DELTA_FLAG_ON_GROUND;
        }
        let packet = CMoveActorDelta::new(
            VarULong(self.entity_id as u64),
            flags,
            position.x as f32,
            position.y as f32,
            position.z as f32,
            0,
            0,
            0,
        );
        let world = self.world.load();
        world.broadcast_to_chunk_bedrock(chunk_pos, &packet);
    }

    pub fn update_last_pos(&self) -> Vector3<f64> {
        let pos = self.pos.load();
        let old = self.last_pos.load();
        self.movement.store(pos - old);
        self.last_pos.store(pos);
        old
    }

    pub fn send_pos(&self) {
        let old = self.last_sent_pos.load();
        let new = self.pos.load();
        let chunk_pos = self.chunk_pos.load();

        let converted = Vector3::new(
            new.x.mul_add(4096.0, -(old.x * 4096.0)) as i16,
            new.y.mul_add(4096.0, -(old.y * 4096.0)) as i16,
            new.z.mul_add(4096.0, -(old.z * 4096.0)) as i16,
        );

        // Only broadcast when position has actually changed.
        if converted.x == 0 && converted.y == 0 && converted.z == 0 {
            return;
        }

        self.last_sent_pos.store(new);

        let je_packet = CUpdateEntityPos::new(
            self.entity_id.into(),
            Vector3::new(converted.x, converted.y, converted.z),
            self.on_ground.load(Relaxed),
        );

        if self.entity_type == &EntityType::PLAYER {
            self.world.load().broadcast_to_chunk_editioned(
                chunk_pos,
                &je_packet,
                &CMovePlayer::new(
                    VarULong(self.entity_id as u64),
                    Vector3::new(new.x as f32, new.y as f32, new.z as f32),
                    self.pitch.load(),
                    self.yaw.load(),
                    self.yaw.load(),
                    CMovePlayer::MODE_NORMAL,
                    self.on_ground.load(Relaxed),
                    VarULong(0),
                    0,
                    0,
                    VarULong(0),
                ),
            );
        } else {
            let mut flags = MOVE_ACTOR_DELTA_FLAG_HAS_X
                | MOVE_ACTOR_DELTA_FLAG_HAS_Y
                | MOVE_ACTOR_DELTA_FLAG_HAS_Z;
            if self.on_ground.load(Relaxed) {
                flags |= MOVE_ACTOR_DELTA_FLAG_ON_GROUND;
            }

            self.world.load().broadcast_to_chunk_editioned(
                chunk_pos,
                &je_packet,
                &CMoveActorDelta::new(
                    VarULong(self.entity_id as u64),
                    flags,
                    new.x as f32,
                    new.y as f32,
                    new.z as f32,
                    0,
                    0,
                    0,
                ),
            );
        }
    }

    // updateWaterState() in yarn

    fn update_fluid_state(&self, caller: &dyn EntityBase) {
        let is_pushed = caller.is_pushed_by_fluids();
        let mut fluids = BTreeMap::new();

        let water_push = Vector3::default();

        let water_n = 0;

        let lava_push = Vector3::default();

        let lava_n = 0;

        let mut fluid_push = [water_push, lava_push];

        let mut fluid_n = [water_n, lava_n];

        let mut in_fluid = [false, false];

        // The maximum fluid height found

        let mut fluid_height: [f64; 2] = [0.0, 0.0];

        let bounding_box = self.bounding_box.load().expand(-0.001, -0.001, -0.001);

        let min = bounding_box.min_block_pos();

        let max = bounding_box.max_block_pos();

        let world = self.world.load();

        for x in min.0.x..=max.0.x {
            for y in min.0.y..=max.0.y {
                for z in min.0.z..=max.0.z {
                    let pos = BlockPos::new(x, y, z);

                    let (fluid, state) = world.get_fluid_and_fluid_state(&pos);

                    if fluid.id != Fluid::EMPTY.id {
                        let marginal_height =
                            f64::from(state.height) + f64::from(y) - bounding_box.min.y;

                        if marginal_height >= 0.0 {
                            let i = usize::from(
                                fluid.id == Fluid::FLOWING_LAVA.id || fluid.id == Fluid::LAVA.id,
                            );

                            fluid_height[i] = fluid_height[i].max(marginal_height);

                            in_fluid[i] = true;

                            if !is_pushed {
                                fluids.insert(fluid.id, fluid);

                                continue;
                            }

                            let mut fluid_velo = world.get_fluid_velocity(pos, fluid, state);

                            if fluid_height[i] < 0.4 {
                                fluid_velo = fluid_velo * fluid_height[i];
                            }

                            fluid_push[i] += fluid_velo;

                            fluid_n[i] += 1;

                            fluids.insert(fluid.id, fluid);
                        }
                    }
                }
            }
        }

        // BTreeMap auto-sorts water before lava as in vanilla

        for (_, fluid) in fluids {
            world
                .block_registry
                .on_entity_collision_fluid(fluid, caller);
        }

        let lava_speed = if world.dimension.fast_lava {
            0.007
        } else {
            0.002_333_333
        };

        self.push_by_fluid(0.014, fluid_push[0], fluid_n[0]);

        self.push_by_fluid(lava_speed, fluid_push[1], fluid_n[1]);

        let water_height = fluid_height[0];

        let in_water = in_fluid[0];

        if in_water {
            if let Some(living) = caller.get_living_entity() {
                living.fall_distance.store(0.0);
            }

            if !self.touching_water.load(Ordering::SeqCst) {

                // TODO: Spawn splash particles
            }
        }

        self.water_height.store(water_height);

        self.touching_water.store(in_water, Ordering::SeqCst);

        let lava_height = fluid_height[1];

        let in_lava = in_fluid[1];

        if in_lava && let Some(living) = caller.get_living_entity() {
            let halved_fall = living.fall_distance.load() / 2.0;

            if halved_fall != 0.0 {
                living.fall_distance.store(halved_fall);
            }
        }

        self.lava_height.store(lava_height);

        self.touching_lava.store(in_lava, Ordering::SeqCst);
    }

    fn push_by_fluid(&self, speed: f64, mut push: Vector3<f64>, n: usize) {
        if push.length_squared() != 0.0 {
            if n > 0 {
                push = push * (1.0 / (n as f64));
            }

            if self.entity_type != &EntityType::PLAYER {
                push = push.normalize();
            }

            push = push * speed;

            let velo = self.velocity.load();

            if velo.x.abs() < 0.003 && velo.z.abs() < 0.003 && velo.length_squared() < 0.000_020_25
            {
                push = push.normalize() * 0.0045;
            }

            self.velocity.store(velo + push);
        }
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

    // Move by a delta, adjust for collisions, and send

    // Does not send movement. That must be done separately
    pub fn move_entity(&self, caller: &dyn EntityBase, mut motion: Vector3<f64>) {
        if caller.get_player().is_some() {
            return;
        }

        if self.no_physics.load(Ordering::Relaxed) {
            self.move_pos(motion);
            self.horizontal_collision.store(false, Ordering::Relaxed);
            self.on_ground.store(false, Ordering::Relaxed);

            return;
        }

        let movement_multiplier = self.movement_multiplier.swap(Vector3::default());

        if movement_multiplier.length_squared() > 1.0e-7 {
            motion = motion.multiply(
                movement_multiplier.x,
                movement_multiplier.y,
                movement_multiplier.z,
            );

            self.velocity.store(Vector3::default());
        }

        let final_move = self.adjust_movement_for_collisions(motion, caller);

        self.move_pos(final_move);

        let velocity_multiplier = f64::from(caller.get_block_speed_factor());

        self.velocity.store(final_move * velocity_multiplier);

        if let Some(living) = caller.get_living_entity() {
            let on_ground = self.on_ground.load(Ordering::SeqCst);
            living.fall(caller, final_move.y, on_ground, false);
        }

        if motion.y != final_move.y {
            let world = self.world.load();
            let block = self.get_block_with_y_offset(0.2).1;
            world
                .block_registry
                .update_entity_movement_after_fall_on(block, caller);
        }
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

        let amplitude = rand::random::<f64>().mul_add(0.2, 0.1);

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

                let rt_handle = world_clone.server.upgrade().map(|s| s.runtime.clone());
                rayon::spawn(move || {
                    let _guard = rt_handle.as_ref().map(tokio::runtime::Handle::enter);
                    let Some(entity_arc) = world_clone.get_entity_by_id(entity_id) else {
                        return;
                    };
                    let transition = portal_type.get_portal_destination(
                        &world_clone,
                        dest_world_opt,
                        entity_arc.as_ref(),
                        src_portal.as_ref(),
                    );

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
            server
                .plugin_manager
                .fire_blocking(&server, &mut portal_event);
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
            && self.age.load(Ordering::Relaxed) % Self::FREEZE_DAMAGE_INTERVAL == 0
        {
            let world = self.world.load_full();
            if let Some(entity) = world.get_entity_by_id(self.entity_id) {
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
        self.set_pitch(pitch);
    }

    pub fn set_pitch(&self, pitch: f32) {
        self.pitch.store(pitch.clamp(-90.0, 90.0) % 360.0);
    }

    /// Removes the `Entity` from their current `World`
    pub fn remove(&self) {
        self.world.load().remove_entity(self);
    }

    pub fn create_spawn_packet(&self) -> CSpawnEntity {
        let entity_loc = self.pos.load();
        let entity_vel = self.velocity.load();
        CSpawnEntity::new(
            VarInt(self.entity_id),
            self.entity_uuid,
            VarInt(i32::from(self.entity_type.id)),
            entity_loc,
            self.pitch.load(),
            self.yaw.load(),
            self.head_yaw.load(), // todo: head_yaw and yaw are swapped, find out why
            self.data.load(Relaxed).into(),
            entity_vel,
        )
    }

    pub fn create_spawn_living_packet(&self, metadata: Option<Box<[u8]>>) -> CSpawnLivingEntity {
        let entity_loc = self.pos.load();
        let entity_vel = self.velocity.load();
        CSpawnLivingEntity::new(
            VarInt(self.entity_id),
            self.entity_uuid,
            VarInt(i32::from(self.entity_type.id)),
            entity_loc,
            self.pitch.load(),
            self.yaw.load(),
            self.head_yaw.load(),
            entity_vel,
            metadata,
        )
    }
    pub fn width(&self) -> f32 {
        self.entity_dimension.load().width
    }

    pub fn height(&self) -> f32 {
        self.entity_dimension.load().height
    }

    /// Applies knockback to the entity, following vanilla Minecraft's mechanics.
    ///
    /// This function calculates the entity's new velocity based on the specified knockback strength and direction.
    pub fn knockback(&self, strength: f64, x: f64, z: f64) {
        // This has some vanilla magic
        let mut x = x;
        let mut z = z;
        while x.mul_add(x, z * z) < 1.0E-5 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * strength;
        let velocity = self.velocity.load();
        self.velocity.store(Vector3::new(
            velocity.x / 2.0 - var8.x,
            if self.on_ground.load(Relaxed) {
                (velocity.y / 2.0 + strength).min(0.4)
            } else {
                velocity.y
            },
            velocity.z / 2.0 - var8.z,
        ));
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

    #[must_use]
    pub fn is_submerged_in_water(&self) -> bool {
        let pos = self.pos.load();
        let eye_height = self.get_eye_height();
        let eye_pos = BlockPos::floored(pos.x, pos.y + eye_height - 0.111_111_11, pos.z);
        let world = self.world.load();
        let (fluid, _) = world.get_fluid_and_fluid_state(&eye_pos);
        fluid.id == Fluid::WATER.id || fluid.id == Fluid::FLOWING_WATER.id
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
                server.plugin_manager.fire_blocking(&server, &mut event);
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

    /// Sets whether the entity is glowing and sends updated metadata.
    pub fn set_glowing(&self, glowing: bool) {
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
    pub fn check_fall_flying(&self) -> bool {
        !self.on_ground.load(Relaxed)
    }

    pub fn set_fall_flying(&self, fall_flying: bool) {
        assert_ne!(self.fall_flying.load(Relaxed), fall_flying);
        self.fall_flying.store(fall_flying, Relaxed);
        self.set_flag(Flag::FallFlying, fall_flying);
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
        if self.synched_data.set(tracked, value) {
            self.send_dirty_entity_data();
            true
        } else {
            false
        }
    }

    pub fn send_bedrock_actor_data(&self, bedrock_meta: &SyncedActorDataList) {
        let world = self.world.load();
        let players = world.players.load();
        let mut bedrock_recipients = Vec::new();

        if let Some(tracked) = world.entity_tracker.get_tracked_entity(self.entity_id) {
            for player in players.iter() {
                if (tracked.seen_by.contains(&player.gameprofile.id)
                    || player.entity_id() == self.entity_id)
                    && let ClientPlatform::Bedrock(client) = player.client.as_ref()
                {
                    bedrock_recipients.push(client);
                }
            }
        } else {
            let chunk_pos = self.chunk_pos.load();
            for player in players.iter() {
                let center = player.get_entity().chunk_pos.load();
                let view_distance = crate::world::chunker::get_view_distance(player).get() as i32;

                if is_within_view_distance(chunk_pos, center, view_distance)
                    && let ClientPlatform::Bedrock(client) = player.client.as_ref()
                {
                    bedrock_recipients.push(client);
                }
            }
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

    pub fn send_dirty_entity_data(&self) {
        if !self.synched_data.is_dirty() {
            return;
        }

        let world = self.world.load();
        let players = world.players.load();

        let mut java_recipients = Vec::new();

        if let Some(tracked) = world.entity_tracker.get_tracked_entity(self.entity_id) {
            for player in players.iter() {
                if (tracked.seen_by.contains(&player.gameprofile.id)
                    || player.entity_id() == self.entity_id)
                    && let ClientPlatform::Java(_) = player.client.as_ref()
                {
                    java_recipients.push(player);
                }
            }
        } else {
            let chunk_pos = self.chunk_pos.load();
            for player in players.iter() {
                let center = player.get_entity().chunk_pos.load();
                let view_distance = crate::world::chunker::get_view_distance(player).get() as i32;

                if is_within_view_distance(chunk_pos, center, view_distance)
                    && let ClientPlatform::Java(_) = player.client.as_ref()
                {
                    java_recipients.push(player);
                }
            }
        }

        if java_recipients.is_empty() {
            return;
        }

        let recipients_by_version =
            World::collect_java_recipients_by_version(java_recipients.into_iter());

        let bedrock_player = self.entity_type == &EntityType::PLAYER
            && world
                .get_player_by_id(self.entity_id)
                .is_some_and(|subject| {
                    matches!(subject.client.as_ref(), ClientPlatform::Bedrock(_))
                });
        for (version, recipients) in recipients_by_version {
            // TODO: Support older versions
            if version < JavaMinecraftVersion::V_26_2 {
                continue;
            }
            let serialize = |metadata: Option<Box<[u8]>>| {
                metadata.and_then(|buf| {
                    JavaClient::serialize_packet_for_version(
                        &CSetEntityMetadata::new(self.entity_id.into(), buf),
                        version,
                    )
                    .ok()
                })
            };
            let normal = serialize(self.synched_data.pack_dirty_for_version(&version));
            if bedrock_player {
                let mannequin =
                    serialize(self.synched_data.pack_dirty_for_version_filtered(
                        &version,
                        player::mannequin_shared_metadata,
                    ));
                for recipient in recipients {
                    let presentation = recipient
                        .bedrock_mannequins
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let packet = if presentation.contains(&self.entity_id) {
                        &mannequin
                    } else {
                        &normal
                    };
                    if let Some(packet) = packet {
                        recipient.try_enqueue_packet(packet.clone());
                    }
                }
            } else if let Some(packet) = normal {
                for recipient in recipients {
                    recipient.try_enqueue_packet(packet.clone());
                }
            }
        }
        self.synched_data.clear_dirty();
    }

    pub fn set_pose(&self, pose: EntityPose) {
        if self.pose.load() == pose {
            return;
        }

        let mut pose_event =
            crate::plugin::api::events::entity::entity_pose_change::EntityPoseChangeEvent::new(
                self.entity_id,
                (pose as u8).to_string(),
            );
        if let Some(server) = self.world.load().server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut pose_event);
            if pose_event.cancelled {
                return;
            }
        }

        let dimension = Self::get_entity_dimensions(pose);
        let position = self.pos.load();
        let aabb = BoundingBox::new_from_pos(position.x, position.y, position.z, &dimension);
        self.pose.store(pose);
        self.bounding_box.store(aabb);
        self.entity_dimension.store(dimension);
        let pose = pose as i32;
        let mut bedrock_meta = SyncedActorDataList::new();
        bedrock_meta.set(entity_data_key::POSE_INDEX, MetadataValue::Int(pose));
        bedrock_meta.set(
            entity_data_key::WIDTH,
            MetadataValue::Float(dimension.width),
        );
        bedrock_meta.set(
            entity_data_key::HEIGHT,
            MetadataValue::Float(dimension.height),
        );
        self.set_synced_data(tracked_data::entity::DATA_POSE, VarInt(pose));
        self.send_bedrock_actor_data(&bedrock_meta);
    }

    /// Checks if the entity is invulnerable to the given damage type, considering both general invulnerability and specific immunities.
    pub fn is_invulnerable_to(&self, damage_type: &DamageType) -> bool {
        // Nothing is immune to void or kill
        if matches!(
            *damage_type,
            DamageType::GENERIC_KILL | DamageType::OUT_OF_WORLD
        ) {
            return false;
        }

        // General invulnerability
        if self.invulnerable.load(Ordering::Relaxed) {
            return true;
        }

        // Specific type immunities
        self.damage_immunities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(damage_type)
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
        world.broadcast_to_chunk(
            chunk_pos,
            &CEntityPositionSync::new(
                self.entity_id.into(),
                position,
                self.velocity.load(),
                yaw.unwrap_or(self.yaw.load()),
                pitch.unwrap_or(self.pitch.load()),
                self.on_ground.load(Ordering::SeqCst),
            ),
        );
    }

    pub fn get_eye_pos(&self) -> Vector3<f64> {
        let pos = self.pos.load();
        Vector3::new(
            pos.x,
            pos.y + f64::from(self.entity_dimension.load().eye_height),
            pos.z,
        )
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

    pub fn leash_to(&self, holder: Arc<dyn EntityBase>) {
        let holder_entity_id = holder.get_entity().entity_id;
        let world = self.world.load();
        if let Some(server) = world.server.upgrade()
            && let Some(player) = holder.get_player()
            && let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id)
        {
            let mut event =
                crate::plugin::api::events::player::player_leash_entity::PlayerLeashEntityEvent {
                    player: player_arc,
                    entity_id: self.entity_id,
                    holder_id: holder_entity_id,
                    cancelled: false,
                };
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
        }

        *self
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(holder);

        let je_packet = pumpkin_protocol::java::client::play::CSetEntityLink::new(
            self.entity_id,
            holder_entity_id,
            true,
        );
        let be_packet = pumpkin_protocol::bedrock::client::CSetActorLink {
            link: pumpkin_protocol::bedrock::client::common::ActorLink {
                ridden_unique_id: pumpkin_protocol::codec::var_long::VarLong(self.entity_id as i64),
                rider_unique_id: pumpkin_protocol::codec::var_long::VarLong(
                    holder_entity_id as i64,
                ),
                link_type: 1, // Leash link
                immediate: true,
                rider_initiated: false,
                vehicle_angular_velocity: 0.0,
            },
        };

        self.world.load().broadcast_to_chunk_editioned(
            self.chunk_pos.load(),
            &je_packet,
            &be_packet,
        );
    }

    pub fn unleash(&self) {
        let world = self.world.load();
        if let Some(server) = world.server.upgrade() {
            let mut event =
                crate::plugin::api::events::entity::entity_unleash::EntityUnleashEvent::new(
                    self.entity_id,
                    "unleashed".to_string(),
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
        }

        let old_holder = self
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if old_holder.is_none() {
            return;
        }

        if let Some(holder) = &old_holder
            && let Some(server) = world.server.upgrade()
            && let Some(player) = holder.get_player()
            && let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id)
        {
            let mut event = crate::plugin::api::events::player::player_unleash_entity::PlayerUnleashEntityEvent {
                player: player_arc,
                entity_id: self.entity_id,
                cancelled: false,
            };
            server.plugin_manager.fire_blocking(&server, &mut event);
        }

        let je_packet =
            pumpkin_protocol::java::client::play::CSetEntityLink::new(self.entity_id, -1, true);
        let be_packet = pumpkin_protocol::bedrock::client::CSetActorLink {
            link: pumpkin_protocol::bedrock::client::common::ActorLink {
                ridden_unique_id: pumpkin_protocol::codec::var_long::VarLong(self.entity_id as i64),
                rider_unique_id: pumpkin_protocol::codec::var_long::VarLong(-1),
                link_type: 0, // Unlink
                immediate: true,
                rider_initiated: false,
                vehicle_angular_velocity: 0.0,
            },
        };

        self.world.load().broadcast_to_chunk_editioned(
            self.chunk_pos.load(),
            &je_packet,
            &be_packet,
        );
    }

    pub fn tick_leash(&self) {
        let holder = {
            let Ok(guard) = self.leashed_to.try_lock() else {
                return;
            };
            guard.clone()
        };

        if let Some(holder) = holder {
            let holder_entity = holder.get_entity();

            // Drop leash if entity or holder is removed or dead
            if !self.is_alive() || !holder_entity.is_alive() {
                self.unleash();
                return;
            }

            let self_pos = self.pos.load();
            let holder_pos = holder_entity.pos.load();
            let diff = self_pos - holder_pos;
            let distance = diff.length();

            if distance > Self::LEASH_SNAP_DISTANCE {
                // Too far: snap/break leash and drop lead item
                self.unleash();
                let lead_item =
                    pumpkin_data::item_stack::ItemStack::new(1, &pumpkin_data::item::Item::LEAD);
                self.world
                    .load()
                    .drop_stack(&self.block_pos.load(), lead_item);
            } else if distance > Self::LEASH_ELASTIC_DISTANCE {
                // Elastic pull force towards leash holder
                let dir = (holder_pos - self_pos).normalize();
                let pull_strength = (distance - Self::LEASH_ELASTIC_DISTANCE) * 0.11;
                let current_vel = self.velocity.load();
                self.velocity.store(current_vel + dir * pull_strength);
                self.velocity_dirty.store(true, Relaxed);
            }
        }
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
            .is_some()
    }

    pub fn get_vehicle(&self) -> Option<Arc<dyn EntityBase>> {
        self.vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn is_leashed(&self) -> bool {
        self.leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    pub fn add_passenger(&self, vehicle: Arc<dyn EntityBase>, passenger: Arc<dyn EntityBase>) {
        let mut mount_event =
            crate::plugin::api::events::entity::entity_mount::EntityMountEvent::new(
                passenger.get_entity().entity_id,
                self.entity_id,
            );
        let mut vehicle_enter =
            crate::plugin::api::events::vehicle::vehicle_enter::VehicleEnterEvent::new(
                self.entity_id,
                passenger.get_entity().entity_id,
            );
        if let Some(server) = self.world.load().server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut mount_event);
            server
                .plugin_manager
                .fire_blocking(&server, &mut vehicle_enter);
        }
        if mount_event.cancelled || vehicle_enter.cancelled {
            return;
        }

        let passenger_entity = passenger.get_entity();
        *passenger_entity
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(vehicle);

        let mut passengers = self
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        passengers.push(passenger);

        let passenger_ids: Vec<VarInt> = passengers
            .iter()
            .map(|p| VarInt(p.get_entity().entity_id))
            .collect();

        let world = self.world.load();
        let chunk_pos = self.chunk_pos.load();
        world.broadcast_to_chunk(
            chunk_pos,
            &CSetPassengers::new(VarInt(self.entity_id), &passenger_ids),
        );
    }

    pub(crate) fn remove_passenger_on_disconnect(&self, passenger_id: i32) {
        let mut passengers = self
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(index) = passengers
            .iter()
            .position(|passenger| passenger.get_entity().entity_id == passenger_id)
        {
            let passenger = passengers.remove(index);
            *passenger
                .get_entity()
                .vehicle
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        }

        let passenger_ids: Vec<VarInt> = passengers
            .iter()
            .map(|passenger| VarInt(passenger.get_entity().entity_id))
            .collect();
        drop(passengers);

        self.world.load().broadcast_to_chunk(
            self.chunk_pos.load(),
            &CSetPassengers::new(VarInt(self.entity_id), &passenger_ids),
        );
    }

    pub fn remove_passenger_sync(&self, passenger_id: i32) {
        self.remove_passenger_on_disconnect(passenger_id);
    }

    pub fn remove_passenger(&self, passenger_id: i32) {
        self.remove_passenger_internal(passenger_id, true);
    }

    pub fn remove_passenger_before_teleport(&self, passenger_id: i32) {
        self.remove_passenger_internal(passenger_id, false);
    }

    #[allow(clippy::too_many_lines)]
    fn remove_passenger_internal(&self, passenger_id: i32, reposition: bool) {
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
            server
                .plugin_manager
                .fire_blocking(&server, &mut dismount_event);
            server
                .plugin_manager
                .fire_blocking(&server, &mut vehicle_exit);
        }
        if dismount_event.cancelled || vehicle_exit.cancelled {
            return;
        }

        let (removed_passenger, passenger_ids) = {
            let mut passengers = self
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let removed_passenger = passengers
                .iter()
                .position(|p| p.get_entity().entity_id == passenger_id)
                .map(|idx| {
                    let passenger = passengers.remove(idx);
                    *passenger
                        .get_entity()
                        .vehicle
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
                    passenger
                });

            let passenger_ids: Vec<VarInt> = passengers
                .iter()
                .map(|p| VarInt(p.get_entity().entity_id))
                .collect();
            (removed_passenger, passenger_ids)
        };

        let chunk_pos = self.chunk_pos.load();

        if let Some(passenger) = removed_passenger {
            let vehicle_box = self.bounding_box.load();
            let passenger_entity = passenger.get_entity();

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
                let placeholder =
                    Vector3::new(self.pos.load().x, vehicle_box.max.y, self.pos.load().z);
                *player
                    .awaiting_teleport
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                    Some((id.into(), placeholder));
                Some(id)
            } else {
                None
            };

            // Vanilla: ridingCooldown = 60 (prevents immediate re-mount)
            passenger_entity.riding_cooldown.store(60, Relaxed);
            // TODO: world.emitGameEvent(passenger, GameEvent.ENTITY_DISMOUNT, vehicle.pos)

            // Send CSetPassengers directly to the dismounting player before broadcasting it.
            let world = self.world.load();
            let passengers_packet = CSetPassengers::new(VarInt(self.entity_id), &passenger_ids);
            if let Some(player) = passenger.get_player() {
                player.try_send_client_packet(&passengers_packet);
                world.broadcast_to_chunk_except(
                    chunk_pos,
                    &[player.get_entity().entity_uuid],
                    &passengers_packet,
                );
            } else {
                world.broadcast_to_chunk(chunk_pos, &passengers_packet);
            }

            if !reposition {
                return;
            }

            // Calculate dismount directions and offsets (vanilla DismountHelper)
            let vehicle_yaw = self.yaw.load();
            // Wrap yaw to 0..360 range
            let wrapped_yaw = (vehicle_yaw % 360.0 + 360.0) % 360.0;
            let forward_dir = if !(45.0..315.0).contains(&wrapped_yaw) {
                BlockDirection::South
            } else if (45.0..135.0).contains(&wrapped_yaw) {
                BlockDirection::West
            } else if (135.0..225.0).contains(&wrapped_yaw) {
                BlockDirection::North
            } else {
                BlockDirection::East
            };

            let get_step = |dir: BlockDirection| -> (i32, i32) {
                match dir {
                    BlockDirection::North => (0, -1),
                    BlockDirection::South => (0, 1),
                    BlockDirection::East => (1, 0),
                    BlockDirection::West => (-1, 0),
                    _ => (0, 0),
                }
            };

            let get_clockwise = |dir: BlockDirection| -> BlockDirection {
                match dir {
                    BlockDirection::North => BlockDirection::East,
                    BlockDirection::East => BlockDirection::South,
                    BlockDirection::South => BlockDirection::West,
                    BlockDirection::West => BlockDirection::North,
                    other => other,
                }
            };

            let get_opposite = |dir: BlockDirection| -> BlockDirection {
                match dir {
                    BlockDirection::North => BlockDirection::South,
                    BlockDirection::South => BlockDirection::North,
                    BlockDirection::East => BlockDirection::West,
                    BlockDirection::West => BlockDirection::East,
                    other => other,
                }
            };

            let right_dir = get_clockwise(forward_dir);
            let left_dir = get_opposite(right_dir);
            let back_dir = get_opposite(forward_dir);

            let (fx, fz) = get_step(forward_dir);
            let (rx, rz) = get_step(right_dir);
            let (lx, lz) = get_step(left_dir);
            let (bx, bz) = get_step(back_dir);

            let offsets = [
                (rx, rz),
                (lx, lz),
                (bx + rx, bz + rz),
                (bx + lx, bz + lz),
                (fx + rx, fz + rz),
                (fx + lx, fz + lz),
                (bx, bz),
                (fx, fz),
            ];

            let target_block_y = vehicle_box.max.y.floor() as i32;
            let below_pos = BlockPos(Vector3::new(
                self.pos.load().x.floor() as i32,
                target_block_y - 1,
                self.pos.load().z.floor() as i32,
            ));

            let below_state_id = world.get_block_state_id(&below_pos);
            // Vanilla: isWater checks specifically for water fluid, not any fluid
            let is_water = Fluid::from_state_id(below_state_id)
                .is_some_and(|f| f.id == Fluid::WATER.id || f.id == Fluid::FLOWING_WATER.id);

            let fallback_pos =
                Vector3::new(self.pos.load().x, vehicle_box.max.y, self.pos.load().z);

            let dismount_pos = if is_water {
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
                    let dims = Self::get_entity_dimensions(pose);

                    for y_offset in y_offsets {
                        for &(ox, oz) in &offsets {
                            let target_block_x = vehicle_block_pos.0.x + ox;
                            let target_block_y = vehicle_block_pos.0.y + y_offset;
                            let target_block_z = vehicle_block_pos.0.z + oz;

                            let target_pos = BlockPos(Vector3::new(
                                target_block_x,
                                target_block_y,
                                target_block_z,
                            ));
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
                        let dims = Self::get_entity_dimensions(pose);
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
            };

            // Clean up any remaining reference to the dismounted passenger.
            passenger_entity.set_pos(dismount_pos);

            // Phase 2: Teleport to safety (unblocks movement)
            if let Some(player) = passenger.get_player() {
                if let Some(id) = teleport_id {
                    player.get_entity().set_pos(dismount_pos);
                    // Update awaiting_teleport with the real dismount position
                    *player
                        .awaiting_teleport
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) =
                        Some((id.into(), dismount_pos));
                    // Use send_client_packet so the teleport goes through
                    // the same packet queue as CSetPassengers, preserving send order.
                    // Vanilla uses DELTA | ROT flags: position absolute, delta/rotation relative.
                    // With rotation relative and yaw/pitch=0, the client preserves its current look.
                    player.try_send_client_packet(&CPlayerPosition::new(
                        id.into(),
                        dismount_pos,
                        Vector3::new(0.0, 0.0, 0.0),
                        0.0,
                        0.0,
                        vec![
                            PositionFlag::DeltaX,
                            PositionFlag::DeltaY,
                            PositionFlag::DeltaZ,
                            PositionFlag::YRot,
                            PositionFlag::XRot,
                        ],
                    ));
                }

                // Vanilla: setSneaking(false) after dismount via sneak input
                if passenger_entity.sneaking.load(Relaxed) {
                    passenger_entity.set_sneaking(false);
                }
            } else {
                passenger_entity.set_pos(dismount_pos);
            }
        } else {
            // No passenger was removed, still need to broadcast the passenger list
            let world = self.world.load();
            world.broadcast_to_chunk(
                chunk_pos,
                &CSetPassengers::new(VarInt(self.entity_id), &passenger_ids),
            );
        }
    }

    pub fn check_out_of_world(&self, dyn_self: &dyn EntityBase) {
        if self.pos.load().y < f64::from(self.world.load().dimension.min_y) - 64.0 {
            dyn_self.tick_in_void(dyn_self);
        }
    }

    pub fn reset_state(&self) {
        self.pose.store(EntityPose::Standing);
        self.fall_flying.store(false, Relaxed);
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
        nbt.put(
            "Pos",
            NbtTag::List(vec![
                position.x.into(),
                position.y.into(),
                position.z.into(),
            ]),
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
        nbt.put_short("Fire", self.fire_ticks.load(Relaxed) as i16);
        nbt.put_bool("OnGround", self.on_ground.load(Relaxed));
        nbt.put_bool("Invulnerable", self.invulnerable.load(Relaxed));
        nbt.put_int("PortalCooldown", self.portal_cooldown.load(Relaxed) as i32);
        if self.has_visual_fire.load(Relaxed) {
            nbt.put_bool("HasVisualFire", true);
        }
        nbt.put_int("TicksFrozen", self.frozen_ticks.load(Relaxed));
        if let Some(custom_name) = &**self.custom_name.load()
            && let Ok(name_json) = pumpkin_util::serde_json::to_string(custom_name)
        {
            nbt.put_string("CustomName", name_json);
        }
        nbt.put_bool("CustomNameVisible", self.custom_name_visible.load(Relaxed));

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
            nbt.put_compound("PumpkinCustomData", custom_data.clone());
        }

        // todo more...
    }

    pub fn read_nbt_non_mut(&self, nbt: &NbtCompound) {
        if let Some(position) = nbt.get_list("Pos")
            && position.len() >= 3
        {
            let x = position[0].extract_double().unwrap_or(0.0);
            let y = position[1].extract_double().unwrap_or(0.0);
            let z = position[2].extract_double().unwrap_or(0.0);
            let pos = Vector3::new(x, y, z);
            self.set_pos(pos);
            self.last_sent_pos.store(pos);
        }
        if let Some(velocity) = nbt.get_list("Motion")
            && velocity.len() >= 3
        {
            let x = velocity[0].extract_double().unwrap_or(0.0);
            let y = velocity[1].extract_double().unwrap_or(0.0);
            let z = velocity[2].extract_double().unwrap_or(0.0);
            self.velocity.store(Vector3::new(x, y, z));
        }
        if let Some(rotation) = nbt.get_list("Rotation")
            && rotation.len() >= 2
        {
            let yaw = rotation[0].extract_float().unwrap_or(0.0);
            let pitch = rotation[1].extract_float().unwrap_or(0.0);
            self.set_rotation(yaw, pitch);
            let yaw_byte = (yaw * 256.0 / 360.0).rem_euclid(256.0) as u8;
            let pitch_byte = (pitch * 256.0 / 360.0).rem_euclid(256.0) as u8;
            self.last_sent_yaw.store(yaw_byte, Relaxed);
            self.last_sent_pitch.store(pitch_byte, Relaxed);
            self.head_yaw.store(yaw);
            self.last_sent_head_yaw.store(yaw_byte, Relaxed);
        }
        self.fire_ticks
            .store(i32::from(nbt.get_short("Fire").unwrap_or(0)), Relaxed);
        self.on_ground
            .store(nbt.get_bool("OnGround").unwrap_or(false), Relaxed);
        self.invulnerable
            .store(nbt.get_bool("Invulnerable").unwrap_or(false), Relaxed);
        self.portal_cooldown
            .store(nbt.get_int("PortalCooldown").unwrap_or(0) as u32, Relaxed);
        self.has_visual_fire
            .store(nbt.get_bool("HasVisualFire").unwrap_or(false), Relaxed);
        self.frozen_ticks
            .store(nbt.get_int("TicksFrozen").unwrap_or(0), Relaxed);
        if let Some(name_json) = nbt.get_string("CustomName")
            && let Ok(component) = pumpkin_util::serde_json::from_str(name_json)
        {
            self.custom_name.store(Arc::new(Some(component)));
        }
        self.custom_name_visible
            .store(nbt.get_bool("CustomNameVisible").unwrap_or(false), Relaxed);

        if let Some(tag_list) = nbt.get_list("Tags") {
            let mut tags = self
                .scoreboard_tags
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            tags.clear();
            tags.extend(
                tag_list
                    .iter()
                    .filter_map(|tag| tag.extract_string().map(str::to_owned))
                    .take(MAX_SCOREBOARD_TAGS),
            );
        }

        if let Some(custom_data) = nbt
            .get_compound("PumpkinCustomData")
            .or_else(|| nbt.get_compound("BukkitValues"))
        {
            let mut data = self
                .custom_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *data = custom_data.clone();
        }

        // todo more...
    }
}

impl EntityBase for Entity {
    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        // Recomputed during movement/block-collision handling in the same tick.
        let was_in_powder_snow = self.is_in_powder_snow.load(Ordering::Relaxed);
        self.was_in_powder_snow
            .store(was_in_powder_snow, Ordering::Relaxed);
        self.is_in_powder_snow.store(false, Ordering::Relaxed);

        self.update_last_pos();
        self.tick_portal(caller);
        self.update_fluid_state(caller);
        self.check_out_of_world(caller);
        let fire_ticks = self.fire_ticks.load(Ordering::Relaxed);

        // Check for fire immunity (or if the specific entity is)
        let is_immune = self.entity_type.fire_immune || self.fire_immune.load(Ordering::Relaxed);
        if fire_ticks > 0 {
            if is_immune {
                self.fire_ticks.store(fire_ticks - 4, Ordering::Relaxed);
                if self.fire_ticks.load(Ordering::Relaxed) < 0 {
                    self.extinguish();
                }
            } else {
                if fire_ticks % 20 == 0 {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
