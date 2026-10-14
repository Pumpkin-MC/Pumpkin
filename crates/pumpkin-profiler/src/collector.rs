use std::{
    collections::HashMap,
    io,
    sync::{
        Mutex,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

pub const MAX_FRAMES: usize = 32;
const SHARDS: usize = 128;
const SAMPLES_PER_SHARD: usize = 256;
pub const MAX_SAMPLES: usize = SHARDS * SAMPLES_PER_SHARD;

pub static COLLECTOR: Collector = Collector::new();

#[derive(Clone, Copy)]
pub struct Sample {
    pub address: usize,
    pub size: usize,
    pub frames: [usize; MAX_FRAMES],
    pub frame_count: usize,
}

struct Shard {
    samples: Mutex<Option<HashMap<usize, Sample>>>,
    count: AtomicUsize,
}

impl Shard {
    const fn new() -> Self {
        Self {
            samples: Mutex::new(None),
            count: AtomicUsize::new(0),
        }
    }
}

pub struct Collector {
    generation: AtomicU64,
    dropped: AtomicU64,
    shards: [Shard; SHARDS],
}

impl Collector {
    const fn new() -> Self {
        Self {
            generation: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            shards: [const { Shard::new() }; SHARDS],
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub fn start(&self, generation: u64) -> io::Result<()> {
        for shard in &self.shards {
            let mut samples = HashMap::new();
            if let Err(error) = samples.try_reserve(SAMPLES_PER_SHARD) {
                self.stop();
                return Err(io::Error::other(error));
            }
            *shard
                .samples
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(samples);
            shard.count.store(0, Ordering::Release);
        }
        self.dropped.store(0, Ordering::Relaxed);
        self.generation.store(generation, Ordering::Release);
        Ok(())
    }

    pub fn stop(&self) {
        self.generation.store(0, Ordering::Release);
        for shard in &self.shards {
            *shard
                .samples
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            shard.count.store(0, Ordering::Release);
        }
    }

    pub fn insert(&self, sample: &Sample, generation: u64) {
        let shard = &self.shards[shard_index(sample.address)];
        let mut storage = shard
            .samples
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if generation == 0 || self.generation() != generation {
            return;
        }
        let Some(samples) = storage.as_mut() else {
            return;
        };
        if samples.len() == SAMPLES_PER_SHARD || samples.len() == samples.capacity() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        }
        // HashMap capacity can shrink after deletions. Check it as well as the
        // logical bound so insert never grows storage inside an allocator hook.
        samples.insert(sample.address, *sample);
        shard.count.store(samples.len(), Ordering::Release);
    }

    pub fn remove(&self, address: usize, generation: u64) -> Option<Sample> {
        let shard = &self.shards[shard_index(address)];
        if shard.count.load(Ordering::Acquire) == 0 {
            return None;
        }
        let mut storage = shard
            .samples
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.generation() != generation {
            return None;
        }
        let samples = storage.as_mut()?;
        let removed = samples.remove(&address);
        shard.count.store(samples.len(), Ordering::Release);
        removed
    }

    pub fn snapshot(&self) -> (Vec<Sample>, u64) {
        // Growing this buffer under a shard lock could re-enter that shard on free.
        let mut result = Vec::with_capacity(MAX_SAMPLES);
        for shard in &self.shards {
            let storage = shard
                .samples
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(samples) = storage.as_ref() {
                result.extend(samples.values().copied());
            }
        }
        (result, self.dropped.load(Ordering::Relaxed))
    }
}

const fn shard_index(address: usize) -> usize {
    let mut value = address as u64;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    (value ^ (value >> 31)) as usize & (SHARDS - 1)
}
