// Not warn event sending macros
#![allow(unused_labels)]

use crate::logging::PumpkinCommandCompleter;
use crate::net::DisconnectReason;
use crate::net::bedrock::BedrockClient;
use crate::net::java::JavaClient;
use crate::net::{lan_broadcast::LANBroadcast, query, rcon::RCONServer};
use crate::server::{Server, ticker::Ticker};
use net::authentication::fetch_mojang_public_keys;
use plugin::PluginManager;
use plugin::server::server_command::ServerCommandEvent;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::ConnectionState::Play;
use pumpkin_util::permission::{PermissionManager, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use rustyline::history::FileHistory;
use rustyline::{Config, error::ReadlineError};
use rustyline::{Editor, ExternalPrinter};
use std::collections::HashMap;
use std::io::{self, Cursor, ErrorKind, IsTerminal, stdin};
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex as StdMutex, OnceLock, mpsc};
use std::time::Duration;
use std::{net::SocketAddr, sync::LazyLock};
use tokio::net::{TcpListener, UdpSocket};
use tokio::select;
use tokio::sync::{Mutex as TokioMutex, RwLock};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{Instrument, info_span, instrument};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer};

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

static CONSOLE_LOG_TX: OnceLock<Sender<String>> = OnceLock::new();

pub type LoggerOption =
    Option<Arc<tokio::sync::Mutex<Option<Editor<PumpkinCommandCompleter, FileHistory>>>>>;
pub static LOGGER_IMPL: LazyLock<Arc<OnceLock<LoggerOption>>> =
    LazyLock::new(|| Arc::new(OnceLock::new()));

enum ConsoleOut {
    Stderr,
    Printer(Box<dyn ExternalPrinter + Send>),
}
static CONSOLE_OUT: OnceLock<Arc<StdMutex<ConsoleOut>>> = OnceLock::new();

struct ConsoleChanWriter {
    tx: Sender<String>,
    buf: Vec<u8>,
}

impl ConsoleChanWriter {
    fn new(tx: Sender<String>) -> Self {
        Self {
            tx,
            buf: Vec::new(),
        }
    }

    fn emit_complete_lines(&mut self) {
        while let Some(nl) = self.buf.iter().position(|&b| b == b'\n') {
            let chunk = self.buf.drain(..=nl).collect::<Vec<u8>>();
            let s = String::from_utf8_lossy(&chunk);
            let line = s.trim_end_matches('\n');
            if !line.is_empty() {
                let _ = self.tx.send(line.to_string());
            }
        }
    }
}

impl io::Write for ConsoleChanWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(bytes);
        self.emit_complete_lines();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buf.is_empty() {
            let buf = mem::take(&mut self.buf);
            let s = String::from_utf8_lossy(&buf);
            let line = s.trim_end_matches('\n');
            if !line.is_empty() {
                let _ = self.tx.send(line.to_string());
            }
        }
        Ok(())
    }
}

#[cfg(feature = "otel")]
pub fn init_opentelemetry() {
    use opentelemetry::{KeyValue, global};
    use opentelemetry_sdk::Resource;

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create OTLP span exporter");

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes(vec![
                    KeyValue::new("service.name", "pumpkin-server"),
                    KeyValue::new(
                        "service.optimizations",
                        if cfg!(debug_assertions) {
                            "debug"
                        } else {
                            "release"
                        },
                    ),
                ])
                .build(),
        )
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(provider);
}

pub fn init_logger(advanced_config: &AdvancedConfiguration) -> Option<WorkerGuard> {
    let filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));

    let (tx, rx) = mpsc::channel::<String>();
    let out = Arc::new(StdMutex::new(ConsoleOut::Stderr));
    let _ = CONSOLE_OUT.set(out.clone());
    let _ = CONSOLE_LOG_TX.set(tx.clone());

    std::thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            let mut guard = out.lock().unwrap();
            match &mut *guard {
                ConsoleOut::Stderr => eprintln!("{msg}"),
                ConsoleOut::Printer(p) => {
                    let _ = p.print(msg);
                }
            }
        }
    });

    let rl_storage: LoggerOption = if advanced_config.commands.use_tty && stdin().is_terminal() {
        let rl_config = Config::builder()
            .auto_add_history(true)
            .completion_type(rustyline::CompletionType::List)
            .edit_mode(rustyline::EditMode::Emacs)
            .build();

        match Editor::with_config(rl_config) {
            Ok(mut editor) => {
                editor.set_helper(Some(PumpkinCommandCompleter::new()));
                Some(Arc::new(TokioMutex::new(Some(editor))))
            }
            Err(e) => {
                eprintln!("Failed to init Readline: {e}");
                None
            }
        }
    } else {
        None
    };

    let make_console_writer = {
        let tx = CONSOLE_LOG_TX
            .get()
            .expect("Failed to get CONSOLE_LOG_TX")
            .clone();
        BoxMakeWriter::new(move || -> Box<dyn io::Write + Send> {
            Box::new(ConsoleChanWriter::new(tx.clone()))
        })
    };

    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(make_console_writer)
        .with_ansi(advanced_config.logging.color)
        .with_thread_ids(advanced_config.logging.threads)
        .with_target(true)
        .with_filter(filter.clone());

    let file_appender =
        tracing_appender::rolling::daily(logging::LOG_DIR, advanced_config.logging.file.clone());
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_thread_ids(advanced_config.logging.threads)
        .with_target(true)
        .with_filter(filter);

    #[cfg(feature = "otel")]
    let registry = {
        use opentelemetry::global::tracer;
        init_opentelemetry();
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer("pumpkin"));
        tracing_subscriber::registry()
            .with(console_layer)
            .with(file_layer)
            .with(otel_layer)
    };

    #[cfg(not(feature = "otel"))]
    let registry = tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer);

    tracing::subscriber::set_global_default(registry).expect("Failed to set tracing subscriber");
    LogTracer::init().expect("Failed to set LogTracer");

    if LOGGER_IMPL.set(rl_storage).is_err() {
        panic!("Failed to set logger. Already initialized");
    }

    Some(file_guard)
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
    #[instrument(name = "PumpkinServer::new")]
    pub async fn new(
        basic_config: BasicConfiguration,
        advanced_config: AdvancedConfiguration,
    ) -> Self {
        let server = Server::new(basic_config, advanced_config).await;

        let rcon = server.advanced_config.networking.rcon.clone();

        let mut ticker = Ticker::new();

        if server.advanced_config.commands.use_console
            && let Some(editor) = LOGGER_IMPL.wait().as_ref()
        {
            let editor = editor.clone();
            if let Some(rl) = editor.lock().await.take() {
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
            let address = server.basic_config.java_edition_address;
            // Setup the TCP server socket.
            let listener = match TcpListener::bind(address).await {
                Ok(l) => l,
                Err(e) => match e.kind() {
                    ErrorKind::AddrInUse => {
                        log::error!("Error: Address {} is already in use.", address);
                        log::error!(
                            "Make sure another instance of the server isn't already running"
                        );
                        std::process::exit(1);
                    }
                    ErrorKind::PermissionDenied => {
                        log::error!("Error: Permission denied when binding to {}.", address);
                        log::error!("You might need sudo/admin privileges to use ports below 1024");
                        std::process::exit(1);
                    }
                    ErrorKind::AddrNotAvailable => {
                        log::error!(
                            "Error: The address {} is not available on this machine",
                            address
                        );
                        std::process::exit(1);
                    }
                    _ => {
                        log::error!("Failed to start TcpListener on {}: {}", address, e);
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
        let bedrock_clients = Arc::new(TokioMutex::new(HashMap::new()));

        while !SHOULD_STOP.load(Ordering::Relaxed) {
            if !self
                .unified_listener_task(&mut master_client_id, &tasks, &bedrock_clients)
                .await
            {
                break;
            }
        }

        tracing::info!("Stopped accepting incoming connections");

        if let Err(e) = self
            .server
            .player_data_storage
            .save_all_players(&self.server)
            .instrument(info_span!("save_all_players"))
            .await
        {
            tracing::error!("Error saving all players during shutdown: {e}");
        }

        let kick_message = TextComponent::text("Server stopped");
        for player in self.server.get_all_players().await {
            player
                .kick(DisconnectReason::Shutdown, kick_message.clone())
                .await;
        }

        tracing::info!("Ending player tasks");

        tasks.close();
        tasks
            .wait()
            .instrument(info_span!("PumpkinServer::tasks.wait()"))
            .await;

        self.unload_plugins().await;

        tracing::info!("Starting save.");

        self.server.shutdown().await;

        tracing::info!("Completed save!");

        if let Some(rl_storage) = LOGGER_IMPL.wait()
            && let Some(editor) = rl_storage.lock().await.take()
        {
            drop(editor)
        }
    }

    #[instrument(skip(self, bedrock_clients))]
    pub async fn unified_listener_task(
        &self,
        master_client_id_counter: &mut u64,
        tasks: &Arc<TaskTracker>,
        bedrock_clients: &Arc<TokioMutex<HashMap<SocketAddr, Arc<BedrockClient>>>>,
    ) -> bool {
        let mut udp_buf = [0; 1496]; // Buffer for UDP receive

        select! {
            // Branch for TCP connections (Java Edition)
            tcp_result = resolve_some(self.tcp_listener.as_ref(), |listener| listener.accept()) => {
                match tcp_result {
                    Ok((connection, client_addr)) => {
                        if let Err(e) = connection.set_nodelay(true) {
                            tracing::warn!("Failed to set TCP_NODELAY: {e}");
                        }

                        let client_id = *master_client_id_counter;
                        *master_client_id_counter += 1;

                        let formatted_address = if self.server.basic_config.scrub_ips {
                            scrub_address(&format!("{client_addr}"))
                        } else {
                            format!("{client_addr}")
                        };
                        let new_connection_span = tracing::info_span!("new_java_connection", protocol = "java", client_id, addr = %formatted_address);

                        tracing::info!("Accepted connection from Java Edition: {formatted_address} (id {client_id})");

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
                                tracing::debug!("Cleaning up player for id {client_id}");

                                if let Err(e) = server_clone.player_data_storage
                                        .handle_player_leave(player)
                                        .await
                                {
                                    tracing::error!("Failed to save player data on disconnect: {e}");
                                }

                                player.remove().await;
                                server_clone.remove_player(player).await;
                            } else if java_client.connection_state.load() == Play {
                                tracing::error!("No player found for id {client_id}. This should not happen!");
                            }
                        }.instrument(new_connection_span));
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept Java client connection: {e}");
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

fn setup_console(mut rl: Editor<PumpkinCommandCompleter, FileHistory>, server: Arc<Server>) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    if let Some(helper) = rl.helper_mut() {
        if let Ok(mut server_lock) = helper.server.write() {
            *server_lock = Some(server.clone());
        }
        let _ = helper.rt.set(tokio::runtime::Handle::current());
    }

    if let Ok(printer) = rl.create_external_printer()
        && let Some(out) = CONSOLE_OUT.get()
    {
        *out.lock().unwrap() = ConsoleOut::Printer(Box::new(printer));
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
        if let Some(rl_storage) = LOGGER_IMPL.wait().as_ref() {
            *rl_storage.blocking_lock() = Some(rl);
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
                    ServerCommandEvent::new(line.clone());

                    'after: {
                        let dispatcher = server.command_dispatcher.read().await;

                        dispatcher
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
