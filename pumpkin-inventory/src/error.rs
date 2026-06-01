use thiserror::Error;

/// Errors that can occur during inventory operations.
///
/// These errors represent various failure conditions when handling inventory
/// interactions, such as invalid slot indices, permission issues, or protocol errors.
#[derive(Error, Debug)]
pub enum InventoryError {
    /// Failed to acquire a lock on an inventory or slot.
    #[error("Unable to lock")]
    LockError,
    /// The specified slot index is invalid or out of bounds.
    #[error("Invalid slot")]
    InvalidSlot,
    /// A player attempted to interact with a container that is closed.
    ///
    /// The parameter is the player's entity ID.
    #[error("Player '{0}' tried to interact with a closed container")]
    ClosedContainerInteract(i32),
    /// Multiple players attempted to drag items in the same container simultaneously.
    #[error("Multiple players dragging in a container at once")]
    MultiplePlayersDragging,
    /// Drag operation was performed out of order (e.g., end before start).
    #[error("Out of order dragging")]
    OutOfOrderDragging,
    /// The received inventory packet is malformed or invalid.
    #[error("Invalid inventory packet")]
    InvalidPacket,
    /// The player lacks permission to perform this inventory operation.
    #[error("Player does not have enough permissions")]
    PermissionError,
}
