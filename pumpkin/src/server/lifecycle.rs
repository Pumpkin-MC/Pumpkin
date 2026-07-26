use super::Server;
use super::connection_cache::{CachedBranding, CachedStatus};
use super::key_store::KeyStore;
use super::recipe;
use crate::command::commands::default_dispatcher;
use crate::command::commands::defaultgamemode::DefaultGamemode;
use crate::data::VanillaData;
use crate::data::advancement_data::AdvancementManager;
use crate::data::player_server::ServerPlayerData;
use crate::net::authentication::fetch_mojang_public_keys;
use crate::plugin::PluginManager;
use crate::server::scheduler::TaskScheduler;
use crate::server::tick_rate_manager::ServerTickRateManager;
use crate::world::World;
use crate::world::WorldPortal;
use crate::world::custom_bossbar::CustomBossbars;
use crate::world::map::MapManager;
use arc_swap::ArcSwap;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::permission::{PermissionManager, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use pumpkin_world::dimension::into_level;
use pumpkin_world::world::WorldPortalExt;
use pumpkin_world::world_info::anvil::{
    AnvilLevelInfo, LEVEL_DAT_BACKUP_FILE_NAME, LEVEL_DAT_FILE_NAME,
};
use pumpkin_world::world_info::{LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter};
use std::fs;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64};
use std::time::Duration;
use tokio::sync::{Mutex, OnceCell, RwLock};
use tokio::task::JoinHandle;
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info, warn};

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

        crate::command::set_broadcast_console_to_ops(
            advanced_config.commands.broadcast_console_to_ops,
        );

        let world_path = basic_config.get_world_path();

        let block_registry = crate::block::registry::default_registry();

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

        let listing = Mutex::new(CachedStatus::new(
            &basic_config,
            &advanced_config.networking.java.motd,
            advanced_config.networking.java.max_players,
        ));
        let defaultgamemode = Mutex::new(DefaultGamemode {
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

        let mojang_keys_task = tokio::spawn({
            let auth_config = advanced_config.networking.java.authentication.clone();
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
            recipe_manager: Arc::new(recipe::RecipeManager::new()),
            map_id: level_info.load().map_id.into(),
            worlds: ArcSwap::from_pointee(vec![]),
            dimensions,
            command_dispatcher,
            block_registry: block_registry.clone(),
            item_registry: crate::item::items::default_registry(),
            key_store: OnceCell::new(),
            bedrock_oidc_keys: OnceCell::new(),
            bedrock_private_key: OnceCell::new(),
            listing,
            branding: CachedBranding::new(),
            bossbars: Mutex::new(CustomBossbars::new()),
            map_manager: MapManager::new(),
            defaultgamemode,
            player_data_storage,
            advancement_manager,
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
                let portal: Arc<dyn WorldPortalExt> = Arc::new(WorldPortal(world.clone()));
                level.world_portal.store(Arc::new(Some(portal)));
                world
            })
        };

        info!("Starting parallel world load...");
        let mut world_futures = Vec::new();
        for dim in &server.dimensions {
            world_futures.push(world_loader(dim.clone()));
        }

        let (worlds_results, keys) =
            tokio::join!(futures::future::join_all(world_futures), mojang_keys_task);

        let mut worlds_vec = Vec::new();
        for world_result in worlds_results {
            worlds_vec.push(world_result.expect("World loading panicked"));
        }

        server.worlds.store(Arc::new(worlds_vec));
        if let Ok(k) = keys {
            server.mojang_public_keys.store(Arc::new(k));
        }

        info!("All worlds loaded successfully.");

        if server.advanced_config.networking.bedrock.online_mode {
            let server_clone = server.clone();
            tokio::spawn(async move {
                server_clone
                    .bedrock_oidc_keys
                    .get_or_init(|| async {
                        tokio::task::block_in_place(|| {
                            let auth = &server_clone
                                .advanced_config
                                .networking
                                .bedrock
                                .authentication;
                            pumpkin_util::jwt::fetch_oidc_jwks(
                                auth.url.as_deref(),
                                auth.connect_timeout,
                                auth.read_timeout,
                            )
                            .unwrap_or_else(|e| {
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
        self.worlds
            .load()
            .iter()
            .find(|w| w.dimension.minecraft_name == dimension.minecraft_name)
            .cloned()
            .unwrap_or_else(|| {
                self.worlds
                    .load()
                    .first()
                    .expect("Default world should exist")
                    .clone()
            })
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
            let portal: Arc<dyn WorldPortalExt> = Arc::new(WorldPortal(world.clone()));
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
}
