// Not warn event sending macros
#![allow(unused_labels)]

#[cfg(not(target_family = "wasm"))]
use crate::net::{Client, lan_broadcast, query, rcon::RCONServer};
use crate::server::{Server, ticker::Ticker};
use log::{Level, LevelFilter, Log};
#[cfg(not(target_family = "wasm"))]
use logging::LOGGER_IMPL;
use plugin::PluginManager;
use pumpkin_config::{BASIC_CONFIG, advanced_config};
use pumpkin_util::text::TextComponent;
use std::sync::atomic::AtomicBool;
use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock},
};
use tokio::select;
use tokio::sync::Notify;
use tokio::sync::Mutex;
#[cfg(not(target_family = "wasm"))]
use tokio::net::TcpListener;
use tokio_util::task::TaskTracker;

pub mod block;
pub mod command;
pub mod data;
pub mod entity;
pub mod error;
pub mod item;
pub mod net;
pub mod plugin;
pub mod server;
pub mod world;

#[cfg(not(target_family = "wasm"))]
pub mod logging;

const GIT_VERSION: &str = env!("GIT_VERSION");

pub static PLUGIN_MANAGER: LazyLock<Mutex<PluginManager>> =
    LazyLock::new(|| Mutex::new(PluginManager::new()));

pub static SHOULD_STOP: AtomicBool = AtomicBool::new(false);
pub static STOP_INTERRUPT: LazyLock<Notify> = LazyLock::new(Notify::new);

pub fn stop_server() {
    SHOULD_STOP.store(true, std::sync::atomic::Ordering::Relaxed);
    STOP_INTERRUPT.notify_waiters();
}

pub struct PumpkinServer {
    pub server: Arc<Server>,
    #[cfg(not(target_family = "wasm"))]
    pub listener: TcpListener,
    pub server_addr: SocketAddr,
}

impl PumpkinServer {
    #[cfg(not(target_family = "wasm"))]
    pub async fn new() -> Self {
        let server = Arc::new(Server::new());

        for world in &*server.worlds.read().await {
            world.level.read_spawn_chunks(&Server::spawn_chunks()).await;
        }

        // Setup the TCP server socket.
        let listener = tokio::net::TcpListener::bind(BASIC_CONFIG.server_address)
            .await
            .expect("Failed to start `TcpListener`");
        // In the event the user puts 0 for their port, this will allow us to know what port it is running on
        let addr = listener
            .local_addr()
            .expect("Unable to get the address of the server!");

        let rcon = advanced_config().networking.rcon.clone();

        let mut ticker = Ticker::new(BASIC_CONFIG.tps);

        if let Some((wrapper, _)) = &*LOGGER_IMPL {
            if let Some(rl) = wrapper.take_readline() {
                logging::setup_console(rl, server.clone());
            }
        }

        if rcon.enabled {
            let rcon_server = server.clone();
            server.spawn_task(async move {
                RCONServer::run(&rcon, rcon_server).await.unwrap();
            });
        }

        if advanced_config().networking.query.enabled {
            log::info!("Query protocol is enabled. Starting...");
            server.spawn_task(query::start_query_handler(server.clone(), addr));
        }

        if advanced_config().networking.lan_broadcast.enabled {
            log::info!("LAN broadcast is enabled. Starting...");
            server.spawn_task(lan_broadcast::start_lan_broadcast(addr));
        }

        // Ticker
        {
            let ticker_server = server.clone();
            server.spawn_task(async move {
                ticker.run(&ticker_server).await;
            });
        };

        Self {
            server: server.clone(),
            listener,
            server_addr: addr,
        }
    }

    pub async fn init_plugins(&self) {
        let mut loader_lock = PLUGIN_MANAGER.lock().await;
        loader_lock.set_server(self.server.clone());
        if let Err(err) = loader_lock.load_plugins().await {
            log::error!("{}", err);
        };
    }

    #[cfg(not(target_family = "wasm"))]
    pub async fn start(self) {
        let mut master_client_id: usize = 0;
        let tasks = TaskTracker::new();

        while !SHOULD_STOP.load(std::sync::atomic::Ordering::Relaxed) {
            let await_new_client = || async {
                let t1 = self.listener.accept();
                let t2 = STOP_INTERRUPT.notified();

                select! {
                    client = t1 => Some(client.unwrap()),
                    () = t2 => None,
                }
            };

            // Asynchronously wait for an inbound socket.
            let Some((connection, client_addr)) = await_new_client().await else {
                break;
            };

            if let Err(e) = connection.set_nodelay(true) {
                log::warn!("Failed to set TCP_NODELAY {e}");
            }

            let id = master_client_id;
            master_client_id = master_client_id.wrapping_add(1);

            let formatted_address = if BASIC_CONFIG.scrub_ips {
                scrub_address(&format!("{client_addr}"))
            } else {
                format!("{client_addr}")
            };
            log::info!(
                "Accepted connection from: {} (id {})",
                formatted_address,
                id
            );

            let mut client = Client::new(connection, client_addr, id);
            client.init();
            let server = self.server.clone();

            tasks.spawn(async move {
                // TODO: We need to add a time-out here for un-cooperative clients
                client.process_packets(&server).await;

                if client
                    .make_player
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    // Client is kicked if this fails
                    if let Some((player, world)) = server.add_player(client).await {
                        world
                            .spawn_player(&BASIC_CONFIG, player.clone(), &server)
                            .await;

                        player.process_packets(&server).await;
                        player.close().await;

                        //TODO: Move these somewhere less likely to be forgotten
                        log::debug!("Cleaning up player for id {}", id);

                        // Save player data on disconnect
                        if let Err(e) = server
                            .player_data_storage
                            .handle_player_leave(&player)
                            .await
                        {
                            log::error!("Failed to save player data on disconnect: {}", e);
                        }

                        // Remove the player from its world
                        player.remove().await;
                        // Tick down the online count
                        server.remove_player().await;
                    }
                } else {
                    // Also handle case of client connects but does not become a player (like a server
                    // ping)
                    client.close();
                    log::debug!("Awaiting tasks for client {}", id);
                    client.await_tasks().await;
                    log::debug!("Finished awaiting tasks for client {}", id);
                }
            });
        }

        log::info!("Stopped accepting incoming connections");

        if let Err(e) = self
            .server
            .player_data_storage
            .save_all_players(&self.server)
            .await
        {
            log::error!("Error saving all players during shutdown: {}", e);
        }

        let kick_message = TextComponent::text("Server stopped");
        for player in self.server.get_all_players().await {
            player.kick(kick_message.clone()).await;
        }

        log::info!("Ending player tasks");

        tasks.close();
        tasks.wait().await;

        log::info!("Starting save.");

        self.server.shutdown().await;

        log::info!("Completed save!");

        // Explicitly drop the line reader to return the terminal to the original state.
        if let Some((wrapper, _)) = &*LOGGER_IMPL {
            if let Some(rl) = wrapper.take_readline() {
                let _ = rl;
            }
        }
    }
}

fn scrub_address(ip: &str) -> String {
    ip.chars()
        .map(|ch| if ch == '.' || ch == ':' { ch } else { 'x' })
        .collect()
}
