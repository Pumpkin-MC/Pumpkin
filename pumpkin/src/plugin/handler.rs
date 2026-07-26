use super::{BoxFuture, EventPriority, Payload, PluginManager, TypedEventHandler};
use crate::server::Server;
use futures::future::join_all;
use std::sync::Arc;

/// A trait for handling events dynamically.
///
/// This trait allows for handling events of any type that implements the `Event` trait.
pub trait DynEventHandler: Send + Sync {
    /// Asynchronously handles a dynamic event.
    ///
    /// # Arguments
    /// - `event`: A reference to the event to handle.
    fn handle_dyn<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a (dyn Payload + Send + Sync),
    ) -> BoxFuture<'a, ()>;

    /// Asynchronously handles a blocking dynamic event.
    ///
    /// # Arguments
    /// - `event`: A mutable reference to the event to handle.
    fn handle_blocking_dyn<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        _event: &'a mut (dyn Payload + Send + Sync),
    ) -> BoxFuture<'a, ()>;

    /// Checks if the event handler is blocking.
    ///
    /// # Returns
    /// A boolean indicating whether the handler is blocking.
    fn is_blocking(&self) -> bool;

    /// Retrieves the priority of the event handler.
    ///
    /// # Returns
    /// The priority of the event handler.
    fn get_priority(&self) -> &EventPriority;
}

/// A trait for handling specific events.
///
/// This trait allows for handling events of a specific type that implements the `Event` trait.
pub trait EventHandler<E: Payload>: Send + Sync {
    /// Asynchronously handles an event of type `E`.
    ///
    /// # Arguments
    /// - `event`: A reference to the event to handle.
    fn handle<'a>(&'a self, _server: &'a Arc<Server>, _event: &'a E) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }

    /// Asynchronously handles a blocking event of type `E`.
    ///
    /// # Arguments
    /// - `event`: A mutable reference to the event to handle.
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        _event: &'a mut E,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
}

impl PluginManager {
    /// Register an event handler
    pub async fn register<E, H>(&self, handler: Arc<H>, priority: EventPriority, blocking: bool)
    where
        E: Payload + Send + Sync + 'static,
        H: EventHandler<E> + 'static,
    {
        let mut handlers = self.handlers.write().await;
        let typed_handler = TypedEventHandler {
            handler,
            priority,
            blocking,
            _phantom: std::marker::PhantomData,
        };

        handlers
            .entry(E::get_name_static())
            .or_default()
            .push(Box::new(typed_handler));
    }

    /// Fire an event to all registered handlers
    pub async fn fire<E: Payload + Send + Sync + 'static>(&self, mut event: E) -> E {
        if let Some(server) = self.server.read().await.as_ref() {
            let handlers = self.handlers.read().await;
            if let Some(handlers) = handlers.get(&E::get_name_static()) {
                let (blocking, non_blocking): (Vec<_>, Vec<_>) =
                    handlers.iter().partition(|h| h.is_blocking());

                // Process blocking handlers first
                for handler in blocking {
                    handler.handle_blocking_dyn(server, &mut event).await;
                }

                // Process non-blocking handlers
                join_all(
                    non_blocking
                        .into_iter()
                        .map(|h| h.handle_dyn(server, &event)),
                )
                .await;
            }
        }
        event
    }
}

#[cfg(test)]
mod tests {
    use super::{EventHandler, EventPriority, Payload, PluginManager};
    use std::any::Any;
    use std::sync::Arc;

    struct ProbeEvent {
        touched: bool,
    }

    impl Payload for ProbeEvent {
        fn get_name_static() -> &'static str {
            "ProbeEvent"
        }

        fn get_name(&self) -> &'static str {
            "ProbeEvent"
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    struct ProbeHandler;

    impl EventHandler<ProbeEvent> for ProbeHandler {}

    #[tokio::test]
    async fn fire_without_server_returns_event_unchanged() {
        let manager = PluginManager::new();
        let event = manager.fire(ProbeEvent { touched: false }).await;
        assert!(!event.touched);
    }

    #[tokio::test]
    async fn register_then_fire_without_server_skips_handlers() {
        let manager = PluginManager::new();
        manager
            .register::<ProbeEvent, ProbeHandler>(
                Arc::new(ProbeHandler),
                EventPriority::Normal,
                true,
            )
            .await;
        let event = manager.fire(ProbeEvent { touched: false }).await;
        assert!(!event.touched);
    }
}
