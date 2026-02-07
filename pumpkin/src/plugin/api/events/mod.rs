use std::any::Any;
use std::sync::Arc;

pub mod block;
pub mod entity;
pub mod player;
pub mod server;
pub mod world;

/// A trait representing an event in the system.
///
/// This trait provides methods for retrieving the event's name and for type-safe downcasting.
pub trait Payload: Send + Sync {
    /// Returns the static name of the event type.
    ///
    /// # Returns
    /// A static string slice representing the name of the payload type.
    fn get_name_static() -> &'static str
    where
        Self: Sized;

    /// Returns the name of the payload instance.
    ///
    /// # Returns
    /// A static string slice representing the name of the payload instance.
    fn get_name(&self) -> &'static str;

    /// Provides an immutable reference to the payload as a trait object.
    ///
    /// This method allows for type-safe downcasting of the payload.
    ///
    /// # Returns
    /// An immutable reference to the payload as a `dyn Any` trait object.
    fn as_any(&self) -> &dyn Any;

    /// Provides a mutable reference to the payload as a trait object.
    ///
    /// This method allows for type-safe downcasting of the payload.
    ///
    /// # Returns
    /// A mutable reference to the payload as a `dyn Any` trait object.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Helper functions for safe downcasting of Payload implementations.
impl dyn Payload + '_ {
    /// Attempts to downcast an Arc<dyn Payload> to Arc<T> using name-based type checking.
    ///
    /// This method is safe to use across compilation boundaries as it uses string-based
    /// type identification instead of `TypeId`.
    ///
    /// # Type Parameters
    /// - `T`: The target type to downcast to. Must implement Payload.
    ///
    /// # Arguments
    /// - `payload`: The Arc<dyn Payload> to downcast.
    ///
    /// # Returns
    /// Some(Arc<T>) if the downcast succeeds, None otherwise.
    pub fn downcast_arc<T: Payload + 'static>(payload: Arc<dyn Payload>) -> Option<Arc<T>> {
        if payload.get_name() == T::get_name_static() {
            // Safe to downcast since we verified the type name
            unsafe {
                let raw = Arc::into_raw(payload);
                let typed = raw.cast::<T>();
                Some(Arc::from_raw(typed))
            }
        } else {
            None
        }
    }

    /// Attempts to downcast a &mut dyn Payload to &mut T using name-based type checking.
    ///
    /// # Type Parameters
    /// - `T`: The target type to downcast to. Must implement Payload.
    ///
    /// # Returns
    /// Some(&mut T) if the downcast succeeds, None otherwise.
    pub fn downcast_mut<T: Payload + 'static>(&mut self) -> Option<&mut T> {
        if self.get_name() == T::get_name_static() {
            // Safe to downcast since we verified the type name
            unsafe { Some(&mut *(std::ptr::from_mut::<dyn Payload>(self).cast::<T>())) }
        } else {
            None
        }
    }

    /// Attempts to downcast a &dyn Payload to &T using name-based type checking.
    ///
    /// # Type Parameters
    /// - `T`: The target type to downcast to. Must implement Payload.
    ///
    /// # Returns
    /// Some(&T) if the downcast succeeds, None otherwise.
    pub fn downcast_ref<T: Payload + 'static>(&self) -> Option<&T> {
        if self.get_name() == T::get_name_static() {
            // Safe to downcast since we verified the type name
            unsafe { Some(&*(std::ptr::from_ref::<dyn Payload>(self).cast::<T>())) }
        } else {
            None
        }
    }
}

/// A trait for cancellable events.
///
/// This trait provides methods to check and set the cancellation state of an event.
pub trait Cancellable: Send + Sync {
    /// Checks if the event has been cancelled.
    ///
    /// # Returns
    /// A boolean indicating whether the event is cancelled.
    fn cancelled(&self) -> bool;

    /// Sets the cancellation state of the event.
    ///
    /// # Arguments
    /// - `cancelled`: A boolean indicating the new cancellation state.
    fn set_cancelled(&mut self, cancelled: bool);
}
/// An enumeration representing the priority levels of events.
///
/// Mirrors Bukkit's EventPriority for compatibility:
/// - `Lowest` through `Highest`: executed in order, each can modify the event.
/// - `Monitor`: executed last, must NOT modify the event — used for logging and observation only.
///
/// Events with lower priority values are executed first, allowing higher priority events
/// to override their changes.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub enum EventPriority {
    /// Highest priority level. Executed first — can override all lower priority handlers.
    Highest,

    /// High priority level.
    High,

    /// Normal priority level. Default for most handlers.
    Normal,

    /// Low priority level.
    Low,

    /// Lowest priority level. Executed last among modifying handlers.
    Lowest,

    /// Monitor priority level. Executed after all other handlers.
    ///
    /// Handlers at this priority MUST NOT modify the event in any way.
    /// This is intended for logging, metrics, and observation only.
    /// Matches Bukkit's EventPriority.MONITOR.
    Monitor,
}

#[cfg(test)]
mod tests {
    use super::*;
    use server::server_command::ServerCommandEvent;
    use server::server_started::ServerStartedEvent;
    use server::server_stop::ServerStopEvent;
    use server::server_tick::ServerTickEvent;

    // --- Payload trait tests ---

    #[test]
    fn server_command_event_payload_name() {
        assert_eq!(ServerCommandEvent::get_name_static(), "ServerCommandEvent");
        let event = ServerCommandEvent::new("test".to_string());
        assert_eq!(event.get_name(), "ServerCommandEvent");
    }

    #[test]
    fn server_started_event_payload_name() {
        assert_eq!(ServerStartedEvent::get_name_static(), "ServerStartedEvent");
        let event = ServerStartedEvent::new(3, 5);
        assert_eq!(event.get_name(), "ServerStartedEvent");
    }

    #[test]
    fn server_tick_event_payload_name() {
        assert_eq!(ServerTickEvent::get_name_static(), "ServerTickEvent");
        let event = ServerTickEvent::new(42);
        assert_eq!(event.get_name(), "ServerTickEvent");
    }

    #[test]
    fn server_stop_event_payload_name() {
        assert_eq!(ServerStopEvent::get_name_static(), "ServerStopEvent");
        let event = ServerStopEvent::new("shutdown".to_string());
        assert_eq!(event.get_name(), "ServerStopEvent");
    }

    // --- Cancellable trait tests ---

    #[test]
    fn cancellable_event_starts_not_cancelled() {
        let event = ServerCommandEvent::new("test".to_string());
        assert!(!event.cancelled());
    }

    #[test]
    fn cancellable_event_can_be_cancelled() {
        let mut event = ServerCommandEvent::new("test".to_string());
        assert!(!event.cancelled());
        event.set_cancelled(true);
        assert!(event.cancelled());
    }

    #[test]
    fn cancellable_event_can_be_uncancelled() {
        let mut event = ServerCommandEvent::new("test".to_string());
        event.set_cancelled(true);
        assert!(event.cancelled());
        event.set_cancelled(false);
        assert!(!event.cancelled());
    }

    // --- Downcast tests ---

    #[test]
    fn downcast_ref_same_type_succeeds() {
        let event = ServerCommandEvent::new("hello".to_string());
        let payload: &dyn Payload = &event;
        let downcasted = payload.downcast_ref::<ServerCommandEvent>();
        assert!(downcasted.is_some());
        assert_eq!(downcasted.unwrap().command, "hello");
    }

    #[test]
    fn downcast_ref_different_type_fails() {
        let event = ServerCommandEvent::new("hello".to_string());
        let payload: &dyn Payload = &event;
        let downcasted = payload.downcast_ref::<ServerStartedEvent>();
        assert!(downcasted.is_none());
    }

    #[test]
    fn downcast_mut_same_type_succeeds() {
        let mut event = ServerCommandEvent::new("hello".to_string());
        let payload: &mut dyn Payload = &mut event;
        let downcasted = payload.downcast_mut::<ServerCommandEvent>();
        assert!(downcasted.is_some());
        downcasted.unwrap().command = "modified".to_string();
        assert_eq!(event.command, "modified");
    }

    #[test]
    fn downcast_mut_different_type_fails() {
        let mut event = ServerCommandEvent::new("hello".to_string());
        let payload: &mut dyn Payload = &mut event;
        let downcasted = payload.downcast_mut::<ServerTickEvent>();
        assert!(downcasted.is_none());
    }

    #[test]
    fn downcast_arc_same_type_succeeds() {
        let event = ServerCommandEvent::new("arc_test".to_string());
        let payload: Arc<dyn Payload> = Arc::new(event);
        let downcasted = <dyn Payload>::downcast_arc::<ServerCommandEvent>(payload);
        assert!(downcasted.is_some());
        assert_eq!(downcasted.unwrap().command, "arc_test");
    }

    #[test]
    fn downcast_arc_different_type_fails() {
        let event = ServerCommandEvent::new("arc_test".to_string());
        let payload: Arc<dyn Payload> = Arc::new(event);
        let downcasted = <dyn Payload>::downcast_arc::<ServerStartedEvent>(payload);
        assert!(downcasted.is_none());
    }

    // --- Non-cancellable event tests ---

    #[test]
    fn server_started_event_construction() {
        let event = ServerStartedEvent::new(3, 5);
        assert_eq!(event.world_count, 3);
        assert_eq!(event.plugin_count, 5);
    }

    #[test]
    fn server_tick_event_construction() {
        let event = ServerTickEvent::new(100);
        assert_eq!(event.tick_count, 100);
    }

    #[test]
    fn server_stop_event_construction() {
        let event = ServerStopEvent::new("operator shutdown".to_string());
        assert_eq!(event.reason, "operator shutdown");
    }

    #[test]
    fn server_command_event_construction() {
        let event = ServerCommandEvent::new("say hello".to_string());
        assert_eq!(event.command, "say hello");
        assert!(!event.cancelled());
    }

    // --- Event clone tests ---

    #[test]
    fn cancellable_event_clone_preserves_cancelled_state() {
        let mut event = ServerCommandEvent::new("test".to_string());
        event.set_cancelled(true);
        let cloned = event.clone();
        assert!(cloned.cancelled());
        assert_eq!(cloned.command, "test");
    }

    #[test]
    fn non_cancellable_event_clone() {
        let event = ServerTickEvent::new(42);
        let cloned = event.clone();
        assert_eq!(cloned.tick_count, 42);
    }
}
