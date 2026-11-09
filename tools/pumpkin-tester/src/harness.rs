use std::sync::Arc;

use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_core::{
    data::VanillaData,
    net::{PacketRateLimiter, java::pending::PendingConnection},
    serve_java_connection,
    server::Server,
};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::{PermissionLvl, math::vector2::Vector2};
use tokio::net::TcpListener;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub fn test_configs(temp_dir: &tempfile::TempDir) -> (BasicConfiguration, AdvancedConfiguration) {
    let basic_config = BasicConfiguration {
        default_level_name: temp_dir.path().to_string_lossy().into_owned(),
        allow_nether: false,
        allow_end: false,
        allow_chat_reports: false,
        op_permission_level: PermissionLvl::Four,
        ..Default::default()
    };

    let mut advanced_config = AdvancedConfiguration::default();
    advanced_config.networking.java.online_mode = false;
    advanced_config.networking.java.encryption = false;
    advanced_config.networking.java.compression.enabled = false;
    advanced_config.networking.java.keep_alive_time = 600;
    advanced_config.networking.bedrock.enabled = false;
    advanced_config.networking.bedrock.online_mode = false;
    advanced_config.networking.bedrock.authentication.enabled = false;
    advanced_config.player_data.save_player_data = false;
    advanced_config.advancement.save_advancements = false;
    advanced_config.update_checker.enabled = false;

    (basic_config, advanced_config)
}

pub async fn create_test_server(temp_dir: &tempfile::TempDir) -> Result<Arc<Server>, String> {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/tests/datapacks/pumpkin-unit-test");
    let target = temp_dir.path().join("datapacks/pumpkin-unit-test");
    copy_datapack(&source, &target)?;
    let (basic_config, advanced_config) = test_configs(temp_dir);
    let server = Server::new(
        basic_config,
        advanced_config,
        TelemetryConfig::default(),
        VanillaData::default(),
        Vec::new(),
    )
    .await
    .map_err(|err| format!("Failed to initialize test Server: {err}"))?;
    let mut level_info = (**server.level_info.load()).clone();
    level_info
        .data_packs
        .enabled
        .push("file/pumpkin-unit-test".to_string());
    server.level_info.store(Arc::new(level_info));
    server.reload_datapacks(&server);
    Ok(server)
}

fn copy_datapack(source: &std::path::Path, target: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(target).map_err(|error| error.to_string())?;
    for entry in std::fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let destination = target.join(entry.file_name());
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            copy_datapack(&entry.path(), &destination)?;
        } else {
            std::fs::copy(entry.path(), destination).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

/// Generates the overworld spawn chunk, so a bot's join waits on the network, not on world
/// generation, which can outlast the bot packet timeout on a slow runner.
pub async fn prepare_spawn_chunk(server: &Server) {
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let info = world.level_info.load();
    let spawn_chunk = Vector2::new(info.spawn_x >> 4, info.spawn_z >> 4);
    world.level.get_or_fetch_chunk(spawn_chunk, |_| ()).await;
}

pub fn spawn_java_listener(
    listener: TcpListener,
    server: Arc<Server>,
    cancel: CancellationToken,
    tasks: &TaskTracker,
) {
    let loop_tasks = tasks.clone();
    tasks.spawn(async move {
        let mut next_client_id: u64 = 1;
        loop {
            tokio::select! {
                () = cancel.cancelled() => break,
                accepted = listener.accept() => {
                    let Ok((connection, client_addr)) = accepted else {
                        continue;
                    };
                    let _ = connection.set_nodelay(true);
                    let client_id = next_client_id;
                    next_client_id += 1;
                    let server_clone = server.clone();

                    loop_tasks.spawn(async move {
                        let packet_limiter = PacketRateLimiter::from_config(
                            &server_clone.advanced_config.networking.java.packet_limiter,
                        );
                        let pending = PendingConnection::new(
                            connection,
                            client_addr,
                            client_id,
                            packet_limiter,
                            Arc::downgrade(&server_clone),
                        );
                        serve_java_connection(server_clone, pending).await;
                    });
                }
            }
        }
    });
}
