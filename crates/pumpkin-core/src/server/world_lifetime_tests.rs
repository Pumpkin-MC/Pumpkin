use super::*;
use crate::plugin::api::events::world::world_load::WorldUnloadEvent;
use crate::plugin::{BoxFuture, EventHandler, EventPriority};
use pumpkin_util::world_seed::Seed;
use std::path::Path;
use std::sync::atomic::AtomicUsize;

// Avoid Server::new's unrelated key generation and external authentication tasks.
fn offline_server(root: &Path) -> Arc<Server> {
    let basic_config = BasicConfiguration {
        seed: Seed(0),
        default_level_name: root.to_string_lossy().into_owned(),
        use_favicon: false,
        ..Default::default()
    };
    let advanced_config = AdvancedConfiguration::default();
    let permission_manager = Arc::new(PermissionManager::new());
    let command_dispatcher = ArcSwap::from_pointee(default_dispatcher(
        &permission_manager,
        &advanced_config.commands,
    ));
    let management_settings = Arc::new(crate::net::management::hub::ManagementSettings::new(
        &basic_config,
        &advanced_config,
        &advanced_config.networking.management,
    ));
    let listing = std::sync::Mutex::new(CachedStatus::new(&basic_config, "", 0));
    let defaultgamemode = std::sync::Mutex::new(DefaultGamemode {
        gamemode: basic_config.default_gamemode,
    });
    Arc::new(Server {
        level_info: Arc::new(ArcSwap::from_pointee(LevelData::default(basic_config.seed))),
        basic_config,
        advanced_config,
        telemetry_config: TelemetryConfig::default(),
        data: VanillaData {
            banned_ip_list: std::sync::RwLock::default(),
            banned_player_list: std::sync::RwLock::default(),
            operator_config: std::sync::RwLock::default(),
            user_cache: std::sync::RwLock::default(),
            whitelist_config: std::sync::RwLock::default(),
        },
        plugin_manager: Arc::new(PluginManager::new(Vec::new())),
        permission_manager,
        key_store: OnceCell::new(),
        bedrock_oidc_keys: Arc::new(OnceCell::new()),
        listing,
        branding: CachedBranding::new(),
        command_dispatcher,
        block_registry: crate::block::registry::default_registry(),
        item_registry: crate::item::items::default_registry(),
        worlds: ArcSwap::from_pointee(Vec::new()),
        dimensions: vec![Dimension::OVERWORLD],
        container_id: AtomicU32::new(0),
        recipe_manager: Arc::new(recipe::RecipeManager::new()),
        datapack_manager: Arc::new(crate::data::datapack::DatapackManager::new()),
        enchantment_manager: Arc::new(enchantment::EnchantmentManager::new()),
        map_id: AtomicI32::new(0),
        mojang_public_keys: ArcSwap::from_pointee(Vec::new()),
        bossbars: std::sync::Mutex::new(CustomBossbars::new()),
        map_manager: MapManager::new(),
        defaultgamemode,
        player_data_storage: ServerPlayerData::new(
            root.join("players"),
            Duration::from_secs(60),
            false,
        ),
        command_storage: std::sync::Mutex::default(),
        stopwatches: std::sync::Mutex::new(crate::world::stopwatches::Stopwatches::new()),
        random_sequences: std::sync::Mutex::new(
            crate::world::random_sequences::RandomSequences::new(),
        ),
        advancement_manager: Arc::new(AdvancementManager::new(root.join("players"), false)),
        white_list: AtomicBool::new(false),
        tick_rate_manager: Arc::new(ServerTickRateManager::new(20.0)),
        tick_times_nanos: std::sync::Mutex::new([0; 100]),
        aggregated_tick_times_nanos: AtomicI64::new(0),
        tick_count: AtomicI32::new(0),
        debug_profiler: debug_profiler::DebugProfiler::new(),
        server_guid: 0,
        player_idle_timeout: AtomicI32::new(0),
        scheduled_functions: Arc::new(scheduler::ScheduledFunctionQueue::new()),
        tasks: TaskTracker::new(),
        runtime: tokio::runtime::Handle::current(),
        management_hub: Arc::new(crate::net::management::hub::ManagementHub::new(
            management_settings,
        )),
        world_info_writer: Arc::new(AnvilLevelInfo),
    })
}

struct UnloadObserver {
    completed_task: Arc<AtomicBool>,
    calls: AtomicUsize,
}

impl EventHandler<WorldUnloadEvent> for UnloadObserver {
    fn handle_blocking<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a mut WorldUnloadEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            assert!(self.completed_task.load(Ordering::Acquire));
            assert!(event.world.level.cancel_token.is_cancelled());
            assert!(event.world.level.chunk_system_tasks.is_empty());
            assert!(event.world.level.world_portal.load().is_some());
            assert!(
                server
                    .worlds
                    .load()
                    .iter()
                    .any(|world| Arc::ptr_eq(world, &event.world))
            );
            tokio::task::yield_now().await;
            assert!(event.world.level.world_portal.load().is_some());
            self.calls.fetch_add(1, Ordering::Release);
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unload_releases_empty_secondary_world_after_event()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let server = offline_server(directory.path());
    let primary = server.create_world("primary".to_string(), Dimension::OVERWORLD);
    let secondary = server.create_world("secondary".to_string(), Dimension::OVERWORLD);
    let weak_world = Arc::downgrade(&secondary);
    let weak_level = Arc::downgrade(&secondary.level);
    let completed_task = Arc::new(AtomicBool::new(false));
    let task_flag = completed_task.clone();
    secondary.level.spawn_task(async move {
        task_flag.store(true, Ordering::Release);
    });
    let observer = Arc::new(UnloadObserver {
        completed_task,
        calls: AtomicUsize::new(0),
    });
    server
        .plugin_manager
        .register(observer.clone(), EventPriority::Normal, true);
    drop(secondary);

    let unloaded = server.unload_world("secondary").await;
    let released_world = weak_world.upgrade().is_none();
    let released_level = weak_level.upgrade().is_none();
    let event_calls = observer.calls.load(Ordering::Acquire);
    let remaining_worlds = server.worlds.load().len();

    // Observe first; only then break the baseline cycle to release the test's resources.
    if let Some(level) = weak_level.upgrade() {
        level.world_portal.store(Arc::new(None));
    }
    primary.shutdown().await;
    primary.level.world_portal.store(Arc::new(None));
    server.worlds.store(Arc::new(Vec::new()));
    drop(primary);
    drop(server);

    unloaded?;
    assert_eq!(event_calls, 1);
    assert_eq!(remaining_worlds, 1);
    assert!(
        released_world,
        "unloaded secondary World is still strongly owned"
    );
    assert!(
        released_level,
        "unloaded secondary Level is still strongly owned"
    );
    Ok(())
}
