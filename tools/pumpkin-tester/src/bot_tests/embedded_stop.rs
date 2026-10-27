use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};

use pumpkin_config::TelemetryConfig;
use pumpkin_core::{
    PumpkinServer, STOP_INTERRUPT,
    data::VanillaData,
    net::{PacketRateLimiter, java::pending::PendingConnection},
    serve_java_connection,
};
use tokio::time::timeout;

use crate::{bot::HeadlessBot, harness};

const STOP_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn run_embedded_stop_test(summary_lines: &mut Vec<String>) -> Result<(), String> {
    let test_start = Instant::now();
    let temp_dir = tempfile::tempdir()
        .map_err(|err| format!("Failed to create embedded world directory: {err}"))?;
    let (basic_config, mut advanced_config) = harness::test_configs(&temp_dir);
    advanced_config.networking.java.enabled = false;
    advanced_config.player_data.save_player_data = true;
    let telemetry_config = TelemetryConfig {
        enabled: false,
        ..Default::default()
    };

    let pumpkin_server = PumpkinServer::new(
        basic_config,
        advanced_config,
        telemetry_config,
        VanillaData::default(),
        Vec::new(),
        false,
    )
    .await
    .map_err(|err| format!("Failed to initialize embedded server: {err}"))?;
    let server = pumpkin_server.server.clone();
    harness::prepare_spawn_chunk(&server).await;

    let (client, transport) = tokio::io::duplex(64 * 1024);
    let (server_read, server_write) = tokio::io::split(transport);
    let (client_read, client_write) = tokio::io::split(client);
    let pending = PendingConnection::from_transport(
        Box::new(server_read),
        Box::new(server_write),
        SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        1,
        PacketRateLimiter::from_config(&server.advanced_config.networking.java.packet_limiter),
        Arc::downgrade(&server),
    );
    let connection = tokio::spawn(serve_java_connection(server.clone(), pending));

    let session = async {
        let joined = HeadlessBot::join(
            Box::new(client_read),
            Box::new(client_write),
            0,
            "EmbeddedBot",
        )
        .await;
        let player_id = server
            .get_all_players()
            .first()
            .map(|player| player.gameprofile.id);
        server.stop();
        joined.map(|bot| (bot, player_id))
    };
    let ((), joined) = timeout(STOP_TIMEOUT, async {
        tokio::join!(pumpkin_server.start(), session)
    })
    .await
    .map_err(|_| "Embedded server did not finish stopping".to_string())?;
    let (_bot, player_id) = joined?;

    timeout(STOP_TIMEOUT, connection)
        .await
        .map_err(|_| "Embedded connection did not end after stop".to_string())?
        .map_err(|err| format!("Embedded connection task failed: {err}"))?;

    if STOP_INTERRUPT.is_cancelled() {
        return Err("Stopping the embedded server stopped the whole process".to_string());
    }

    let player_id = player_id.ok_or("EmbeddedBot never showed up on the server")?;
    let player_data = server
        .basic_config
        .get_world_path()
        .join("players/data")
        .join(format!("{player_id}.dat"));
    if !player_data.exists() {
        return Err(format!(
            "Embedded server did not save {}",
            player_data.display()
        ));
    }

    summary_lines.push(format!(
        "[PASS] bot:embedded_stop ({}ms)",
        test_start.elapsed().as_millis()
    ));
    Ok(())
}
