//! Isolated real server/player fixtures for tests spanning packet handlers and world state.
//!
//! No server listener or ticker is started. Tests drive the production operations explicitly,
//! and must call `shutdown` before dropping the temporary world to join its chunk workers.

use std::num::NonZeroU8;
use std::sync::{Arc, Mutex, RwLock};

use arc_swap::ArcSwap;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::GameMode;
use tempfile::TempDir;
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

use crate::data::VanillaData;
use crate::entity::player::Player;
use crate::net::bedrock::{BedrockClient, nethernet::NetherNetSession};
use crate::net::java::{JavaClient, pending::PendingConnection};
use crate::net::{ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig};
use crate::server::Server;
use crate::world::World;

pub struct TestServer {
    pub(crate) server: Arc<Server>,
    pub(crate) world: Arc<World>,
    // Keep the peer endpoints alive, but do not start socket readers/writers: tests inspect
    // state and the real outgoing queues without depending on wall-clock network scheduling.
    java_peers: Mutex<Vec<TcpStream>>,
    _directory: TempDir,
}

impl TestServer {
    pub(crate) async fn new() -> Self {
        let directory = tempfile::tempdir().expect("temporary test world");
        let basic = BasicConfiguration {
            seed: pumpkin_util::world_seed::Seed(0),
            default_level_name: directory
                .path()
                .join("world")
                .to_string_lossy()
                .into_owned(),
            allow_nether: false,
            allow_end: false,
            allow_chat_reports: false,
            use_favicon: false,
            ..BasicConfiguration::default()
        };
        let mut advanced = AdvancedConfiguration::default();
        advanced.networking.java.online_mode = false;
        advanced.networking.java.authentication.enabled = false;
        advanced.networking.bedrock.online_mode = false;
        advanced.networking.bedrock.authentication.enabled = false;
        advanced.networking.java.view_distance = NonZeroU8::new(2).unwrap();
        advanced.networking.java.simulation_distance = NonZeroU8::new(2).unwrap();
        advanced.networking.bedrock.view_distance = NonZeroU8::new(2).unwrap();
        advanced.networking.bedrock.simulation_distance = NonZeroU8::new(2).unwrap();
        advanced.world.autosave_ticks = 0;
        advanced.player_data.save_player_data = false;
        advanced.advancement.save_advancements = false;

        // Loading VanillaData would touch the user's data/ files. Empty real stores are enough
        // for these isolated players, and still exercise the normal Server constructor.
        let vanilla_data = VanillaData {
            banned_ip_list: RwLock::default(),
            banned_player_list: RwLock::default(),
            operator_config: RwLock::default(),
            user_cache: RwLock::default(),
            whitelist_config: RwLock::default(),
        };
        let server = Server::new(basic, advanced, vanilla_data).await;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        Self {
            server,
            world,
            java_peers: Mutex::new(Vec::new()),
            _directory: directory,
        }
    }

    pub(crate) async fn new_java_player(&self) -> Arc<Player> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("local test listener");
        let address = listener.local_addr().unwrap();
        let (peer, accepted) = tokio::join!(TcpStream::connect(address), listener.accept());
        let peer = peer.expect("local test peer");
        let (stream, address) = accepted.expect("accepted test client");
        let profile = Self::profile();
        let config = Self::player_config();
        let pending =
            PendingConnection::new(stream, address, 0, PacketRateLimiter::new(false, 0.0, 0.0));
        let client = JavaClient::from_pending(pending, profile.clone(), config.clone());
        client
            .connection_state
            .store(pumpkin_protocol::ConnectionState::Play);
        let client = Arc::new(ClientPlatform::Java(client));
        let player = Arc::new(Player::new(
            client,
            profile,
            config,
            &self.world,
            GameMode::Creative,
        ));
        player.set_client_loaded(true);
        player.client.java().unwrap().set_player(player.clone());
        self.java_peers.lock().unwrap().push(peer);
        self.add_player(&player);
        player
    }

    pub(crate) async fn new_bedrock_player(&self) -> Arc<Player> {
        let session = Arc::new(NetherNetSession::new_for_test().await);
        let client = Arc::new(BedrockClient::new(
            session,
            (std::net::Ipv4Addr::LOCALHOST, 0).into(),
            Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            PacketRateLimiter::new(false, 0.0, 0.0),
        ));
        client
            .version
            .store(pumpkin_util::version::BedrockMinecraftVersion::V_1_26_45);
        let player = Arc::new(Player::new(
            Arc::new(ClientPlatform::Bedrock(client.clone())),
            Self::profile(),
            Self::player_config(),
            &self.world,
            GameMode::Creative,
        ));
        player
            .bedrock_spawned
            .store(true, std::sync::atomic::Ordering::Release);
        player.set_client_loaded(true);
        client.set_player(player.clone());
        self.add_player(&player);
        player
    }

    fn profile() -> GameProfile {
        let id = Uuid::new_v4();
        GameProfile {
            id,
            name: format!("test_{}", &id.simple().to_string()[..8]),
            properties: ArcSwap::from_pointee(Vec::new()),
            profile_actions: None,
        }
    }

    fn player_config() -> PlayerConfig {
        PlayerConfig {
            view_distance: NonZeroU8::new(2).unwrap(),
            ..PlayerConfig::default()
        }
    }

    fn add_player(&self, player: &Arc<Player>) {
        self.world.players.rcu(|players| {
            let mut players = (**players).clone();
            players.push(player.clone());
            players
        });
    }

    pub(crate) async fn shutdown(self) {
        for player in self.world.players.load().iter() {
            match player.client.as_ref() {
                ClientPlatform::Java(client) => {
                    client.close();
                    client.await_tasks().await;
                    client.player.store(Arc::new(None));
                }
                ClientPlatform::Bedrock(client) => {
                    client.close().await;
                    client.await_tasks().await;
                    client.player.store(Arc::new(None));
                }
            }
        }
        self.world.players.store(Arc::new(Vec::new()));
        self.java_peers.lock().unwrap().clear();
        self.server.shutdown().await;
    }
}
