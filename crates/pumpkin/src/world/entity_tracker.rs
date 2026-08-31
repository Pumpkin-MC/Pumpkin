use std::sync::Arc;

use bytes::BufMut;
use dashmap::DashMap;
use dashmap::DashSet;
use pumpkin_protocol::bedrock::client::remove_actor::CRemoveActor;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::java::client::play::{
    CEntityVelocity, CHeadRot, CRemoveEntities, CSetEntityMetadata, Metadata,
};
use pumpkin_protocol::{BClientPacket, ClientPacket};
use pumpkin_util::version::JavaMinecraftVersion;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::net::ClientPlatform;
use crate::net::java::JavaClient;
use crate::world::World;
use crate::world::chunker::{get_view_distance, is_within_view_distance};

/// Represents an entity tracked in the world and its current watchers,
/// corresponding to Vanilla's `ChunkMap.TrackedEntity`.
pub struct TrackedEntity {
    pub entity: Arc<dyn EntityBase>,
    pub entity_id: i32,
    pub tracking_range: u32,
    pub update_interval: u32,
    pub track_deltas: bool,
    pub seen_by: DashSet<Uuid>,
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
        Self {
            entity,
            entity_id,
            tracking_range: range,
            update_interval,
            track_deltas,
            seen_by: DashSet::new(),
        }
    }

    /// Gets the effective tracking range in chunks, taking passengers into account.
    #[must_use]
    pub fn get_effective_range(&self) -> u32 {
        let mut effective_range = self.tracking_range;
        if let Ok(passengers) = self.entity.get_entity().passengers.try_lock() {
            for passenger in passengers.iter() {
                let passenger_range = passenger.get_entity().entity_type.client_tracking_range;
                if passenger_range > effective_range {
                    effective_range = passenger_range;
                }
            }
        }
        effective_range
    }

    /// Updates visibility for a single player.
    pub fn update_player(&self, player: &Arc<Player>, _world: &World) {
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
        let effective_range = self.get_effective_range();
        let visible_range_blocks = f64::from((effective_range as i32).min(player_vd) * 16);
        let range_sq = visible_range_blocks * visible_range_blocks;

        let entity_chunk = self.entity.get_entity().chunk_pos.load();
        let player_chunk = player_entity.chunk_pos.load();
        let in_view = is_within_view_distance(entity_chunk, player_chunk, player_vd);

        let is_visible = dist_sq <= range_sq && in_view;

        if is_visible {
            if self.seen_by.insert(player.gameprofile.id) {
                self.add_pairing(player);
            }
        } else if self.seen_by.remove(&player.gameprofile.id).is_some() {
            self.remove_pairing(player);
        }
    }

    /// Updates visibility for a list of players.
    pub fn update_players(&self, players: &[Arc<Player>], world: &World) {
        for player in players {
            self.update_player(player, world);
        }
    }

    /// Sends spawn and initial state packets to the new watcher.
    pub fn add_pairing(&self, player: &Arc<Player>) {
        player.client.try_enqueue_spawn_packet(&self.entity);

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
    }

    /// Sends despawn packet to a player leaving visibility range.
    pub fn remove_pairing(&self, player: &Arc<Player>) {
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

    /// Broadcasts removal of this entity to all current watchers.
    pub fn broadcast_removed(&self, world: &World) {
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
    }

    /// Removes a player by UUID without sending despawn packet (used on disconnect).
    pub fn remove_player(&self, player_uuid: &Uuid) {
        self.seen_by.remove(player_uuid);
    }

    /// Sends a Java client packet to all tracking players.
    pub fn send_to_tracking_players<P: ClientPacket + Sync>(&self, packet: &P, world: &World) {
        let players = world.players.load();
        let recipients = players
            .iter()
            .filter(|p| self.seen_by.contains(&p.gameprofile.id));
        let recipients_by_version = World::collect_java_recipients_by_version(recipients);
        World::broadcast_java_grouped(packet, recipients_by_version);
    }

    /// Sends a Bedrock client packet to all tracking players.
    pub fn send_to_tracking_players_bedrock<P: BClientPacket + Sync>(
        &self,
        packet: &P,
        world: &World,
    ) {
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

    /// Sends an editioned packet (Java + Bedrock) to all tracking players.
    pub fn send_to_tracking_players_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
        world: &World,
    ) {
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

    /// Sends a packet to all tracking players and the entity itself if it is a player.
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

    /// Sends an editioned packet to all tracking players and the entity itself if it is a player.
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

    /// Sends a packet to tracking players filtered by predicate.
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

    /// Sends an editioned packet to tracking players filtered by predicate.
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

/// The entity tracking system for a world, corresponding to Vanilla's `ChunkMap.entityMap`.
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

        // If the newly added entity is a player, update all other tracked entities for this player
        if let Some(player) = entity.get_player()
            && let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id)
        {
            for entry in &self.entity_map {
                if *entry.key() != entity_id {
                    entry.value().update_player(&player_arc, world);
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

        if let Some((_, tracked)) = self.entity_map.remove(&entity_id) {
            tracked.broadcast_removed(world);
        }
    }

    pub fn update_player_position(&self, player: &Arc<Player>, world: &World) {
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
            let players = world.players.load();
            tracked.update_players(players.as_ref(), world);
        }
    }

    pub fn update_all(&self, world: &World) {
        let players = world.players.load();
        for entry in &self.entity_map {
            entry.value().update_players(players.as_ref(), world);
        }
    }
}
