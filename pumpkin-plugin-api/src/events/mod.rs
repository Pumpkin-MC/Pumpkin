use std::{
    collections::BTreeMap,
    marker::PhantomData,
    pin::Pin,
    sync::{
        Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

pub use crate::wit::pumpkin::plugin::event::{Event, EventPriority};
use crate::{Context, Result, Server, wit::pumpkin::plugin::event::EventType};

pub mod block;
pub mod player;
pub mod server;

pub use block::*;
pub use player::*;
pub use server::*;

pub(crate) static NEXT_HANDLER_ID: AtomicU32 = AtomicU32::new(0);
pub(crate) static EVENT_HANDLERS: Mutex<BTreeMap<u32, Box<dyn ErasedEventHandler>>> =
    Mutex::new(BTreeMap::new());

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

/// A convenience alias for the data type associated with an event.
pub type EventData<E> = <E as FromIntoEvent>::Data;

/// A type alias for a pinned, boxed, dynamically-dispatched future.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A handler for a specific event type.
///
/// Implement this trait to process an event. The `handle` method receives the server
/// handle and the event data, and returns the (potentially modified) event data.
pub trait EventHandler<E: FromIntoEvent> {
    /// Processes the event and returns the (potentially modified) data.
    fn handle(&self, server: Server, event: E::Data) -> E::Data;
}

pub(crate) trait ErasedEventHandler: Send + Sync {
    fn handle_erased(&self, server: Server, event: Event) -> Event;
}

struct HandlerWrapper<E: FromIntoEvent, H> {
    handler: H,
    _phantom: PhantomData<E>,
}

impl<E: FromIntoEvent + Send + Sync, H: EventHandler<E> + Send + Sync> ErasedEventHandler
    for HandlerWrapper<E, H>
{
    fn handle_erased(&self, server: Server, event: Event) -> Event {
        let data = E::data_from_event(event);
        let result = self.handler.handle(server, data);
        E::data_into_event(result)
    }
}

impl Context {
    /// Registers an event handler with the plugin.
    ///
    /// The handler must implement the [`EventHandler`] trait.
    /// If the event is blocking, returning an event from the handler will modify the event.
    pub fn register_event_handler<
        E: FromIntoEvent + Send + Sync + 'static,
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
            .insert(id, Box::new(wrapped));

        self.register_event(id, E::EVENT_TYPE, event_priority, blocking);
        Ok(id)
    }
}
