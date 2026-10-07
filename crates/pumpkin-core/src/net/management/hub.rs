use arc_swap::ArcSwap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicU32, AtomicU64},
};
use tokio::sync::broadcast;
use tracing::warn;

use super::{
    dto::{IpBanDto, OperatorDto, PlayerDto, ServerState, TypedGameRule, UserBanDto},
    rpc::RpcNotification,
};
use pumpkin_config::{
    AdvancedConfiguration, BasicConfiguration, networking::management::ManagementServerConfig,
};

pub struct ManagementSettings {
    pub autosave: AtomicBool,
    pub difficulty: ArcSwap<String>,
    pub enforce_allowlist: AtomicBool,
    pub max_players: AtomicU32,
    pub pause_when_empty_seconds: AtomicI32,
    pub allow_flight: AtomicBool,
    pub motd: ArcSwap<String>,
    pub spawn_protection_radius: AtomicU32,
    pub force_game_mode: AtomicBool,
    pub game_mode: ArcSwap<String>,
    pub view_distance: AtomicU32,
    pub simulation_distance: AtomicU32,
    pub accept_transfers: AtomicBool,
    pub status_heartbeat_interval: AtomicU64,
    pub operator_user_permission_level: AtomicU8,
    pub hide_online_players: AtomicBool,
    pub status_replies: AtomicBool,
    pub entity_broadcast_range: AtomicU32,
}

impl ManagementSettings {
    #[must_use]
    pub fn new(
        basic: &BasicConfiguration,
        advanced: &AdvancedConfiguration,
        management: &ManagementServerConfig,
    ) -> Self {
        Self {
            autosave: AtomicBool::new(true),
            difficulty: ArcSwap::from_pointee(
                format!("{:?}", basic.default_difficulty).to_lowercase(),
            ),
            enforce_allowlist: AtomicBool::new(basic.enforce_whitelist),
            max_players: AtomicU32::new(advanced.networking.java.max_players),
            pause_when_empty_seconds: AtomicI32::new(60),
            allow_flight: AtomicBool::new(false),
            motd: ArcSwap::from_pointee(advanced.networking.java.motd.clone()),
            spawn_protection_radius: AtomicU32::new(basic.spawn_protection),
            force_game_mode: AtomicBool::new(basic.force_gamemode),
            game_mode: ArcSwap::from_pointee(
                format!("{:?}", basic.default_gamemode).to_lowercase(),
            ),
            view_distance: AtomicU32::new(advanced.networking.java.view_distance.get() as u32),
            simulation_distance: AtomicU32::new(
                advanced.networking.java.simulation_distance.get() as u32
            ),
            accept_transfers: AtomicBool::new(basic.accepts_transfers),
            status_heartbeat_interval: AtomicU64::new(management.status_heartbeat_interval.into()),
            operator_user_permission_level: AtomicU8::new(basic.op_permission_level as u8),
            hide_online_players: AtomicBool::new(false),
            status_replies: AtomicBool::new(true),
            entity_broadcast_range: AtomicU32::new(100),
        }
    }
}

pub struct ManagementHub {
    sender: broadcast::Sender<String>,
    pub settings: Arc<ManagementSettings>,
}

impl ManagementHub {
    #[must_use]
    pub fn new(settings: Arc<ManagementSettings>) -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self { sender, settings }
    }

    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }

    fn broadcast(&self, notification: &RpcNotification) {
        if self.sender.receiver_count() == 0 {
            return;
        }
        match serde_json::to_string(notification) {
            Ok(json) => {
                let _ = self.sender.send(json);
            }
            Err(e) => {
                warn!("Failed to serialize management notification: {e}");
            }
        }
    }

    pub fn broadcast_server_started(&self) {
        self.broadcast(&RpcNotification::new(
            "minecraft:notification/server/started",
            None,
        ));
    }

    pub fn broadcast_server_stopping(&self) {
        self.broadcast(&RpcNotification::new(
            "minecraft:notification/server/stopping",
            None,
        ));
    }

    pub fn broadcast_server_saving(&self) {
        self.broadcast(&RpcNotification::new(
            "minecraft:notification/server/saving",
            None,
        ));
    }

    pub fn broadcast_server_saved(&self) {
        self.broadcast(&RpcNotification::new(
            "minecraft:notification/server/saved",
            None,
        ));
    }

    pub fn broadcast_server_activity(&self) {
        self.broadcast(&RpcNotification::new(
            "minecraft:notification/server/activity",
            None,
        ));
    }

    pub fn broadcast_world_upgrade_started(&self) {
        self.broadcast(&RpcNotification::new(
            "minecraft:notification/world/upgrade_started",
            None,
        ));
    }

    pub fn broadcast_world_upgrade_progress(&self, progress: f32) {
        self.broadcast(&RpcNotification::new(
            "minecraft:notification/world/upgrade_progress",
            Some(serde_json::json!(progress)),
        ));
    }

    pub fn broadcast_world_upgrade_finished(&self) {
        self.broadcast(&RpcNotification::new(
            "minecraft:notification/world/upgrade_finished",
            None,
        ));
    }

    pub fn broadcast_world_upgrade_failed(&self, reason: &str) {
        self.broadcast(&RpcNotification::new(
            "minecraft:notification/world/upgrade_failed",
            Some(serde_json::json!(reason)),
        ));
    }

    pub fn broadcast_player_joined(&self, player: &PlayerDto) {
        if let Ok(val) = serde_json::to_value(player) {
            self.broadcast(&RpcNotification::new(
                "minecraft:notification/players/joined",
                Some(val),
            ));
        }
    }

    pub fn broadcast_player_left(&self, player: &PlayerDto) {
        if let Ok(val) = serde_json::to_value(player) {
            self.broadcast(&RpcNotification::new(
                "minecraft:notification/players/left",
                Some(val),
            ));
        }
    }

    pub fn broadcast_player_oped(&self, op: &OperatorDto) {
        if let Ok(val) = serde_json::to_value(op) {
            self.broadcast(&RpcNotification::new(
                "minecraft:notification/operators/added",
                Some(val),
            ));
        }
    }

    pub fn broadcast_player_deoped(&self, op: &OperatorDto) {
        if let Ok(val) = serde_json::to_value(op) {
            self.broadcast(&RpcNotification::new(
                "minecraft:notification/operators/removed",
                Some(val),
            ));
        }
    }

    pub fn broadcast_player_added_to_allowlist(&self, player: &PlayerDto) {
        if let Ok(val) = serde_json::to_value(player) {
            self.broadcast(&RpcNotification::new(
                "minecraft:notification/allowlist/added",
                Some(val),
            ));
        }
    }

    pub fn broadcast_player_removed_from_allowlist(&self, player: &PlayerDto) {
        if let Ok(val) = serde_json::to_value(player) {
            self.broadcast(&RpcNotification::new(
                "minecraft:notification/allowlist/removed",
                Some(val),
            ));
        }
    }

    pub fn broadcast_ip_banned(&self, ban: &IpBanDto) {
        if let Ok(val) = serde_json::to_value(ban) {
            self.broadcast(&RpcNotification::new(
                "minecraft:notification/ip_bans/added",
                Some(val),
            ));
        }
    }

    pub fn broadcast_ip_unbanned(&self, ip: &str) {
        self.broadcast(&RpcNotification::new(
            "minecraft:notification/ip_bans/removed",
            Some(serde_json::json!(ip)),
        ));
    }

    pub fn broadcast_player_banned(&self, ban: &UserBanDto) {
        if let Ok(val) = serde_json::to_value(ban) {
            self.broadcast(&RpcNotification::new(
                "minecraft:notification/bans/added",
                Some(val),
            ));
        }
    }

    pub fn broadcast_player_unbanned(&self, player: &PlayerDto) {
        if let Ok(val) = serde_json::to_value(player) {
            self.broadcast(&RpcNotification::new(
                "minecraft:notification/bans/removed",
                Some(val),
            ));
        }
    }

    pub fn broadcast_gamerule_changed(&self, rule: &TypedGameRule) {
        if let Ok(val) = serde_json::to_value(rule) {
            self.broadcast(&RpcNotification::new(
                "minecraft:notification/gamerules/updated",
                Some(val),
            ));
        }
    }

    pub fn broadcast_status_heartbeat(&self, status: &ServerState) {
        if let Ok(val) = serde_json::to_value(status) {
            self.broadcast(&RpcNotification::new(
                "minecraft:notification/server/status",
                Some(val),
            ));
        }
    }
}
