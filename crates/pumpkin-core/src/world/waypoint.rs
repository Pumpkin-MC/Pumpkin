//! Locator bar waypoints, a port of vanilla `ServerWaypointManager` and `WaypointTransmitter`.
//!
//! Only players transmit and receive here. Vanilla lets any living entity transmit once its
//! `waypoint_transmit_range` goes above zero, but every non player type defaults to zero.

use std::sync::Arc;

use pumpkin_data::attributes::Attributes;
use pumpkin_protocol::java::client::play::{
    CWaypoint, WaypointIcon, WaypointOperation, WaypointTarget,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use rustc_hash::FxHashMap;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;

/// Vanilla `WaypointTransmitter.REALLY_FAR_DISTANCE`.
const REALLY_FAR_DISTANCE: f32 = 332.0;
/// Smallest azimuth change in radians worth an update (vanilla `EntityAzimuthConnection.update`).
const AZIMUTH_UPDATE_THRESHOLD: f32 = 0.008_726_646;

/// Vanilla `WaypointTransmitter.Connection`, one per (receiver, source) pair.
#[derive(Clone, Copy)]
enum Connection {
    Block { last_position: BlockPos },
    Chunk { last_position: Vector2<i32> },
    Azimuth { last_angle: f32 },
}

impl Connection {
    /// Vanilla `LivingEntity.makeWaypointConnectionWith`.
    fn make(source: &Player, receiver: &Player) -> Option<Self> {
        let entity = source.get_entity();
        if entity.first_tick.load(std::sync::atomic::Ordering::Relaxed)
            || entity.entity_id == receiver.entity_id()
            || does_source_ignore_receiver(source, receiver)
        {
            return None;
        }
        if is_really_far(source, receiver) {
            Some(Self::Azimuth {
                last_angle: azimuth(source, receiver),
            })
        } else if is_chunk_visible(entity.chunk_pos.load(), receiver) {
            Some(Self::Block {
                last_position: entity.block_pos.load(),
            })
        } else {
            Some(Self::Chunk {
                last_position: entity.chunk_pos.load(),
            })
        }
    }

    const fn target(&self) -> WaypointTarget {
        match *self {
            Self::Block { last_position } => WaypointTarget::Position(last_position),
            Self::Chunk { last_position } => WaypointTarget::Chunk(last_position),
            Self::Azimuth { last_angle } => WaypointTarget::Azimuth(last_angle),
        }
    }

    fn send(&self, operation: WaypointOperation, source: &Player, receiver: &Player) {
        receiver.try_send_client_packet(&CWaypoint::new(
            operation,
            source.get_entity().entity_uuid,
            waypoint_icon(),
            self.target(),
        ));
    }

    fn connect(&self, source: &Player, receiver: &Player) {
        self.send(WaypointOperation::Track, source, receiver);
    }

    fn disconnect(source: Uuid, receiver: &Player) {
        receiver.try_send_client_packet(&CWaypoint::remove(source));
    }

    fn update(&mut self, source: &Player, receiver: &Player) {
        let entity = source.get_entity();
        match self {
            Self::Block { last_position } => {
                let current = entity.block_pos.load();
                if current.manhattan_distance(*last_position) == 0 {
                    return;
                }
                *last_position = current;
            }
            Self::Chunk { last_position } => {
                let current = entity.chunk_pos.load();
                if chessboard_distance(current, *last_position) == 0 {
                    return;
                }
                *last_position = current;
            }
            Self::Azimuth { last_angle } => {
                let current = azimuth(source, receiver);
                if (current - *last_angle).abs() <= AZIMUTH_UPDATE_THRESHOLD {
                    return;
                }
                *last_angle = current;
            }
        }
        self.send(WaypointOperation::Update, source, receiver);
    }

    fn is_broken(&self, source: &Player, receiver: &Player) -> bool {
        let entity = source.get_entity();
        match *self {
            Self::Block { last_position } => {
                last_position.manhattan_distance(entity.block_pos.load()) > 1
                    || does_source_ignore_receiver(source, receiver)
            }
            Self::Chunk { last_position } => {
                chessboard_distance(last_position, entity.chunk_pos.load()) > 1
                    || does_source_ignore_receiver(source, receiver)
                    || is_chunk_visible(last_position, receiver)
            }
            Self::Azimuth { .. } => {
                does_source_ignore_receiver(source, receiver)
                    || is_chunk_visible(entity.chunk_pos.load(), receiver)
                    || !is_really_far(source, receiver)
            }
        }
    }
}

/// The icon a player transmits with.
///
/// hmm, vanilla falls back to the team color here, but Pumpkin teams always have a color
/// (white by default) while vanilla ones start without one. Sending it would paint every team
/// member white, so we leave the color to the client, which derives it from the UUID.
const fn waypoint_icon() -> WaypointIcon<'static> {
    WaypointIcon::NULL
}

/// Vanilla `Entity.distanceTo`, done in floats like the Java.
fn distance_to(source: &Player, receiver: &Player) -> f32 {
    let a = source.get_entity().pos.load();
    let b = receiver.get_entity().pos.load();
    let dx = (a.x - b.x) as f32;
    let dy = (a.y - b.y) as f32;
    let dz = (a.z - b.z) as f32;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Vanilla `Entity.hasIndirectPassenger`: is `receiver` riding `source` at any depth.
fn has_indirect_passenger(source: &Player, receiver: &Player) -> bool {
    let source_id = source.entity_id();
    let mut vehicle = receiver.get_entity().get_vehicle();
    while let Some(current) = vehicle {
        if current.get_entity().entity_id == source_id {
            return true;
        }
        vehicle = current.get_entity().get_vehicle();
    }
    false
}

/// Vanilla `WaypointTransmitter.doesSourceIgnoreReceiver`.
fn does_source_ignore_receiver(source: &Player, receiver: &Player) -> bool {
    if receiver.is_spectator() {
        return false;
    }
    if source.is_spectator() || has_indirect_passenger(source, receiver) {
        return true;
    }
    let broadcast_range = source
        .living_entity
        .get_attribute_value(&Attributes::WAYPOINT_TRANSMIT_RANGE)
        .min(
            receiver
                .living_entity
                .get_attribute_value(&Attributes::WAYPOINT_RECEIVE_RANGE),
        );
    f64::from(distance_to(source, receiver)) >= broadcast_range
}

/// Vanilla `WaypointTransmitter.isChunkVisible`, i.e. `ChunkTrackingView.isInViewDistance`.
///
/// ok so this is not `Cylindrical::is_within_distance`, that one pads by two chunks
/// (vanilla's `includeNeighbors`), the view distance check only pads by one.
fn is_chunk_visible(chunk: Vector2<i32>, receiver: &Player) -> bool {
    let view = receiver.watched_section.load();
    let dx = i64::from(((chunk.x - view.center.x).abs() - 1).max(0));
    let dz = i64::from(((chunk.y - view.center.y).abs() - 1).max(0));
    let view_distance = std::num::NonZeroI64::from(view.view_distance).get();
    dx * dx + dz * dz < view_distance * view_distance
}

/// Vanilla `WaypointTransmitter.isReallyFar`.
fn is_really_far(source: &Player, receiver: &Player) -> bool {
    distance_to(source, receiver) > REALLY_FAR_DISTANCE
}

fn chessboard_distance(a: Vector2<i32>, b: Vector2<i32>) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

/// Angle the receiver sees the source at, vanilla `EntityAzimuthConnection`.
fn azimuth(source: &Player, receiver: &Player) -> f32 {
    let from = source.get_entity().pos.load();
    let to = receiver.get_entity().pos.load();
    // yeah, `rotateClockwise90` turns (x, y, z) into (-z, y, x), so atan2(z', x') is atan2(dx, -dz)
    pumpkin_util::math::atan2(to.x - from.x, -(to.z - from.z)) as f32
}

/// Vanilla `MinecraftServer.onGameRuleChanged` for `locator_bar`. All worlds share the rule.
pub fn on_locator_bar_changed(server: &Server) {
    for world in server.worlds.load().iter() {
        world.with_waypoints(ServerWaypointManager::on_locator_bar_changed);
    }
}

/// Per world waypoint bookkeeping, vanilla `ServerWaypointManager`.
///
/// Every method takes the world's `locator_bar` game rule value, the manager lives behind the
/// world's lock so it can't read it itself.
#[derive(Default)]
pub struct ServerWaypointManager {
    waypoints: FxHashMap<Uuid, Arc<Player>>,
    players: FxHashMap<Uuid, Arc<Player>>,
    /// receiver -> source -> connection
    connections: FxHashMap<Uuid, FxHashMap<Uuid, Connection>>,
}

impl ServerWaypointManager {
    fn is_transmitting(player: &Player) -> bool {
        player
            .living_entity
            .get_attribute_value(&Attributes::WAYPOINT_TRANSMIT_RANGE)
            > 0.0
    }

    fn is_receiving(player: &Player) -> bool {
        player
            .living_entity
            .get_attribute_value(&Attributes::WAYPOINT_RECEIVE_RANGE)
            > 0.0
    }

    pub fn track_waypoint(&mut self, waypoint: &Arc<Player>, locator_bar: bool) {
        self.waypoints
            .insert(waypoint.get_entity().entity_uuid, waypoint.clone());
        let players: Vec<_> = self.players.values().cloned().collect();
        for player in &players {
            self.create_connection(player, waypoint, locator_bar);
        }
    }

    fn update_waypoint(&mut self, waypoint: &Arc<Player>, locator_bar: bool) {
        let source = waypoint.get_entity().entity_uuid;
        // Existing connections update first, then the rest try to connect, like vanilla.
        let (connected, potential): (Vec<_>, Vec<_>) =
            self.players.values().cloned().partition(|player| {
                self.connections
                    .get(&player.get_entity().entity_uuid)
                    .is_some_and(|row| row.contains_key(&source))
            });
        for player in &connected {
            self.update_connection(player, waypoint, locator_bar);
        }
        for player in &potential {
            self.create_connection(player, waypoint, locator_bar);
        }
    }

    pub fn untrack_waypoint(&mut self, waypoint: &Player) {
        let source = waypoint.get_entity().entity_uuid;
        for (receiver, row) in &mut self.connections {
            if row.remove(&source).is_some()
                && let Some(player) = self.players.get(receiver)
            {
                Connection::disconnect(source, player);
            }
        }
        self.waypoints.remove(&source);
    }

    pub fn add_player(&mut self, player: &Arc<Player>, locator_bar: bool) {
        self.players
            .insert(player.get_entity().entity_uuid, player.clone());
        let waypoints: Vec<_> = self.waypoints.values().cloned().collect();
        for waypoint in &waypoints {
            self.create_connection(player, waypoint, locator_bar);
        }
        if Self::is_transmitting(player) {
            self.track_waypoint(player, locator_bar);
        }
    }

    fn update_player(&mut self, player: &Arc<Player>, locator_bar: bool) {
        let receiver = player.get_entity().entity_uuid;
        let (connected, potential): (Vec<_>, Vec<_>) =
            self.waypoints.values().cloned().partition(|waypoint| {
                self.connections
                    .get(&receiver)
                    .is_some_and(|row| row.contains_key(&waypoint.get_entity().entity_uuid))
            });
        for waypoint in &connected {
            self.update_connection(player, waypoint, locator_bar);
        }
        for waypoint in &potential {
            self.create_connection(player, waypoint, locator_bar);
        }
    }

    pub fn remove_player(&mut self, player: &Player) {
        let receiver = player.get_entity().entity_uuid;
        if let Some(row) = self.connections.remove(&receiver) {
            for source in row.keys() {
                Connection::disconnect(*source, player);
            }
        }
        self.untrack_waypoint(player);
        self.players.remove(&receiver);
    }

    /// Vanilla `breakAllConnections`, for when `locator_bar` gets turned off.
    pub fn break_all_connections(&mut self) {
        for (receiver, row) in self.connections.drain() {
            if let Some(player) = self.players.get(&receiver) {
                for source in row.keys() {
                    Connection::disconnect(*source, player);
                }
            }
        }
    }

    /// Vanilla `MinecraftServer.onGameRuleChanged` for `locator_bar`.
    pub fn on_locator_bar_changed(&mut self, locator_bar: bool) {
        if locator_bar {
            let players: Vec<_> = self.players.values().cloned().collect();
            for player in &players {
                self.update_player(player, locator_bar);
            }
        } else {
            self.break_all_connections();
        }
    }

    /// The waypoint half of vanilla `Entity.setPosRaw`, call it after a player's position changed.
    pub fn on_player_moved(&mut self, uuid: &Uuid, locator_bar: bool) {
        if let Some(waypoint) = self.waypoints.get(uuid).cloned() {
            self.update_waypoint(&waypoint, locator_bar);
        }
        if let Some(player) = self.players.get(uuid).cloned() {
            self.update_player(&player, locator_bar);
        }
    }

    /// Vanilla `LivingEntity.onAttributeUpdated` and `ServerPlayer.onAttributeUpdated`.
    pub fn on_attribute_updated(&mut self, player: &Arc<Player>, attribute: u8, locator_bar: bool) {
        if attribute == Attributes::WAYPOINT_TRANSMIT_RANGE.id {
            if Self::is_transmitting(player) {
                self.track_waypoint(player, locator_bar);
            } else {
                self.untrack_waypoint(player);
            }
        } else if attribute == Attributes::WAYPOINT_RECEIVE_RANGE.id {
            if Self::is_receiving(player) {
                self.add_player(player, locator_bar);
            } else {
                self.remove_player(player);
            }
        }
    }

    /// Vanilla `onTrackingStart`: a player became part of this world.
    pub fn on_tracking_start(&mut self, player: &Arc<Player>, locator_bar: bool) {
        if Self::is_receiving(player) {
            self.add_player(player, locator_bar);
        } else if Self::is_transmitting(player) {
            self.track_waypoint(player, locator_bar);
        }
    }

    fn create_connection(&mut self, player: &Player, waypoint: &Player, locator_bar: bool) {
        if player.entity_id() == waypoint.entity_id() || !locator_bar {
            return;
        }
        let receiver = player.get_entity().entity_uuid;
        let source = waypoint.get_entity().entity_uuid;
        if let Some(connection) = Connection::make(waypoint, player) {
            self.connections
                .entry(receiver)
                .or_default()
                .insert(source, connection);
            connection.connect(waypoint, player);
        } else if self
            .connections
            .get_mut(&receiver)
            .and_then(|row| row.remove(&source))
            .is_some()
        {
            Connection::disconnect(source, player);
        }
    }

    fn update_connection(&mut self, player: &Player, waypoint: &Player, locator_bar: bool) {
        if player.entity_id() == waypoint.entity_id() || !locator_bar {
            return;
        }
        let receiver = player.get_entity().entity_uuid;
        let source = waypoint.get_entity().entity_uuid;
        let Some(row) = self.connections.get_mut(&receiver) else {
            return;
        };
        let Some(connection) = row.get_mut(&source) else {
            return;
        };
        if !connection.is_broken(waypoint, player) {
            connection.update(waypoint, player);
        } else if let Some(new_connection) = Connection::make(waypoint, player) {
            new_connection.connect(waypoint, player);
            *connection = new_connection;
        } else {
            Connection::disconnect(source, player);
            row.remove(&source);
        }
    }
}
