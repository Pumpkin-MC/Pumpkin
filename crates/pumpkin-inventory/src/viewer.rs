use std::sync::atomic::{AtomicU16, Ordering};

#[derive(Debug)]
pub struct ViewerCountTracker {
    pub old: AtomicU16,
    pub current: AtomicU16,
}

impl Default for ViewerCountTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewerCountTracker {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            old: AtomicU16::new(0),
            current: AtomicU16::new(0),
        }
    }

    pub fn open_container(&self) {
        self.current.fetch_add(1, Ordering::Relaxed);
    }

    pub fn close_container(&self) {
        self.current.fetch_sub(1, Ordering::Relaxed);
    }

    /// Returns the current number of players viewing this container
    pub fn get_viewer_count(&self) -> u16 {
        self.current.load(Ordering::Relaxed)
    }
}
