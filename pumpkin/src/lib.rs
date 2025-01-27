use crate::net::{lan_broadcast, query, rcon::RCONServer, Client};
use crate::server::{ticker::Ticker, Server};
use plugin::PluginManager;
use pumpkin_config::{ADVANCED_CONFIG, BASIC_CONFIG};
use pumpkin_util::text::TextComponent;
use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{tcp::OwnedReadHalf, TcpListener},
    sync::Mutex,
};

pub mod block;
pub mod command;
pub mod data;
pub mod entity;
pub mod error;
pub mod net;
pub mod plugin;
pub mod server;
pub mod world;

const GIT_VERSION: &str = env!("GIT_VERSION");

pub static PLUGIN_MANAGER: LazyLock<Mutex<PluginManager>> =
    LazyLock::new(|| Mutex::new(PluginManager::new()));

pub struct PumpkinServer {
    pub server: Arc<Server>,
    pub listener: TcpListener,
    pub addr: SocketAddr,
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

        let pumpkin_server = Self {
            server: server.clone(),
            listener,
            addr,
        };

        let use_console = ADVANCED_CONFIG.commands.use_console;
        let rcon = ADVANCED_CONFIG.networking.rcon.clone();

        let mut ticker = Ticker::new(BASIC_CONFIG.tps);

        if use_console {
            setup_console(server.clone());
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

        // Ticker
        {
            let server = server.clone();
            tokio::spawn(async move {
                ticker.run(&server).await;
            })
        };

        pumpkin_server
    }

    pub async fn init_plugins(&self) {
        let mut loader_lock = PLUGIN_MANAGER.lock().await;
        loader_lock.set_server(self.server.clone());
        loader_lock.load_plugins().await.unwrap();
    }

    pub async fn start(&self) {
        let mut master_client_id: u16 = 0;
        loop {
            // Asynchronously wait for an inbound socket.
            let (connection, address) = self.listener.accept().await.unwrap();

            if let Err(e) = connection.set_nodelay(true) {
                log::warn!("failed to set TCP_NODELAY {e}");
            }

            let id = master_client_id;
            master_client_id = master_client_id.wrapping_add(1);

            let formatted_address = if BASIC_CONFIG.scrub_ips {
                scrub_address(&format!("{address}"))
            } else {
                format!("{address}")
            };
            log::info!(
                "Accepted connection from: {} (id {})",
                formatted_address,
                id
            );

            let (tx, mut rx) = tokio::sync::mpsc::channel(64);
            let (connection_reader, connection_writer) = connection.into_split();
            let connection_reader = Arc::new(Mutex::new(connection_reader));
            let connection_writer = Arc::new(Mutex::new(connection_writer));

            let client = Arc::new(Client::new(tx, self.addr, id));

            let client_clone = client.clone();
            tokio::spawn(async move {
                while (rx.recv().await).is_some() {
                    let mut enc = client_clone.enc.lock().await;
                    let buf = enc.take();
                    if let Err(e) = connection_writer.lock().await.write_all(&buf).await {
                        log::warn!("Failed to write packet to client: {e}");
                        client_clone.close();
                        break;
                    }
                }
            });

            let server = self.server.clone();
            tokio::spawn(async move {
                while !client.closed.load(std::sync::atomic::Ordering::Relaxed)
                    && !client
                        .make_player
                        .load(std::sync::atomic::Ordering::Relaxed)
                {
                    let open = poll(&client, connection_reader.clone()).await;
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
                        let open = poll(&player.client, connection_reader.clone()).await;
                        if open {
                            player.process_packets(&server).await;
                        };
                    }
                    log::debug!("Cleaning up player for id {}", id);
                    player.remove().await;
                    server.remove_player().await;
                }
            });
        }
    }
}

fn setup_console(server: Arc<Server>) {
    tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        loop {
            let mut out = String::new();

            reader
                .read_line(&mut out)
                .await
                .expect("Failed to read console line");

            if !out.is_empty() {
                let server_clone = server.clone();
                tokio::spawn(async move {
                    let dispatcher = server_clone.command_dispatcher.read().await;
                    dispatcher
                        .handle_command(&mut command::CommandSender::Console, &server_clone, &out)
                        .await;
                });
            }
        }
    });
}

async fn poll(client: &Client, connection_reader: Arc<Mutex<OwnedReadHalf>>) -> bool {
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
                client.close();
                return false; // return to avoid reserving additional bytes
            }
        }

        dec.reserve(4096);
        let mut buf = dec.take_capacity();

        let bytes_read = connection_reader.lock().await.read_buf(&mut buf).await;
        match bytes_read {
            Ok(cnt) => {
                //log::debug!("Read {} bytes", cnt);
                if cnt == 0 {
                    client.close();
                    return false;
                }
            }
            Err(error) => {
                log::error!("Error while reading incoming packet {}", error);
                client.close();
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
