use std::{net::SocketAddr, sync::atomic::Ordering};

use packet::{ClientboundPacket, Packet, PacketError, ServerboundPacket};
use pumpkin_config::RCONConfig;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
};

use crate::{SHOULD_STOP, STOP_INTERRUPT, net::rate_limiter::RateLimiter, server::Server};

mod packet;

/// RCON rate limiter configuration
/// 5 failed attempts within 5 minutes (300 seconds) = 15 minute (900 seconds) block
const RCON_MAX_FAILED_ATTEMPTS: u32 = 5;
const RCON_WINDOW_SECS: u64 = 300; // 5 minutes
const RCON_BLOCK_SECS: u64 = 900; // 15 minutes

pub struct RCONServer;

impl RCONServer {
    pub async fn run(config: &RCONConfig, server: Arc<Server>) -> Result<(), std::io::Error> {
        let listener = tokio::net::TcpListener::bind(config.address).await.unwrap();

        let password = Arc::new(config.password.clone());
        
        // Create rate limiter for RCON authentication
        let rate_limiter = Arc::new(RateLimiter::new(
            RCON_MAX_FAILED_ATTEMPTS,
            RCON_WINDOW_SECS,
            RCON_BLOCK_SECS,
        ));

        let mut connections = 0;
        while !SHOULD_STOP.load(Ordering::Relaxed) {
            let await_new_client = || async {
                let t1 = listener.accept();
                let t2 = STOP_INTERRUPT.notified();

                select! {
                    client = t1 => Some(client),
                    () = t2 => None,
                }
            };
            // Asynchronously wait for an inbound socket.

            let Some(result) = await_new_client().await else {
                break;
            };
            let (connection, address) = result?;

            // Check if IP is blocked by rate limiter
            if rate_limiter.is_blocked(&address.ip()).await {
                log::warn!("RCON: Rejected connection from blocked IP {}", address.ip());
                drop(connection);
                continue;
            }

            // Reject new connections when max_connections limit is reached
            if config.max_connections != 0 && connections >= config.max_connections {
                log::warn!("RCON: Rejected connection from {} - max connections reached", address);
                drop(connection);
                continue;
            }

            connections += 1;
            let mut client = RCONClient::new(connection, address);

            let password = password.clone();
            let server = server.clone();
            let rate_limiter = rate_limiter.clone();
            tokio::spawn(async move { while !client.handle(&server, &password, &rate_limiter).await {} });
            log::debug!("closed RCON connection");
            connections -= 1;
        }
        Ok(())
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
    pub async fn handle(&mut self, server: &Arc<Server>, password: &str, rate_limiter: &Arc<RateLimiter>) -> bool {
        if !self.closed {
            match self.read_bytes().await {
                // The stream is closed, so we can't reply, so we just close everything.
                Ok(true) => return true,
                Ok(false) => {}
                Err(e) => {
                    log::error!("Could not read packet: {e}");
                    return true;
                }
            }
            // If we get a close here, we might have a reply, which we still want to write.
            let _ = self.poll(server, password, rate_limiter).await.map_err(|e| {
                log::error!("RCON error: {e}");
                self.closed = true;
            });
        }
        self.closed
    }

    async fn poll(&mut self, server: &Arc<Server>, password: &str, rate_limiter: &Arc<RateLimiter>) -> Result<(), PacketError> {
        let Some(packet) = self.receive_packet().await? else {
            return Ok(());
        };
        let config = &server.advanced_config.networking.rcon;
        match packet.get_type() {
            ServerboundPacket::Auth => {
                // Check if IP is blocked before processing auth
                if rate_limiter.is_blocked(&self.address.ip()).await {
                    log::warn!("RCON ({}): Auth attempt from blocked IP", self.address);
                    self.send(ClientboundPacket::AuthResponse, -1, "").await?;
                    self.closed = true;
                    return Ok(());
                }

                // Use constant-time comparison to prevent timing attacks
                let password_bytes = password.as_bytes();
                let provided_bytes = packet.get_body().as_bytes();
                
                // Constant-time comparison: compare byte by byte without early exit
                let password_matches: bool = if password_bytes.len() == provided_bytes.len() {
                    password_bytes.ct_eq(provided_bytes).into()
                } else {
                    // Still do a comparison to maintain constant time even for different lengths
                    // Compare against password itself to keep timing consistent
                    let _ = password_bytes.ct_eq(password_bytes);
                    false
                };

                if password_matches {
                    self.send(ClientboundPacket::AuthResponse, packet.get_id(), "")
                        .await?;
                    if config.logging.logged_successfully {
                        log::info!("RCON ({}): Client logged in successfully", self.address);
                    }
                    self.logged_in = true;
                } else {
                    if config.logging.wrong_password {
                        log::info!("RCON ({}): Client tried the wrong password", self.address);
                    }
                    
                    // Record failed attempt in rate limiter
                    rate_limiter.record(&self.address.ip()).await;
                    
                    // Check if IP is now blocked after this failed attempt
                    if rate_limiter.is_blocked(&self.address.ip()).await {
                        log::warn!("RCON ({}): IP blocked after too many failed attempts", self.address);
                    }
                    
                    self.send(ClientboundPacket::AuthResponse, -1, "").await?;
                    self.closed = true;
                }
            }
            ServerboundPacket::ExecCommand => {
                if self.logged_in {
                    let output = Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));

                    let server_clone = server.clone();
                    let output_clone = output.clone();
                    let packet_body = packet.get_body().to_owned();
                    tokio::spawn(async move {
                        server_clone
                            .command_dispatcher
                            .read()
                            .await
                            .handle_command(
                                &crate::command::CommandSender::Rcon(output_clone),
                                &server_clone,
                                &packet_body,
                            )
                            .await;
                    });

                    let output = output.lock().await;
                    for line in output.iter() {
                        if config.logging.commands {
                            log::info!("RCON ({}): {}", self.address, line);
                        }
                        self.send(ClientboundPacket::Output, packet.get_id(), line)
                            .await?;
                    }
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
            .write(&buf)
            .await
            .map_err(PacketError::FailedSend)?;
        Ok(())
    }

    async fn receive_packet(&mut self) -> Result<Option<Packet>, PacketError> {
        Packet::deserialize(&mut self.incoming).await
    }
}
