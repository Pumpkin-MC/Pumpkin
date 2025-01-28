use std::{any::Any, collections::HashMap, sync::LazyLock};

use async_trait::async_trait;
use tokio::sync::{Mutex, RwLock};

pub trait Event: Send + Sync {
    fn get_name_static() -> &'static str
    where
        Self: Sized;
    fn get_name(&self) -> &'static str;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_any(&self) -> &dyn Any;
}

pub trait Cancellable: Send + Sync {
    fn is_cancelled(&self) -> bool;
    fn set_cancelled(&mut self, cancelled: bool);
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone)]
// Lowest priority events are executed first, so that higher priority events can override their changes
pub enum EventPriority {
    Highest,
    High,
    Normal,
    Low,
    Lowest,
}

#[async_trait]
pub trait DynEventHandler: Send + Sync {
    async fn handle_dyn(&self, event: &(dyn Event + Send + Sync));
    async fn handle_blocking_dyn(&self, _event: &mut (dyn Event + Send + Sync));
    fn is_blocking(&self) -> bool;
    fn get_priority(&self) -> EventPriority;
}

#[async_trait]
pub trait EventHandler<E: Event>: Send + Sync {
    async fn handle(&self, _event: &E) {
        unimplemented!();
    }
    async fn handle_blocking(&self, _event: &mut E) {
        unimplemented!();
    }
}

struct TypedEventHandler<E, H>
where
    E: Event + Send + Sync + 'static,
    H: EventHandler<E> + Send + Sync,
{
    handler: H,
    priority: EventPriority,
    blocking: bool,
    _phantom: std::marker::PhantomData<E>,
}

#[async_trait]
impl<E, H> DynEventHandler for TypedEventHandler<E, H>
where
    E: Event + Send + Sync + 'static,
    H: EventHandler<E> + Send + Sync,
{
    async fn handle_blocking_dyn(&self, event: &mut (dyn Event + Send + Sync)) {
        // Check if the event is the same type as E. We can not use the type_id because it is
        // different in the plugin and the main program
        if E::get_name_static() == event.get_name() {
            // This is fully safe as long as the event's get_name() and get_name_static()
            // functions are correctly implemented and don't conflict with other events
            let event = unsafe {
                &mut *std::ptr::from_mut::<dyn std::any::Any>(event.as_any_mut()).cast::<E>()
            };
            self.handler.handle_blocking(event).await;
        }
    }

    async fn handle_dyn(&self, event: &(dyn Event + Send + Sync)) {
        // Check if the event is the same type as E. We can not use the type_id because it is
        // different in the plugin and the main program
        if E::get_name_static() == event.get_name() {
            // This is fully safe as long as the event's get_name() and get_name_static()
            // functions are correctly implemented and don't conflict with other events
            let event =
                unsafe { &*std::ptr::from_ref::<dyn std::any::Any>(event.as_any()).cast::<E>() };
            self.handler.handle(event).await;
        }
    }

    fn is_blocking(&self) -> bool {
        self.blocking
    }

    fn get_priority(&self) -> EventPriority {
        self.priority.clone()
    }
}

pub type HandlerMap = HashMap<&'static str, Vec<Box<dyn DynEventHandler>>>;

#[derive(Default)]
pub struct Events {
    handlers: RwLock<HandlerMap>,
}

impl Events {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn fire<E: Event + Send + Sync + 'static>(&self, mut event: E) -> E {
        // Take a snapshot of handlers to avoid lifetime issues
        let handlers = self.handlers.read().await;

        log::debug!("Firing event: {}", E::get_name_static());

        if let Some(handlers_vec) = handlers.get(&E::get_name_static()) {
            log::debug!(
                "Found {} handlers for event: {}",
                handlers_vec.len(),
                E::get_name_static()
            );

            let (blocking_handlers, non_blocking_handlers): (Vec<_>, Vec<_>) = handlers_vec
                .iter()
                .partition(|handler| handler.is_blocking());

            for handler in blocking_handlers {
                handler.handle_blocking_dyn(&mut event).await;
            }

            // TODO: Run non-blocking handlers in parallel
            for handler in non_blocking_handlers {
                handler.handle_dyn(&event).await;
            }
        }

        event
    }

    pub async fn register<E: Event + 'static, H>(
        &self,
        handler: H,
        priority: EventPriority,
        blocking: bool,
    ) where
        H: EventHandler<E> + 'static,
    {
        let mut handlers = self.handlers.write().await;

        let handlers_vec = handlers
            .entry(E::get_name_static())
            .or_insert_with(Vec::new);

        let typed_handler = TypedEventHandler {
            handler,
            priority,
            blocking,
            _phantom: std::marker::PhantomData,
        };

        handlers_vec.push(Box::new(typed_handler));
    }
}

pub static EVENTS: LazyLock<Mutex<Events>> = LazyLock::new(Default::default);
