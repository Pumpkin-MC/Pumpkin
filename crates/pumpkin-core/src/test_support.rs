//! A server in a temporary folder and Java players on in-memory connections, for tests
//! that need real players.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Weak};

use pumpkin_config::telemetry::TelemetryConfig;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_protocol::java::packet_decoder::TCPNetworkDecoder;
use pumpkin_protocol::packet::MultiVersionJavaPacket;
use pumpkin_protocol::{ConnectionState, RawPacket};
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
use tokio::io::{DuplexStream, ReadHalf};
use uuid::Uuid;

use crate::data::VanillaData;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::net::java::JavaClient;
use crate::net::java::pending::PendingConnection;
use crate::net::{ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig};
use crate::server::Server;
use crate::world::chunker::get_view_distance;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Ends a read of everything queued so far. No real packet has this id.
const MARKER: i32 = i32::MAX;

pub async fn server(folder: &tempfile::TempDir) -> Result<Arc<Server>, Box<dyn std::error::Error>> {
    server_advanced(folder, |_| {}).await
}

/// A server with the values `configure` sets.
pub async fn server_advanced(
    folder: &tempfile::TempDir,
    configure: impl FnOnce(&mut AdvancedConfiguration),
) -> Result<Arc<Server>, Box<dyn std::error::Error>> {
    let mut advanced = AdvancedConfiguration::default();
    configure(&mut advanced);
    let basic = BasicConfiguration {
        default_level_name: folder.path().to_string_lossy().into_owned(),
        allow_nether: false,
        allow_end: false,
        ..Default::default()
    };
    let telemetry = TelemetryConfig {
        enabled: false,
        ..Default::default()
    };
    Ok(Server::new(
        basic,
        advanced,
        telemetry,
        VanillaData::default(),
        Vec::new(),
    )
    .await?)
}

pub async fn shutdown(server: &Server) {
    for world in server.worlds.load().iter() {
        world.level.shutdown().await;
    }
}

/// A Java player and the packets its client receives.
pub struct Client {
    pub player: Arc<Player>,
    packets: TCPNetworkDecoder<ReadHalf<DuplexStream>>,
}

impl Client {
    /// A player standing at `pos` in the first world, the chunks around it already sent.
    pub fn join(server: &Arc<Server>, id: u64, pos: Vector3<f64>) -> Self {
        let (client_end, server_end) = tokio::io::duplex(1 << 20);
        let (read, write) = tokio::io::split(server_end);
        let pending = PendingConnection::from_transport(
            Box::new(read),
            Box::new(write),
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            id,
            PacketRateLimiter::new(false, 0.0, 0.0),
            Weak::new(),
        );
        let profile = GameProfile {
            id: Uuid::from_u128(u128::from(id)),
            name: format!("player{id}"),
            properties: arc_swap::ArcSwap::default(),
            profile_actions: None,
        };
        let mut client =
            JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
        client.connection_state.store(ConnectionState::Play);
        client.start_outgoing_packet_task();

        let world = server.worlds.load()[0].clone();
        let player = Arc::new(Player::new(
            Arc::new(ClientPlatform::Java(client)),
            profile,
            PlayerConfig::default(),
            &world,
            GameMode::Survival,
        ));
        player.client.set_player(player.clone());
        let entity = player.get_entity();
        entity.set_pos(pos);
        let chunk = entity.chunk_pos.load();
        let view_distance = get_view_distance(&player);
        player
            .watched_section
            .store(Cylindrical::new(chunk, view_distance));
        {
            let mut chunk_sender = player
                .chunk_sender
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for x in -4..=4 {
                for z in -4..=4 {
                    chunk_sender.mark_sent_out_of_band(chunk.add_raw(x, z));
                }
            }
        }
        let _ = world.add_player(&player);
        let tracked: Arc<dyn EntityBase> = player.clone();
        world.entity_tracker.add_entity(&tracked, &world);
        world
            .entity_tracker
            .pair_new_player_with_tracked_entities(&player, &world);
        let (read, _write) = tokio::io::split(client_end);
        Self {
            player,
            packets: TCPNetworkDecoder::new(read),
        }
    }

    pub fn entity_id(&self) -> i32 {
        self.player.get_entity().entity_id
    }

    /// Every packet queued for this client up to now.
    pub async fn received(&mut self) -> Result<Vec<RawPacket>, Box<dyn std::error::Error>> {
        if let Some(java) = self.player.client.java() {
            java.try_enqueue_raw_packet_direct(MARKER, &[]);
        }
        let mut packets = Vec::new();
        loop {
            let packet = self.packets.get_raw_packet().await?;
            if packet.id == MARKER {
                return Ok(packets);
            }
            packets.push(packet);
        }
    }
}

pub fn id<P: MultiVersionJavaPacket>() -> i32 {
    P::to_id(CURRENT_MC_VERSION)
}

pub fn count<P: MultiVersionJavaPacket>(packets: &[RawPacket]) -> usize {
    packets.iter().filter(|p| p.id == id::<P>()).count()
}
