use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use packet::{ClientboundPacket, Packet, PacketError, ServerboundPacket};
use pumpkin_config::RCONConfig;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
};
use tracing::{debug, error, info, warn};

use crate::command::CommandSender;
use crate::{SHOULD_STOP, STOP_INTERRUPT, server::Server};

pub use pumpkin_protocol::rcon as packet;

pub struct RCONServer;

impl RCONServer {
    pub async fn run(config: &RCONConfig, server: Arc<Server>) {
        if config.password.trim().is_empty() {
            error!(
                "RCON is enabled but password is empty! Refusing to start RCON server for security."
            );
            return;
        }

        let listener = match tokio::net::TcpListener::bind(config.address).await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind RCON server on {}: {e}", config.address);
                return;
            }
        };

        info!("RCON server is listening on {}", config.address);

        let password = Arc::new(config.password.clone());
        let connections = Arc::new(AtomicU32::new(0));

        while !SHOULD_STOP.load(Ordering::Relaxed) {
            let await_new_client = || async {
                let t1 = listener.accept();
                let t2 = STOP_INTERRUPT.cancelled();

                select! {
                    client = t1 => Some(client),
                    () = t2 => None,
                }
            };
            // Asynchronously wait for an inbound socket.

            let Some(Ok((connection, address))) = await_new_client().await else {
                break;
            };

            let current_conns = connections.load(Ordering::Relaxed);
            if config.max_connections != 0 && current_conns >= config.max_connections {
                warn!(
                    "RCON ({}): Connection rejected, maximum connections ({}) reached",
                    address, config.max_connections
                );
                continue;
            }

            connections.fetch_add(1, Ordering::Relaxed);
            let mut client = RCONClient::new(connection, address);

            let password = password.clone();
            let server = server.clone();
            let connections = connections.clone();
            tokio::spawn(async move {
                while !client.handle(&server, &password).await {}
                connections.fetch_sub(1, Ordering::Relaxed);
                if server.advanced_config.networking.rcon.logging.quit {
                    info!("RCON ({}): Client disconnected", address);
                }
                debug!("closed RCON connection with {}", address);
            });
        }
    }
}

pub struct RCONClient {
    connection: tokio::net::TcpStream,
    address: SocketAddr,
    logged_in: bool,
    incoming: Vec<u8>,
    closed: bool,
}

impl RCONClient {
    #[must_use]
    pub const fn new(connection: tokio::net::TcpStream, address: SocketAddr) -> Self {
        Self {
            connection,
            address,
            logged_in: false,
            incoming: Vec::new(),
            closed: false,
        }
    }

    /// Returns whether the client is closed or not.
    pub async fn handle(&mut self, server: &Arc<Server>, password: &str) -> bool {
        if !self.closed {
            match self.read_bytes().await {
                // The stream is closed, so we can't reply, so we just close everything.
                Ok(true) => return true,
                Ok(false) => {}
                Err(e) => {
                    error!("Could not read packet: {e}");
                    return true;
                }
            }
            while !self.closed {
                match self.receive_packet() {
                    Ok(Some(packet)) => {
                        if let Err(e) = self.process_packet(server, password, packet).await {
                            error!("RCON error: {e}");
                            self.closed = true;
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        error!("RCON packet error: {e}");
                        self.closed = true;
                        break;
                    }
                }
            }
        }
        self.closed
    }

    async fn process_packet(
        &mut self,
        server: &Arc<Server>,
        password: &str,
        packet: Packet,
    ) -> Result<(), PacketError> {
        let config = &server.advanced_config.networking.rcon;
        match packet.get_type() {
            ServerboundPacket::Auth => {
                if !password.is_empty() && packet.get_body() == password {
                    self.send(ClientboundPacket::AuthResponse, packet.get_id(), "")
                        .await?;
                    if config.logging.logged_successfully {
                        info!("RCON ({}): Client logged in successfully", self.address);
                    }
                    self.logged_in = true;
                } else {
                    if config.logging.wrong_password {
                        info!("RCON ({}): Client tried the wrong password", self.address);
                    }
                    self.send(ClientboundPacket::AuthResponse, -1, "").await?;
                    self.closed = true;
                }
            }
            ServerboundPacket::ExecCommand => {
                if self.logged_in {
                    let output = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
                    let packet_body = packet.get_body().to_owned();

                    let command_source = CommandSender::Rcon(output.clone()).into_source(server);

                    server
                        .command_dispatcher
                        .load()
                        .handle_command(&command_source, &packet_body);

                    let output_lines: Vec<String> = output
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .clone();
                    if output_lines.is_empty() {
                        if config.logging.commands {
                            info!(
                                "RCON ({}): Executed command: {}",
                                self.address,
                                packet.get_body()
                            );
                        }
                        self.send(ClientboundPacket::Output, packet.get_id(), "")
                            .await?;
                    } else {
                        for line in &output_lines {
                            if config.logging.commands {
                                info!("RCON ({}): {}", self.address, line);
                            }
                            self.send(ClientboundPacket::Output, packet.get_id(), line)
                                .await?;
                        }
                    }
                } else {
                    if config.logging.wrong_password {
                        info!(
                            "RCON ({}): Unauthenticated client tried to execute command",
                            self.address
                        );
                    }
                    self.send(ClientboundPacket::AuthResponse, -1, "").await?;
                    self.closed = true;
                }
            }
        }
        Ok(())
    }

    async fn read_bytes(&mut self) -> std::io::Result<bool> {
        let mut buf = [0; 1460];
        let n = self.connection.read(&mut buf).await?;
        if n == 0 {
            return Ok(true);
        }
        self.incoming.extend_from_slice(&buf[..n]);
        Ok(false)
    }

    async fn send(
        &mut self,
        packet: ClientboundPacket,
        id: i32,
        body: &str,
    ) -> Result<(), PacketError> {
        let buf = packet.write_buf(id, body);
        self.connection
            .write_all(&buf)
            .await
            .map_err(PacketError::FailedSend)?;
        Ok(())
    }

    fn receive_packet(&mut self) -> Result<Option<Packet>, PacketError> {
        Packet::deserialize(&mut self.incoming)
    }
}
