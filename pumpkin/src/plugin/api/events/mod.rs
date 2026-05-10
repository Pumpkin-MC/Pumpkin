use std::any::Any;
use std::sync::Arc;

pub mod block;
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
/// Events with lower priority values are executed first, allowing higher priority events
/// to override their changes.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub enum EventPriority {
    /// Highest priority level.
    Highest,

    /// High priority level.
    High,

    /// Normal priority level.
    Normal,

    /// Low priority level.
    Low,

    /// Lowest priority level.
    Lowest,
}
