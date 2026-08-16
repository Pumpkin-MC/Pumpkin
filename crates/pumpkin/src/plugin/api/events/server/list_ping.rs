use std::{
    fmt,
    net::{SocketAddr, ToSocketAddrs},
};

use pumpkin_macros::Event;
use pumpkin_util::text::TextComponent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerListPingAddress {
    host: String,
    port: u16,
}

impl ServerListPingAddress {
    #[must_use]
    pub const fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    #[must_use]
    pub fn from_socket_addr(address: SocketAddr) -> Self {
        Self {
            host: address.ip().to_string(),
            port: address.port(),
        }
    }

    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    #[must_use]
    pub fn as_socket_addr(&self) -> Option<SocketAddr> {
        (self.host.as_str(), self.port)
            .to_socket_addrs()
            .ok()
            .and_then(|mut addrs| addrs.next())
    }
}

impl fmt::Display for ServerListPingAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.host.contains(':') {
            write!(f, "[{}]:{}", self.host, self.port)
        } else {
            write!(f, "{}:{}", self.host, self.port)
        }
    }
}

/// An event that occurs when the server responds to a status ping.
#[derive(Event, Clone)]
pub struct ServerListPingEvent {
    /// The hostname the client used to ping the server.
    pub(crate) hostname: String,

    /// The address the ping came from.
    pub(crate) address: ServerListPingAddress,

    /// The MOTD shown in the server list.
    pub motd: TextComponent,

    /// The maximum player count.
    pub max_players: u32,

    /// The current online player count.
    pub num_players: u32,

    /// The favicon as a data URI (if any).
    pub favicon: Option<String>,
}

impl ServerListPingEvent {
    /// Creates a new `ServerListPingEvent`.
    #[must_use]
    pub fn new(
        hostname: String,
        address: SocketAddr,
        motd: TextComponent,
        max_players: u32,
        num_players: u32,
        favicon: Option<String>,
    ) -> Self {
        Self {
            hostname,
            address: ServerListPingAddress::from_socket_addr(address),
            motd,
            max_players,
            num_players,
            favicon,
        }
    }

    /// The hostname provided by the client during the status handshake.
    #[must_use]
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    /// The remote socket address of the client requesting the status ping.
    #[must_use]
    pub const fn address(&self) -> &ServerListPingAddress {
        &self.address
    }
}
