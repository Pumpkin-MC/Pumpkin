use crate::block::registry::BlockRegistry;
use crate::command::commands::defaultgamemode::DefaultGamemode;
use crate::data::VanillaData;
use crate::data::player_server::ServerPlayerData;
use crate::item::registry::ItemRegistry;
use crate::plugin::PluginManager;
use crate::server::tick_rate_manager::ServerTickRateManager;
use crate::world::custom_bossbar::CustomBossbars;
use crate::{command::node::dispatcher::CommandDispatcher, world::World, world::map::MapManager};
use arc_swap::ArcSwap;
use connection_cache::{CachedBranding, CachedStatus};
use key_store::KeyStore;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::permission::{PermissionManager, PermissionRegistry};
use pumpkin_world::world_info::{LevelData, WorldInfoWriter};
use rsa::RsaPublicKey;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32};
use tokio::sync::{Mutex, OnceCell, RwLock};
use tokio_util::task::TaskTracker;

mod connection_cache;
mod key_store;
mod lifecycle;
mod player;
mod query;
pub mod recipe;
pub mod scheduler;
pub mod seasonal_events;
mod selectors;
pub mod tick_rate_manager;
pub mod ticker;
mod ticking;

pub use recipe::RecipeManager;

use crate::data::advancement_data::AdvancementManager;
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
    /// Cached Bedrock server private key (process-lifetime). Generated on first Bedrock login and reused.
    pub bedrock_private_key: OnceCell<Arc<pumpkin_util::p384::ecdsa::SigningKey>>,
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
    pub recipe_manager: Arc<recipe::RecipeManager>,
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
    // Manages player advancement
    pub advancement_manager: Arc<AdvancementManager>,
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

#[cfg(test)]
mod tests {
    use super::Server;
    use crate::entity::player::Player;
    use pumpkin_util::Difficulty;
    use std::sync::Arc;

    #[test]
    fn moved_server_api_remains_reachable() {
        // Typed fn-pointer coercions fail to compile if the moved methods
        // change their paths or signatures.
        std::hint::black_box::<fn(&Server) -> usize>(Server::get_player_count);
        std::hint::black_box::<fn(&Server, usize) -> bool>(Server::has_n_players);
        std::hint::black_box::<fn(&Server) -> Vec<Arc<Player>>>(Server::get_all_players);
        std::hint::black_box::<fn(&Server, &str) -> Option<Arc<Player>>>(
            Server::get_player_by_name,
        );
        std::hint::black_box::<fn(&Server, uuid::Uuid) -> Option<Arc<Player>>>(
            Server::get_player_by_uuid,
        );
        std::hint::black_box::<fn(&Server) -> Option<Arc<Player>>>(Server::get_random_player);
        std::hint::black_box::<fn(&Server) -> Difficulty>(Server::get_difficulty);
        std::hint::black_box::<fn(&Server) -> f64>(Server::get_mspt);
        std::hint::black_box::<fn(&Server) -> f64>(Server::get_tps);
        std::hint::black_box::<fn(&Server) -> i64>(Server::get_average_tick_time_nanos);
        std::hint::black_box::<fn(&Server) -> u32>(Server::new_container_id);
        std::hint::black_box::<fn(&Server) -> i32>(Server::next_map_id);
        // Existence checks for the moved async and generic-free entry points.
        std::hint::black_box(Server::new);
        std::hint::black_box(Server::create_world);
        std::hint::black_box(Server::get_world_from_dimension);
        std::hint::black_box(Server::shutdown);
        std::hint::black_box(Server::add_player);
        std::hint::black_box(Server::remove_player);
        std::hint::black_box(Server::get_players_by_ip);
        std::hint::black_box(Server::broadcast_message);
        std::hint::black_box(Server::broadcast_tab_list_header_footer);
        std::hint::black_box(Server::set_difficulty);
        std::hint::black_box(Server::tick);
        std::hint::black_box(Server::tick_players_and_network);
        std::hint::black_box(Server::tick_worlds);
        std::hint::black_box(Server::update_tick_times);
        std::hint::black_box(Server::get_tick_times_nanos_copy);
        std::hint::black_box(Server::encryption_request);
        std::hint::black_box(Server::decrypt);
        std::hint::black_box(Server::digest_secret);
        std::hint::black_box(Server::get_branding);
        std::hint::black_box(Server::get_status);
        std::hint::black_box(Server::select_players);
        std::hint::black_box(Server::select_entities);
    }
}
