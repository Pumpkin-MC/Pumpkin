use std::{
    any::Any,
    fmt,
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use pumpkin_util::{math::vector2::Vector2, text::TextComponent};
use pumpkin_world::level::SyncChunk;

use crate::{net::chunk_sender::PreparedChunk, server::Server, world::World};

pub mod generated;
pub mod packet;

pub use generated::*;
pub use packet::*;

/// A trait representing an event in the system.
///
/// This trait provides methods for retrieving the event's name and for type-safe downcasting.
pub trait Payload: Send + Sync {
    /// Returns the static name of the event type.
    fn get_name_static() -> &'static str
    where
        Self: Sized;

    /// Returns the name of the payload instance.
    fn get_name(&self) -> &'static str;

    /// Provides an immutable reference to the payload as a trait object.
    fn as_any(&self) -> &dyn Any;

    /// Provides a mutable reference to the payload as a trait object.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Helper functions for safe downcasting of Payload implementations.
impl dyn Payload + '_ {
    /// Attempts to downcast an `Arc<dyn Payload>` to `Arc<T>` using name-based type checking.
    pub fn downcast_arc<T: Payload + 'static>(payload: Arc<dyn Payload>) -> Option<Arc<T>> {
        if payload.get_name() == T::get_name_static() {
            // SAFETY: Type name matches `T::get_name_static()`, guaranteeing underlying type is `T`.
            unsafe {
                let raw = Arc::into_raw(payload);
                let typed = raw.cast::<T>();
                Some(Arc::from_raw(typed))
            }
        } else {
            None
        }
    }

    /// Attempts to downcast a `&mut dyn Payload` to `&mut T` using name-based type checking.
    pub fn downcast_mut<T: Payload + 'static>(&mut self) -> Option<&mut T> {
        if self.get_name() == T::get_name_static() {
            // SAFETY: Type name matches `T::get_name_static()`, guaranteeing underlying type is `T`.
            unsafe { Some(&mut *(std::ptr::from_mut::<dyn Payload>(self).cast::<T>())) }
        } else {
            None
        }
    }

    /// Attempts to downcast a `&dyn Payload` to `&T` using name-based type checking.
    pub fn downcast_ref<T: Payload + 'static>(&self) -> Option<&T> {
        if self.get_name() == T::get_name_static() {
            // SAFETY: Type name matches `T::get_name_static()`, guaranteeing underlying type is `T`.
            unsafe { Some(&*(std::ptr::from_ref::<dyn Payload>(self).cast::<T>())) }
        } else {
            None
        }
    }
}

/// A trait for cancellable events.
pub trait Cancellable: Send + Sync {
    /// Checks if the event has been cancelled.
    fn cancelled(&self) -> bool;

    /// Sets the cancellation state of the event.
    fn set_cancelled(&mut self, cancelled: bool);
}

/// An enumeration representing the priority levels of events.
///
/// Events with lower priority values are executed first, allowing higher priority events
/// to override their changes.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub enum EventPriority {
    Highest,
    High,
    Normal,
    Low,
    Lowest,
}

impl InteractAction {
    #[must_use]
    #[inline]
    pub const fn is_left_click(self) -> bool {
        matches!(self, Self::LeftClickAir | Self::LeftClickBlock)
    }

    #[must_use]
    #[inline]
    pub const fn is_right_click(self) -> bool {
        matches!(self, Self::RightClickAir | Self::RightClickBlock)
    }
}

impl CreatureSpawnReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SpawnerEgg => "spawner_egg",
            Self::DispenseEgg => "dispense_egg",
            Self::Bucket => "bucket",
        }
    }
}

impl ServerListPingAddress {
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

impl ServerListPingEvent {
    #[must_use]
    pub fn from_socket_addr(
        hostname: String,
        address: SocketAddr,
        motd: TextComponent,
        max_players: u32,
        num_players: u32,
        favicon: Option<String>,
    ) -> Self {
        Self::new(
            hostname,
            ServerListPingAddress::from_socket_addr(address),
            motd,
            max_players,
            num_players,
            favicon,
        )
    }

    #[must_use]
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    #[must_use]
    pub const fn address(&self) -> &ServerListPingAddress {
        &self.address
    }
}

impl ChunkSendEvent {
    /// Fires the event per chunk and returns the chunks no plugin cancelled.
    pub async fn filter(
        server: &Arc<Server>,
        world: &Arc<World>,
        chunks: &[SyncChunk],
    ) -> Vec<SyncChunk> {
        let mut valid_chunks = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let mut event = Self::new(Arc::clone(world), chunk.x, chunk.z);
            server.plugin_manager.fire(server, &mut event).await;
            if !event.cancelled {
                valid_chunks.push(Arc::clone(chunk));
            }
        }
        valid_chunks
    }

    /// Blocking per-chunk fire for the tick; removes cancelled chunks and returns their positions.
    pub fn retain_blocking(
        server: &Arc<Server>,
        world: &Arc<World>,
        chunks: &mut Vec<PreparedChunk>,
    ) -> Vec<Vector2<i32>> {
        let mut cancelled = Vec::new();
        chunks.retain(|prepared| {
            let mut event = Self::new(Arc::clone(world), prepared.position.x, prepared.position.y);
            server.plugin_manager.fire_blocking(server, &mut event);
            if event.cancelled {
                cancelled.push(prepared.position);
            }
            !event.cancelled
        });
        cancelled
    }
}
