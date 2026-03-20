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
use crate::{
    Context, Result, Server,
    wit::pumpkin::plugin::event::{EventType, PlayerJoinEventData, PlayerLeaveEventData},
};

pub(crate) static NEXT_HANDLER_ID: AtomicU32 = AtomicU32::new(0);
pub(crate) static EVENT_HANDLERS: Mutex<BTreeMap<u32, Box<dyn ErasedEventHandler>>> =
    Mutex::new(BTreeMap::new());

pub trait FromIntoEvent: Sized {
    const EVENT_TYPE: EventType;
    type Data;

    fn data_from_event(event: Event) -> Self::Data;
    fn data_into_event(data: Self::Data) -> Event;
}

pub type EventData<E> = <E as FromIntoEvent>::Data;

pub struct PlayerJoinEvent;
impl FromIntoEvent for PlayerJoinEvent {
    const EVENT_TYPE: EventType = EventType::PlayerJoinEvent;
    type Data = PlayerJoinEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerJoinEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerJoinEvent(data)
    }
}

pub struct PlayerLeaveEvent;
impl FromIntoEvent for PlayerLeaveEvent {
    const EVENT_TYPE: EventType = EventType::PlayerLeaveEvent;
    type Data = PlayerLeaveEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerLeaveEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerLeaveEvent(data)
    }
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub trait EventHandler<E: FromIntoEvent> {
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
