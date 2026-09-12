use std::sync::Arc;
use std::sync::atomic::{
    AtomicBool, AtomicU32,
    Ordering::{self, Relaxed},
};

use bytes::BufMut;
use crossbeam::atomic::AtomicCell;
use dashmap::DashMap;
use dashmap::DashSet;
use pumpkin_data::entity::EntityType;
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
use pumpkin_util::math::get_section_cord;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::version::JavaMinecraftVersion;
use rustc_hash::FxHashSet;
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

/// Vanilla `Mth.packDegrees`.
fn pack_degrees(degrees: f32) -> u8 {
    (degrees * 256.0 / 360.0).floor() as i32 as u8
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

pub struct TrackedEntity {
    pub entity: Arc<dyn EntityBase>,
    pub entity_id: i32,
    pub tracking_range: u32,
    pub update_interval: u32,
    pub track_deltas: bool,
    pub seen_by: DashSet<Uuid>,
    /// Bedrock players in `seen_by`.
    bedrock_watchers: AtomicU32,
    pub last_section_pos: AtomicCell<Vector3<i32>>,
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
        let last_section_pos = Vector3::new(
            get_section_cord(pos.x.floor() as i32),
            get_section_cord(pos.y.floor() as i32),
            get_section_cord(pos.z.floor() as i32),
        );
        Self {
            entity,
            entity_id,
            tracking_range: range,
            update_interval,
            track_deltas,
            seen_by: DashSet::new(),
            bedrock_watchers: AtomicU32::new(0),
            last_section_pos: AtomicCell::new(last_section_pos),
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

    /// Vanilla `ServerEntity.sendChanges`. Non-player entities only.
    pub fn send_changes(&self, world: &World) {
        let entity = self.entity.get_entity();
        self.send_bedrock_move(entity, world);
        let tick = self.tick_count.fetch_add(1, Relaxed);
        let needs_sync = entity.velocity_dirty.swap(false, Ordering::SeqCst);
        if !(tick.is_multiple_of(self.update_interval.max(1))
            || needs_sync
            || entity.synched_data.is_dirty())
        {
            return;
        }

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
        let diff = (velocity - entity.last_sent_velocity.load()).length_squared();
        if diff > 1.0e-7 || (diff > 0.0 && velocity.length_squared() == 0.0) {
            entity.last_sent_velocity.store(velocity);
            self.send_to_tracking_players_editioned(
                &CEntityVelocity::new(VarInt(self.entity_id), velocity),
                &CSetActorMotion {
                    target_runtime_id: VarULong(self.entity_id as u64),
                    motion: Vector3::new(velocity.x as f32, velocity.y as f32, velocity.z as f32),
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
        self.update_player_in_range(player, self.get_effective_range());
    }

    fn update_player_in_range(&self, player: &Arc<Player>, effective_range: u32) {
        if player.get_entity().entity_id == self.entity_id {
            return;
        }

        let player_entity = player.get_entity();
        let player_pos = player_entity.pos.load();
        let entity_pos = self.entity.get_entity().pos.load();
        let dx = player_pos.x - entity_pos.x;
        let dz = player_pos.z - entity_pos.z;
        let dist_sq = dx.mul_add(dx, dz * dz);

        let player_vd = get_view_distance(player).get() as i32;
        let visible_range_blocks = f64::from((effective_range as i32).min(player_vd) * 16);
        let range_sq = visible_range_blocks * visible_range_blocks;

        let entity_chunk = self.entity.get_entity().chunk_pos.load();
        let in_view = player
            .watched_section
            .load()
            .is_within_distance(entity_chunk.x, entity_chunk.y);

        // Vanilla `isChunkTracked`: never spawn before the chunk packet.
        let is_visible = dist_sq <= range_sq
            && self.broadcast_to_player(player)
            && in_view
            && player
                .chunk_sender
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_chunk_ready(&entity_chunk);

        if is_visible {
            if self.seen_by.insert(player.gameprofile.id) {
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
        let effective_range = self.get_effective_range();
        for player in players {
            self.update_player_in_range(player, effective_range);
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
                let version = client.version.load();
                if version >= JavaMinecraftVersion::V_1_21 {
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
                }

                let head_yaw = target_entity.head_yaw.load();
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
            let head_rot_packet = CHeadRot::new(
                self.entity_id.into(),
                (head_yaw * 256.0 / 360.0).rem_euclid(256.0) as u8,
            );
            if let Ok(data) = client.serialize_packet(&head_rot_packet) {
                client.try_enqueue_packet(data);
            }
        }

        let vel = self.entity.get_entity().velocity.load();
        if vel.length_squared() > 1e-4
            && let ClientPlatform::Java(client) = player.client.as_ref()
        {
            let motion = CEntityVelocity::new(self.entity_id.into(), vel);
            if let Ok(data) = client.serialize_packet(&motion) {
                client.try_enqueue_packet(data);
            }
        }

        if let ClientPlatform::Java(client) = player.client.as_ref() {
            let version = client.version.load();
            // TODO: Support older versions
            if version >= JavaMinecraftVersion::V_26_2
                && let Some(non_default) = self
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

        if let Some(living) = self.entity.get_living_entity()
            && let Ok(equipment_guard) = living.entity_equipment.try_lock()
        {
            let mut equipment_list = Vec::new();
            for (slot, item_stack) in &equipment_guard.equipment {
                if !item_stack.is_empty() {
                    equipment_list.push((slot.discriminant(), item_stack.clone()));
                }
            }
            if !equipment_list.is_empty() {
                let equipment: Vec<(i8, ItemStackSerializer)> = equipment_list
                    .iter()
                    .map(|(slot, stack)| (*slot, ItemStackSerializer::from(stack.clone())))
                    .collect();
                let packet = CSetEquipment::new(self.entity_id.into(), equipment);
                if let ClientPlatform::Java(client) = player.client.as_ref()
                    && let Ok(data) = client.serialize_packet(&packet)
                {
                    client.try_enqueue_packet(data);
                }
            }
        }

        if let Ok(passengers) = self.entity.get_entity().passengers.try_lock()
            && !passengers.is_empty()
        {
            let passenger_ids: Vec<VarInt> = passengers
                .iter()
                .map(|p| VarInt(p.get_entity().entity_id))
                .collect();
            let packet = CSetPassengers::new(VarInt(self.entity_id), &passenger_ids);
            if let ClientPlatform::Java(client) = player.client.as_ref()
                && let Ok(data) = client.serialize_packet(&packet)
            {
                client.try_enqueue_packet(data);
            }
        }

        if let Ok(vehicle_guard) = self.entity.get_entity().vehicle.try_lock()
            && let Some(vehicle) = vehicle_guard.as_ref()
            && let Ok(vehicle_passengers) = vehicle.get_entity().passengers.try_lock()
        {
            let passenger_ids: Vec<VarInt> = vehicle_passengers
                .iter()
                .map(|p| VarInt(p.get_entity().entity_id))
                .collect();
            let packet =
                CSetPassengers::new(VarInt(vehicle.get_entity().entity_id), &passenger_ids);
            if let ClientPlatform::Java(client) = player.client.as_ref()
                && let Ok(data) = client.serialize_packet(&packet)
            {
                client.try_enqueue_packet(data);
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

    pub fn broadcast_removed(&self, world: &World) {
        if self.seen_by.is_empty() {
            return;
        }
        let entity_ids = [self.entity_id.into()];
        let je_packet = CRemoveEntities::new(&entity_ids);
        let be_packet = CRemoveActor::new(VarLong(i64::from(self.entity_id)));

        let players = world.players.load();
        let recipients = players
            .iter()
            .filter(|p| self.seen_by.contains(&p.gameprofile.id));

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
        World::broadcast_java_grouped(&je_packet, recipients_by_version);
        World::broadcast_bedrock_grouped(&be_packet, bedrock_recipients.into_iter());

        self.seen_by.clear();
        self.bedrock_watchers.store(0, Relaxed);
    }

    /// Vanilla `TrackedEntity.removePlayer`: despawn on the client, only if it was paired.
    pub fn remove_player(&self, player: &Player) {
        if self.seen_by.remove(&player.gameprofile.id).is_some() {
            if matches!(player.client.as_ref(), ClientPlatform::Bedrock(_)) {
                self.bedrock_watchers.fetch_sub(1, Relaxed);
            }
            self.remove_pairing(player);
        }
    }

    pub fn send_to_tracking_players<P: ClientPacket + Sync>(&self, packet: &P, world: &World) {
        if self.seen_by.is_empty() {
            return;
        }
        let players = world.players.load();
        let recipients = players
            .iter()
            .filter(|p| self.seen_by.contains(&p.gameprofile.id));
        let recipients_by_version = World::collect_java_recipients_by_version(recipients);
        World::broadcast_java_grouped(packet, recipients_by_version);
    }

    pub fn send_to_tracking_players_bedrock<P: BClientPacket + Sync>(
        &self,
        packet: &P,
        world: &World,
    ) {
        if !self.has_bedrock_watchers() {
            return;
        }
        let players = world.players.load();
        let recipients = players.iter().filter_map(|p| {
            if self.seen_by.contains(&p.gameprofile.id)
                && let ClientPlatform::Bedrock(client) = p.client.as_ref()
            {
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
        world: &World,
    ) {
        if self.seen_by.is_empty() {
            return;
        }
        let players = world.players.load();
        let recipients = players
            .iter()
            .filter(|p| self.seen_by.contains(&p.gameprofile.id));

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
        world: &World,
        filter: F,
    ) {
        let players = world.players.load();
        let recipients = players
            .iter()
            .filter(|p| self.seen_by.contains(&p.gameprofile.id) && filter(p));
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
        world: &World,
        filter: F,
    ) {
        let players = world.players.load();
        let recipients = players
            .iter()
            .filter(|p| self.seen_by.contains(&p.gameprofile.id) && filter(p));

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
            .is_some_and(|t| !t.seen_by.is_empty())
    }

    pub fn for_each_entity_tracked_by<F: FnMut(&Arc<dyn EntityBase>)>(
        &self,
        player: &Player,
        mut f: F,
    ) {
        for entry in &self.entity_map {
            if entry.value().seen_by.contains(&player.gameprofile.id) {
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

    /// Pairs entities in chunks whose packet was just queued for `player`.
    pub fn update_player_chunks(
        &self,
        player: &Arc<Player>,
        world: &World,
        chunks: &[Vector2<i32>],
    ) {
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

    pub fn update_player_position(&self, player: &Arc<Player>, world: &World) {
        let pos = player.get_entity().pos.load();
        let new_pos = Vector3::new(
            get_section_cord(pos.x.floor() as i32),
            get_section_cord(pos.y.floor() as i32),
            get_section_cord(pos.z.floor() as i32),
        );
        if let Some(tracked) = self.entity_map.get(&player.get_entity().entity_id) {
            tracked.last_section_pos.store(new_pos);
        }
        for entry in &self.entity_map {
            if *entry.key() == player.get_entity().entity_id {
                let players = world.players.load();
                entry.value().update_players(players.as_ref(), world);
            } else {
                entry.value().update_player(player, world);
            }
        }
    }

    pub fn update_entity_position(&self, entity: &dyn EntityBase, world: &World) {
        if let Some(tracked) = self.entity_map.get(&entity.get_entity().entity_id) {
            let pos = entity.get_entity().pos.load();
            let new_pos = Vector3::new(
                get_section_cord(pos.x.floor() as i32),
                get_section_cord(pos.y.floor() as i32),
                get_section_cord(pos.z.floor() as i32),
            );
            tracked.last_section_pos.store(new_pos);
            let players = world.players.load();
            tracked.update_players(players.as_ref(), world);
        }
    }

    pub fn update_all(&self, world: &World) {
        let players = world.players.load();
        let mut moved_players = Vec::new();

        for entry in &self.entity_map {
            let tracked = entry.value();
            let pos = tracked.entity.get_entity().pos.load();
            let new_pos = Vector3::new(
                get_section_cord(pos.x.floor() as i32),
                get_section_cord(pos.y.floor() as i32),
                get_section_cord(pos.z.floor() as i32),
            );
            let old_pos = tracked.last_section_pos.load();
            if old_pos != new_pos {
                tracked.update_players(players.as_ref(), world);
                if let Some(player) = tracked.entity.get_player()
                    && let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id)
                {
                    moved_players.push(player_arc);
                }
                tracked.last_section_pos.store(new_pos);
            }
        }

        if !moved_players.is_empty() {
            for entry in &self.entity_map {
                entry.value().update_players(&moved_players, world);
            }
        }

        for entry in &self.entity_map {
            let tracked = entry.value();
            if tracked.entity.get_player().is_none() {
                tracked.send_changes(world);
            } else if tracked.entity.get_entity().synched_data.is_dirty() {
                tracked.entity.get_entity().send_dirty_entity_data();
            }
        }
    }
}
