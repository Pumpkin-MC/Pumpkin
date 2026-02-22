use std::{
    collections::BTreeMap,
    marker::PhantomData,
    pin::Pin,
    sync::{
        Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

pub use crate::wit::pumpkin::plugin::event::{Event, EventPriority, PlayerJoinEventData};
use crate::{Context, Result, Server, wit::pumpkin::plugin::event::EventType};

pub(crate) static NEXT_HANDLER_ID: AtomicU32 = AtomicU32::new(0);
pub(crate) static EVENT_HANDLERS: Mutex<BTreeMap<u32, Box<dyn ErasedEventHandler>>> =
    Mutex::new(BTreeMap::new());

pub trait FromIntoEvent: Sized {
    const EVENT_TYPE: EventType;

    fn from_event(event: Event) -> Self;
    fn into_event(self) -> Event;
}

impl FromIntoEvent for PlayerJoinEventData {
    const EVENT_TYPE: EventType = EventType::PlayerJoinEvent;

    fn from_event(event: Event) -> Self {
        match event {
            Event::PlayerJoinEvent(data) => data,
            // _ => None,
        }
    }

    fn into_event(self) -> Event {
        Event::PlayerJoinEvent(self)
    }
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub trait EventHandler<E> {
    fn handle<'a>(&'a self, server: Server, event: E) -> E;
}

pub(crate) trait ErasedEventHandler: Send + Sync {
    fn handle_erased<'a>(&'a self, server: Server, event: Event) -> Event;
}

struct HandlerWrapper<E, H> {
    handler: H,
    _phantom: PhantomData<E>,
}

impl<E: FromIntoEvent + Send + Sync, H: EventHandler<E> + Send + Sync> ErasedEventHandler
    for HandlerWrapper<E, H>
{
    fn handle_erased(&self, server: Server, event: Event) -> Event {
        let specific_event = E::from_event(event);
        self.handler.handle(server, specific_event).into_event()
    }
}

impl Context {
    /// Registers an event handler with the plugin.
    ///
    /// The handler must implement the [`EventHandler`] trait.
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
