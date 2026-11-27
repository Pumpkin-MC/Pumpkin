use std::cell::{Cell, RefCell};
use std::sync::atomic::{
    AtomicBool, AtomicU32, AtomicU64,
    Ordering::{self, Relaxed},
};
use std::sync::{Arc, Mutex, RwLock};

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
    CSetEquipment, CSetPassengers, CUpdateEntityPos, CUpdateEntityPosRot, CUpdateEntityRot,
    Metadata,
};
use pumpkin_protocol::ser::NetworkReadExt;
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

#[derive(Clone, Copy)]
pub(crate) struct EquipmentSession {
    viewer: Uuid,
    generation: u64,
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
    bedrock_items: FxHashMap<BedrockEquipmentGroup, Bytes>,
    pending: Vec<EquipmentSlot>,
    needs_snapshot: bool,
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
        self.send_bedrock_move(entity, world);
        let tick = self.tick_count.fetch_add(1, Relaxed);
        let needs_sync = entity.velocity_dirty.swap(false, Ordering::SeqCst);
        // Head yaw stays live every tick, only position/rotation sync is throttled.
        if tick.is_multiple_of(self.update_interval.max(1))
            || needs_sync
            || entity.synched_data.is_dirty()
        {
            let yaw = pack_degrees(entity.yaw.load());
            let pitch = pack_degrees(entity.pitch.load());
            let rot_changed = yaw != entity.last_sent_yaw.load(Relaxed)
                || pitch != entity.last_sent_pitch.load(Relaxed);

            if self.entity.is_passenger() {
                if rot_changed {
                    self.send_move(entity, MoveSync::Rot, yaw, pitch, world);
                    entity.last_sent_yaw.store(yaw, Relaxed);
                    entity.last_sent_pitch.store(pitch, Relaxed);
                }
                entity.last_sent_pos.store(entity.pos.load());
                entity.send_dirty_entity_data();
                self.was_riding.store(true, Relaxed);
            } else {
                let pos = entity.pos.load();
                let sync = self.pick_move(entity, pos, tick, rot_changed);
                if needs_sync || self.track_deltas || entity.is_fall_flying() {
                    self.send_motion(entity, world);
                }
                if let Some(sync) = sync {
                    self.send_move(entity, sync, yaw, pitch, world);
                }
                entity.send_dirty_entity_data();
                if let Some(sync) = sync {
                    if sync.sends_position() {
                        entity.last_sent_pos.store(pos);
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
            self.send_motion(entity, world);
        }

        // Java `CHeadRot` is per-tick. Bedrock watchers of this entity get head yaw from
        // `send_bedrock_move` (`MoveActorDelta` HAS_HEAD_YAW), not from this packet.
        let head_yaw = pack_degrees(entity.head_yaw.load());
        if head_yaw != entity.last_sent_head_yaw.load(Relaxed) {
            self.send_to_tracking_players(&CHeadRot::new(VarInt(self.entity_id), head_yaw), world);
            entity.last_sent_head_yaw.store(head_yaw, Relaxed);
        }
    }

    fn pick_move(
        &self,
        entity: &Entity,
        pos: Vector3<f64>,
        tick: u32,
        rot_changed: bool,
    ) -> Option<MoveSync> {
        let teleport_delay = self.teleport_delay.fetch_add(1, Relaxed) + 1;
        let base = entity.last_sent_pos.load();
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
        let on_ground = entity.on_ground.load(Relaxed);
        let is_arrow = [
            &EntityType::ARROW,
            &EntityType::SPECTRAL_ARROW,
            &EntityType::TRIDENT,
        ]
        .contains(&entity.entity_type);

        match delta {
            Some(delta)
                if teleport_delay <= 400
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
            _ => {
                self.was_on_ground.store(on_ground, Relaxed);
                self.teleport_delay.store(0, Relaxed);
                Some(MoveSync::Absolute)
            }
        }
    }

    fn send_motion(&self, entity: &Entity, world: &World) {
        let velocity = entity.velocity.load();
        if entity.velocity_changed_since_sent() {
            entity.last_sent_velocity.store(velocity);
            self.send_to_tracking_players_editioned(
                &CEntityVelocity::new(VarInt(self.entity_id), velocity),
                &CSetActorMotion {
                    target_runtime_id: VarULong(self.entity_id as u64),
                    motion: velocity.to_f32_lossy(),
                    tick: VarULong(0),
                },
                world,
            );
        }
    }

    /// Java only -> Bedrock uses [`Self::send_bedrock_move`].
    fn send_move(&self, entity: &Entity, sync: MoveSync, yaw: u8, pitch: u8, world: &World) {
        let on_ground = entity.on_ground.load(Relaxed);
        let id = VarInt(self.entity_id);
        match sync {
            MoveSync::Pos(delta) => {
                self.send_to_tracking_players(&CUpdateEntityPos::new(id, delta, on_ground), world);
            }
            MoveSync::Rot => {
                self.send_to_tracking_players(
                    &CUpdateEntityRot::new(id, yaw, pitch, on_ground),
                    world,
                );
            }
            MoveSync::PosRot(delta) => self.send_to_tracking_players(
                &CUpdateEntityPosRot::new(id, delta, yaw, pitch, on_ground),
                world,
            ),
            MoveSync::Absolute => self.send_to_tracking_players(
                &CEntityPositionSync::new(
                    id,
                    entity.pos.load(),
                    entity.velocity.load(),
                    entity.yaw.load(),
                    entity.pitch.load(),
                    on_ground,
                ),
                world,
            ),
        }
    }

    /// Bedrock -> every tick, only changed fields.
    fn send_bedrock_move(&self, entity: &Entity, world: &World) {
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

        if flags
            & (MOVE_ACTOR_DELTA_FLAG_HAS_X
                | MOVE_ACTOR_DELTA_FLAG_HAS_Y
                | MOVE_ACTOR_DELTA_FLAG_HAS_Z)
            != 0
        {
            self.bedrock_pos.store(pos);
        }
        self.bedrock_rot.store(rot);
        if !self.has_bedrock_watchers() {
            return;
        }
        if entity.on_ground.load(Relaxed) {
            flags |= MOVE_ACTOR_DELTA_FLAG_ON_GROUND;
        }
        self.send_to_tracking_players_bedrock(
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
            world,
        );
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
                    bedrock_items: FxHashMap::default(),
                    pending: Vec::new(),
                    needs_snapshot: false,
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

    fn java_passenger_update_overlaps(
        mut payload: &[u8],
        protocol: pumpkin_util::version::JavaMinecraftVersion,
        carrier: i32,
        riders: &[VarInt],
    ) -> bool {
        use pumpkin_protocol::{ServerPacket, java::client::play::CSetEntityLink};
        use pumpkin_util::version::JavaMinecraftVersion;

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

    #[allow(clippy::too_many_lines)]
    fn send_pairing(&self, player: &Arc<Player>, session: EquipmentSession) {
        let _pairing = EquipmentPairing {
            tracker: self,
            session,
        };
        let still_paired = || self.lock_pairing_session(session).is_some();
        let enqueue_java = |client: &JavaClient, data| {
            client.try_enqueue_packet_data_filtering(
                data,
                || self.lock_pairing_session(session),
                |data| {
                    let version = client.version.load();
                    (Self::java_pairing_actor(&data, version)
                        .is_none_or(|(_, actor)| actor != self.entity_id)
                        && !Self::java_passenger_update_overlaps(
                            &data,
                            version,
                            self.entity_id,
                            &[VarInt(self.entity_id)],
                        ))
                    .then_some(data)
                },
            );
        };
        let enqueue_bedrock = |client: &BedrockClient, data| {
            client.try_enqueue_packet_data_filtering(
                data,
                || self.lock_pairing_session(session),
                |data| (!self.is_bedrock_pairing_actor(&data)).then_some(data),
            );
        };
        match player.client.as_ref() {
            ClientPlatform::Java(client) => {
                self.entity
                    .send_java_spawn_packets(client, &mut |data| enqueue_java(client, data));
            }
            ClientPlatform::Bedrock(client) => {
                self.entity
                    .send_bedrock_spawn_packets(client, true, &mut |data| {
                        enqueue_bedrock(client, data);
                    });
            }
            ClientPlatform::SimulatedPlayer(_) => {}
        }
        if !still_paired() {
            return;
        }
        if let Some(state) =
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
                    enqueue_java(client, packet_data);
                }

                let head_yaw = target_entity.head_yaw.load();
                // TODO: use `pumpkin_util::math::pack_degrees`.
                let head_rot_packet = CHeadRot::new(
                    target_id.into(),
                    (head_yaw * 256.0 / 360.0).rem_euclid(256.0) as u8,
                );
                if let Ok(data) = client.serialize_packet(&head_rot_packet) {
                    enqueue_java(client, data);
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
                enqueue_java(client, data);
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
                        enqueue_java(client, data);
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
                        enqueue_bedrock(client, data);
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
                    enqueue_java(client, packet_data);
                }
            }
        }

        if let Some(living) = self.entity.get_living_entity() {
            self.send_pairing_equipment(living, player, session);

            if let Ok(effects) = living
                .active_effects
                .try_lock()
                .map(|effects| effects.clone())
            {
                for effect in effects.values() {
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
                                enqueue_java(client, data);
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
                                enqueue_bedrock(client, data);
                            }
                        }
                        ClientPlatform::SimulatedPlayer(_) => {}
                    }
                }
            }
        }

        if !still_paired() {
            return;
        }
        let entity = self.entity.get_entity();
        self.send_pairing_passengers(entity, player, session, None);
        let vehicle = entity
            .vehicle
            .try_lock()
            .ok()
            .and_then(|vehicle| vehicle.as_ref().and_then(std::sync::Weak::upgrade));
        if let Some(vehicle) = vehicle {
            self.send_pairing_passengers(
                vehicle.get_entity(),
                player,
                session,
                Some(self.entity_id),
            );
        }
    }

    fn send_pairing_passengers(
        &self,
        carrier: &Entity,
        player: &Player,
        session: EquipmentSession,
        paired_rider: Option<i32>,
    ) {
        let Ok(ids) = carrier.passengers.try_lock().map(|passengers| {
            passengers
                .iter()
                .map(|rider| VarInt(rider.get_entity().entity_id))
                .collect::<Vec<_>>()
        }) else {
            return;
        };
        if ids.is_empty() || paired_rider.is_some_and(|id| !ids.contains(&VarInt(id))) {
            return;
        }
        let session_invalid = std::cell::Cell::new(false);
        let lock_current = || {
            let pairing = self.lock_pairing_session(session);
            session_invalid.set(pairing.is_none());
            let pairing = pairing?;
            let passengers = carrier.passengers.try_lock().ok()?;
            (!carrier.is_removed()
                && passengers
                    .iter()
                    .map(|rider| VarInt(rider.get_entity().entity_id))
                    .eq(ids.iter().copied()))
            .then_some((pairing, passengers))
        };
        match player.client.as_ref() {
            ClientPlatform::Java(client) => {
                let packet = CSetPassengers::new(VarInt(carrier.entity_id), &ids);
                if let Ok(data) = client.serialize_packet(&packet) {
                    client.try_enqueue_packet_data_filtering(data, lock_current, |data| {
                        let version = client.version.load();
                        let actor_removed = session_invalid.get()
                            && (Self::java_passenger_update_overlaps(
                                &data,
                                version,
                                self.entity_id,
                                &[VarInt(self.entity_id)],
                            ) || Self::java_pairing_actor(&data, version)
                                .is_some_and(|(_, actor)| actor == self.entity_id));
                        (!actor_removed
                            && !Self::java_passenger_update_overlaps(
                                &data,
                                version,
                                carrier.entity_id,
                                &ids,
                            ))
                        .then_some(data)
                    });
                }
            }
            ClientPlatform::Bedrock(client) => {
                use pumpkin_protocol::bedrock::client::{CSetActorLink, common::ActorLink};
                for (index, rider) in ids.iter().enumerate() {
                    if paired_rider.is_some_and(|id| id != rider.0) {
                        continue;
                    }
                    let packet = CSetActorLink {
                        link: ActorLink {
                            ridden_unique_id: i64::from(carrier.entity_id).into(),
                            rider_unique_id: i64::from(rider.0).into(),
                            link_type: if index == 0 { 1 } else { 2 },
                            immediate: true,
                            rider_initiated: false,
                            vehicle_angular_velocity: 0.0,
                        },
                    };
                    if let Ok(data) = client.serialize_packet(&packet) {
                        client.try_enqueue_packet_data_filtering(data, lock_current, |data| {
                            let mut payload = data.as_ref();
                            let related = VarUInt::read(&mut payload)
                                .is_ok_and(|id| id.0 == CSetActorLink::PACKET_ID as u32)
                                && (VarLong::read(&mut payload)
                                    .is_ok_and(|id| id.0 == i64::from(carrier.entity_id))
                                    || VarLong::read(&mut payload).is_ok_and(|id| {
                                        ids.iter().any(|rider| i64::from(rider.0) == id.0)
                                    }));
                            let actor_removed =
                                session_invalid.get() && self.is_bedrock_pairing_actor(&data);
                            (!related && !actor_removed).then_some(data)
                        });
                    }
                }
            }
            ClientPlatform::SimulatedPlayer(_) => {}
        }
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
        let Some(viewer) = viewers.remove(&player.gameprofile.id) else {
            return false;
        };
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
        for attempt in 0..attempts {
            if slots.is_empty() || living.entity.is_removed() {
                return;
            }
            if attempt != 0 {
                let Some(next) = self.try_equipment_snapshot(living) else {
                    break;
                };
                snapshot = next;
            }
            if player.client.java().is_some() {
                let mut pairings = self.lock_equipment_pairings();
                let Some(state) = Self::current_equipment_pairing(&mut pairings, session) else {
                    return;
                };
                slots.retain(|slot| {
                    !state
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
        }
        if !slots.is_empty()
            && let Some(state) =
                Self::current_equipment_pairing(&mut self.lock_equipment_pairings(), session)
        {
            state.pending = slots;
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
            .filter(|(_, state)| state.needs_snapshot || !state.pending.is_empty())
            .map(|(&viewer, state)| {
                (
                    state.viewer.clone(),
                    EquipmentSession {
                        viewer,
                        generation: state.generation,
                    },
                    if std::mem::take(&mut state.needs_snapshot) {
                        Self::occupied_slots(&snapshot)
                    } else {
                        std::mem::take(&mut state.pending)
                    },
                )
            })
            .collect::<Vec<_>>();
        for (player, session, slots) in pending {
            self.send_equipment_pairing(living, &player, session, snapshot.clone(), slots, 1);
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
                let stale = changed.borrow();
                if active.get() && stale.is_empty() {
                    return Some(data);
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
        let entity_type = entity.get_entity().entity_type;
        let range = entity_type.client_tracking_range;
        if range == 0 {
            return;
        }
        let update_interval = entity_type.update_interval;
        let track_deltas = entity_type.track_deltas;
        let entity_id = entity.get_entity().entity_id;

        let tracked = Arc::new(TrackedEntity::new(
            entity.clone(),
            range,
            update_interval,
            track_deltas,
        ));
        match self.entity_map.entry(entity_id) {
            dashmap::mapref::entry::Entry::Occupied(_) => return,
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(tracked.clone());
            }
        }

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
    use super::*;
    use pumpkin_protocol::{
        PacketDecodeError, RawPacket,
        java::{client::play::CKeepAlive, packet_decoder::TCPNetworkDecoder},
    };
    use std::time::Duration;
    use tokio::io::{BufReader, DuplexStream};

    use crate::plugin::{
        BoxFuture, EventPriority, EventRwHandler, server::packet::PacketSentEvent,
    };
    use crate::server::Server;
    use crate::world::item_inventory_tests::{
        CENTER, TestResult, inventory_world as vehicle_world, java_player as java_rider,
    };

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
        for packet in queued_packets(client, &mut decoder).await? {
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
        let carrier = cart.get_entity();
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
        mob.set_item_slot(&EquipmentSlot::OFF_HAND, ItemStack::new(1, &Item::APPLE));
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
