//! Guest-side handler tables. IDs are stable indices and are never reused.

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

/// A boxed callback future that need not be `Send`.
pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Stores shared handlers without holding a lock while they execute.
pub struct Registry<T: ?Sized> {
    entries: Mutex<Vec<Option<Arc<T>>>>,
}

impl<T: ?Sized> Registry<T> {
    pub const fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
        }
    }

    #[expect(clippy::expect_used, reason = "handler IDs must fit in u32")]
    pub fn register(&self, handler: Arc<T>) -> u32 {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let id = u32::try_from(entries.len()).expect("handler ID space exhausted");
        entries.push(Some(handler));
        id
    }

    pub fn get(&self, id: u32) -> Option<Arc<T>> {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(id as usize)
            .and_then(Clone::clone)
    }

    /// Removes a handler without making its ID available for another registration.
    pub fn take(&self, id: u32) -> Option<Arc<T>> {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_mut(id as usize)
            .and_then(Option::take)
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
