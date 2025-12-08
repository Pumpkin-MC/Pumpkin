// Not warn event sending macros
#![allow(unused_labels)]

use crate::logging::{GzipRollingLogger, ReadlineLogWrapper};
use crate::net::DisconnectReason;
use crate::net::bedrock::BedrockClient;
use crate::net::java::JavaClient;
use crate::net::{lan_broadcast::LANBroadcast, query, rcon::RCONServer};
use crate::server::{Server, ticker::Ticker};
use log::{Level, LevelFilter};
use net::authentication::fetch_mojang_public_keys;
use plugin::PluginManager;
use plugin::server::server_command::ServerCommandEvent;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::ConnectionState::Play;
use pumpkin_util::permission::{PermissionManager, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use rustyline_async::{Readline, ReadlineEvent};
use std::collections::HashMap;
use std::io::{Cursor, IsTerminal, stdin};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use std::{net::SocketAddr, sync::LazyLock};
use tokio::net::{TcpListener, UdpSocket};
use tokio::select;
use tokio::sync::{Mutex, Notify, RwLock};
use tokio::time::sleep;
use tokio_util::task::TaskTracker;

pub mod block;
pub mod command;
pub mod data;
pub mod entity;
pub mod error;
pub mod item;
pub mod logging;
pub mod net;
pub mod plugin;
pub mod server;
pub mod world;

pub static PLUGIN_MANAGER: LazyLock<Arc<PluginManager>> =
    LazyLock::new(|| Arc::new(PluginManager::new()));

pub static PERMISSION_REGISTRY: LazyLock<Arc<RwLock<PermissionRegistry>>> =
    LazyLock::new(|| Arc::new(RwLock::new(PermissionRegistry::new())));

pub static PERMISSION_MANAGER: LazyLock<Arc<RwLock<PermissionManager>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(PermissionManager::new(
        PERMISSION_REGISTRY.clone(),
    )))
});

pub static LOGGER_IMPL: OnceLock<Option<(ReadlineLogWrapper, LevelFilter)>> = OnceLock::new();

pub fn init_logger(advanced_config: &AdvancedConfiguration) {
    use simplelog::{ConfigBuilder, SharedLogger, SimpleLogger, WriteLogger};

    let logger = if advanced_config.logging.enabled {
        let mut config = ConfigBuilder::new();

        if advanced_config.logging.timestamp {
            config.set_time_format_custom(time::macros::format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second]"
            ));
            config.set_time_level(LevelFilter::Error);
            let _ = config.set_time_offset_to_local();
        } else {
            config.set_time_level(LevelFilter::Off);
        }

        if !advanced_config.logging.color {
            for level in Level::iter() {
                config.set_level_color(level, None);
            }
        } else {
            // We are technically logging to a file-like object.
            config.set_write_log_enable_colors(true);
        }

        if !advanced_config.logging.threads {
            config.set_thread_level(LevelFilter::Off);
        } else {
            config.set_thread_level(LevelFilter::Info);
        }

        let level = std::env::var("RUST_LOG")
            .ok()
            .as_deref()
            .map(LevelFilter::from_str)
            .and_then(Result::ok)
            .unwrap_or(LevelFilter::Info);

        let file_logger: Option<Box<dyn SharedLogger + 'static>> =
            if advanced_config.logging.file.is_empty() {
                None
            } else {
                Some(
                    GzipRollingLogger::new(
                        level,
                        {
                            let mut config = config.clone();
                            for level in Level::iter() {
                                config.set_level_color(level, None);
                            }
                            config.build()
                        },
                        advanced_config.logging.file.clone(),
                    )
                    .expect("Failed to initialize file logger.")
                        as Box<dyn SharedLogger>,
                )
            };

        let (logger, rl): (Box<dyn SharedLogger + 'static>, _) = if advanced_config.commands.use_tty
            && stdin().is_terminal()
        {
            match Readline::new("$ ".to_owned()) {
                Ok((rl, stdout)) => (WriteLogger::new(level, config.build(), stdout), Some(rl)),
                Err(e) => {
                    log::warn!(
                        "Failed to initialize console input ({e}); falling back to simple logger"
                    );
                    (SimpleLogger::new(level, config.build()), None)
                }
            }
        } else {
            (SimpleLogger::new(level, config.build()), None)
        };

        Some((ReadlineLogWrapper::new(logger, file_logger, rl), level))
    } else {
        None
    };

    if LOGGER_IMPL.set(logger).is_err() {
        panic!("Failed to set logger. already initialized");
    }
}

#[macro_export]
macro_rules! init_log {
    () => {
        if let Some((logger_impl, level)) = pumpkin::LOGGER_IMPL.wait() {
            log::set_logger(logger_impl).unwrap();
            log::set_max_level(*level);
        }
    };
}

pub static SHOULD_STOP: AtomicBool = AtomicBool::new(false);
pub static STOP_INTERRUPT: LazyLock<Notify> = LazyLock::new(Notify::new);

pub fn stop_server() {
    SHOULD_STOP.store(true, Ordering::Relaxed);
    STOP_INTERRUPT.notify_waiters();
}

fn resolve_some<T: Future, D, F: FnOnce(D) -> T>(
    opt: Option<D>,
    func: F,
) -> futures::future::Either<T, std::future::Pending<T::Output>> {
    use futures::future::Either;
    match opt {
        Some(val) => Either::Left(func(val)),
        None => Either::Right(std::future::pending()),
    }
}

pub struct PumpkinServer {
    pub server: Arc<Server>,
    pub tcp_listener: Option<TcpListener>,
    pub udp_socket: Option<Arc<UdpSocket>>,
}

impl PumpkinServer {
    pub async fn new(
        basic_config: BasicConfiguration,
        advanced_config: AdvancedConfiguration,
    ) -> Self {
        let server = Server::new(basic_config, advanced_config).await;

        let rcon = server.advanced_config.networking.rcon.clone();

        let mut ticker = Ticker::new();

        if server.advanced_config.commands.use_console
            && let Some((wrapper, _)) = LOGGER_IMPL.wait()
        {
            if let Some(rl) = wrapper.take_readline() {
                setup_console(rl, server.clone());
            } else {
                if server.advanced_config.commands.use_tty {
                    log::warn!(
                        "The input is not a TTY; falling back to simple logger and ignoring `use_tty` setting"
                    );
                }
                setup_stdin_console(server.clone()).await;
            }
        }

        if rcon.enabled {
            log::warn!(
                "RCON is enabled, but it's highly insecure as it transmits passwords and commands in plain text. This makes it vulnerable to interception and exploitation by anyone on the network"
            );
            let rcon_server = server.clone();
            server.spawn_task(async move {
                RCONServer::run(&rcon, rcon_server).await.unwrap();
            });
        }

        let mut tcp_listener = None;

        if server.basic_config.java_edition {
            // Setup the TCP server socket.
            let listener = tokio::net::TcpListener::bind(server.basic_config.java_edition_address)
                .await
                .expect("Failed to start `TcpListener`");
            // In the event the user puts 0 for their port, this will allow us to know what port it is running on
            let addr = listener
                .local_addr()
                .expect("Unable to get the address of the server!");

            if server.advanced_config.networking.query.enabled {
                log::info!("Query protocol is enabled. Starting...");
                server.spawn_task(query::start_query_handler(
                    server.clone(),
                    server.advanced_config.networking.query.address,
                ));
            }

            if server.advanced_config.networking.lan_broadcast.enabled {
                log::info!("LAN broadcast is enabled. Starting...");

                let lan_broadcast = LANBroadcast::new(
                    &server.advanced_config.networking.lan_broadcast,
                    &server.basic_config,
                );
                server.spawn_task(lan_broadcast.start(addr));
            }

            tcp_listener = Some(listener);
        }

        if server.basic_config.allow_chat_reports {
            let mojang_public_keys =
                fetch_mojang_public_keys(&server.advanced_config.networking.authentication)
                    .unwrap();
            *server.mojang_public_keys.lock().await = mojang_public_keys;
        }

        // Ticker
        {
            let ticker_server = server.clone();
            server.spawn_task(async move {
                ticker.run(&ticker_server).await;
            });
        };

        let mut udp_socket = None;

        if server.basic_config.bedrock_edition {
            udp_socket = Some(Arc::new(
                UdpSocket::bind(server.basic_config.bedrock_edition_address)
                    .await
                    .expect("Failed to bind UDP Socket"),
            ));
        }

        Self {
            server: server.clone(),
            tcp_listener,
            udp_socket,
        }
    }

    pub async fn init_plugins(&self) {
        PLUGIN_MANAGER.set_self_ref(PLUGIN_MANAGER.clone()).await;
        PLUGIN_MANAGER.set_server(self.server.clone()).await;
        if let Err(err) = PLUGIN_MANAGER.load_plugins().await {
            log::error!("{err}");
        };
    }

    pub async fn unload_plugins(&self) {
        if let Err(err) = PLUGIN_MANAGER.unload_all_plugins().await {
            log::error!("Error unloading plugins: {err}");
        } else {
            log::info!("All plugins unloaded successfully");
        }
    }

    pub async fn start(&self) {
        let tasks = Arc::new(TaskTracker::new());
        let mut master_client_id: u64 = 0;
        let bedrock_clients = Arc::new(Mutex::new(HashMap::new()));

        while !SHOULD_STOP.load(Ordering::Relaxed) {
            if !self
                .unified_listener_task(&mut master_client_id, &tasks, &bedrock_clients)
                .await
            {
                break;
            }
        }

        log::info!("Stopped accepting incoming connections");

        if let Err(e) = self
            .server
            .player_data_storage
            .save_all_players(&self.server)
            .await
        {
            log::error!("Error saving all players during shutdown: {e}");
        }

        let kick_message = TextComponent::text("Server stopped");
        for player in self.server.get_all_players().await {
            player
                .kick(DisconnectReason::Shutdown, kick_message.clone())
                .await;
        }

        log::info!("Ending player tasks");

        tasks.close();
        tasks.wait().await;

        self.unload_plugins().await;

        log::info!("Starting save.");

        self.server.shutdown().await;

        log::info!("Completed save!");

        // Explicitly drop the line reader to return the terminal to the original state.
        if let Some((wrapper, _)) = LOGGER_IMPL.wait()
            && let Some(rl) = wrapper.take_readline()
        {
            let _ = rl;
        }
    }

    pub async fn unified_listener_task(
        &self,
        master_client_id_counter: &mut u64,
        tasks: &Arc<TaskTracker>,
        bedrock_clients: &Arc<Mutex<HashMap<SocketAddr, Arc<BedrockClient>>>>,
    ) -> bool {
        let mut udp_buf = [0; 1496]; // Buffer for UDP receive

        select! {
            // Branch for TCP connections (Java Edition)
            tcp_result = resolve_some(self.tcp_listener.as_ref(), |listener| listener.accept()) => {
                match tcp_result {
                    Ok((connection, client_addr)) => {
                        if let Err(e) = connection.set_nodelay(true) {
                            log::warn!("Failed to set TCP_NODELAY: {e}");
                        }

                        let client_id = *master_client_id_counter;
                        *master_client_id_counter += 1;

                        let formatted_address = if self.server.basic_config.scrub_ips {
                            scrub_address(&format!("{client_addr}"))
                        } else {
                            format!("{client_addr}")
                        };
                        log::debug!("Accepted connection from Java Edition: {formatted_address} (id {client_id})");

                        let mut java_client = JavaClient::new(connection, client_addr, client_id);
                        java_client.start_outgoing_packet_task();
                        let java_client = Arc::new(java_client);

                        let server_clone = self.server.clone();

                        tasks.spawn(async move {
                            java_client.process_packets(&server_clone).await;
                            java_client.close();
                            java_client.await_tasks().await;

                            let player = java_client.player.lock().await;
                            if let Some(player) = player.as_ref() {
                                log::debug!("Cleaning up player for id {client_id}");

                                if let Err(e) = server_clone.player_data_storage
                                        .handle_player_leave(player)
                                        .await
                                {
                                    log::error!("Failed to save player data on disconnect: {e}");
                                }

                                player.remove().await;
                                server_clone.remove_player(player).await;
                            } else if java_client.connection_state.load() == Play {
                                log::error!("No player found for id {client_id}. This should not happen!");
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to accept Java client connection: {e}");
                        sleep(Duration::from_millis(50)).await;
                    }
                }
            },

            // Branch for UDP packets (Bedrock Edition)
            udp_result = resolve_some(self.udp_socket.as_ref(), |sock: &Arc<UdpSocket>| sock.recv_from(&mut udp_buf)) => {
                match udp_result {
                    Ok((len, client_addr)) => {
                        if len == 0 {
                            log::warn!("Received empty UDP packet from {client_addr}");
                        } else {
                            let id = udp_buf[0];
                            let is_online = id & 128 != 0;

                            if is_online {
                                let be_clients = bedrock_clients.clone();
                                let mut clients_guard = bedrock_clients.lock().await;

                                if let Some(client) = clients_guard.get(&client_addr) {
                                    let client = client.clone();
                                    let reader = Cursor::new(udp_buf[..len].to_vec());
                                    let server = self.server.clone();

                                    tasks.spawn(async move {
                                        client.process_packet(&server, reader).await;
                                    });
                                } else if let Ok(packet) = BedrockClient::is_connection_request(&mut Cursor::new(&udp_buf[4..len])) {
                                    *master_client_id_counter += 1;

                                    let mut platform = BedrockClient::new(self.udp_socket.clone().unwrap(), client_addr, be_clients);
                                    platform.handle_connection_request(packet).await;
                                    platform.start_outgoing_packet_task();

                                    clients_guard.insert(client_addr,
                                    Arc::new(
                                        platform
                                    ));
                                }
                            } else {
                                // Please keep the function as simple as possible!
                                // We dont care about the result, the client just resends the packet
                                // Since offline packets are very small we dont need to move and clone the data
                                let _ = BedrockClient::handle_offline_packet(&self.server, id, &mut Cursor::new(&udp_buf[1..len]), client_addr, self.udp_socket.as_ref().unwrap()).await;
                            }

                        }
                    }
                    // Since all packets go over this match statement, there should be not waiting
                    Err(e) => {
                        log::error!("{e}");
                    }
                }
            },

            // Branch for the global stop signal
            () = STOP_INTERRUPT.notified() => {
                return false;
            }
        }
        true
    }
}

async fn setup_stdin_console(server: Arc<Server>) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let rt = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        while !SHOULD_STOP.load(Ordering::Relaxed) {
            let mut line = String::new();
            if let Ok(size) = stdin().read_line(&mut line) {
                // if no bytes were read, we may have hit EOF
                if size == 0 {
                    break;
                }
            } else {
                break;
            };
            if line.is_empty() || line.as_bytes()[line.len() - 1] != b'\n' {
                log::warn!("Console command was not terminated with a newline");
            }
            rt.block_on(tx.send(line.trim().to_string()))
                .expect("Failed to send command to server");
        }
    });
    tokio::spawn(async move {
        while !SHOULD_STOP.load(Ordering::Relaxed) {
            if let Some(command) = rx.recv().await {
                send_cancellable! {{
                    ServerCommandEvent::new(command.clone());

                    'after: {
                        let dispatcher = &server.command_dispatcher.read().await;
                        dispatcher
                            .handle_command(&command::CommandSender::Console, &server, command.as_str())
                            .await;
                    };
                }}
            }
        }
    });
}

fn setup_console(rl: Readline, server: Arc<Server>) {
    // This needs to be async, or it will hog a thread.
    server.clone().spawn_task(async move {
        let mut rl = rl;
        while !SHOULD_STOP.load(Ordering::Relaxed) {
            let t1 = rl.readline();
            let t2 = STOP_INTERRUPT.notified();

            let result = select! {
                line = t1 => Some(line),
                () = t2 => None,
            };

            let Some(result) = result else { break };

            match result {
                Ok(ReadlineEvent::Line(line)) => {
                    send_cancellable! {{
                        ServerCommandEvent::new(line.clone());

                        'after: {
                            let dispatcher = server.command_dispatcher.read().await;

                            dispatcher
                                .handle_command(&command::CommandSender::Console, &server, &line)
                                .await;
                            rl.add_history_entry(line).unwrap();
                        }
                    }}
                }
                Ok(ReadlineEvent::Interrupted) => {
                    stop_server();
                    break;
                }
                err => {
                    log::error!("Console command loop failed!");
                    log::error!("{err:?}");
                    break;
                }
            }
        }
        if let Some((wrapper, _)) = LOGGER_IMPL.wait() {
            wrapper.return_readline(rl);
        }

        log::debug!("Stopped console commands task");
    });
}

fn scrub_address(ip: &str) -> String {
    ip.chars()
        .map(|ch| if ch == '.' || ch == ':' { ch } else { 'x' })
        .collect()
}
