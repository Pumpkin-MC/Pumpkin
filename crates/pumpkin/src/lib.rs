#![deny(clippy::unwrap_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
// Not warn event sending macros
#![allow(unused_labels, deprecated)]

#[macro_use]
extern crate pumpkin_macros;

use crate::crash::CrashReport;
use crate::data::VanillaData;
use crate::logging::{GzipRollingLogger, PumpkinCommandCompleter, ReadlineLogWrapper};
use crate::net::bedrock::{
    BedrockClient,
    nethernet::{NetherNetListener, load_or_create_identity_key},
    status::{IceSocket, StatusResponder},
};
use crate::net::java::JavaClient;
use crate::net::java::pending::PendingConnection;
use crate::net::{ClientPlatform, DisconnectReason, PacketHandlerResult};
use crate::net::{lan_broadcast::LANBroadcast, query, rcon::RCONServer};
use crate::plugin::server::server_command::ServerCommandEvent;
use crate::server::{Server, ticker::Ticker};
use plugin::server::server_load::{LoadType, ServerLoadEvent};
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::{Color, NamedColor};
use rustyline::Editor;
use rustyline::history::FileHistory;
use rustyline::{Config, error::ReadlineError};
use std::collections::HashMap;
use std::io::{ErrorKind, IsTerminal, stdin};
use std::process::exit;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use std::{net::SocketAddr, sync::LazyLock};
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info, warn};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub mod block;
pub mod command;
pub mod crash;
pub mod data;
pub mod entity;
pub mod error;
pub mod item;
pub mod logging;
pub mod net;
pub mod plugin;
pub mod server;
pub mod world;

pub struct LoggingConfig {
    pub color: bool,
    pub threads: bool,
    pub timestamp: bool,
}

pub type LoggerOption = Option<(ReadlineLogWrapper, LevelFilter, LoggingConfig)>;
pub static LOGGER_IMPL: LazyLock<Arc<OnceLock<LoggerOption>>> =
    LazyLock::new(|| Arc::new(OnceLock::new()));

#[expect(clippy::print_stderr)]
pub fn init_logger(advanced_config: &AdvancedConfiguration) {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::fmt;

    let logger = advanced_config.logging.enabled.then(|| {
        let level = std::env::var("RUST_LOG")
            .ok()
            .as_deref()
            .map(LevelFilter::from_str)
            .and_then(Result::ok)
            .unwrap_or(LevelFilter::INFO);

        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            let level_str = match level {
                LevelFilter::OFF => "off",
                LevelFilter::ERROR => "error",
                LevelFilter::WARN => "warn",
                LevelFilter::INFO => "info",
                LevelFilter::DEBUG => "debug",
                LevelFilter::TRACE => "trace",
            };
            EnvFilter::new(level_str)
        });

        let file_logger: Option<GzipRollingLogger> = if advanced_config.logging.file.is_empty() {
            None
        } else {
            match GzipRollingLogger::new(level, advanced_config.logging.file.clone()) {
                Ok(logger) => Some(logger),
                Err(err) => {
                    error!("Failed to initialize file logger: {err}");
                    None
                }
            }
        };

        let (logger, rl): (
            Box<dyn std::io::Write + Send + 'static>,
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
                    (Box::new(std::io::stdout()), Some(rl))
                }
                Err(e) => {
                    eprintln!(
                        "Failed to initialize console input ({e}); falling back to simple logger"
                    );
                    (Box::new(std::io::stdout()), None)
                }
            }
        } else {
            (Box::new(std::io::stdout()), None)
        };

        let fmt_layer = fmt::layer()
            .with_writer(std::sync::Mutex::new(logger))
            .with_ansi(advanced_config.logging.color)
            .with_ansi_sanitization(false)
            .with_target(true)
            .with_thread_names(advanced_config.logging.threads)
            .with_thread_ids(advanced_config.logging.threads);

        if advanced_config.logging.timestamp {
            let local_offset =
                time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
            let fmt_layer = fmt_layer.with_timer(fmt::time::OffsetTime::new(
                local_offset,
                time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
            ));
            let registry = tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer);
            if let Some(file_logger) = file_logger {
                registry.with(file_logger).init();
            } else {
                registry.init();
            }
        } else {
            let fmt_layer = fmt_layer.without_time();
            let registry = tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer);
            if let Some(file_logger) = file_logger {
                registry.with(file_logger).init();
            } else {
                registry.init();
            }
        }

        let logging_config = LoggingConfig {
            color: advanced_config.logging.color,
            threads: advanced_config.logging.threads,
            timestamp: advanced_config.logging.timestamp,
        };

        (ReadlineLogWrapper::new(rl), level, logging_config)
    });

    assert!(
        LOGGER_IMPL.set(logger).is_ok(),
        "Failed to set logger. already initialized"
    );
}

pub static SHOULD_STOP: AtomicBool = AtomicBool::new(false);
pub static STOP_INTERRUPT: LazyLock<CancellationToken> = LazyLock::new(CancellationToken::new);
pub static SERVER_IS_STOPPING: AtomicBool = AtomicBool::new(false);
pub static CRASH_REPORT: OnceLock<CrashReport> = OnceLock::new();
pub static SERVER_EXIT_CODE: AtomicI32 = AtomicI32::new(0);

pub fn stop_server() {
    SHOULD_STOP.store(true, Ordering::Relaxed);
    STOP_INTERRUPT.cancel();
}

pub fn stop_or_exit_server() {
    if SERVER_IS_STOPPING.load(Ordering::Acquire) {
        // Server is already stopping, so we forcefully exit.
        exit(SERVER_EXIT_CODE.load(Ordering::Acquire));
    }
    stop_server();
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
    pub bedrock_status: Option<StatusResponder>,
    pub nethernet_listener: Option<NetherNetListener>,
}

impl PumpkinServer {
    pub fn log_info(&self, message: &str) {
        tracing::info!(target: "plugin", "{}", message);
    }
    pub async fn new(
        basic_config: BasicConfiguration,
        advanced_config: AdvancedConfiguration,
        vanilla_data: VanillaData,
    ) -> Self {
        let server = Server::new(basic_config, advanced_config, vanilla_data).await;

        let rcon = server.advanced_config.networking.rcon.clone();

        if rcon.enabled {
            warn!(
                "RCON is enabled, but it's highly insecure as it transmits passwords and commands in plain text. This makes it vulnerable to interception and exploitation by anyone on the network"
            );
            let rcon_server = server.clone();
            server.spawn_task(async move {
                RCONServer::run(&rcon, rcon_server).await;
            });
        }

        let tcp_listener = if server.advanced_config.networking.java.enabled {
            let address = server.advanced_config.networking.java.address;
            // Setup the TCP server socket.
            let listener = match TcpListener::bind(address).await {
                Ok(l) => l,
                Err(e) => match e.kind() {
                    ErrorKind::AddrInUse => {
                        error!("Error: Address {address} is already in use.");
                        error!("Make sure another instance of the server isn't already running");
                        std::process::exit(1);
                    }
                    ErrorKind::PermissionDenied => {
                        error!("Error: Permission denied when binding to {address}.");
                        error!("You might need sudo/admin privileges to use ports below 1024");
                        std::process::exit(1);
                    }
                    ErrorKind::AddrNotAvailable => {
                        error!("Error: The address {address} is not available on this machine");
                        std::process::exit(1);
                    }
                    _ => {
                        error!("Failed to start TcpListener on {address}: {e}");
                        std::process::exit(1);
                    }
                },
            };
            // In the event the user puts 0 for their port, this will allow us to know what port it is running on
            let addr = listener.local_addr().unwrap_or_else(|_| {
                std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0)
            });

            if server.advanced_config.networking.query.enabled {
                info!("Query protocol is enabled. Starting...");
                server.spawn_task(query::start_query_handler(
                    server.clone(),
                    server.advanced_config.networking.query.address,
                ));
            }

            if server.advanced_config.networking.lan_broadcast.enabled {
                info!("LAN broadcast is enabled. Starting...");

                let lan_broadcast = LANBroadcast::new(
                    &server.advanced_config.networking.lan_broadcast,
                    &server.advanced_config.networking.java.motd,
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
                Ticker::run(&ticker_server).await;
            });
        };

        let (bedrock_status, ice_socket) = Self::bind_bedrock_status(&server).await;
        let nethernet_listener = Self::bind_nethernet(&server, ice_socket).await;

        Self {
            server,
            tcp_listener,
            bedrock_status,
            nethernet_listener,
        }
    }

    async fn bind_nethernet(
        server: &Arc<Server>,
        ice_socket: Option<IceSocket>,
    ) -> Option<NetherNetListener> {
        let config = &server.advanced_config.networking.bedrock;
        if !config.enabled || !config.nethernet.enabled {
            return None;
        }
        let Some(ice_socket) = ice_socket else {
            error!("Bedrock UDP should be bound before NetherNet");
            return None;
        };
        let identity_key = match load_or_create_identity_key(&config.nethernet.identity_key) {
            Ok(key) => key,
            Err(err) => {
                error!("Failed to load or create the Bedrock NetherNet identity key: {err}");
                return None;
            }
        };
        let _ = server.bedrock_private_key.set(identity_key.clone());
        let oidc_verifier = if config.online_mode && config.authentication.enabled {
            let Some((issuer, keys)) = server.bedrock_oidc_keys.get() else {
                error!("Bedrock OIDC keys should be initialized before binding NetherNet");
                return None;
            };
            Some(Arc::new((issuer.clone(), keys.clone())))
        } else {
            None
        };
        match NetherNetListener::bind(
            config.nethernet.address,
            ice_socket,
            config.nethernet.external_ip,
            identity_key,
            config.online_mode,
            oidc_verifier,
            config.nethernet.stun_servers.clone(),
        )
        .await
        {
            Ok(l) => Some(l),
            Err(err) => {
                error!("Failed to bind Bedrock NetherNet signaling endpoint: {err}");
                None
            }
        }
    }

    async fn bind_bedrock_status(server: &Server) -> (Option<StatusResponder>, Option<IceSocket>) {
        let config = &server.advanced_config.networking.bedrock;
        if !config.enabled || !config.nethernet.enabled {
            return (None, None);
        }
        match StatusResponder::bind(config.nethernet.address).await {
            Ok((responder, ice_socket)) => {
                if let Ok((ipv4, ipv6)) = responder.local_addrs() {
                    info!(
                        "Bedrock server-list status is listening on {ipv4} (IPv4) and {ipv6} (IPv6)"
                    );
                }
                (Some(responder), Some(ice_socket))
            }
            Err(err) => {
                error!("Failed to bind Bedrock UDP status/ICE endpoint: {err}");
                (None, None)
            }
        }
    }

    pub async fn init_plugins(&self) -> std::time::Duration {
        if !self.server.advanced_config.plugins.enabled {
            info!("Plugin system is disabled in configuration.");
            return std::time::Duration::ZERO;
        }

        let duration = match self.server.plugin_manager.load_plugins(&self.server).await {
            Ok(duration) => duration,
            Err(err) => {
                error!("{err}");
                std::time::Duration::ZERO
            }
        };

        if self.server.advanced_config.plugins.hot_reload {
            if let Err(err) = self.server.plugin_manager.start_watcher(&self.server).await {
                error!("Failed to start plugin hot-reloading watcher: {err}");
            } else {
                info!("Plugin hot-reloading watcher started from configuration.");
            }
        }

        duration
    }

    pub async fn unload_plugins(&self) {
        if let Err(err) = self.server.plugin_manager.unload_all_plugins().await {
            error!("Error unloading plugins: {err}");
        } else {
            info!("All plugins unloaded successfully");
        }
    }

    pub async fn start(&self) {
        if self.server.advanced_config.commands.use_console
            && let Some((wrapper, _, _)) = LOGGER_IMPL.wait()
        {
            if let Some(rl) = wrapper.take_readline() {
                setup_console(rl, self.server.clone());
            } else {
                if self.server.advanced_config.commands.use_tty {
                    warn!(
                        "The input is not a TTY; falling back to simple logger and ignoring `use_tty` setting"
                    );
                }
                setup_stdin_console(self.server.clone());
            }
        }

        let tasks = Arc::new(TaskTracker::new());
        let mut master_client_id: u64 = 0;
        let bedrock_clients = Arc::new(Mutex::new(HashMap::new()));

        self.server
            .plugin_manager
            .fire(&self.server, &mut ServerLoadEvent::new(LoadType::Startup))
            .await;

        while !SHOULD_STOP.load(Ordering::Relaxed) {
            if !self
                .unified_listener_task(&mut master_client_id, &tasks, &bedrock_clients)
                .await
            {
                break;
            }
        }

        SERVER_IS_STOPPING.store(true, Ordering::Release);

        if let Some(crash_report) = CRASH_REPORT.get() {
            crash_report.print_to_console();
            crash_report.save_and_log();

            info!(
                "{}",
                TextComponent::text("Gracefully shutting down...")
                    .color(Color::Named(NamedColor::Green))
                    .to_pretty_console()
            );

            SERVER_EXIT_CODE.store(1, Ordering::Release);
        }

        info!("Stopped accepting incoming connections");

        if let Err(e) = self
            .server
            .player_data_storage
            .save_all_players(&self.server)
            .await
        {
            error!("Error saving all players during shutdown: {e}");
        }

        if let Err(e) = self
            .server
            .advancement_manager
            .save_all_players(&self.server.get_all_players())
            .await
        {
            error!("Error saving all players advancements during shutdown: {e}");
        }

        let kick_message = TextComponent::text("Server stopped");
        for player in self.server.get_all_players() {
            player
                .kick(DisconnectReason::Shutdown, kick_message.clone())
                .await;
        }

        info!("Ending player tasks");

        tasks.close();
        tasks.wait().await;

        self.unload_plugins().await;

        info!("Starting save.");

        self.server.shutdown().await;

        info!("Completed save!");

        if let Some((wrapper, _, _)) = LOGGER_IMPL.wait()
            && let Some(rl) = wrapper.take_readline()
        {
            let _ = rl;
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn unified_listener_task(
        &self,
        master_client_id_counter: &mut u64,
        tasks: &Arc<TaskTracker>,
        bedrock_clients: &Arc<Mutex<HashMap<SocketAddr, Arc<BedrockClient>>>>,
    ) -> bool {
        select! {
            // Branch for TCP connections (Java Edition)
            tcp_result = resolve_some(self.tcp_listener.as_ref(), tokio::net::TcpListener::accept) => {
                match tcp_result {
                    Ok((connection, client_addr)) => {
                        if let Err(e) = connection.set_nodelay(true) {
                            warn!("Failed to set TCP_NODELAY: {e}");
                        }

                        let client_id = *master_client_id_counter;
                        *master_client_id_counter += 1;

                        let formatted_address = if self.server.basic_config.scrub_ips {
                            scrub_address(&format!("{client_addr}"))
                        } else {
                            format!("{client_addr}")
                        };
                        debug!("Accepted connection from Java Edition: {formatted_address} (id {client_id})");
                        let server_clone = self.server.clone();

                        tasks.spawn(async move {
                            let mut pending = PendingConnection::new(connection, client_addr, client_id);
                            let login_result = pending.handle_login_sequence(&server_clone).await;

                            match login_result {
                                PacketHandlerResult::Stop => {
                                     pending.close();
                                },
                                PacketHandlerResult::ReadyToPlay(profile, config) => {
                                     let mut java_client = JavaClient::from_pending(pending, profile.clone(), config.clone());
                                     java_client.start_outgoing_packet_task();
                                     if let Some((player, world)) = server_clone
                                     .add_player(Arc::new(ClientPlatform::Java(java_client)), profile, Some(config))
                                          .await
                                {
                                    if let ClientPlatform::Java(client) = player.client.as_ref() {
                                        client.set_player(player.clone());
                                    }
                                    world
                                        .spawn_java_player(&server_clone.basic_config, &player, &server_clone)
                                        .await;
                                    if let ClientPlatform::Java(client) = player.client.as_ref() {
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
                                            error!("Failed to save player data on disconnect: {e}");
                                        }
                                    if let Err(e) = server_clone.advancement_manager
                                        .save_player(&player)
                                        .await {
                                            error!("Failed to save player advancement on disconnect: {e}");
                                        }
                                    }
                                },
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept Java client connection: {e}");
                        sleep(Duration::from_millis(50)).await;
                    }
                }
            },

            // Remote server-list status remains a RakNet unconnected ping/pong even
            // when the game connection itself is negotiated over NetherNet.
            status_result = resolve_some(
                self.bedrock_status.as_ref(),
                |status: &StatusResponder| status.receive(&self.server),
            ) => {
                if let Err(error) = status_result {
                    debug!("Bedrock status packet failed: {error}");
                }
            },

            // Branch for Bedrock NetherNet connections negotiated over HTTP/WebRTC.
            nethernet_result = resolve_some(self.nethernet_listener.as_ref(), NetherNetListener::accept) => {
                if let Some((session, client_addr)) = nethernet_result {
                    *master_client_id_counter += 1;
                    let be_clients = bedrock_clients.clone();
                    let client = Arc::new(BedrockClient::new(
                        session.clone(),
                        client_addr,
                        be_clients,
                    ));
                    client.start_outgoing_packet_task();
                    bedrock_clients.lock().await.insert(client_addr, client.clone());

                    let packet_client = client.clone();
                    let packet_server = self.server.clone();
                    tasks.spawn(async move {
                        while let Some(packet) = session.recv().await {
                            packet_client
                                .process_nethernet_packet(&packet_server, packet)
                                .await;
                            if packet_client.is_closed() {
                                break;
                            }
                        }
                        packet_client.close().await;
                    });

                    self.spawn_bedrock_client_task(client, tasks);
                }
            },

            // Branch for the global stop signal
            () = STOP_INTERRUPT.cancelled() => {
                return false;
            }
        }
        true
    }

    fn spawn_bedrock_client_task(&self, client: Arc<BedrockClient>, tasks: &Arc<TaskTracker>) {
        let server = self.server.clone();
        tasks.spawn(async move {
            let login_result = client.handle_login_sequence(&server).await;
            match login_result {
                PacketHandlerResult::Stop => {
                    client.close().await;
                    client.await_tasks().await;
                }
                PacketHandlerResult::ReadyToPlay(profile, config) => {
                    if let Some((player, _world)) = server
                        .add_player(
                            Arc::new(ClientPlatform::Bedrock(client.clone())),
                            profile,
                            Some(config),
                        )
                        .await
                    {
                        client.set_player(player.clone());
                        client.progress_player_packets(&player, &server).await;
                        client.close().await;
                        client.await_tasks().await;
                        player.remove().await;
                        server.remove_player(&player).await;
                        if let Err(error) = server
                            .player_data_storage
                            .handle_player_leave(&player)
                            .await
                        {
                            error!("Failed to save player data on disconnect: {error}");
                        }
                    }
                }
            }
        });
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
                warn!("Console command was not terminated with a newline");
            }
            let _ = rt.block_on(tx.send(line.trim().to_string()));
        }
    });
    tokio::spawn(async move {
        while !SHOULD_STOP.load(Ordering::Relaxed)
            && let Some(command) = rx.recv().await
        {
            let mut event = ServerCommandEvent::new(command.clone());
            server.plugin_manager.fire(&server, &mut event).await;
            if !event.cancelled {
                server
                    .command_dispatcher
                    .read()
                    .await
                    .handle_command(
                        &command::CommandSender::Console.into_source(&server).await,
                        command.as_str(),
                    )
                    .await;
            }
        }
    });
}

fn setup_console(mut rl: Editor<PumpkinCommandCompleter, FileHistory>, server: Arc<Server>) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let (tx_reply, mut rx_reply) = tokio::sync::mpsc::channel(1);

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

                    // Wait for the command to be fully processed before continuing
                    let _ = rx_reply.blocking_recv();
                }
                Err(ReadlineError::Interrupted) => {
                    info!("CTRL-C");
                    stop_or_exit_server();
                    break;
                }
                Err(ReadlineError::Eof) => {
                    info!("CTRL-D");
                    stop_server();
                    break;
                }
                Err(err) => {
                    error!("Error reading console input: {err}");
                    break;
                }
            }
        }
        if let Some((wrapper, _, _)) = LOGGER_IMPL.wait() {
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
                let mut event = ServerCommandEvent::new(line.clone());
                server.plugin_manager.fire(&server, &mut event).await;
                if !event.cancelled {
                    server
                        .command_dispatcher
                        .read()
                        .await
                        .handle_command(
                            &command::CommandSender::Console.into_source(&server).await,
                            &line,
                        )
                        .await;
                    let _ = tx_reply.send(1).await;
                }
            } else {
                break;
            }
        }
        drop(rx);
        debug!("Stopped console commands task");
    });
}

fn scrub_address(ip: &str) -> String {
    ip.chars()
        .map(|ch| if ch == '.' || ch == ':' { ch } else { 'x' })
        .collect()
}
