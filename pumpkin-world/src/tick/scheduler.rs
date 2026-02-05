use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashSet;

use crate::tick::{MAX_TICK_DELAY, OrderedTick, ScheduledTick};

pub struct ChunkTickScheduler<T> {
    tick_queue: Mutex<[Vec<OrderedTick<T>>; MAX_TICK_DELAY]>,
    queued_ticks: Mutex<FxHashSet<(BlockPos, T)>>,
    offset: AtomicUsize,
}

impl<'a, T: std::hash::Hash + Eq> ChunkTickScheduler<&'a T> {
    pub fn step_tick(&self) -> Vec<OrderedTick<&'a T>> {
        // Atomic update for the offset
        let current_offset = self.offset.fetch_add(1, Ordering::SeqCst) % MAX_TICK_DELAY;
        let next_offset = (current_offset + 1) % MAX_TICK_DELAY;
        self.offset.store(next_offset, Ordering::SeqCst);

        let res = {
            let mut queue = self.tick_queue.lock().unwrap();
            std::mem::take(&mut queue[current_offset])
        };

        if !res.is_empty() {
            let mut set = self.queued_ticks.lock().unwrap();
            for next_tick in &res {
                set.remove(&(next_tick.position, next_tick.value));
            }
        }
        res
    }

    pub fn schedule_tick(&self, tick: &ScheduledTick<&'a T>, sub_tick_order: u64) {
        let mut set = self.queued_ticks.lock().unwrap();

        if set.insert((tick.position, tick.value)) {
            let mut queue = self.tick_queue.lock().unwrap();
            let offset = self.offset.load(Ordering::SeqCst);
            let index = (offset + tick.delay as usize) % MAX_TICK_DELAY;

            queue[index].push(OrderedTick {
                priority: tick.priority,
                sub_tick_order,
                position: tick.position,
                value: tick.value,
            });
        }
    }

    pub fn is_scheduled(&self, pos: BlockPos, value: &T) -> bool {
        self.queued_ticks.lock().unwrap().contains(&(pos, value))
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<ScheduledTick<&'a T>> {
        let offset = self.offset.load(Ordering::SeqCst);
        let queue = self.tick_queue.lock().unwrap();
        let mut res = Vec::new();

        for i in 0..MAX_TICK_DELAY {
            let index = (offset + i) % MAX_TICK_DELAY;
            res.extend(queue[index].iter().map(|x| ScheduledTick {
                delay: i as u8,
                priority: x.priority,
                position: x.position,
                value: x.value,
            }));
        }
        res
    }
}

impl<'a, T: std::hash::Hash + Eq + 'static> FromIterator<ScheduledTick<&'a T>>
    for ChunkTickScheduler<&'a T>
{
    fn from_iter<I: IntoIterator<Item = ScheduledTick<&'a T>>>(iter: I) -> Self {
        let scheduler = Self::default();
        let iter = iter.into_iter();

        let (lower, _) = iter.size_hint();
        if lower > 0 {
            scheduler.queued_ticks.lock().unwrap().reserve(lower);
        }

        for tick in iter {
            scheduler.schedule_tick(&tick, 0);
        }
        scheduler
    }
}

impl<T> Default for ChunkTickScheduler<T> {
    fn default() -> Self {
        Self {
            tick_queue: Mutex::new(std::array::from_fn(|_| Vec::new())),
            queued_ticks: Mutex::new(FxHashSet::default()),
            offset: AtomicUsize::new(0),
        }
    }
}
