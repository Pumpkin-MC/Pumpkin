//! The tables that map handler ids to handlers
//!
//! Every kind of callback the host can start (events, commands, suggestions, tasks, AI goals,
//! chunk generators and each `GameTest` callback) has its own [`Registry`]
//!
//! The guest picks the id, tells the host only that number and looks the handler up again when
//! the host calls back, so the host never holds anything that belongs to the guest

use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

/// A boxed future that is not required to be `Send`
///
/// Component guests run on one thread and host imports return futures that are not `Send`, so
/// the erased handler traits return this instead of a `Send` boxed future
pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Maps handler ids chosen by the guest to shared callables
///
/// Holding the lock across an await is impossible by construction: [`register`](Self::register)
/// and [`get`](Self::get) are synchronous and take it for the length of one call
///
/// That matters because the host can start a second callback while the first is suspended; if
/// the first still held the table, the second would wait on it forever
pub struct Registry<T: ?Sized> {
    entries: Mutex<Entries<T>>,
}

struct Entries<T: ?Sized> {
    next_id: u32,
    handlers: BTreeMap<u32, Arc<T>>,
}

impl<T: ?Sized> Registry<T> {
    /// Creates an empty table
    ///
    /// This is `const` so a table can live in a `static`
    pub const fn new() -> Self {
        Self {
            entries: Mutex::new(Entries {
                next_id: 0,
                handlers: BTreeMap::new(),
            }),
        }
    }

    /// Stores `handler` and returns the id the host uses to call it back
    ///
    /// Ids count up from zero per table and are never reused, so an id the host remembers can
    /// never point at a different handler later
    pub fn register(&self, handler: Arc<T>) -> u32 {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let id = entries.next_id;
        entries.next_id += 1;
        entries.handlers.insert(id, handler);
        id
    }

    /// Returns an owned handle to the handler stored under `id`
    ///
    /// The `Arc` is cloned while the lock is held and the lock is released before this returns,
    /// so the caller runs the handler with no guard alive
    pub fn get(&self, id: u32) -> Option<Arc<T>> {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .handlers
            .get(&id)
            .cloned()
    }
}
