//! Guest-side handler tables. IDs are stable indices and are never reused.

use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

/// A boxed callback future that need not be `Send`.
pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

struct RegistryInner<T: ?Sized> {
    next_id: u32,
    entries: BTreeMap<u32, Arc<T>>,
}

/// Stores shared handlers without holding a lock while they execute.
pub struct Registry<T: ?Sized> {
    inner: Mutex<RegistryInner<T>>,
}

impl<T: ?Sized> Registry<T> {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(RegistryInner {
                next_id: 0,
                entries: BTreeMap::new(),
            }),
        }
    }

    #[expect(clippy::expect_used, reason = "handler IDs must fit in u32")]
    pub fn register(&self, handler: Arc<T>) -> u32 {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let id = inner.next_id;
        inner.next_id = inner
            .next_id
            .checked_add(1)
            .expect("handler ID space exhausted");
        inner.entries.insert(id, handler);
        id
    }

    pub fn get(&self, id: u32) -> Option<Arc<T>> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entries
            .get(&id)
            .cloned()
    }

    /// Removes a handler without making its ID available for another registration.
    pub fn take(&self, id: u32) -> Option<Arc<T>> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entries
            .remove(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::Registry;
    use std::sync::Arc;

    #[test]
    fn removed_ids_are_not_reused() {
        let registry = Registry::new();
        let first = registry.register(Arc::new(10));
        let second = registry.register(Arc::new(20));
        assert_eq!(registry.take(first).as_deref(), Some(&10));
        assert!(registry.take(first).is_none());
        assert!(registry.get(first).is_none());
        assert_eq!(registry.get(second).as_deref(), Some(&20));
        let third = registry.register(Arc::new(30));
        assert!(third > second);
        assert_eq!(registry.get(third).as_deref(), Some(&30));
        assert!(registry.get(u32::MAX).is_none());
        assert!(registry.take(u32::MAX).is_none());
    }
}
