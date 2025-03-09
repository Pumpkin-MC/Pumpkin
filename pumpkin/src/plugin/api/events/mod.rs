use std::any::Any;

pub mod block;
pub mod player;
pub mod server;
pub mod world;

/// A trait representing an event in the system.
///
/// This trait provides methods for retrieving the event's name and for type-safe downcasting.
pub trait Event: Send + Sync {
    /// Returns the static name of the event type.
    ///
    /// # Returns
    /// A static string slice representing the name of the event type.
    fn get_name_static() -> &'static str
    where
        Self: Sized;

    /// Returns the name of the event instance.
    ///
    /// # Returns
    /// A static string slice representing the name of the event instance.
    fn get_name(&self) -> &'static str;

    /// Provides a mutable reference to the event as a trait object.
    ///
    /// This method allows for type-safe downcasting of the event.
    ///
    /// # Returns
    /// A mutable reference to the event as a `dyn Any` trait object.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Provides an immutable reference to the event as a trait object.
    ///
    /// This method allows for type-safe downcasting of the event.
    ///
    /// # Returns
    /// An immutable reference to the event as a `dyn Any` trait object.
    fn as_any(&self) -> &dyn Any;
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
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone)]
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
