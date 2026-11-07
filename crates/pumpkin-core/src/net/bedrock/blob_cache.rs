use std::collections::VecDeque;

use bytes::Bytes;
use rustc_hash::FxHashMap;

/// Blob bytes kept per client before the oldest are evicted.
const MAX_BLOB_CACHE_BYTES: usize = 16 * 1024 * 1024;

/// Chunk blobs a client may still request by hash, oldest evicted first once over the cap.
#[derive(Default)]
pub struct BlobCache {
    blobs: FxHashMap<u64, Bytes>,
    order: VecDeque<u64>,
    bytes: usize,
}

impl BlobCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, hash: u64, blob: Bytes) {
        // Equal hash means equal content, so a repeat only needs to keep its slot.
        if self.blobs.contains_key(&hash) {
            return;
        }
        self.bytes += blob.len();
        self.blobs.insert(hash, blob);
        self.order.push_back(hash);
        // Removed hashes stay queued until here, so the queue cannot grow without bound.
        if self.order.len() > 2 * self.blobs.len() + 64 {
            let blobs = &self.blobs;
            self.order.retain(|queued| blobs.contains_key(queued));
        }

        while self.bytes > MAX_BLOB_CACHE_BYTES
            && let Some(oldest) = self.order.pop_front()
        {
            self.remove_blob(oldest);
        }
    }

    #[must_use]
    pub fn get(&self, hash: u64) -> Option<&Bytes> {
        self.blobs.get(&hash)
    }

    /// The client reported the blob as held (`Found Ids`), so it is not requested again.
    pub fn remove(&mut self, hash: u64) {
        self.remove_blob(hash);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.blobs.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blobs.is_empty()
    }

    #[must_use]
    pub const fn bytes(&self) -> usize {
        self.bytes
    }

    fn remove_blob(&mut self, hash: u64) {
        if let Some(blob) = self.blobs.remove(&hash) {
            self.bytes -= blob.len();
        }
    }
}
