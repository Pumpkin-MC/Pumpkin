use std::sync::{
    Mutex, MutexGuard,
    atomic::{AtomicUsize, Ordering},
};

use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashSet;

use crate::tick::{MAX_TICK_DELAY, OrderedTick, ScheduledTick};

pub struct ChunkTickScheduler<T> {
    inner: Mutex<Option<Box<ChunkTickSchedulerInner<T>>>>,
    offset: AtomicUsize,
}

struct ChunkTickSchedulerInner<T> {
    tick_queue: [Vec<OrderedTick<T>>; MAX_TICK_DELAY],
    queued_ticks: FxHashSet<(BlockPos, T)>,
}

impl<T> ChunkTickSchedulerInner<T> {
    fn boxed() -> Box<Self> {
        Box::new(Self {
            tick_queue: std::array::from_fn(|_| Vec::new()),
            queued_ticks: FxHashSet::default(),
        })
    }
}

impl<T: std::hash::Hash + Eq + Copy> ChunkTickSchedulerInner<T> {
    fn schedule(&mut self, offset: usize, tick: &ScheduledTick<T>, sub_tick_order: i64) {
        if self.queued_ticks.insert((tick.position, tick.value)) {
            let index = (offset + tick.delay as usize) % MAX_TICK_DELAY;
            self.tick_queue[index].push(OrderedTick {
                priority: tick.priority,
                sub_tick_order,
                position: tick.position,
                value: tick.value,
            });
        }
    }
}

impl<T> ChunkTickScheduler<T> {
    fn lock_inner(&self) -> MutexGuard<'_, Option<Box<ChunkTickSchedulerInner<T>>>> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Caller holds `lock_inner`.
    fn ring_offset(&self) -> usize {
        self.offset.load(Ordering::Relaxed) % MAX_TICK_DELAY
    }

    /// Caller holds `lock_inner`. Returns the slot to drain, then advances.
    fn advance_ring_offset(&self) -> usize {
        let current = self.ring_offset();
        self.offset
            .store((current + 1) % MAX_TICK_DELAY, Ordering::Relaxed);
        current
    }
}

impl<'a, T: std::hash::Hash + Eq> ChunkTickScheduler<&'a T> {
    pub fn step_tick(&self) -> Vec<OrderedTick<&'a T>> {
        let mut inner_guard = self.lock_inner();
        let current_offset = self.advance_ring_offset();

        let Some(inner) = inner_guard.as_mut() else {
            return Vec::new();
        };

        let res = std::mem::take(&mut inner.tick_queue[current_offset]);

        if !res.is_empty() {
            for next_tick in &res {
                inner
                    .queued_ticks
                    .remove(&(next_tick.position, next_tick.value));
            }
            if inner.queued_ticks.is_empty() {
                *inner_guard = None;
            }
        }
        res
    }

    pub fn schedule_tick(&self, tick: &ScheduledTick<&'a T>, sub_tick_order: i64) {
        let mut inner_guard = self.lock_inner();
        let offset = self.ring_offset();
        inner_guard
            .get_or_insert_with(ChunkTickSchedulerInner::boxed)
            .schedule(offset, tick, sub_tick_order);
    }

    pub fn is_scheduled(&self, pos: BlockPos, value: &T) -> bool {
        self.lock_inner()
            .as_ref()
            .is_some_and(|inner| inner.queued_ticks.contains(&(pos, value)))
    }

    /// Drops the pending `(pos, type)` tick so `step_tick` will not run it.
    #[must_use]
    pub fn cancel_tick(&self, pos: BlockPos, value: &T) -> bool {
        let mut inner_guard = self.lock_inner();
        let Some(inner) = inner_guard.as_mut() else {
            return false;
        };
        let before = inner.queued_ticks.len();
        inner
            .queued_ticks
            .retain(|(p, v)| *p != pos || **v != *value);
        if inner.queued_ticks.len() == before {
            return false;
        }
        for bucket in &mut inner.tick_queue {
            if let Some(i) = bucket
                .iter()
                .position(|tick| tick.position == pos && *tick.value == *value)
            {
                bucket.swap_remove(i);
                break;
            }
        }
        if inner.queued_ticks.is_empty() {
            *inner_guard = None;
        }
        true
    }

    pub fn has_ticks(&self) -> bool {
        self.lock_inner()
            .as_ref()
            .is_some_and(|inner| !inner.queued_ticks.is_empty())
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<ScheduledTick<&'a T>> {
        let inner_guard = self.lock_inner();
        let Some(inner) = inner_guard.as_ref() else {
            return Vec::new();
        };
        let offset = self.ring_offset();

        let mut res = Vec::new();

        for i in 0..MAX_TICK_DELAY {
            let index = (offset + i) % MAX_TICK_DELAY;
            // NBT has no `subTickOrder`; `from_iter` reassigns by list index, so emit
            // `(priority, sub_tick_order)` order, not concurrent push order.
            let mut bucket: Vec<&OrderedTick<&'a T>> = inner.tick_queue[index].iter().collect();
            bucket.sort();
            res.extend(bucket.into_iter().map(|x| ScheduledTick {
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
        let ticks: Vec<ScheduledTick<&'a T>> = iter.into_iter().collect();
        if ticks.is_empty() {
            return scheduler;
        }

        // Vanilla `LevelChunkTicks.unpack`: `i = -pendingTicks.size()`, then `i++`.
        // Negative `subTickOrder` sorts ahead of live `LevelTicks.nextSubTickCount`.
        let first_order = -(i64::try_from(ticks.len()).unwrap_or(i64::MAX));
        {
            let mut inner_guard = scheduler.lock_inner();
            let inner = inner_guard.get_or_insert_with(ChunkTickSchedulerInner::boxed);
            inner.queued_ticks.reserve(ticks.len());
            let offset = scheduler.ring_offset();
            for (sub_tick_order, tick) in (first_order..).zip(ticks.iter()) {
                inner.schedule(offset, tick, sub_tick_order);
            }
        }
        scheduler
    }
}

impl<T> Default for ChunkTickScheduler<T> {
    fn default() -> Self {
        Self {
            inner: Mutex::new(None),
            offset: AtomicUsize::new(0),
        }
    }
}
