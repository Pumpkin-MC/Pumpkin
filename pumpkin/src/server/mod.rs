use crate::block::registry::BlockRegistry;
use crate::command::commands::default_dispatcher;
use crate::command::commands::defaultgamemode::DefaultGamemode;
use crate::data::player_server_data::ServerPlayerData;
use crate::entity::NBTStorage;
use crate::item::registry::ItemRegistry;
use crate::net::EncryptionError;
use crate::plugin::player::player_login::PlayerLoginEvent;
use crate::plugin::server::server_broadcast::ServerBroadcastEvent;
use crate::world::custom_bossbar::CustomBossbars;
use crate::{
    command::dispatcher::CommandDispatcher, entity::player::Player, net::Client, world::World,
};
use bytes::Bytes;
use connection_cache::{CachedBranding, CachedStatus};
use key_store::KeyStore;
use pumpkin_config::{BASIC_CONFIG, advanced_config};

use pumpkin_macros::send_cancellable;
use pumpkin_protocol::client::login::CEncryptionRequest;
use pumpkin_protocol::client::play::CChangeDifficulty;
use pumpkin_protocol::{ClientPacket, client::config::CPluginMessage};
use pumpkin_registry::{DimensionType, Registry};
use pumpkin_util::Difficulty;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::text::TextComponent;
use pumpkin_world::dimension::Dimension;
use pumpkin_world::lock::LevelLocker;
use pumpkin_world::lock::anvil::AnvilLevelLocker;
use pumpkin_world::world_info::anvil::{
    AnvilLevelInfo, LEVEL_DAT_BACKUP_FILE_NAME, LEVEL_DAT_FILE_NAME,
};
use pumpkin_world::world_info::{LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter};
use rand::prelude::SliceRandom;
use rsa::RsaPublicKey;
use std::fs;
use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio_util::task::TaskTracker;

mod connection_cache;
mod key_store;
pub mod seasonal_events;
pub mod ticker;

pub const CURRENT_MC_VERSION: &str = "1.21.5";

/// Represents a Minecraft server instance.
pub struct Server {
    /// Handles cryptographic keys for secure communication.
    key_store: KeyStore,
    /// Manages server status information.
    listing: Mutex<CachedStatus>,
    /// Saves server branding information.
    branding: CachedBranding,
    /// Saves and dispatches commands to appropriate handlers.
    pub command_dispatcher: RwLock<CommandDispatcher>,
    /// Block behaviour.
    pub block_registry: Arc<BlockRegistry>,
    /// Item behaviour.
    pub item_registry: Arc<ItemRegistry>,
    /// Manages multiple worlds within the server.
    pub worlds: RwLock<Vec<Arc<World>>>,
    /// All the dimensions that exist on the server.
    pub dimensions: Vec<DimensionType>,
    /// Caches game registries for efficient access.
    pub cached_registry: Vec<Registry>,
    /// Assigns unique IDs to containers.
    container_id: AtomicU32,
    /// Manages authentication with an authentication server, if enabled.
    pub auth_client: Option<reqwest::Client>,
    /// Mojang's public keys, used for chat session signing
    /// Pulled from Mojang API on startup
    pub mojang_public_keys: Mutex<Vec<RsaPublicKey>>,
    /// The server's custom bossbars
    pub bossbars: Mutex<CustomBossbars>,
    /// The default gamemode when a player joins the server (reset every restart)
    pub defaultgamemode: Mutex<DefaultGamemode>,
    /// Manages player data storage
    pub player_data_storage: ServerPlayerData,
    // Whether the server whitelist is on or off
    pub white_list: AtomicBool,
    tasks: TaskTracker,

    // world stuff which maybe should be put into a struct
    pub level_info: Arc<RwLock<LevelData>>,
    world_info_writer: Arc<dyn WorldInfoWriter>,
    // Gets unlocked when dropped
    // TODO: Make this a trait
    _locker: Arc<AnvilLevelLocker>,
}

impl Server {
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub async fn new() -> Self {
        let auth_client = BASIC_CONFIG.online_mode.then(|| {
            reqwest::Client::builder()
                .connect_timeout(Duration::from_millis(u64::from(
                    advanced_config().networking.authentication.connect_timeout,
                )))
                .read_timeout(Duration::from_millis(u64::from(
                    advanced_config().networking.authentication.read_timeout,
                )))
                .build()
                .expect("Failed to to make reqwest client")
        });

        // First register the default commands. After that, plugins can put in their own.
        let command_dispatcher = RwLock::new(default_dispatcher().await);
        let world_path = BASIC_CONFIG.get_world_path();

        let block_registry = super::block::default_registry();

        let level_info = AnvilLevelInfo.read_world_info(&world_path);
        if let Err(error) = &level_info {
            match error {
                // If it doesn't exist, just make a new one
                WorldInfoError::InfoNotFound => (),
                WorldInfoError::UnsupportedVersion(version) => {
                    log::error!("Failed to load world info!, {version}");
                    log::error!("{error}");
                    panic!("Unsupported world data! See the logs for more info.");
                }
                e => {
                    panic!("World Error {e}");
                }
            }
        } else {
            let dat_path = world_path.join(LEVEL_DAT_FILE_NAME);
            if dat_path.exists() {
                let backup_path = world_path.join(LEVEL_DAT_BACKUP_FILE_NAME);
                fs::copy(dat_path, backup_path).unwrap();
            }
        }

        let level_info = level_info.unwrap_or_default(); // TODO: Improve error handling
        let seed = level_info.world_gen_settings.seed;
        log::info!("Loading Overworld: {seed}");
        let overworld = World::load(
            Dimension::Overworld.into_level(world_path.clone(), block_registry.clone(), seed),
            level_info.clone(),
            DimensionType::Overworld,
            block_registry.clone(),
        );
        log::info!("Loading Nether: {seed}");
        let nether = World::load(
            Dimension::Nether.into_level(world_path.clone(), block_registry.clone(), seed),
            level_info.clone(),
            DimensionType::TheNether,
            block_registry.clone(),
        );
        // log::info!("Loading End: {}", seed);
        // let end = World::load(
        //     Dimension::End.into_level(world_path.clone(), seed),
        //     level_info.clone(),
        //     DimensionType::TheEnd,
        //     block_registry.clone(),
        // );

        // if we fail to lock, lets crash ???. maybe not the best solution when we have a large server with many worlds and one is locked.
        // So TODO
        let locker = AnvilLevelLocker::lock(&world_path).expect("Failed to lock level");

        let world_name = world_path.to_str().unwrap();

        Self {
            cached_registry: Registry::get_synced(),
            container_id: 0.into(),
            worlds: RwLock::new(vec![Arc::new(overworld), Arc::new(nether)]),
            dimensions: vec![
                DimensionType::Overworld,
                DimensionType::OverworldCaves,
                DimensionType::TheNether,
                DimensionType::TheEnd,
            ],
            command_dispatcher,
            block_registry,
            item_registry: super::item::items::default_registry(),
            auth_client,
            key_store: KeyStore::new(),
            listing: Mutex::new(CachedStatus::new()),
            branding: CachedBranding::new(),
            bossbars: Mutex::new(CustomBossbars::new()),
            defaultgamemode: Mutex::new(DefaultGamemode {
                gamemode: BASIC_CONFIG.default_gamemode,
            }),
            player_data_storage: ServerPlayerData::new(
                format!("{world_name}/playerdata"),
                Duration::from_secs(advanced_config().player_data.save_player_cron_interval),
            ),
            white_list: AtomicBool::new(BASIC_CONFIG.white_list),
            tasks: TaskTracker::new(),
            mojang_public_keys: Mutex::new(Vec::new()),
            world_info_writer: Arc::new(AnvilLevelInfo),
            level_info: Arc::new(RwLock::new(level_info)),
            _locker: Arc::new(locker),
        }
    }

    const SPAWN_CHUNK_RADIUS: i32 = 1;

    #[must_use]
    pub fn spawn_chunks() -> Box<[Vector2<i32>]> {
        (-Self::SPAWN_CHUNK_RADIUS..=Self::SPAWN_CHUNK_RADIUS)
            .flat_map(|x| {
                (-Self::SPAWN_CHUNK_RADIUS..=Self::SPAWN_CHUNK_RADIUS)
                    .map(move |z| Vector2::new(x, z))
            })
            .collect()
    }

    /// Spawns a task associated with this server. All tasks spawned with this method are awaited
    /// when the server stops. This means tasks should complete in a reasonable (no looping) amount of time.
    pub fn spawn_task<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tasks.spawn(task)
    }

    pub async fn get_world_from_dimension(&self, dimension: DimensionType) -> Arc<World> {
        // TODO: this is really bad
        let world_guard = self.worlds.read().await;
        let world = match dimension {
            DimensionType::Overworld => world_guard.first(),
            DimensionType::OverworldCaves => todo!(),
            DimensionType::TheEnd => todo!(),
            DimensionType::TheNether => world_guard.get(1),
        };
        world.cloned().unwrap()
    }

    #[allow(clippy::if_then_some_else_none)]
    /// Adds a new player to the server.
    ///
    /// This function takes an `Arc<Client>` representing the connected client and performs the following actions:
    ///
    /// 1. Generates a new entity ID for the player.
    /// 2. Determines the player's gamemode (defaulting to Survival if not specified in configuration).
    /// 3. **(TODO: Select default from config)** Selects the world for the player (currently uses the first world).
    /// 4. Creates a new `Player` instance using the provided information.
    /// 5. Adds the player to the chosen world.
    /// 6. **(TODO: Config if we want increase online)** Optionally updates server listing information based on the player's configuration.
    ///
    /// # Arguments
    ///
    /// * `client`: An `Arc<Client>` representing the connected client.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    ///
    /// - `Arc<Player>`: A reference to the newly created player object.
    /// - `Arc<World>`: A reference to the world the player was added to.
    ///
    /// # Note
    ///
    /// You still have to spawn the `Player` in a `World` to let them join and make them visible.
    pub async fn add_player(&self, client: Client) -> Option<(Arc<Player>, Arc<World>)> {
        let gamemode = self.defaultgamemode.lock().await.gamemode;
        let uuid = client.gameprofile.lock().await.as_ref().unwrap().id;

        let (world, nbt) = if let Ok(Some(data)) = self.player_data_storage.load_data(&uuid) {
            if let Some(dimension_key) = data.get_string("Dimension") {
                if let Some(dimension) = DimensionType::from_name(dimension_key) {
                    let world = self.get_world_from_dimension(dimension).await;
                    (world, Some(data))
                } else {
                    log::warn!("Invalid dimension key in player data: {dimension_key}");
                    let default_world_guard = self.worlds.read().await;
                    let default_world = default_world_guard
                        .first()
                        .expect("Default world should exist");
                    (default_world.clone(), Some(data))
                }
            } else {
                // Player data exists but doesn't have a "Dimension" key.
                let default_world_guard = self.worlds.read().await;
                let default_world = default_world_guard
                    .first()
                    .expect("Default world should exist");
                (default_world.clone(), Some(data))
            }
        } else {
            // No player data found or an error occurred, default to the Overworld.
            let default_world_guard = self.worlds.read().await;
            let default_world = default_world_guard
                .first()
                .expect("Default world should exist");
            (default_world.clone(), None)
        };

        let mut player = Player::new(client, world.clone(), gamemode).await;

        if let Some(mut nbt_data) = nbt {
            player.read_nbt(&mut nbt_data).await;
        }

        // Wrap in Arc after data is loaded
        let player = Arc::new(player);

        send_cancellable! {{
            PlayerLoginEvent::new(player.clone(), TextComponent::text("You have been kicked from the server"));
            'after: {
                player.screen_handler_sync_handler.store_player(player.clone()).await;
                if world
                    .add_player(player.gameprofile.id, player.clone())
                    .await.is_ok() {
                    // TODO: Config if we want increase online
                    if let Some(config) = player.client.config.lock().await.as_ref() {
                        // TODO: Config so we can also just ignore this hehe
                        if config.server_listing {
                            self.listing.lock().await.add_player(&player);
                        }
                    }

                    Some((player, world.clone()))
                } else {
                    None
                }
            }

            'cancelled: {
                player.kick(event.kick_message).await;
                None
            }
        }}
    }

    pub async fn remove_player(&self, player: &Player) {
        // TODO: Config if we want decrease online
        self.listing.lock().await.remove_player(player);
    }

    pub async fn shutdown(&self) {
        self.tasks.close();
        log::debug!("Awaiting tasks for server");
        self.tasks.wait().await;
        log::debug!("Done awaiting tasks for server");

        log::info!("Starting worlds");
        for world in self.worlds.read().await.iter() {
            world.shutdown().await;
        }
        // then lets save the world info
        if let Err(err) = self.world_info_writer.write_world_info(
            &*self.level_info.read().await,
            &BASIC_CONFIG.get_world_path(),
        ) {
            log::error!("Failed to save level.dat: {err}");
        }
        log::info!("Completed worlds");
    }

    /// Broadcasts a packet to all players in all worlds.
    ///
    /// This function sends the specified packet to every connected player in every world managed by the server.
    ///
    /// # Arguments
    ///
    /// * `packet`: A reference to the packet to be broadcast. The packet must implement the `ClientPacket` trait.
    pub async fn broadcast_packet_all<P>(&self, packet: &P)
    where
        P: ClientPacket,
    {
        let mut packet_buf = Vec::new();
        if let Err(err) = packet.write(&mut packet_buf) {
            log::error!("Failed to serialize packet {}: {}", P::PACKET_ID, err);
            return;
        }
        let packet_data: Bytes = packet_buf.into();

        for world in self.worlds.read().await.iter() {
            let current_players = world.players.read().await;
            for player in current_players.values() {
                player.client.enqueue_packet_data(packet_data.clone()).await;
            }
        }
    }

    pub async fn broadcast_message(
        &self,
        message: &TextComponent,
        sender_name: &TextComponent,
        chat_type: u8,
        target_name: Option<&TextComponent>,
    ) {
        send_cancellable! {{
            ServerBroadcastEvent::new(message.clone(), sender_name.clone());

            'after: {
                for world in self.worlds.read().await.iter() {
                    world
                        .broadcast_message(&event.message, &event.sender, chat_type, target_name)
                        .await;
                }
            }
        }}
    }

    /// Sets the difficulty of the server.
    ///
    /// This function updates the difficulty level of the server and broadcasts the change to all players.
    /// It also iterates through all worlds to ensure the difficulty is applied consistently.
    /// If `force_update` is `Some(true)`, the difficulty will be set regardless of the current state.
    /// If `force_update` is `Some(false)` or `None`, the difficulty will only be updated if it is not locked.
    ///
    /// # Arguments
    ///
    /// * `difficulty`: The new difficulty level to set. This should be one of the variants of the `Difficulty` enum.
    /// * `force_update`: An optional boolean that, if set to `Some(true)`, forces the difficulty to be updated even if it is currently locked.
    ///
    /// # Note
    ///
    /// This function does not handle the actual mob spawn options update, which is a TODO item for future implementation.
    pub async fn set_difficulty(&self, difficulty: Difficulty, force_update: Option<bool>) {
        let mut level_info = self.level_info.write().await;
        if force_update.unwrap_or_default() || !level_info.difficulty_locked {
            level_info.difficulty = if BASIC_CONFIG.hardcore {
                Difficulty::Hard
            } else {
                difficulty
            };
            // Minecraft server updates mob spawn options here
            // but its not implemented in Pumpkin yet
            // todo: update mob spawn options

            for world in &*self.worlds.read().await {
                world.level_info.write().await.difficulty = level_info.difficulty;
            }

            self.broadcast_packet_all(&CChangeDifficulty::new(
                level_info.difficulty as u8,
                level_info.difficulty_locked,
            ))
            .await;
        }
    }

    /// Searches for a player by their username across all worlds.
    ///
    /// This function iterates through each world managed by the server and attempts to find a player with the specified username.
    /// If a player is found in any world, it returns an `Arc<Player>` reference to that player. Otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `name`: The username of the player to search for.
    ///
    /// # Returns
    ///
    /// An `Option<Arc<Player>>` containing the player if found, or `None` if not found.
    pub async fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        for world in self.worlds.read().await.iter() {
            if let Some(player) = world.get_player_by_name(name).await {
                return Some(player);
            }
        }
        None
    }

    pub async fn get_players_by_ip(&self, ip: IpAddr) -> Vec<Arc<Player>> {
        let mut players = Vec::<Arc<Player>>::new();

        for world in self.worlds.read().await.iter() {
            for player in world.players.read().await.values() {
                if player.client.address.lock().await.ip() == ip {
                    players.push(player.clone());
                }
            }
        }

        players
    }

    /// Returns all players from all worlds.
    pub async fn get_all_players(&self) -> Vec<Arc<Player>> {
        let mut players = Vec::<Arc<Player>>::new();

        for world in self.worlds.read().await.iter() {
            for player in world.players.read().await.values() {
                players.push(player.clone());
            }
        }

        players
    }

    /// Returns a random player from any of the worlds, or `None` if all worlds are empty.
    pub async fn get_random_player(&self) -> Option<Arc<Player>> {
        let players = self.get_all_players().await;

        players.choose(&mut rand::thread_rng()).map(Arc::<_>::clone)
    }

    /// Searches for a player by their UUID across all worlds.
    ///
    /// This function iterates through each world managed by the server and attempts to find a player with the specified UUID.
    /// If a player is found in any world, it returns an `Arc<Player>` reference to that player. Otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `id`: The UUID of the player to search for.
    ///
    /// # Returns
    ///
    /// An `Option<Arc<Player>>` containing the player if found, or `None` if not found.
    pub async fn get_player_by_uuid(&self, id: uuid::Uuid) -> Option<Arc<Player>> {
        for world in self.worlds.read().await.iter() {
            if let Some(player) = world.get_player_by_uuid(id).await {
                return Some(player);
            }
        }
        None
    }

    /// Counts the total number of players across all worlds.
    ///
    /// This function iterates through each world and sums up the number of players currently connected to that world.
    ///
    /// # Returns
    ///
    /// The total number of players connected to the server.
    pub async fn get_player_count(&self) -> usize {
        let mut count = 0;
        for world in self.worlds.read().await.iter() {
            count += world.players.read().await.len();
        }
        count
    }

    /// Similar to [`Server::get_player_count`] >= n, but may be more efficient since it stops its iteration through all worlds as soon as n players were found.
    pub async fn has_n_players(&self, n: usize) -> bool {
        let mut count = 0;
        for world in self.worlds.read().await.iter() {
            count += world.players.read().await.len();
            if count >= n {
                return true;
            }
        }
        false
    }

    /// Generates a new container id.
    pub fn new_container_id(&self) -> u32 {
        self.container_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn get_branding(&self) -> CPluginMessage<'_> {
        self.branding.get_branding()
    }

    pub fn get_status(&self) -> &Mutex<CachedStatus> {
        &self.listing
    }

    pub fn encryption_request<'a>(
        &'a self,
        verification_token: &'a [u8; 4],
        should_authenticate: bool,
    ) -> CEncryptionRequest<'a> {
        self.key_store
            .encryption_request("", verification_token, should_authenticate)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        self.key_store.decrypt(data)
    }

    pub fn digest_secret(&self, secret: &[u8]) -> String {
        self.key_store.get_digest(secret)
    }

    async fn tick(&self) {
        for world in self.worlds.read().await.iter() {
            world.tick(self).await;
        }

        if let Err(e) = self.player_data_storage.tick(self).await {
            log::error!("Error ticking player data: {e}");
        }
    }
}
