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
    seen_by: RwLock<FxHashMap<Uuid, Arc<Player>>>,
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
struct EquipmentSession {
    viewer: Uuid,
    generation: u64,
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
            .filter(|player| player.client.bedrock().is_some())
            .cloned()
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
                    return;
                }
                entity.send_dirty_entity_data();
                let Some(_motion_guard) = update.lock_if_current() else {
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
                let inserted = self
                    .seen_by
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .insert(player.gameprofile.id, player.clone())
                    .is_none();
                if inserted {
                    if matches!(player.client.as_ref(), ClientPlatform::Bedrock(_)) {
                        self.bedrock_watchers.fetch_add(1, Relaxed);
                    }
                    self.add_pairing(player);
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

    #[allow(clippy::too_many_lines)]
    pub fn add_pairing(&self, player: &Arc<Player>) {
        player.client.try_enqueue_spawn_packet(&self.entity);
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
                    client.try_enqueue_packet(packet_data);
                }

                let head_yaw = target_entity.head_yaw.load();
                // TODO: use `pumpkin_util::math::pack_degrees`.
                let head_rot_packet = CHeadRot::new(
                    target_id.into(),
                    (head_yaw * 256.0 / 360.0).rem_euclid(256.0) as u8,
                );
                if let Ok(data) = client.serialize_packet(&head_rot_packet) {
                    client.try_enqueue_packet(data);
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
                client.try_enqueue_packet(data);
            }
        }

        let vel = self.entity.get_entity().velocity.load();
        if vel.length_squared() > 1e-4 {
            match player.client.as_ref() {
                ClientPlatform::Java(client) => {
                    let motion = CEntityVelocity::new(self.entity_id.into(), vel);
                    if let Ok(data) = client.serialize_packet(&motion) {
                        client.try_enqueue_packet(data);
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
                        client.try_enqueue_packet(data);
                    }
                }
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
                    client.try_enqueue_packet(packet_data);
                }
            }
        }

        if let Some(living) = self.entity.get_living_entity() {
            self.send_pairing_equipment(living, player);

            if let Ok(effects_guard) = living.active_effects.try_lock() {
                for effect in effects_guard.values() {
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
                                client.try_enqueue_packet(data);
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
                                client.try_enqueue_packet(data);
                            }
                        }
                    }
                }
            }
        }

        let entity = self.entity.get_entity();
        Self::send_pairing_passengers(entity, player, None);
        if let Some(vehicle) = entity.get_vehicle() {
            Self::send_pairing_passengers(vehicle.get_entity(), player, Some(self.entity_id));
        }
    }

    fn send_pairing_passengers(carrier: &Entity, player: &Player, paired_rider: Option<i32>) {
        // Packet observers can change either link while pairing this carrier or its rider.
        let ids: Vec<VarInt> = carrier
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|p| VarInt(p.get_entity().entity_id))
            .collect();
        if ids.is_empty() || paired_rider.is_some_and(|rider_id| !ids.contains(&VarInt(rider_id))) {
            return;
        }
        let links = if player.client.bedrock().is_some() {
            ids.iter()
                .enumerate()
                .filter(|(_, id)| paired_rider.is_none_or(|rider_id| id.0 == rider_id))
                .map(|(index, id)| {
                    Some(carrier.bedrock_passenger_link(
                        id.0,
                        if index == 0 { 1 } else { 2 },
                        false,
                    ))
                })
                .collect()
        } else {
            Vec::new()
        };
        carrier.send_passenger_update_to(&carrier.world.load(), player, &ids, &links);
    }

    pub fn remove_pairing(&self, player: &Player) {
        let entity_ids = [self.entity_id.into()];
        match player.client.as_ref() {
            ClientPlatform::Java(client) => {
                let packet = CRemoveEntities::new(&entity_ids);
                if let Ok(data) = client.serialize_packet(&packet) {
                    client.try_enqueue_packet(data);
                }
            }
            ClientPlatform::Bedrock(client) => {
                let packet = CRemoveActor::new(VarLong(i64::from(self.entity_id)));
                if let Ok(data) = client.serialize_packet(&packet) {
                    client.try_enqueue_packet(data);
                }
            }
        }
    }

    pub fn broadcast_removed(&self, _world: &World) {
        let had_bedrock = self.bedrock_watchers.swap(0, Relaxed) > 0;
        let seen_by = {
            let mut guard = self
                .seen_by
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            self.lock_equipment_pairings().clear();
            std::mem::take(&mut *guard)
        };
        if seen_by.is_empty() {
            return;
        }
        let entity_ids = [self.entity_id.into()];
        let je_packet = CRemoveEntities::new(&entity_ids);
        World::broadcast_java_players(&je_packet, seen_by.values());
        if had_bedrock {
            let be_packet = CRemoveActor::new(VarLong(i64::from(self.entity_id)));
            World::broadcast_bedrock_grouped(
                &be_packet,
                seen_by.values().filter_map(|p| match p.client.as_ref() {
                    ClientPlatform::Bedrock(be_client) => Some(be_client),
                    ClientPlatform::Java(_) => None,
                }),
            );
        }
    }

    /// Vanilla `TrackedEntity.removePlayer`: despawn on the client, only if it was paired.
    pub fn remove_player(&self, player: &Player) {
        let removed = self.remove_viewer(player);
        if removed {
            if matches!(player.client.as_ref(), ClientPlatform::Bedrock(_)) {
                self.bedrock_watchers.fetch_sub(1, Relaxed);
            }
            self.remove_pairing(player);
        }
    }

    /// Forgets the player as a viewer without a removal packet; the client already dropped it.
    pub fn forget_player(&self, player: &Player) {
        let removed = self.remove_viewer(player);
        if removed && matches!(player.client.as_ref(), ClientPlatform::Bedrock(_)) {
            self.bedrock_watchers.fetch_sub(1, Relaxed);
        }
    }

    fn remove_viewer(&self, player: &Player) -> bool {
        let mut viewers = self
            .seen_by
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if viewers.remove(&player.gameprofile.id).is_none() {
            return false;
        }
        self.lock_equipment_pairings()
            .remove(&player.gameprofile.id);
        true
    }

    fn viewers_snapshot(&self) -> Vec<Arc<Player>> {
        // A packet observer may remove this entity and take the write lock.
        self.seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .cloned()
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
                .is_some_and(|player| Arc::ptr_eq(player, recipient)))
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

    fn equipment_session(&self, viewer: Uuid) -> Option<EquipmentSession> {
        self.lock_equipment_pairings()
            .get(&viewer)
            .map(|state| EquipmentSession {
                viewer,
                generation: state.generation,
            })
    }

    fn current_equipment_pairing(
        pairings: &mut EquipmentPairings,
        session: EquipmentSession,
    ) -> Option<&mut EquipmentPairingState> {
        pairings
            .get_mut(&session.viewer)
            .filter(|state| state.generation == session.generation)
    }

    fn send_pairing_equipment(&self, living: &LivingEntity, player: &Arc<Player>) {
        let Some(snapshot) = living
            .entity_equipment
            .try_lock()
            .ok()
            .filter(|gear| !gear.is_empty())
            .map(|gear| gear.clone())
        else {
            return;
        };
        let session = EquipmentSession {
            viewer: player.gameprofile.id,
            generation: self.next_equipment_pairing.fetch_add(1, Relaxed),
        };
        {
            let mut pairings = self.lock_equipment_pairings();
            let std::collections::hash_map::Entry::Vacant(entry) = pairings.entry(session.viewer)
            else {
                return;
            };
            entry.insert(EquipmentPairingState {
                viewer: player.clone(),
                generation: session.generation,
                java_items: FxHashMap::default(),
                bedrock_items: FxHashMap::default(),
                pending: Vec::new(),
            })
        };
        let slots = snapshot
            .equipment
            .iter()
            .filter(|(_, stack)| !stack.is_empty())
            .map(|(slot, _)| slot.clone())
            .collect();
        self.send_equipment_pairing(living, player, session, snapshot, slots, 2);
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
                snapshot = living
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
            }
            slots = match player.client.as_ref() {
                ClientPlatform::Java(client) => self.send_java_equipment(
                    client,
                    living,
                    slots.into_iter().map(|slot| {
                        let stack = snapshot.get(&slot);
                        (slot, stack)
                    }),
                    Some(session),
                ),
                ClientPlatform::Bedrock(client) => {
                    self.send_bedrock_pairing(client, living, &snapshot, &slots, session)
                }
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
        let pending = self
            .lock_equipment_pairings()
            .iter_mut()
            .filter(|(_, state)| !state.pending.is_empty())
            .map(|(&viewer, state)| {
                (
                    state.viewer.clone(),
                    EquipmentSession {
                        viewer,
                        generation: state.generation,
                    },
                    std::mem::take(&mut state.pending),
                )
            })
            .collect::<Vec<_>>();
        for (player, session, slots) in pending {
            let snapshot = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            self.send_equipment_pairing(living, &player, session, snapshot, slots, 1);
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
                && self.send_bedrock_hand(
                    client,
                    living,
                    &slot,
                    &snapshot.get(&slot),
                    Some(session),
                )
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

    fn equipment_slot_matches(
        current: &EntityEquipment,
        slot: &EquipmentSlot,
        snapshot: Option<&ItemStack>,
        bedrock: bool,
    ) -> bool {
        let current = current
            .equipment
            .get(slot)
            .filter(|stack| !stack.is_empty());
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

    fn send_java_equipment(
        &self,
        client: &JavaClient,
        living: &LivingEntity,
        slots: impl IntoIterator<Item = (EquipmentSlot, ItemStack)>,
        session: Option<EquipmentSession>,
    ) -> Vec<EquipmentSlot> {
        let (slots, equipment): (Vec<_>, Vec<_>) = slots
            .into_iter()
            .map(|(slot, stack)| {
                let id = slot.discriminant();
                (slot, (id, ItemStackSerializer::from(stack)))
            })
            .unzip();
        if slots.is_empty() || living.entity.is_removed() {
            return Vec::new();
        }
        let packet = CSetEquipment::new(self.entity_id.into(), equipment);
        let Ok(data) = client.serialize_packet(&packet) else {
            return Vec::new();
        };
        let version = client.version.load();
        let changed = RefCell::new(Vec::new());
        let active = Cell::new(true);
        client.try_enqueue_packet_data_projecting(
            data,
            || {
                let gear = living
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut pairings = session.map(|_| self.lock_equipment_pairings());
                active.set(
                    !living.entity.is_removed()
                        && session.is_none_or(|session| {
                            pairings
                                .as_deref_mut()
                                .and_then(|pairings| {
                                    Self::current_equipment_pairing(pairings, session)
                                })
                                .is_some()
                        }),
                );
                if active.get() {
                    changed.borrow_mut().extend(
                        slots
                            .iter()
                            .zip(&packet.equipment)
                            .filter(|(slot, (_, stack))| {
                                !Self::equipment_slot_matches(&gear, slot, Some(&stack.0), false)
                            })
                            .map(|(slot, _)| slot.clone()),
                    );
                }
                (gear, pairings)
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
            },
            |data, (_, pairings)| {
                if let Some(state) = session.and_then(|session| {
                    Self::current_equipment_pairing(pairings.as_deref_mut()?, session)
                }) && let Some(packet) =
                    Self::java_equipment_from_packet(data, version, self.entity_id)
                {
                    state.java_items.extend(
                        packet
                            .equipment
                            .into_iter()
                            .map(|(slot, item)| (slot, item.0.into_owned())),
                    );
                }
            },
        );
        let mut changed = changed.into_inner();
        if let Some(session) = session {
            let gear = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut pairings = self.lock_equipment_pairings();
            let Some(state) = Self::current_equipment_pairing(&mut pairings, session) else {
                return Vec::new();
            };
            changed.retain(|slot| {
                !state
                    .java_items
                    .get(&slot.discriminant())
                    .is_some_and(|sent| {
                        Self::equipment_slot_matches(&gear, slot, Some(sent), false)
                    })
            });
        }
        changed
    }

    fn project_java_equipment(
        data: Bytes,
        primary: bool,
        active: bool,
        stale: &[EquipmentSlot],
        version: pumpkin_util::version::JavaMinecraftVersion,
        entity_id: i32,
    ) -> Option<Bytes> {
        let Some(mut packet) = Self::java_equipment_from_packet(&data, version, entity_id) else {
            return (!primary).then_some(data);
        };
        if !active {
            return None;
        }
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
        session: Option<EquipmentSession>,
    ) -> bool {
        use pumpkin_protocol::bedrock::server::inventory_transaction::WINDOW_ID_OFF_HAND;
        let packet = CMobEquipment {
            target_runtime_id: (self.entity_id as u64).into(),
            item: stack.into(),
            slot: 0,
            selected_slot: 0,
            container_id: if *slot == EquipmentSlot::OFF_HAND {
                WINDOW_ID_OFF_HAND as u8
            } else {
                0
            },
        };
        self.send_bedrock_equipment(client, living, &packet, session, |current| {
            Self::equipment_slot_matches(current, slot, Some(stack), true)
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
        self.send_bedrock_equipment(client, living, &packet, Some(session), |current| {
            Self::ARMOR_SLOTS.iter().all(|slot| {
                Self::equipment_slot_matches(current, slot, snapshot.equipment.get(slot), true)
            })
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
        session: Option<EquipmentSession>,
        matches: impl Fn(&EntityEquipment) -> bool,
    ) -> bool {
        let Ok(data) = client.serialize_packet(packet) else {
            return false;
        };
        let Some(group) = Self::bedrock_equipment_group(&data) else {
            return false;
        };
        if let Some(session) = session {
            let gear = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut pairings = self.lock_equipment_pairings();
            let Some(state) = Self::current_equipment_pairing(&mut pairings, session) else {
                return false;
            };
            if state.bedrock_items.get(&group) == Some(&data) {
                return !matches(&gear);
            }
        }
        let canonical = data.clone();
        let stale = Cell::new(false);
        let active = Cell::new(true);
        client.try_enqueue_packet_data_projecting(
            data,
            || {
                let gear = living
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut pairings = session.map(|_| self.lock_equipment_pairings());
                active.set(
                    !living.entity.is_removed()
                        && session.is_none_or(|session| {
                            pairings
                                .as_deref_mut()
                                .and_then(|pairings| {
                                    Self::current_equipment_pairing(pairings, session)
                                })
                                .is_some()
                        }),
                );
                stale.set(active.get() && !matches(&gear));
                (gear, pairings)
            },
            |data, primary, _| {
                ((active.get() && !stale.get())
                    || (!primary && Self::bedrock_equipment_group(&data) != Some(group)))
                .then_some(data)
            },
            |data, primary, (_, pairings)| {
                // The primary retains its typed group even if the translator changes its encoding.
                if (primary || Self::bedrock_equipment_group(data) == Some(group))
                    && let Some(state) = session.and_then(|session| {
                        Self::current_equipment_pairing(pairings.as_deref_mut()?, session)
                    })
                {
                    state.bedrock_items.insert(group, canonical.clone());
                }
            },
        );
        stale.get()
    }

    pub(crate) fn send_cleared_hand_equipment(&self, living: &LivingEntity, slot: &EquipmentSlot) {
        for viewer in self.viewers_snapshot() {
            let session = self.equipment_session(viewer.gameprofile.id);
            match viewer.client.as_ref() {
                ClientPlatform::Java(client) => {
                    self.send_java_equipment(
                        client,
                        living,
                        [(slot.clone(), ItemStack::EMPTY.clone())],
                        session,
                    );
                }
                ClientPlatform::Bedrock(client) => {
                    self.send_bedrock_hand(client, living, slot, ItemStack::EMPTY, session);
                }
            }
        }
    }

    pub(crate) fn broadcast_java_equipment(&self, packet: &CSetEquipment) {
        let viewers = self.viewers_snapshot();
        if self.lock_equipment_pairings().is_empty() {
            World::broadcast_java_players(packet, viewers.iter());
            return;
        }
        let Ok(data) = JavaClient::serialize_packet_for_version(packet, CURRENT_MC_VERSION) else {
            return;
        };
        for viewer in viewers {
            let Some(client) = viewer.client.java() else {
                continue;
            };
            let Some(session) = self.equipment_session(viewer.gameprofile.id) else {
                client.try_enqueue_packet(data.clone());
                continue;
            };
            let version = client.version.load();
            client.try_enqueue_packet_data_projecting(
                data.clone(),
                || self.lock_equipment_pairings(),
                |data, _, pairings| {
                    pairings
                        .get(&session.viewer)
                        .is_some_and(|state| state.generation == session.generation)
                        .then_some(data)
                },
                |data, pairings| {
                    if let Some(state) = Self::current_equipment_pairing(pairings, session)
                        && let Some(packet) =
                            Self::java_equipment_from_packet(data, version, self.entity_id)
                    {
                        state.java_items.extend(
                            packet
                                .equipment
                                .into_iter()
                                .map(|(slot, item)| (slot, item.0.into_owned())),
                        );
                    }
                },
            );
        }
    }

    pub(crate) fn broadcast_bedrock_equipment<P: BClientPacket + Sync>(
        &self,
        packet: &P,
        world: &World,
    ) {
        if self.lock_equipment_pairings().is_empty() {
            self.send_to_tracking_players_bedrock(packet, world);
            return;
        }
        for viewer in self.bedrock_watchers_snapshot() {
            let Some(client) = viewer.client.bedrock() else {
                continue;
            };
            let Ok(data) = client.serialize_packet(packet) else {
                continue;
            };
            let (Some(session), Some(group)) = (
                self.equipment_session(viewer.gameprofile.id),
                Self::bedrock_equipment_group(&data),
            ) else {
                client.try_enqueue_packet(data);
                continue;
            };
            let canonical = data.clone();
            client.try_enqueue_packet_data_projecting(
                data,
                || self.lock_equipment_pairings(),
                |data, _, pairings| {
                    pairings
                        .get(&session.viewer)
                        .is_some_and(|state| state.generation == session.generation)
                        .then_some(data)
                },
                |data, primary, pairings| {
                    if (primary || Self::bedrock_equipment_group(data) == Some(group))
                        && let Some(state) = Self::current_equipment_pairing(pairings, session)
                    {
                        state.bedrock_items.insert(group, canonical.clone());
                    }
                },
            );
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
                    ClientPlatform::Java(_) => None,
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
        for entry in &self.entity_map {
            if entry.value().is_seen_by(&player.gameprofile.id) {
                f(&entry.value().entity);
            }
        }
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
        self.entity_map.insert(entity_id, tracked.clone());

        let players = world.players.load();
        tracked.update_players(players.as_ref(), world);
    }

    /// Must only be called after the player's own `CLogin` packet has been sent.
    pub fn pair_new_player_with_tracked_entities(&self, player_arc: &Arc<Player>, world: &World) {
        let entity_id = player_arc.get_entity().entity_id;
        for entry in &self.entity_map {
            if *entry.key() != entity_id {
                entry.value().update_player(player_arc, world);
            }
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
        for entry in &self.entity_map {
            if *entry.key() != entity_id {
                entry.value().forget_player(player);
                entry.value().update_player(player, world);
            }
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
        for entry in &self.entity_map {
            if *entry.key() != entity_id
                && chunks.contains(&entry.value().entity.get_entity().chunk_pos.load())
            {
                entry.value().update_player(player, world);
            }
        }
    }

    pub fn remove_entity(&self, entity: &dyn EntityBase, world: &World) {
        let entity_id = entity.get_entity().entity_id;
        if let Some(player) = entity.get_player() {
            for entry in &self.entity_map {
                entry.value().remove_player(player);
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
        if let Some(tracked) = self.entity_map.get(&entity.get_entity().entity_id) {
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
    use crate::plugin::{BoxFuture, EventHandler, EventPriority, server::packet::PacketSentEvent};
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

    enum CartObserverAction {
        Remove,
        Teleport(Vector3<f64>),
    }

    struct MoveCartFromVelocity {
        cart: Arc<MinecartEntity>,
        action: CartObserverAction,
        ran: AtomicBool,
    }

    impl EventHandler<PacketSentEvent> for MoveCartFromVelocity {
        fn handle_blocking<'a>(
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

    impl EventHandler<PacketSentEvent> for SupersedeCartTeleport {
        fn handle_blocking<'a>(
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

    impl EventHandler<PacketSentEvent> for DismountCartFromPairing {
        fn handle_blocking<'a>(
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
        tracked
            .seen_by
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(viewer.gameprofile.id, viewer.clone());
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
        server.plugin_manager.register::<PacketSentEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            true,
        );
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
        server.plugin_manager.register::<PacketSentEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            true,
        );
        server.plugin_manager.register::<PacketSentEvent, _>(
            superseding.clone(),
            EventPriority::Normal,
            true,
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
        server.plugin_manager.register::<PacketSentEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            true,
        );

        for tracker in [cart_tracker, rider_tracker] {
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
        tracked
            .seen_by
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(viewer.gameprofile.id, viewer.clone());
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
}
