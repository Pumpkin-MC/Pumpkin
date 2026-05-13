use crate::block::registry::BlockRegistry;
use crate::command::commands::default_dispatcher;
use crate::command::commands::defaultgamemode::DefaultGamemode;
use crate::data::VanillaData;
use crate::data::player_server::ServerPlayerData;
use crate::entity::{EntityBase, NBTStorage};
use crate::item::registry::ItemRegistry;
use crate::net::authentication::fetch_mojang_public_keys;
use crate::net::{ClientPlatform, DisconnectReason, EncryptionError, GameProfile, PlayerConfig};
use crate::plugin::PluginManager;
use crate::plugin::player::player_login::PlayerLoginEvent;
use crate::plugin::server::server_broadcast::ServerBroadcastEvent;
use crate::server::tick_rate_manager::ServerTickRateManager;
use crate::world::WorldPortal;
use crate::world::custom_bossbar::CustomBossbars;
use crate::{
    command::node::dispatcher::CommandDispatcher, entity::player::Player, world::World,
    world::map::MapManager,
};
use arc_swap::ArcSwap;
use connection_cache::{CachedBranding, CachedStatus};
use key_store::KeyStore;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityType;
use pumpkin_util::permission::{PermissionManager, PermissionRegistry};
use pumpkin_util::text::color::NamedColor;
use pumpkin_world::dimension::into_level;
use pumpkin_world::world::WorldPortalExt;
use tracing::{debug, error, info, warn};

use crate::command::CommandSender;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::java::client::login::CEncryptionRequest;
use pumpkin_protocol::java::client::play::{CChangeDifficulty, CTabList};
use pumpkin_protocol::{ClientPacket, java::client::config::CPluginMessage};
use pumpkin_util::Difficulty;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world_info::anvil::{
    AnvilLevelInfo, LEVEL_DAT_BACKUP_FILE_NAME, LEVEL_DAT_FILE_NAME,
};
use pumpkin_world::world_info::{LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter};
use rand::seq::{IndexedRandom, SliceRandom};
use rsa::RsaPublicKey;
use std::collections::HashSet;
use std::fs;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32};
use std::{future::Future, sync::atomic::Ordering, time::Duration};
use tokio::sync::{Mutex, OnceCell, RwLock};
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::task::TaskTracker;

mod connection_cache;
mod key_store;
pub mod scheduler;
pub mod seasonal_events;
pub mod tick_rate_manager;
pub mod ticker;

use crate::command::args::entities::{
    EntityFilter, EntityFilterSort, EntitySelectorType, TargetSelector, ValueCondition,
};
use crate::server::scheduler::TaskScheduler;

/// Represents a Minecraft server instance.
pub struct Server {
    pub basic_config: BasicConfiguration,
    pub advanced_config: AdvancedConfiguration,

    pub data: VanillaData,

    /// Plugin manager
    pub plugin_manager: Arc<PluginManager>,

    /// Permission manager for the server.
    pub permission_manager: Arc<RwLock<PermissionManager>>,
    /// Permission registry for the server.
    pub permission_registry: Arc<RwLock<PermissionRegistry>>,

    /// Handles cryptographic keys for secure communication.
    key_store: OnceCell<Arc<KeyStore>>,
    /// Bedrock OIDC provider keys, fetched on startup for 1.26.10+ token validation.
    pub bedrock_oidc_keys: OnceCell<(String, pumpkin_util::jwt::Jwks)>,
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
    pub worlds: ArcSwap<Vec<Arc<World>>>,
    /// All the dimensions that exist on the server.
    pub dimensions: Vec<Dimension>,
    /// Assigns unique IDs to containers.
    container_id: AtomicU32,
    /// Assigns unique IDs to maps.
    map_id: AtomicI32,
    /// Mojang's public keys, used for chat session signing
    /// Pulled from Mojang API on startup
    pub mojang_public_keys: ArcSwap<Vec<RsaPublicKey>>,
    /// The server's custom bossbars
    pub bossbars: Mutex<CustomBossbars>,
    /// Manages all maps on the server
    pub map_manager: MapManager,
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
    /// Player idle timeout in minutes (0 = disabled)
    pub player_idle_timeout: AtomicI32,
    /// Manages scheduled tasks (e.g. from plugins)
    pub task_scheduler: Arc<TaskScheduler>,
    tasks: TaskTracker,

    // world stuff which maybe should be put into a struct
    pub level_info: Arc<ArcSwap<LevelData>>,
    world_info_writer: Arc<dyn WorldInfoWriter>,
}

impl Server {
    #[expect(clippy::too_many_lines)]
    #[must_use]
    pub async fn new(
        basic_config: BasicConfiguration,
        advanced_config: AdvancedConfiguration,
        vanilla_data: VanillaData,
    ) -> Arc<Self> {
        let permission_registry = Arc::new(RwLock::new(PermissionRegistry::new()));
        // First register the default commands. After that, plugins can put in their own.
        let command_dispatcher =
            RwLock::new(default_dispatcher(&permission_registry, &basic_config).await);
        let world_path = basic_config.get_world_path();

        let block_registry = super::block::registry::default_registry();

        let level_info = AnvilLevelInfo.read_world_info(&world_path);
        if let Err(error) = &level_info {
            match error {
                // If it doesn't exist, just make a new one
                WorldInfoError::InfoNotFound => (),
                WorldInfoError::UnsupportedDataVersion(_version)
                | WorldInfoError::UnsupportedLevelVersion(_version) => {
                    error!("Failed to load world info!");
                    error!("{error}");
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
        let level_info = level_info.unwrap_or_else(|err| {
            warn!("Failed to get level_info, using default instead: {err}");
            let default_data = LevelData::default(basic_config.seed);
            if let Err(err) = AnvilLevelInfo.write_world_info(&default_data, &world_path) {
                error!("Failed to save level.dat: {err}");
            }
            default_data
        });

        let seed = level_info.world_gen_settings.seed;
        let level_info = Arc::new(ArcSwap::new(Arc::new(level_info)));

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

        let mojang_keys_task = tokio::spawn({
            let auth_config = advanced_config.networking.authentication.clone();
            let allow_chat = basic_config.allow_chat_reports;
            async move {
                if allow_chat {
                    fetch_mojang_public_keys(&auth_config).unwrap_or_else(|e| {
                        error!("Failed to fetch Mojang keys: {e}");
                        Vec::new()
                    })
                } else {
                    Vec::new()
                }
            }
        });

        let server = Self {
            basic_config,
            advanced_config,
            data: vanilla_data,
            plugin_manager: Arc::new(PluginManager::new()),
            permission_manager: Arc::new(RwLock::new(PermissionManager::new(
                permission_registry.clone(),
            ))),
            permission_registry,
            container_id: 0.into(),
            map_id: level_info.load().map_id.into(),
            worlds: ArcSwap::from_pointee(vec![]),
            dimensions: vec![
                Dimension::OVERWORLD,
                Dimension::THE_NETHER,
                Dimension::THE_END,
            ],
            command_dispatcher,
            block_registry: block_registry.clone(),
            item_registry: super::item::items::default_registry(),
            key_store: OnceCell::new(),
            bedrock_oidc_keys: OnceCell::new(),
            listing,
            branding: CachedBranding::new(),
            bossbars: Mutex::new(CustomBossbars::new()),
            map_manager: MapManager::new(),
            defaultgamemode,
            player_data_storage,
            white_list,
            tick_rate_manager,
            tick_times_nanos: Mutex::new([0; 100]),
            aggregated_tick_times_nanos: AtomicI64::new(0),
            tick_count: AtomicI32::new(0),
            tasks: TaskTracker::new(),
            task_scheduler: Arc::new(TaskScheduler::new()),
            server_guid: rand::random(),
            player_idle_timeout: AtomicI32::new(0),
            mojang_public_keys: ArcSwap::from_pointee(Vec::new()),
            world_info_writer: Arc::new(AnvilLevelInfo),
            level_info,
        };
        let server = Arc::new(server);

        let gen_pool = Arc::new(
            rayon::ThreadPoolBuilder::new()
                .thread_name(|i| format!("Gen-Pool-{i}"))
                .build()
                .expect("Failed to build generation thread pool"),
        );

        let server_clone = server.clone();
        tokio::spawn(async move {
            server_clone
                .key_store
                .get_or_init(|| async { Arc::new(KeyStore::new()) })
                .await;
        });

        let world_loader = |dim: Dimension| {
            let path = world_path.clone();
            let registry = block_registry.clone();
            let l_info = server.level_info.clone(); // Access from struct
            let weak = Arc::downgrade(&server);
            let config = Arc::new(server.advanced_config.world.clone());
            let pool = gen_pool.clone();

            tokio::task::spawn_blocking(move || {
                info!(
                    "Loading {}",
                    TextComponent::text(dim.minecraft_name.to_string())
                        .color_named(NamedColor::DarkGreen)
                        .to_pretty_console()
                );
                let level = into_level(dim.clone(), &config, path, seed, Some(pool));
                let world = Arc::new(World::load(level.clone(), l_info, dim, registry, weak));
                let portal: Arc<dyn WorldPortalExt> = Arc::new(WorldPortal(
                    world.clone(),
                    tokio::runtime::Handle::current(),
                ));
                level.world_portal.store(Arc::new(Some(portal)));
                world
            })
        };

        info!("Starting parallel world load...");
        let (overworld, nether, end, keys) = tokio::join!(
            world_loader(Dimension::OVERWORLD),
            world_loader(Dimension::THE_NETHER),
            world_loader(Dimension::THE_END),
            mojang_keys_task
        );

        let worlds_vec = vec![
            overworld.expect("Overworld panicked"),
            nether.expect("Nether panicked"),
            end.expect("End panicked"),
        ];
        server.worlds.store(Arc::new(worlds_vec));
        if let Ok(k) = keys {
            server.mojang_public_keys.store(Arc::new(k));
        }

        info!("All worlds loaded successfully.");

        if server.basic_config.online_mode {
            let server_clone = server.clone();
            tokio::spawn(async move {
                server_clone
                    .bedrock_oidc_keys
                    .get_or_init(|| async {
                        tokio::task::block_in_place(|| {
                            pumpkin_util::jwt::fetch_oidc_jwks().unwrap_or_else(|e| {
                                error!("Failed to fetch Bedrock OIDC keys: {e}");
                                (String::new(), pumpkin_util::jwt::Jwks { keys: Vec::new() })
                            })
                        })
                    })
                    .await;
            });
        }
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

    pub fn get_world_from_dimension(&self, dimension: &Dimension) -> Arc<World> {
        // TODO: this is really bad
        let world_guard = self.worlds.load();

        if dimension == &Dimension::OVERWORLD {
            world_guard.first()
        } else if dimension == &Dimension::THE_NETHER {
            world_guard.get(1)
        } else {
            world_guard.get(2)
        }
        .cloned()
        .unwrap()
    }

    pub async fn create_world(self: &Arc<Self>, name: String, dimension: Dimension) -> Arc<World> {
        {
            let worlds = self.worlds.load();
            if let Some(world) = worlds
                .iter()
                .find(|w| w.get_world_name() == name && w.dimension == dimension)
            {
                return world.clone();
            }
        }

        let server = self.clone();
        let name_clone = name.clone();
        tokio::task::spawn_blocking(move || {
            let world_path = server.basic_config.get_world_path().join(name_clone);
            let registry = server.block_registry.clone();
            let l_info = server.level_info.clone();
            let weak = Arc::downgrade(&server);
            let config = Arc::new(server.advanced_config.world.clone());
            let seed = server.level_info.load().world_gen_settings.seed;

            // TODO: gen_pool should be reused
            let level = pumpkin_world::dimension::into_level(
                dimension.clone(),
                &config,
                world_path,
                seed,
                None,
            );
            let world: World = World::load(level.clone(), l_info, dimension, registry, weak);
            let world = Arc::new(world);
            let portal: Arc<dyn WorldPortalExt> = Arc::new(WorldPortal(
                world.clone(),
                tokio::runtime::Handle::current(),
            ));
            level.world_portal.store(Arc::new(Some(portal)));
            server.worlds.rcu(|worlds| {
                let mut new_worlds = (**worlds).clone();
                new_worlds.push(world.clone());
                new_worlds
            });
            world
        })
        .await
        .expect("World creation panicked")
    }

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

        let (world, nbt) =
            if let Ok(Some(data)) = self.player_data_storage.load_data(&profile.id).await {
                if let Some(dimension_key) = data.get_string("Dimension") {
                    if let Some(dimension) = Dimension::from_name(dimension_key) {
                        let world = self.get_world_from_dimension(dimension);
                        (world, Some(data))
                    } else {
                        warn!("Invalid dimension key in player data: {dimension_key}");
                        let default_world = self
                            .worlds
                            .load()
                            .first()
                            .expect("Default world should exist")
                            .clone();
                        (default_world, Some(data))
                    }
                } else {
                    // Player data exists but doesn't have a "Dimension" key.
                    let default_world = self
                        .worlds
                        .load()
                        .first()
                        .expect("Default world should exist")
                        .clone();
                    (default_world, Some(data))
                }
            } else {
                // No player data found or an error occurred, default to the Overworld.
                let default_world = self
                    .worlds
                    .load()
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
            self;
            PlayerLoginEvent::new(player.clone(), TextComponent::text("You have been kicked from the server"));
            'after: {
                player.screen_handler_sync_handler.store_player(player.clone()).await;
                if world
                    .add_player(&player)
                    .is_ok() {
                    let mut user_cache = self.data.user_cache.write().await;
                    user_cache.upsert(player.gameprofile.id, player.gameprofile.name.clone());

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
        debug!("Awaiting tasks for server");
        self.tasks.wait().await;
        debug!("Done awaiting tasks for server");

        info!("Starting worlds");
        for world in self.worlds.load().iter() {
            world.shutdown().await;
        }
        let level_data = self.level_info.load();
        // then lets save the world info

        if let Err(err) = self
            .world_info_writer
            .write_world_info(&level_data, &self.basic_config.get_world_path())
        {
            error!("Failed to save level.dat: {err}");
        }
        info!("Completed worlds");
    }

    /// Broadcasts a packet to all players in all worlds.
    ///
    /// This function sends the specified packet to every connected player in every world managed by the server.
    ///
    /// # Arguments
    ///
    /// * `packet`: A reference to the packet to be broadcast. The packet must implement the `ClientPacket` trait.
    pub async fn broadcast_packet_all<P: ClientPacket>(&self, packet: &P) {
        for world in self.worlds.load().iter() {
            world.broadcast_packet_all(packet).await;
        }
    }

    pub async fn broadcast_tab_list_header_footer(
        &self,
        header: &TextComponent,
        footer: &TextComponent,
    ) {
        let packet = CTabList::new(header, footer);
        for world in self.worlds.load().iter() {
            for player in world.players.load().iter() {
                *player.tab_list_header.lock().await = header.clone();
                *player.tab_list_footer.lock().await = footer.clone();
                player.client.enqueue_packet(&packet).await;
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
            self;
            ServerBroadcastEvent::new(message.clone(), sender_name.clone());

            'after: {
                for world in self.worlds.load().iter() {
                    world
                        .broadcast_message(&event.message, &event.sender, chat_type, target_name)
                        .await;
                }
            }
        }}
    }

    /// Gets the current difficulty of the server.
    pub fn get_difficulty(&self) -> Difficulty {
        self.level_info.load().difficulty
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
    pub async fn set_difficulty(&self, difficulty: Difficulty, force_update: bool) {
        let current_info = self.level_info.load();
        if current_info.difficulty_locked && !force_update {
            return;
        }

        let new_difficulty = if self.basic_config.hardcore {
            Difficulty::Hard
        } else {
            difficulty
        };

        let mut new_info = (**current_info).clone();

        new_info.difficulty = new_difficulty;
        let locked = new_info.difficulty_locked;
        self.level_info.store(Arc::new(new_info));

        for world in self.worlds.load().iter() {
            world.set_difficulty(difficulty);
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
    pub fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        for world in self.worlds.load().iter() {
            if let Some(player) = world.get_player_by_name(name) {
                return Some(player);
            }
        }
        None
    }

    pub async fn get_players_by_ip(&self, ip: IpAddr) -> Vec<Arc<Player>> {
        let mut players = Vec::<Arc<Player>>::new();

        for world in self.worlds.load().iter() {
            for player in world.players.load().iter() {
                if player.client.address().await.ip() == ip {
                    players.push(player.clone());
                }
            }
        }

        players
    }

    /// Returns all players from all worlds.
    pub fn get_all_players(&self) -> Vec<Arc<Player>> {
        let mut players = Vec::<Arc<Player>>::new();

        for world in self.worlds.load().iter() {
            players.extend(world.players.load().iter().cloned());
        }

        players
    }

    pub fn for_each_player<F>(&self, mut f: F)
    where
        F: FnMut(&Arc<Player>),
    {
        let worlds = self.worlds.load();

        for world in worlds.iter() {
            let players = world.players.load();
            for player in players.iter() {
                f(player);
            }
        }
    }

    /// Returns a random player from any of the worlds, or `None` if all worlds are empty.
    pub fn get_random_player(&self) -> Option<Arc<Player>> {
        let players = self.get_all_players();
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
    pub fn get_player_by_uuid(&self, id: uuid::Uuid) -> Option<Arc<Player>> {
        for world in self.worlds.load().iter() {
            if let Some(player) = world.get_player_by_uuid(id) {
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
    pub fn get_player_count(&self) -> usize {
        let mut count = 0;
        for world in self.worlds.load().iter() {
            count += world.players.load().len();
        }
        count
    }

    /// Similar to [`Server::get_player_count`] >= n, but may be more efficient since it stops its iteration through all worlds as soon as n players were found.
    pub fn has_n_players(&self, n: usize) -> bool {
        let mut count = 0;
        for world in self.worlds.load().iter() {
            count += world.players.load().len();
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

    /// Generates a new map id.
    pub fn next_map_id(&self) -> i32 {
        let id = self.map_id.fetch_add(1, Ordering::SeqCst);
        self.level_info.rcu(|level_info| {
            let mut new_level_info = (**level_info).clone();
            new_level_info.map_id = self.map_id.load(Ordering::SeqCst);
            new_level_info
        });
        id
    }

    pub fn get_branding(&self) -> CPluginMessage<'_> {
        self.branding.get_branding()
    }

    pub const fn get_status(&self) -> &Mutex<CachedStatus> {
        &self.listing
    }

    pub async fn encryption_request<'a>(
        &'a self,
        verification_token: &'a [u8; 4],
        should_authenticate: bool,
    ) -> CEncryptionRequest<'a> {
        self.key_store
            .get_or_init(|| async { Arc::new(KeyStore::new()) })
            .await
            .encryption_request("", verification_token, should_authenticate)
    }

    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        self.key_store
            .get_or_init(|| async { Arc::new(KeyStore::new()) })
            .await
            .decrypt(data)
    }

    pub async fn digest_secret(&self, secret: &[u8]) -> String {
        self.key_store
            .get_or_init(|| async { Arc::new(KeyStore::new()) })
            .await
            .get_digest(secret)
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
    pub async fn tick_players_and_network(self: &Arc<Self>) {
        let worlds = self.worlds.load();

        for world in worlds.iter() {
            world.flush_block_updates().await;
            world.flush_synced_block_events().await;
        }

        let mut set = JoinSet::new();
        for world in worlds.iter() {
            let players = world.players.load();
            for player in players.iter() {
                let player_clone = player.clone();
                let server_clone = self.clone();
                set.spawn(async move {
                    player_clone.tick(&server_clone).await;
                });
            }
        }
        set.join_all().await;
    }
    /// Ticks the game logic for all worlds. This is the part that is affected by `/tick freeze`.
    pub async fn tick_worlds(self: &Arc<Self>) {
        self.task_scheduler.tick(self).await;

        let mut set = JoinSet::new();

        for world in self.worlds.load().iter() {
            let world = world.clone();
            let server = self.clone();

            set.spawn(async move {
                world.tick(server).await;
            });
        }

        set.join_all().await;

        // Global tasks
        if let Err(e) = self.player_data_storage.tick(self).await {
            error!("Error ticking player data: {e}");
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

    /// Returns the average Milliseconds Per Tick (MSPT).
    pub fn get_mspt(&self) -> f64 {
        let avg_nanos = self.get_average_tick_time_nanos();
        // Convert nanoseconds to decimal milliseconds
        avg_nanos as f64 / 1_000_000.0
    }

    /// Returns the Ticks Per Second (TPS).
    pub fn get_tps(&self) -> f64 {
        let mspt = self.get_mspt();
        if mspt <= 0.0 {
            return 0.0;
        }
        1000.0 / mspt
    }

    /// Returns a copy of the last 100 tick times.
    pub async fn get_tick_times_nanos_copy(&self) -> [i64; 100] {
        *self.tick_times_nanos.lock().await
    }

    #[allow(clippy::too_many_lines, clippy::option_if_let_else)]
    pub fn select_players(
        &self,
        target_selector: &TargetSelector,
        source: Option<&CommandSender>,
    ) -> Vec<Arc<Player>> {
        let mut players = match &target_selector.selector_type {
            EntitySelectorType::Source => source
                .and_then(CommandSender::as_player)
                .map_or_else(Vec::new, |player| vec![player]),
            EntitySelectorType::NearestPlayer
            | EntitySelectorType::NearestEntity
            | EntitySelectorType::RandomPlayer
            | EntitySelectorType::AllPlayers
            | EntitySelectorType::AllEntities => self.get_all_players(),
            EntitySelectorType::NamedPlayer(name) => self
                .get_player_by_name(name)
                .map_or_else(Vec::new, |player| vec![player]),
            EntitySelectorType::Uuid(uuid) => self
                .get_player_by_uuid(*uuid)
                .map_or_else(Vec::new, |player| vec![player]),
        };

        let player_type = EntityType::from_name("player").expect("entity type player must exist");
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

        players.retain(|_| {
            (type_excluded.is_empty() || !type_excluded.contains(player_type))
                && (type_included.is_empty() || type_included.contains(player_type))
        });

        let limit = target_selector.get_limit();
        if limit == 0 {
            return Vec::new();
        }

        match target_selector
            .get_sort()
            .unwrap_or(EntityFilterSort::Arbitrary)
        {
            EntityFilterSort::Arbitrary => players.into_iter().take(limit).collect(),
            EntityFilterSort::Random => {
                players.shuffle(&mut rand::rng());
                players.into_iter().take(limit).collect()
            }
            EntityFilterSort::Nearest | EntityFilterSort::Furthest => {
                let center = source.and_then(CommandSender::position).unwrap_or_default();
                let nearest_first = target_selector
                    .get_sort()
                    .is_none_or(|sort| sort == EntityFilterSort::Nearest);
                players.sort_by(|a, b| {
                    let a_distance = a.get_entity().pos.load().squared_distance_to_vec(&center);
                    let b_distance = b.get_entity().pos.load().squared_distance_to_vec(&center);
                    if nearest_first {
                        a_distance
                            .partial_cmp(&b_distance)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    } else {
                        b_distance
                            .partial_cmp(&a_distance)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    }
                });
                players.into_iter().take(limit).collect()
            }
        }
    }

    #[allow(clippy::too_many_lines, clippy::option_if_let_else)]
    pub fn select_entities(
        &self,
        target_selector: &TargetSelector,
        source: Option<&CommandSender>,
    ) -> Vec<Arc<dyn EntityBase>> {
        let all_entities_and_players = || {
            let mut entities = Vec::new();
            for world in self.worlds.load().iter() {
                entities.extend(world.entities.load().iter().cloned());
                entities.extend(
                    world
                        .players
                        .load()
                        .iter()
                        .cloned()
                        .map(|player| player as Arc<dyn EntityBase>),
                );
            }
            entities
        };
        let all_players_as_entities = || {
            self.get_all_players()
                .into_iter()
                .map(|player| player as Arc<dyn EntityBase>)
                .collect::<Vec<_>>()
        };

        let mut entities = match &target_selector.selector_type {
            EntitySelectorType::Source => source
                .and_then(CommandSender::as_player)
                .map_or_else(Vec::new, |player| vec![player as Arc<dyn EntityBase>]),
            EntitySelectorType::NearestPlayer
            | EntitySelectorType::RandomPlayer
            | EntitySelectorType::AllPlayers => all_players_as_entities(),
            EntitySelectorType::NearestEntity | EntitySelectorType::AllEntities => {
                all_entities_and_players()
            }
            EntitySelectorType::NamedPlayer(name) => self
                .get_player_by_name(name)
                .map_or_else(Vec::new, |player| vec![player as Arc<dyn EntityBase>]),
            EntitySelectorType::Uuid(uuid) => self
                .get_player_by_uuid(*uuid)
                .map_or_else(Vec::new, |player| vec![player as Arc<dyn EntityBase>]),
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
        entities.retain(|entity| {
            // Filter by entity type
            (type_excluded.is_empty() || !type_excluded.contains(&entity.get_entity().entity_type))
                && (type_included.is_empty()
                    || type_included.contains(&entity.get_entity().entity_type))
        });

        let limit = target_selector.get_limit();
        if limit == 0 {
            return vec![];
        }

        match target_selector
            .get_sort()
            .unwrap_or(EntityFilterSort::Arbitrary)
        {
            EntityFilterSort::Arbitrary => entities.into_iter().take(limit).collect(),
            EntityFilterSort::Random => {
                entities.shuffle(&mut rand::rng());
                entities.into_iter().take(limit).collect()
            }
            EntityFilterSort::Nearest | EntityFilterSort::Furthest => {
                let center = source.and_then(CommandSender::position).unwrap_or_default();
                let nearest_first = target_selector
                    .get_sort()
                    .is_none_or(|sort| sort == EntityFilterSort::Nearest);
                entities.sort_by(|a, b| {
                    let a_distance = a.get_entity().pos.load().squared_distance_to_vec(&center);
                    let b_distance = b.get_entity().pos.load().squared_distance_to_vec(&center);
                    if nearest_first {
                        a_distance
                            .partial_cmp(&b_distance)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    } else {
                        b_distance
                            .partial_cmp(&a_distance)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    }
                });
                entities.into_iter().take(limit).collect()
            }
        }
    }
}
