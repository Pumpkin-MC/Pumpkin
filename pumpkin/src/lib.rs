// Not warn event sending macros
#![allow(unused_labels)]

use crate::data::VanillaData;
use crate::logging::{GzipRollingLogger, PumpkinCommandCompleter, ReadlineLogWrapper};
use crate::net::bedrock::BedrockClient;
use crate::net::java::{JavaClient, PacketHandlerResult};
use crate::net::{ClientPlatform, DisconnectReason};
use crate::net::{lan_broadcast::LANBroadcast, query, rcon::RCONServer};
use crate::server::{Server, ticker::Ticker};
use log::{Level, LevelFilter};
use plugin::server::server_command::ServerCommandEvent;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use pumpkin_macros::send_cancellable;
use pumpkin_util::text::TextComponent;
use rustyline::Editor;
use rustyline::history::FileHistory;
use rustyline::{Config, error::ReadlineError};
use std::collections::HashMap;
use std::io::{Cursor, ErrorKind, IsTerminal, stdin};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use std::{net::SocketAddr, sync::LazyLock};
use tokio::net::{TcpListener, UdpSocket};
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
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

pub type LoggerOption = Option<(ReadlineLogWrapper, LevelFilter)>;
pub static LOGGER_IMPL: LazyLock<Arc<OnceLock<LoggerOption>>> =
    LazyLock::new(|| Arc::new(OnceLock::new()));

pub fn init_logger(advanced_config: &AdvancedConfiguration) {
    use simplelog::{ConfigBuilder, SharedLogger, SimpleLogger, WriteLogger};

    let logger = advanced_config.logging.enabled.then(|| {
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

        if advanced_config.logging.color {
            config.set_write_log_enable_colors(true);
        } else {
            for level in Level::iter() {
                config.set_level_color(level, None);
            }
        }

        if advanced_config.logging.threads {
            config.set_thread_level(LevelFilter::Info);
        } else {
            config.set_thread_level(LevelFilter::Off);
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

        let (logger, rl): (
            Box<dyn SharedLogger + 'static>,
            Option<Editor<PumpkinCommandCompleter, FileHistory>>,
        ) = if advanced_config.commands.use_tty && stdin().is_terminal() {
            let rl_config = Config::builder()
                .auto_add_history(true)
                .completion_type(rustyline::CompletionType::List)
                .edit_mode(rustyline::EditMode::Emacs)
                .build();
            let helper = PumpkinCommandCompleter::new();

            match Editor::with_config(rl_config) {
                Ok(mut rl) => {
                    rl.set_helper(Some(helper));
                    (
                        WriteLogger::new(level, config.build(), std::io::stdout()),
                        Some(rl),
                    )
                }
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
        (ReadlineLogWrapper::new(logger, file_logger, rl), level)
    });

    assert!(
        LOGGER_IMPL.set(logger).is_ok(),
        "Failed to set logger. already initialized"
    );
}

pub static SHOULD_STOP: AtomicBool = AtomicBool::new(false);
pub static STOP_INTERRUPT: LazyLock<CancellationToken> = LazyLock::new(CancellationToken::new);

pub fn stop_server() {
    SHOULD_STOP.store(true, Ordering::Relaxed);
    STOP_INTERRUPT.cancel();
}

fn resolve_some<T: Future, D, F: FnOnce(D) -> T>(
    opt: Option<D>,
    func: F,
) -> futures::future::Either<T, std::future::Pending<T::Output>> {
    use futures::future::Either;
    opt.map_or_else(
        || Either::Right(std::future::pending()),
        |val| Either::Left(func(val)),
    )
}

pub struct PumpkinServer {
    pub server: Arc<Server>,
    pub tcp_listener: Option<TcpListener>,
    pub udp_socket: Option<Arc<UdpSocket>>,
}

impl PumpkinServer {
    #[expect(clippy::if_then_some_else_none)]
    pub async fn new(
        basic_config: BasicConfiguration,
        advanced_config: AdvancedConfiguration,
        vanilla_data: VanillaData,
    ) -> Self {
        let server = Server::new(basic_config, advanced_config, vanilla_data).await;

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
                setup_stdin_console(server.clone());
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

        let tcp_listener = if server.basic_config.java_edition {
            let address = server.basic_config.java_edition_address;
            // Setup the TCP server socket.
            let listener = match TcpListener::bind(address).await {
                Ok(l) => l,
                Err(e) => match e.kind() {
                    ErrorKind::AddrInUse => {
                        log::error!("Error: Address {address} is already in use.");
                        log::error!(
                            "Make sure another instance of the server isn't already running"
                        );
                        std::process::exit(1);
                    }
                    ErrorKind::PermissionDenied => {
                        log::error!("Error: Permission denied when binding to {address}.");
                        log::error!("You might need sudo/admin privileges to use ports below 1024");
                        std::process::exit(1);
                    }
                    ErrorKind::AddrNotAvailable => {
                        log::error!(
                            "Error: The address {address} is not available on this machine"
                        );
                        std::process::exit(1);
                    }
                    _ => {
                        log::error!("Failed to start TcpListener on {address}: {e}");
                        std::process::exit(1);
                    }
                },
            };
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

            Some(listener)
        } else {
            None
        };

        // Ticker
        {
            let ticker_server = server.clone();
            server.spawn_task(async move {
                ticker.run(&ticker_server).await;
            });
        };

        let udp_socket = if server.basic_config.bedrock_edition {
            Some(Arc::new(
                UdpSocket::bind(server.basic_config.bedrock_edition_address)
                    .await
                    .expect("Failed to bind UDP Socket"),
            ))
        } else {
            None
        };

        Self {
            server,
            tcp_listener,
            udp_socket,
        }
    }

    pub async fn init_plugins(&self) {
        self.server
            .plugin_manager
            .set_self_ref(self.server.plugin_manager.clone())
            .await;
        self.server
            .plugin_manager
            .set_server(self.server.clone())
            .await;
        if let Err(err) = self.server.plugin_manager.load_plugins().await {
            log::error!("{err}");
        }
    }

    pub async fn unload_plugins(&self) {
        if let Err(err) = self.server.plugin_manager.unload_all_plugins().await {
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
        for player in self.server.get_all_players() {
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

        if let Some((wrapper, _)) = LOGGER_IMPL.wait()
            && let Some(rl) = wrapper.take_readline()
        {
            let _ = rl;
        }
    }

    #[expect(clippy::too_many_lines)]
    pub async fn unified_listener_task(
        &self,
        master_client_id_counter: &mut u64,
        tasks: &Arc<TaskTracker>,
        bedrock_clients: &Arc<Mutex<HashMap<SocketAddr, Arc<BedrockClient>>>>,
    ) -> bool {
        let mut udp_buf = [0; 1496]; // Buffer for UDP receive

        select! {
            // Branch for TCP connections (Java Edition)
            tcp_result = resolve_some(self.tcp_listener.as_ref(), tokio::net::TcpListener::accept) => {
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
                        let server_clone = self.server.clone();

                        tasks.spawn(async move {
                            let mut java_client = JavaClient::new(connection, client_addr, client_id);
                            java_client.start_outgoing_packet_task();
                            let login_result = java_client.handle_login_sequence(&server_clone).await;

                            match login_result {
                                PacketHandlerResult::Stop => {
                                     java_client.close();
                                     java_client.await_tasks().await;
                                },
                                PacketHandlerResult::ReadyToPlay(profile,config) => {
                                     if let Some((player, world)) = server_clone
                                     .add_player(ClientPlatform::Java(java_client), profile, Some(config))
                                          .await
                                {
                                    world
                                        .spawn_java_player(&server_clone.basic_config, &player, &server_clone)
                                        .await;
                                    if let ClientPlatform::Java(client) = &player.client {
                                        client.progress_player_packets(&player, &server_clone).await;
                                        // Close when done
                                        client.close();
                                        client.await_tasks().await;
                                    }
                                    player.remove().await;
                                    server_clone.remove_player(&player).await;
                                    if let Err(e) = server_clone.player_data_storage
                                        .handle_player_leave(&player)
                                        .await {
                                            log::error!("Failed to save player data on disconnect: {e}");
                                        }
                                    }
                                },
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
            () = STOP_INTERRUPT.cancelled() => {
                return false;
            }
        }
        true
    }
}

fn setup_stdin_console(server: Arc<Server>) {
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
            }
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
                    &server;
                    ServerCommandEvent::new(command.clone());

                    'after: {
                        server.command_dispatcher.read().await
                            .handle_command(&command::CommandSender::Console, &server, command.as_str())
                            .await;
                    };
                }}
            }
        }
    });
}

fn setup_console(mut rl: Editor<PumpkinCommandCompleter, FileHistory>, server: Arc<Server>) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    if let Some(helper) = rl.helper_mut() {
        if let Ok(mut server_lock) = helper.server.write() {
            *server_lock = Some(server.clone());
        }
        let _ = helper.rt.set(tokio::runtime::Handle::current());
    }

    std::thread::spawn(move || {
        while !SHOULD_STOP.load(Ordering::Relaxed) {
            let readline = rl.readline("$ ");
            match readline {
                Ok(line) => {
                    let _ = rl.add_history_entry(line.clone());
                    if tx.blocking_send(line).is_err() {
                        break;
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    log::info!("CTRL-C");
                    stop_server();
                    break;
                }
                Err(ReadlineError::Eof) => {
                    log::info!("CTRL-D");
                    stop_server();
                    break;
                }
                Err(err) => {
                    log::error!("Error reading console input: {err}");
                    break;
                }
            }
        }
        if let Some((wrapper, _)) = LOGGER_IMPL.wait() {
            wrapper.return_readline(rl);
        }
    });

    server.clone().spawn_task(async move {
        while !SHOULD_STOP.load(Ordering::Relaxed) {
            let t1 = rx.recv();
            let t2 = STOP_INTERRUPT.cancelled();

            let result = select! {
                line = t1 => line,
                () = t2 => None,
            };

            if let Some(line) = result {
                send_cancellable! {{
                    &server;
                    ServerCommandEvent::new(line.clone());

                    'after: {
                        server.command_dispatcher.read().await
                            .handle_command(&command::CommandSender::Console, &server, &line)
                            .await;
                    }
                }}
            } else {
                break;
            }
        }
        log::debug!("Stopped console commands task");
    });
}

fn scrub_address(ip: &str) -> String {
    ip.chars()
        .map(|ch| if ch == '.' || ch == ':' { ch } else { 'x' })
        .collect()
}
