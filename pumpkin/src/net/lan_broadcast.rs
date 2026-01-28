use pumpkin_config::{BasicConfiguration, LANBroadcastConfig};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::{select, time};

use crate::{SHOULD_STOP, STOP_INTERRUPT};

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
    pub fn new(config: &LANBroadcastConfig, basic_config: &BasicConfiguration) -> Self {
        let port = config.port.unwrap_or(0);

        let advanced_motd = config.motd.clone().unwrap_or_default();

        let motd = if advanced_motd.is_empty() {
            log::warn!(
                "Using the server MOTD as the LAN broadcast MOTD. Note that the LAN broadcast MOTD does not support multiple lines, RGB colors, or gradients so consider defining it accordingly."
            );
            basic_config.motd.replace('\n', " ")
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
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", self.port))
            .await
            .expect("Unable to bind to address");

        socket.set_broadcast(true).unwrap();

        let mut interval = time::interval(Duration::from_millis(1500));

        let advertisement = format!("[MOTD]{}[/MOTD][AD]{}[/AD]", &self.motd, bound_addr.port());

        log::info!(
            "LAN broadcast running on {}",
            socket
                .local_addr()
                .expect("Unable to find running address!")
        );

        while !SHOULD_STOP.load(Ordering::Relaxed) {
            let t1 = interval.tick();
            let t2 = STOP_INTERRUPT.cancelled();

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
