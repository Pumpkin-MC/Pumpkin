use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bytes::BufMut;
use crossbeam::atomic::AtomicCell;
use dashmap::DashMap;
use dashmap::DashSet;
use pumpkin_protocol::bedrock::client::remove_actor::CRemoveActor;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::java::client::play::{
    CEntityVelocity, CHeadRot, CRemoveEntities, CSetEntityMetadata, CSetEquipment, CSetPassengers,
    Metadata,
};
use pumpkin_protocol::{BClientPacket, ClientPacket};
use pumpkin_util::GameMode;
use pumpkin_util::math::get_section_cord;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::version::JavaMinecraftVersion;
use rustc_hash::{FxHashMap, FxHashSet};
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::net::ClientPlatform;
use crate::net::java::JavaClient;
use crate::world::World;
use crate::world::chunker::{get_view_distance, is_within_view_distance};

/// Actor packets share Bedrock's bounded normal queue with terrain. A small per-player
/// budget prevents a dense entity chunk from filling that queue in a single world tick.
const BEDROCK_ACTOR_PAIRINGS_PER_TICK: usize = 64;

fn take_bedrock_pairing_budget(budgets: &mut FxHashMap<Uuid, usize>, player_id: Uuid) -> bool {
    let Some(remaining) = budgets.get_mut(&player_id) else {
        return false;
    };
    if *remaining == 0 {
        return false;
    }
    *remaining -= 1;
    true
}

fn player_is_in_world(player: &Player, world: &World) -> bool {
    let current_world = player.world();
    std::ptr::eq(Arc::as_ptr(&current_world), world)
}

fn is_entity_visible_to_player(
    entity: &dyn EntityBase,
    player: &Player,
    effective_range: u32,
) -> bool {
    let player_entity = player.get_entity();
    let player_pos = player_entity.pos.load();
    let entity_pos = entity.get_entity().pos.load();
    let dx = player_pos.x - entity_pos.x;
    let dz = player_pos.z - entity_pos.z;
    let dist_sq = dx.mul_add(dx, dz * dz);

    let player_vd = get_view_distance(player).get() as i32;
    let visible_range_blocks = f64::from((effective_range as i32).min(player_vd) * 16);
    let range_sq = visible_range_blocks * visible_range_blocks;
    let entity_chunk = entity.get_entity().chunk_pos.load();
    let player_chunk = player_entity.chunk_pos.load();
    let in_view = is_within_view_distance(entity_chunk, player_chunk, player_vd);
    let spectator_visible = entity.get_player().is_none_or(|target_player| {
        player.gamemode.load() == GameMode::Spectator
            || target_player.gamemode.load() != GameMode::Spectator
    });
    let chunk_ready = match player.client.as_ref() {
        ClientPlatform::Java(_) => true,
        ClientPlatform::Bedrock(_) => {
            player.bedrock_spawned.load(Ordering::Acquire)
                && player
                    .chunk_sender
                    .lock()
                    .is_ok_and(|sender| sender.is_bedrock_chunk_ready(&entity_chunk))
        }
    };

    dist_sq <= range_sq && spectator_visible && in_view && chunk_ready
}

pub struct TrackedEntity {
    pub entity: Arc<dyn EntityBase>,
    pub entity_id: i32,
    pub tracking_range: u32,
    pub update_interval: u32,
    pub track_deltas: bool,
    pub seen_by: DashSet<Uuid>,
    /// Bedrock pairings whose spawn packet could not fit in the bounded outgoing queue.
    pending_pairings: DashSet<Uuid>,
    /// Prevents concurrent chunk/entity tasks from queueing the same actor spawn twice.
    pairings_in_progress: DashSet<Uuid>,
    /// Bedrock removals which could not yet enter the normal FIFO behind their spawn.
    pending_removals: DashSet<Uuid>,
    /// Temporarily hides this actor from Bedrock clients, for example while a Java player is dead.
    bedrock_suspended: AtomicBool,
    /// Keeps a removed entity's tracker alive until all accepted Bedrock actors are removed.
    removing: AtomicBool,
    pub last_section_pos: AtomicCell<Vector3<i32>>,
}

impl TrackedEntity {
    #[must_use]
    pub fn new(
        entity: Arc<dyn EntityBase>,
        range: u32,
        update_interval: u32,
        track_deltas: bool,
    ) -> Self {
        let entity_id = entity.get_entity().entity_id;
        let pos = entity.get_entity().pos.load();
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
            pending_pairings: DashSet::new(),
            pairings_in_progress: DashSet::new(),
            pending_removals: DashSet::new(),
            bedrock_suspended: AtomicBool::new(false),
            removing: AtomicBool::new(false),
            last_section_pos: AtomicCell::new(last_section_pos),
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

    pub fn update_player(&self, player: &Arc<Player>, world: &World) {
        if self.removing.load(Ordering::Acquire)
            || player.get_entity().entity_id == self.entity_id
            || !player_is_in_world(player, world)
        {
            return;
        }

        let player_id = player.gameprofile.id;
        if matches!(player.client.as_ref(), ClientPlatform::Bedrock(_))
            && self.bedrock_suspended.load(Ordering::Acquire)
        {
            self.pending_pairings.remove(&player_id);
            if self.seen_by.contains(&player_id) {
                self.pending_removals.insert(player_id);
            }
            return;
        }

        let effective_range = self.get_effective_range();
        let is_visible = is_entity_visible_to_player(self.entity.as_ref(), player, effective_range);

        if is_visible {
            // Once a removal is pending, preserve FIFO ordering even if the actor becomes visible
            // again. The accepted removal will re-evaluate visibility and enqueue a fresh spawn.
            if self.pending_removals.contains(&player_id) {
                return;
            }
            match player.client.as_ref() {
                ClientPlatform::Java(_) => {
                    // Preserve Java's existing pairing and fire-and-forget queue behavior.
                    if self.seen_by.insert(player_id) {
                        let _ = self.add_pairing(player);
                    }
                }
                ClientPlatform::Bedrock(_) => {
                    if !self.seen_by.contains(&player_id)
                        && self.pairings_in_progress.insert(player_id)
                    {
                        if self.removing.load(Ordering::Acquire) {
                            self.pairings_in_progress.remove(&player_id);
                            return;
                        }
                        if self.add_pairing(player) {
                            self.seen_by.insert(player_id);
                            self.pending_pairings.remove(&player_id);
                            if self.removing.load(Ordering::Acquire)
                                || self.bedrock_suspended.load(Ordering::Acquire)
                                || !player_is_in_world(player, world)
                                || !is_entity_visible_to_player(
                                    self.entity.as_ref(),
                                    player,
                                    effective_range,
                                )
                            {
                                // A lifecycle transition may race the synchronous enqueue above.
                                // Keep the accepted spawn ordered before its required removal.
                                self.pending_removals.insert(player_id);
                            }
                        } else {
                            // The Bedrock outgoing queue is deliberately bounded. Leave the actor
                            // unseen and retry later instead of silently losing its spawn.
                            if !self.removing.load(Ordering::Acquire)
                                && !self.bedrock_suspended.load(Ordering::Acquire)
                            {
                                self.pending_pairings.insert(player_id);
                            }
                        }
                        self.pairings_in_progress.remove(&player_id);
                    }
                }
            }
        } else {
            let player_id = player.gameprofile.id;
            self.pending_pairings.remove(&player_id);
            match player.client.as_ref() {
                ClientPlatform::Java(_) => {
                    if self.seen_by.remove(&player_id).is_some() {
                        let _ = self.remove_pairing(player);
                    }
                }
                ClientPlatform::Bedrock(_) => {
                    if self.seen_by.contains(&player_id) {
                        self.pending_removals.insert(player_id);
                    }
                }
            }
        }
    }

    pub fn update_players(&self, players: &[Arc<Player>], world: &World) {
        for player in players {
            match player.client.as_ref() {
                ClientPlatform::Java(_) => self.update_player(player, world),
                ClientPlatform::Bedrock(_) => self.update_player_deferred(player, world),
            }
        }
    }

    /// Applies visibility changes for a Bedrock player without immediately enqueueing a spawn.
    /// Bulk scans use this path; the world-tick retry path is the sole actor-spawn rate limiter.
    fn update_player_deferred(&self, player: &Arc<Player>, world: &World) {
        if self.removing.load(Ordering::Acquire)
            || player.get_entity().entity_id == self.entity_id
            || !player_is_in_world(player, world)
        {
            return;
        }

        let player_id = player.gameprofile.id;
        if self.bedrock_suspended.load(Ordering::Acquire) {
            self.pending_pairings.remove(&player_id);
            if self.seen_by.contains(&player_id) {
                self.pending_removals.insert(player_id);
            }
            return;
        }

        if is_entity_visible_to_player(self.entity.as_ref(), player, self.get_effective_range()) {
            if !self.pending_removals.contains(&player_id) && !self.seen_by.contains(&player_id) {
                self.pending_pairings.insert(player_id);
            }
        } else {
            self.pending_pairings.remove(&player_id);
            match player.client.as_ref() {
                ClientPlatform::Java(_) => {
                    if self.seen_by.remove(&player_id).is_some() {
                        let _ = self.remove_pairing(player);
                    }
                }
                ClientPlatform::Bedrock(_) => {
                    if self.seen_by.contains(&player_id) {
                        self.pending_removals.insert(player_id);
                    }
                }
            }
        }
    }

    fn retry_pending_pairings(
        &self,
        players: &[Arc<Player>],
        world: &World,
        bedrock_budgets: &mut FxHashMap<Uuid, usize>,
    ) {
        if self.pending_pairings.is_empty() {
            return;
        }

        for player in players {
            if self.pending_pairings.contains(&player.gameprofile.id)
                && take_bedrock_pairing_budget(bedrock_budgets, player.gameprofile.id)
            {
                self.update_player(player, world);
            }
        }
    }

    fn retry_pending_removals(
        &self,
        players: &[Arc<Player>],
        world: &World,
        bedrock_budgets: &mut FxHashMap<Uuid, usize>,
    ) {
        if self.pending_removals.is_empty() {
            return;
        }

        for player in players {
            let player_id = player.gameprofile.id;
            if self.pending_removals.contains(&player_id)
                && take_bedrock_pairing_budget(bedrock_budgets, player_id)
                && self.remove_pairing(player)
            {
                self.seen_by.remove(&player_id);
                self.pending_removals.remove(&player_id);
                if !self.removing.load(Ordering::Acquire) {
                    // Visibility may have changed while the full queue delayed this removal.
                    // Re-evaluate now so a visible actor is respawned after the accepted remove.
                    self.update_player_deferred(player, world);
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn add_pairing(&self, player: &Arc<Player>) -> bool {
        if !player.client.try_enqueue_spawn_packet(&self.entity) {
            return false;
        }

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

        true
    }

    pub fn remove_pairing(&self, player: &Arc<Player>) -> bool {
        let entity_ids = [self.entity_id.into()];
        match player.client.as_ref() {
            ClientPlatform::Java(client) => {
                let packet = CRemoveEntities::new(&entity_ids);
                if let Ok(data) = client.serialize_packet(&packet) {
                    client.try_enqueue_packet(data);
                }
                true
            }
            ClientPlatform::Bedrock(client) => {
                let packet = CRemoveActor::new(VarLong(i64::from(self.entity_id)));
                client
                    .serialize_packet(&packet)
                    .is_ok_and(|data| client.try_enqueue_packet_data_checked(data))
            }
        }
    }

    fn suspend_bedrock(&self, world: &World) {
        if self.removing.load(Ordering::Acquire) {
            return;
        }
        self.bedrock_suspended.store(true, Ordering::Release);
        self.pending_pairings.clear();
        let players = world.players.load();
        for player in players.iter() {
            let player_id = player.gameprofile.id;
            if matches!(player.client.as_ref(), ClientPlatform::Bedrock(_))
                && self.seen_by.contains(&player_id)
            {
                self.pending_removals.insert(player_id);
            }
        }
    }

    fn resume_bedrock(&self, world: &World) {
        if self.removing.load(Ordering::Acquire) {
            return;
        }
        self.bedrock_suspended.store(false, Ordering::Release);
        let players = world.players.load();
        for player in players.iter() {
            if matches!(player.client.as_ref(), ClientPlatform::Bedrock(_)) {
                if self.seen_by.contains(&player.gameprofile.id) {
                    // Respawn may be requested before the delayed death-time suspension ran.
                    // Always refresh an already-paired actor so the client cannot retain its
                    // non-interactable death state.
                    self.pending_removals.insert(player.gameprofile.id);
                }
                self.update_player_deferred(player, world);
            }
        }
    }

    pub fn begin_removal(&self, world: &World) {
        self.removing.store(true, Ordering::Release);
        self.pending_pairings.clear();
        let players = world.players.load();
        for player in players.iter() {
            let player_id = player.gameprofile.id;
            if !self.seen_by.contains(&player_id) {
                continue;
            }
            match player.client.as_ref() {
                ClientPlatform::Java(_) => {
                    let _ = self.remove_pairing(player);
                    self.seen_by.remove(&player_id);
                }
                ClientPlatform::Bedrock(_) => {
                    self.pending_removals.insert(player_id);
                }
            }
        }
    }

    fn removal_complete(&self) -> bool {
        self.removing.load(Ordering::Acquire)
            && self.seen_by.is_empty()
            && self.pending_removals.is_empty()
            && self.pairings_in_progress.is_empty()
    }

    pub fn remove_player(&self, player_uuid: &Uuid) {
        self.seen_by.remove(player_uuid);
        self.pending_pairings.remove(player_uuid);
        self.pairings_in_progress.remove(player_uuid);
        self.pending_removals.remove(player_uuid);
    }

    #[must_use]
    pub fn has_active_bedrock_pairing(&self, player_uuid: &Uuid) -> bool {
        !self.removing.load(Ordering::Acquire)
            && !self.bedrock_suspended.load(Ordering::Acquire)
            && self.seen_by.contains(player_uuid)
            && !self.pending_removals.contains(player_uuid)
    }

    pub fn send_to_tracking_players<P: ClientPacket + Sync>(&self, packet: &P, world: &World) {
        let players = world.players.load();
        let recipients = players
            .iter()
            .filter(|p| player_is_in_world(p, world) && self.seen_by.contains(&p.gameprofile.id));
        let recipients_by_version = World::collect_java_recipients_by_version(recipients);
        World::broadcast_java_grouped(packet, recipients_by_version);
    }

    pub fn send_to_tracking_players_bedrock<P: BClientPacket + Sync>(
        &self,
        packet: &P,
        world: &World,
    ) {
        if self.removing.load(Ordering::Acquire) || self.bedrock_suspended.load(Ordering::Acquire) {
            return;
        }
        let players = world.players.load();
        let recipients = players.iter().filter_map(|p| {
            if player_is_in_world(p, world)
                && self.seen_by.contains(&p.gameprofile.id)
                && !self.pending_removals.contains(&p.gameprofile.id)
                && let ClientPlatform::Bedrock(client) = p.client.as_ref()
            {
                return Some(client);
            }
            None
        });
        World::broadcast_bedrock_grouped(packet, recipients);
    }

    pub fn send_to_tracking_players_and_self_bedrock_filtered<
        P: BClientPacket + Sync,
        F: Fn(&Player) -> bool,
    >(
        &self,
        packet: &P,
        world: &World,
        filter: F,
    ) {
        if self.removing.load(Ordering::Acquire) || self.bedrock_suspended.load(Ordering::Acquire) {
            return;
        }
        let players = world.players.load();
        let recipients = players.iter().filter_map(|player| {
            let tracks_entity = self.seen_by.contains(&player.gameprofile.id)
                || player.entity_id() == self.entity_id;
            if player_is_in_world(player, world)
                && tracks_entity
                && !self.pending_removals.contains(&player.gameprofile.id)
                && filter(player)
                && let ClientPlatform::Bedrock(client) = player.client.as_ref()
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
        let players = world.players.load();
        let recipients = players
            .iter()
            .filter(|p| player_is_in_world(p, world) && self.seen_by.contains(&p.gameprofile.id));

        let mut java_recipients = Vec::new();
        let mut bedrock_recipients = Vec::new();
        for p in recipients {
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => {
                    if !self.removing.load(Ordering::Acquire)
                        && !self.bedrock_suspended.load(Ordering::Acquire)
                        && !self.pending_removals.contains(&p.gameprofile.id)
                    {
                        bedrock_recipients.push(be_client);
                    }
                }
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
        let recipients = players.iter().filter(|p| {
            player_is_in_world(p, world) && self.seen_by.contains(&p.gameprofile.id) && filter(p)
        });
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
        let recipients = players.iter().filter(|p| {
            player_is_in_world(p, world) && self.seen_by.contains(&p.gameprofile.id) && filter(p)
        });

        let mut java_recipients = Vec::new();
        let mut bedrock_recipients = Vec::new();
        for p in recipients {
            match p.client.as_ref() {
                ClientPlatform::Java(_) => java_recipients.push(p),
                ClientPlatform::Bedrock(be_client) => {
                    if !self.removing.load(Ordering::Acquire)
                        && !self.bedrock_suspended.load(Ordering::Acquire)
                        && !self.pending_removals.contains(&p.gameprofile.id)
                    {
                        bedrock_recipients.push(be_client);
                    }
                }
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
        for player in players.iter() {
            match player.client.as_ref() {
                ClientPlatform::Java(_) => tracked.update_player(player, world),
                ClientPlatform::Bedrock(_) => tracked.update_player_deferred(player, world),
            }
        }
    }

    /// Must only be called after the player's own `CLogin` packet has been sent.
    pub fn pair_new_player_with_tracked_entities(&self, player_arc: &Arc<Player>, world: &World) {
        let entity_id = player_arc.get_entity().entity_id;
        for entry in &self.entity_map {
            if *entry.key() != entity_id {
                match player_arc.client.as_ref() {
                    ClientPlatform::Java(_) => entry.value().update_player(player_arc, world),
                    ClientPlatform::Bedrock(_) => {
                        entry.value().update_player_deferred(player_arc, world);
                    }
                }
            }
        }
    }

    pub fn remove_entity(&self, entity: &dyn EntityBase, world: &World) {
        let entity_id = entity.get_entity().entity_id;
        if let Some(player) = entity.get_player() {
            let player_id = player.gameprofile.id;
            for entry in &self.entity_map {
                entry.value().remove_player(&player_id);
            }
        }

        if let Some(tracked) = self.get_tracked_entity(entity_id) {
            tracked.begin_removal(world);
            if tracked.removal_complete() {
                self.entity_map.remove(&entity_id);
            }
        }
    }

    pub fn suspend_bedrock_entity(&self, entity_id: i32, world: &World) {
        if let Some(tracked) = self.get_tracked_entity(entity_id) {
            tracked.suspend_bedrock(world);
        }
    }

    pub fn resume_bedrock_entity(&self, entity_id: i32, world: &World) {
        if let Some(tracked) = self.get_tracked_entity(entity_id) {
            tracked.resume_bedrock(world);
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
                match player.client.as_ref() {
                    ClientPlatform::Java(_) => entry.value().update_player(player, world),
                    ClientPlatform::Bedrock(_) => {
                        entry.value().update_player_deferred(player, world);
                    }
                }
            }
        }
    }

    /// Re-evaluates one player's entity pairings for a newly available set of chunks.
    /// This is used after Bedrock level-chunk delivery, so actor packets cannot overtake terrain.
    pub fn update_player_for_chunks(
        &self,
        player: &Arc<Player>,
        world: &World,
        chunks: &FxHashSet<Vector2<i32>>,
    ) {
        for entry in &self.entity_map {
            if chunks.contains(&entry.value().entity.get_entity().chunk_pos.load()) {
                player.try_restore_vehicle(&entry.value().entity);
                entry.value().update_player_deferred(player, world);
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

    /// Advances Bedrock actor spawn/removal queues without ticking entity simulation. This also
    /// runs while game ticks are frozen so chunk streaming cannot leave joins or despawns stuck.
    pub fn update_bedrock_lifecycles(&self, world: &World) {
        let players = world.players.load();
        let mut bedrock_pairing_budgets = players
            .iter()
            .filter_map(|player| {
                matches!(player.client.as_ref(), ClientPlatform::Bedrock(_))
                    .then_some((player.gameprofile.id, BEDROCK_ACTOR_PAIRINGS_PER_TICK))
            })
            .collect::<FxHashMap<_, _>>();
        let mut completed_removals = Vec::new();

        // Drain lifecycle removals before spending any of the shared per-player budget on
        // new pairings. This prevents a large spawn backlog from delaying despawns for seconds.
        for entry in &self.entity_map {
            let tracked = entry.value();
            tracked.retry_pending_removals(players.as_ref(), world, &mut bedrock_pairing_budgets);
            if tracked.removing.load(Ordering::Acquire) && tracked.removal_complete() {
                completed_removals.push(tracked.entity_id);
            }
        }

        for entity_id in completed_removals {
            self.entity_map.remove(&entity_id);
        }

        for entry in &self.entity_map {
            let tracked = entry.value();
            if tracked.removing.load(Ordering::Acquire) {
                continue;
            }
            tracked.retry_pending_pairings(players.as_ref(), world, &mut bedrock_pairing_budgets);
        }
    }

    pub fn update_all(&self, world: &World) {
        self.update_bedrock_lifecycles(world);

        let players = world.players.load();
        let mut moved_players = Vec::new();
        for entry in &self.entity_map {
            let tracked = entry.value();
            if tracked.removing.load(Ordering::Acquire) {
                continue;
            }
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
                for player in &moved_players {
                    match player.client.as_ref() {
                        ClientPlatform::Java(_) => entry.value().update_player(player, world),
                        ClientPlatform::Bedrock(_) => {
                            entry.value().update_player_deferred(player, world);
                        }
                    }
                }
            }
        }

        for entry in &self.entity_map {
            let tracked = entry.value();
            if tracked.entity.get_entity().synched_data.is_dirty() {
                tracked.entity.get_entity().send_dirty_entity_data();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BEDROCK_ACTOR_PAIRINGS_PER_TICK, take_bedrock_pairing_budget};
    use rustc_hash::FxHashMap;
    use uuid::Uuid;

    #[test]
    fn bedrock_actor_pairing_budget_is_strictly_bounded_per_tick() {
        let player_id = Uuid::from_u128(1);
        let mut budgets = FxHashMap::from_iter([(player_id, BEDROCK_ACTOR_PAIRINGS_PER_TICK)]);

        let accepted = (0..BEDROCK_ACTOR_PAIRINGS_PER_TICK + 10)
            .filter(|_| take_bedrock_pairing_budget(&mut budgets, player_id))
            .count();

        assert_eq!(accepted, BEDROCK_ACTOR_PAIRINGS_PER_TICK);
        assert_eq!(budgets[&player_id], 0);
    }
}
