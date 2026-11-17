//! Event handlers for the async v0.2 world
//!
//! # How an event reaches your code
//!
//! The server fires an event, the host calls the `handle-event(id, server, event)` export, and
//! the dispatcher looks `id` up in the table that [`Context::register_event_handler`] filled; your
//! [`EventHandler`] gets an [`EventRef`] to the live event and reads or edits it through that
//!
//! The event is a borrowed host resource, not a value that moves through the call. It stays
//! valid for the whole callback, awaits included, and the host takes it back when the handler
//! returns
//!
//! # Reading and editing
//!
//! An event kind is the WIT record for that event, such as [`PlayerChatEvent`]
//! [`EventRef::get`] reads the whole record and [`EventRef::set`] writes it back, and a `set`
//! takes effect immediately, so later handlers for the same event see it
//!
//! `blocking` on registration decides whether `set` is allowed: a blocking handler can edit the
//! event, and a notification handler gets an [`EventAccessError`] from `set`, which makes a
//! notification that tries to change the event visible instead of silently ignored
//!
//! # Where to look
//!
//! * [`EventHandler`] is the trait you implement
//! * [`Context::register_event_handler`] registers it and explains `blocking`
//! * [`unregister_event_handler`] drops a handler again

use std::{fmt, marker::PhantomData, sync::Arc};

pub use crate::host::event::{Event, EventPriority, EventType};
use crate::{
    Context, Server,
    registry::{LocalBoxFuture, Registry},
};

static EVENT_HANDLERS: Registry<dyn ErasedEventHandler> = Registry::new();

/// The host refused a read or a write of an event
///
/// Reading fails when the event is a different kind than the one asked for, and writing also
/// fails when the handler was registered as a notification and so may not edit the event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventAccessError;

impl fmt::Display for EventAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the event is a different kind or cannot be edited by this handler")
    }
}

impl std::error::Error for EventAccessError {}

/// A kind of event the host can dispatch, implemented by the WIT record for that event
///
/// Every record in the WIT `event` resource has an implementation, so you name an event by its
/// record: `EventHandler<PlayerChatEvent>`
pub trait EventKind: Sized {
    /// The discriminant the host uses to identify this event when a handler is registered
    const EVENT_TYPE: EventType;

    /// Reads this kind of record from the event
    ///
    /// # Errors
    ///
    /// Returns [`EventAccessError`] when the event is a different kind
    fn get(event: &Event) -> Result<Self, EventAccessError>;

    /// Writes this record into the event, replacing what the host holds
    ///
    /// # Errors
    ///
    /// Returns [`EventAccessError`] when the event is a different kind or the handler is not
    /// allowed to edit it
    fn set(self, event: &Event) -> Result<(), EventAccessError>;
}

/// A handler's view of the event being dispatched
///
/// It borrows the host's event for the length of the callback, so it cannot be kept past the
/// handler
pub struct EventRef<'a, E> {
    event: &'a Event,
    _kind: PhantomData<fn() -> E>,
}

impl<'a, E: EventKind> EventRef<'a, E> {
    fn new(event: &'a Event) -> Self {
        Self {
            event,
            _kind: PhantomData,
        }
    }

    /// Reads the whole event record from the host
    ///
    /// Every call builds the record again, resource handles included, so read it once and keep it
    /// instead of calling this repeatedly
    ///
    /// # Errors
    ///
    /// Returns [`EventAccessError`] when the host dispatched a different kind of event than this
    /// handler was registered for
    pub fn get(&self) -> Result<E, EventAccessError> {
        E::get(self.event)
    }

    /// Replaces the event record on the host with `data`
    ///
    /// The change is visible to later handlers right away. The record is moved, including any
    /// resource handles in it
    ///
    /// # Errors
    ///
    /// Returns [`EventAccessError`] when this handler was registered as a notification
    pub fn set(&self, data: E) -> Result<(), EventAccessError> {
        data.set(self.event)
    }

    /// Reads the record, applies `change` and writes it back
    ///
    /// `change` is synchronous. To await between the read and the write, use [`get`](Self::get)
    /// and [`set`](Self::set)
    ///
    /// # Errors
    ///
    /// Returns [`EventAccessError`] if either the read or the write is refused
    pub fn update(&self, change: impl FnOnce(&mut E)) -> Result<(), EventAccessError> {
        let mut data = self.get()?;
        change(&mut data);
        self.set(data)
    }

    /// The underlying host resource, for calls this wrapper does not cover
    #[must_use]
    pub const fn raw(&self) -> &'a Event {
        self.event
    }
}

/// A handler for one kind of event
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
///     async fn handle(&self, _server: Server, event: EventRef<'_, PlayerChatEvent>) {
///         let Ok(mut chat) = event.get() else { return };
///         if !chat.player.has_permission("chat.send".to_string()).await {
///             chat.cancelled = true;
///             let _ = event.set(chat);
///         }
///     }
/// }
/// ```
#[allow(async_fn_in_trait, reason = "guest futures are intentionally not Send")]
pub trait EventHandler<E: EventKind>: Send + Sync + 'static {
    /// Handles one dispatch of the event
    ///
    /// The server handle is owned and the event is borrowed for the whole call, so the handler can
    /// suspend as often as it needs to. Nothing is returned: edits go through [`EventRef::set`]
    ///
    /// # Reentry
    ///
    /// The host can start another dispatch while this one is suspended, which is why the receiver
    /// is `&self`; shared mutable state needs interior mutability, and no lock guard may be held
    /// across an await
    async fn handle(&self, server: Server, event: EventRef<'_, E>);
}

trait ErasedEventHandler: Send + Sync {
    fn handle_erased<'a>(&'a self, server: Server, event: &'a Event) -> LocalBoxFuture<'a, ()>;
}

struct HandlerWrapper<E, H> {
    handler: H,
    _kind: PhantomData<fn() -> E>,
}

impl<E: EventKind, H: EventHandler<E>> ErasedEventHandler for HandlerWrapper<E, H> {
    fn handle_erased<'a>(&'a self, server: Server, event: &'a Event) -> LocalBoxFuture<'a, ()> {
        Box::pin(self.handler.handle(server, EventRef::new(event)))
    }
}

pub(crate) async fn dispatch(event_id: u32, server: Server, event: &Event) {
    // The registry lock is released before the handler runs
    if let Some(handler) = EVENT_HANDLERS.get(event_id) {
        handler.handle_erased(server, event).await;
    }
}

impl Context {
    /// Registers an event handler and returns its handler id
    ///
    /// The handler is stored in this crate's table under the id, and the host is told the id, the
    /// event type, the priority and `blocking`; registering is a synchronous host call, and only
    /// the handler is async
    ///
    /// The handler lives until [`unregister_event_handler`] is called or the plugin unloads
    ///
    /// `blocking` says whether the handler may edit the event, and it does not say the callback
    /// blocks an operating system thread: a blocking handler's [`EventRef::set`] changes the
    /// event, and a notification handler's `set` fails with [`EventAccessError`]
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
        E: EventKind + 'static,
        H: EventHandler<E>,
    {
        let id = EVENT_HANDLERS.register(Arc::new(HandlerWrapper {
            handler,
            _kind: PhantomData::<fn() -> E>,
        }));
        self.register_event(id, E::EVENT_TYPE, event_priority, blocking);
        id
    }
}

/// Drops the handler registered under `handler_id` and returns whether there was one
///
/// The handler, and everything its struct owns, is released; a dispatch already running keeps its
/// own handle and finishes
///
/// The v0.2 WIT has no `unregister-event`, so the host is not told: it keeps dispatching to the
/// id until the plugin unloads, and each of those calls now does nothing. For a blocking handler
/// that is a round trip with no effect
pub fn unregister_event_handler(handler_id: u32) -> bool {
    EVENT_HANDLERS.take(handler_id).is_some()
}

// Implements `EventKind` for each record and re-exports it from this module
macro_rules! event_kinds {
    (
        by_value { $($vrecord:ident => $vvariant:ident, $vget:ident, $vset:ident;)* }
        by_ref { $($rrecord:ident => $rvariant:ident, $rget:ident, $rset:ident;)* }
    ) => {
        $(
            pub use crate::host::event::$vrecord;

            impl EventKind for $vrecord {
                const EVENT_TYPE: EventType = EventType::$vvariant;

                fn get(event: &Event) -> Result<Self, EventAccessError> {
                    event.$vget().map_err(|()| EventAccessError)
                }

                fn set(self, event: &Event) -> Result<(), EventAccessError> {
                    event.$vset(self).map_err(|()| EventAccessError)
                }
            }
        )*

        $(
            pub use crate::host::event::$rrecord;

            impl EventKind for $rrecord {
                const EVENT_TYPE: EventType = EventType::$rvariant;

                fn get(event: &Event) -> Result<Self, EventAccessError> {
                    event.$rget().map_err(|()| EventAccessError)
                }

                fn set(self, event: &Event) -> Result<(), EventAccessError> {
                    event.$rset(&self).map_err(|()| EventAccessError)
                }
            }
        )*

        #[cfg(test)]
        pub(super) const KIND_GETTERS: &[&str] =
            &[$(stringify!($vget),)* $(stringify!($rget),)*];
    };
}

mod kinds;
pub use kinds::*;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::kinds::KIND_GETTERS;

    #[test]
    fn every_event_in_the_wit_has_a_kind() {
        let wit = include_str!("../../pumpkin-plugin-wit/v0.2/event.wit");
        let resource = wit.split("resource event {").nth(1).unwrap_or_default();
        let in_wit: BTreeSet<String> = resource
            .lines()
            .filter_map(|line| line.trim().strip_prefix("get-"))
            .filter_map(|rest| rest.split(':').next())
            .map(|name| format!("get_{}", name.replace('-', "_")))
            .collect();
        let in_sdk: BTreeSet<String> = KIND_GETTERS.iter().map(|name| (*name).to_owned()).collect();

        assert!(
            !in_wit.is_empty(),
            "found no get- methods in the event resource"
        );
        let missing: Vec<_> = in_wit.difference(&in_sdk).collect();
        assert!(
            missing.is_empty(),
            "events in event.wit with no kind in events/kinds.rs: {missing:?}"
        );
    }
}
