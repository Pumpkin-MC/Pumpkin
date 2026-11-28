use std::cell::{Cell, RefCell};
use std::collections::hash_map::Entry;
use std::sync::atomic::{
    AtomicBool, AtomicU32, AtomicU64,
    Ordering::{self, Relaxed},
};
use std::sync::{Arc, Mutex, MutexGuard, RwLock, Weak};

use bytes::{BufMut, Bytes};
use crossbeam::atomic::AtomicCell;
use dashmap::DashMap;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_data::{data_component_impl::EquipmentSlot, entity::EntityType, item_stack::ItemStack};
use pumpkin_inventory::entity_equipment::EntityEquipment;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_protocol::bedrock::client::move_actor_delta::{
    CMoveActorDelta, MOVE_ACTOR_DELTA_FLAG_HAS_HEAD_YAW, MOVE_ACTOR_DELTA_FLAG_HAS_PITCH,
    MOVE_ACTOR_DELTA_FLAG_HAS_X, MOVE_ACTOR_DELTA_FLAG_HAS_Y, MOVE_ACTOR_DELTA_FLAG_HAS_YAW,
    MOVE_ACTOR_DELTA_FLAG_HAS_Z, MOVE_ACTOR_DELTA_FLAG_ON_GROUND,
};
use pumpkin_protocol::bedrock::client::remove_actor::CRemoveActor;
use pumpkin_protocol::bedrock::{
    client::{
        CSetActorMotion, mob_armor_equipment::CMobArmorEquipment, mob_equipment::CMobEquipment,
    },
    network_item::NetworkItemStackDescriptor,
};
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::codec::{var_uint::VarUInt, var_ulong::VarULong};
use pumpkin_protocol::java::client::play::{
    CEntityPositionSync, CEntityVelocity, CHeadRot, CRemoveEntities, CSetEntityMetadata,
    CSetEquipment, CUpdateEntityPos, CUpdateEntityPosRot, CUpdateEntityRot, Metadata,
};
use pumpkin_protocol::ser::{NetworkReadExt, WritingError};
use pumpkin_protocol::serial::PacketRead;
use pumpkin_protocol::{
    BClientPacket, ClientPacket, Packet, ServerPacket, packet::MultiVersionJavaPacket,
};
use pumpkin_util::GameMode;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{pack_degrees, position::chunk_section_from_pos};
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use uuid::Uuid;

use crate::entity::{Entity, EntityBase};
use crate::entity::{living::LivingEntity, player::Player};
use crate::net::java::JavaClient;
use crate::net::{ClientPlatform, bedrock::BedrockClient};
use crate::world::World;
use crate::world::chunker::get_view_distance;

/// Vanilla `VecDeltaCodec.encode`: Java `Math.round` on the 1/4096
fn encode_pos(value: f64) -> i64 {
    (value * 4096.0 + 0.5).floor() as i64
}

const MAX_RELATIVE_MOVE_TICKS: u32 = 400;

/// Vanilla `ServerEntity.sendChanges` packet choice.
#[derive(Clone, Copy)]
enum MoveSync {
    Pos(Vector3<i16>),
    Rot,
    PosRot(Vector3<i16>),
    Absolute,
}

impl MoveSync {
    const fn sends_position(self) -> bool {
        !matches!(self, Self::Rot)
    }

    const fn sends_rotation(self) -> bool {
        !matches!(self, Self::Pos(_))
    }
}

#[derive(Clone, Copy)]
struct PlayerViewSnapshot<'a> {
    player: &'a Arc<Player>,
    entity_id: i32,
    pos: Vector3<f64>,
    watched: Cylindrical,
    view_distance: i32,
}

struct TrackingUpdate<'a> {
    entity: &'a Entity,
    world: &'a World,
    position: Vector3<f64>,
    teleport_epoch: u64,
}

impl TrackingUpdate<'_> {
    fn lock_if_current(&self) -> Option<impl Sized + '_> {
        let motion_guard = self
            .entity
            .lock_vehicle_motion_if_current(self.teleport_epoch)?;
        (!self.entity.is_removed()
            && std::ptr::eq(self.entity.world.load().as_ref(), self.world)
            && self.entity.pos.load() == self.position)
            .then_some(motion_guard)
    }
}

pub struct TrackedEntity {
    pub entity: Arc<dyn EntityBase>,
    pub entity_id: i32,
    pub tracking_range: u32,
    pub update_interval: u32,
    pub track_deltas: bool,
    seen_by: RwLock<FxHashMap<Uuid, TrackingViewer>>,
    retired: AtomicBool,
    pairing_equipment: Mutex<EquipmentPairings>,
    next_equipment_pairing: AtomicU64,
    equipment_retry_pending: AtomicBool,
    passenger_retries: Mutex<FxHashMap<(Uuid, bool), PassengerPairingRetry>>,
    passenger_retry_pending: AtomicBool,
    effect_retries: Mutex<EffectPairingRetries>,
    effect_retry_pending: AtomicBool,
    /// Bedrock players in `seen_by`.
    bedrock_watchers: AtomicU32,
    pub last_section_pos: AtomicCell<Vector3<i32>>,
    pos_dirty: AtomicBool,
    tick_count: AtomicU32,
    teleport_delay: AtomicU32,
    was_on_ground: AtomicBool,
    was_riding: AtomicBool,
    bedrock_pos: AtomicCell<Vector3<f64>>,
    /// Packed pitch, yaw, head yaw.
    bedrock_rot: AtomicCell<[u8; 3]>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct EquipmentSession {
    viewer: Uuid,
    generation: u64,
}

type EffectPairingRetries = FxHashMap<
    (Uuid, Option<&'static pumpkin_data::effect::StatusEffect>),
    (Weak<Player>, EquipmentSession),
>;

struct PassengerPairingRetry {
    viewer: Weak<Player>,
    carrier: Weak<dyn EntityBase>,
    session: EquipmentSession,
    paired_rider: Option<i32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PassengerSnapshot {
    Finished,
    Changed,
    Busy,
}

#[derive(Clone)]
struct TrackingViewer {
    player: Arc<Player>,
    generation: u64,
}

pub(crate) struct EquippedStacks<'a> {
    gear: std::sync::MutexGuard<'a, EntityEquipment>,
    player_main: Option<(
        &'a PlayerInventory,
        std::sync::RwLockReadGuard<'a, [ItemStack; PlayerInventory::MAIN_SIZE]>,
    )>,
}

impl<'a> EquippedStacks<'a> {
    pub(crate) fn lock(living: &'a LivingEntity, player: Option<&'a Player>) -> Self {
        let player_main = player.map(|player| {
            let inventory = player.inventory().as_ref();
            (
                inventory,
                inventory
                    .main_inventory
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            )
        });
        let gear = living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self { gear, player_main }
    }

    pub(crate) fn get(&self, slot: &EquipmentSlot) -> Option<&ItemStack> {
        if *slot == EquipmentSlot::MAIN_HAND
            && let Some((inventory, main)) = &self.player_main
        {
            main.get(inventory.get_selected_slot() as usize)
        } else {
            self.gear.equipment.get(slot)
        }
    }

    pub(crate) fn matches(
        &self,
        slot: &EquipmentSlot,
        snapshot: Option<&ItemStack>,
        bedrock: bool,
    ) -> bool {
        TrackedEntity::equipment_stacks_match(self.get(slot), snapshot, bedrock)
    }
}

struct JavaEquipmentUpdate {
    slots: Vec<EquipmentSlot>,
    packet: CSetEquipment,
    data: Bytes,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct BedrockEquipmentGroup {
    packet_id: u32,
    entity_id: u64,
    container: Option<u8>,
}

type EquipmentPairings = FxHashMap<Uuid, EquipmentPairingState>;

struct EquipmentPairingState {
    viewer: Arc<Player>,
    generation: u64,
    java_items: FxHashMap<i8, ItemStack>,
    java_spawn_overrides: FxHashSet<i8>,
    bedrock_items: FxHashMap<BedrockEquipmentGroup, Bytes>,
    pending: Vec<EquipmentSlot>,
    needs_snapshot: bool,
    retry_in_flight: bool,
    spawned: bool,
}

struct EquipmentPairing<'a> {
    tracker: &'a TrackedEntity,
    session: EquipmentSession,
}

impl Drop for EquipmentPairing<'_> {
    fn drop(&mut self) {
        let mut pairings = self.tracker.lock_equipment_pairings();
        if let std::collections::hash_map::Entry::Occupied(entry) =
            pairings.entry(self.session.viewer)
            && entry.get().generation == self.session.generation
            && entry.get().pending.is_empty()
            && !entry.get().needs_snapshot
            && !entry.get().retry_in_flight
        {
            entry.remove();
        }
    }
}

impl TrackedEntity {
    #[must_use]
    pub fn new(
        entity: Arc<dyn EntityBase>,
        range: u32,
        update_interval: u32,
        track_deltas: bool,
    ) -> Self {
        let base = entity.get_entity();
        let entity_id = base.entity_id;
        let pos = base.pos.load();
        let on_ground = base.on_ground.load(Relaxed);
        let bedrock_pos = entity.bedrock_pos();
        let rot = [
            pack_degrees(base.pitch.load()),
            pack_degrees(base.yaw.load()),
            pack_degrees(base.head_yaw.load()),
        ];
        // Vanilla `ServerEntity` ctor: sync state starts at the current values.
        base.last_sent_pos.store(pos);
        base.last_sent_velocity.store(base.velocity.load());
        base.last_sent_pitch.store(rot[0], Relaxed);
        base.last_sent_yaw.store(rot[1], Relaxed);
        base.last_sent_head_yaw.store(rot[2], Relaxed);
        let last_section_pos = chunk_section_from_pos(&base.block_pos.load());
        Self {
            entity,
            entity_id,
            tracking_range: range,
            update_interval,
            track_deltas,
            seen_by: RwLock::default(),
            retired: AtomicBool::new(false),
            pairing_equipment: Mutex::default(),
            next_equipment_pairing: AtomicU64::new(0),
            equipment_retry_pending: AtomicBool::new(false),
            passenger_retries: Mutex::default(),
            passenger_retry_pending: AtomicBool::new(false),
            effect_retries: Mutex::default(),
            effect_retry_pending: AtomicBool::new(false),
            bedrock_watchers: AtomicU32::new(0),
            last_section_pos: AtomicCell::new(last_section_pos),
            pos_dirty: AtomicBool::new(false),
            tick_count: AtomicU32::new(0),
            teleport_delay: AtomicU32::new(0),
            was_on_ground: AtomicBool::new(on_ground),
            was_riding: AtomicBool::new(false),
            bedrock_pos: AtomicCell::new(bedrock_pos),
            bedrock_rot: AtomicCell::new(rot),
        }
    }

    #[must_use]
    pub fn has_bedrock_watchers(&self) -> bool {
        self.bedrock_watchers.load(Relaxed) > 0
    }

    pub(crate) fn bedrock_watchers_snapshot(&self) -> Vec<Arc<Player>> {
        if !self.has_bedrock_watchers() {
            return Vec::new();
        }
        self.seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .filter(|viewer| viewer.player.client.bedrock().is_some())
            .map(|viewer| viewer.player.clone())
            .collect()
    }

    #[must_use]
    pub fn has_viewers(&self) -> bool {
        !self
            .seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
    }

    #[must_use]
    pub fn is_seen_by(&self, uuid: &Uuid) -> bool {
        self.seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(uuid)
    }

    /// Vanilla `ServerEntity.sendChanges`. Non-player entities only.
    pub fn send_changes(&self, world: &World) {
        let entity = self.entity.get_entity();
        let update = TrackingUpdate {
            entity,
            world,
            teleport_epoch: entity.vehicle_motion_epoch(),
            position: entity.pos.load(),
        };
        self.send_bedrock_move(&update);
        if update.lock_if_current().is_none() {
            return;
        }
        let interval = self.update_interval.max(1);
        let tick = if entity.sync_position.swap(false, Ordering::SeqCst) {
            // Java `UpdateInterval.nextInterval` advances even when this tick was due.
            let next_interval = |tick: u32| tick.wrapping_add(interval - tick % interval);
            let previous = self
                .tick_count
                .update(Relaxed, Relaxed, |tick| next_interval(tick).wrapping_add(1));
            next_interval(previous)
        } else {
            self.tick_count.fetch_add(1, Relaxed)
        };
        let needs_sync = entity.velocity_dirty.swap(false, Ordering::SeqCst);
        // Head yaw stays live every tick, only position/rotation sync is throttled.
        if tick.is_multiple_of(interval) || needs_sync || entity.synched_data.is_dirty() {
            let yaw = pack_degrees(entity.yaw.load());
            let pitch = pack_degrees(entity.pitch.load());
            let rot_changed = yaw != entity.last_sent_yaw.load(Relaxed)
                || pitch != entity.last_sent_pitch.load(Relaxed);

            if self.entity.is_passenger() {
                if rot_changed {
                    self.send_move(
                        &update,
                        MoveSync::Rot,
                        yaw,
                        pitch,
                        entity.on_ground.load(Relaxed),
                    );
                }
                if update.lock_if_current().is_none() {
                    return;
                }
                entity.send_dirty_entity_data();
                let Some(_motion_guard) = update.lock_if_current() else {
                    return;
                };
                if rot_changed {
                    entity.last_sent_yaw.store(yaw, Relaxed);
                    entity.last_sent_pitch.store(pitch, Relaxed);
                }
                entity.last_sent_pos.store(update.position);
                self.was_riding.store(true, Relaxed);
            } else {
                let on_ground = entity.on_ground.load(Relaxed);
                let teleport_delay = self.teleport_delay.load(Relaxed).wrapping_add(1);
                let sync = self.pick_move(&update, tick, rot_changed, on_ground, teleport_delay);
                if needs_sync || self.track_deltas || entity.is_fall_flying() {
                    self.send_motion(&update);
                }
                if let Some(sync) = sync {
                    self.send_move(&update, sync, yaw, pitch, on_ground);
                }
                if update.lock_if_current().is_none() {
                    self.recover_partial_move(&update, sync);
                    return;
                }
                entity.send_dirty_entity_data();
                let Some(_motion_guard) = update.lock_if_current() else {
                    self.recover_partial_move(&update, sync);
                    return;
                };
                if matches!(sync, Some(MoveSync::Absolute)) {
                    self.was_on_ground.store(on_ground, Relaxed);
                    self.teleport_delay.store(0, Relaxed);
                } else {
                    self.teleport_delay.store(teleport_delay, Relaxed);
                }
                if let Some(sync) = sync {
                    if sync.sends_position() {
                        entity.last_sent_pos.store(update.position);
                    }
                    if sync.sends_rotation() {
                        entity.last_sent_yaw.store(yaw, Relaxed);
                        entity.last_sent_pitch.store(pitch, Relaxed);
                    }
                }
                self.was_riding.store(false, Relaxed);
            }
        }

        // Vanilla `ServerEntity.sendChanges`: after the needsSync block, `syncVelocity`
        // (`hurtMarked`) sends motion to tracking players (and self, for players).
        if entity.sync_velocity.swap(false, Ordering::SeqCst) {
            self.send_motion(&update);
        }

        // Java `CHeadRot` is per-tick. Bedrock watchers of this entity get head yaw from
        // `send_bedrock_move` (`MoveActorDelta` HAS_HEAD_YAW), not from this packet.
        let head_yaw = pack_degrees(entity.head_yaw.load());
        if head_yaw != entity.last_sent_head_yaw.load(Relaxed) {
            self.send_java_update(&CHeadRot::new(VarInt(self.entity_id), head_yaw), &update);
            if let Some(_motion_guard) = update.lock_if_current() {
                entity.last_sent_head_yaw.store(head_yaw, Relaxed);
            }
        }
    }

    fn recover_partial_move(&self, update: &TrackingUpdate<'_>, sync: Option<MoveSync>) {
        if sync.is_some_and(MoveSync::sends_position)
            && update.entity.vehicle_motion_epoch() == update.teleport_epoch
        {
            self.teleport_delay.store(MAX_RELATIVE_MOVE_TICKS, Relaxed);
            update.entity.sync_position.store(true, Ordering::SeqCst);
        }
    }

    fn pick_move(
        &self,
        update: &TrackingUpdate<'_>,
        tick: u32,
        rot_changed: bool,
        on_ground: bool,
        teleport_delay: u32,
    ) -> Option<MoveSync> {
        let pos = update.position;
        let base = update.entity.last_sent_pos.load();
        let send_pos = (pos - base).length_squared() >= f64::from(7.629_394_5e-6f32)
            || tick.is_multiple_of(60);
        let delta = match (
            i16::try_from(encode_pos(pos.x) - encode_pos(base.x)),
            i16::try_from(encode_pos(pos.y) - encode_pos(base.y)),
            i16::try_from(encode_pos(pos.z) - encode_pos(base.z)),
        ) {
            (Ok(x), Ok(y), Ok(z)) => Some(Vector3::new(x, y, z)),
            _ => None,
        };
        let is_arrow = [
            &EntityType::ARROW,
            &EntityType::SPECTRAL_ARROW,
            &EntityType::TRIDENT,
        ]
        .contains(&update.entity.entity_type);

        match delta {
            Some(delta)
                if teleport_delay <= MAX_RELATIVE_MOVE_TICKS
                    && !self.was_riding.load(Relaxed)
                    && self.was_on_ground.load(Relaxed) == on_ground =>
            {
                if (send_pos && rot_changed) || is_arrow {
                    Some(MoveSync::PosRot(delta))
                } else if send_pos {
                    Some(MoveSync::Pos(delta))
                } else if rot_changed {
                    Some(MoveSync::Rot)
                } else {
                    None
                }
            }
            _ => Some(MoveSync::Absolute),
        }
    }

    fn send_motion(&self, update: &TrackingUpdate<'_>) {
        let entity = update.entity;
        let velocity = entity.velocity.load();
        if entity.velocity_changed_since_sent() {
            self.send_java_update(
                &CEntityVelocity::new(VarInt(self.entity_id), velocity),
                update,
            );
            self.send_bedrock_update(
                &CSetActorMotion {
                    target_runtime_id: VarULong(self.entity_id as u64),
                    motion: velocity.to_f32_lossy(),
                    tick: VarULong(0),
                },
                update,
            );
            if let Some(_motion_guard) = update.lock_if_current() {
                entity.last_sent_velocity.store(velocity);
            }
        }
    }

    /// Java only -> Bedrock uses [`Self::send_bedrock_move`].
    fn send_move(
        &self,
        update: &TrackingUpdate<'_>,
        sync: MoveSync,
        yaw: u8,
        pitch: u8,
        on_ground: bool,
    ) {
        let id = VarInt(self.entity_id);
        match sync {
            MoveSync::Pos(delta) => {
                self.send_java_update(&CUpdateEntityPos::new(id, delta, on_ground), update);
            }
            MoveSync::Rot => {
                self.send_java_update(&CUpdateEntityRot::new(id, yaw, pitch, on_ground), update);
            }
            MoveSync::PosRot(delta) => self.send_java_update(
                &CUpdateEntityPosRot::new(id, delta, yaw, pitch, on_ground),
                update,
            ),
            MoveSync::Absolute => self.send_java_update(
                &CEntityPositionSync::new(
                    id,
                    update.position,
                    update.entity.velocity.load(),
                    update.entity.yaw.load(),
                    update.entity.pitch.load(),
                    on_ground,
                ),
                update,
            ),
        }
    }

    /// Bedrock -> every tick, only changed fields.
    fn send_bedrock_move(&self, update: &TrackingUpdate<'_>) {
        let entity = update.entity;
        let pos = self.entity.bedrock_pos();
        let rot = [
            pack_degrees(entity.pitch.load()),
            pack_degrees(entity.yaw.load()),
            pack_degrees(entity.head_yaw.load()),
        ];
        let last_pos = self.bedrock_pos.load();
        let last_rot = self.bedrock_rot.load();

        let mut flags = 0;
        if !self.entity.is_passenger() {
            for (now, last, flag) in [
                (pos.x, last_pos.x, MOVE_ACTOR_DELTA_FLAG_HAS_X),
                (pos.y, last_pos.y, MOVE_ACTOR_DELTA_FLAG_HAS_Y),
                (pos.z, last_pos.z, MOVE_ACTOR_DELTA_FLAG_HAS_Z),
            ] {
                if (now as f32).to_bits() != (last as f32).to_bits() {
                    flags |= flag;
                }
            }
        }
        for (i, flag) in [
            MOVE_ACTOR_DELTA_FLAG_HAS_PITCH,
            MOVE_ACTOR_DELTA_FLAG_HAS_YAW,
            MOVE_ACTOR_DELTA_FLAG_HAS_HEAD_YAW,
        ]
        .into_iter()
        .enumerate()
        {
            if rot[i] != last_rot[i] {
                flags |= flag;
            }
        }
        if flags == 0 {
            return;
        }

        if entity.on_ground.load(Relaxed) {
            flags |= MOVE_ACTOR_DELTA_FLAG_ON_GROUND;
        }
        self.send_bedrock_update(
            &CMoveActorDelta::new(
                VarULong(self.entity_id as u64),
                flags,
                pos.x as f32,
                pos.y as f32,
                pos.z as f32,
                rot[0],
                rot[1],
                rot[2],
            ),
            update,
        );
        if let Some(_motion_guard) = update.lock_if_current() {
            if flags
                & (MOVE_ACTOR_DELTA_FLAG_HAS_X
                    | MOVE_ACTOR_DELTA_FLAG_HAS_Y
                    | MOVE_ACTOR_DELTA_FLAG_HAS_Z)
                != 0
            {
                self.bedrock_pos.store(pos);
            }
            self.bedrock_rot.store(rot);
        }
    }

    fn collect_indirect_passengers(
        entity: &Arc<dyn EntityBase>,
        result: &mut Vec<Arc<dyn EntityBase>>,
    ) {
        if let Ok(passengers) = entity.get_entity().passengers.try_lock() {
            for passenger in passengers.iter() {
                result.push(passenger.clone());
                Self::collect_indirect_passengers(passenger, result);
            }
        }
    }

    #[must_use]
    pub fn get_effective_range(&self) -> u32 {
        let mut effective_range = self.tracking_range;
        let mut passengers = Vec::new();
        Self::collect_indirect_passengers(&self.entity, &mut passengers);
        for passenger in passengers {
            let passenger_range = passenger.get_entity().entity_type.client_tracking_range;
            if passenger_range > effective_range {
                effective_range = passenger_range;
            }
        }
        effective_range
    }

    fn broadcast_to_player(&self, player: &Player) -> bool {
        self.entity.get_player().is_none_or(|target_player| {
            player.gamemode.load() == GameMode::Spectator
                || target_player.gamemode.load() != GameMode::Spectator
        })
    }

    pub fn update_player(&self, player: &Arc<Player>, _world: &World) {
        if player.get_entity().entity_id == self.entity_id {
            return;
        }
        let entity_chunk = self.entity.get_entity().chunk_pos.load();
        if !player
            .watched_section
            .load()
            .is_within_distance(entity_chunk.x, entity_chunk.y)
        {
            self.remove_player(player);
            return;
        }
        let player_pos = player.get_entity().pos.load();
        let player_vd = get_view_distance(player).get() as i32;
        let entity_pos = self.entity.get_entity().pos.load();
        self.update_player_in_view(
            player,
            player_pos,
            player_vd,
            entity_chunk,
            entity_pos,
            self.get_effective_range(),
            true,
        );
    }

    #[expect(clippy::too_many_arguments)]
    fn update_player_in_view(
        &self,
        player: &Arc<Player>,
        player_pos: Vector3<f64>,
        player_vd: i32,
        entity_chunk: Vector2<i32>,
        entity_pos: Vector3<f64>,
        effective_range: u32,
        chunk_changed: bool,
    ) {
        if self.entity.get_entity().is_removed() {
            return;
        }
        let dx = player_pos.x - entity_pos.x;
        let dz = player_pos.z - entity_pos.z;
        let dist_sq = dx.mul_add(dx, dz * dz);

        let visible_range_blocks = f64::from((effective_range as i32).min(player_vd) * 16);
        let range_sq = visible_range_blocks * visible_range_blocks;

        if dist_sq <= range_sq && self.broadcast_to_player(player) {
            if !chunk_changed
                && self
                    .seen_by
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .contains_key(&player.gameprofile.id)
            {
                return;
            }
            // Vanilla `isChunkTracked`: never spawn before the chunk packet.
            if player
                .chunk_sender
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_chunk_ready(&entity_chunk)
            {
                if chunk_changed
                    && self
                        .seen_by
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .contains_key(&player.gameprofile.id)
                {
                    return;
                }
                if let Some(session) = self.insert_viewer(player) {
                    self.send_pairing(player, session);
                }
                return;
            }
        }
        self.remove_player(player);
    }

    pub fn update_players(&self, players: &[Arc<Player>], _world: &World) {
        if players.is_empty() {
            return;
        }
        let entity_chunk = self.entity.get_entity().chunk_pos.load();
        let mut lazy_state: Option<(Vector3<f64>, u32)> = None;
        for player in players {
            let player_entity = player.get_entity();
            if player_entity.entity_id == self.entity_id {
                continue;
            }
            if !player
                .watched_section
                .load()
                .is_within_distance(entity_chunk.x, entity_chunk.y)
            {
                self.remove_player(player);
                continue;
            }
            let (entity_pos, effective_range) = *lazy_state.get_or_insert_with(|| {
                (
                    self.entity.get_entity().pos.load(),
                    self.get_effective_range(),
                )
            });
            let player_pos = player_entity.pos.load();
            let player_vd = get_view_distance(player).get() as i32;
            self.update_player_in_view(
                player,
                player_pos,
                player_vd,
                entity_chunk,
                entity_pos,
                effective_range,
                true,
            );
        }
    }

    fn update_players_from_snapshots(
        &self,
        snapshots: &[PlayerViewSnapshot<'_>],
        chunk_changed: bool,
    ) {
        if snapshots.is_empty() {
            return;
        }
        let entity_chunk = self.entity.get_entity().chunk_pos.load();
        let mut lazy_state: Option<(Vector3<f64>, u32)> = None;
        for snap in snapshots {
            if snap.entity_id == self.entity_id {
                continue;
            }
            if !snap
                .watched
                .is_within_distance(entity_chunk.x, entity_chunk.y)
            {
                self.remove_player(snap.player);
                continue;
            }
            let (entity_pos, effective_range) = *lazy_state.get_or_insert_with(|| {
                (
                    self.entity.get_entity().pos.load(),
                    self.get_effective_range(),
                )
            });
            self.update_player_in_view(
                snap.player,
                snap.pos,
                snap.view_distance,
                entity_chunk,
                entity_pos,
                effective_range,
                chunk_changed,
            );
        }
    }

    pub fn add_pairing(&self, player: &Arc<Player>) {
        if let Some(session) = self.insert_viewer(player) {
            self.send_pairing(player, session);
        }
    }

    fn insert_viewer(&self, player: &Arc<Player>) -> Option<EquipmentSession> {
        use std::collections::hash_map::Entry;

        let viewer = player.gameprofile.id;
        // Insertion must be ordered with a despawn validated after its observers.
        let _pairing = self.lock_pairing_transitions();
        let mut viewers = self
            .seen_by
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.retired.load(Relaxed) || self.entity.get_entity().is_removed() {
            return None;
        }
        let Entry::Vacant(entry) = viewers.entry(viewer) else {
            return None;
        };
        let generation = self.next_equipment_pairing.fetch_add(1, Relaxed);
        entry.insert(TrackingViewer {
            player: player.clone(),
            generation,
        });
        if player.client.bedrock().is_some() {
            self.bedrock_watchers.fetch_add(1, Relaxed);
        }
        if self.entity.get_living_entity().is_some() {
            self.lock_equipment_pairings().insert(
                viewer,
                EquipmentPairingState {
                    viewer: player.clone(),
                    generation,
                    java_items: FxHashMap::default(),
                    java_spawn_overrides: FxHashSet::default(),
                    bedrock_items: FxHashMap::default(),
                    pending: Vec::new(),
                    needs_snapshot: false,
                    retry_in_flight: false,
                    spawned: false,
                },
            );
        }
        Some(EquipmentSession { viewer, generation })
    }

    fn lock_pairing_session(&self, session: EquipmentSession) -> Option<impl Sized + '_> {
        let viewers = self
            .seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (!self.retired.load(Relaxed)
            && !self.entity.get_entity().is_removed()
            && viewers
                .get(&session.viewer)
                .is_some_and(|viewer| viewer.generation == session.generation))
        .then_some(viewers)
    }

    pub(crate) fn java_pairing_actor(
        mut payload: &[u8],
        version: pumpkin_util::version::JavaMinecraftVersion,
    ) -> Option<(i32, i32)> {
        use pumpkin_protocol::java::client::play::{
            CRemoveMobEffect, CSetEntityLink, CSpawnEntity, CUpdateMobEffect,
        };
        use pumpkin_util::version::JavaMinecraftVersion;

        let id = payload.get_var_int().ok()?.0;
        let actor = if id == CSetEntityLink::to_id(version)
            || (version <= JavaMinecraftVersion::V_1_7_6
                && [
                    CSetEntityMetadata::to_id(version),
                    CHeadRot::to_id(version),
                    CEntityVelocity::to_id(version),
                    CUpdateMobEffect::to_id(version),
                    CRemoveMobEffect::to_id(version),
                    CSetEquipment::to_id(version),
                ]
                .contains(&id))
        {
            payload.get_i32_be().ok()?
        } else if [
            CSpawnEntity::to_id(version),
            CSetEntityMetadata::to_id(version),
            CHeadRot::to_id(version),
            CEntityVelocity::to_id(version),
            CUpdateMobEffect::to_id(version),
            CRemoveMobEffect::to_id(version),
            CSetEquipment::to_id(version),
        ]
        .contains(&id)
        {
            payload.get_var_int().ok()?.0
        } else {
            return None;
        };
        Some((id, actor))
    }

    fn java_pairing_link(
        payload: &[u8],
        version: pumpkin_util::version::JavaMinecraftVersion,
        actor: i32,
    ) -> bool {
        Entity::java_passenger_update_overlaps(payload, version, actor, &[VarInt(actor)])
    }

    fn java_relationship_is_current(
        &self,
        mut payload: &[u8],
        protocol: pumpkin_util::version::JavaMinecraftVersion,
    ) -> bool {
        use pumpkin_protocol::java::client::play::CSetPassengers;

        if !payload
            .get_var_int()
            .is_ok_and(|id| id.0 == CSetPassengers::to_id(protocol))
        {
            return true;
        }
        let Ok(carrier) = payload.get_var_int() else {
            return false;
        };
        let Ok(count) = payload.get_var_int() else {
            return false;
        };
        let ids: Vec<_> = (0..count.0)
            .map_while(|_| payload.get_var_int().ok())
            .collect();
        let includes_actor =
            carrier.0 == self.entity_id || ids.iter().any(|id| id.0 == self.entity_id);
        let world = self.entity.get_entity().world.load();
        let Some(entity) = world.get_entity_by_id(carrier.0) else {
            return !includes_actor;
        };
        let Ok(current) = entity.get_entity().passengers.try_lock() else {
            return true;
        };
        let carries_actor = current
            .iter()
            .any(|passenger| passenger.get_entity().entity_id == self.entity_id);
        (!includes_actor && !carries_actor)
            || current
                .iter()
                .map(|passenger| VarInt(passenger.get_entity().entity_id))
                .eq(ids)
    }

    fn bedrock_relationship_is_current(&self, mut payload: &[u8]) -> bool {
        use pumpkin_protocol::bedrock::client::CSetActorLink;

        if !VarUInt::read(&mut payload).is_ok_and(|id| id.0 == CSetActorLink::PACKET_ID as u32) {
            return true;
        }
        let Ok(carrier) = VarLong::read(&mut payload) else {
            return false;
        };
        let Ok(rider) = VarLong::read(&mut payload) else {
            return false;
        };
        let actor = i64::from(self.entity_id);
        if carrier.0 != actor && rider.0 != actor {
            return true;
        }
        let Ok(expected) = u8::read(&mut payload) else {
            return false;
        };
        let world = self.entity.get_entity().world.load();
        let Some(entity) = i32::try_from(carrier.0)
            .ok()
            .and_then(|id| world.get_entity_by_id(id))
        else {
            return expected == 0;
        };
        let Ok(current) = entity.get_entity().passengers.try_lock() else {
            return true;
        };
        current
            .iter()
            .position(|passenger| i64::from(passenger.get_entity().entity_id) == rider.0)
            .map_or(0, |index| if index == 0 { 1 } else { 2 })
            == expected
    }

    fn send_java_pairing_packet<T>(
        &self,
        client: &JavaClient,
        session: EquipmentSession,
        data: Bytes,
        lock_if_current: impl FnOnce() -> Option<T>,
    ) {
        use pumpkin_protocol::java::client::play::CSpawnEntity;
        let version = client.version.load();
        let session_current = Cell::new(false);
        client.try_enqueue_packet_data_projecting(
            data,
            || {
                let viewing = self.lock_pairing_session(session)?;
                session_current.set(true);
                Some((viewing, lock_if_current()?))
            },
            |data, primary, guard| {
                if !primary
                    && session_current.get()
                    && !self.java_relationship_is_current(&data, version)
                {
                    return None;
                }
                if guard.is_some() {
                    return Some(data);
                }
                if primary {
                    return None;
                }
                let stale = !session_current.get()
                    && Self::java_pairing_actor(&data, version)
                        .is_some_and(|(_, actor)| actor == self.entity_id);
                if stale
                    || (!session_current.get()
                        && Self::java_pairing_link(&data, version, self.entity_id))
                {
                    return None;
                }
                if session_current.get() {
                    Some(data)
                } else {
                    Self::without_java_removal(data, version, self.entity_id)
                }
            },
            |data, _, guard| {
                if guard.is_none() {
                    return;
                }
                let spawn = Self::java_pairing_actor(data, version)
                    == Some((CSpawnEntity::to_id(version), self.entity_id));
                let mut pairings = self.lock_equipment_pairings();
                let Some(state) = Self::current_equipment_pairing(&mut pairings, session) else {
                    return;
                };
                if spawn {
                    state.spawned = true;
                } else if state.spawned
                    && let Some(packet) =
                        Self::java_equipment_from_packet(data, version, self.entity_id)
                {
                    state
                        .java_spawn_overrides
                        .extend(packet.equipment.iter().map(|(id, _)| *id));
                }
            },
        );
    }

    fn send_bedrock_pairing_packet<T>(
        &self,
        client: &BedrockClient,
        session: EquipmentSession,
        data: Bytes,
        lock_if_current: impl FnOnce() -> Option<T>,
    ) {
        let session_current = Cell::new(false);
        client.try_enqueue_packet_data_projecting(
            data,
            || {
                let viewing = self.lock_pairing_session(session)?;
                session_current.set(true);
                Some((viewing, lock_if_current()?))
            },
            |data, primary, guard| {
                if !primary && session_current.get() && !self.bedrock_relationship_is_current(&data)
                {
                    return None;
                }
                if guard.is_some() {
                    return Some(data);
                }
                if primary {
                    return None;
                }
                (!self.is_bedrock_pairing_actor(&data)).then_some(data)
            },
            |_, _, _| {},
        );
    }

    #[allow(clippy::too_many_lines)]
    fn send_pairing(&self, player: &Arc<Player>, session: EquipmentSession) {
        let _pairing = EquipmentPairing {
            tracker: self,
            session,
        };
        let still_paired = || self.lock_pairing_session(session).is_some();
        match player.client.as_ref() {
            ClientPlatform::Java(client) => {
                self.entity.send_java_spawn_packets(client, &mut |data| {
                    if still_paired() {
                        self.send_java_pairing_packet(client, session, data, || Some(()));
                    }
                });
            }
            ClientPlatform::Bedrock(client) => {
                self.entity
                    .send_bedrock_spawn_packets(client, true, &mut |data| {
                        if still_paired() {
                            self.send_bedrock_pairing_packet(client, session, data, || Some(()));
                        }
                    });
            }
            ClientPlatform::SimulatedPlayer(_) => {}
        }
        if !still_paired() {
            return;
        }
        if player.client.bedrock().is_some()
            && let Some(state) =
                Self::current_equipment_pairing(&mut self.lock_equipment_pairings(), session)
        {
            state.spawned = true;
        }
        player.try_restore_vehicle(&self.entity);

        if let Some(target_player) = self.entity.get_player() {
            let skin_parts = target_player.config.load().skin_parts;
            let target_entity = target_player.get_entity();
            let target_id = target_entity.entity_id;

            if let ClientPlatform::Java(client) = player.client.as_ref() {
                let version = CURRENT_MC_VERSION;
                let mut buf = Vec::new();
                for meta in [
                    Metadata::new(
                        pumpkin_data::tracked_data::player::PLAYER_MODE_CUSTOMISATION,
                        skin_parts,
                    ),
                    Metadata::new(
                        pumpkin_data::tracked_data::player::PLAYER_MODE_CUSTOMIZATION_ID,
                        skin_parts,
                    ),
                ] {
                    let _ = meta.write(&mut buf, &version);
                }
                buf.put_u8(255);
                let meta_packet = CSetEntityMetadata::new(target_id.into(), buf.into());
                if let Ok(packet_data) =
                    JavaClient::serialize_packet_for_version(&meta_packet, version)
                {
                    self.send_java_pairing_packet(client, session, packet_data, || Some(()));
                }

                let head_yaw = target_entity.head_yaw.load();
                // TODO: use `pumpkin_util::math::pack_degrees`.
                let head_rot_packet = CHeadRot::new(
                    target_id.into(),
                    (head_yaw * 256.0 / 360.0).rem_euclid(256.0) as u8,
                );
                if let Ok(data) = client.serialize_packet(&head_rot_packet) {
                    self.send_java_pairing_packet(client, session, data, || Some(()));
                }
            }
        } else if self.entity.get_living_entity().is_some()
            && let ClientPlatform::Java(client) = player.client.as_ref()
        {
            let head_yaw = self.entity.get_entity().head_yaw.load();
            // TODO: use `pumpkin_util::math::pack_degrees`.
            let head_rot_packet = CHeadRot::new(
                self.entity_id.into(),
                (head_yaw * 256.0 / 360.0).rem_euclid(256.0) as u8,
            );
            if let Ok(data) = client.serialize_packet(&head_rot_packet) {
                self.send_java_pairing_packet(client, session, data, || Some(()));
            }
        }

        if !still_paired() {
            return;
        }
        let vel = self.entity.get_entity().velocity.load();
        if vel.length_squared() > 1e-4 {
            match player.client.as_ref() {
                ClientPlatform::Java(client) => {
                    let motion = CEntityVelocity::new(self.entity_id.into(), vel);
                    if let Ok(data) = client.serialize_packet(&motion) {
                        self.send_java_pairing_packet(client, session, data, || Some(()));
                    }
                }
                ClientPlatform::Bedrock(client) => {
                    let motion =
                        pumpkin_protocol::bedrock::client::set_actor_motion::CSetActorMotion {
                            target_runtime_id: (self.entity_id as u64).into(),
                            motion: vel.to_f32_lossy(),
                            tick: 0u64.into(),
                        };
                    if let Ok(data) = client.serialize_packet(&motion) {
                        self.send_bedrock_pairing_packet(client, session, data, || Some(()));
                    }
                }
                ClientPlatform::SimulatedPlayer(_) => {}
            }
        }

        if let ClientPlatform::Java(client) = player.client.as_ref() {
            let version = CURRENT_MC_VERSION;
            if let Some(non_default) = self
                .entity
                .get_entity()
                .synched_data
                .get_non_default_values_for_version(&version)
            {
                let packet = CSetEntityMetadata::new(self.entity_id.into(), non_default);
                if let Ok(packet_data) = JavaClient::serialize_packet_for_version(&packet, version)
                {
                    self.send_java_pairing_packet(client, session, packet_data, || Some(()));
                }
            }
        }

        if let Some(living) = self.entity.get_living_entity() {
            self.send_pairing_equipment(living, player, session);

            self.enumerate_pairing_effects(living, player, session);
        }

        if !still_paired() {
            return;
        }
        self.send_pairing_passengers(&self.entity, player, session, None);
        let vehicle = self
            .entity
            .get_entity()
            .vehicle
            .try_lock()
            .ok()
            .and_then(|vehicle| vehicle.as_ref().and_then(Weak::upgrade));
        if let Some(vehicle) = vehicle {
            self.send_pairing_passengers(&vehicle, player, session, Some(self.entity_id));
        }
    }

    fn filter_invalid_java_pairing_extra(&self, client: &JavaClient, data: Bytes) -> Option<Bytes> {
        let version = client.version.load();
        (Self::java_pairing_actor(&data, version).is_none_or(|(_, actor)| actor != self.entity_id)
            && !Self::java_pairing_link(&data, version, self.entity_id))
        .then_some(data)
        .and_then(|data| Self::without_java_removal(data, version, self.entity_id))
    }

    fn enumerate_pairing_effects(
        &self,
        living: &LivingEntity,
        player: &Arc<Player>,
        session: EquipmentSession,
    ) {
        let effect_types = match living
            .active_effects
            .try_lock()
            .map(|effects| effects.keys().copied().collect::<Vec<_>>())
        {
            Ok(effect_types) => effect_types,
            Err(std::sync::TryLockError::WouldBlock) => {
                self.defer_pairing_effect(player, session, None);
                return;
            }
            Err(std::sync::TryLockError::Poisoned(_)) => return,
        };
        for effect_type in effect_types {
            self.send_pairing_effect(living, player, session, effect_type);
        }
    }

    fn send_pairing_effect(
        &self,
        living: &LivingEntity,
        player: &Arc<Player>,
        session: EquipmentSession,
        effect_type: &'static pumpkin_data::effect::StatusEffect,
    ) {
        for _ in 0..2 {
            let Some(effect) = living
                .active_effects
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(effect_type)
                .filter(|effect| effect.duration != 0)
                .cloned()
            else {
                return;
            };
            let retry = std::cell::Cell::new(false);
            let validate = || {
                let pairing = self.lock_pairing_session(session)?;
                let effects = living
                    .active_effects
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let current = effects.get(effect_type)?;
                let matches = current.duration == effect.duration
                    && current.amplifier == effect.amplifier
                    && current.ambient == effect.ambient
                    && current.show_particles == effect.show_particles
                    && current.show_icon == effect.show_icon;
                retry.set(!matches && current.duration != 0);
                matches.then_some((pairing, effects))
            };
            match player.client.as_ref() {
                ClientPlatform::Java(client) => {
                    let mut flags: i8 = 0;
                    if effect.ambient {
                        flags |= 0x01;
                    }
                    if effect.show_particles {
                        flags |= 0x02;
                    }
                    if effect.show_icon {
                        flags |= 0x04;
                    }
                    let pkt = pumpkin_protocol::java::client::play::CUpdateMobEffect::new(
                        self.entity_id.into(),
                        i32::from(effect.effect_type.id).into(),
                        i32::from(effect.amplifier).into(),
                        effect.duration.into(),
                        flags,
                    );
                    if let Ok(data) = client.serialize_packet(&pkt) {
                        client.try_enqueue_packet_data_filtering(data, validate, |data| {
                            self.filter_invalid_java_pairing_extra(client, data)
                        });
                    }
                }
                ClientPlatform::Bedrock(client) => {
                    let pkt = pumpkin_protocol::bedrock::client::CMobEffect {
                        target_runtime_id: (self.entity_id as u64).into(),
                        event_id: pumpkin_protocol::bedrock::client::CMobEffect::EVENT_ADD,
                        effect_id: effect.effect_type.to_bedrock_id().into(),
                        effect_amplifier: i32::from(effect.amplifier).into(),
                        show_particles: effect.show_particles,
                        effect_duration_ticks: effect.duration.into(),
                        tick: 0u64.into(),
                        ambient: effect.ambient,
                    };
                    if let Ok(data) = client.serialize_packet(&pkt) {
                        client.try_enqueue_packet_data_filtering(data, validate, |data| {
                            (!self.is_bedrock_pairing_actor(&data)).then_some(data)
                        });
                    }
                }
                ClientPlatform::SimulatedPlayer(_) => {}
            }
            if !retry.get() {
                return;
            }
        }
        self.defer_pairing_effect(player, session, Some(effect_type));
    }

    fn defer_pairing_effect(
        &self,
        player: &Arc<Player>,
        session: EquipmentSession,
        effect_type: Option<&'static pumpkin_data::effect::StatusEffect>,
    ) {
        if let Some(_pairing) = self.lock_pairing_session(session) {
            self.effect_retries
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(
                    (session.viewer, effect_type),
                    (Arc::downgrade(player), session),
                );
            self.effect_retry_pending.store(true, Relaxed);
        }
    }

    fn flush_pairing_effects(&self) {
        if !self.effect_retry_pending.swap(false, Relaxed) {
            return;
        }
        let pending = std::mem::take(
            &mut *self
                .effect_retries
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        if let Some(living) = self.entity.get_living_entity() {
            for ((_, effect_type), (viewer, session)) in pending {
                if let Some(viewer) = viewer.upgrade()
                    && self.lock_pairing_session(session).is_some()
                {
                    if let Some(effect_type) = effect_type {
                        self.send_pairing_effect(living, &viewer, session, effect_type);
                    } else {
                        self.enumerate_pairing_effects(living, &viewer, session);
                    }
                }
            }
        }
    }

    fn send_pairing_passengers(
        &self,
        carrier: &Arc<dyn EntityBase>,
        player: &Arc<Player>,
        session: EquipmentSession,
        paired_rider: Option<i32>,
    ) {
        let bedrock = player.client.bedrock().is_some();
        let mut prior_viewers = None;
        let mut retried_change = false;
        loop {
            let outcome = Cell::new(PassengerSnapshot::Finished);
            let viewers = self.send_pairing_passenger_projection(
                carrier.get_entity(),
                player,
                session,
                paired_rider,
                prior_viewers.as_deref(),
                &outcome,
            );
            match outcome.get() {
                PassengerSnapshot::Busy => break,
                PassengerSnapshot::Changed if bedrock && !retried_change => {
                    retried_change = true;
                }
                PassengerSnapshot::Changed if bedrock => break,
                PassengerSnapshot::Finished | PassengerSnapshot::Changed => {
                    let Some(viewers) = viewers else { return };
                    prior_viewers = Some(viewers);
                }
            }
        }
        if let Some(_pairing) = self.lock_pairing_session(session) {
            self.passenger_retries
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(
                    (session.viewer, paired_rider.is_some()),
                    PassengerPairingRetry {
                        viewer: Arc::downgrade(player),
                        carrier: Arc::downgrade(carrier),
                        session,
                        paired_rider,
                    },
                );
            self.passenger_retry_pending.store(true, Relaxed);
        }
    }

    fn flush_pairing_passengers(&self) {
        if !self.passenger_retry_pending.swap(false, Relaxed) {
            return;
        }
        let pending = std::mem::take(
            &mut *self
                .passenger_retries
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for retry in pending.into_values() {
            if let (Some(carrier), Some(viewer)) = (retry.carrier.upgrade(), retry.viewer.upgrade())
            {
                self.send_pairing_passengers(&carrier, &viewer, retry.session, retry.paired_rider);
            }
        }
    }

    fn send_pairing_passenger_projection(
        &self,
        carrier: &Entity,
        player: &Player,
        session: EquipmentSession,
        paired_rider: Option<i32>,
        prior_viewers: Option<&[(Arc<Self>, EquipmentSession)]>,
        outcome: &Cell<PassengerSnapshot>,
    ) -> Option<Vec<(Arc<Self>, EquipmentSession)>> {
        if carrier.is_removed() || self.lock_pairing_session(session).is_none() {
            return None;
        }
        // Packet observers can change either link while pairing this carrier or its rider.
        let passengers = Self::try_lock_passenger_snapshot(carrier, outcome)?.clone();
        let ids: Vec<VarInt> = passengers
            .iter()
            .map(|p| VarInt(p.get_entity().entity_id))
            .collect();
        if ids.is_empty() || paired_rider.is_some_and(|rider_id| !ids.contains(&VarInt(rider_id))) {
            return None;
        }
        let world = carrier.world.load();
        let mut counterpart_viewers = Vec::new();
        let mut visible = |id| {
            if id == self.entity_id || id == player.entity_id() {
                return true;
            }
            let Some(tracked) = world.entity_tracker.get_tracked_entity(id) else {
                return false;
            };
            let Some(session) = tracked.viewer_session(player) else {
                return false;
            };
            counterpart_viewers.push((tracked, session));
            true
        };
        if !visible(carrier.entity_id) {
            return None;
        }
        let mut unseen = Vec::new();
        let mut visible_ids = Vec::new();
        for (passenger, id) in passengers.iter().zip(&ids) {
            if visible(id.0) {
                visible_ids.push(*id);
            } else {
                unseen.push(passenger.get_entity());
            }
        }
        unseen.sort_unstable_by_key(|entity| entity.entity_id);
        unseen.dedup_by_key(|entity| entity.entity_id);
        if visible_ids.is_empty() {
            return None;
        }

        // Only retry a strictly smaller set of the original tracker generations.
        // A newly paired rider already publishes a newer projection.
        if prior_viewers
            .is_some_and(|prior| !Self::passenger_viewers_shrank(prior, &counterpart_viewers))
        {
            return None;
        }
        let mut endpoints: Vec<_> = counterpart_viewers
            .iter()
            .map(|(tracked, session)| (tracked.as_ref(), *session))
            .chain(std::iter::once((self, session)))
            .collect();
        // Pairing snapshots can share a carrier or riders on different tracking threads.
        endpoints.sort_unstable_by_key(|(tracked, _)| tracked.entity_id);
        endpoints.dedup_by_key(|(tracked, _)| tracked.entity_id);
        let links =
            Self::pairing_passenger_links(carrier, player, &ids, &visible_ids, paired_rider);
        let visibility_shrank = Cell::new(false);
        carrier.send_passenger_update_to_if(
            &world,
            player,
            &ids,
            &visible_ids,
            &links,
            |link_rider| {
                let unseen = if link_rider.is_none() {
                    Some(Self::lock_unseen_passengers(&world, player, &unseen)?)
                } else {
                    None
                };
                let viewers = self.lock_passenger_viewers(
                    &world,
                    player,
                    carrier.entity_id,
                    &endpoints,
                    link_rider,
                    &visibility_shrank,
                )?;
                if carrier.is_removed() {
                    return None;
                }
                let current = Self::try_lock_passenger_snapshot(carrier, outcome)?;
                if link_rider.is_some()
                    && !current
                        .iter()
                        .map(|rider| VarInt(rider.get_entity().entity_id))
                        .eq(ids.iter().copied())
                {
                    outcome.set(PassengerSnapshot::Changed);
                    return None;
                }
                Some(((unseen, viewers), current))
            },
            |data| self.keep_stale_passenger_extra(player, session, data),
        );
        // A despawn has no paired publication to repair the surviving Java riders.
        visibility_shrank.get().then_some(counterpart_viewers)
    }

    fn try_lock_passenger_snapshot<'a>(
        carrier: &'a Entity,
        outcome: &Cell<PassengerSnapshot>,
    ) -> Option<MutexGuard<'a, Vec<Arc<dyn EntityBase>>>> {
        match carrier.passengers.try_lock() {
            Ok(passengers) => Some(passengers),
            Err(std::sync::TryLockError::WouldBlock) => {
                outcome.set(PassengerSnapshot::Busy);
                None
            }
            Err(std::sync::TryLockError::Poisoned(_)) => None,
        }
    }

    fn keep_stale_passenger_extra(
        &self,
        player: &Player,
        session: EquipmentSession,
        data: &[u8],
    ) -> bool {
        self.lock_pairing_session(session).is_some()
            || player.client.java().map_or_else(
                || !self.is_bedrock_pairing_actor(data),
                |client| {
                    let version = client.version.load();
                    !Self::java_pairing_link(data, version, self.entity_id)
                        && Self::java_pairing_actor(data, version)
                            .is_none_or(|(_, actor)| actor != self.entity_id)
                },
            )
    }

    fn passenger_viewers_shrank(
        prior_viewers: &[(Arc<Self>, EquipmentSession)],
        current_viewers: &[(Arc<Self>, EquipmentSession)],
    ) -> bool {
        current_viewers.len() < prior_viewers.len()
            && current_viewers.iter().all(|(tracked, session)| {
                prior_viewers
                    .iter()
                    .any(|(prior, generation)| Arc::ptr_eq(tracked, prior) && session == generation)
            })
    }

    fn pairing_passenger_links(
        carrier: &Entity,
        player: &Player,
        ids: &[VarInt],
        visible_ids: &[VarInt],
        paired_rider: Option<i32>,
    ) -> Vec<Option<pumpkin_protocol::bedrock::client::CSetActorLink>> {
        if player.client.bedrock().is_none() {
            return Vec::new();
        }
        ids.iter()
            .enumerate()
            .filter(|(_, id)| {
                visible_ids.contains(id) && paired_rider.is_none_or(|rider_id| id.0 == rider_id)
            })
            .map(|(index, id)| {
                Some(carrier.bedrock_passenger_link(id.0, if index == 0 { 1 } else { 2 }, false))
            })
            .collect()
    }

    fn lock_passenger_viewers<'a>(
        &self,
        world: &World,
        player: &Player,
        carrier_id: i32,
        endpoints: &[(&'a Self, EquipmentSession)],
        link_rider: Option<i32>,
        visibility_shrank: &Cell<bool>,
    ) -> Option<Vec<impl Sized + 'a>> {
        let mut shrank = false;
        let mut viewers = Vec::with_capacity(endpoints.len());
        for &(tracked, session) in endpoints {
            let id = tracked.entity_id;
            let required = id == self.entity_id || id == carrier_id;
            if link_rider.is_some_and(|rider| id != rider && !required) {
                continue;
            }
            if let Some(viewing) = tracked.lock_pairing_session(session) {
                viewers.push(viewing);
            } else if required
                || link_rider.is_some()
                || world
                    .entity_tracker
                    .get_tracked_entity(id)
                    .is_some_and(|current| current.viewer_session(player).is_some())
            {
                return None;
            } else {
                shrank = true;
            }
        }
        visibility_shrank.set(shrank);
        (!shrank).then_some(viewers)
    }

    fn lock_unseen_passengers<'a>(
        world: &World,
        player: &Player,
        unseen: &[&'a Entity],
    ) -> Option<Vec<std::sync::MutexGuard<'a, ()>>> {
        // Pin absence across enqueue as well as the generations already visible.
        let transitions = unseen
            .iter()
            .map(|entity| {
                entity
                    .tracking_pairing_lock
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
            })
            .collect();
        unseen
            .iter()
            .all(|entity| {
                world
                    .entity_tracker
                    .get_tracked_entity(entity.entity_id)
                    .is_none_or(|tracked| tracked.viewer_session(player).is_none())
            })
            .then_some(transitions)
    }

    pub(crate) fn viewer_session(&self, player: &Player) -> Option<EquipmentSession> {
        let viewer = player.gameprofile.id;
        let viewers = self
            .seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tracked = viewers.get(&viewer)?;
        (!self.retired.load(Relaxed)
            && !self.entity.get_entity().is_removed()
            && std::ptr::eq(tracked.player.as_ref(), player))
        .then_some(EquipmentSession {
            viewer,
            generation: tracked.generation,
        })
    }

    fn lock_pairing_transitions(&self) -> std::sync::MutexGuard<'_, ()> {
        self.entity
            .get_entity()
            .tracking_pairing_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn remove_pairing(&self, player: &Player) {
        if !self.remove_viewer(player) {
            return;
        }
        let world = self.entity.get_entity().world.load_full();
        self.send_removal(player, || self.lock_removal_if_unpaired(&world, player));
    }

    fn lock_removal_if_unpaired(
        &self,
        world: &World,
        player: &Player,
    ) -> Option<std::sync::MutexGuard<'_, ()>> {
        let pairing = self.lock_pairing_transitions();
        let seen = world
            .entity_tracker
            .get_tracked_entity(self.entity_id)
            .is_some_and(|tracked| tracked.is_seen_by(&player.gameprofile.id));
        (!seen && !self.is_seen_by(&player.gameprofile.id)).then_some(pairing)
    }

    fn send_removal<T>(&self, player: &Player, lock_if_current: impl FnOnce() -> Option<T>) {
        match player.client.as_ref() {
            ClientPlatform::Java(client) => {
                let ids = [self.entity_id.into()];
                if let Ok(data) = client.serialize_packet(&CRemoveEntities::new(&ids)) {
                    let version = client.version.load();
                    client.try_enqueue_packet_data_filtering(data, lock_if_current, |data| {
                        Self::without_java_removal(data, version, self.entity_id)
                    });
                }
            }
            ClientPlatform::Bedrock(client) => {
                let packet = CRemoveActor::new(VarLong(i64::from(self.entity_id)));
                if let Ok(data) = client.serialize_packet(&packet) {
                    client.try_enqueue_packet_data_if(
                        data,
                        CRemoveActor::PACKET_ID,
                        |mut payload| {
                            VarLong::read(&mut payload)
                                .is_ok_and(|id| id.0 == i64::from(self.entity_id))
                        },
                        lock_if_current,
                    );
                }
            }
            ClientPlatform::SimulatedPlayer(_) => {}
        }
    }

    fn without_java_removal(
        data: Bytes,
        version: pumpkin_util::version::JavaMinecraftVersion,
        entity_id: i32,
    ) -> Option<Bytes> {
        use pumpkin_util::version::JavaMinecraftVersion;

        let mut payload = data.as_ref();
        if !payload
            .get_var_int()
            .is_ok_and(|id| id.0 == CRemoveEntities::to_id(version))
        {
            return Some(data);
        }
        if version == JavaMinecraftVersion::V_1_17 {
            return payload
                .get_var_int()
                .is_ok_and(|id| id.0 != entity_id)
                .then_some(data);
        }
        let legacy = version <= JavaMinecraftVersion::V_1_7_6;
        let count = if legacy {
            i32::from(payload.get_u8().ok()?)
        } else {
            payload.get_var_int().ok()?.0
        };
        let mut ids = Vec::new();
        for _ in 0..count {
            let id = if legacy {
                VarInt(payload.get_i32_be().ok()?)
            } else {
                payload.get_var_int().ok()?
            };
            if id.0 != entity_id {
                ids.push(id);
            }
        }
        if ids.is_empty() {
            None
        } else if ids.len() == count as usize {
            Some(data)
        } else {
            JavaClient::serialize_packet_for_version(&CRemoveEntities::new(&ids), version).ok()
        }
    }

    pub fn broadcast_removed(&self, world: &World) {
        let viewers = {
            let mut guard = self
                .seen_by
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            self.retired.store(true, Relaxed);
            self.lock_equipment_pairings().clear();
            self.bedrock_watchers.store(0, Relaxed);
            std::mem::take(&mut *guard)
        };
        for viewer in viewers.values() {
            self.send_removal(&viewer.player, || {
                self.lock_removal_if_unpaired(world, &viewer.player)
            });
        }
    }

    /// Vanilla `TrackedEntity.removePlayer`: despawn on the client, only if it was paired.
    pub fn remove_player(&self, player: &Player) {
        self.remove_pairing(player);
    }

    /// Forgets the player as a viewer without a removal packet; the client already dropped it.
    pub fn forget_player(&self, player: &Player) {
        self.remove_viewer(player);
    }

    fn remove_viewer(&self, player: &Player) -> bool {
        let mut viewers = self
            .seen_by
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Entry::Occupied(viewer) = viewers.entry(player.gameprofile.id) else {
            return false;
        };
        if !std::ptr::eq(viewer.get().player.as_ref(), player) {
            return false;
        }
        let viewer = viewer.remove();
        if viewer.player.client.bedrock().is_some() {
            self.bedrock_watchers.fetch_sub(1, Relaxed);
        }
        self.lock_equipment_pairings()
            .remove(&player.gameprofile.id);
        true
    }

    pub(crate) fn viewers_snapshot(&self) -> Vec<Arc<Player>> {
        // A packet observer may remove this entity and take the write lock.
        self.seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .map(|viewer| viewer.player.clone())
            .collect()
    }

    fn lock_viewer_if_current<'a>(
        &'a self,
        recipient: &Arc<Player>,
        update: &'a TrackingUpdate<'_>,
    ) -> Option<impl Sized + 'a> {
        let motion_guard = update.lock_if_current()?;
        let viewers = self
            .seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (std::ptr::eq(recipient.world().as_ref(), update.world)
            && viewers
                .get(&recipient.gameprofile.id)
                .is_some_and(|viewer| Arc::ptr_eq(&viewer.player, recipient)))
        .then_some((motion_guard, viewers))
    }

    fn send_java_update<P: ClientPacket>(&self, packet: &P, update: &TrackingUpdate<'_>) {
        let mut packet_data = None;
        for recipient in self.viewers_snapshot() {
            if update.lock_if_current().is_none() {
                return;
            }
            let Some(client) = recipient.client.java() else {
                continue;
            };
            let data = match &packet_data {
                Some(data) => data,
                None => match client.serialize_packet(packet) {
                    Ok(data) => packet_data.insert(data),
                    Err(WritingError::UnsupportedVersion(_)) => return,
                    Err(error) => {
                        tracing::error!(
                            "Failed to serialize packet {} for version {:?}: {}",
                            std::any::type_name::<P>(),
                            CURRENT_MC_VERSION,
                            error
                        );
                        return;
                    }
                },
            };
            client.try_enqueue_packet_data_if(
                data.clone(),
                P::to_id(client.version.load()),
                |mut payload| payload.get_var_int().is_ok_and(|id| id.0 == self.entity_id),
                || self.lock_viewer_if_current(&recipient, update),
            );
        }
    }

    fn send_bedrock_update<P: BClientPacket>(&self, packet: &P, update: &TrackingUpdate<'_>) {
        for recipient in self.bedrock_watchers_snapshot() {
            if update.lock_if_current().is_none() {
                return;
            }
            let Some(client) = recipient.client.bedrock() else {
                continue;
            };
            match client.serialize_packet(packet) {
                Ok(data) => client.try_enqueue_packet_data_if(
                    data,
                    P::PACKET_ID,
                    |mut payload| {
                        VarULong::read(&mut payload).is_ok_and(|id| id.0 == self.entity_id as u64)
                    },
                    || self.lock_viewer_if_current(&recipient, update),
                ),
                Err(error) => tracing::error!(
                    "Failed to serialize bedrock packet {}: {}",
                    std::any::type_name::<P>(),
                    error
                ),
            }
        }
    }

    fn lock_equipment_pairings(&self) -> std::sync::MutexGuard<'_, EquipmentPairings> {
        self.pairing_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn equipment_viewers(&self) -> Vec<TrackingViewer> {
        self.seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .cloned()
            .collect()
    }

    fn current_equipment_pairing(
        pairings: &mut EquipmentPairings,
        session: EquipmentSession,
    ) -> Option<&mut EquipmentPairingState> {
        pairings
            .get_mut(&session.viewer)
            .filter(|state| state.generation == session.generation)
    }

    fn try_equipment_snapshot(&self, living: &LivingEntity) -> Option<EntityEquipment> {
        let mut snapshot = match living.entity_equipment.try_lock() {
            Ok(gear) => gear.clone(),
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner().clone(),
            Err(std::sync::TryLockError::WouldBlock) => return None,
        };
        if let Some(player) = self.entity.get_player() {
            let inventory = player.inventory();
            let main = match inventory.main_inventory.try_read() {
                Ok(main) => main,
                Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
                Err(std::sync::TryLockError::WouldBlock) => return None,
            };
            let hand = main
                .get(inventory.get_selected_slot() as usize)
                .cloned()
                .unwrap_or_else(|| ItemStack::EMPTY.clone());
            snapshot.put(&EquipmentSlot::MAIN_HAND, hand);
        }
        Some(snapshot)
    }

    fn send_pairing_equipment(
        &self,
        living: &LivingEntity,
        player: &Arc<Player>,
        session: EquipmentSession,
    ) {
        let Some(snapshot) = self.try_equipment_snapshot(living) else {
            if let Some(state) =
                Self::current_equipment_pairing(&mut self.lock_equipment_pairings(), session)
            {
                state.spawned = true;
                state.needs_snapshot = true;
                self.equipment_retry_pending.store(true, Relaxed);
            }
            return;
        };
        let slots = Self::occupied_slots(&snapshot);
        self.send_equipment_pairing(living, player, session, snapshot, slots, 2);
    }

    fn occupied_slots(snapshot: &EntityEquipment) -> Vec<EquipmentSlot> {
        snapshot
            .equipment
            .iter()
            .filter(|(_, stack)| !stack.is_empty())
            .map(|(slot, _)| slot.clone())
            .collect()
    }

    fn send_equipment_pairing(
        &self,
        living: &LivingEntity,
        player: &Player,
        session: EquipmentSession,
        mut snapshot: EntityEquipment,
        mut slots: Vec<EquipmentSlot>,
        attempts: usize,
    ) {
        let _pairing = EquipmentPairing {
            tracker: self,
            session,
        };
        let mut needs_snapshot = false;
        for _ in 0..attempts {
            if slots.is_empty() || living.entity.is_removed() {
                return;
            }
            if player.client.java().is_some() {
                let mut pairings = self.lock_equipment_pairings();
                let Some(state) = Self::current_equipment_pairing(&mut pairings, session) else {
                    return;
                };
                slots.retain(|slot| {
                    !state.java_spawn_overrides.contains(&slot.discriminant())
                        && !state
                            .java_items
                            .get(&slot.discriminant())
                            .is_some_and(|sent| {
                                Self::equipment_stacks_match(
                                    snapshot.equipment.get(slot),
                                    Some(sent),
                                    false,
                                )
                            })
                });
            }
            slots = match player.client.as_ref() {
                ClientPlatform::Java(client) => self
                    .java_equipment_update(slots.into_iter().map(|slot| {
                        let stack = snapshot.get(&slot);
                        (slot, stack)
                    }))
                    .map_or_else(Vec::new, |update| {
                        self.send_java_equipment(client, living, &update, session)
                    }),
                ClientPlatform::Bedrock(client) => {
                    self.send_bedrock_pairing(client, living, &snapshot, &slots, session)
                }
                ClientPlatform::SimulatedPlayer(_) => Vec::new(),
            };
            let Some(next) = self.try_equipment_snapshot(living) else {
                // A previously sent slot may have been cleared while the snapshot was busy.
                for slot in snapshot.equipment.keys() {
                    if !slots.contains(slot) {
                        slots.push(slot.clone());
                    }
                }
                needs_snapshot = true;
                break;
            };
            let bedrock = player.client.bedrock().is_some();
            for slot in snapshot.equipment.keys().chain(next.equipment.keys()) {
                if !Self::equipment_stacks_match(
                    snapshot.equipment.get(slot),
                    next.equipment.get(slot),
                    bedrock,
                ) && !slots.contains(slot)
                {
                    slots.push(slot.clone());
                }
            }
            snapshot = next;
        }
        if (needs_snapshot || !slots.is_empty())
            && let Some(state) =
                Self::current_equipment_pairing(&mut self.lock_equipment_pairings(), session)
        {
            for slot in slots {
                if !state.pending.contains(&slot) {
                    state.pending.push(slot);
                }
            }
            state.needs_snapshot |= needs_snapshot;
            self.equipment_retry_pending.store(true, Relaxed);
        }
    }

    fn flush_pairing_equipment(&self) {
        if !self.equipment_retry_pending.load(Relaxed)
            || !self.equipment_retry_pending.swap(false, Relaxed)
        {
            return;
        }
        let Some(living) = self.entity.get_living_entity() else {
            return;
        };
        let Some(snapshot) = self.try_equipment_snapshot(living) else {
            self.equipment_retry_pending.store(true, Relaxed);
            return;
        };
        let pending = self
            .lock_equipment_pairings()
            .iter_mut()
            .filter(|(_, state)| {
                !state.retry_in_flight && (state.needs_snapshot || !state.pending.is_empty())
            })
            .map(|(&viewer, state)| {
                state.retry_in_flight = true;
                let mut slots = std::mem::take(&mut state.pending);
                if std::mem::take(&mut state.needs_snapshot) {
                    for slot in Self::occupied_slots(&snapshot) {
                        if !slots.contains(&slot) {
                            slots.push(slot);
                        }
                    }
                }
                (
                    state.viewer.clone(),
                    EquipmentSession {
                        viewer,
                        generation: state.generation,
                    },
                    slots,
                )
            })
            .collect::<Vec<_>>();
        for (player, session, slots) in pending {
            self.send_equipment_pairing(living, &player, session, snapshot.clone(), slots, 1);
            let _pairing = EquipmentPairing {
                tracker: self,
                session,
            };
            if let Some(state) =
                Self::current_equipment_pairing(&mut self.lock_equipment_pairings(), session)
            {
                state.retry_in_flight = false;
                if state.needs_snapshot || !state.pending.is_empty() {
                    self.equipment_retry_pending.store(true, Relaxed);
                }
            }
        }
    }

    const ARMOR_SLOTS: [EquipmentSlot; 5] = [
        EquipmentSlot::HEAD,
        EquipmentSlot::CHEST,
        EquipmentSlot::LEGS,
        EquipmentSlot::FEET,
        EquipmentSlot::BODY,
    ];

    fn send_bedrock_pairing(
        &self,
        client: &BedrockClient,
        living: &LivingEntity,
        snapshot: &EntityEquipment,
        slots: &[EquipmentSlot],
        session: EquipmentSession,
    ) -> Vec<EquipmentSlot> {
        let mut stale = Vec::new();
        for slot in [EquipmentSlot::MAIN_HAND, EquipmentSlot::OFF_HAND] {
            if slots.contains(&slot)
                && self.send_bedrock_hand(client, living, &slot, &snapshot.get(&slot), session)
            {
                stale.push(slot);
            }
        }
        if slots.iter().any(|slot| Self::ARMOR_SLOTS.contains(slot))
            && self.send_bedrock_armor(client, living, snapshot, session)
        {
            // Armor is sent and retried as one group.
            stale.push(EquipmentSlot::HEAD);
        }
        stale
    }

    fn equipment_stacks_match(
        current: Option<&ItemStack>,
        snapshot: Option<&ItemStack>,
        bedrock: bool,
    ) -> bool {
        let current = current.filter(|stack| !stack.is_empty());
        match (current, snapshot.filter(|stack| !stack.is_empty())) {
            (Some(current), Some(snapshot)) => {
                current.are_equal(snapshot) && (!bedrock || current.uid == snapshot.uid)
            }
            (None, None) => true,
            _ => false,
        }
    }

    fn java_equipment_from_packet(
        data: &[u8],
        version: pumpkin_util::version::JavaMinecraftVersion,
        entity_id: i32,
    ) -> Option<CSetEquipment> {
        let mut payload = data;
        if payload.get_var_int().ok()?.0 != CSetEquipment::to_id(version) {
            return None;
        }
        let packet = CSetEquipment::read(&mut payload, &version).ok()?;
        (packet.entity_id.0 == entity_id).then_some(packet)
    }

    fn java_equipment_update(
        &self,
        slots: impl IntoIterator<Item = (EquipmentSlot, ItemStack)>,
    ) -> Option<JavaEquipmentUpdate> {
        let (slots, equipment): (Vec<_>, Vec<_>) = slots
            .into_iter()
            .map(|(slot, stack)| {
                let id = slot.discriminant();
                (slot, (id, ItemStackSerializer::from(stack)))
            })
            .unzip();
        if slots.is_empty() || self.entity.get_entity().is_removed() {
            return None;
        }
        let packet = CSetEquipment::new(self.entity_id.into(), equipment);
        let data = JavaClient::serialize_packet_for_version(&packet, CURRENT_MC_VERSION).ok()?;
        Some(JavaEquipmentUpdate {
            slots,
            packet,
            data,
        })
    }

    fn send_java_equipment(
        &self,
        client: &JavaClient,
        living: &LivingEntity,
        update: &JavaEquipmentUpdate,
        session: EquipmentSession,
    ) -> Vec<EquipmentSlot> {
        let version = client.version.load();
        let changed = RefCell::new(Vec::new());
        let active = Cell::new(true);
        client.try_enqueue_packet_data_projecting(
            update.data.clone(),
            || {
                let gear = EquippedStacks::lock(living, self.entity.get_player());
                let viewing = self.lock_pairing_session(session);
                active.set(viewing.is_some());
                if active.get() {
                    changed.borrow_mut().extend(
                        update
                            .slots
                            .iter()
                            .zip(&update.packet.equipment)
                            .filter(|(slot, (_, stack))| !gear.matches(slot, Some(&stack.0), false))
                            .map(|(slot, _)| slot.clone()),
                    );
                }
                let pairings = active.get().then(|| self.lock_equipment_pairings());
                (gear, viewing, pairings)
            },
            |data, primary, _| {
                if !primary && active.get() && !self.java_relationship_is_current(&data, version) {
                    return None;
                }
                let stale = changed.borrow();
                if active.get() && stale.is_empty() {
                    return Some(data);
                }
                if !active.get() && Self::java_pairing_link(&data, version, self.entity_id) {
                    return None;
                }
                Self::project_java_equipment(
                    data,
                    primary,
                    active.get(),
                    &stale,
                    version,
                    self.entity_id,
                )
                .and_then(|data| {
                    if active.get() {
                        Some(data)
                    } else {
                        Self::without_java_removal(data, version, self.entity_id)
                    }
                })
            },
            |data, _, (_, _, pairings)| {
                if let Some(state) = pairings
                    .as_deref_mut()
                    .and_then(|records| Self::current_equipment_pairing(records, session))
                    .filter(|state| state.spawned)
                    && let Some(represented) =
                        Self::java_equipment_from_packet(data, version, self.entity_id)
                {
                    Self::record_java_equipment(state, &update.packet, &represented);
                }
            },
        );
        let mut changed = changed.into_inner();
        if !changed.is_empty() {
            let gear = EquippedStacks::lock(living, self.entity.get_player());
            let mut pairings = self.lock_equipment_pairings();
            let Some(state) = Self::current_equipment_pairing(&mut pairings, session) else {
                return Vec::new();
            };
            changed.retain(|slot| {
                !state
                    .java_items
                    .get(&slot.discriminant())
                    .is_some_and(|sent| gear.matches(slot, Some(sent), false))
            });
        }
        changed
    }

    fn record_java_equipment(
        state: &mut EquipmentPairingState,
        authoritative: &CSetEquipment,
        represented: &CSetEquipment,
    ) {
        for (slot, item) in &authoritative.equipment {
            if represented
                .equipment
                .iter()
                .any(|(delivered_slot, _)| slot == delivered_slot)
            {
                state.java_spawn_overrides.remove(slot);
                state.java_items.insert(*slot, item.0.as_ref().clone());
            }
        }
    }

    fn project_java_equipment(
        data: Bytes,
        primary: bool,
        active: bool,
        stale: &[EquipmentSlot],
        version: pumpkin_util::version::JavaMinecraftVersion,
        entity_id: i32,
    ) -> Option<Bytes> {
        if !active {
            return (!primary
                && Self::java_pairing_actor(&data, version)
                    .is_none_or(|(_, actor)| actor != entity_id))
            .then_some(data);
        }
        let Some(mut packet) = Self::java_equipment_from_packet(&data, version, entity_id) else {
            return (!primary).then_some(data);
        };
        let original = packet.equipment.len();
        packet
            .equipment
            .retain(|(id, _)| !stale.iter().any(|slot| slot.discriminant() == *id));
        match packet.equipment.len() {
            0 => None,
            count if count == original => Some(data),
            _ => JavaClient::serialize_packet_for_version(&packet, version).ok(),
        }
    }

    fn send_bedrock_hand(
        &self,
        client: &BedrockClient,
        living: &LivingEntity,
        slot: &EquipmentSlot,
        stack: &ItemStack,
        session: EquipmentSession,
    ) -> bool {
        use pumpkin_protocol::bedrock::server::inventory_transaction::WINDOW_ID_OFF_HAND;
        let container_id = if *slot == EquipmentSlot::OFF_HAND {
            WINDOW_ID_OFF_HAND as u8
        } else {
            0
        };
        let packet = CMobEquipment {
            target_runtime_id: (self.entity_id as u64).into(),
            item: stack.into(),
            slot: 0,
            selected_slot: 0,
            container_id,
        };
        let group = BedrockEquipmentGroup {
            packet_id: CMobEquipment::PACKET_ID as u32,
            entity_id: packet.target_runtime_id.0,
            container: Some(container_id),
        };
        self.send_bedrock_equipment(client, living, &packet, group, session, |current| {
            current.matches(slot, Some(stack), true)
        })
    }

    fn send_bedrock_armor(
        &self,
        client: &BedrockClient,
        living: &LivingEntity,
        snapshot: &EntityEquipment,
        session: EquipmentSession,
    ) -> bool {
        let descriptor = |slot: &EquipmentSlot| {
            snapshot.equipment.get(slot).map_or_else(
                NetworkItemStackDescriptor::default,
                NetworkItemStackDescriptor::from,
            )
        };
        let packet = CMobArmorEquipment {
            target_runtime_id: (self.entity_id as u64).into(),
            head: descriptor(&EquipmentSlot::HEAD),
            chest: descriptor(&EquipmentSlot::CHEST),
            legs: descriptor(&EquipmentSlot::LEGS),
            feet: descriptor(&EquipmentSlot::FEET),
            body: descriptor(&EquipmentSlot::BODY),
        };
        let group = BedrockEquipmentGroup {
            packet_id: CMobArmorEquipment::PACKET_ID as u32,
            entity_id: packet.target_runtime_id.0,
            container: None,
        };
        self.send_bedrock_equipment(client, living, &packet, group, session, |current| {
            Self::ARMOR_SLOTS
                .iter()
                .all(|slot| current.matches(slot, snapshot.equipment.get(slot), true))
        })
    }

    fn bedrock_equipment_group(data: &[u8]) -> Option<BedrockEquipmentGroup> {
        let mut payload = data;
        let packet_id = VarUInt::read(&mut payload).ok()?.0;
        let entity_id = VarULong::read(&mut payload).ok()?.0;
        let container = if packet_id == CMobEquipment::PACKET_ID as u32 {
            NetworkItemStackDescriptor::read(&mut payload).ok()?;
            u8::read(&mut payload).ok()?;
            u8::read(&mut payload).ok()?;
            Some(u8::read(&mut payload).ok()?)
        } else if packet_id == CMobArmorEquipment::PACKET_ID as u32 {
            None
        } else {
            return None;
        };
        Some(BedrockEquipmentGroup {
            packet_id,
            entity_id,
            container,
        })
    }

    fn send_bedrock_equipment<P: BClientPacket>(
        &self,
        client: &BedrockClient,
        living: &LivingEntity,
        packet: &P,
        group: BedrockEquipmentGroup,
        session: EquipmentSession,
        matches: impl Fn(&EquippedStacks<'_>) -> bool,
    ) -> bool {
        let Ok(data) = client.serialize_packet(packet) else {
            return false;
        };
        let previously_delivered =
            Self::current_equipment_pairing(&mut self.lock_equipment_pairings(), session)
                .is_some_and(|state| state.bedrock_items.get(&group) == Some(&data));
        if previously_delivered {
            return !matches(&EquippedStacks::lock(living, self.entity.get_player()));
        }
        let canonical = data.clone();
        let stale = Cell::new(false);
        let active = Cell::new(true);
        client.try_enqueue_packet_data_projecting(
            data,
            || {
                let gear = EquippedStacks::lock(living, self.entity.get_player());
                let viewing = self.lock_pairing_session(session);
                active.set(viewing.is_some());
                stale.set(active.get() && !matches(&gear));
                let pairings = active.get().then(|| self.lock_equipment_pairings());
                (gear, viewing, pairings)
            },
            |data, primary, _| {
                if !primary && active.get() && !self.bedrock_relationship_is_current(&data) {
                    return None;
                }
                if !active.get() {
                    return (!primary && !self.is_bedrock_pairing_actor(&data)).then_some(data);
                }
                (!stale.get() || (!primary && Self::bedrock_equipment_group(&data) != Some(group)))
                    .then_some(data)
            },
            |data, primary, (_, _, pairings)| {
                // The primary retains its typed group even if the translator changes its encoding.
                if let Some(state) = pairings
                    .as_deref_mut()
                    .and_then(|records| Self::current_equipment_pairing(records, session))
                    .filter(|state| state.spawned)
                    && (primary || Self::bedrock_equipment_group(data) == Some(group))
                {
                    state.bedrock_items.insert(group, canonical.clone());
                }
            },
        );
        stale.get()
    }

    fn is_bedrock_pairing_actor(&self, mut payload: &[u8]) -> bool {
        use pumpkin_protocol::bedrock::client::{
            CAddActor, CAddItemActor, CAddPlayer, CMobEffect, set_actor_data::CSetActorData,
        };
        let Ok(id) = VarUInt::read(&mut payload) else {
            return false;
        };
        let id = id.0 as i32;
        if id == pumpkin_protocol::bedrock::client::CSetActorLink::PACKET_ID {
            let actor = i64::from(self.entity_id);
            return (0..2).any(|_| VarLong::read(&mut payload).is_ok_and(|id| id.0 == actor));
        }
        if id == CRemoveActor::PACKET_ID {
            return VarLong::read(&mut payload).is_ok_and(|id| id.0 == i64::from(self.entity_id));
        }
        if id == CAddPlayer::PACKET_ID {
            return self.entity.get_player().is_some_and(|player| {
                Uuid::read(&mut payload).is_ok_and(|uuid| uuid == player.gameprofile.id)
            });
        }
        if id == CAddActor::PACKET_ID || id == CAddItemActor::PACKET_ID {
            return VarLong::read(&mut payload).is_ok_and(|id| id.0 == i64::from(self.entity_id));
        }
        [
            CSetActorMotion::PACKET_ID,
            CSetActorData::PACKET_ID,
            CMobEffect::PACKET_ID,
            CMobEquipment::PACKET_ID,
            CMobArmorEquipment::PACKET_ID,
        ]
        .contains(&id)
            && VarULong::read(&mut payload).is_ok_and(|id| id.0 == self.entity_id as u64)
    }

    pub(crate) fn send_cleared_hand_equipment(&self, living: &LivingEntity, slot: &EquipmentSlot) {
        self.broadcast_equipment(living, &[(slot.clone(), ItemStack::EMPTY.clone())]);
    }

    pub(crate) fn broadcast_equipment(
        &self,
        living: &LivingEntity,
        equipment: &[(EquipmentSlot, ItemStack)],
    ) {
        let viewers = self.equipment_viewers();
        let java = viewers
            .iter()
            .any(|viewer| viewer.player.client.java().is_some())
            .then(|| self.java_equipment_update(equipment.iter().cloned()))
            .flatten();
        let armor_changed = equipment
            .iter()
            .any(|(slot, _)| Self::ARMOR_SLOTS.contains(slot));
        for viewer in viewers {
            let session = EquipmentSession {
                viewer: viewer.player.gameprofile.id,
                generation: viewer.generation,
            };
            match viewer.player.client.as_ref() {
                ClientPlatform::Java(client) => {
                    if let Some(update) = &java {
                        self.send_java_equipment(client, living, update, session);
                    }
                }
                ClientPlatform::Bedrock(client) => {
                    for (slot, stack) in equipment {
                        if *slot == EquipmentSlot::MAIN_HAND || *slot == EquipmentSlot::OFF_HAND {
                            self.send_bedrock_hand(client, living, slot, stack, session);
                        }
                    }
                    if armor_changed {
                        let snapshot = living
                            .entity_equipment
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .clone();
                        if equipment.iter().any(|(slot, stack)| {
                            Self::ARMOR_SLOTS.contains(slot)
                                && Self::equipment_stacks_match(
                                    snapshot.equipment.get(slot),
                                    Some(stack),
                                    true,
                                )
                        }) {
                            self.send_bedrock_armor(client, living, &snapshot, session);
                        }
                    }
                }
                ClientPlatform::SimulatedPlayer(_) => {}
            }
        }
    }

    pub fn send_to_tracking_players<P: ClientPacket + Sync>(&self, packet: &P, _world: &World) {
        let seen_by = self.viewers_snapshot();
        World::broadcast_java_players(packet, seen_by.iter());
    }

    pub fn send_to_tracking_players_bedrock<P: BClientPacket + Sync>(
        &self,
        packet: &P,
        _world: &World,
    ) {
        if !self.has_bedrock_watchers() {
            return;
        }
        let seen_by = self.bedrock_watchers_snapshot();
        let recipients = seen_by.iter().filter_map(|p| {
            if let ClientPlatform::Bedrock(client) = p.client.as_ref() {
                return Some(client);
            }
            None
        });
        World::broadcast_bedrock_grouped(packet, recipients);
    }

    pub fn send_to_tracking_players_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
        _world: &World,
    ) {
        let seen_by = self.viewers_snapshot();
        World::broadcast_java_players(je_packet, seen_by.iter());
        if self.has_bedrock_watchers() {
            World::broadcast_bedrock_grouped(
                be_packet,
                seen_by.iter().filter_map(|p| match p.client.as_ref() {
                    ClientPlatform::Bedrock(be_client) => Some(be_client),
                    ClientPlatform::Java(_) | ClientPlatform::SimulatedPlayer(_) => None,
                }),
            );
        }
    }

    pub fn send_to_tracking_players_and_self<P: ClientPacket + Sync>(
        &self,
        packet: &P,
        world: &World,
    ) {
        self.send_to_tracking_players(packet, world);
        if let Some(player) = self.entity.get_player() {
            player.try_send_client_packet(packet);
        }
    }

    pub fn send_to_tracking_players_and_self_editioned<
        J: ClientPacket + Sync,
        B: BClientPacket + Sync,
    >(
        &self,
        je_packet: &J,
        be_packet: &B,
        world: &World,
    ) {
        self.send_to_tracking_players_editioned(je_packet, be_packet, world);
        if let Some(player) = self.entity.get_player() {
            player.try_enqueue_packet_editioned(je_packet, be_packet);
        }
    }

    pub fn send_to_tracking_players_filtered<P: ClientPacket + Sync, F: Fn(&Player) -> bool>(
        &self,
        packet: &P,
        _world: &World,
        filter: F,
    ) {
        let seen_by = self.viewers_snapshot();
        World::broadcast_java_players(packet, seen_by.iter().filter(|p| filter(p)));
    }

    pub fn send_to_tracking_players_filtered_editioned<
        J: ClientPacket + Sync,
        B: BClientPacket + Sync,
        F: Fn(&Player) -> bool,
    >(
        &self,
        je_packet: &J,
        be_packet: &B,
        _world: &World,
        filter: F,
    ) {
        let seen_by = self.viewers_snapshot();
        if seen_by.is_empty() {
            return;
        }
        if !self.has_bedrock_watchers() {
            World::broadcast_java_players(je_packet, seen_by.iter().filter(|p| filter(p)));
            return;
        }
        let mut java_recipients = Vec::new();
        let mut bedrock_recipients = Vec::new();
        for p in seen_by.iter().filter(|p| filter(p)) {
            match p.client.as_ref() {
                ClientPlatform::Java(java_client) => java_recipients.push(java_client),
                ClientPlatform::Bedrock(be_client) => bedrock_recipients.push(be_client),
                ClientPlatform::SimulatedPlayer(_) => {}
            }
        }
        World::broadcast_java_clients(je_packet, java_recipients.into_iter());
        World::broadcast_bedrock_grouped(be_packet, bedrock_recipients.into_iter());
    }
}

pub struct EntityTracker {
    pub entity_map: DashMap<i32, Arc<TrackedEntity>>,
}

impl Default for EntityTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityTracker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entity_map: DashMap::new(),
        }
    }

    #[must_use]
    pub fn get_tracked_entity(&self, entity_id: i32) -> Option<Arc<TrackedEntity>> {
        self.entity_map.get(&entity_id).map(|r| r.value().clone())
    }

    #[must_use]
    pub fn has_entity_with_id(&self, entity_id: i32) -> bool {
        self.entity_map.contains_key(&entity_id)
    }

    #[must_use]
    pub fn is_tracked_by_any_player(&self, entity_id: i32) -> bool {
        self.entity_map
            .get(&entity_id)
            .is_some_and(|t| t.has_viewers())
    }

    pub fn for_each_entity_tracked_by<F: FnMut(&Arc<dyn EntityBase>)>(
        &self,
        player: &Player,
        mut f: F,
    ) {
        for tracked in self.tracking_snapshot(|tracked| tracked.is_seen_by(&player.gameprofile.id))
        {
            f(&tracked.entity);
        }
    }

    fn tracking_snapshot(
        &self,
        include: impl Fn(&TrackedEntity) -> bool,
    ) -> Vec<Arc<TrackedEntity>> {
        // Packet callbacks can remove entities. Release every map guard before dispatch.
        self.entity_map
            .iter()
            .filter(|entry| include(entry.value()))
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn add_entity(&self, entity: &Arc<dyn EntityBase>, world: &World) {
        let base = entity.get_entity();
        let entity_type = base.entity_type;
        let range = entity_type.client_tracking_range;
        if range == 0 {
            return;
        }
        let update_interval = entity_type.update_interval;
        let track_deltas = entity_type.track_deltas;
        let entity_id = base.entity_id;

        let tracked = match self.entity_map.entry(entity_id) {
            dashmap::mapref::entry::Entry::Occupied(_) => return,
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                // World removal marks the actor before removing this map entry.
                if base.is_removed() {
                    return;
                }
                let tracked = Arc::new(TrackedEntity::new(
                    entity.clone(),
                    range,
                    update_interval,
                    track_deltas,
                ));
                entry.insert(tracked.clone());
                tracked
            }
        };

        let players = world.players.load();
        tracked.update_players(players.as_ref(), world);
    }

    /// Must only be called after the player's own `CLogin` packet has been sent.
    pub fn pair_new_player_with_tracked_entities(&self, player_arc: &Arc<Player>, world: &World) {
        let entity_id = player_arc.get_entity().entity_id;
        for tracked in self.tracking_snapshot(|tracked| tracked.entity_id != entity_id) {
            tracked.update_player(player_arc, world);
        }
    }

    /// Vanilla `PlayerList.respawn` -> viewers drop the dead entity, then spawn it fresh.
    pub fn respawn_entity(&self, entity: &Arc<dyn EntityBase>, world: &World) {
        if let Some((_, tracked)) = self.entity_map.remove(&entity.get_entity().entity_id) {
            tracked.broadcast_removed(world);
        }
        self.add_entity(entity, world);
    }

    /// Vanilla `PlayerList.respawn` recreates the player -> forget and re-pair it as a viewer.
    pub fn repair_respawned_player(&self, player: &Arc<Player>, world: &World) {
        let entity_id = player.get_entity().entity_id;
        for tracked in self.tracking_snapshot(|tracked| tracked.entity_id != entity_id) {
            tracked.forget_player(player);
            tracked.update_player(player, world);
        }
    }

    /// Pairs entities in chunks whose packet was just queued for `player`.
    pub fn update_player_chunks(
        &self,
        player: &Arc<Player>,
        world: &World,
        chunks: &[Vector2<i32>],
    ) {
        if chunks.is_empty() {
            return;
        }
        let chunks: FxHashSet<_> = chunks.iter().copied().collect();
        let entity_id = player.get_entity().entity_id;
        for tracked in self.tracking_snapshot(|tracked| {
            tracked.entity_id != entity_id
                && chunks.contains(&tracked.entity.get_entity().chunk_pos.load())
        }) {
            tracked.update_player(player, world);
        }
    }

    pub fn remove_entity(&self, entity: &dyn EntityBase, world: &World) {
        let entity_id = entity.get_entity().entity_id;
        if let Some(player) = entity.get_player() {
            for tracked in
                self.tracking_snapshot(|tracked| tracked.is_seen_by(&player.gameprofile.id))
            {
                tracked.remove_player(player);
            }
        }

        if let Some((_, tracked)) = self.entity_map.remove(&entity_id) {
            tracked.broadcast_removed(world);
        }
    }

    pub fn update_player_position(&self, player: &Arc<Player>, _world: &World) {
        let player_id = player.get_entity().entity_id;
        if let Some(tracked) = self.entity_map.get(&player_id) {
            tracked.pos_dirty.store(true, Relaxed);
        }
    }

    pub fn update_entity_position(&self, entity: &dyn EntityBase, world: &World) {
        if let Some(tracked) = self.get_tracked_entity(entity.get_entity().entity_id) {
            let new_pos = chunk_section_from_pos(&entity.get_entity().block_pos.load());
            tracked.last_section_pos.store(new_pos);
            let players = world.players.load();
            tracked.update_players(players.as_ref(), world);
        }
    }

    pub fn update_all(&self, world: &World) {
        let players = world.players.load();
        let all_snapshots: Vec<PlayerViewSnapshot<'_>> = players
            .iter()
            .map(|player| {
                let entity = player.get_entity();
                PlayerViewSnapshot {
                    player,
                    entity_id: entity.entity_id,
                    pos: entity.pos.load(),
                    watched: player.watched_section.load(),
                    view_distance: get_view_distance(player).get() as i32,
                }
            })
            .collect();

        let tracked_entities: Vec<Arc<TrackedEntity>> = self
            .entity_map
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        let mut entity_moved = vec![false; tracked_entities.len()];
        let moved_player_ids: FxHashSet<i32> = tracked_entities
            .par_iter()
            .zip(entity_moved.par_iter_mut())
            .filter_map(|(tracked, moved_slot)| {
                // A pairing created below waits for this entity's next tracking tick.
                tracked.flush_pairing_equipment();
                tracked.flush_pairing_passengers();
                tracked.flush_pairing_effects();
                let new_pos = chunk_section_from_pos(&tracked.entity.get_entity().block_pos.load());
                let old_pos = tracked.last_section_pos.load();
                let section_changed = old_pos != new_pos;
                let sub_section_moved = tracked.pos_dirty.swap(false, Relaxed);
                if !section_changed && !sub_section_moved {
                    None
                } else {
                    *moved_slot = true;
                    let chunk_changed = old_pos.x != new_pos.x || old_pos.z != new_pos.z;
                    tracked.update_players_from_snapshots(&all_snapshots, chunk_changed);
                    if section_changed {
                        tracked.last_section_pos.store(new_pos);
                    }
                    tracked.entity.get_player().map(|_| tracked.entity_id)
                }
            })
            .collect();

        if !moved_player_ids.is_empty() {
            let moved_snapshots: Vec<PlayerViewSnapshot<'_>> = all_snapshots
                .iter()
                .copied()
                .filter(|snap| moved_player_ids.contains(&snap.entity_id))
                .collect();
            tracked_entities
                .par_iter()
                .zip(entity_moved.par_iter())
                .for_each(|(tracked, &moved)| {
                    if !moved {
                        tracked.update_players_from_snapshots(&moved_snapshots, false);
                    }
                });
        }

        tracked_entities.par_iter().for_each(|tracked| {
            tracked
                .entity
                .get_entity()
                .flush_bedrock_flags(tracked.entity.as_ref());
            if tracked.entity.get_player().is_none() {
                tracked.send_changes(world);
            } else if tracked.entity.get_entity().synched_data.is_dirty() {
                tracked.entity.get_entity().send_dirty_entity_data();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;
    use std::time::Duration;

    use pumpkin_protocol::{
        PacketDecodeError, RawPacket,
        java::{
            client::play::{CKeepAlive, CSetPassengers},
            packet_decoder::TCPNetworkDecoder,
        },
        packet::MultiVersionJavaPacket,
    };
    use tokio::io::{BufReader, DuplexStream};

    use super::*;
    use crate::entity::vehicle::{boat::BoatEntity, minecart::MinecartEntity};
    use crate::plugin::{
        BoxFuture, EventPriority, EventRwHandler, server::packet::PacketSentEvent,
    };
    use crate::server::Server;
    use crate::world::vehicle_tests::{CENTER, TestResult, java_rider, vehicle_world};

    async fn tracking_frames(
        tracked: &TrackedEntity,
        world: &World,
        client: &JavaClient,
        decoder: &mut TCPNetworkDecoder<BufReader<DuplexStream>>,
    ) -> Result<Vec<RawPacket>, Box<dyn std::error::Error>> {
        tracked.send_changes(world);
        let movement_ids = [
            CEntityPositionSync::to_id(CURRENT_MC_VERSION),
            CEntityVelocity::to_id(CURRENT_MC_VERSION),
            CUpdateEntityPos::to_id(CURRENT_MC_VERSION),
            CUpdateEntityPosRot::to_id(CURRENT_MC_VERSION),
            CUpdateEntityRot::to_id(CURRENT_MC_VERSION),
        ];
        Ok(queued_packets(client, decoder)
            .await?
            .into_iter()
            .filter(|packet| movement_ids.contains(&packet.id))
            .collect())
    }

    async fn queued_packets(
        client: &JavaClient,
        decoder: &mut TCPNetworkDecoder<BufReader<DuplexStream>>,
    ) -> Result<Vec<RawPacket>, Box<dyn std::error::Error>> {
        assert!(client.try_send_packet(&CKeepAlive::new(73)));
        let packets = tokio::time::timeout(Duration::from_secs(5), async {
            let mut packets = Vec::new();
            loop {
                let packet = decoder.get_raw_packet().await?;
                if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                    return Ok::<_, PacketDecodeError>(packets);
                }
                packets.push(packet);
            }
        })
        .await??;
        Ok(packets)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn duplicate_registration_preserves_the_last_sent_position() -> TestResult {
        let folder = tempfile::tempdir()?;
        let (_, world) = vehicle_world(&folder).await?;
        let cart: Arc<dyn EntityBase> = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            CENTER,
            &EntityType::MINECART,
        )));
        world.add_entity_silent(cart.clone());
        let entity = cart.get_entity();
        let tracked = world
            .entity_tracker
            .get_tracked_entity(entity.entity_id)
            .ok_or("Missing tracker")?;
        assert_eq!(entity.last_sent_pos.load(), CENTER);

        let new_pos = CENTER.add_raw(0.5, 0.0, 0.0);
        entity.set_pos(new_pos);
        assert_eq!(entity.pos.load(), new_pos);
        world.entity_tracker.add_entity(&cart, &world);

        let current = world
            .entity_tracker
            .get_tracked_entity(entity.entity_id)
            .ok_or("Missing tracker")?;
        assert!(Arc::ptr_eq(&tracked, &current));
        assert_eq!(entity.last_sent_pos.load(), CENTER);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn java_cooldown_observer_can_publish_a_newer_duration() -> TestResult {
        use pumpkin_protocol::java::client::play::CItemCooldown;

        const GROUP: &str = "minecraft:test_cooldown";
        const OTHER_GROUP: &str = "minecraft:other_cooldown";
        struct ReplaceCooldown(Arc<Player>, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for ReplaceCooldown {
            fn handle<'a>(
                &'a self,
                _: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    if event.packet_id == CItemCooldown::to_id(CURRENT_MC_VERSION)
                        && !self.1.swap(true, Relaxed)
                    {
                        self.0.start_cooldown(GROUP.to_owned(), 80);
                        event
                            .extra_packets
                            .push((event.packet_id, event.payload.clone()));
                        let mut payload = Vec::new();
                        assert!(
                            CItemCooldown::new(OTHER_GROUP.to_owned(), VarInt(40))
                                .write_packet_data(&mut payload, &CURRENT_MC_VERSION)
                                .is_ok()
                        );
                        event.extra_packets.push((event.packet_id, payload.into()));
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (player, peer) = java_rider(&server, &world, "Notch")?;
        let client = player.client.java().ok_or("Java player")?;
        client.player.store(Arc::new(Some(player.clone())));
        let observer = Arc::new(ReplaceCooldown(player.clone(), AtomicBool::new(false)));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        player.start_cooldown(GROUP.to_owned(), 20);
        assert!(observer.1.load(Relaxed));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let mut durations = Vec::new();
        for packet in queued_packets(client, &mut decoder).await? {
            if packet.id == CItemCooldown::to_id(CURRENT_MC_VERSION) {
                let mut payload = packet.payload.as_ref();
                let group = payload.get_str()?;
                durations.push((group, payload.get_var_int()?.0));
            }
        }
        assert_eq!(durations, [(GROUP.into(), 80), (OTHER_GROUP.into(), 40)]);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn legacy_java_cooldown_filters_only_the_superseded_item() -> TestResult {
        use pumpkin_protocol::{java::client::play::CItemCooldown, ser::NetworkWriteExt};
        use pumpkin_util::version::JavaMinecraftVersion;

        const VERSION: JavaMinecraftVersion = JavaMinecraftVersion::V_1_21;
        const GROUP: &str = "minecraft:test_cooldown";
        const ITEM: VarInt = VarInt(8);
        const UNRELATED: VarInt = VarInt(9);
        struct TranslateCooldown(Arc<Player>, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for TranslateCooldown {
            fn handle<'a>(
                &'a self,
                _: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    if event.packet_id != CItemCooldown::to_id(CURRENT_MC_VERSION) {
                        return;
                    }
                    let mut source = event.payload.as_ref();
                    assert!(source.get_str().is_ok_and(|group| group.as_ref() == GROUP));
                    let Ok(duration) = source.get_var_int() else {
                        return;
                    };
                    let encode = |item: VarInt, duration: VarInt| {
                        let mut payload = Vec::new();
                        assert!(payload.write_var_int(&item).is_ok());
                        assert!(payload.write_var_int(&duration).is_ok());
                        Bytes::from(payload)
                    };
                    event.packet_id = CItemCooldown::to_id(VERSION);
                    event.payload = encode(ITEM, duration);
                    if !self.1.swap(true, Relaxed) {
                        self.0.start_cooldown(GROUP.to_owned(), 80);
                        event
                            .extra_packets
                            .push((event.packet_id, event.payload.clone()));
                        event
                            .extra_packets
                            .push((event.packet_id, encode(UNRELATED, VarInt(40))));
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (player, peer) = java_rider(&server, &world, "Notch")?;
        let client = player.client.java().ok_or("Java player")?;
        client.player.store(Arc::new(Some(player.clone())));
        client.version.store(VERSION);
        let observer = Arc::new(TranslateCooldown(player.clone(), AtomicBool::new(false)));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        player.start_cooldown(GROUP.to_owned(), 20);
        assert!(observer.1.load(Relaxed));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let mut durations = Vec::new();
        for packet in queued_packets(client, &mut decoder).await? {
            if packet.id == CItemCooldown::to_id(VERSION) {
                let mut payload = packet.payload.as_ref();
                durations.push((payload.get_var_int()?, payload.get_var_int()?.0));
            }
        }
        assert_eq!(durations, [(ITEM, 80), (UNRELATED, 40)]);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_does_not_send_effects_removed_by_a_packet_observer() -> TestResult {
        use crate::entity::passive::pig::PigEntity;
        use pumpkin_data::potion::Potion;
        use pumpkin_protocol::java::client::play::CUpdateMobEffect;

        struct ClearEffects(Arc<PigEntity>, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for ClearEffects {
            fn handle<'a>(
                &'a self,
                _: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    if event.packet_id == CUpdateMobEffect::to_id(CURRENT_MC_VERSION)
                        && event.payload.as_ref().get_var_int().ok()
                            == Some(VarInt(self.0.get_entity().entity_id))
                        && !self.1.swap(true, Relaxed)
                        && let Some(living) = self.0.get_living_entity()
                    {
                        for effect in Potion::FIRE_RESISTANCE
                            .effects
                            .iter()
                            .chain(Potion::WATER_BREATHING.effects)
                        {
                            assert!(living.remove_effect(effect.effect_type));
                        }
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        let living = pig.get_living_entity().ok_or("Living pig")?;
        for effect in Potion::FIRE_RESISTANCE
            .effects
            .iter()
            .chain(Potion::WATER_BREATHING.effects)
        {
            living.add_effect(effect.clone());
        }
        assert!(world.spawn_entity(pig.clone()));
        let tracker = world
            .entity_tracker
            .get_tracked_entity(pig.get_entity().entity_id)
            .ok_or("Pig tracker")?;
        let observer = Arc::new(ClearEffects(pig.clone(), AtomicBool::new(false)));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        tracker.add_pairing(&viewer);
        assert!(observer.1.load(Relaxed));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        assert!(
            queued_packets(client, &mut decoder)
                .await?
                .iter()
                .all(|packet| packet.id != CUpdateMobEffect::to_id(CURRENT_MC_VERSION))
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_keeps_the_shorter_reapplied_effect() -> TestResult {
        use crate::entity::passive::pig::PigEntity;
        use pumpkin_data::potion::Potion;
        use pumpkin_protocol::java::client::play::CUpdateMobEffect;

        struct ReapplyEffect(Arc<PigEntity>, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for ReapplyEffect {
            fn handle<'a>(
                &'a self,
                _: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    let mut payload = event.payload.as_ref();
                    if event.packet_id == CUpdateMobEffect::to_id(CURRENT_MC_VERSION)
                        && payload.get_var_int().ok() == Some(VarInt(self.0.get_entity().entity_id))
                        && let Ok(effect_id) = payload.get_var_int()
                        && !self.1.swap(true, Relaxed)
                        && let Some(living) = self.0.get_living_entity()
                    {
                        for effect in Potion::FIRE_RESISTANCE
                            .effects
                            .iter()
                            .chain(Potion::WATER_BREATHING.effects)
                        {
                            assert!(living.remove_effect(effect.effect_type));
                            if i32::from(effect.effect_type.id) == effect_id.0 {
                                let mut replacement = effect.clone();
                                replacement.duration = 20;
                                living.add_effect(replacement);
                            }
                        }
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        let living = pig.get_living_entity().ok_or("Living pig")?;
        for effect in Potion::FIRE_RESISTANCE
            .effects
            .iter()
            .chain(Potion::WATER_BREATHING.effects)
        {
            living.add_effect(effect.clone());
        }
        assert!(world.spawn_entity(pig.clone()));
        let tracker = world
            .entity_tracker
            .get_tracked_entity(pig.get_entity().entity_id)
            .ok_or("Pig tracker")?;
        viewer
            .watched_section
            .store(pumpkin_world::cylindrical_chunk_iterator::Cylindrical::new(
                Vector2::new(0, 0),
                std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
            ));
        let observer = Arc::new(ReapplyEffect(pig.clone(), AtomicBool::new(false)));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        tracker.add_pairing(&viewer);
        assert!(observer.1.load(Relaxed));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let mut durations = Vec::new();
        for packet in queued_packets(client, &mut decoder).await? {
            if packet.id == CUpdateMobEffect::to_id(CURRENT_MC_VERSION) {
                let mut payload = packet.payload.as_ref();
                let _entity = payload.get_var_int()?;
                let _effect = payload.get_var_int()?;
                let _amplifier = payload.get_var_int()?;
                durations.push(payload.get_var_int()?.0);
            }
        }
        assert!(!durations.is_empty());
        assert!(
            durations.iter().all(|duration| *duration == 20),
            "{durations:?}"
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_retries_an_effect_after_both_immediate_attempts_change() -> TestResult {
        use crate::entity::passive::pig::PigEntity;
        use pumpkin_data::potion::Potion;
        use pumpkin_protocol::java::client::play::CUpdateMobEffect;

        struct TickEffect(Arc<PigEntity>, AtomicU32);
        impl EventRwHandler<PacketSentEvent> for TickEffect {
            fn handle<'a>(
                &'a self,
                _: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    if event.packet_id == CUpdateMobEffect::to_id(CURRENT_MC_VERSION)
                        && event.payload.as_ref().get_var_int().ok()
                            == Some(VarInt(self.0.get_entity().entity_id))
                        && self.1.fetch_add(1, Relaxed) < 2
                        && let Some(living) = self.0.get_living_entity()
                    {
                        let mut effects = living
                            .active_effects
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        for effect in effects.values_mut() {
                            effect.duration -= 1;
                        }
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        let living = pig.get_living_entity().ok_or("Living pig")?;
        let mut effect = Potion::FIRE_RESISTANCE
            .effects
            .first()
            .ok_or("Fire resistance")?
            .clone();
        effect.duration = 60;
        living.add_effect(effect);
        assert!(world.spawn_entity(pig.clone()));
        let tracker = world
            .entity_tracker
            .get_tracked_entity(pig.get_entity().entity_id)
            .ok_or("Pig tracker")?;
        let observer = Arc::new(TickEffect(pig.clone(), AtomicU32::new(0)));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);

        tracker.add_pairing(&viewer);
        assert_eq!(observer.1.load(Relaxed), 2);
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        assert!(
            queued_packets(client, &mut decoder)
                .await?
                .iter()
                .all(|packet| packet.id != CUpdateMobEffect::to_id(CURRENT_MC_VERSION))
        );
        world.entity_tracker.update_all(&world);
        let mut durations = Vec::new();
        for packet in queued_packets(client, &mut decoder).await? {
            if packet.id == CUpdateMobEffect::to_id(CURRENT_MC_VERSION) {
                let mut payload = packet.payload.as_ref();
                assert_eq!(payload.get_var_int()?.0, pig.get_entity().entity_id);
                let _effect = payload.get_var_int()?;
                let _amplifier = payload.get_var_int()?;
                durations.push(payload.get_var_int()?.0);
            }
        }
        assert_eq!(durations, [58]);
        world.level.shutdown().await;
        Ok(())
    }

    struct SelectStick {
        rider: Arc<Player>,
        called: AtomicBool,
    }
    impl EventRwHandler<PacketSentEvent> for SelectStick {
        fn handle<'a>(
            &'a self,
            server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CSetEquipment::to_id(CURRENT_MC_VERSION)
                    && event
                        .payload
                        .as_ref()
                        .get_var_int()
                        .is_ok_and(|id| id.0 == self.rider.get_entity().entity_id)
                    && !self.called.swap(true, Relaxed)
                    && let Some(client) = self.rider.client.java()
                {
                    client.handle_set_held_item(
                        server,
                        &self.rider,
                        &pumpkin_protocol::java::server::play::SSetHeldItem { slot: 2 },
                    );
                }
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn retired_tracker_cannot_pair_a_snapshot_after_respawn() -> TestResult {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (rider, _rider_peer) = java_rider(&server, &world, "Notch")?;
        let (viewer, _viewer_peer) = java_rider(&server, &world, "jeb_")?;
        let rider_id = rider.get_entity().entity_id;
        let actor: Arc<dyn EntityBase> = rider;
        world.entity_tracker.add_entity(&actor, &world);
        let previous = world
            .entity_tracker
            .get_tracked_entity(rider_id)
            .ok_or("Previous tracker")?;
        world.entity_tracker.respawn_entity(&actor, &world);
        let current = world
            .entity_tracker
            .get_tracked_entity(rider_id)
            .ok_or("Current tracker")?;
        assert!(!Arc::ptr_eq(&previous, &current));
        previous.add_pairing(&viewer);
        assert!(!previous.is_seen_by(&viewer.gameprofile.id));
        current.add_pairing(&viewer);
        assert!(current.is_seen_by(&viewer.gameprofile.id));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn removal_observer_replacement_keeps_the_new_spawn_visible() -> TestResult {
        use pumpkin_protocol::java::client::play::CSpawnEntity;

        struct RepairOnRemoval(Arc<TrackedEntity>, Arc<Player>, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for RepairOnRemoval {
            fn handle<'a>(
                &'a self,
                _server: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    if event.packet_id == CRemoveEntities::to_id(CURRENT_MC_VERSION)
                        && !self.2.swap(true, Relaxed)
                    {
                        let world = self.0.entity.get_entity().world.load_full();
                        world.entity_tracker.respawn_entity(&self.0.entity, &world);
                        if let Some(replacement) =
                            world.entity_tracker.get_tracked_entity(self.0.entity_id)
                        {
                            replacement.add_pairing(&self.1);
                        }
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let boat: Arc<dyn EntityBase> = Arc::new(BoatEntity::new(Entity::new(
            world.clone(),
            CENTER,
            &EntityType::OAK_BOAT,
        )));
        world.entity_tracker.add_entity(&boat, &world);
        let tracked = world
            .entity_tracker
            .get_tracked_entity(boat.get_entity().entity_id)
            .ok_or("Boat tracker")?;
        tracked.add_pairing(&viewer);
        queued_packets(client, &mut decoder).await?;

        let listener = Arc::new(RepairOnRemoval(
            tracked.clone(),
            viewer.clone(),
            AtomicBool::new(false),
        ));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(listener.clone(), EventPriority::Normal, None);
        tracked.remove_pairing(&viewer);
        let changes: Vec<_> = queued_packets(client, &mut decoder)
            .await?
            .into_iter()
            .map(|packet| packet.id)
            .filter(|id| {
                [
                    CSpawnEntity::to_id(CURRENT_MC_VERSION),
                    CRemoveEntities::to_id(CURRENT_MC_VERSION),
                ]
                .contains(id)
            })
            .collect();
        assert!(listener.2.load(Relaxed));
        let replacement = world
            .entity_tracker
            .get_tracked_entity(tracked.entity_id)
            .ok_or("replacement tracker")?;
        assert!(!Arc::ptr_eq(&tracked, &replacement));
        assert!(replacement.is_seen_by(&viewer.gameprofile.id));
        assert_eq!(changes, [CSpawnEntity::to_id(CURRENT_MC_VERSION)]);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_does_not_spawn_a_mob_removed_by_its_spawn_observer() -> TestResult {
        use crate::entity::passive::pig::PigEntity;
        use pumpkin_protocol::java::client::play::CSpawnEntity;

        const UNRELATED_CARRIER: VarInt = VarInt(98765);

        struct RemoveOnSpawn(Arc<PigEntity>, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for RemoveOnSpawn {
            fn handle<'a>(
                &'a self,
                _server: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    if event.packet_id == CSpawnEntity::to_id(CURRENT_MC_VERSION)
                        && event
                            .payload
                            .as_ref()
                            .get_var_int()
                            .is_ok_and(|id| id.0 == self.0.get_entity().entity_id)
                        && !self.1.swap(true, Relaxed)
                    {
                        let actor = VarInt(self.0.get_entity().entity_id);
                        self.0.get_entity().remove();
                        let mut removal = Vec::new();
                        assert!(
                            CRemoveEntities::new(&[actor, UNRELATED_CARRIER])
                                .write_packet_data(&mut removal, &CURRENT_MC_VERSION)
                                .is_ok()
                        );
                        event
                            .extra_packets
                            .push((CRemoveEntities::to_id(CURRENT_MC_VERSION), removal.into()));
                        for (carrier, riders) in [
                            (actor, &[][..]),
                            (UNRELATED_CARRIER, &[actor][..]),
                            (UNRELATED_CARRIER, &[][..]),
                        ] {
                            let mut payload = Vec::new();
                            assert!(
                                CSetPassengers::new(carrier, riders)
                                    .write_packet_data(&mut payload, &CURRENT_MC_VERSION)
                                    .is_ok()
                            );
                            event
                                .extra_packets
                                .push((CSetPassengers::to_id(CURRENT_MC_VERSION), payload.into()));
                        }
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let mob = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        let id = mob.get_entity().entity_id;
        let actor: Arc<dyn EntityBase> = mob.clone();
        assert!(world.spawn_entity(actor));
        let tracked = world
            .entity_tracker
            .get_tracked_entity(id)
            .ok_or("Pig tracker")?;
        let observer = Arc::new(RemoveOnSpawn(mob.clone(), AtomicBool::new(false)));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        tracked.add_pairing(&viewer);
        assert!(observer.1.load(Relaxed));
        assert!(mob.get_entity().is_removed());
        let mut passenger_updates = Vec::new();
        let mut unrelated_removals = Vec::new();
        for packet in queued_packets(client, &mut decoder).await? {
            if packet.id == CRemoveEntities::to_id(CURRENT_MC_VERSION) {
                let mut payload = packet.payload.as_ref();
                let count = payload.get_var_int()?.0;
                let ids = (0..count)
                    .map(|_| payload.get_var_int())
                    .collect::<Result<Vec<_>, _>>()?;
                if ids.contains(&UNRELATED_CARRIER) {
                    unrelated_removals.push(ids);
                }
            }
            if packet.id == CSetPassengers::to_id(CURRENT_MC_VERSION) {
                let mut payload = packet.payload.as_ref();
                passenger_updates.push((payload.get_var_int()?, payload.get_var_int()?.0));
            }
            assert!(
                packet.id != CSpawnEntity::to_id(CURRENT_MC_VERSION)
                    || !packet
                        .payload
                        .as_ref()
                        .get_var_int()
                        .is_ok_and(|actor| actor.0 == id)
            );
        }
        assert_eq!(passenger_updates, [(UNRELATED_CARRIER, 0)]);
        assert_eq!(unrelated_removals, [vec![UNRELATED_CARRIER]]);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_does_not_queue_a_passenger_removed_by_its_observer() -> TestResult {
        use crate::entity::vehicle::minecart::MinecartEntity;

        struct DismountOnLink(Arc<MinecartEntity>, i32, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for DismountOnLink {
            fn handle<'a>(
                &'a self,
                _server: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    let mut payload = event.payload.as_ref();
                    let cart = self.0.get_entity();
                    if event.packet_id == CSetPassengers::to_id(CURRENT_MC_VERSION)
                        && payload.get_var_int().is_ok_and(|id| id.0 == cart.entity_id)
                        && payload.get_var_int().is_ok_and(|count| count.0 == 1)
                        && payload.get_var_int().is_ok_and(|rider| rider.0 == self.1)
                        && !self.2.swap(true, Relaxed)
                    {
                        cart.remove_passenger_sync(self.1);
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let (rider, _rider_peer) = java_rider(&server, &world, "jeb_")?;
        let client = viewer.client.java().ok_or("Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let cart = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            CENTER,
            &EntityType::MINECART,
        )));
        let entity = cart.get_entity();
        assert!(world.spawn_entity(cart.clone()));
        let tracker = world
            .entity_tracker
            .get_tracked_entity(entity.entity_id)
            .ok_or("Minecart tracker")?;
        let rider_id = rider.entity_id();
        world
            .entity_tracker
            .add_entity(&(rider.clone() as Arc<dyn EntityBase>), &world);
        world
            .entity_tracker
            .get_tracked_entity(rider.entity_id())
            .ok_or("Rider tracker")?
            .add_pairing(&viewer);
        entity.add_passenger_silently(cart.clone(), rider);
        let observer = Arc::new(DismountOnLink(
            cart.clone(),
            rider_id,
            AtomicBool::new(false),
        ));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);

        tracker.add_pairing(&viewer);
        assert!(observer.2.load(Relaxed));
        assert!(!entity.has_passenger(rider_id));
        for packet in queued_packets(client, &mut decoder).await? {
            if packet.id == CSetPassengers::to_id(CURRENT_MC_VERSION) {
                let mut payload = packet.payload.as_ref();
                if payload.get_var_int()?.0 == entity.entity_id {
                    assert_eq!(payload.get_var_int()?.0, 0);
                }
            }
        }
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn passenger_pairing_filters_extras_for_the_removed_rider() -> TestResult {
        use crate::entity::{passive::pig::PigEntity, vehicle::minecart::MinecartEntity};

        const UNRELATED: VarInt = VarInt(98_765);
        struct RemoveRider(Arc<PigEntity>, i32, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for RemoveRider {
            fn handle<'a>(
                &'a self,
                _server: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    let mut payload = event.payload.as_ref();
                    let rider = self.0.get_entity();
                    if event.packet_id == CSetPassengers::to_id(CURRENT_MC_VERSION)
                        && payload.get_var_int().is_ok_and(|id| id.0 == self.1)
                        && payload.get_var_int().is_ok_and(|count| count.0 == 1)
                        && payload
                            .get_var_int()
                            .is_ok_and(|id| id.0 == rider.entity_id)
                        && !self.2.swap(true, Relaxed)
                    {
                        rider.remove();
                        for carrier in [VarInt(rider.entity_id), UNRELATED] {
                            let mut payload = Vec::new();
                            assert!(
                                CSetPassengers::new(carrier, &[VarInt(98_766)])
                                    .write_packet_data(&mut payload, &CURRENT_MC_VERSION)
                                    .is_ok()
                            );
                            event
                                .extra_packets
                                .push((CSetPassengers::to_id(CURRENT_MC_VERSION), payload.into()));
                        }
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let rider = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        let cart = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            CENTER,
            &EntityType::MINECART,
        )));
        assert!(world.spawn_entity(rider.clone()));
        let tracker = world
            .entity_tracker
            .get_tracked_entity(rider.get_entity().entity_id)
            .ok_or("Pig tracker")?;
        assert!(world.spawn_entity(cart.clone()));
        let carrier = cart.get_entity();
        world
            .entity_tracker
            .get_tracked_entity(carrier.entity_id)
            .ok_or("Minecart tracker")?
            .add_pairing(&viewer);
        carrier.add_passenger_silently(cart.clone(), rider.clone());
        let observer = Arc::new(RemoveRider(
            rider.clone(),
            carrier.entity_id,
            AtomicBool::new(false),
        ));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);

        tracker.add_pairing(&viewer);
        assert!(observer.2.load(Relaxed));
        assert!(rider.get_entity().is_removed());
        let mut extras = Vec::new();
        for packet in queued_packets(client, &mut decoder).await? {
            if packet.id == CSetPassengers::to_id(CURRENT_MC_VERSION) {
                let mut payload = packet.payload.as_ref();
                let vehicle = payload.get_var_int()?;
                if payload.get_var_int()?.0 > 0 {
                    extras.push(vehicle);
                }
            }
        }
        assert_eq!(extras, [UNRELATED]);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn java_pairing_retries_a_contended_passenger_snapshot() -> TestResult {
        use crate::entity::vehicle::minecart::MinecartEntity;

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let (rider, _rider_peer) = java_rider(&server, &world, "jeb_")?;
        let cart = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            CENTER,
            &EntityType::MINECART,
        )));
        let entity = cart.get_entity();
        assert!(world.spawn_entity(cart.clone()));
        let tracker = world
            .entity_tracker
            .get_tracked_entity(entity.entity_id)
            .ok_or("Minecart tracker")?;
        world
            .entity_tracker
            .add_entity(&(rider.clone() as Arc<dyn EntityBase>), &world);
        world
            .entity_tracker
            .get_tracked_entity(rider.entity_id())
            .ok_or("Rider tracker")?
            .add_pairing(&viewer);
        entity.add_passenger_silently(cart.clone(), rider);
        let retry_pending = {
            let _passengers = entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            tracker.add_pairing(&viewer);
            tracker.passenger_retry_pending.load(Relaxed)
        };
        assert!(retry_pending);
        tracker.flush_pairing_passengers();

        assert!(
            queued_packets(client, &mut decoder)
                .await?
                .iter()
                .any(|packet| packet.id == CSetPassengers::to_id(CURRENT_MC_VERSION))
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn bedrock_pairing_clears_a_hand_changed_during_the_next_packet() -> TestResult {
        use crate::entity::mob::Mob;
        use crate::entity::passive::pig::PigEntity;
        use crate::world::item_inventory_tests::bedrock_player;
        use pumpkin_data::item::Item;
        use pumpkin_protocol::bedrock::server::inventory_transaction::WINDOW_ID_OFF_HAND;

        struct ClearMainHand(Arc<PigEntity>, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for ClearMainHand {
            fn handle<'a>(
                &'a self,
                _server: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    let mut payload = event.payload.as_ref();
                    if event.packet_id == CMobEquipment::PACKET_ID
                        && VarULong::read(&mut payload)
                            .is_ok_and(|id| id.0 == self.0.get_entity().entity_id as u64)
                        && NetworkItemStackDescriptor::read(&mut payload).is_ok()
                        && u8::read(&mut payload).is_ok()
                        && u8::read(&mut payload).is_ok()
                        && u8::read(&mut payload).is_ok_and(|id| id == WINDOW_ID_OFF_HAND as u8)
                        && !self.1.swap(true, Relaxed)
                    {
                        self.0
                            .get_mob_entity()
                            .living_entity
                            .entity_equipment
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .equipment
                            .remove(&EquipmentSlot::MAIN_HAND);
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, mut packets) = bedrock_player(&server, &world, "Notch").await?;
        let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        let mob = pig.get_mob_entity();
        mob.set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, &Item::SHIELD));
        mob.set_item_slot(&EquipmentSlot::OFF_HAND, ItemStack::new(1, &Item::STICK));
        assert!(world.spawn_entity(pig.clone()));
        let id = pig.get_entity().entity_id;
        let tracker = world
            .entity_tracker
            .get_tracked_entity(id)
            .ok_or("Pig tracker")?;
        let observer = Arc::new(ClearMainHand(pig, AtomicBool::new(false)));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        while packets.try_recv().is_ok() {}
        tracker.add_pairing(&viewer);
        assert!(observer.1.load(Relaxed));
        let mut main_hand = Vec::new();
        while let Ok(packet) = packets.try_recv() {
            if let Some((packet_id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
                && i32::from(packet_id) == CMobEquipment::PACKET_ID
            {
                let mut payload = payload.as_ref();
                if VarULong::read(&mut payload)?.0 != id as u64 {
                    continue;
                }
                let item = NetworkItemStackDescriptor::read(&mut payload)?;
                let _slot = u8::read(&mut payload)?;
                let _selected = u8::read(&mut payload)?;
                if u8::read(&mut payload)? == 0 {
                    main_hand.push(item.id == 0);
                }
            }
        }
        assert_eq!(main_hand, [false, true]);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn bedrock_pairing_resends_the_surviving_rider_in_the_front_seat() -> TestResult {
        use crate::entity::vehicle::minecart::MinecartEntity;
        use crate::world::item_inventory_tests::bedrock_player;
        use pumpkin_protocol::bedrock::client::CSetActorLink;

        struct DismountFront(Arc<MinecartEntity>, i32, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for DismountFront {
            fn handle<'a>(
                &'a self,
                _server: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    let mut payload = event.payload.as_ref();
                    if event.packet_id == CSetActorLink::PACKET_ID
                        && VarLong::read(&mut payload)
                            .is_ok_and(|id| id.0 == i64::from(self.0.get_entity().entity_id))
                        && VarLong::read(&mut payload).is_ok_and(|id| id.0 == i64::from(self.1))
                        && u8::read(&mut payload).is_ok_and(|kind| kind == 1)
                        && !self.2.swap(true, Relaxed)
                    {
                        self.0.get_entity().remove_passenger_sync(self.1);
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, mut packets) = bedrock_player(&server, &world, "Notch").await?;
        viewer.watched_section.store(Cylindrical::new(
            Vector2::new(32, 32),
            std::num::NonZeroU8::MIN,
        ));
        let (front, _front_peer) = java_rider(&server, &world, "jeb_")?;
        let (back, _back_peer) = java_rider(&server, &world, "deadmau5")?;
        let cart = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            CENTER,
            &EntityType::MINECART,
        )));
        let entity = cart.get_entity();
        assert!(world.spawn_entity(cart.clone()));
        let tracker = world
            .entity_tracker
            .get_tracked_entity(entity.entity_id)
            .ok_or("Minecart tracker")?;
        let front_id = front.entity_id();
        let back_id = back.entity_id();
        for rider in [&front, &back] {
            world
                .entity_tracker
                .add_entity(&(rider.clone() as Arc<dyn EntityBase>), &world);
            world
                .entity_tracker
                .get_tracked_entity(rider.entity_id())
                .ok_or("Rider tracker")?
                .add_pairing(&viewer);
        }
        entity.add_passenger_silently(cart.clone(), front);
        entity.add_passenger_silently(cart.clone(), back);
        let observer = Arc::new(DismountFront(
            cart.clone(),
            front_id,
            AtomicBool::new(false),
        ));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        while packets.try_recv().is_ok() {}

        let passengers = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        tracker.add_pairing(&viewer);
        assert!(!observer.2.load(Relaxed));
        drop(passengers);
        tracker.flush_pairing_passengers();
        assert!(observer.2.load(Relaxed));
        assert!(!entity.has_passenger(front_id));
        let mut links = Vec::new();
        while let Ok(packet) = packets.try_recv() {
            if let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
                && i32::from(id) == CSetActorLink::PACKET_ID
            {
                let mut payload = payload.as_ref();
                let carrier = VarLong::read(&mut payload)?.0;
                let rider = VarLong::read(&mut payload)?.0;
                let kind = u8::read(&mut payload)?;
                if carrier == i64::from(entity.entity_id) && kind != 0 {
                    links.push((rider, kind));
                }
            }
        }
        assert_eq!(links, [(i64::from(back_id), 1)]);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn bedrock_player_pairing_drops_embedded_links_before_player_list_observation()
    -> TestResult {
        use crate::world::item_inventory_tests::bedrock_player;
        use pumpkin_protocol::bedrock::client::{CAddPlayer, CPlayerList};

        struct DismountOnList(Arc<Player>, i32, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for DismountOnList {
            fn handle<'a>(
                &'a self,
                _server: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    if event.packet_id == CPlayerList::PACKET_ID && !self.2.swap(true, Relaxed) {
                        self.0.get_entity().remove_passenger_sync(self.1);
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, mut packets) = bedrock_player(&server, &world, "Notch").await?;
        let (carrier, _peer) = java_rider(&server, &world, "jeb_")?;
        let (rider, _rider_peer) = java_rider(&server, &world, "deadmau5")?;
        let client = viewer.client.bedrock().ok_or("Bedrock viewer")?;
        let actor: Arc<dyn EntityBase> = carrier.clone();
        world.entity_tracker.add_entity(&actor, &world);
        let entity = carrier.get_entity();
        let tracker = world
            .entity_tracker
            .get_tracked_entity(entity.entity_id)
            .ok_or("Carrier tracker")?;
        let rider_id = rider.entity_id();
        entity.add_passenger_silently(actor, rider);
        let (_, mut add_player) = carrier.bedrock_spawn_packets();
        assert_eq!(add_player.actor_links.len(), 1);
        let linked_size = client.serialize_packet(&add_player)?.len();
        add_player.actor_links.clear();
        let unlinked_size = client.serialize_packet(&add_player)?.len();
        assert!(unlinked_size < linked_size);
        while packets.try_recv().is_ok() {}
        let observer = Arc::new(DismountOnList(
            carrier.clone(),
            rider_id,
            AtomicBool::new(false),
        ));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);

        tracker.add_pairing(&viewer);
        assert!(observer.2.load(Relaxed));
        assert!(!entity.has_passenger(rider_id));
        let mut spawns = Vec::new();
        while let Ok(packet) = packets.try_recv() {
            if crate::net::bedrock::split_raw_packet(&packet.data)
                .is_some_and(|(id, _)| i32::from(id) == CAddPlayer::PACKET_ID)
            {
                spawns.push(packet.data.len());
            }
        }
        // Metadata fields can be serialized in different orders. The embedded
        // link is the only field removed between the two reference packets.
        assert_eq!(spawns, [unlinked_size]);
        client.clear_player();
        client.close().await;
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn player_keeps_equipment_selected_by_the_observer() -> TestResult {
        use pumpkin_data::{attributes::Attributes, item::Item};
        use pumpkin_protocol::java::server::play::SSetHeldItem;

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (rider, _rider_peer) = java_rider(&server, &world, "Notch")?;
        let (viewer, peer) = java_rider(&server, &world, "jeb_")?;
        let client = viewer.client.java().ok_or("Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let rider_actor: Arc<dyn EntityBase> = rider.clone();
        world.entity_tracker.add_entity(&rider_actor, &world);
        let rider_id = rider.get_entity().entity_id;
        let tracked = world
            .entity_tracker
            .get_tracked_entity(rider_id)
            .ok_or("Rider tracker")?;
        tracked.add_pairing(&viewer);
        let _initial = queued_packets(client, &mut decoder).await?;
        rider
            .inventory()
            .set_slot(1, ItemStack::new(1, &Item::DIAMOND_SWORD));
        rider
            .inventory()
            .set_slot(2, ItemStack::new(1, &Item::STICK));
        let listener = Arc::new(SelectStick {
            rider: rider.clone(),
            called: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(listener.clone(), EventPriority::Normal, None);
        rider
            .client
            .java()
            .ok_or("Java rider")?
            .handle_set_held_item(&server, &rider, &SSetHeldItem { slot: 1 });
        assert!(listener.called.load(Relaxed));
        assert_eq!(rider.inventory().held_item().item, &Item::STICK);
        assert_eq!(
            rider
                .living_entity
                .get_attribute_value(&Attributes::ATTACK_DAMAGE)
                .to_bits(),
            rider
                .living_entity
                .get_attribute_base(&Attributes::ATTACK_DAMAGE)
                .to_bits()
        );
        let mut hand_items = Vec::new();
        for packet in queued_packets(client, &mut decoder).await? {
            if packet.id == CSetEquipment::to_id(CURRENT_MC_VERSION) {
                let packet =
                    CSetEquipment::read(&mut packet.payload.as_ref(), &CURRENT_MC_VERSION)?;
                if packet.entity_id.0 == rider_id {
                    hand_items.extend(
                        packet
                            .equipment
                            .into_iter()
                            .filter(|(slot, _)| *slot == EquipmentSlot::MAIN_HAND.discriminant())
                            .map(|(_, stack)| stack.0.item),
                    );
                }
            }
        }
        assert_eq!(hand_items, [&Item::STICK]);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_resends_equipment_after_initial_contention() -> TestResult {
        use crate::entity::{mob::Mob, passive::pig::PigEntity};
        use pumpkin_data::item::Item;

        struct ChangeViewerHand(i32, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for ChangeViewerHand {
            fn handle<'a>(
                &'a self,
                _server: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    use pumpkin_data::item::Item;
                    if event.packet_id == CSetEquipment::to_id(CURRENT_MC_VERSION)
                        && let Ok(mut packet) =
                            CSetEquipment::read(&mut event.payload.as_ref(), &CURRENT_MC_VERSION)
                        && packet.entity_id.0 == self.0
                        && !self.1.swap(true, Relaxed)
                    {
                        packet.equipment[0].1 =
                            ItemStackSerializer::from(ItemStack::new(1, &Item::CARROT));
                        let mut payload = Vec::new();
                        assert!(
                            packet
                                .write_packet_data(&mut payload, &CURRENT_MC_VERSION)
                                .is_ok()
                        );
                        event.cancelled = true;
                        event.extra_packets.push((event.packet_id, payload.into()));
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Missing Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        let mob = pig.get_mob_entity();
        let id = pig.get_entity().entity_id;
        mob.set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, &Item::STICK));
        world.spawn_entity(pig.clone());
        let tracked = world
            .entity_tracker
            .get_tracked_entity(id)
            .ok_or("Pig tracker")?;
        let session = tracked.insert_viewer(&viewer).ok_or("New viewer")?;
        let equipment = mob
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        tracked.send_pairing_equipment(&mob.living_entity, &viewer, session);
        drop(equipment);
        assert!(tracked.equipment_retry_pending.load(Relaxed));
        assert!(
            queued_packets(client, &mut decoder)
                .await?
                .iter()
                .all(|packet| packet.id != CSetEquipment::to_id(CURRENT_MC_VERSION))
        );

        let observer = Arc::new(ChangeViewerHand(id, AtomicBool::new(false)));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        mob.set_item_slot(
            &EquipmentSlot::MAIN_HAND,
            ItemStack::new(1, &Item::DIAMOND_SWORD),
        );
        assert!(observer.1.load(Relaxed));
        world.entity_tracker.update_all(&world);
        let packets = queued_packets(client, &mut decoder).await?;
        let mut equipment = packets
            .iter()
            .filter(|packet| packet.id == CSetEquipment::to_id(CURRENT_MC_VERSION));
        let update = equipment.next().ok_or("Viewer equipment packet")?;
        let packet = CSetEquipment::read(&mut update.payload.as_ref(), &CURRENT_MC_VERSION)?;
        assert_eq!(packet.entity_id.0, id);
        assert!(packet.equipment.iter().any(|(slot, stack)| *slot
            == EquipmentSlot::MAIN_HAND.discriminant()
            && stack.0.item == &Item::CARROT));
        assert!(equipment.next().is_none());
        world.level.shutdown().await;
        Ok(())
    }

    enum CartObserverAction {
        Remove,
        Teleport(Vector3<f64>),
    }

    struct MoveCartFromVelocity {
        cart: Arc<MinecartEntity>,
        action: CartObserverAction,
        ran: AtomicBool,
    }

    impl EventRwHandler<PacketSentEvent> for MoveCartFromVelocity {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                let cart = self.cart.get_entity();
                let mut payload = event.payload.as_ref();
                if event.packet_id != CEntityVelocity::to_id(CURRENT_MC_VERSION)
                    || !payload.get_var_int().is_ok_and(|id| id.0 == cart.entity_id)
                    || self.ran.swap(true, Relaxed)
                {
                    return;
                }
                match self.action {
                    CartObserverAction::Remove => cart.remove(),
                    CartObserverAction::Teleport(destination) => {
                        cart.teleport(destination, None, None, &cart.world.load());
                    }
                }
                event
                    .extra_packets
                    .push((event.packet_id, event.payload.clone()));
            })
        }
    }

    struct SupersedeCartTeleport {
        cart: Arc<MinecartEntity>,
        destination: Vector3<f64>,
        ran: AtomicBool,
    }

    impl EventRwHandler<PacketSentEvent> for SupersedeCartTeleport {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                let cart = self.cart.get_entity();
                let mut payload = event.payload.as_ref();
                if event.packet_id != CEntityPositionSync::to_id(CURRENT_MC_VERSION)
                    || !payload.get_var_int().is_ok_and(|id| id.0 == cart.entity_id)
                    || self.ran.swap(true, Relaxed)
                {
                    return;
                }
                cart.teleport(self.destination, None, None, &cart.world.load());
                event
                    .extra_packets
                    .push((event.packet_id, event.payload.clone()));
            })
        }
    }

    struct DismountCartFromPairing {
        cart: Arc<MinecartEntity>,
        rider_id: i32,
        viewer_id: Uuid,
        ran: AtomicBool,
    }

    impl EventRwHandler<PacketSentEvent> for DismountCartFromPairing {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                let cart = self.cart.get_entity();
                let mut payload = event.payload.as_ref();
                if event.player.gameprofile.id != self.viewer_id
                    || event.packet_id != CSetPassengers::to_id(CURRENT_MC_VERSION)
                    || !payload.get_var_int().is_ok_and(|id| id.0 == cart.entity_id)
                    || !payload.get_var_int().is_ok_and(|count| count.0 == 1)
                    || !payload.get_var_int().is_ok_and(|id| id.0 == self.rider_id)
                    || self.ran.swap(true, Relaxed)
                {
                    return;
                }
                cart.remove_passenger(self.rider_id);
                event
                    .extra_packets
                    .push((event.packet_id, event.payload.clone()));
            })
        }
    }

    fn watched_cart(
        world: &Arc<World>,
        viewer: &Arc<Player>,
    ) -> Result<(Arc<MinecartEntity>, Arc<TrackedEntity>), Box<dyn std::error::Error>> {
        let cart = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            CENTER,
            &EntityType::MINECART,
        )));
        let entity = cart.get_entity();
        world.add_entity_silent(cart.clone());
        let tracked = world
            .entity_tracker
            .get_tracked_entity(entity.entity_id)
            .ok_or("Missing tracked cart")?;
        assert!(tracked.insert_viewer(viewer).is_some());
        viewer.watched_section.store(Cylindrical::new(
            entity.chunk_pos.load(),
            std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
        ));
        entity.set_pos(CENTER.add_raw(0.125, 0.0, 0.0));
        entity.velocity.store(Vector3::new(0.2, 0.0, 0.0));
        entity.velocity_dirty.store(true, Relaxed);
        assert!(matches!(
            tracked.pick_move(
                &TrackingUpdate {
                    entity,
                    world,
                    position: entity.pos.load(),
                    teleport_epoch: entity.vehicle_motion_epoch(),
                },
                0,
                false,
                entity.on_ground.load(Relaxed),
                1,
            ),
            Some(MoveSync::Pos(_)),
        ));
        Ok((cart, tracked))
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn tracked_cart_removal_from_velocity_observer_sends_only_despawn() -> TestResult {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Missing Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let (cart, tracked) = watched_cart(&world, &viewer)?;
        let listener = Arc::new(MoveCartFromVelocity {
            cart: cart.clone(),
            action: CartObserverAction::Remove,
            ran: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(listener.clone(), EventPriority::Normal, None);
        let (finished, done) = tokio::sync::oneshot::channel();
        let tracking_world = world.clone();
        let tracker = std::thread::spawn(move || {
            tracked.send_changes(&tracking_world);
            let _ = finished.send(());
        });
        tokio::time::timeout(Duration::from_secs(5), done)
            .await
            .map_err(|_| "Tracking blocked while its velocity observer removed the cart")??;
        tracker.join().map_err(|_| "Cart tracker panicked")?;
        assert!(listener.ran.load(Relaxed));
        let entity = cart.get_entity();
        assert!(entity.is_removed());
        assert!(!world.entity_tracker.has_entity_with_id(entity.entity_id));
        assert_eq!(
            entity.last_sent_velocity.load(),
            Vector3::new(0.0, 0.0, 0.0)
        );
        let packets = queued_packets(client, &mut decoder).await?;
        let mut expected = Vec::new();
        CRemoveEntities::new(&[entity.entity_id.into()])
            .write_packet_data(&mut expected, &CURRENT_MC_VERSION)?;
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].id, CRemoveEntities::to_id(CURRENT_MC_VERSION));
        assert_eq!(packets[0].payload.as_ref(), expected);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn tracked_cart_teleport_from_velocity_observer_keeps_winning_position() -> TestResult {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Missing Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let (cart, tracked) = watched_cart(&world, &viewer)?;
        let destination = CENTER.add_raw(3.0, 1.0, 0.0);
        let listener = Arc::new(MoveCartFromVelocity {
            cart: cart.clone(),
            action: CartObserverAction::Teleport(CENTER.add_raw(2.0, 1.0, 0.0)),
            ran: AtomicBool::new(false),
        });
        let superseding = Arc::new(SupersedeCartTeleport {
            cart: cart.clone(),
            destination,
            ran: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(listener.clone(), EventPriority::Normal, None);
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(
                superseding.clone(),
                EventPriority::Normal,
                None,
            );
        tracked.send_changes(&world);
        assert!(listener.ran.load(Relaxed));
        assert!(superseding.ran.load(Relaxed));
        let entity = cart.get_entity();
        let packets = queued_packets(client, &mut decoder).await?;
        assert_eq!(packets.len(), 1);
        assert_eq!(
            packets[0].id,
            CEntityPositionSync::to_id(CURRENT_MC_VERSION)
        );
        let mut payload = packets[0].payload.as_ref();
        assert_eq!(payload.get_var_int()?.0, entity.entity_id);
        assert_eq!(payload.get_var_int()?.0, 0);
        assert_eq!(
            Vector3::new(
                payload.get_f64_be()?,
                payload.get_f64_be()?,
                payload.get_f64_be()?
            ),
            destination,
        );
        assert_eq!(entity.pos.load(), destination);
        assert_eq!(entity.last_sent_pos.load(), destination);
        assert_eq!(tracked.teleport_delay.load(Relaxed), 0);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_cart_and_rider_allows_observer_dismount() -> TestResult {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Missing Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let (rider, _rider_peer) = java_rider(&server, &world, "jeb_")?;
        let cart = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            CENTER,
            &EntityType::MINECART,
        )));
        let entity = cart.get_entity();
        world.add_entity_silent(cart.clone());
        let cart_tracker = world
            .entity_tracker
            .get_tracked_entity(entity.entity_id)
            .ok_or("Missing tracked cart")?;
        let rider_id = rider.entity_id();
        world
            .entity_tracker
            .add_entity(&(rider.clone() as Arc<dyn EntityBase>), &world);
        let rider_tracker = world
            .entity_tracker
            .get_tracked_entity(rider_id)
            .ok_or("Missing tracked rider")?;
        viewer.watched_section.store(Cylindrical::new(
            entity.chunk_pos.load(),
            std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
        ));
        let chunk = entity.chunk_pos.load();
        assert!(
            viewer
                .watched_section
                .load()
                .is_within_distance(chunk.x, chunk.y)
        );
        let listener = Arc::new(DismountCartFromPairing {
            cart: cart.clone(),
            rider_id,
            viewer_id: viewer.gameprofile.id,
            ran: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(listener.clone(), EventPriority::Normal, None);

        for (tracker, counterpart) in [
            (cart_tracker.clone(), &rider_tracker),
            (rider_tracker.clone(), &cart_tracker),
        ] {
            tracker.remove_pairing(&viewer);
            counterpart.add_pairing(&viewer);
            entity.add_passenger_silently(cart.clone(), rider.clone());
            assert!(entity.has_passenger(rider_id));
            listener.ran.store(false, Relaxed);
            let (finished, done) = tokio::sync::oneshot::channel();
            let recipient = viewer.clone();
            let pairing = std::thread::spawn(move || {
                tracker.add_pairing(&recipient);
                let _ = finished.send(());
            });
            tokio::time::timeout(Duration::from_secs(5), done)
                .await
                .map_err(
                    |_| "Pairing blocked while its passenger observer dismounted the rider",
                )??;
            pairing.join().map_err(|_| "Cart pairing panicked")?;
            assert!(listener.ran.load(Relaxed));
            assert!(!entity.has_passenger(rider_id));
            assert!(!rider.get_entity().has_vehicle());
            let mut lists = Vec::new();
            for packet in queued_packets(client, &mut decoder).await? {
                if packet.id == CSetPassengers::to_id(CURRENT_MC_VERSION) {
                    let mut payload = packet.payload.as_ref();
                    if payload.get_var_int()?.0 == entity.entity_id {
                        lists.push(payload.get_var_int()?.0);
                    }
                }
            }
            assert_eq!(lists, [0]);
        }
        world.level.shutdown().await;
        Ok(())
    }

    struct ObservePassengerExtra {
        carrier_id: i32,
        payload: Bytes,
        unpair: Option<Arc<TrackedEntity>>,
        called: AtomicBool,
    }

    impl EventRwHandler<PacketSentEvent> for ObservePassengerExtra {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CSetPassengers::to_id(CURRENT_MC_VERSION)
                    && event
                        .payload
                        .as_ref()
                        .get_var_int()
                        .is_ok_and(|id| id.0 == self.carrier_id)
                    && !self.called.swap(true, Relaxed)
                {
                    if let Some(counterpart) = &self.unpair {
                        counterpart.remove_pairing(&event.player);
                    }
                    event
                        .extra_packets
                        .push((event.packet_id, self.payload.clone()));
                }
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn current_passenger_update_keeps_plugin_extra() -> TestResult {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let cart = MinecartEntity::new(Entity::new(world.clone(), CENTER, &EntityType::MINECART));
        let carrier_id = cart.get_entity().entity_id;
        let fake = VarInt(999_999);
        let mut payload = Vec::new();
        CSetPassengers::new(VarInt(carrier_id), &[fake])
            .write_packet_data(&mut payload, &CURRENT_MC_VERSION)?;
        let observer = Arc::new(ObservePassengerExtra {
            carrier_id,
            payload: payload.into(),
            unpair: None,
            called: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        cart.get_entity()
            .send_passenger_update_to(&world, &viewer, &[], &[]);
        let mut lists = Vec::new();
        for packet in queued_packets(client, &mut decoder).await? {
            let mut data = packet.payload.as_ref();
            if packet.id == CSetPassengers::to_id(CURRENT_MC_VERSION)
                && data.get_var_int()?.0 == carrier_id
            {
                let count = data.get_var_int()?.0;
                let ids = (0..count)
                    .map(|_| data.get_var_int())
                    .collect::<Result<Vec<_>, _>>()?;
                lists.push(ids);
            }
        }
        assert!(observer.called.load(Relaxed));
        assert_eq!(lists, [vec![], vec![fake]]);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_drops_passenger_update_when_other_endpoint_is_unpaired() -> TestResult {
        use crate::entity::passive::pig::PigEntity;

        for pair_rider in [false, true] {
            let folder = tempfile::tempdir()?;
            let (server, world) = vehicle_world(&folder).await?;
            let (viewer, peer) = java_rider(&server, &world, "Notch")?;
            let client = viewer.client.java().ok_or("Java viewer")?;
            client.player.store(Arc::new(Some(viewer.clone())));
            let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
            let cart = Arc::new(MinecartEntity::new(Entity::new(
                world.clone(),
                CENTER,
                &EntityType::MINECART,
            )));
            let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
            world.add_entity_silent(cart.clone());
            world.add_entity_silent(pig.clone());
            let (carrier_id, rider_id) = (cart.get_entity().entity_id, pig.get_entity().entity_id);
            let cart_tracker = world
                .entity_tracker
                .get_tracked_entity(carrier_id)
                .ok_or("Cart tracker")?;
            let pig_tracker = world
                .entity_tracker
                .get_tracked_entity(rider_id)
                .ok_or("Pig tracker")?;
            let (next, counterpart) = if pair_rider {
                (pig_tracker, cart_tracker)
            } else {
                (cart_tracker, pig_tracker)
            };
            counterpart.add_pairing(&viewer);
            cart.get_entity().add_passenger_silently(cart.clone(), pig);
            let fake_carrier = VarInt(999_999);
            let mut payload = Vec::new();
            CSetPassengers::new(fake_carrier, &[VarInt(rider_id)])
                .write_packet_data(&mut payload, &CURRENT_MC_VERSION)?;
            let observer = Arc::new(ObservePassengerExtra {
                carrier_id,
                payload: payload.into(),
                unpair: Some(counterpart.clone()),
                called: AtomicBool::new(false),
            });
            server
                .plugin_manager
                .register_blocking::<PacketSentEvent, _>(
                    observer.clone(),
                    EventPriority::Normal,
                    None,
                );
            next.add_pairing(&viewer);
            let packets = queued_packets(client, &mut decoder).await?;
            assert!(observer.called.load(Relaxed));
            assert!(next.is_seen_by(&viewer.gameprofile.id));
            assert!(!counterpart.is_seen_by(&viewer.gameprofile.id));
            assert!(cart.get_entity().has_passenger(rider_id));
            assert!(
                packets
                    .iter()
                    .any(|packet| packet.id == CRemoveEntities::to_id(CURRENT_MC_VERSION))
            );
            assert!(
                !packets
                    .iter()
                    .any(|packet| packet.id == CSetPassengers::to_id(CURRENT_MC_VERSION))
            );
            world.level.shutdown().await;
        }
        Ok(())
    }

    struct AfterPassengerPublication<F> {
        carrier_id: i32,
        action: F,
        remaining: AtomicUsize,
    }

    impl<F: Fn(&mut PacketSentEvent, usize) + Send + Sync> EventRwHandler<PacketSentEvent>
        for AfterPassengerPublication<F>
    {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CSetPassengers::to_id(CURRENT_MC_VERSION)
                    && event
                        .payload
                        .as_ref()
                        .get_var_int()
                        .is_ok_and(|id| id.0 == self.carrier_id)
                    && let Ok(remaining) =
                        self.remaining
                            .fetch_update(Relaxed, Relaxed, |remaining| remaining.checked_sub(1))
                {
                    (self.action)(event, remaining - 1);
                }
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_keeps_newly_visible_rider_in_passenger_projection() -> TestResult {
        assert_passenger_projection_after_visibility_change(PassengerVisibilityChange::Reveal, 2)
            .await
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_keeps_remaining_rider_in_passenger_projection() -> TestResult {
        assert_passenger_projection_after_visibility_change(PassengerVisibilityChange::Hide, 2)
            .await
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_retries_after_two_successive_rider_departures() -> TestResult {
        // Forced mounts can put a third rider in a classic cart.
        assert_passenger_projection_after_visibility_change(PassengerVisibilityChange::Hide, 3)
            .await
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_does_not_retry_with_a_new_visible_rider() -> TestResult {
        assert_passenger_projection_after_visibility_change(PassengerVisibilityChange::Replace, 3)
            .await
    }

    enum PassengerVisibilityChange {
        Reveal,
        Hide,
        Replace,
    }

    async fn assert_passenger_projection_after_visibility_change(
        change: PassengerVisibilityChange,
        rider_count: usize,
    ) -> TestResult {
        use crate::entity::passive::pig::PigEntity;

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let carrier: Arc<dyn EntityBase> = if rider_count > 2 {
            Arc::new(MinecartEntity::new(Entity::new(
                world.clone(),
                CENTER,
                &EntityType::MINECART,
            )))
        } else {
            Arc::new(BoatEntity::new(Entity::new(
                world.clone(),
                CENTER,
                &EntityType::OAK_BOAT,
            )))
        };
        world.add_entity_silent(carrier.clone());
        let carrier_id = carrier.get_entity().entity_id;
        let tracker = &world.entity_tracker;
        let tracked = |id| {
            tracker
                .get_tracked_entity(id)
                .ok_or("Missing tracked entity")
        };
        let mut rider_ids = Vec::new();
        let mut changing_riders = Vec::new();
        let reveals_rider = matches!(change, PassengerVisibilityChange::Reveal);
        for index in 0..rider_count {
            let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
            let rider_id = pig.get_entity().entity_id;
            world.add_entity_silent(pig.clone());
            let tracker = tracked(rider_id)?;
            let visible = index == 0 || !reveals_rider;
            if visible {
                tracker.add_pairing(&viewer);
            }
            assert_eq!(tracker.is_seen_by(&viewer.gameprofile.id), visible);
            carrier
                .get_entity()
                .add_passenger_silently(carrier.clone(), pig);
            rider_ids.push(rider_id);
            if index != 0 {
                changing_riders.push(tracker);
            }
        }
        let replacement = matches!(change, PassengerVisibilityChange::Replace)
            .then(|| pair_projection_replacement(&world, &viewer, carrier.get_entity()))
            .transpose()?;
        let replacement_id = replacement.as_ref().map(|pig| pig.get_entity().entity_id);
        let replacement_carrier = carrier.clone();
        let observer = Arc::new(AfterPassengerPublication {
            carrier_id,
            action: move |event: &mut PacketSentEvent, index: usize| {
                if let Some(replacement) = &replacement {
                    for tracker in &changing_riders {
                        tracker.remove_pairing(&event.player);
                    }
                    replacement_carrier
                        .get_entity()
                        .add_passenger(replacement_carrier.clone(), replacement.clone());
                } else if reveals_rider {
                    changing_riders[index].add_pairing(&event.player);
                } else {
                    changing_riders[index].remove_pairing(&event.player);
                }
            },
            remaining: AtomicUsize::new(if replacement_id.is_some() {
                1
            } else {
                rider_count - 1
            }),
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        tracked(carrier_id)?.add_pairing(&viewer);
        assert_eq!(observer.remaining.load(Relaxed), 0);
        assert_passenger_projection_packets(
            queued_packets(client, &mut decoder).await?,
            carrier_id,
            rider_ids,
            reveals_rider,
            replacement_id,
        )?;
        world.level.shutdown().await;
        Ok(())
    }

    fn pair_projection_replacement(
        world: &Arc<World>,
        viewer: &Arc<Player>,
        carrier: &Entity,
    ) -> Result<Arc<crate::entity::passive::pig::PigEntity>, Box<dyn std::error::Error>> {
        use crate::entity::passive::pig::PigEntity;

        let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        world.add_entity_silent(pig.clone());
        world
            .entity_tracker
            .get_tracked_entity(pig.get_entity().entity_id)
            .ok_or("Missing replacement tracker")?
            .add_pairing(viewer);
        viewer.watched_section.store(Cylindrical::new(
            carrier.chunk_pos.load(),
            std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
        ));
        Ok(pig)
    }

    fn assert_passenger_projection_packets(
        packets: Vec<RawPacket>,
        carrier_id: i32,
        mut rider_ids: Vec<i32>,
        reveals_rider: bool,
        replacement_id: Option<i32>,
    ) -> TestResult {
        let mut lists = Vec::new();
        let mut removed = Vec::new();
        for packet in packets {
            let mut payload = packet.payload.as_ref();
            if packet.id == CSetPassengers::to_id(CURRENT_MC_VERSION)
                && payload.get_var_int()?.0 == carrier_id
            {
                let count = payload.get_var_int()?.0;
                lists.push(
                    (0..count)
                        .map(|_| payload.get_var_int().map(|id| id.0))
                        .collect::<Result<Vec<_>, _>>()?,
                );
            } else if packet.id == CRemoveEntities::to_id(CURRENT_MC_VERSION) {
                assert_eq!(payload.get_var_int()?.0, 1);
                removed.push(payload.get_var_int()?.0);
            }
        }
        if let Some(replacement_id) = replacement_id {
            assert_eq!(removed, rider_ids[1..]);
            // The forced mount broadcasts every attached rider, including the unpaired ones.
            rider_ids.push(replacement_id);
            assert_eq!(lists, [rider_ids]);
        } else if reveals_rider {
            assert!(removed.is_empty());
            assert_eq!(lists, [rider_ids]);
        } else {
            rider_ids.reverse();
            let (retained, departed) = rider_ids.split_last().ok_or("No initial riders")?;
            assert_eq!(removed, departed);
            assert_eq!(lists, [vec![*retained]]);
        }
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn superseded_unlink_drops_extra_for_departed_rider() -> TestResult {
        use crate::entity::passive::pig::PigEntity;

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let cart = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            CENTER,
            &EntityType::MINECART,
        )));
        world.add_entity_silent(cart.clone());
        let departed = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        let replacement = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        world.add_entity_silent(departed.clone());
        world.add_entity_silent(replacement.clone());
        let departed_id = departed.get_entity().entity_id;
        let carrier_id = cart.get_entity().entity_id;
        viewer.watched_section.store(Cylindrical::new(
            cart.get_entity().chunk_pos.load(),
            std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
        ));
        cart.get_entity()
            .add_passenger_silently(cart.clone(), departed);
        let mut extra = Vec::new();
        CSetPassengers::new(VarInt(999_999), &[VarInt(departed_id)])
            .write_packet_data(&mut extra, &CURRENT_MC_VERSION)?;
        let extra: Bytes = extra.into();
        let attached = cart.clone();
        let observer = Arc::new(AfterPassengerPublication {
            carrier_id,
            action: move |event: &mut PacketSentEvent, _: usize| {
                attached
                    .get_entity()
                    .add_passenger_silently(attached.clone(), replacement.clone());
                event.extra_packets.push((event.packet_id, extra.clone()));
            },
            remaining: AtomicUsize::new(1),
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        cart.get_entity()
            .remove_passenger_on_disconnect(departed_id);
        let packets = queued_packets(client, &mut decoder).await?;
        assert_eq!(observer.remaining.load(Relaxed), 0);
        assert!(
            !packets
                .iter()
                .any(|packet| packet.id == CSetPassengers::to_id(CURRENT_MC_VERSION))
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn boat_position_sync_sends_impact_and_advances_tracking_interval() -> TestResult {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Missing Java viewer")?;
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let boat_type = &EntityType::OAK_BOAT;
        let boat = Arc::new(BoatEntity::new(Entity::new(
            world.clone(),
            CENTER.add_raw(0.0, 0.125, 0.0),
            boat_type,
        )));
        let entity = boat.get_entity();
        let tracked = TrackedEntity::new(
            boat.clone(),
            boat_type.client_tracking_range,
            boat_type.update_interval,
            boat_type.track_deltas,
        );
        assert!(tracked.insert_viewer(&viewer).is_some());
        assert_eq!(tracked.update_interval, 3);
        assert!(
            !tracking_frames(&tracked, &world, client, &mut decoder)
                .await?
                .is_empty()
        );
        assert_eq!(tracked.tick_count.load(Relaxed), 1);
        assert!(!entity.synched_data.is_dirty());
        assert!(!entity.velocity_dirty.load(Relaxed));

        // Impact at a non-due tick, with the movement and rebound supplied by physics.
        let rebound = Vector3::new(0.0, 0.208_666_669_776, 0.0);
        entity.set_pos(CENTER);
        entity.velocity.store(rebound);
        entity.on_ground.store(true, Relaxed);
        entity.sync_position.store(true, Relaxed);
        let impact = tracking_frames(&tracked, &world, client, &mut decoder).await?;
        assert_eq!(
            impact.iter().map(|packet| packet.id).collect::<Vec<_>>(),
            [
                CEntityVelocity::to_id(CURRENT_MC_VERSION),
                CEntityPositionSync::to_id(CURRENT_MC_VERSION),
            ]
        );
        let mut expected = Vec::new();
        CEntityVelocity::new(entity.entity_id.into(), rebound)
            .write_packet_data(&mut expected, &CURRENT_MC_VERSION)?;
        assert_eq!(impact[0].payload.as_ref(), expected);
        expected.clear();
        CEntityPositionSync::new(entity.entity_id.into(), CENTER, rebound, 0.0, 0.0, true)
            .write_packet_data(&mut expected, &CURRENT_MC_VERSION)?;
        assert_eq!(impact[1].payload.as_ref(), expected);
        assert_eq!(tracked.tick_count.load(Relaxed), 4);
        assert!(!entity.sync_position.load(Relaxed));

        entity.set_pos(CENTER + rebound);
        entity.on_ground.store(false, Relaxed);
        for _ in 0..2 {
            assert!(
                tracking_frames(&tracked, &world, client, &mut decoder)
                    .await?
                    .is_empty()
            );
        }
        let next = tracking_frames(&tracked, &world, client, &mut decoder).await?;
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].id, CEntityPositionSync::to_id(CURRENT_MC_VERSION));
        assert_eq!(tracked.tick_count.load(Relaxed), 7);

        // A sync on an already-due tick still moves to the next interval.
        tracked.tick_count.store(9, Relaxed);
        entity.sync_position.store(true, Relaxed);
        assert!(
            tracking_frames(&tracked, &world, client, &mut decoder)
                .await?
                .is_empty()
        );
        assert_eq!(tracked.tick_count.load(Relaxed), 13);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn equipment_retry_survives_original_pairing_cleanup() -> TestResult {
        use crate::entity::{mob::Mob, passive::pig::PigEntity};
        use pumpkin_data::item::Item;

        struct FinishPairing(Arc<TrackedEntity>, EquipmentSession, AtomicBool);
        impl EventRwHandler<PacketSentEvent> for FinishPairing {
            fn handle<'a>(
                &'a self,
                _: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    if event.packet_id == CSetEquipment::to_id(CURRENT_MC_VERSION)
                        && !self.2.swap(true, Relaxed)
                    {
                        drop(EquipmentPairing {
                            tracker: &self.0,
                            session: self.1,
                        });
                        self.2.store(
                            self.0
                                .lock_equipment_pairings()
                                .contains_key(&self.1.viewer),
                            Relaxed,
                        );
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Missing Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        let mob = pig.get_mob_entity();
        mob.set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, &Item::STICK));
        world.spawn_entity(pig.clone());
        let tracked = world
            .entity_tracker
            .get_tracked_entity(pig.get_entity().entity_id)
            .ok_or("Pig tracker")?;
        let session = tracked.insert_viewer(&viewer).ok_or("New viewer")?;
        let equipment = mob
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        tracked.send_pairing_equipment(&mob.living_entity, &viewer, session);
        drop(equipment);
        assert!(tracked.equipment_retry_pending.load(Relaxed));

        let observer = Arc::new(FinishPairing(
            tracked.clone(),
            session,
            AtomicBool::new(false),
        ));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        tracked.flush_pairing_equipment();
        assert!(observer.2.load(Relaxed));
        assert!(
            !tracked
                .lock_equipment_pairings()
                .contains_key(&session.viewer)
        );

        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let packets = queued_packets(client, &mut decoder).await?;
        let mut equipment = packets
            .iter()
            .filter(|packet| packet.id == CSetEquipment::to_id(CURRENT_MC_VERSION));
        let update = equipment.next().ok_or("Retry lost the equipment packet")?;
        let packet = CSetEquipment::read(&mut update.payload.as_ref(), &CURRENT_MC_VERSION)?;
        assert_eq!(packet.entity_id.0, pig.get_entity().entity_id);
        assert!(packet.equipment.iter().any(|(slot, stack)| *slot
            == EquipmentSlot::MAIN_HAND.discriminant()
            && stack.0.item == &Item::STICK));
        assert!(equipment.next().is_none());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_refreshes_shield_worn_by_packet_observer() -> TestResult {
        use crate::entity::{mob::Mob, passive::pig::PigEntity};
        use pumpkin_data::item::Item;

        struct WearShield(Arc<PigEntity>, AtomicBool);

        impl EventRwHandler<PacketSentEvent> for WearShield {
            fn handle<'a>(
                &'a self,
                _server: &'a Arc<Server>,
                event: &'a mut PacketSentEvent,
            ) -> BoxFuture<'a, ()> {
                Box::pin(async move {
                    if event.packet_id == CSetEquipment::to_id(CURRENT_MC_VERSION)
                        && let Ok(packet) =
                            CSetEquipment::read(&mut event.payload.as_ref(), &CURRENT_MC_VERSION)
                        && packet.entity_id.0 == self.0.get_entity().entity_id
                        && !self.1.swap(true, Relaxed)
                    {
                        let mob = self.0.get_mob_entity();
                        let mut gear = mob
                            .living_entity
                            .entity_equipment
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        if let Some(shield) = gear.equipment.get_mut(&EquipmentSlot::MAIN_HAND) {
                            shield.set_damage(7);
                        }
                        gear.put(&EquipmentSlot::OFF_HAND, ItemStack::new(1, &Item::APPLE));
                    }
                })
            }
        }

        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (viewer, peer) = java_rider(&server, &world, "Notch")?;
        let client = viewer.client.java().ok_or("Missing Java viewer")?;
        client.player.store(Arc::new(Some(viewer.clone())));
        let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
        let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        let mob = pig.get_mob_entity();
        mob.set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, &Item::SHIELD));
        assert!(
            mob.living_entity
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&EquipmentSlot::OFF_HAND)
                .is_empty()
        );
        world.spawn_entity(pig.clone());
        let id = pig.get_entity().entity_id;
        let tracked = world
            .entity_tracker
            .get_tracked_entity(id)
            .ok_or("Pig tracker")?;
        let observer = Arc::new(WearShield(pig, AtomicBool::new(false)));
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        tracked.add_pairing(&viewer);
        assert!(observer.1.load(Relaxed));

        let mut hands = Vec::new();
        for packet in queued_packets(client, &mut decoder).await? {
            if packet.id == CSetEquipment::to_id(CURRENT_MC_VERSION) {
                let packet =
                    CSetEquipment::read(&mut packet.payload.as_ref(), &CURRENT_MC_VERSION)?;
                if packet.entity_id.0 == id {
                    hands.extend(packet.equipment.into_iter().filter_map(|(slot, stack)| {
                        (slot == EquipmentSlot::MAIN_HAND.discriminant()
                            || slot == EquipmentSlot::OFF_HAND.discriminant())
                        .then(|| (stack.0.item, stack.0.get_damage()))
                    }));
                }
            }
        }
        assert_eq!(hands.len(), 2);
        assert!(hands.contains(&(&Item::SHIELD, 7)));
        assert!(hands.iter().any(|(item, _)| *item == &Item::APPLE));
        world.level.shutdown().await;
        Ok(())
    }
}
