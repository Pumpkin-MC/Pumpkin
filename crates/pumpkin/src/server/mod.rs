use crate::block::registry::BlockRegistry;
use crate::command::commands::default_dispatcher;
use crate::command::commands::defaultgamemode::DefaultGamemode;
use crate::data::VanillaData;
use crate::data::player_server::ServerPlayerData;
use crate::entity::NBTStorage;
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
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::permission::PermissionManager;
use pumpkin_util::text::color::NamedColor;
use pumpkin_world::dimension::into_level;
use pumpkin_world::generation::generator::GeneratorInit;
use pumpkin_world::world::WorldPortalExt;
use tracing::{debug, error, info, warn};

use pumpkin_protocol::java::client::login::CEncryptionRequest;
use pumpkin_protocol::java::client::play::{CChangeDifficulty, CTabList};
use pumpkin_protocol::{ClientPacket, java::client::config::CPluginMessage};
use pumpkin_util::Difficulty;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world_info::anvil::{
    AnvilLevelInfo, LEVEL_DAT_BACKUP_FILE_NAME, LEVEL_DAT_FILE_NAME,
};
use pumpkin_world::world_info::{LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter};
use rand::seq::IndexedRandom;
use rayon::prelude::*;
use rsa::RsaPublicKey;
use std::fs;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32};
use std::{future::Future, sync::atomic::Ordering, time::Duration};
use tokio::sync::OnceCell;
use tokio::task::JoinHandle;
use tokio_util::task::TaskTracker;

mod connection_cache;
pub(crate) mod debug_profiler;
pub mod enchantment;
mod key_store;
pub mod recipe;
pub mod scheduler;
pub mod seasonal_events;
pub mod server_test_manager;
pub mod tick_rate_manager;
pub mod ticker;

pub use recipe::RecipeManager;

use crate::data::advancement_data::AdvancementManager;
use crate::server::scheduler::TaskScheduler;

/// Represents a Minecraft server instance.
pub struct Server {
    pub basic_config: BasicConfiguration,
    pub advanced_config: AdvancedConfiguration,
    pub telemetry_config: TelemetryConfig,

    pub data: VanillaData,

    /// Plugin manager
    pub plugin_manager: Arc<PluginManager>,

    /// Permission manager for the server.
    pub permission_manager: Arc<PermissionManager>,

    /// Handles cryptographic keys for secure communication.
    key_store: OnceCell<Arc<KeyStore>>,
    /// Bedrock OIDC provider keys, fetched on startup for 1.26.10+ token validation.
    pub bedrock_oidc_keys: Arc<OnceCell<(String, pumpkin_util::jwt::Jwks)>>,
    /// Cached Bedrock server private key (process-lifetime). Generated on first Bedrock login and reused.
    pub bedrock_private_key: OnceCell<Arc<pumpkin_util::p384::ecdsa::SigningKey>>,
    /// Manages server status information.
    listing: std::sync::Mutex<CachedStatus>,
    /// Saves server branding information.
    branding: CachedBranding,
    /// Saves and dispatches commands to appropriate handlers.
    pub command_dispatcher: ArcSwap<CommandDispatcher>,
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
    pub recipe_manager: Arc<recipe::RecipeManager>,
    pub datapack_manager: Arc<crate::data::datapack::DatapackManager>,
    pub enchantment_manager: Arc<enchantment::EnchantmentManager>,
    /// Assigns unique IDs to maps.
    map_id: AtomicI32,
    /// Mojang's public keys, used for chat session signing
    /// Pulled from Mojang API on startup
    pub mojang_public_keys: ArcSwap<Vec<RsaPublicKey>>,
    /// The server's custom bossbars
    pub bossbars: std::sync::Mutex<CustomBossbars>,
    /// Manages all maps on the server
    pub map_manager: MapManager,
    /// The default gamemode when a player joins the server (reset every restart)
    pub defaultgamemode: std::sync::Mutex<DefaultGamemode>,
    /// Manages player data storage
    pub player_data_storage: ServerPlayerData,
    /// Command storage for `/data` command
    pub command_storage:
        std::sync::Mutex<std::collections::HashMap<String, pumpkin_nbt::compound::NbtCompound>>,
    /// Global server stopwatches
    pub stopwatches: std::sync::Mutex<crate::world::stopwatches::Stopwatches>,
    /// Global server random sequences
    pub random_sequences: std::sync::Mutex<crate::world::random_sequences::RandomSequences>,
    // Manages player advancement
    pub advancement_manager: Arc<AdvancementManager>,
    // Whether the server whitelist is on or off
    pub white_list: AtomicBool,
    /// Manages the server's tick rate, freezing, and sprinting
    pub tick_rate_manager: Arc<ServerTickRateManager>,
    /// Stores the duration of the last 100 ticks for performance analysis
    pub tick_times_nanos: std::sync::Mutex<[i64; 100]>,
    /// Aggregated tick times for efficient rolling average calculation
    pub aggregated_tick_times_nanos: AtomicI64,
    /// Total number of ticks processed by the server
    pub tick_count: AtomicI32,
    /// Owns the server-wide tick profiling session used by `/debug`.
    pub(crate) debug_profiler: debug_profiler::DebugProfiler,
    /// Random unique Server ID used by Bedrock Edition
    pub server_guid: u64,
    /// Player idle timeout in minutes (0 = disabled)
    pub player_idle_timeout: AtomicI32,
    /// Manages scheduled tasks (e.g. from plugins)
    pub task_scheduler: Arc<TaskScheduler>,
    /// Manages scheduled datapack functions (`/schedule`)
    pub scheduled_functions: Arc<crate::server::scheduler::ScheduledFunctionQueue>,
    tasks: TaskTracker,
    pub runtime: tokio::runtime::Handle,

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
        telemetry_config: TelemetryConfig,
        vanilla_data: VanillaData,
    ) -> Arc<Self> {
        let permission_manager = Arc::new(PermissionManager::new());
        // First register the default commands. After that, plugins can put in their own.
        let command_dispatcher = ArcSwap::from_pointee(default_dispatcher(
            &permission_manager,
            &advanced_config.commands,
        ));

        crate::command::set_broadcast_console_to_ops(
            advanced_config.commands.broadcast_console_to_ops,
        );

        let world_path = basic_config.get_world_path();

        let block_registry = super::block::registry::default_registry();

        let level_info = match AnvilLevelInfo.read_world_info(&world_path) {
            Ok(level_info) => {
                let dat_path = world_path.join(LEVEL_DAT_FILE_NAME);
                if dat_path.exists() {
                    let backup_path = world_path.join(LEVEL_DAT_BACKUP_FILE_NAME);
                    if let Err(err) = fs::copy(&dat_path, &backup_path) {
                        warn!("Failed to create backup {LEVEL_DAT_BACKUP_FILE_NAME}: {err}");
                    }
                }
                level_info
            }
            Err(WorldInfoError::InfoNotFound) => {
                warn!(
                    "No {LEVEL_DAT_FILE_NAME} in {}, creating a new world with seed {}",
                    world_path.display(),
                    basic_config.seed.0 as i64
                );
                let overworld_gen = pumpkin_world::generation::generator::VanillaGenerator::new(
                    basic_config.seed,
                    Dimension::OVERWORLD,
                );
                let default_data =
                    LevelData::from_world_generator(basic_config.seed, &overworld_gen);
                if let Err(err) = AnvilLevelInfo.write_world_info(&default_data, &world_path) {
                    error!("Failed to save level.dat: {err}");
                }
                default_data
            }
            Err(
                error @ (WorldInfoError::UnsupportedDataVersion(_)
                | WorldInfoError::UnsupportedLevelVersion(_)),
            ) => {
                error!("Failed to load world info!");
                error!("{error}");
                error!("Unsupported world version! See the logs for more info.");
                std::process::exit(1);
            }
            Err(error) => {
                error!("Failed to load the world data in {}!", world_path.display());
                error!("{error}");
                error!(
                    "Refusing to continue: a default world would generate different terrain on top of the existing region files. Restore {LEVEL_DAT_FILE_NAME} from {LEVEL_DAT_BACKUP_FILE_NAME}, which also holds a copy of the world seed, or move the world folder aside to start a new world."
                );
                error!("Failed to load the world data! See the logs for more info.");
                std::process::exit(1);
            }
        };

        let seed = level_info.world_gen_settings.seed;
        let level_info = Arc::new(ArcSwap::new(Arc::new(level_info)));

        let listing = std::sync::Mutex::new(CachedStatus::new(
            &basic_config,
            &advanced_config.networking.java.motd,
            advanced_config.networking.java.max_players,
        ));
        let defaultgamemode = std::sync::Mutex::new(DefaultGamemode {
            gamemode: basic_config.default_gamemode,
        });
        let players_dir = world_path.join("players");
        let player_data_storage = ServerPlayerData::new(
            players_dir.join("data"),
            Duration::from_secs(advanced_config.player_data.save_player_cron_interval),
            advanced_config.player_data.save_player_data,
        );
        let advancement_manager = Arc::new(AdvancementManager::new(
            players_dir.clone(),
            advanced_config.advancement.save_advancements,
        ));
        let white_list = AtomicBool::new(basic_config.white_list);

        let tick_rate_manager = Arc::new(ServerTickRateManager::new(basic_config.tps));

        let dimensions = {
            let mut dimensions = vec![Dimension::OVERWORLD];
            if basic_config.allow_nether {
                dimensions.push(Dimension::THE_NETHER);
            }
            if basic_config.allow_end {
                dimensions.push(Dimension::THE_END);
            }
            dimensions
        };
        info!(
            "Enabled dimensions: {:?}",
            dimensions
                .iter()
                .map(|d| d.minecraft_name)
                .collect::<Vec<_>>()
        );

        let verify_plugin_signatures = advanced_config.plugins.verify_signatures;
        if !verify_plugin_signatures {
            warn!(
                "Plugin signature verification is disabled. Only do this if you fully trust your plugins and their sources, because unsigned or tampered WASM plugins will be loaded without verification."
            );
        }

        let server = Self {
            basic_config,
            advanced_config,
            telemetry_config,
            data: vanilla_data,
            plugin_manager: Arc::new(PluginManager::new(verify_plugin_signatures)),
            permission_manager,
            container_id: 0.into(),
            recipe_manager: Arc::new(recipe::RecipeManager::new()),
            datapack_manager: Arc::new(crate::data::datapack::DatapackManager::new()),
            enchantment_manager: Arc::new(enchantment::EnchantmentManager::new()),
            map_id: level_info.load().map_id.into(),
            worlds: ArcSwap::from_pointee(vec![]),
            dimensions,
            command_dispatcher,
            block_registry: block_registry.clone(),
            item_registry: super::item::items::default_registry(),
            key_store: OnceCell::new(),
            bedrock_oidc_keys: Arc::new(OnceCell::new()),
            bedrock_private_key: OnceCell::new(),
            listing,
            branding: CachedBranding::new(),
            bossbars: std::sync::Mutex::new(CustomBossbars::new()),
            map_manager: MapManager::new(),
            defaultgamemode,
            player_data_storage,
            command_storage: std::sync::Mutex::new(std::collections::HashMap::new()),
            stopwatches: std::sync::Mutex::new(crate::world::stopwatches::Stopwatches::new()),
            random_sequences: std::sync::Mutex::new(
                crate::world::random_sequences::RandomSequences::new(),
            ),
            advancement_manager,
            white_list,
            tick_rate_manager,
            tick_times_nanos: std::sync::Mutex::new([0; 100]),
            aggregated_tick_times_nanos: AtomicI64::new(0),
            tick_count: AtomicI32::new(0),
            debug_profiler: debug_profiler::DebugProfiler::new(),
            tasks: TaskTracker::new(),
            runtime: tokio::runtime::Handle::current(),
            task_scheduler: Arc::new(TaskScheduler::new()),
            scheduled_functions: Arc::new(crate::server::scheduler::ScheduledFunctionQueue::new()),
            server_guid: rand::random(),
            player_idle_timeout: AtomicI32::new(0),
            mojang_public_keys: ArcSwap::from_pointee(Vec::new()),
            world_info_writer: Arc::new(AnvilLevelInfo),
            level_info,
        };
        let server = Arc::new(server);

        // Fetch / generate keys in background tasks to avoid blocking startup
        let server_clone = server.clone();
        server.spawn_task(async move {
            let key_store = Arc::new(KeyStore::new());
            let _ = server_clone.key_store.set(key_store);
        });

        if server.basic_config.allow_chat_reports {
            let server_clone = server.clone();
            server.spawn_task(async move {
                let auth_config = server_clone
                    .advanced_config
                    .networking
                    .java
                    .authentication
                    .clone();
                let keys = fetch_mojang_public_keys(&auth_config)
                    .await
                    .unwrap_or_else(|e| {
                        error!("Failed to fetch Mojang keys: {e}");
                        Vec::new()
                    });
                server_clone.mojang_public_keys.store(Arc::new(keys));
            });
        }

        if server.advanced_config.networking.bedrock.online_mode
            && server
                .advanced_config
                .networking
                .bedrock
                .authentication
                .enabled
        {
            let server_clone = server.clone();
            server.spawn_task(async move {
                let auth = server_clone
                    .advanced_config
                    .networking
                    .bedrock
                    .authentication
                    .clone();
                let keys = match pumpkin_util::jwt::fetch_oidc_jwks(
                    auth.url.as_deref(),
                    auth.connect_timeout,
                    auth.read_timeout,
                )
                .await
                {
                    Ok(keys) => keys,
                    Err(error) => {
                        error!("Failed to fetch Bedrock OIDC keys: {error}");
                        (String::new(), pumpkin_util::jwt::Jwks { keys: Vec::new() })
                    }
                };
                let _ = server_clone.bedrock_oidc_keys.set(keys);
            });
        }

        let mut worlds_vec = Vec::new();
        for dim in &server.dimensions {
            info!(
                "Loading {}",
                TextComponent::text(dim.minecraft_name.to_string())
                    .color_named(NamedColor::DarkGreen)
                    .to_pretty_console()
            );
            let config = Arc::new(server.advanced_config.world.clone());
            let level = into_level(dim.clone(), &config, world_path.clone(), seed);
            let world = Arc::new(World::load(
                level.clone(),
                server.level_info.clone(),
                dim.clone(),
                block_registry.clone(),
                Arc::downgrade(&server),
            ));
            let portal: Arc<dyn WorldPortalExt> = Arc::new(WorldPortal(world.clone()));
            level.world_portal.store(Arc::new(Some(portal)));
            worlds_vec.push(world);
        }

        for world in &worlds_vec {
            let mut world_init_event =
                crate::plugin::api::events::world::world_init::WorldInitEvent::new(world.clone());
            server
                .plugin_manager
                .fire(&server, &mut world_init_event)
                .await;

            let mut world_load_event =
                crate::plugin::api::events::world::world_load::WorldLoadEvent::new(world.clone());
            server
                .plugin_manager
                .fire(&server, &mut world_load_event)
                .await;
        }

        server.worlds.store(Arc::new(worlds_vec));

        info!("All worlds loaded successfully.");

        let enabled_packs = server.level_info.load().data_packs.enabled.clone();
        server
            .datapack_manager
            .load_all(&world_path, &enabled_packs, &server.recipe_manager);

        let source = crate::command::CommandSender::Console.into_source(&server);
        let _ = server
            .datapack_manager
            .execute_function(&server, &source, "#minecraft:load");

        server
    }

    /// Spawns a task associated with this server. All tasks spawned with this method are awaited
    /// when the server stops. This means tasks should complete in a reasonable (no looping) amount of time.
    pub fn spawn_task<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tasks.spawn_on(task, &self.runtime)
    }

    pub fn get_world_from_dimension(&self, dimension: &Dimension) -> Arc<World> {
        let worlds = self.worlds.load();
        worlds
            .iter()
            .find(|w| w.dimension.minecraft_name == dimension.minecraft_name)
            .cloned()
            .or_else(|| worlds.first().cloned())
            .unwrap_or_else(|| {
                error!("No default world exists");
                std::process::exit(1);
            })
    }

    pub fn create_world(self: &Arc<Self>, name: String, dimension: Dimension) -> Arc<World> {
        {
            let worlds = self.worlds.load();
            let world = worlds
                .iter()
                .find(|w| w.get_world_name() == name && w.dimension == dimension)
                .cloned();
            if let Some(world) = world {
                return world;
            }
        }

        let world_path = self.basic_config.get_world_path().join(name);
        let registry = self.block_registry.clone();
        let l_info = self.level_info.clone();
        let weak = Arc::downgrade(self);
        let config = Arc::new(self.advanced_config.world.clone());
        let seed = self.level_info.load().world_gen_settings.seed;

        let level =
            pumpkin_world::dimension::into_level(dimension.clone(), &config, world_path, seed);
        let world: World = World::load(level.clone(), l_info, dimension, registry, weak);
        let world = Arc::new(world);
        let portal: Arc<dyn WorldPortalExt> = Arc::new(WorldPortal(world.clone()));
        level.world_portal.store(Arc::new(Some(portal)));
        self.worlds.rcu(|worlds| {
            let mut new_worlds = (**worlds).clone();
            new_worlds.push(world.clone());
            new_worlds
        });
        let mut event =
            crate::plugin::api::events::world::world_init::WorldInitEvent::new(world.clone());
        self.plugin_manager.fire_blocking(self, &mut event);
        world
    }

    pub async fn unload_world(&self, name: &str) -> Result<(), String> {
        let worlds = self.worlds.load();
        let world_to_unload = worlds
            .iter()
            .find(|w| w.get_world_name() == name || w.dimension.minecraft_name == name)
            .cloned()
            .ok_or_else(|| format!("World '{name}' not found"))?;

        if let Some(first_world) = worlds.first()
            && Arc::ptr_eq(first_world, &world_to_unload)
        {
            return Err("Cannot unload the primary/default world".to_string());
        }

        let player_count = world_to_unload.players.load().len();
        if player_count > 0 {
            return Err(format!(
                "Cannot unload world '{name}': {player_count} players are still in this world"
            ));
        }

        world_to_unload.shutdown().await;
        world_to_unload.unload().await;

        self.worlds.rcu(|w_list| {
            let mut new_list = (**w_list).clone();
            new_list.retain(|w| !Arc::ptr_eq(w, &world_to_unload));
            new_list
        });

        Ok(())
    }

    pub fn save_world_info(&self) -> Result<(), WorldInfoError> {
        let level_data = self.level_info.load();
        self.world_info_writer
            .write_world_info(&level_data, &self.basic_config.get_world_path())
    }

    pub fn reload_datapacks(&self, server: &Arc<Self>) {
        let enabled_packs = self.level_info.load().data_packs.enabled.clone();
        let world_path = self.basic_config.get_world_path();
        self.datapack_manager
            .load_all(&world_path, &enabled_packs, &self.recipe_manager);

        let source = crate::command::CommandSender::Console.into_source(server);
        let _ = self
            .datapack_manager
            .execute_function(server, &source, "#minecraft:load");

        let dynamic_recipes = self.recipe_manager.get_dynamic_recipes_internal();
        for player in self.get_all_players() {
            if let crate::net::ClientPlatform::Java(java_client) = player.client.as_ref() {
                let add_packet = pumpkin_protocol::java::client::play::CRecipeBookAdd::new(
                    true,
                    &dynamic_recipes,
                );
                if let Ok(data) = java_client.serialize_packet(&add_packet) {
                    java_client.try_enqueue_packet(data);
                }
            }
        }
    }

    pub async fn save_all(&self) -> Result<(), String> {
        if let Err(err) = self.save_world_info() {
            error!("Failed to save world info: {err}");
            return Err(format!("Failed to save world info: {err}"));
        }

        if let Err(err) = self.player_data_storage.save_all_players(self) {
            error!("Failed to save player data: {err}");
            return Err(format!("Failed to save player data: {err}"));
        }

        if let Err(err) = self
            .advancement_manager
            .save_all_players(&self.get_all_players())
            .await
        {
            error!("Failed to save player advancements: {err}");
            return Err(format!("Failed to save player advancements: {err}"));
        }

        for world in self.worlds.load().iter() {
            world.save().await;
        }

        Ok(())
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
    pub fn add_player(
        self: &Arc<Self>,
        client: Arc<ClientPlatform>,
        profile: GameProfile,
        config: Option<PlayerConfig>,
    ) -> Option<(Arc<Player>, Arc<World>)> {
        let gamemode = self
            .defaultgamemode
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .gamemode;

        let first_world = self.worlds.load().first().cloned()?;

        let (world, nbt) = if let Ok(Some(data)) = self.player_data_storage.load_data(&profile.id) {
            if let Some(dimension_key) = data.get_string("Dimension") {
                if let Some(dimension) = Dimension::from_name(dimension_key) {
                    let world = self.get_world_from_dimension(dimension);
                    (world, Some(data))
                } else {
                    warn!("Invalid dimension key in player data: {dimension_key}");
                    (first_world, Some(data))
                }
            } else {
                // Player data exists but doesn't have a "Dimension" key.
                (first_world, Some(data))
            }
        } else {
            // No player data found or an error occurred, default to the Overworld.
            (first_world, None)
        };

        let mut player = Player::new(
            client,
            profile,
            config.clone().unwrap_or_default(),
            &world,
            gamemode,
        );

        if let Some(mut nbt_data) = nbt {
            player.read_nbt(&mut nbt_data);
            // The data file itself proves this is a returning player. Older Bedrock
            // sessions could persist HasPlayedBefore as false and mask a valid Pos.
            player.has_played_before.store(true, Ordering::Relaxed);
        }

        // Wrap in Arc after data is loaded
        let player = Arc::new(player);
        {
            let mut advancements = player
                .advancements
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Err(e) = advancements.load() {
                warn!("Error loading player {}: {e}", player.gameprofile.id);
            }
            advancements.player = Arc::downgrade(&player);
        };

        send_cancellable_blocking! {{
            self;
            &mut PlayerLoginEvent::new(player.clone(), TextComponent::text("You have been kicked from the server"));
            'after: {
                player.screen_handler_sync_handler.store_player(player.clone());
                world.add_player(&player).is_ok().then(|| {
                    {
                        let mut user_cache = self
                            .data
                            .user_cache
                            .write()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        user_cache.upsert(player.gameprofile.id, player.gameprofile.name.clone());
                    };

                    // TODO: Config if we want increase online
                    if let Some(config) = config {
                        // TODO: Config so we can also just ignore this hehe
                        if config.server_listing {
                            self.listing
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .add_player(&player);
                        }
                    }

                    (player, world)
                })
            }

            'cancelled: {
                player.kick(DisconnectReason::Kicked, &event.kick_message);
                None
            }
        }}
    }

    pub fn remove_player(&self, player: &Player) {
        player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            pumpkin_data::statistic::CustomStatistic::LeaveGame as i32,
            1,
        );
        // TODO: Config if we want decrease online
        self.listing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove_player(player);
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
    pub fn broadcast_packet_all<P: ClientPacket>(&self, packet: &P) {
        for world in self.worlds.load().iter() {
            world.broadcast_packet_all(packet);
        }
    }

    /// Broadcasts custom server links to all players on the server.
    pub fn broadcast_server_links(&self, links: &[pumpkin_protocol::Link<'_>]) {
        let packet = pumpkin_protocol::java::client::play::CPlayServerLinks::new(links);
        self.broadcast_packet_all(&packet);
    }

    pub fn broadcast_tab_list_header_footer(&self, header: &TextComponent, footer: &TextComponent) {
        let packet = CTabList::new(header, footer);
        for world in self.worlds.load().iter() {
            for player in world.players.load().iter() {
                *player
                    .tab_list_header
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = header.clone();
                *player
                    .tab_list_footer
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = footer.clone();
            }
            world.broadcast_packet_all(&packet);
        }
    }

    pub fn broadcast_message(
        self: &Arc<Self>,
        message: &TextComponent,
        sender_name: &TextComponent,
        chat_type: u8,
        target_name: Option<&TextComponent>,
    ) {
        let mut event = ServerBroadcastEvent::new(message.clone(), sender_name.clone());
        self.plugin_manager.fire_blocking(self, &mut event);
        if !event.cancelled {
            for world in self.worlds.load().iter() {
                world.broadcast_message(&event.message, &event.sender, chat_type, target_name);
            }
        }
    }

    pub fn broadcast_chat_message(
        self: &Arc<Self>,
        message: &crate::net::chat::PlayerChatMessage,
        is_filtered: impl Fn(&crate::entity::player::Player) -> bool,
        sender_player: Option<&Arc<crate::entity::player::Player>>,
        chat_type: pumpkin_protocol::codec::var_int::VarInt,
        sender_name: &TextComponent,
        target_name: Option<&TextComponent>,
    ) {
        for world in self.worlds.load().iter() {
            world.broadcast_chat_message(
                message,
                &is_filtered,
                sender_player,
                chat_type,
                sender_name,
                target_name,
            );
        }
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
    pub fn set_difficulty(&self, difficulty: Difficulty, force_update: bool) {
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
            world.broadcast_editioned(
                &CChangeDifficulty::new(difficulty as u8, locked),
                &pumpkin_protocol::bedrock::client::CSetDifficulty {
                    difficulty: (difficulty as u32).into(),
                },
            );
        }
    }

    /// Sets the difficulty lock status of the server and broadcasts the update to all players.
    pub fn set_difficulty_locked(&self, locked: bool) {
        let current_info = self.level_info.load();
        let mut new_info = (**current_info).clone();
        new_info.difficulty_locked = locked;
        let difficulty = new_info.difficulty;
        self.level_info.store(Arc::new(new_info));

        for world in self.worlds.load().iter() {
            world.broadcast_editioned(
                &CChangeDifficulty::new(difficulty as u8, locked),
                &pumpkin_protocol::bedrock::client::CSetDifficulty {
                    difficulty: (difficulty as u32).into(),
                },
            );
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
    pub fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        for world in self.worlds.load().iter() {
            if let Some(player) = world.get_player_by_name(name) {
                return Some(player);
            }
        }
        None
    }

    pub fn get_players_by_ip(&self, ip: IpAddr) -> Vec<Arc<Player>> {
        let mut players = Vec::<Arc<Player>>::new();

        for world in self.worlds.load().iter() {
            for player in world.players.load().iter() {
                if player.client.address().ip() == ip {
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

    /// Returns the maximum number of players allowed on the server.
    #[must_use]
    pub const fn max_players(&self) -> u32 {
        self.advanced_config.networking.java.max_players
    }

    /// Starts the background telemetry task if enabled in configuration.
    pub fn start_telemetry(self: &Arc<Self>) {
        crate::telemetry::start_telemetry(self.clone());
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

    pub const fn get_branding(&self) -> CPluginMessage<'_> {
        self.branding.get_branding()
    }

    pub const fn get_status(&self) -> &std::sync::Mutex<CachedStatus> {
        &self.listing
    }

    async fn get_or_init_key_store(&self) -> &Arc<KeyStore> {
        self.key_store
            .get_or_init(|| async {
                let (tx, rx) = tokio::sync::oneshot::channel();
                rayon::spawn(move || {
                    let _ = tx.send(Arc::new(KeyStore::new()));
                });
                rx.await.unwrap_or_else(|_| Arc::new(KeyStore::new()))
            })
            .await
    }

    pub async fn encryption_request<'a>(
        &'a self,
        verification_token: &'a [u8; 4],
        should_authenticate: bool,
    ) -> CEncryptionRequest<'a> {
        self.get_or_init_key_store().await.encryption_request(
            "",
            verification_token,
            should_authenticate,
        )
    }

    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key_store = self.get_or_init_key_store().await.clone();
        let data = data.to_vec();
        let (tx, rx) = tokio::sync::oneshot::channel();
        rayon::spawn(move || {
            let _ = tx.send(key_store.decrypt(&data));
        });
        rx.await.map_err(|_| EncryptionError::FailedDecrypt)?
    }

    pub fn digest_secret(&self, secret: &[u8]) -> String {
        self.key_store.get().map_or_else(
            || KeyStore::new().get_digest(secret),
            |key_store| key_store.get_digest(secret),
        )
    }

    /// Main server tick method. This now handles both player/network ticking (which always runs)
    /// and world/game logic ticking (which is affected by freeze state).
    pub fn tick(self: &Arc<Self>) {
        if self.tick_rate_manager.runs_normally() || self.tick_rate_manager.is_sprinting() {
            self.tick_worlds();
            // Always run player and network ticking, even when game is frozen
        } else {
            self.tick_players_and_network();
        }
    }

    /// Ticks essential server functions that must run even when the game is frozen.
    /// This includes player ticking (network, keep-alives) and flushing world updates to clients.
    pub fn tick_players_and_network(self: &Arc<Self>) {
        let worlds = self.worlds.load();
        let handle = self.runtime.clone();

        for world in worlds.iter() {
            world.flush_block_updates();
            world.flush_synced_block_events();

            let players = world.players.load();
            let player_handle = handle.clone();
            players.par_iter().for_each(|player| {
                let _guard = player_handle.enter();
                player.tick(self);
            });
        }
    }

    /// Ticks the game logic for all worlds. This is the part that is affected by `/tick freeze`.
    pub fn tick_worlds(self: &Arc<Self>) {
        self.task_scheduler.tick(self);
        self.scheduled_functions.tick(
            self,
            self.tick_count.load(std::sync::atomic::Ordering::Relaxed) as u64,
        );

        let worlds = self.worlds.load();
        let handle = self.runtime.clone();

        worlds.par_iter().for_each(|world| {
            let _guard = handle.enter();
            world.tick(self);
        });

        // Global tasks
        self.player_data_storage.tick(self);
    }

    /// Updates the tick time statistics with the duration of the last tick.
    pub fn update_tick_times(&self, tick_duration_nanos: i64) {
        let tick_count = self.tick_count.fetch_add(1, Ordering::Relaxed);
        let index = (tick_count % 100) as usize;

        let mut tick_times = self
            .tick_times_nanos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let old_time = tick_times[index];
        tick_times[index] = tick_duration_nanos;
        drop(tick_times);

        self.aggregated_tick_times_nanos
            .fetch_add(tick_duration_nanos - old_time, Ordering::Relaxed);

        let target_tick_nanos = self.tick_rate_manager.nanoseconds_per_tick();
        let idle_nanos = target_tick_nanos.saturating_sub(tick_duration_nanos);

        let sample_slice = [
            tick_duration_nanos.max(target_tick_nanos),
            tick_duration_nanos,
            0,
            idle_nanos,
        ];
        let packet = pumpkin_protocol::java::client::play::CDebugSample::new(
            &sample_slice,
            pumpkin_protocol::codec::var_int::VarInt(0),
        );
        for world in self.worlds.load().iter() {
            let players = world.players.load();
            let recipients = players
                .iter()
                .filter(|player| {
                    player.subscribed_debug_sample.load(Ordering::Relaxed)
                        && player.permission_lvl.load() >= pumpkin_util::PermissionLvl::Two
                })
                .filter_map(|player| player.client.java());
            World::broadcast_java_clients(&packet, recipients);
        }
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
    pub fn get_tick_times_nanos_copy(&self) -> [i64; 100] {
        *self
            .tick_times_nanos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub async fn execute_remote_command(self: &Arc<Self>, command: String) {
        let mut remote_event = crate::plugin::api::events::server::remote_server_command::RemoteServerCommandEvent::new(
            command,
        );
        self.plugin_manager.fire(self, &mut remote_event).await;
    }

    pub async fn register_service(self: &Arc<Self>, service_name: String) {
        let mut service_event =
            crate::plugin::api::events::server::service_register::ServiceRegisterEvent::new(
                service_name,
            );
        self.plugin_manager.fire(self, &mut service_event).await;
    }

    pub async fn unregister_service(self: &Arc<Self>, service_name: String) {
        let mut service_event =
            crate::plugin::api::events::server::service_unregister::ServiceUnregisterEvent::new(
                service_name,
            );
        self.plugin_manager.fire(self, &mut service_event).await;
    }

    pub async fn tab_complete(self: &Arc<Self>, buffer: String, completions: Vec<String>) {
        let mut tab_event = crate::plugin::api::events::server::tab_complete::TabCompleteEvent::new(
            buffer,
            completions,
        );
        self.plugin_manager.fire(self, &mut tab_event).await;
    }

    pub async fn enable_plugin(self: &Arc<Self>, plugin_name: String) {
        let mut enable_event =
            crate::plugin::api::events::server::plugin_enable::PluginEnableEvent::new(plugin_name);
        self.plugin_manager.fire(self, &mut enable_event).await;
    }

    pub async fn disable_plugin(self: &Arc<Self>, plugin_name: String) {
        let mut disable_event =
            crate::plugin::api::events::server::plugin_disable::PluginDisableEvent::new(
                plugin_name,
            );
        self.plugin_manager.fire(self, &mut disable_event).await;
    }
}
