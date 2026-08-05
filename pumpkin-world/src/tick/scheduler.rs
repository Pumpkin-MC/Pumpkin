use std::sync::{
    Mutex,
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
    inflight_ticks: FxHashSet<(BlockPos, T)>,
}

impl<'a, T: std::hash::Hash + Eq> ChunkTickScheduler<&'a T> {
    pub fn step_tick(&self) -> Vec<OrderedTick<&'a T>> {
        // Atomic update for the offset
        let current_offset = self.offset.fetch_add(1, Ordering::SeqCst) % MAX_TICK_DELAY;
        let next_offset = (current_offset + 1) % MAX_TICK_DELAY;
        self.offset.store(next_offset, Ordering::SeqCst);

        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(inner) = inner_guard.as_mut() else {
            return Vec::new();
        };

        let res = std::mem::take(&mut inner.tick_queue[current_offset]);

        if !res.is_empty() {
            for next_tick in &res {
                inner
                    .queued_ticks
                    .remove(&(next_tick.position, next_tick.value));
                inner
                    .inflight_ticks
                    .insert((next_tick.position, next_tick.value));
            }
        }
        res
    }

    pub fn schedule_tick(&self, tick: &ScheduledTick<&'a T>, sub_tick_order: u64) {
        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let inner = inner_guard.get_or_insert_with(|| {
            Box::new(ChunkTickSchedulerInner {
                tick_queue: std::array::from_fn(|_| Vec::new()),
                queued_ticks: FxHashSet::default(),
                inflight_ticks: FxHashSet::default(),
            })
        });

        if inner.queued_ticks.insert((tick.position, tick.value)) {
            let offset = self.offset.load(Ordering::SeqCst);
            let index = (offset + tick.delay as usize) % MAX_TICK_DELAY;

            inner.tick_queue[index].push(OrderedTick {
                priority: tick.priority,
                sub_tick_order,
                position: tick.position,
                value: tick.value,
            });
        }
    }

    pub fn is_scheduled(&self, pos: BlockPos, value: &T) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|inner| {
                inner.queued_ticks.contains(&(pos, value))
                    || inner.inflight_ticks.contains(&(pos, value))
            })
    }

    pub fn has_ticks(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|inner| !inner.queued_ticks.is_empty() || !inner.inflight_ticks.is_empty())
    }

    pub fn clear_inflight(&self) {
        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(inner) = inner_guard.as_mut() {
            inner.inflight_ticks.clear();
            if inner.queued_ticks.is_empty() && inner.inflight_ticks.is_empty() {
                *inner_guard = None;
            }
        }
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<ScheduledTick<&'a T>> {
        let offset = self.offset.load(Ordering::SeqCst);
        let inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(inner) = inner_guard.as_ref() else {
            return Vec::new();
        };

        let mut res = Vec::new();

        for i in 0..MAX_TICK_DELAY {
            let index = (offset + i) % MAX_TICK_DELAY;
            res.extend(inner.tick_queue[index].iter().map(|x| ScheduledTick {
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
            let mut inner_guard = scheduler
                .inner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let inner = inner_guard.get_or_insert_with(|| {
                Box::new(ChunkTickSchedulerInner {
                    tick_queue: std::array::from_fn(|_| Vec::new()),
                    queued_ticks: FxHashSet::default(),
                    inflight_ticks: FxHashSet::default(),
                })
            });
            inner.queued_ticks.reserve(lower);
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
            inner: Mutex::new(None),
            offset: AtomicUsize::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tick::TickPriority;
    use pumpkin_util::math::position::BlockPos;

    static VALUE: u8 = 0;

    #[test]
    fn inflight_tick_still_appears_scheduled() {
        let scheduler: ChunkTickScheduler<&'static u8> = ChunkTickScheduler::default();
        let pos = BlockPos::new(0, 0, 0);

        scheduler.schedule_tick(
            &ScheduledTick {
                delay: 0,
                priority: TickPriority::Normal,
                position: pos,
                value: &VALUE,
            },
            0,
        );

        assert!(scheduler.is_scheduled(pos, &VALUE));

        let ticks = scheduler.step_tick();
        assert_eq!(ticks.len(), 1);
        assert_eq!(ticks[0].position, pos);

        // Tick is in-flight — is_scheduled must still return true
        assert!(scheduler.is_scheduled(pos, &VALUE));

        // Scheduling a fresh tick for the same position must succeed
        // while the old one is in-flight
        scheduler.schedule_tick(
            &ScheduledTick {
                delay: 5,
                priority: TickPriority::Normal,
                position: pos,
                value: &VALUE,
            },
            1,
        );

        scheduler.clear_inflight();

        // After clear, in-flight is gone but the fresh queued tick remains
        assert!(scheduler.is_scheduled(pos, &VALUE));
        assert!(scheduler.has_ticks());

        // Step again to retrieve the fresh tick
        for _ in 0..5 {
            scheduler.step_tick();
        }
        let ticks = scheduler.step_tick();
        assert_eq!(ticks.len(), 1);
        assert_eq!(ticks[0].position, pos);
    }

    #[test]
    fn clear_inflight_drops_inner_when_empty() {
        let scheduler: ChunkTickScheduler<&'static u8> = ChunkTickScheduler::default();
        let pos = BlockPos::new(0, 0, 0);

        scheduler.schedule_tick(
            &ScheduledTick {
                delay: 0,
                priority: TickPriority::Normal,
                position: pos,
                value: &VALUE,
            },
            0,
        );

        let _ticks = scheduler.step_tick();
        scheduler.clear_inflight();

        assert!(!scheduler.has_ticks());
    }
}
