//! Event handlers for the async v0.2 world
//!
//! # How an event reaches your code
//!
//! The server fires an event, the host calls the `handle-event(id, server, event)` export, and
//! the dispatcher looks `id` up in the table that [`Context::register_event_handler`] filled; your
//! [`EventHandler`] gets the event data by value and hands back the data the host should use
//!
//! Nothing is borrowed from the host across that call, so a handler can `.await` as often as it
//! likes and the event data is still its own afterwards
//!
//! # Where to look
//!
//! * [`EventHandler`] is the trait you implement
//! * [`Context::register_event_handler`] registers it and explains `blocking`
//! * [`define_event!`](crate::define_event) adds an event marker that this module does not ship

use std::{marker::PhantomData, sync::Arc};

pub use crate::host::event::{Event, EventPriority, EventType};
use crate::{
    Context, Server,
    registry::{LocalBoxFuture, Registry},
};

static EVENT_HANDLERS: Registry<dyn ErasedEventHandler> = Registry::new();

/// Ties an event marker type to its WIT data record and [`EventType`] discriminant
///
/// Implement it with [`define_event!`](crate::define_event) instead of by hand; a marker is a unit
/// struct that exists so [`EventHandler`] and [`Context::register_event_handler`] can name an
/// event type
pub trait FromIntoEvent: Sized {
    /// The discriminant the host uses to identify this event when a handler is registered
    const EVENT_TYPE: EventType;

    /// The WIT record this event carries
    type Data;

    /// Extracts the event data from an [`Event`]
    ///
    /// # Errors
    ///
    /// Returns the event back, boxed because [`Event`] is over 900 bytes, when it is a different
    /// variant than this marker
    ///
    /// The dispatcher hands that event to the host untouched, so an id that points at the wrong
    /// handler is a host bug the plugin survives instead of a trap
    fn data_from_event(event: Event) -> Result<Self::Data, Box<Event>>;

    /// Wraps event data back into the matching [`Event`] variant
    fn data_into_event(data: Self::Data) -> Event;
}

/// The data type carried by an event
pub type EventData<E> = <E as FromIntoEvent>::Data;

/// A handler for one event type
///
/// Register an implementation with [`Context::register_event_handler`]
///
/// # Examples
///
/// A handler that awaits a permission check before it decides whether the chat goes through:
///
/// ```rust,ignore
/// struct ChatGuard;
///
/// impl EventHandler<PlayerChatEvent> for ChatGuard {
///     async fn handle(
///         &self,
///         _server: Server,
///         mut event: EventData<PlayerChatEvent>,
///     ) -> EventData<PlayerChatEvent> {
///         if !event.player.has_permission("chat.send".to_string()).await {
///             event.cancelled = true;
///         }
///         event
///     }
/// }
/// ```
#[allow(async_fn_in_trait, reason = "guest futures are intentionally not Send")]
pub trait EventHandler<E: FromIntoEvent>: Send + Sync + 'static {
    /// Processes an event and returns the data the host should use afterwards
    ///
    /// The server handle and the event data are owned, so nothing is borrowed from the host across
    /// an await and the handler can suspend as often as it needs to
    ///
    /// The host treats a returned variant that differs from the dispatched one as a handler error
    /// and keeps the original event; the data type pins the variant, so this signature cannot
    /// produce one
    ///
    /// # Reentry
    ///
    /// The host can start another dispatch while this one is suspended, which is why the receiver
    /// is `&self`; shared mutable state needs interior mutability, and no lock guard may be held
    /// across an await
    async fn handle(&self, server: Server, event: E::Data) -> E::Data;
}

trait ErasedEventHandler: Send + Sync {
    fn handle_erased(&self, server: Server, event: Event) -> LocalBoxFuture<'_, Event>;
}

struct HandlerWrapper<E, H> {
    handler: H,
    _event: PhantomData<fn() -> E>,
}

impl<E: FromIntoEvent, H: EventHandler<E>> ErasedEventHandler for HandlerWrapper<E, H> {
    fn handle_erased(&self, server: Server, event: Event) -> LocalBoxFuture<'_, Event> {
        Box::pin(async move {
            match E::data_from_event(event) {
                Ok(data) => E::data_into_event(self.handler.handle(server, data).await),
                Err(event) => *event,
            }
        })
    }
}

pub(crate) async fn dispatch(event_id: u32, server: Server, event: Event) -> Event {
    // The registry lock is released before the handler runs
    let Some(handler) = EVENT_HANDLERS.get(event_id) else {
        return event;
    };
    handler.handle_erased(server, event).await
}

impl Context {
    /// Registers an event handler and returns its handler id
    ///
    /// The handler is stored in this crate's table under the id, and the host is told the id, the
    /// event type, the priority and `blocking`; registering is a synchronous host call, and only
    /// the handler is async
    ///
    /// `blocking` says whether the dispatcher uses the value the handler returns, and it does not
    /// say the callback blocks an operating system thread: the host applies the returned event for
    /// a blocking handler and ignores it for a notification handler, so a notification handler
    /// must not count on its edits being seen
    ///
    /// `event_priority` decides when this handler runs relative to the others for the same event
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// context.register_event_handler::<PlayerChatEvent, _>(
    ///     ChatGuard,
    ///     EventPriority::Normal,
    ///     true,
    /// );
    /// ```
    pub fn register_event_handler<E, H>(
        &self,
        handler: H,
        event_priority: EventPriority,
        blocking: bool,
    ) -> u32
    where
        E: FromIntoEvent + 'static,
        H: EventHandler<E>,
    {
        let id = EVENT_HANDLERS.register(Arc::new(HandlerWrapper {
            handler,
            _event: PhantomData::<fn() -> E>,
        }));
        self.register_event(id, E::EVENT_TYPE, event_priority, blocking);
        id
    }
}

/// Defines an event marker type that implements [`FromIntoEvent`]
///
/// The marker name has to match both the [`Event`] variant and the [`EventType`] case, because the
/// macro uses it for all three
///
/// # Examples
///
/// ```rust,ignore
/// pumpkin_plugin_api_v0_2::define_event! {
///     /// A player finished joining the server
///     PlayerJoinEvent => pumpkin_plugin_api_v0_2::host::event::PlayerJoinEventData
/// }
/// ```
#[macro_export]
macro_rules! define_event {
    ($(#[$meta:meta])* $name:ident => $data:ty) => {
        $(#[$meta])*
        pub struct $name;

        impl $crate::events::FromIntoEvent for $name {
            const EVENT_TYPE: $crate::events::EventType = $crate::events::EventType::$name;
            type Data = $data;

            fn data_from_event(
                event: $crate::events::Event,
            ) -> ::core::result::Result<Self::Data, ::std::boxed::Box<$crate::events::Event>> {
                match event {
                    $crate::events::Event::$name(data) => Ok(data),
                    other => Err(::std::boxed::Box::new(other)),
                }
            }

            fn data_into_event(data: Self::Data) -> $crate::events::Event {
                $crate::events::Event::$name(data)
            }
        }
    };
}

define_event! {
    /// A player finished joining the server
    PlayerJoinEvent => crate::host::event::PlayerJoinEventData
}

define_event! {
    /// A player sent a chat message
    PlayerChatEvent => crate::host::event::PlayerChatEventData
}

define_event! {
    /// A server tick is about to run
    ServerTickStartEvent => crate::host::event::ServerTickStartEventData
}
