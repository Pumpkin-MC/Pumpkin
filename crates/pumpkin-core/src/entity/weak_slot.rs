use std::sync::{Arc, Mutex, Weak};

/// A shared, optionally empty weak reference, e.g. the entity another one currently follows.
pub struct WeakSlot<T: ?Sized>(Mutex<Option<Weak<T>>>);

impl<T: ?Sized> WeakSlot<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self(Mutex::new(None))
    }

    /// The referenced value, or `None` when empty or already dropped.
    pub fn get(&self) -> Option<Arc<T>> {
        self.lock().as_ref().and_then(Weak::upgrade)
    }

    pub fn set(&self, value: &Arc<T>) {
        *self.lock() = Some(Arc::downgrade(value));
    }

    pub fn clear(&self) {
        *self.lock() = None;
    }

    /// Empties the slot, returning what it held.
    pub fn take(&self) -> Option<Arc<T>> {
        self.lock().take().and_then(|weak| weak.upgrade())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Weak<T>>> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl<T: ?Sized> Default for WeakSlot<T> {
    fn default() -> Self {
        Self::new()
    }
}
