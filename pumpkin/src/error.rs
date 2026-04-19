use pumpkin_inventory::InventoryError;
use pumpkin_protocol::ser::ReadingError;
use pumpkin_storage::StorageError;
use std::fmt::Display;
use tracing::Level;

use crate::log_at_level;

pub trait PumpkinError: Send + std::error::Error + Display {
    fn is_kick(&self) -> bool;

    fn log(&self) {
        log_at_level!(self.severity(), "{self}");
    }

    fn severity(&self) -> Level;

    fn client_kick_reason(&self) -> Option<String>;
}

impl<ErrorType: PumpkinError + 'static> From<ErrorType> for Box<dyn PumpkinError> {
    fn from(error: ErrorType) -> Self {
        Box::new(error)
    }
}
impl PumpkinError for InventoryError {
    fn is_kick(&self) -> bool {
        use InventoryError::{
            ClosedContainerInteract, InvalidPacket, InvalidSlot, LockError,
            MultiplePlayersDragging, OutOfOrderDragging, PermissionError,
        };
        match self {
            InvalidSlot | ClosedContainerInteract(..) | InvalidPacket | PermissionError => true,
            LockError | OutOfOrderDragging | MultiplePlayersDragging => false,
        }
    }
    fn severity(&self) -> Level {
        use InventoryError::{
            ClosedContainerInteract, InvalidPacket, InvalidSlot, LockError,
            MultiplePlayersDragging, OutOfOrderDragging, PermissionError,
        };
        match self {
            LockError
            | InvalidSlot
            | ClosedContainerInteract(..)
            | InvalidPacket
            | PermissionError => Level::ERROR,
            OutOfOrderDragging => Level::INFO,
            MultiplePlayersDragging => Level::WARN,
        }
    }

    fn client_kick_reason(&self) -> Option<String> {
        None
    }
}

impl PumpkinError for ReadingError {
    fn is_kick(&self) -> bool {
        true
    }

    fn severity(&self) -> Level {
        Level::ERROR
    }

    fn client_kick_reason(&self) -> Option<String> {
        None
    }
}

impl PumpkinError for StorageError {
    fn is_kick(&self) -> bool {
        false
    }

    fn severity(&self) -> Level {
        Level::WARN
    }

    fn client_kick_reason(&self) -> Option<String> {
        Some(format!("Storage error: {self}"))
    }
}
