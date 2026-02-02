use super::{ChunkLevel, ChunkPos, HashMapType};
use std::collections::hash_map::Entry;
use std::sync::{Arc, Condvar, Mutex};
use std::mem::swap;
use crate::level::Level;
use std::sync::atomic::Ordering::SeqCst;
use crate::chunk_system::chunk_state::StagedChunkEnum;

pub type LevelChange = (
    HashMapType<ChunkPos, (StagedChunkEnum, StagedChunkEnum)>,
    ChunkLevel,
);

pub struct LevelChannel {
    pub value: Mutex<(Option<LevelChange>, Option<Vec<ChunkPos>>)>,
    pub notify: Condvar,
}

impl Default for LevelChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelChannel {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            value: Mutex::new((None, None)),
            notify: Condvar::new(),
        }
    }
    pub fn set_both(
        &self,
        new_value: (
            HashMapType<ChunkPos, (StagedChunkEnum, StagedChunkEnum)>,
            ChunkLevel,
        ),
        pos: Vec<ChunkPos>,
    ) {
        // debug!("set new level and priority");
        let mut value = self.value.lock().unwrap();
        value.1 = Some(pos);
        if let Some(old) = &mut value.0 {
            for (pos, change) in new_value.0 {
                match old.0.entry(pos) {
                    Entry::Occupied(mut entry) => {
                        let tmp = entry.get_mut();
                        debug_assert_eq!(tmp.1, change.0);
                        if tmp.0 == change.1 {
                            entry.remove();
                        } else {
                            tmp.1 = change.1;
                        }
                    }
                    Entry::Vacant(entry) => {
                        debug_assert_ne!(change.0, change.1);
                        entry.insert(change);
                    }
                }
            }
            old.1 = new_value.1;
        } else {
            value.0 = Some(new_value);
        }
        self.notify.notify_one();
    }
    pub fn set_level(
        &self,
        new_value: (
            HashMapType<ChunkPos, (StagedChunkEnum, StagedChunkEnum)>,
            ChunkLevel,
        ),
    ) {
        // debug!("set new level");
        let mut value = self.value.lock().unwrap();
        if let Some(old) = &mut value.0 {
            for (pos, change) in new_value.0 {
                match old.0.entry(pos) {
                    Entry::Occupied(mut entry) => {
                        let tmp = entry.get_mut();
                        debug_assert_eq!(tmp.1, change.0);
                        if tmp.0 == change.1 {
                            entry.remove();
                        } else {
                            tmp.1 = change.1;
                        }
                    }
                    Entry::Vacant(entry) => {
                        debug_assert_ne!(change.0, change.1);
                        entry.insert(change);
                    }
                }
            }
            old.1 = new_value.1;
        } else {
            value.0 = Some(new_value);
        }
        self.notify.notify_one();
    }
    pub fn set_priority(&self, pos: Vec<ChunkPos>) {
        // debug!("set new priority");
        self.value.lock().unwrap().1 = Some(pos);
        self.notify.notify_one();
    }
    pub fn get(&self) -> (Option<LevelChange>, Option<Vec<ChunkPos>>) {
        let mut lock = self.value.lock().unwrap();
        let mut ret = (None, None);
        swap(&mut ret, &mut *lock);
        ret
    }
    pub fn wait_and_get(&self, level: &Arc<Level>) -> (Option<LevelChange>, Option<Vec<ChunkPos>>) {
        let mut lock = self.value.lock().unwrap();
        while lock.0.is_none()
            && lock.1.is_none()
            && !level.should_unload.load(SeqCst)
            && !level.should_save.load(SeqCst)
            && !level.shut_down_chunk_system.load(SeqCst)
        {
            lock = self.notify.wait(lock).unwrap();
        }
        let mut ret = (None, None);
        swap(&mut ret, &mut *lock);
        ret
    }
    pub fn notify(&self) {
        let val = self.value.lock().unwrap();
        drop(val);
        self.notify.notify_one();
    }
}
