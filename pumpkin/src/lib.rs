// Not warn event sending macros
#![allow(unused_labels)]

use crate::net::{lan_broadcast, query, rcon::RCONServer, Client};
use crate::server::{ticker::Ticker, Server};
use log::{logger, Level, LevelFilter, Log};
use net::PacketHandlerState;
use plugin::PluginManager;
use pumpkin_config::{ADVANCED_CONFIG, BASIC_CONFIG};
use pumpkin_util::text::TextComponent;
use rustyline_async::{Readline, ReadlineEvent};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::OnceLock;
use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock},
};
use tokio::select;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{tcp::OwnedReadHalf, TcpListener},
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
    tasks_to_await: Vec<JoinHandle<()>>,
}

impl PumpkinServer {
    pub async fn new() -> Self {
        let server = Arc::new(Server::new());

        // Setup the TCP server socket.
        let listener = tokio::net::TcpListener::bind(BASIC_CONFIG.server_address)
            .await
            .expect("Failed to start TcpListener");
        // In the event the user puts 0 for their port, this will allow us to know what port it is running on
        let addr = listener
            .local_addr()
            .expect("Unable to get the address of server!");

        let rcon = ADVANCED_CONFIG.networking.rcon.clone();

        let mut ticker = Ticker::new(BASIC_CONFIG.tps);

        let mut readline = None;
        if let Some(rt) = _INPUT_HOLDER.get() {
            let mut rt = rt.lock().await;
            let rt = rt.take().unwrap();
            let handle = setup_console(rt, server.clone());
            readline = Some(handle);
        }

        if rcon.enabled {
            let server = server.clone();
            tokio::spawn(async move {
                RCONServer::new(&rcon, server).await.unwrap();
            });
        }

        if ADVANCED_CONFIG.networking.query.enabled {
            log::info!("Query protocol enabled. Starting...");
            tokio::spawn(query::start_query_handler(server.clone(), addr));
        }

        if ADVANCED_CONFIG.networking.lan_broadcast.enabled {
            log::info!("LAN broadcast enabled. Starting...");
            tokio::spawn(lan_broadcast::start_lan_broadcast(addr));
        }

        let mut tasks_to_await = Vec::new();
        // Ticker
        {
            let server = server.clone();
            let handle = tokio::spawn(async move {
                ticker.run(&server).await;
            });
            tasks_to_await.push(handle);
        };

        Self {
            server: server.clone(),
            listener,
            server_addr: addr,
            readline_handle: readline,
            tasks_to_await,
        }
    }

    pub async fn init_plugins(&self) {
        let mut loader_lock = PLUGIN_MANAGER.lock().await;
        loader_lock.set_server(self.server.clone());
        if let Err(err) = loader_lock.load_plugins().await {
            log::error!("{}", err.to_string());
        };
    }

    pub async fn start(self) {
        let mut master_client_id: usize = 0;
        let tasks = Arc::new(Mutex::new(HashMap::new()));

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
                log::warn!("failed to set TCP_NODELAY {e}");
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

            let (tx, mut rx) = tokio::sync::mpsc::channel(64);
            let (mut connection_reader, connection_writer) = connection.into_split();

            let client = Arc::new(Client::new(tx, client_addr, id));

            let client_clone = client.clone();
            // This task will be cleaned up on its own
            tokio::spawn(async move {
                let mut connection_writer = connection_writer;

                // We clone ownership of `tx` into here thru the client so this will never drop
                // since there is always a tx in memory. We need to explicitly tell the recv to stop
                while let Some(notif) = rx.recv().await {
                    match notif {
                        PacketHandlerState::PacketReady => {
                            let buf = {
                                let mut enc = client_clone.enc.lock().await;
                                enc.take()
                            };

                            if let Err(e) = connection_writer.write_all(&buf).await {
                                log::warn!("Failed to write packet to client: {e}");
                                client_clone.close().await;
                                break;
                            }
                        }
                        PacketHandlerState::Stop => break,
                    }
                }
            });

            let server = self.server.clone();
            let tasks_clone = tasks.clone();
            // We need to await these to verify all cleanup code is complete
            let handle = tokio::spawn(async move {
                while !client.closed.load(std::sync::atomic::Ordering::Relaxed)
                    && !client
                        .make_player
                        .load(std::sync::atomic::Ordering::Relaxed)
                {
                    let open = poll(&client, &mut connection_reader).await;
                    if open {
                        client.process_packets(&server).await;
                    };
                }
                if client
                    .make_player
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    let (player, world) = server.add_player(client).await;
                    world
                        .spawn_player(&BASIC_CONFIG, player.clone(), &server)
                        .await;

                    // poll Player
                    while !player
                        .client
                        .closed
                        .load(core::sync::atomic::Ordering::Relaxed)
                    {
                        let open = poll(&player.client, &mut connection_reader).await;
                        if open {
                            player.process_packets(&server).await;
                        };
                    }
                    log::debug!("Cleaning up player for id {}", id);
                    player.remove().await;
                    server.remove_player().await;
                    tasks_clone.lock().await.remove(&id);
                }
            });
            tasks.lock().await.insert(id, Some(handle));
        }
        // Keep it in scope for logging
        let rl = match self.readline_handle {
            Some(rl) => Some(rl.await.unwrap()),
            None => None,
        };

        log::info!("Stopped accepting incoming connections");

        let kick_message = TextComponent::text("Server stopped");
        for player in self.server.get_all_players().await {
            player.kick(kick_message.clone()).await;
        }

        log::info!("Ending server tasks");

        for handle in self.tasks_to_await.into_iter() {
            if let Err(err) = handle.await {
                log::error!("Failed to join server task: {}", err.to_string());
            }
        }

        let handles: Vec<Option<JoinHandle<()>>> = tasks
            .lock()
            .await
            .values_mut()
            .map(|val| val.take())
            .collect();

        log::info!("Ending player tasks");

        for handle in handles.into_iter().flatten() {
            if let Err(err) = handle.await {
                log::error!("Failed to join player task: {}", err.to_string());
            }
        }

        self.server.save().await;

        log::info!("Completed save!");
        logger().flush();
        if let Some(mut rl) = rl {
            let _ = rl.flush();
        }
    }
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
