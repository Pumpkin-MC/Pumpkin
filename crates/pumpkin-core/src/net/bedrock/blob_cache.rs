use std::collections::VecDeque;

use bytes::Bytes;
use rustc_hash::FxHashMap;

/// Blob bytes kept per client before the oldest are evicted.
const MAX_BLOB_CACHE_BYTES: usize = 16 * 1024 * 1024;

struct Entry {
    blob: Bytes,
    /// Identifies this insertion in the queues; older queue entries of the hash are stale.
    seq: u64,
    /// The client has reported on the blob (`Missing Ids` served or `Found Ids`).
    answered: bool,
}

/// Chunk blobs a client may still request by hash. Over the cap, answered blobs go first
/// since the client has them; blobs it has not answered yet go only once nothing else is left.
#[derive(Default)]
pub struct BlobCache {
    blobs: FxHashMap<u64, Entry>,
    /// `(hash, seq)` of blobs the client has not answered yet, oldest first.
    unanswered: VecDeque<(u64, u64)>,
    /// `(hash, seq)` of answered blobs, oldest first.
    answered: VecDeque<(u64, u64)>,
    next_seq: u64,
    bytes: usize,
}

impl BlobCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, hash: u64, blob: Bytes) {
        let seq = self.take_seq();
        // Equal hash means equal content. A new chunk may get it requested again, so an
        // answered copy goes back to waiting for an answer
        if let Some(entry) = self.blobs.get_mut(&hash) {
            if entry.answered {
                entry.answered = false;
                entry.seq = seq;
                self.unanswered.push_back((hash, seq));
                self.compact();
            }
            return;
        }
        self.bytes += blob.len();
        self.blobs.insert(
            hash,
            Entry {
                blob,
                seq,
                answered: false,
            },
        );
        self.unanswered.push_back((hash, seq));
        self.compact();
        self.evict();
    }

    #[must_use]
    pub fn get(&self, hash: u64) -> Option<&Bytes> {
        self.blobs.get(&hash).map(|entry| &entry.blob)
    }

    /// The client reported the blob as held (`Found Ids`), so it is not requested again.
    pub fn remove(&mut self, hash: u64) {
        if let Some(entry) = self.blobs.remove(&hash) {
            self.bytes -= entry.blob.len();
        }
    }

    /// The blob was served on a miss, so it is the first to go when the cache is full.
    pub fn mark_answered(&mut self, hash: u64) {
        let seq = self.take_seq();
        let Some(entry) = self.blobs.get_mut(&hash) else {
            return;
        };
        if entry.answered {
            return;
        }
        entry.answered = true;
        entry.seq = seq;
        self.answered.push_back((hash, seq));
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

    const fn take_seq(&mut self) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        seq
    }

    /// Removed and re-queued hashes leave stale queue entries; drop them once they
    /// outnumber the live blobs, which leaves at most one entry per blob.
    fn compact(&mut self) {
        let limit = 2 * self.blobs.len() + 64;
        let blobs = &self.blobs;
        for queue in [&mut self.unanswered, &mut self.answered] {
            if queue.len() > limit {
                queue.retain(|&(hash, seq)| blobs.get(&hash).is_some_and(|e| e.seq == seq));
            }
        }
    }

    fn evict(&mut self) {
        while self.bytes > MAX_BLOB_CACHE_BYTES {
            let Some((hash, seq)) = self
                .answered
                .pop_front()
                .or_else(|| self.unanswered.pop_front())
            else {
                break;
            };
            if self.blobs.get(&hash).is_some_and(|e| e.seq == seq) {
                self.remove(hash);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blob(size: usize) -> Bytes {
        Bytes::from(vec![0; size])
    }

    #[test]
    fn reinserted_blob_is_not_evicted_through_its_stale_queue_entry() {
        let mut cache = BlobCache::new();
        cache.insert(1, blob(1));
        cache.remove(1);
        cache.insert(2, blob(MAX_BLOB_CACHE_BYTES - 1));
        cache.insert(1, blob(1));

        // Over the cap: 2 is older than the new copy of 1 and goes first
        cache.insert(3, blob(1));

        assert!(cache.get(2).is_none());
        assert!(cache.get(1).is_some());
        assert!(cache.get(3).is_some());
    }

    #[test]
    fn answered_blobs_are_evicted_before_unanswered_ones() {
        let mut cache = BlobCache::new();
        cache.insert(1, blob(MAX_BLOB_CACHE_BYTES / 2));
        cache.insert(2, blob(MAX_BLOB_CACHE_BYTES / 2));
        cache.mark_answered(2);

        cache.insert(3, blob(1));

        assert!(cache.get(1).is_some());
        assert!(cache.get(2).is_none());
        assert_eq!(cache.bytes(), MAX_BLOB_CACHE_BYTES / 2 + 1);
    }

    #[test]
    fn answered_blob_sent_again_is_not_evicted_first() {
        let mut cache = BlobCache::new();
        cache.insert(1, blob(MAX_BLOB_CACHE_BYTES / 2));
        cache.insert(2, blob(MAX_BLOB_CACHE_BYTES / 2));
        cache.mark_answered(2);
        cache.mark_answered(1);
        // A new chunk references 2 again before the client answered for it
        cache.insert(2, blob(MAX_BLOB_CACHE_BYTES / 2));

        cache.insert(3, blob(1));

        assert!(cache.get(1).is_none());
        assert!(cache.get(2).is_some());
    }

    #[test]
    fn queues_stay_bounded_when_a_common_hash_cycles() {
        let mut cache = BlobCache::new();
        cache.insert(0, blob(1));
        for _ in 0..1000 {
            cache.insert(1, blob(1));
            cache.remove(1);
        }

        assert!(cache.unanswered.len() <= 2 * cache.len() + 64 + 1);
    }
}
