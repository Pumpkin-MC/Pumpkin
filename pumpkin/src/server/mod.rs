use crate::block::registry::BlockRegistry;
use crate::command::commands::default_dispatcher;
use crate::command::commands::defaultgamemode::DefaultGamemode;
use crate::data::player_server_data::ServerPlayerData;
use crate::entity::{EntityBase, NBTStorage};
use crate::item::registry::ItemRegistry;
use crate::net::{ClientPlatform, DisconnectReason, EncryptionError, GameProfile, PlayerConfig};
use crate::plugin::player::player_login::PlayerLoginEvent;
use crate::plugin::server::server_broadcast::ServerBroadcastEvent;
use crate::server::tick_rate_manager::ServerTickRateManager;
use crate::world::custom_bossbar::CustomBossbars;
use crate::{command::dispatcher::CommandDispatcher, entity::player::Player, world::World};
use connection_cache::{CachedBranding, CachedStatus};
use key_store::KeyStore;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};

use crate::command::CommandSender;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::java::client::login::CEncryptionRequest;
use pumpkin_protocol::java::client::play::CChangeDifficulty;
use pumpkin_protocol::{ClientPacket, java::client::config::CPluginMessage};
use pumpkin_registry::{Registry, VanillaDimensionType};
use pumpkin_util::Difficulty;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::dimension::Dimension;
use pumpkin_world::lock::LevelLocker;
use pumpkin_world::lock::anvil::AnvilLevelLocker;
use pumpkin_world::world_info::anvil::{
    AnvilLevelInfo, LEVEL_DAT_BACKUP_FILE_NAME, LEVEL_DAT_FILE_NAME,
};
use pumpkin_world::world_info::{LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter};
use rand::seq::{IndexedRandom, IteratorRandom, SliceRandom};
use rsa::RsaPublicKey;
use std::collections::HashSet;
use std::fs;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32};
use std::{future::Future, sync::atomic::Ordering, time::Duration};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio_util::task::TaskTracker;

mod connection_cache;
mod key_store;
pub mod seasonal_events;
pub mod tick_rate_manager;
pub mod ticker;

use super::command::args::entities::{
    EntityFilter, EntityFilterSort, EntitySelectorType, TargetSelector, ValueCondition,
};

/// Represents a Minecraft server instance.
pub struct Server {
    pub basic_config: BasicConfiguration,
    pub advanced_config: AdvancedConfiguration,

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
    pub dimensions: Vec<VanillaDimensionType>,
    /// Caches game registries for efficient access.
    pub cached_registry: Vec<Registry>,
    /// Assigns unique IDs to containers.
    container_id: AtomicU32,
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
    /// Manages the server's tick rate, freezing, and sprinting
    pub tick_rate_manager: Arc<ServerTickRateManager>,
    /// Stores the duration of the last 100 ticks for performance analysis
    pub tick_times_nanos: Mutex<[i64; 100]>,
    /// Aggregated tick times for efficient rolling average calculation
    pub aggregated_tick_times_nanos: AtomicI64,
    /// Total number of ticks processed by the server
    pub tick_count: AtomicI32,
    /// Random unique Server ID used by Bedrock Edition
    pub server_guid: u64,
    tasks: TaskTracker,

    // world stuff which maybe should be put into a struct
    pub level_info: Arc<RwLock<LevelData>>,
    world_info_writer: Arc<dyn WorldInfoWriter>,
    // Gets unlocked when dropped
    // TODO: Make this a trait
    _locker: Arc<Option<AnvilLevelLocker>>,
}

impl Server {
    #[allow(clippy::new_without_default)]
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub async fn new(
        basic_config: BasicConfiguration,
        advanced_config: AdvancedConfiguration,
    ) -> Arc<Self> {
        // First register the default commands. After that, plugins can put in their own.
        let command_dispatcher = RwLock::new(default_dispatcher(&basic_config).await);
        let world_path = basic_config.get_world_path();

        let block_registry = super::block::registry::default_registry();

        let level_info = AnvilLevelInfo.read_world_info(&world_path);
        if let Err(error) = &level_info {
            match error {
                // If it doesn't exist, just make a new one
                WorldInfoError::InfoNotFound => (),
                WorldInfoError::UnsupportedDataVersion(_version)
                | WorldInfoError::UnsupportedLevelVersion(_version) => {
                    log::error!("Failed to load world info!");
                    log::error!("{error}");
                    panic!("Unsupported world version! See the logs for more info.");
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
        let locker = match AnvilLevelLocker::lock(&world_path) {
            Ok(l) => Some(l),
            Err(err) => {
                log::warn!(
                    "Could not lock the level file. Data corruption is possible if the world is accessed by multiple processes simultaneously. Error: {err}"
                );
                None
            }
        };

        let level_info = level_info.unwrap_or_else(|err| {
            log::warn!("Failed to get level_info, using default instead: {err}");
            LevelData::default(basic_config.seed)
        });

        let seed = level_info.world_gen_settings.seed;
        let level_info = Arc::new(RwLock::new(level_info));

        let listing = Mutex::new(CachedStatus::new(&basic_config));
        let defaultgamemode = Mutex::new(DefaultGamemode {
            gamemode: basic_config.default_gamemode,
        });
        let player_data_storage = ServerPlayerData::new(
            world_path.join("playerdata"),
            Duration::from_secs(advanced_config.player_data.save_player_cron_interval),
            advanced_config.player_data.save_player_data,
        );
        let white_list = AtomicBool::new(basic_config.white_list);

        let tick_rate_manager = Arc::new(ServerTickRateManager::new(basic_config.tps));

        let server = Self {
            basic_config,
            advanced_config,

            cached_registry: Registry::get_synced(),
            container_id: 0.into(),
            worlds: RwLock::new(vec![]),
            dimensions: vec![
                VanillaDimensionType::Overworld,
                VanillaDimensionType::OverworldCaves,
                VanillaDimensionType::TheNether,
                VanillaDimensionType::TheEnd,
            ],
            command_dispatcher,
            block_registry: block_registry.clone(),
            item_registry: super::item::items::default_registry(),
            key_store: KeyStore::new(),
            listing,
            branding: CachedBranding::new(),
            bossbars: Mutex::new(CustomBossbars::new()),
            defaultgamemode,
            player_data_storage,
            white_list,
            tick_rate_manager,
            tick_times_nanos: Mutex::new([0; 100]),
            aggregated_tick_times_nanos: AtomicI64::new(0),
            tick_count: AtomicI32::new(0),
            tasks: TaskTracker::new(),
            server_guid: rand::random(),
            mojang_public_keys: Mutex::new(Vec::new()),
            world_info_writer: Arc::new(AnvilLevelInfo),
            level_info: level_info.clone(),
            _locker: Arc::new(locker),
        };

        let server = Arc::new(server);
        let weak = Arc::downgrade(&server);
        let level_config = &server.advanced_config.world;

        log::info!("Loading Overworld: {seed}");
        let overworld = World::load(
            Dimension::Overworld.into_level(
                level_config,
                world_path.clone(),
                block_registry.clone(),
                seed,
            ),
            level_info.clone(),
            VanillaDimensionType::Overworld,
            block_registry.clone(),
            weak.clone(),
        );
        log::info!("Loading Nether: {seed}");
        let nether = World::load(
            Dimension::Nether.into_level(
                level_config,
                world_path.clone(),
                block_registry.clone(),
                seed,
            ),
            level_info.clone(),
            VanillaDimensionType::TheNether,
            block_registry.clone(),
            weak.clone(),
        );
        log::info!("Loading End: {seed}");
        let end = World::load(
            Dimension::End.into_level(level_config, world_path, block_registry.clone(), seed),
            level_info,
            VanillaDimensionType::TheEnd,
            block_registry,
            weak,
        );
        *server
            .worlds
            .try_write()
            .expect("Nothing should hold a lock of worlds before server startup") =
            // vec![overworld.into()];
            vec![overworld.into(), nether.into(), end.into()];
        server
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

    pub async fn get_world_from_dimension(&self, dimension: VanillaDimensionType) -> Arc<World> {
        // TODO: this is really bad
        let world_guard = self.worlds.read().await;
        match dimension {
            VanillaDimensionType::Overworld => world_guard.first(),
            VanillaDimensionType::OverworldCaves => todo!(),
            VanillaDimensionType::TheEnd => world_guard.get(2),
            VanillaDimensionType::TheNether => world_guard.get(1),
        }
        .cloned()
        .unwrap()
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
    pub async fn add_player(
        &self,
        client: ClientPlatform,
        profile: GameProfile,
        config: Option<PlayerConfig>,
    ) -> Option<(Arc<Player>, Arc<World>)> {
        let gamemode = self.defaultgamemode.lock().await.gamemode;

        let (world, nbt) = if let Ok(Some(data)) = self.player_data_storage.load_data(&profile.id) {
            if let Some(dimension_key) = data.get_string("Dimension") {
                if let Some(dimension) =
                    VanillaDimensionType::from_resource_location_string(dimension_key)
                {
                    let world = self.get_world_from_dimension(dimension).await;
                    (world, Some(data))
                } else {
                    log::warn!("Invalid dimension key in player data: {dimension_key}");
                    let default_world = self
                        .worlds
                        .read()
                        .await
                        .first()
                        .expect("Default world should exist")
                        .clone();
                    (default_world, Some(data))
                }
            } else {
                // Player data exists but doesn't have a "Dimension" key.
                let default_world = self
                    .worlds
                    .read()
                    .await
                    .first()
                    .expect("Default world should exist")
                    .clone();
                (default_world, Some(data))
            }
        } else {
            // No player data found or an error occurred, default to the Overworld.
            let default_world = self
                .worlds
                .read()
                .await
                .first()
                .expect("Default world should exist")
                .clone();
            (default_world, None)
        };

        let mut player = Player::new(
            client,
            profile,
            config.clone().unwrap_or_default(),
            world.clone(),
            gamemode,
        )
        .await;

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
                    if let Some(config) = config {
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
                player.kick(DisconnectReason::Kicked, event.kick_message).await;
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
        let level_data = self.level_info.read().await;
        // then lets save the world info

        if let Err(err) = self
            .world_info_writer
            .write_world_info(&level_data, &self.basic_config.get_world_path())
        {
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
    pub async fn broadcast_packet_all<P: ClientPacket>(&self, packet: &P) {
        for world in self.worlds.read().await.iter() {
            let current_players = world.players.read().await;
            for player in current_players.values() {
                player.client.enqueue_packet(packet).await;
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
        if level_info.difficulty_locked && !force_update.unwrap_or_default() {
            return;
        }

        let difficulty = if self.basic_config.hardcore {
            Difficulty::Hard
        } else {
            difficulty
        };

        level_info.difficulty = difficulty;
        let locked = level_info.difficulty_locked;
        drop(level_info);

        for world in &*self.worlds.read().await {
            world.level_info.write().await.difficulty = difficulty;
        }

        self.broadcast_packet_all(&CChangeDifficulty::new(difficulty as u8, locked))
            .await;
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
                if player.client.address().await.ip() == ip {
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

        players.choose(&mut rand::rng()).map(Arc::<_>::clone)
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

    /// Main server tick method. This now handles both player/network ticking (which always runs)
    /// and world/game logic ticking (which is affected by freeze state).
    pub async fn tick(self: &Arc<Self>) {
        if self.tick_rate_manager.runs_normally() || self.tick_rate_manager.is_sprinting() {
            self.tick_worlds().await;
            // Always run player and network ticking, even when game is frozen
        } else {
            self.tick_players_and_network().await;
        }
    }

    /// Ticks essential server functions that must run even when the game is frozen.
    /// This includes player ticking (network, keep-alives) and flushing world updates to clients.
    pub async fn tick_players_and_network(&self) {
        // First, flush pending block updates and synced block events to clients
        for world in self.worlds.read().await.iter() {
            world.flush_block_updates().await;
            world.flush_synced_block_events().await;
        }

        let players_to_tick: Vec<_> = self.get_all_players().await;
        for player in players_to_tick {
            player.tick(self).await;
        }
    }
    /// Ticks the game logic for all worlds. This is the part that is affected by `/tick freeze`.
    pub async fn tick_worlds(self: &Arc<Self>) {
        let worlds = self.worlds.read().await.clone();
        let mut handles = Vec::with_capacity(worlds.len());
        for world in &worlds {
            let world = world.clone();
            let server = self.clone();
            handles.push(tokio::spawn(async move {
                world.tick(&server).await;
            }));
        }
        for handle in handles {
            // Wait for all world ticks to complete
            let _ = handle.await;
        }

        // Global periodic tasks
        if let Err(e) = self.player_data_storage.tick(self).await {
            log::error!("Error ticking player data: {e}");
        }
    }

    /// Updates the tick time statistics with the duration of the last tick.
    pub async fn update_tick_times(&self, tick_duration_nanos: i64) {
        let tick_count = self.tick_count.fetch_add(1, Ordering::Relaxed);
        let index = (tick_count % 100) as usize;

        let mut tick_times = self.tick_times_nanos.lock().await;
        let old_time = tick_times[index];
        tick_times[index] = tick_duration_nanos;
        drop(tick_times);

        self.aggregated_tick_times_nanos
            .fetch_add(tick_duration_nanos - old_time, Ordering::Relaxed);
    }

    /// Gets the rolling average tick time over the last 100 ticks, in nanoseconds.
    pub fn get_average_tick_time_nanos(&self) -> i64 {
        let tick_count = self.tick_count.load(Ordering::Relaxed);
        let sample_size = (tick_count as usize).min(100);
        if sample_size == 0 {
            return 0;
        }
        self.aggregated_tick_times_nanos.load(Ordering::Relaxed) / sample_size as i64
    }

    /// Returns a copy of the last 100 tick times.
    pub async fn get_tick_times_nanos_copy(&self) -> [i64; 100] {
        *self.tick_times_nanos.lock().await
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::option_if_let_else)]
    pub async fn select_entities(
        &self,
        target_selector: &TargetSelector,
        source: Option<&CommandSender>,
    ) -> Vec<Arc<dyn EntityBase>> {
        let iter = match &target_selector.selector_type {
            EntitySelectorType::Source
            | EntitySelectorType::NearestEntity
            | EntitySelectorType::NearestPlayer => {
                // todo: command context, currently the nearest entity is the player itself
                if let Some(sender) = source {
                    if let Some(player) = sender.as_player() {
                        vec![player as Arc<dyn EntityBase>].into_iter()
                    } else {
                        vec![].into_iter()
                    }
                } else {
                    vec![].into_iter()
                }
            }
            EntitySelectorType::RandomPlayer => {
                if let Some(player) = self.get_random_player().await {
                    vec![player as Arc<dyn EntityBase>].into_iter()
                } else {
                    vec![].into_iter()
                }
            }
            EntitySelectorType::AllPlayers => self
                .get_all_players()
                .await
                .into_iter()
                .map(|p| p as Arc<dyn EntityBase>)
                .collect::<Vec<_>>()
                .into_iter(),
            EntitySelectorType::AllEntities => {
                let mut entities = Vec::new();
                for world in self.worlds.read().await.iter() {
                    entities.extend(world.entities.read().await.values().cloned());
                    entities.extend(
                        world
                            .players
                            .read()
                            .await
                            .values()
                            .cloned()
                            .map(|p| p as Arc<dyn EntityBase>),
                    );
                }
                entities.into_iter()
            }
            EntitySelectorType::NamedPlayer(name) => {
                if let Some(player) = self.get_player_by_name(name).await {
                    vec![player as Arc<dyn EntityBase>].into_iter()
                } else {
                    vec![].into_iter()
                }
            }
            EntitySelectorType::Uuid(uuid) => {
                if let Some(player) = self.get_player_by_uuid(*uuid).await {
                    vec![player as Arc<dyn EntityBase>].into_iter()
                } else {
                    vec![].into_iter()
                }
            }
        };
        let type_included = target_selector
            .conditions
            .iter()
            .filter_map(|f| {
                if let EntityFilter::Type(ValueCondition::Equals(entity_type)) = f {
                    Some(*entity_type)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        let type_excluded = target_selector
            .conditions
            .iter()
            .filter_map(|f| {
                if let EntityFilter::Type(ValueCondition::NotEquals(entity_type)) = f {
                    Some(*entity_type)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        let type_filtered = iter.filter(|e| {
            // Filter by entity type
            (type_excluded.is_empty() || !type_excluded.contains(&e.get_entity().entity_type))
                && (type_included.is_empty() || type_included.contains(&e.get_entity().entity_type))
        });
        let iter = type_filtered;
        match target_selector
            .get_sort()
            .unwrap_or(EntityFilterSort::Arbitrary)
        {
            // If the sort is arbitrary, we just return all entities in all worlds
            EntityFilterSort::Arbitrary => iter.take(target_selector.get_limit()).collect(),
            EntityFilterSort::Random => {
                if target_selector.get_limit() == 0 {
                    return vec![];
                } else if target_selector.get_limit() == 1 {
                    // If the limit is 1, we just return a random entity
                    return if let Some(entity) = iter.choose(&mut rand::rng()) {
                        vec![entity]
                    } else {
                        vec![]
                    };
                }
                // If the sort is random, we shuffle the entities and then take the limit
                let mut entities: Vec<_> = iter.collect();
                entities.shuffle(&mut rand::rng());
                entities
                    .into_iter()
                    .take(target_selector.get_limit())
                    .collect()
            }
            EntityFilterSort::Nearest | EntityFilterSort::Furthest => {
                if target_selector.get_limit() == 0 {
                    return vec![];
                }
                // sort entities first
                // todo: command context
                let center = if let Some(source) = source {
                    source.position().unwrap_or_default()
                } else {
                    Vector3::default()
                };
                let mut entities = iter.collect::<Vec<_>>();
                entities.sort_by(|a, b| {
                    let a_distance = a.get_entity().pos.load().squared_distance_to_vec(center);
                    let b_distance = b.get_entity().pos.load().squared_distance_to_vec(center);
                    if target_selector.get_sort() == Some(EntityFilterSort::Nearest) {
                        a_distance
                            .partial_cmp(&b_distance)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    } else {
                        b_distance
                            .partial_cmp(&a_distance)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    }
                });
                entities
                    .into_iter()
                    .take(target_selector.get_limit())
                    .collect()
            }
        }
    }
}
