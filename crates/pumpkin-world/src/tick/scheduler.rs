use std::collections::BTreeMap;
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashSet;

use crate::tick::{OrderedTick, ScheduledTick, TICK_QUEUE_SIZE};

pub struct ChunkTickScheduler<T> {
    inner: Mutex<Option<Box<ChunkTickSchedulerInner<T>>>>,
    /// Number of steps this chunk has taken; `offset % TICK_QUEUE_SIZE` is the next queue slot.
    offset: AtomicUsize,
}

struct ChunkTickSchedulerInner<T> {
    tick_queue: [Vec<OrderedTick<T>>; TICK_QUEUE_SIZE],
    /// Ticks too far ahead for `tick_queue`, keyed by the step they are due on.
    far_ticks: BTreeMap<usize, Vec<OrderedTick<T>>>,
    queued_ticks: FxHashSet<(BlockPos, T)>,
}

impl<T> ChunkTickSchedulerInner<T> {
    fn new() -> Self {
        Self {
            tick_queue: std::array::from_fn(|_| Vec::new()),
            far_ticks: BTreeMap::new(),
            queued_ticks: FxHashSet::default(),
        }
    }
}

impl<'a, T: std::hash::Hash + Eq> ChunkTickScheduler<&'a T> {
    pub fn step_tick(&self) -> Vec<OrderedTick<&'a T>> {
        let current_step = self.offset.fetch_add(1, Ordering::SeqCst);

        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(inner) = inner_guard.as_mut() else {
            return Vec::new();
        };

        while let Some(entry) = inner.far_ticks.first_entry()
            && *entry.key() < current_step + TICK_QUEUE_SIZE
        {
            let due_step = *entry.key();
            inner.tick_queue[due_step % TICK_QUEUE_SIZE].append(&mut entry.remove());
        }

        let res = std::mem::take(&mut inner.tick_queue[current_step % TICK_QUEUE_SIZE]);

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

    pub fn schedule_tick(&self, tick: &ScheduledTick<&'a T>, sub_tick_order: u64) {
        let offset = self.offset.load(Ordering::SeqCst);
        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let inner = inner_guard.get_or_insert_with(|| Box::new(ChunkTickSchedulerInner::new()));

        if inner.queued_ticks.insert((tick.position, tick.value)) {
            let due_step = offset + tick.delay as usize;
            let ordered_tick = OrderedTick {
                priority: tick.priority,
                sub_tick_order,
                position: tick.position,
                value: tick.value,
            };

            if (tick.delay as usize) < TICK_QUEUE_SIZE {
                inner.tick_queue[due_step % TICK_QUEUE_SIZE].push(ordered_tick);
            } else {
                inner
                    .far_ticks
                    .entry(due_step)
                    .or_default()
                    .push(ordered_tick);
            }
        }
    }

    pub fn is_scheduled(&self, pos: BlockPos, value: &T) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|inner| inner.queued_ticks.contains(&(pos, value)))
    }

    pub fn clear_area(&self, min: &BlockPos, max: &BlockPos) {
        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(inner) = inner_guard.as_mut() else {
            return;
        };

        let contains = |position: &BlockPos| {
            position.0.x >= min.0.x
                && position.0.x < max.0.x
                && position.0.y >= min.0.y
                && position.0.y < max.0.y
                && position.0.z >= min.0.z
                && position.0.z < max.0.z
        };

        for queue in &mut inner.tick_queue {
            queue.retain(|tick| !contains(&tick.position));
        }
        inner.far_ticks.retain(|_, queue| {
            queue.retain(|tick| !contains(&tick.position));
            !queue.is_empty()
        });
        inner
            .queued_ticks
            .retain(|(position, _)| !contains(position));
        let became_empty = inner.queued_ticks.is_empty();

        if became_empty {
            *inner_guard = None;
        }
    }

    pub fn has_ticks(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|inner| !inner.queued_ticks.is_empty())
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

        for i in 0..TICK_QUEUE_SIZE {
            let index = (offset + i) % TICK_QUEUE_SIZE;
            res.extend(inner.tick_queue[index].iter().map(|x| ScheduledTick {
                delay: i as u32,
                priority: x.priority,
                position: x.position,
                value: x.value,
            }));
        }
        for (due_step, queue) in &inner.far_ticks {
            res.extend(queue.iter().map(|x| ScheduledTick {
                delay: (due_step - offset) as u32,
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
            let inner = inner_guard.get_or_insert_with(|| Box::new(ChunkTickSchedulerInner::new()));
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

    const VALUE: &u8 = &0;

    fn tick_at(x: i32, delay: u32) -> ScheduledTick<&'static u8> {
        ScheduledTick {
            delay,
            priority: TickPriority::Normal,
            position: BlockPos::new(x, 0, 0),
            value: VALUE,
        }
    }

    /// Steps the scheduler and returns the 1-based step each position's tick fired on.
    fn fired_steps(scheduler: &ChunkTickScheduler<&'static u8>, steps: usize) -> Vec<(i32, usize)> {
        (1..=steps)
            .flat_map(|step| {
                scheduler
                    .step_tick()
                    .into_iter()
                    .map(move |tick| (tick.position.0.x, step))
            })
            .collect()
    }

    #[test]
    fn ticks_fire_after_their_delay_on_both_sides_of_the_queue_size() {
        let scheduler = ChunkTickScheduler::default();
        let delays = [0, 255, 256, 3600];
        for (x, delay) in (0..).zip(delays) {
            scheduler.schedule_tick(&tick_at(x, delay), 0);
        }

        let fired = fired_steps(&scheduler, 3601);
        let expected: Vec<_> = (0..).zip(delays.map(|delay| delay as usize + 1)).collect();
        assert_eq!(fired, expected);
        assert!(!scheduler.has_ticks());
    }

    #[test]
    fn far_ticks_keep_their_remaining_delay_when_saved() {
        let scheduler = ChunkTickScheduler::default();
        scheduler.schedule_tick(&tick_at(0, 3600), 0);
        assert!(fired_steps(&scheduler, 100).is_empty());

        let saved = scheduler.to_vec();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].delay, 3500);

        let loaded: ChunkTickScheduler<_> = saved.into_iter().collect();
        assert_eq!(fired_steps(&loaded, 3501), vec![(0, 3501)]);
    }
}
