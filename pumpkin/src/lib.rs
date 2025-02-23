// Not warn event sending macros
#![allow(unused_labels)]

use crate::net::{Client, lan_broadcast, query, rcon::RCONServer};
use crate::server::{Server, ticker::Ticker};
use log::{Level, LevelFilter, Log, logger};
use net::PacketHandlerState;
use plugin::PluginManager;
use pumpkin_config::{ADVANCED_CONFIG, BASIC_CONFIG};
use pumpkin_util::text::TextComponent;
use rustyline_async::{Readline, ReadlineEvent};
use std::str::FromStr;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock},
};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::Notify;
use tokio::task::{JoinHandle, JoinSet};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, tcp::OwnedReadHalf},
    sync::Mutex,
};

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

pub static PLUGIN_MANAGER: LazyLock<Mutex<PluginManager>> =
    LazyLock::new(|| Mutex::new(PluginManager::new()));

// Yucky, is there a way to do this better? revisit our static LOGGER_IMPL?
static _INPUT_HOLDER: OnceLock<Mutex<Option<Readline>>> = OnceLock::new();

pub static LOGGER_IMPL: LazyLock<Option<(Box<dyn Log>, LevelFilter)>> = LazyLock::new(|| {
    if ADVANCED_CONFIG.logging.enabled {
        let mut config = simplelog::ConfigBuilder::new();

        if ADVANCED_CONFIG.logging.timestamp {
            config.set_time_format_custom(time::macros::format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second]"
            ));
            config.set_time_level(LevelFilter::Trace);
        } else {
            config.set_time_level(LevelFilter::Off);
        }

        if !ADVANCED_CONFIG.logging.color {
            for level in Level::iter() {
                config.set_level_color(level, None);
            }
        } else if ADVANCED_CONFIG.commands.use_console {
            // We are technically logging to a file like object
            config.set_write_log_enable_colors(true);
        }

        if !ADVANCED_CONFIG.logging.threads {
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

        if ADVANCED_CONFIG.commands.use_console {
            let (rl, stdout) = Readline::new("$ ".to_owned()).unwrap();
            let logger = simplelog::WriteLogger::new(level, config.build(), stdout);
            let _ = _INPUT_HOLDER.set(Mutex::new(Some(rl)));
            Some((Box::new(logger), level))
        } else {
            let logger = simplelog::SimpleLogger::new(level, config.build());
            Some((Box::new(logger), level))
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
    pub listener: TcpListener,
    pub server_addr: SocketAddr,
    readline_handle: Option<JoinHandle<Readline>>,
    server_tasks: JoinSet<()>,
    player_tasks: JoinSet<()>,
}

impl PumpkinServer {
    pub async fn new() -> Self {
        let server = Arc::new(Server::new());

        let listener = tokio::net::TcpListener::bind(BASIC_CONFIG.server_address)
            .await
            .expect("Failed to start TcpListener");
        let addr = listener.local_addr().expect("Failed to get server address");

        let mut server_tasks = JoinSet::new();
        let player_tasks = JoinSet::new();

        server_tasks.spawn({
            let server = server.clone();
            async move {
                let mut ticker = Ticker::new(BASIC_CONFIG.tps);
                ticker.run(&server).await;
            }
        });

        if ADVANCED_CONFIG.networking.rcon.enabled {
            server_tasks.spawn({
                let server = server.clone();
                let rcon = ADVANCED_CONFIG.networking.rcon.clone();
                async move {
                    RCONServer::new(&rcon, server).await.unwrap();
                }
            });
        }

        if ADVANCED_CONFIG.networking.query.enabled {
            server_tasks.spawn({
                let server = server.clone();
                async move {
                    log::info!("Starting query handler");
                    query::start_query_handler(server, addr).await;
                }
            });
        }

        if ADVANCED_CONFIG.networking.lan_broadcast.enabled {
            server_tasks.spawn(async move {
                log::info!("Starting LAN broadcast");
                lan_broadcast::start_lan_broadcast(addr).await;
            });
        }

        let mut readline = None;

        if let Some(rt) = _INPUT_HOLDER.get() {
            let mut rt = rt.lock().await;
            let rt = rt.take().unwrap();
            let handle = setup_console(rt, server.clone());
            readline = Some(handle);
        }

        Self {
            server,
            listener,
            server_addr: addr,
            readline_handle: readline,
            server_tasks,
            player_tasks,
        }
    }

    pub async fn init_plugins(&self) {
        let mut loader = PLUGIN_MANAGER.lock().await;
        loader.set_server(self.server.clone());
        if let Err(e) = loader.load_plugins().await {
            log::error!("Plugin loading error: {}", e);
        }
    }

    pub async fn start(mut self) {
        let mut master_client_id = 0usize;

        while !SHOULD_STOP.load(std::sync::atomic::Ordering::Relaxed) {
            let (connection, client_addr) = select! {
                conn = self.listener.accept() => match conn {
                    Ok((s, a)) => (s, a),
                    Err(e) => {
                        log::error!("Accept error: {}", e);
                        continue;
                    }
                },
                _ = STOP_INTERRUPT.notified() => break,
            };

            self.handle_connection(connection, client_addr, master_client_id)
                .await;
            master_client_id = master_client_id.wrapping_add(1);
        }

        self.cleanup().await;
    }

    async fn cleanup(mut self) {
        log::info!("Stopping server...");

        // Stopping input
        if let Some(rl) = self.readline_handle.take() {
            let _ = rl.await.unwrap().flush();
        }

        // Kicking players
        let kick_msg = TextComponent::text("Server stopped");
        for player in self.server.get_all_players().await {
            player.kick(kick_msg.clone()).await;
        }

        // Awaiting server tasks
        while let Some(res) = self.server_tasks.join_next().await {
            if let Err(e) = res {
                log::error!("Server task failed: {}", e);
            }
        }

        // Awaiting player tasks
        while let Some(res) = self.player_tasks.join_next().await {
            if let Err(e) = res {
                log::error!("Player task failed: {}", e);
            }
        }

        // Saving
        self.server.save().await;
        logger().flush();
    }

    async fn handle_connection(
        &mut self,
        connection: TcpStream,
        client_addr: SocketAddr,
        id: usize,
    ) {
        if let Err(e) = connection.set_nodelay(true) {
            log::warn!("Failed to set TCP_NODELAY: {}", e);
        }

        let (reader, writer) = connection.into_split();

        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let client = Arc::new(Client::new(tx, client_addr, id));

        // Writer task
        self.player_tasks.spawn({
            let client = client.clone();
            async move {
                let mut writer = writer;
                let mut rx = rx;

                while let Some(state) = rx.recv().await {
                    match state {
                        PacketHandlerState::PacketReady => {
                            let buf = client.enc.lock().await.take();
                            if writer.write_all(&buf).await.is_err() {
                                break;
                            }
                        }
                        PacketHandlerState::Stop => break,
                    }
                }
                client.close().await;
            }
        });

        // Reader task
        self.player_tasks.spawn({
            let server = self.server.clone();
            async move {
                process_client_connection(reader, client, server).await;
            }
        });
    }
}

async fn process_client_connection(
    mut reader: OwnedReadHalf,
    client: Arc<Client>,
    server: Arc<Server>,
) {
    let addr = client.address.lock().await.to_string();
    let id = client.id;

    let formatted_address = if BASIC_CONFIG.scrub_ips {
        scrub_address(&addr)
    } else {
        addr.clone()
    };

    log::info!(
        "Accepted connection from: {} (id {})",
        formatted_address,
        id
    );

    while !client.closed.load(std::sync::atomic::Ordering::Relaxed)
        && !client
            .make_player
            .load(std::sync::atomic::Ordering::Relaxed)
    {
        if !poll(&client, &mut reader).await {
            break;
        }
        client.process_packets(&server).await;
    }

    if client
        .make_player
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        let (player, world) = server.add_player(client).await;
        world
            .spawn_player(&BASIC_CONFIG, player.clone(), &server)
            .await;

        while !player
            .client
            .closed
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            if !poll(&player.client, &mut reader).await {
                break;
            }
            player.process_packets(&server).await;
        }

        log::debug!("Cleaning up player for id {}", id);
        player.remove().await;
        server.remove_player().await;
    }

    log::debug!("Connection {} (ID {}) closed", addr, id);
}

fn setup_console(rl: Readline, server: Arc<Server>) -> JoinHandle<Readline> {
    // This needs to be async or it will hog a thread
    tokio::spawn(async move {
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
                    let dispatcher = server.command_dispatcher.read().await;

                    dispatcher
                        .handle_command(&mut command::CommandSender::Console, &server, &line)
                        .await;
                    rl.add_history_entry(line).unwrap();
                }
                Ok(ReadlineEvent::Interrupted) => {
                    stop_server();
                    break;
                }
                err => {
                    log::error!("Console command loop failed!");
                    log::error!("{:?}", err);
                    break;
                }
            }
        }
        log::debug!("Stopped console commands task");
        let _ = rl.flush();
        rl
    })
}

async fn poll(client: &Client, connection_reader: &mut OwnedReadHalf) -> bool {
    loop {
        if client.closed.load(std::sync::atomic::Ordering::Relaxed) {
            // If we manually close (like a kick) we dont want to keep reading bytes
            return false;
        }

        let mut dec = client.dec.lock().await;

        match dec.decode() {
            Ok(Some(packet)) => {
                client.add_packet(packet).await;
                return true;
            }
            Ok(None) => (), //log::debug!("Waiting for more data to complete packet..."),
            Err(err) => {
                log::warn!("Failed to decode packet for: {}", err.to_string());
                client.close().await;
                return false; // return to avoid reserving additional bytes
            }
        }

        dec.reserve(4096);
        let mut buf = dec.take_capacity();

        let bytes_read = connection_reader.read_buf(&mut buf).await;
        match bytes_read {
            Ok(cnt) => {
                //log::debug!("Read {} bytes", cnt);
                if cnt == 0 {
                    client.close().await;
                    return false;
                }
            }
            Err(error) => {
                log::error!("Error while reading incoming packet {}", error);
                client.close().await;
                return false;
            }
        };

        // This should always be an O(1) unsplit because we reserved space earlier and
        // the call to `read_buf` shouldn't have grown the allocation.
        dec.queue_bytes(buf);
    }
}

fn scrub_address(ip: &str) -> String {
    ip.chars()
        .map(|ch| if ch == '.' || ch == ':' { ch } else { 'x' })
        .collect()
}
