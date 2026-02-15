use super::ChunkPos;
use crate::level::SyncChunk;
use crossbeam::channel::{Receiver, Sender};
use std::sync::Mutex;
use tokio::sync::oneshot;

pub struct ChunkListener {
    single: Mutex<Vec<(ChunkPos, oneshot::Sender<SyncChunk>)>>,
    global: Mutex<Vec<Sender<(ChunkPos, SyncChunk)>>>,
}
impl Default for ChunkListener {
    fn default() -> Self {
        Self::new()
    }
}
impl ChunkListener {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            single: Mutex::new(Vec::new()),
            global: Mutex::new(Vec::new()),
        }
    }
    pub fn add_single_chunk_listener(&self, pos: ChunkPos) -> oneshot::Receiver<SyncChunk> {
        let (tx, rx) = oneshot::channel();
        self.single.lock().unwrap().push((pos, tx));
        rx
    }
    /// Capacity for the global listener channel. Large enough to buffer several ticks
    /// of chunk notifications without blocking, but bounded to prevent memory growth
    /// when chunks are produced faster than the consumer can drain (e.g. void worlds).
    const GLOBAL_LISTENER_CAPACITY: usize = 512;

    pub fn add_global_chunk_listener(&self) -> Receiver<(ChunkPos, SyncChunk)> {
        let (tx, rx) = crossbeam::channel::bounded(Self::GLOBAL_LISTENER_CAPACITY);
        self.global.lock().unwrap().push(tx);
        rx
    }
    pub fn process_new_chunk(&self, pos: ChunkPos, chunk: &SyncChunk) {
        {
            let mut single = self.single.lock().unwrap();
            let mut i = 0;
            let mut len = single.len();
            while i < len {
                if single[i].0 == pos {
                    let (_, send) = single.remove(i);
                    let _ = send.send(chunk.clone());
                    // log::debug!("single listener {i} send {pos:?}");
                    len -= 1;
                    continue;
                }
                if single[i].1.is_closed() {
                    // let listener_pos = single[i].0;
                    single.remove(i);
                    // log::debug!("single listener dropped {listener_pos:?}");
                    len -= 1;
                    continue;
                }
                i += 1;
            }
        }
        {
            let mut global = self.global.lock().unwrap();
            let mut i = 0;
            let mut len = global.len();
            while i < len {
                match global[i].try_send((pos, chunk.clone())) {
                    Ok(()) => {}
                    Err(crossbeam::channel::TrySendError::Full(_)) => {
                        // Channel full - drop this notification. The player will
                        // pick up the chunk via DashMap lookup on the next view
                        // distance update, so missing a notification is safe.
                    }
                    Err(crossbeam::channel::TrySendError::Disconnected(_)) => {
                        global.remove(i);
                        len -= 1;
                        continue;
                    }
                }
                i += 1;
            }
        }
    }
}
