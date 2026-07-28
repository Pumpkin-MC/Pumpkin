use pumpkin_config::LANBroadcastConfig;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::STOP_INTERRUPT;

/// The standard Minecraft multicast address used for LAN discovery
///
/// Bedrock and Java editions use this specific multicast group to "shout"
/// server presence to clients on the same local network
const BROADCAST_ADDRESS: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(224, 0, 2, 60)), 4445);

pub struct LANBroadcast {
    port: u16,
    motd: String,
}

impl LANBroadcast {
    /// Creates a new LAN broadcast instance from the provided configuration
    #[must_use]
    pub fn new(config: &LANBroadcastConfig, server_motd: &str) -> Self {
        let port = config.port.unwrap_or(0);

        let advanced_motd = config.motd.clone().unwrap_or_default();

        let motd = if advanced_motd.is_empty() {
            warn!(
                "Using the server MOTD as the LAN broadcast MOTD. Note that the LAN broadcast MOTD does not support multiple lines, RGB colors, or gradients so consider defining it accordingly."
            );
            server_motd.replace('\n', " ")
        } else {
            advanced_motd
        };

        Self { port, motd }
    }

    /// Starts the UDP broadcast loop. This should be spawned in a separate task
    ///
    /// The loop sends a packet every 1.5 seconds containing the MOTD and the
    /// port the actual game server is listening on.
    ///
    /// # Arguments
    /// * `bound_addr` - The address where the actual Minecraft server is running
    ///   The port from this address is what clients will use to connect
    ///
    /// # Panics
    /// Panics if the UDP socket cannot be bound or if broadcast permissions are denied
    pub async fn start(self, bound_addr: SocketAddr) {
        let socket = self.bind().await.expect("Unable to bind to address");
        self.start_with_socket(bound_addr, socket, STOP_INTERRUPT.clone())
            .await;
    }

    pub async fn bind(&self) -> std::io::Result<UdpSocket> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", self.port)).await?;
        socket.set_broadcast(true)?;
        Ok(socket)
    }

    pub async fn start_with_socket(
        self,
        bound_addr: SocketAddr,
        socket: UdpSocket,
        stop_token: CancellationToken,
    ) {
        let mut interval = time::interval(Duration::from_millis(1500));

        let advertisement = format!("[MOTD]{}[/MOTD][AD]{}[/AD]", self.motd, bound_addr.port());

        info!(
            "LAN broadcast running on {}",
            socket
                .local_addr()
                .expect("Unable to find running address!")
        );

        while !stop_token.is_cancelled() {
            let t1 = interval.tick();
            let t2 = stop_token.cancelled();

            let should_continue = select! {
                _ = t1 => true,
                () = t2 => false,
            };

            if !should_continue {
                break;
            }

            let _ = socket
                .send_to(advertisement.as_bytes(), BROADCAST_ADDRESS)
                .await;
        }
    }
}
