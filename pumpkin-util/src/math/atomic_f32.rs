use std::sync::atomic::{AtomicU32, Ordering};

/// A thread-safe atomic wrapper around `f32` values.
pub struct AtomicF32 {
    /// The underlying atomic storage of the float as `u32` bits.
    storage: AtomicU32,
}

impl AtomicF32 {
    /// Creates a new `AtomicF32` initialized with `value`.
    ///
    /// # Arguments
    /// * `value` – The initial float value.
    ///
    /// # Returns
    /// A new `AtomicF32` instance.
    #[must_use]
    pub const fn new(value: f32) -> Self {
        let as_u32 = value.to_bits();
        Self {
            storage: AtomicU32::new(as_u32),
        }
    }

    /// Stores a new value into the atomic float.
    ///
    /// # Arguments
    /// * `value` – The new float to store.
    /// * `ordering` – The memory ordering for the store operation.
    pub fn store(&self, value: f32, ordering: Ordering) {
        let as_u32 = value.to_bits();
        self.storage.store(as_u32, ordering);
    }

    /// Loads the current value of the atomic float.
    ///
    /// # Arguments
    /// * `ordering` – The memory ordering for the load operation.
    ///
    /// # Returns
    /// The current `f32` value.
    pub fn load(&self, ordering: Ordering) -> f32 {
        let as_u32 = self.storage.load(ordering);
        f32::from_bits(as_u32)
    }

    /// Performs a compare-and-exchange operation on the atomic float.
    ///
    /// # Arguments
    /// * `current` – The value expected to be currently stored.
    /// * `new` – The value to store if `current` matches the stored value.
    /// * `success` – Memory ordering to use on success.
    /// * `failure` – Memory ordering to use on failure.
    ///
    /// # Returns
    /// `Ok(f32)` containing the previous value if the exchange succeeded,
    /// or `Err(f32)` containing the current value if the exchange failed.
    pub fn compare_exchange(
        &self,
        current: f32,
        new: f32,
        success: Ordering,
        failure: Ordering,
    ) -> Result<f32, f32> {
        let current_bits = current.to_bits();
        let new_bits = new.to_bits();
        self.storage
            .compare_exchange(current_bits, new_bits, success, failure)
            .map(f32::from_bits)
            .map_err(f32::from_bits)
    }
}
