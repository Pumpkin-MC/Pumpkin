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
    overdue_ticks: Vec<OrderedTick<T>>,
}

impl<'a, T: std::hash::Hash + Eq> ChunkTickScheduler<&'a T> {
    pub fn step_tick(&self, run_ticks: bool) -> Vec<OrderedTick<&'a T>> {
        // The offset only changes under `inner`, so `schedule_tick` can't pick a slot that was
        // just drained.
        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let current_offset = self.offset.fetch_add(1, Ordering::SeqCst) % MAX_TICK_DELAY;
        let next_offset = (current_offset + 1) % MAX_TICK_DELAY;
        self.offset.store(next_offset, Ordering::SeqCst);

        let Some(inner) = inner_guard.as_mut() else {
            return Vec::new();
        };

        // Deadlines keep advancing in loaded chunks outside simulation distance.
        inner.tick_queue[current_offset].sort_unstable();
        inner
            .overdue_ticks
            .append(&mut inner.tick_queue[current_offset]);
        if !run_ticks {
            return Vec::new();
        }
        let res = std::mem::take(&mut inner.overdue_ticks);

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
        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let offset = self.offset.load(Ordering::SeqCst);
        let inner = inner_guard.get_or_insert_with(|| {
            Box::new(ChunkTickSchedulerInner {
                tick_queue: std::array::from_fn(|_| Vec::new()),
                overdue_ticks: Vec::new(),
                queued_ticks: FxHashSet::default(),
            })
        });

        if inner.queued_ticks.insert((tick.position, tick.value)) {
            // `offset` is the queue the next `step_tick` drains, so a delay of N lands N - 1 slots
            // ahead. Vanilla runs a delay 0 tick on the next tick too.
            let index = (offset + (tick.delay as usize).max(1) - 1) % MAX_TICK_DELAY;

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

        inner.overdue_ticks.retain(|tick| !contains(&tick.position));
        for queue in &mut inner.tick_queue {
            queue.retain(|tick| !contains(&tick.position));
        }
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
        let inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let offset = self.offset.load(Ordering::SeqCst);
        let Some(inner) = inner_guard.as_ref() else {
            return Vec::new();
        };

        let mut res: Vec<_> = inner
            .overdue_ticks
            .iter()
            .map(|tick| ScheduledTick {
                delay: 0,
                priority: tick.priority,
                position: tick.position,
                value: tick.value,
            })
            .collect();

        for i in 0..MAX_TICK_DELAY {
            let index = (offset + i) % MAX_TICK_DELAY;
            // Inverse of `schedule_tick`: the queue at `offset` runs next tick, i.e. delay 1.
            // The last slot is never filled because delay is a u8.
            res.extend(inner.tick_queue[index].iter().map(|x| ScheduledTick {
                delay: (i + 1) as u8,
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
        let ticks: Vec<_> = iter.into_iter().collect();

        let tick_count = ticks.len();
        if tick_count > 0 {
            let mut inner_guard = scheduler
                .inner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let inner = inner_guard.get_or_insert_with(|| {
                Box::new(ChunkTickSchedulerInner {
                    tick_queue: std::array::from_fn(|_| Vec::new()),
                    overdue_ticks: Vec::new(),
                    queued_ticks: FxHashSet::default(),
                })
            });
            inner.queued_ticks.reserve(tick_count);
        }

        // LevelChunkTicks.unpack gives saved ticks orders -N through -1, ahead of live ticks.
        let sub_tick_base = -(tick_count as u64).cast_signed();
        for (sub_tick_order, tick) in (sub_tick_base..).zip(ticks) {
            scheduler.schedule_tick(&tick, sub_tick_order);
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

    static BLOCK: u8 = 0;

    fn tick(delay: u8) -> ScheduledTick<&'static u8> {
        ScheduledTick {
            delay,
            priority: TickPriority::Normal,
            position: BlockPos::new(0, 0, 0),
            value: &BLOCK,
        }
    }

    #[test]
    fn delay_counts_game_ticks_like_vanilla() {
        // Vanilla: scheduled at game time T with delay 2, it runs at T + 2.
        let scheduler = ChunkTickScheduler::default();
        scheduler.schedule_tick(&tick(2), 0);
        assert!(scheduler.step_tick(true).is_empty());
        assert_eq!(scheduler.step_tick(true).len(), 1);
    }

    #[test]
    fn overdue_ticks_survive_wraparound_save_and_clear() {
        let scheduler = ChunkTickScheduler::default();
        let first = tick(2);
        let mut second = tick(5);
        second.position = BlockPos::new(1, 0, 0);
        scheduler.schedule_tick(&first, 0);
        scheduler.schedule_tick(&second, 1);
        for _ in 0..MAX_TICK_DELAY + 5 {
            assert!(scheduler.step_tick(false).is_empty());
        }
        assert!(scheduler.is_scheduled(first.position, first.value));
        scheduler.schedule_tick(&first, 2);
        let saved = scheduler.to_vec();
        assert_eq!(saved.len(), 2);
        assert!(saved.iter().all(|tick| tick.delay == 0));
        let restored = ChunkTickScheduler::from_iter(saved);
        assert_eq!(restored.step_tick(true).len(), 2);

        scheduler.clear_area(&first.position, &BlockPos::new(1, 1, 1));
        assert!(!scheduler.is_scheduled(first.position, first.value));
        let due = scheduler.step_tick(true);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].position, second.position);
        assert!(!scheduler.has_ticks());
    }

    #[test]
    fn saved_delay_round_trips() {
        let scheduler = ChunkTickScheduler::default();
        scheduler.schedule_tick(&tick(5), 0);
        scheduler.step_tick(true);
        assert_eq!(scheduler.to_vec()[0].delay, 4);
    }

    #[test]
    fn saved_list_order_round_trips_for_equal_priority_ticks() {
        let scheduler = ChunkTickScheduler::default();
        for (order, x) in [(0, 2), (1, 0), (2, 1)] {
            let mut pending = tick(2);
            pending.position = BlockPos::new(x, 0, 0);
            scheduler.schedule_tick(&pending, order);
        }
        let restored = ChunkTickScheduler::from_iter(scheduler.to_vec());
        assert!(restored.step_tick(true).is_empty());
        let due = restored.step_tick(true);
        assert_eq!(
            due.iter().map(|tick| tick.position.0.x).collect::<Vec<_>>(),
            [2, 0, 1]
        );
        assert!(due.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn restored_ticks_precede_live_ticks_across_chunks() {
        let restored = |positions: &[i32]| {
            // Count the saved list even when the iterator has no exact size hint.
            let scheduler = ChunkTickScheduler::from_iter(
                positions
                    .iter()
                    .map(|&x| {
                        let mut pending = tick(1);
                        pending.position = BlockPos::new(x, 0, 0);
                        pending
                    })
                    .filter(|_| true),
            );
            scheduler.step_tick(true)
        };
        let live = ChunkTickScheduler::default();
        let mut pending = tick(1);
        pending.position = BlockPos::new(32, 0, 0);
        live.schedule_tick(&pending, 0);

        let due = crate::tick::drain_containers(vec![
            live.step_tick(true),
            restored(&[10]),
            restored(&[20, 21, 22]),
        ]);
        // Vanilla starts each saved list at -len, ahead of the first live order, zero.
        assert_eq!(
            due.iter().map(|tick| tick.position.0.x).collect::<Vec<_>>(),
            [20, 21, 22, 10, 32]
        );
    }
}
