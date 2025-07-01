// Not warn event sending macros
#![allow(unused_labels)]

use crate::net::ClientPlatform;
use crate::net::bedrock::BedrockClientPlatform;
use crate::net::java::JavaClientPlatform;
use crate::net::{Client, lan_broadcast, query, rcon::RCONServer};
use crate::server::{Server, ticker::Ticker};
use bytes::Bytes;
use log::{Level, LevelFilter, Log};
use net::authentication::fetch_mojang_public_keys;
use plugin::PluginManager;
use plugin::server::server_command::ServerCommandEvent;
use pumpkin_config::{BASIC_CONFIG, advanced_config};
use pumpkin_macros::send_cancellable;
use pumpkin_util::permission::{PermissionManager, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use rustyline_async::{Readline, ReadlineEvent};
use std::collections::HashMap;
use std::io::{Cursor, IsTerminal, stdin};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock},
};
use tokio::net::{TcpListener, UdpSocket};
use tokio::select;
use tokio::sync::{Mutex, Notify, RwLock};
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

const GIT_VERSION: &str = env!("GIT_VERSION");

#[cfg(feature = "dhat-heap")]
pub static HEAP_PROFILER: LazyLock<Mutex<Option<dhat::Profiler>>> =
    LazyLock::new(|| Mutex::new(None));

pub static PLUGIN_MANAGER: LazyLock<Arc<RwLock<PluginManager>>> = LazyLock::new(|| {
    let manager = PluginManager::new();
    let arc_manager = Arc::new(RwLock::new(manager));
    let clone = Arc::clone(&arc_manager);
    let arc_manager_clone = arc_manager.clone();
    let mut manager = futures::executor::block_on(arc_manager_clone.write());
    manager.set_self_ref(clone);
    arc_manager
});

pub static PERMISSION_REGISTRY: LazyLock<Arc<RwLock<PermissionRegistry>>> =
    LazyLock::new(|| Arc::new(RwLock::new(PermissionRegistry::new())));

pub static PERMISSION_MANAGER: LazyLock<Arc<RwLock<PermissionManager>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(PermissionManager::new(
        PERMISSION_REGISTRY.clone(),
    )))
});

/// A wrapper for our logger to hold the terminal input while no input is expected in order to
/// properly flush logs to the output while they happen instead of batched
pub struct ReadlineLogWrapper {
    internal: Box<dyn Log>,
    readline: std::sync::Mutex<Option<Readline>>,
}

impl ReadlineLogWrapper {
    fn new(log: impl Log + 'static, rl: Option<Readline>) -> Self {
        Self {
            internal: Box::new(log),
            readline: std::sync::Mutex::new(rl),
        }
    }

    fn take_readline(&self) -> Option<Readline> {
        if let Ok(mut result) = self.readline.lock() {
            result.take()
        } else {
            None
        }
    }

    fn return_readline(&self, rl: Readline) {
        if let Ok(mut result) = self.readline.lock() {
            println!("Returned rl");
            let _ = result.insert(rl);
        }
    }
}

// Writing to `stdout` is expensive anyway, so I don't think having a `Mutex` here is a big deal.
impl Log for ReadlineLogWrapper {
    fn log(&self, record: &log::Record) {
        self.internal.log(record);
        if let Ok(mut lock) = self.readline.lock() {
            if let Some(rl) = lock.as_mut() {
                let _ = rl.flush();
            }
        }
    }

    fn flush(&self) {
        self.internal.flush();
        if let Ok(mut lock) = self.readline.lock() {
            if let Some(rl) = lock.as_mut() {
                let _ = rl.flush();
            }
        }
    }

    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.internal.enabled(metadata)
    }
}

pub static LOGGER_IMPL: LazyLock<Option<(ReadlineLogWrapper, LevelFilter)>> = LazyLock::new(|| {
    if advanced_config().logging.enabled {
        let mut config = simplelog::ConfigBuilder::new();

        if advanced_config().logging.timestamp {
            config.set_time_format_custom(time::macros::format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second]"
            ));
            config.set_time_level(LevelFilter::Trace);
        } else {
            config.set_time_level(LevelFilter::Off);
        }

        if !advanced_config().logging.color {
            for level in Level::iter() {
                config.set_level_color(level, None);
            }
        } else {
            // We are technically logging to a file-like object.
            config.set_write_log_enable_colors(true);
        }

        if !advanced_config().logging.threads {
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

        if advanced_config().commands.use_tty && stdin().is_terminal() {
            match Readline::new("$ ".to_owned()) {
                Ok((rl, stdout)) => {
                    let logger = simplelog::WriteLogger::new(level, config.build(), stdout);
                    Some((ReadlineLogWrapper::new(logger, Some(rl)), level))
                }
                Err(e) => {
                    log::warn!(
                        "Failed to initialize console input ({e}); falling back to simple logger"
                    );
                    let logger = simplelog::SimpleLogger::new(level, config.build());
                    Some((ReadlineLogWrapper::new(logger, None), level))
                }
            }
        } else {
            let logger = simplelog::SimpleLogger::new(level, config.build());
            Some((ReadlineLogWrapper::new(logger, None), level))
        }
    } else {
        None
    }
});

#[macro_export]
macro_rules! init_log {
    () => {
        if let Some((logger_impl, level)) = &*pumpkin::LOGGER_IMPL {
            log::set_logger(logger_impl).unwrap();
            log::set_max_level(*level);
        }
    };
}

pub static SHOULD_STOP: AtomicBool = AtomicBool::new(false);
pub static STOP_INTERRUPT: LazyLock<Notify> = LazyLock::new(Notify::new);

pub fn stop_server() {
    SHOULD_STOP.store(true, std::sync::atomic::Ordering::Relaxed);
    STOP_INTERRUPT.notify_waiters();
}

pub struct PumpkinServer {
    pub server: Arc<Server>,
    pub tcp_listener: TcpListener,
    pub udp_socket: Arc<UdpSocket>,
}

impl PumpkinServer {
    pub async fn new() -> Self {
        let server = Arc::new(Server::new().await);

        for world in &*server.worlds.read().await {
            world.level.read_spawn_chunks(&Server::spawn_chunks()).await;
        }

        let rcon = advanced_config().networking.rcon.clone();

        let mut ticker = Ticker::new();

        if advanced_config().commands.use_console {
            if let Some((wrapper, _)) = &*LOGGER_IMPL {
                if let Some(rl) = wrapper.take_readline() {
                    setup_console(rl, server.clone());
                } else {
                    if advanced_config().commands.use_tty {
                        log::warn!(
                            "The input is not a TTY; falling back to simple logger and ignoring `use_tty` setting"
                        );
                    }
                    setup_stdin_console(server.clone()).await;
                }
            }
        }

        if rcon.enabled {
            let rcon_server = server.clone();
            server.spawn_task(async move {
                RCONServer::run(&rcon, rcon_server).await.unwrap();
            });
        }

        // Setup the TCP server socket.
        let listener = tokio::net::TcpListener::bind(BASIC_CONFIG.java_edition_address)
            .await
            .expect("Failed to start `TcpListener`");
        // In the event the user puts 0 for their port, this will allow us to know what port it is running on
        let addr = listener
            .local_addr()
            .expect("Unable to get the address of the server!");

        if advanced_config().networking.query.enabled {
            log::info!("Query protocol is enabled. Starting...");
            server.spawn_task(query::start_query_handler(
                server.clone(),
                advanced_config().networking.query.address,
            ));
        }

        if advanced_config().networking.lan_broadcast.enabled {
            log::info!("LAN broadcast is enabled. Starting...");
            server.spawn_task(lan_broadcast::start_lan_broadcast(addr));
        }

        if BASIC_CONFIG.allow_chat_reports {
            let mojang_public_keys = fetch_mojang_public_keys(server.auth_client.as_ref().unwrap())
                .await
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

        let udp_socket = UdpSocket::bind(BASIC_CONFIG.bedrock_edition_address)
            .await
            .expect("Failed to bind UDP Socket");

        Self {
            server: server.clone(),
            tcp_listener: listener,
            udp_socket: Arc::new(udp_socket),
        }
    }

    pub async fn init_plugins(&self) {
        let mut loader_lock = PLUGIN_MANAGER.write().await;
        loader_lock.set_server(self.server.clone());
        if let Err(err) = loader_lock.load_plugins().await {
            log::error!("{err}");
        };
    }

    pub async fn unload_plugins(&self) {
        let mut loader_lock = PLUGIN_MANAGER.write().await;
        if let Err(err) = loader_lock.unload_all_plugins().await {
            log::error!("Error unloading plugins: {err}");
        } else {
            log::info!("All plugins unloaded successfully");
        }
    }

    pub async fn start(&self) {
        let tasks = Arc::new(TaskTracker::new());
        let master_client_id: u64 = 0;
        let bedrock_clients = Arc::new(Mutex::new(HashMap::new()));

        while !SHOULD_STOP.load(std::sync::atomic::Ordering::Relaxed) {
            if !self
                .unified_listener_task(master_client_id, &tasks, &bedrock_clients)
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
            player.kick(kick_message.clone()).await;
        }

        log::info!("Ending player tasks");

        tasks.close();
        tasks.wait().await;

        self.unload_plugins().await;

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

    #[expect(unused_assignments)]
    pub async fn unified_listener_task(
        &self,
        mut master_client_id_counter: u64,
        _tasks: &Arc<TaskTracker>,
        bedrock_clients: &Arc<tokio::sync::Mutex<HashMap<SocketAddr, Arc<Client>>>>,
    ) -> bool {
        let mut udp_buf = vec![0; 4096]; // Buffer for UDP receive

        select! {
            // Branch for TCP connections (Java Edition)
            tcp_result = self.tcp_listener.accept() => {
                match tcp_result {
                    Ok((connection, client_addr)) => {
                        if let Err(e) = connection.set_nodelay(true) {
                            log::warn!("Failed to set TCP_NODELAY: {e}");
                        }

                        let client_id = master_client_id_counter;
                        master_client_id_counter += 1;

                        let formatted_address = if BASIC_CONFIG.scrub_ips {
                            scrub_address(&format!("{client_addr}"))
                        } else {
                            format!("{client_addr}")
                        };
                        log::debug!("Accepted connection from Java Edition: {formatted_address} (id {client_id})");

                        // Create a new JavaClientPlatform instance for this specific connection
                        let java_client_platform_instance = JavaClientPlatform::new(connection);

                        let mut client = Client::new(
                            ClientPlatform::Java(java_client_platform_instance),
                            client_addr,
                            client_id,
                        );
                        client.init();

                        let server_clone = self.server.clone();

                        tokio::spawn(async move {
                            // Handles the lifecycle of a single Java client
                            match client.platform.as_ref() {
                                ClientPlatform::Java(java) => {
                                    java.process_packets(&client, &server_clone).await;
                                },
                                ClientPlatform::Bedrock(_) => unreachable!("Java client handler received a Bedrock platform."),
                            };

                            if client.make_player.load(Ordering::Relaxed) {
                                if let Some((player, world)) = server_clone.add_player(client).await { // client needs to be cloned here if moved into add_player
                                    world.spawn_player(&BASIC_CONFIG, player.clone(), &server_clone).await;

                                    player.process_packets(&server_clone).await; // Player's main packet loop
                                    player.close().await; // Signal player to stop its packet processing loop

                                    log::debug!("Cleaning up player for id {client_id}");

                                    if let Err(e) = server_clone.player_data_storage
                                        .handle_player_leave(&player)
                                        .await
                                    {
                                        log::error!("Failed to save player data on disconnect: {e}");
                                    }

                                    player.remove().await;
                                    server_clone.remove_player(&player).await;
                                }
                            } else {
                                client.close();
                                log::debug!("Awaiting tasks for client {}", client.id);
                                client.await_tasks().await;
                                log::debug!("Finished awaiting tasks for client {}", client.id);
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to accept Java client connection: {e}");
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                }
            },

            // Branch for UDP packets (Bedrock Edition)
            udp_result = self.udp_socket.recv_from(&mut udp_buf) => {
                match udp_result {
                    Ok((len, client_addr)) => {
                        if len == 0 {
                            log::warn!("Received empty UDP packet from {client_addr}");
                        }
                        let received_data = Bytes::copy_from_slice(&udp_buf[..len]);


                        let mut clients_guard = bedrock_clients.lock().await;

                        // TODO: don't save clients for offline connections
                        let client = clients_guard.entry(client_addr).or_insert_with(|| {
                            let client_id = master_client_id_counter;
                            master_client_id_counter += 1;
                            log::info!("New Bedrock client detected from: {client_addr} (ID: {client_id})");
                            // Use the prototype to create a new BedrockClientPlatform instance
                          Arc::new(Client::new(ClientPlatform::Bedrock(
                                BedrockClientPlatform::new(self.udp_socket.clone(), client_addr)
                            ), client_addr, client_id))
                        });

                        let server_clone = self.server.clone();

                        let reader = Cursor::new(received_data.to_vec());
                        let client = client.clone();
                        tokio::spawn(async move {
                            if let ClientPlatform::Bedrock(bedrock_plat) = client.platform.as_ref() {
                                bedrock_plat.process_packet(&client, &server_clone, reader).await;
                            }
                            //tasks_clone.track_task_completion(client_clone_for_task.id);
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to receive UDP packet for Bedrock: {e}");
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
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
        while !SHOULD_STOP.load(std::sync::atomic::Ordering::Relaxed) {
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
        while !SHOULD_STOP.load(std::sync::atomic::Ordering::Relaxed) {
            if let Some(command) = rx.recv().await {
                send_cancellable! {{
                    ServerCommandEvent::new(command.clone());

                    'after: {
                        let dispatcher = &server.command_dispatcher.read().await;
                        dispatcher
                            .handle_command(&mut command::CommandSender::Console, &server, command.as_str())
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
        while !SHOULD_STOP.load(std::sync::atomic::Ordering::Relaxed) {
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
                                .handle_command(&mut command::CommandSender::Console, &server, &line)
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
        if let Some((wrapper, _)) = &*LOGGER_IMPL {
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
