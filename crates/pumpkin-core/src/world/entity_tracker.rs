use std::sync::atomic::{
    AtomicBool, AtomicU32,
    Ordering::{self, Relaxed},
};
use std::sync::{Arc, RwLock};

use bytes::BufMut;
use crossbeam::atomic::AtomicCell;
use dashmap::DashMap;
use pumpkin_data::entity::EntityType;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_protocol::bedrock::client::CSetActorMotion;
use pumpkin_protocol::bedrock::client::move_actor_delta::{
    CMoveActorDelta, MOVE_ACTOR_DELTA_FLAG_HAS_HEAD_YAW, MOVE_ACTOR_DELTA_FLAG_HAS_PITCH,
    MOVE_ACTOR_DELTA_FLAG_HAS_X, MOVE_ACTOR_DELTA_FLAG_HAS_Y, MOVE_ACTOR_DELTA_FLAG_HAS_YAW,
    MOVE_ACTOR_DELTA_FLAG_HAS_Z, MOVE_ACTOR_DELTA_FLAG_ON_GROUND,
};
use pumpkin_protocol::bedrock::client::remove_actor::CRemoveActor;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::{
    CEntityPositionSync, CEntityVelocity, CHeadRot, CRemoveEntities, CSetEntityMetadata,
    CSetEquipment, CSetPassengers, CUpdateEntityPos, CUpdateEntityPosRot, CUpdateEntityRot,
    Metadata,
};
use pumpkin_protocol::{BClientPacket, ClientPacket};
use pumpkin_util::GameMode;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{pack_degrees, position::chunk_section_from_pos};
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use uuid::Uuid;

use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::net::ClientPlatform;
use crate::net::java::JavaClient;
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
    seen_by: RwLock<FxHashMap<Uuid, Arc<Player>>>,
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
        );
    }

    fn update_player_in_view(
        &self,
        player: &Arc<Player>,
        player_pos: Vector3<f64>,
        player_vd: i32,
        entity_chunk: Vector2<i32>,
        entity_pos: Vector3<f64>,
        effective_range: u32,
    ) {
        let dx = player_pos.x - entity_pos.x;
        let dz = player_pos.z - entity_pos.z;
        let dist_sq = dx.mul_add(dx, dz * dz);

        let visible_range_blocks = f64::from((effective_range as i32).min(player_vd) * 16);
        let range_sq = visible_range_blocks * visible_range_blocks;

        // Vanilla `isChunkTracked`: never spawn before the chunk packet.
        let is_visible = dist_sq <= range_sq
            && self.broadcast_to_player(player)
            && player
                .chunk_sender
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_chunk_ready(&entity_chunk);

        if is_visible {
            if self
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
        } else {
            self.remove_player(player);
        }
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
            );
        }
    }

    fn update_players_from_snapshots(&self, snapshots: &[PlayerViewSnapshot<'_>]) {
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
            if let Ok(equipment_guard) = living.entity_equipment.try_lock() {
                let mut equipment_list = Vec::new();
                for (slot, item_stack) in &equipment_guard.equipment {
                    if !item_stack.is_empty() {
                        equipment_list.push((slot.discriminant(), item_stack.clone()));
                    }
                }
                if !equipment_list.is_empty() {
                    match player.client.as_ref() {
                        ClientPlatform::Java(client) => {
                            let equipment: Vec<(i8, ItemStackSerializer)> = equipment_list
                                .iter()
                                .map(|(slot, stack)| {
                                    (*slot, ItemStackSerializer::from(stack.clone()))
                                })
                                .collect();
                            let packet = CSetEquipment::new(self.entity_id.into(), equipment);
                            if let Ok(data) = client.serialize_packet(&packet) {
                                client.try_enqueue_packet(data);
                            }
                        }
                        ClientPlatform::Bedrock(client) => {
                            use pumpkin_data::data_component_impl::EquipmentSlot;
                            use pumpkin_protocol::bedrock::{
                                client::{
                                    mob_armor_equipment::CMobArmorEquipment,
                                    mob_equipment::CMobEquipment,
                                },
                                network_item::NetworkItemStackDescriptor,
                            };

                            if let Some(main_hand) =
                                equipment_guard.equipment.get(&EquipmentSlot::MAIN_HAND)
                                && !main_hand.is_empty()
                            {
                                let pkt = CMobEquipment {
                                    target_runtime_id: (self.entity_id as u64).into(),
                                    item: main_hand.into(),
                                    slot: 0,
                                    selected_slot: 0,
                                    container_id: 0,
                                };
                                if let Ok(data) = client.serialize_packet(&pkt) {
                                    client.try_enqueue_packet(data);
                                }
                            }
                            if let Some(off_hand) =
                                equipment_guard.equipment.get(&EquipmentSlot::OFF_HAND)
                                && !off_hand.is_empty()
                            {
                                let pkt = CMobEquipment {
                                    target_runtime_id: (self.entity_id as u64).into(),
                                    item: off_hand.into(),
                                    slot: 0,
                                    selected_slot: 0,
                                    container_id: 119,
                                };
                                if let Ok(data) = client.serialize_packet(&pkt) {
                                    client.try_enqueue_packet(data);
                                }
                            }
                            let get_desc = |slot: &EquipmentSlot| {
                                equipment_guard.equipment.get(slot).map_or_else(
                                    NetworkItemStackDescriptor::default,
                                    NetworkItemStackDescriptor::from,
                                )
                            };
                            let has_armor = [
                                &EquipmentSlot::HEAD,
                                &EquipmentSlot::CHEST,
                                &EquipmentSlot::LEGS,
                                &EquipmentSlot::FEET,
                                &EquipmentSlot::BODY,
                            ]
                            .into_iter()
                            .any(|s| {
                                equipment_guard
                                    .equipment
                                    .get(s)
                                    .is_some_and(|stack| !stack.is_empty())
                            });
                            if has_armor {
                                let armor = CMobArmorEquipment {
                                    target_runtime_id: (self.entity_id as u64).into(),
                                    head: get_desc(&EquipmentSlot::HEAD),
                                    chest: get_desc(&EquipmentSlot::CHEST),
                                    legs: get_desc(&EquipmentSlot::LEGS),
                                    feet: get_desc(&EquipmentSlot::FEET),
                                    body: get_desc(&EquipmentSlot::BODY),
                                };
                                if let Ok(data) = client.serialize_packet(&armor) {
                                    client.try_enqueue_packet(data);
                                }
                            }
                        }
                    }
                }
            }

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

        if let Ok(passengers) = self.entity.get_entity().passengers.try_lock()
            && !passengers.is_empty()
        {
            match player.client.as_ref() {
                ClientPlatform::Java(client) => {
                    let passenger_ids: Vec<VarInt> = passengers
                        .iter()
                        .map(|p| VarInt(p.get_entity().entity_id))
                        .collect();
                    let packet = CSetPassengers::new(VarInt(self.entity_id), &passenger_ids);
                    if let Ok(data) = client.serialize_packet(&packet) {
                        client.try_enqueue_packet(data);
                    }
                }
                ClientPlatform::Bedrock(client) => {
                    for (idx, p) in passengers.iter().enumerate() {
                        let link = pumpkin_protocol::bedrock::client::CSetActorLink {
                            link: pumpkin_protocol::bedrock::client::common::ActorLink {
                                ridden_unique_id: i64::from(self.entity_id).into(),
                                rider_unique_id: i64::from(p.get_entity().entity_id).into(),
                                link_type: if idx == 0 { 1 } else { 2 },
                                immediate: true,
                                rider_initiated: false,
                                vehicle_angular_velocity: 0.0,
                            },
                        };
                        if let Ok(data) = client.serialize_packet(&link) {
                            client.try_enqueue_packet(data);
                        }
                    }
                }
            }
        }

        if let Ok(vehicle_guard) = self.entity.get_entity().vehicle.try_lock()
            && let Some(vehicle) = vehicle_guard.as_ref().and_then(std::sync::Weak::upgrade)
            && let Ok(vehicle_passengers) = vehicle.get_entity().passengers.try_lock()
        {
            match player.client.as_ref() {
                ClientPlatform::Java(client) => {
                    let passenger_ids: Vec<VarInt> = vehicle_passengers
                        .iter()
                        .map(|p| VarInt(p.get_entity().entity_id))
                        .collect();
                    let packet =
                        CSetPassengers::new(VarInt(vehicle.get_entity().entity_id), &passenger_ids);
                    if let Ok(data) = client.serialize_packet(&packet) {
                        client.try_enqueue_packet(data);
                    }
                }
                ClientPlatform::Bedrock(client) => {
                    let vehicle_id = vehicle.get_entity().entity_id;
                    for (idx, p) in vehicle_passengers.iter().enumerate() {
                        if p.get_entity().entity_id == self.entity_id {
                            let link = pumpkin_protocol::bedrock::client::CSetActorLink {
                                link: pumpkin_protocol::bedrock::client::common::ActorLink {
                                    ridden_unique_id: i64::from(vehicle_id).into(),
                                    rider_unique_id: i64::from(self.entity_id).into(),
                                    link_type: if idx == 0 { 1 } else { 2 },
                                    immediate: true,
                                    rider_initiated: false,
                                    vehicle_angular_velocity: 0.0,
                                },
                            };
                            if let Ok(data) = client.serialize_packet(&link) {
                                client.try_enqueue_packet(data);
                            }
                        }
                    }
                }
            }
        }
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
        let seen_by = {
            let mut guard = self
                .seen_by
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *guard)
        };
        self.bedrock_watchers.store(0, Relaxed);
        if seen_by.is_empty() {
            return;
        }
        let entity_ids = [self.entity_id.into()];
        let je_packet = CRemoveEntities::new(&entity_ids);
        let be_packet = CRemoveActor::new(VarLong(i64::from(self.entity_id)));

        let mut java_recipients = Vec::with_capacity(seen_by.len());
        let mut bedrock_recipients = Vec::new();
        for p in seen_by.values() {
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => bedrock_recipients.push(be_client),
            }
        }
        let recipients_by_version =
            World::collect_java_recipients_by_version(java_recipients.into_iter());
        World::broadcast_java_grouped(&je_packet, recipients_by_version);
        World::broadcast_bedrock_grouped(&be_packet, bedrock_recipients.into_iter());
    }

    /// Vanilla `TrackedEntity.removePlayer`: despawn on the client, only if it was paired.
    pub fn remove_player(&self, player: &Player) {
        if !self
            .seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(&player.gameprofile.id)
        {
            return;
        }
        let removed = self
            .seen_by
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&player.gameprofile.id)
            .is_some();
        if removed {
            if matches!(player.client.as_ref(), ClientPlatform::Bedrock(_)) {
                self.bedrock_watchers.fetch_sub(1, Relaxed);
            }
            self.remove_pairing(player);
        }
    }

    /// Forgets the player as a viewer without a removal packet; the client already dropped it.
    pub fn forget_player(&self, player: &Player) {
        if !self
            .seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(&player.gameprofile.id)
        {
            return;
        }
        let removed = self
            .seen_by
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&player.gameprofile.id)
            .is_some();
        if removed && matches!(player.client.as_ref(), ClientPlatform::Bedrock(_)) {
            self.bedrock_watchers.fetch_sub(1, Relaxed);
        }
    }

    pub fn send_to_tracking_players<P: ClientPacket + Sync>(&self, packet: &P, _world: &World) {
        let seen_by = self
            .seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if seen_by.is_empty() {
            return;
        }
        let recipients_by_version = World::collect_java_recipients_by_version(seen_by.values());
        World::broadcast_java_grouped(packet, recipients_by_version);
    }

    pub fn send_to_tracking_players_bedrock<P: BClientPacket + Sync>(
        &self,
        packet: &P,
        _world: &World,
    ) {
        if !self.has_bedrock_watchers() {
            return;
        }
        let seen_by = self
            .seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let recipients = seen_by.values().filter_map(|p| {
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
        let seen_by = self
            .seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if seen_by.is_empty() {
            return;
        }

        let mut java_recipients = Vec::with_capacity(seen_by.len());
        let mut bedrock_recipients = Vec::new();
        for p in seen_by.values() {
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => bedrock_recipients.push(be_client),
            }
        }
        let recipients_by_version =
            World::collect_java_recipients_by_version(java_recipients.into_iter());
        World::broadcast_java_grouped(je_packet, recipients_by_version);
        World::broadcast_bedrock_grouped(be_packet, bedrock_recipients.into_iter());
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
        let seen_by = self
            .seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if seen_by.is_empty() {
            return;
        }
        let recipients = seen_by.values().filter(|p| filter(p));
        let recipients_by_version = World::collect_java_recipients_by_version(recipients);
        World::broadcast_java_grouped(packet, recipients_by_version);
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
        let seen_by = self
            .seen_by
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if seen_by.is_empty() {
            return;
        }
        let recipients = seen_by.values().filter(|p| filter(p));

        let mut java_recipients = Vec::new();
        let mut bedrock_recipients = Vec::new();
        for p in recipients {
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => bedrock_recipients.push(be_client),
            }
        }
        let recipients_by_version =
            World::collect_java_recipients_by_version(java_recipients.into_iter());
        World::broadcast_java_grouped(je_packet, recipients_by_version);
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
                let new_pos = chunk_section_from_pos(&tracked.entity.get_entity().block_pos.load());
                let old_pos = tracked.last_section_pos.load();
                let section_changed = old_pos != new_pos;
                let sub_section_moved = tracked.pos_dirty.swap(false, Relaxed);
                if !section_changed && !sub_section_moved {
                    None
                } else {
                    *moved_slot = true;
                    tracked.update_players_from_snapshots(&all_snapshots);
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
                        tracked.update_players_from_snapshots(&moved_snapshots);
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
