#![allow(clippy::panic)]

use std::{
    collections::BTreeMap,
    marker::PhantomData,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

pub use crate::wit::pumpkin::plugin::event::{
    BedrockClientboundPacket, BedrockServerboundPacket, ClientboundPacket, Event, EventPriority,
    InteractAction, JavaClientboundPacket, JavaServerboundPacket, ServerboundPacket,
};
use crate::{
    Context, Result, Server,
    wit::pumpkin::plugin::event::{EventType, PacketReceivedEventData, PacketSentEventData},
};

/// Block events.
pub mod block;
/// Dialog events.
pub mod dialog;
/// Enchantment events.
pub mod enchantment;
/// Entity events.
pub mod entity;
/// Hanging entity events.
pub mod hanging;
/// Inventory events.
pub mod inventory;
/// Network packet events.
pub mod packet;
/// Player events.
pub mod player;
/// Raid events.
pub mod raid;
/// Server lifecycle events.
pub mod server;
/// Vehicle events.
pub mod vehicle;
/// World events.
pub mod world;

pub use block::*;
pub use dialog::*;
pub use enchantment::*;
pub use entity::*;
pub use hanging::*;
pub use inventory::*;
pub use packet::*;
pub use player::*;
pub use raid::*;
pub use server::*;
pub use vehicle::*;
pub use world::*;

pub(crate) static NEXT_HANDLER_ID: AtomicU32 = AtomicU32::new(0);
pub(crate) static EVENT_HANDLERS: Mutex<BTreeMap<u32, Arc<dyn ErasedEventHandler>>> =
    Mutex::new(BTreeMap::new());

/// The handler registered under `event_id`.
pub(crate) fn event_handler(event_id: u32) -> Option<Arc<dyn ErasedEventHandler>> {
    EVENT_HANDLERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&event_id)
        .cloned()
}

/// Connects an event marker type to its WIT-generated data type and [`EventType`] discriminant.
///
/// Implement this trait for a unit struct to define a new event. The [`EventType`] constant
/// tells the host which event to subscribe to, while `data_from_event` and `data_into_event`
/// provide the conversions between the opaque [`Event`] variant and the concrete data type.
pub trait FromIntoEvent: Sized {
    /// The discriminant used by the host to identify this event.
    const EVENT_TYPE: EventType;

    /// The WIT-generated data record carried by this event.
    type Data;

    /// Extracts the event data from an [`Event`] variant.
    ///
    /// # Panics
    /// Panics if the [`Event`] variant does not match [`Self::EVENT_TYPE`].
    fn data_from_event(event: Event) -> Self::Data;

    /// Wraps event data back into the corresponding [`Event`] variant.
    fn data_into_event(data: Self::Data) -> Event;
}

/// Any event a plugin can register a handler for.
///
/// Every [`FromIntoEvent`] is one. Packet events are delivered through their
/// own exports instead of [`Event`], so they implement this trait directly.
pub trait PluginEvent: Sized {
    /// The discriminant used by the host to identify this event.
    const EVENT_TYPE: EventType;

    /// The WIT-generated data record carried by this event.
    type Data;

    /// Runs `handler` on an event from `handle-event`; other events pass through.
    #[doc(hidden)]
    fn route_event<H: EventHandler<Self>>(_handler: &H, _server: Server, event: Event) -> Event {
        event
    }

    /// Runs `handler` on an event from `handle-packet-received`; other events pass through.
    #[doc(hidden)]
    fn route_packet_received<H: EventHandler<Self>>(
        _handler: &H,
        _server: Server,
        event: PacketReceivedEventData,
    ) -> PacketReceivedEventData {
        event
    }

    /// Runs `handler` on an event from `handle-packet-sent`; other events pass through.
    #[doc(hidden)]
    fn route_packet_sent<H: EventHandler<Self>>(
        _handler: &H,
        _server: Server,
        event: PacketSentEventData,
    ) -> PacketSentEventData {
        event
    }
}

impl<T: FromIntoEvent> PluginEvent for T {
    const EVENT_TYPE: EventType = T::EVENT_TYPE;
    type Data = T::Data;

    fn route_event<H: EventHandler<Self>>(handler: &H, server: Server, event: Event) -> Event {
        T::data_into_event(handler.handle(server, T::data_from_event(event)))
    }
}

/// A convenience alias for the data type associated with an event.
pub type EventData<E> = <E as PluginEvent>::Data;

/// A type alias for a pinned, boxed, dynamically-dispatched future.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A handler for a specific event type.
///
/// Implement this trait to process an event. The `handle` method receives the server
/// handle and the event data, and returns the (potentially modified) event data.
pub trait EventHandler<E: PluginEvent> {
    /// Processes the event and returns the (potentially modified) data.
    fn handle(&self, server: Server, event: E::Data) -> E::Data;
}

pub(crate) trait ErasedEventHandler: Send + Sync {
    fn handle_event(&self, server: Server, event: Event) -> Event;

    fn handle_packet_received(
        &self,
        server: Server,
        event: PacketReceivedEventData,
    ) -> PacketReceivedEventData;

    fn handle_packet_sent(&self, server: Server, event: PacketSentEventData)
    -> PacketSentEventData;
}

struct HandlerWrapper<E: PluginEvent, H> {
    handler: H,
    _phantom: PhantomData<E>,
}

impl<E: PluginEvent + Send + Sync, H: EventHandler<E> + Send + Sync> ErasedEventHandler
    for HandlerWrapper<E, H>
{
    fn handle_event(&self, server: Server, event: Event) -> Event {
        E::route_event(&self.handler, server, event)
    }

    fn handle_packet_received(
        &self,
        server: Server,
        event: PacketReceivedEventData,
    ) -> PacketReceivedEventData {
        E::route_packet_received(&self.handler, server, event)
    }

    fn handle_packet_sent(
        &self,
        server: Server,
        event: PacketSentEventData,
    ) -> PacketSentEventData {
        E::route_packet_sent(&self.handler, server, event)
    }
}

impl Context {
    /// Registers an event handler with the plugin.
    ///
    /// The handler must implement the [`EventHandler`] trait.
    /// If the event is blocking, returning an event from the handler will modify the event.
    pub fn register_event_handler<
        E: PluginEvent + Send + Sync + 'static,
        H: EventHandler<E> + Send + Sync + 'static,
    >(
        &self,
        handler: H,
        event_priority: EventPriority,
        blocking: bool,
    ) -> Result<u32> {
        let id = NEXT_HANDLER_ID.fetch_add(1, Ordering::Relaxed);
        let wrapped = HandlerWrapper {
            handler,
            _phantom: PhantomData::<E>,
        };
        EVENT_HANDLERS
            .lock()
            .map_err(|e| e.to_string())?
            .insert(id, Arc::new(wrapped));

        self.register_event(id, E::EVENT_TYPE, event_priority, blocking);
        Ok(id)
    }
}
