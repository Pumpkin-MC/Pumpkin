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
    pub fn add_global_chunk_listener(&self) -> Receiver<(ChunkPos, SyncChunk)> {
        let (tx, rx) = crossbeam::channel::unbounded();
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
                if matches!(global[i].send((pos, chunk.clone())), Ok(())) {
                    // log::debug!("global listener {i} send {pos:?}");
                } else {
                    // log::debug!("one global listener dropped");
                    global.remove(i);
                    len -= 1;
                    continue;
                }
                i += 1;
            }
        }
    }
}
