use std::sync::Arc;

use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_core::{
    data::VanillaData,
    net::{
        ClientPlatform, PacketHandlerResult, PacketRateLimiter,
        java::{JavaClient, pending::PendingConnection},
    },
    server::Server,
};
use pumpkin_util::PermissionLvl;
use tokio::net::TcpListener;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub async fn create_test_server(temp_dir: &tempfile::TempDir) -> Result<Arc<Server>, String> {
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
    advanced_config.networking.bedrock.enabled = false;
    advanced_config.networking.bedrock.online_mode = false;
    advanced_config.networking.bedrock.authentication.enabled = false;
    advanced_config.player_data.save_player_data = false;
    advanced_config.advancement.save_advancements = false;

    Server::new(
        basic_config,
        advanced_config,
        TelemetryConfig::default(),
        VanillaData::default(),
        Vec::new(),
    )
    .await
    .map_err(|err| format!("Failed to initialize test Server: {err}"))
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
                        let mut pending = PendingConnection::new(
                            connection,
                            client_addr,
                            client_id,
                            packet_limiter,
                            Arc::downgrade(&server_clone),
                        );
                        match pending.handle_login_sequence(&server_clone).await {
                            PacketHandlerResult::Stop => {
                                pending.close();
                            }
                            PacketHandlerResult::ReadyToPlay(profile, config) => {
                                let mut java_client = JavaClient::from_pending(
                                    pending,
                                    profile.clone(),
                                    config.clone(),
                                );
                                java_client.start_outgoing_packet_task();

                                if let Some((player, world)) = server_clone.add_player(
                                    Arc::new(ClientPlatform::Java(java_client)),
                                    profile,
                                    Some(config),
                                ) {
                                    if let ClientPlatform::Java(client) = player.client.as_ref() {
                                        client.set_player(player.clone());
                                    }
                                    world
                                        .spawn_java_player(
                                            &server_clone.basic_config,
                                            &player,
                                            &server_clone,
                                        )
                                        .await;

                                    if let ClientPlatform::Java(client) = player.client.as_ref() {
                                        client.progress_player_packets(&player, &server_clone).await;
                                        client.close();
                                        client.await_tasks().await;
                                    }
                                    player.remove().await;
                                    server_clone.remove_player(&player);
                                }
                            }
                        }
                    });
                }
            }
        }
    });
}
