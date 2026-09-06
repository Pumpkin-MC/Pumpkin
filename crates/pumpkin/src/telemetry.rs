use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use pumpkin_config::TelemetryConfig;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::server::Server;

/// Filename used to persist the server ID across restarts.
pub const SERVER_ID_FILENAME: &str = ".pumpkin-server-id";

/// Information about an active plugin on the server.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PluginInfo {
    /// The name of the plugin.
    pub name: String,
    /// The version of the plugin.
    pub version: String,
}

/// The telemetry heartbeat payload transmitted to the backend.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HeartbeatPayload {
    /// Pseudonymous UUID identifying this server installation.
    pub server_id: Uuid,
    /// The server software implementation type (e.g. "pumpkin").
    pub server_type: String,
    /// The Pumpkin server software version.
    pub server_version: String,
    /// The Minecraft game version string.
    pub minecraft_version: String,
    /// The network protocol version number.
    pub protocol_version: i32,
    /// Current number of connected players.
    pub online_players: usize,
    /// Maximum number of players allowed.
    pub max_players: u32,
    /// Host operating system name.
    pub os: String,
    /// Host CPU architecture.
    pub arch: String,
    /// Number of available CPU cores.
    pub cpu_cores: usize,
    /// CPU model name or brand.
    pub cpu_model: String,
    /// RAM allocated/used by the system in MiB.
    pub ram_allocated_mb: u64,
    /// Total physical RAM available to the system in MiB.
    pub total_ram_mb: u64,
    /// List of active plugins.
    pub plugins: Vec<PluginInfo>,
    /// Whether the server has opted into public directory listing.
    pub is_public: bool,
    /// Public name of the server if opted into public directory listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_name: Option<String>,
}

/// The response payload received from the telemetry backend.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HeartbeatResponse {
    /// Response status ("ok", "throttled", etc.).
    pub status: String,
    /// Status or error message from the backend.
    #[serde(default)]
    pub message: Option<String>,
    /// Confirmed server ID acknowledged by the backend.
    #[serde(default)]
    pub server_id: Option<String>,
    /// Recommended seconds to wait before the next heartbeat.
    #[serde(default)]
    pub next_ping_seconds: Option<u64>,
}

/// Loads an existing server UUID from the given path or generates and persists a new one.
pub fn get_or_create_server_id(path: &Path) -> Uuid {
    if path.exists()
        && let Ok(content) = std::fs::read_to_string(path)
    {
        let trimmed = content.trim();
        if let Ok(id) = Uuid::parse_str(trimmed) {
            return id;
        }
        tracing::warn!(
            "Failed to parse server ID from {}, generating a new one",
            path.display()
        );
    }

    let id = Uuid::new_v4();
    if let Err(err) = std::fs::write(path, id.to_string()) {
        tracing::warn!("Failed to save server ID to {}: {err}", path.display());
    }
    id
}

/// Resolves the server ID by checking default locations or generating a new one.
#[must_use]
pub fn resolve_server_id() -> Uuid {
    let root_path = Path::new(SERVER_ID_FILENAME);
    if root_path.exists() {
        return get_or_create_server_id(root_path);
    }
    let data_path = Path::new("data").join(SERVER_ID_FILENAME);
    if data_path.exists() {
        return get_or_create_server_id(&data_path);
    }
    get_or_create_server_id(root_path)
}

/// Builds the telemetry heartbeat payload from current server and runtime state.
#[must_use]
pub fn build_heartbeat_payload(
    server: &Server,
    server_id: Uuid,
    telemetry_config: &TelemetryConfig,
) -> HeartbeatPayload {
    let os = sysinfo::System::long_os_version().unwrap_or_else(|| {
        let name = sysinfo::System::name().unwrap_or_else(|| std::env::consts::OS.to_string());
        sysinfo::System::os_version().map_or(name.clone(), |ver| format!("{name} {ver}"))
    });
    let arch = std::env::consts::ARCH.to_string();
    let cpu_cores = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);

    let (ram_allocated_mb, total_ram_mb, cpu_model) = if sysinfo::IS_SUPPORTED_SYSTEM {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        let ram_allocated = sys.used_memory() / (1024 * 1024);
        let total_ram = sys.total_memory() / (1024 * 1024);
        let cpu_model = sys.cpus().first().map_or_else(
            || "Unknown".to_string(),
            |c| {
                let brand = c.brand().trim();
                if brand.is_empty() {
                    "Unknown".to_string()
                } else {
                    brand.to_string()
                }
            },
        );
        (ram_allocated, total_ram, cpu_model)
    } else {
        (0, 0, "Unknown".to_string())
    };

    let plugins = server
        .plugin_manager
        .active_plugins()
        .into_iter()
        .map(|p| PluginInfo {
            name: p.name,
            version: p.version,
        })
        .collect();

    let is_public = telemetry_config.public;
    let public_name = if is_public {
        telemetry_config.server_name.clone()
    } else {
        None
    };

    HeartbeatPayload {
        server_id,
        server_type: "pumpkin".to_string(),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
        minecraft_version: pumpkin_data::packet::CURRENT_MC_VERSION.to_string(),
        protocol_version: pumpkin_data::packet::CURRENT_MC_VERSION.protocol_version(),
        online_players: server.get_player_count(),
        max_players: server.advanced_config.networking.java.max_players,
        os,
        arch,
        cpu_cores,
        cpu_model,
        ram_allocated_mb,
        total_ram_mb,
        plugins,
        is_public,
        public_name,
    }
}

/// Starts the background telemetry reporting task using the server's telemetry configuration.
pub fn start_telemetry(server: Arc<Server>) {
    let config = server.telemetry_config.clone();
    start_telemetry_with_config(server, &config);
}

/// Starts the background telemetry reporting task with a specific configuration.
pub fn start_telemetry_with_config(server: Arc<Server>, config: &TelemetryConfig) {
    if !config.enabled {
        tracing::trace!("Telemetry is disabled in configuration.");
        return;
    }

    tracing::info!(
        "Anonymous server telemetry is enabled. Sending periodic heartbeats to {}",
        config.endpoint
    );

    let user_agent = format!("Pumpkin-Server/{}", env!("CARGO_PKG_VERSION"));
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(&user_agent)
        .build()
    {
        Ok(c) => c,
        Err(err) => {
            tracing::debug!("Failed to create telemetry HTTP client: {err}");
            return;
        }
    };

    let server_id = resolve_server_id();
    let interval_secs = config.interval_secs.max(60);
    let endpoint = config.endpoint.clone();
    let config = config.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        while !crate::SHOULD_STOP.load(Ordering::Relaxed) {
            interval.tick().await;
            if crate::SHOULD_STOP.load(Ordering::Relaxed) {
                break;
            }

            let payload = build_heartbeat_payload(&server, server_id, &config);

            let res = client
                .post(&endpoint)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .header(reqwest::header::USER_AGENT, &user_agent)
                .json(&payload)
                .send()
                .await;

            match res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        tracing::trace!("Telemetry heartbeat successfully transmitted");
                    } else {
                        tracing::debug!(
                            "Telemetry heartbeat received non-success HTTP status: {}",
                            resp.status()
                        );
                    }
                }
                Err(err) => {
                    tracing::debug!("Telemetry heartbeat failed to send: {err}");
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_id_generation_and_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let id_path = dir.path().join(".pumpkin-server-id");

        assert!(!id_path.exists());
        let id1 = get_or_create_server_id(&id_path);
        assert!(id_path.exists());

        let id2 = get_or_create_server_id(&id_path);
        assert_eq!(id1, id2);

        // Corrupted file test
        std::fs::write(&id_path, "not-a-uuid").unwrap();
        let id3 = get_or_create_server_id(&id_path);
        assert_ne!(id1, id3);
    }

    #[test]
    fn heartbeat_payload_serialization() {
        let payload = HeartbeatPayload {
            server_id: Uuid::parse_str("a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d").unwrap(),
            server_type: "pumpkin".to_string(),
            server_version: "0.3.2".to_string(),
            minecraft_version: "1.21.4".to_string(),
            protocol_version: 769,
            online_players: 42,
            max_players: 100,
            os: "Ubuntu 24.04 LTS".to_string(),
            arch: "x86_64".to_string(),
            cpu_cores: 8,
            cpu_model: "AMD Ryzen 7 5800X 8-Core Processor".to_string(),
            ram_allocated_mb: 8192,
            total_ram_mb: 16384,
            plugins: vec![
                PluginInfo {
                    name: "Essential Admin Tools".to_string(),
                    version: "1.0.0".to_string(),
                },
                PluginInfo {
                    name: "PumpkinAuth".to_string(),
                    version: "1.2.0".to_string(),
                },
            ],
            is_public: true,
            public_name: Some("My High-Performance Pumpkin SMP".to_string()),
        };

        let json_str = serde_json::to_string_pretty(&payload).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["server_id"], "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d");
        assert_eq!(parsed["server_type"], "pumpkin");
        assert_eq!(parsed["server_version"], "0.3.2");
        assert_eq!(parsed["minecraft_version"], "1.21.4");
        assert_eq!(parsed["protocol_version"], 769);
        assert_eq!(parsed["online_players"], 42);
        assert_eq!(parsed["max_players"], 100);
        assert_eq!(parsed["os"], "Ubuntu 24.04 LTS");
        assert_eq!(parsed["arch"], "x86_64");
        assert_eq!(parsed["cpu_cores"], 8);
        assert_eq!(parsed["cpu_model"], "AMD Ryzen 7 5800X 8-Core Processor");
        assert_eq!(parsed["ram_allocated_mb"], 8192);
        assert_eq!(parsed["total_ram_mb"], 16384);
        assert_eq!(parsed["is_public"], true);
        assert_eq!(parsed["public_name"], "My High-Performance Pumpkin SMP");
        assert_eq!(parsed["plugins"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn heartbeat_response_deserialization() {
        let json_data = r#"{
            "status": "ok",
            "message": "Heartbeat accepted",
            "server_id": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
            "next_ping_seconds": 300
        }"#;

        let resp: HeartbeatResponse = serde_json::from_str(json_data).unwrap();
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.message.as_deref(), Some("Heartbeat accepted"));
        assert_eq!(
            resp.server_id.as_deref(),
            Some("a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d")
        );
        assert_eq!(resp.next_ping_seconds, Some(300));
    }
}
