use std::{
    collections::BTreeMap,
    sync::{
        Mutex,
        atomic::{AtomicI64, Ordering},
    },
};

use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashSet;

use crate::tick::{OrderedTick, ScheduledTick, TickPriority};

/// Per-chunk scheduled-tick queue.
///
/// Entries are keyed by their absolute trigger tick instead of a fixed-size timing wheel. This
/// mirrors vanilla's long trigger time and prevents chunk NBT delays greater than 255 from
/// wrapping into an earlier slot.
pub struct ChunkTickScheduler<T> {
    inner: Mutex<Option<Box<ChunkTickSchedulerInner<T>>>>,
    current_tick: AtomicI64,
}

struct ChunkTickSchedulerInner<T> {
    tick_queue: BTreeMap<(i64, TickPriority, u64), OrderedTick<T>>,
    queued_ticks: FxHashSet<(BlockPos, T)>,
}

impl<'a, T: std::hash::Hash + Eq> ChunkTickScheduler<&'a T> {
    /// Advances this chunk's scheduler and returns every tick due at this world tick in vanilla
    /// priority/sub-tick order.
    pub fn step_tick(&self) -> Vec<OrderedTick<&'a T>> {
        let current_tick = self.current_tick.fetch_add(1, Ordering::SeqCst) + 1;

        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(inner) = inner_guard.as_mut() else {
            return Vec::new();
        };

        let future_ticks = inner.tick_queue.split_off(&(
            current_tick.saturating_add(1),
            TickPriority::ExtremelyHigh,
            0,
        ));
        let due_ticks = std::mem::replace(&mut inner.tick_queue, future_ticks);
        let due_ticks: Vec<_> = due_ticks.into_values().collect();

        for tick in &due_ticks {
            inner.queued_ticks.remove(&(tick.position, tick.value));
        }
        if inner.queued_ticks.is_empty() {
            *inner_guard = None;
        }

        due_ticks
    }

    pub fn schedule_tick(&self, tick: &ScheduledTick<&'a T>, sub_tick_order: u64) {
        let mut inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let inner = inner_guard.get_or_insert_with(|| {
            Box::new(ChunkTickSchedulerInner {
                tick_queue: BTreeMap::new(),
                queued_ticks: FxHashSet::default(),
            })
        });

        if inner.queued_ticks.insert((tick.position, tick.value)) {
            let trigger_tick = self
                .current_tick
                .load(Ordering::SeqCst)
                .saturating_add(tick.delay);
            inner.tick_queue.insert(
                (trigger_tick, tick.priority, sub_tick_order),
                OrderedTick {
                    priority: tick.priority,
                    sub_tick_order,
                    position: tick.position,
                    value: tick.value,
                },
            );
        }
    }

    pub fn is_scheduled(&self, pos: BlockPos, value: &T) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|inner| inner.queued_ticks.contains(&(pos, value)))
    }

    pub fn has_ticks(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|inner| !inner.queued_ticks.is_empty())
    }

    /// Produces the saved-tick form in sub-tick order, matching vanilla's
    /// `LevelChunkTicks.pack`. Loading the NBT list reconstructs that same order.
    #[must_use]
    pub fn to_vec(&self) -> Vec<ScheduledTick<&'a T>> {
        let current_tick = self.current_tick.load(Ordering::SeqCst);
        let inner_guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(inner) = inner_guard.as_ref() else {
            return Vec::new();
        };

        let mut ticks: Vec<_> = inner
            .tick_queue
            .iter()
            .map(|((trigger_tick, priority, sub_tick_order), tick)| {
                (
                    *sub_tick_order,
                    ScheduledTick {
                        delay: trigger_tick.saturating_sub(current_tick),
                        priority: *priority,
                        position: tick.position,
                        value: tick.value,
                    },
                )
            })
            .collect();
        ticks.sort_unstable_by_key(|(sub_tick_order, _)| *sub_tick_order);
        ticks.into_iter().map(|(_, tick)| tick).collect()
    }
}

impl<'a, T: std::hash::Hash + Eq + 'static> FromIterator<ScheduledTick<&'a T>>
    for ChunkTickScheduler<&'a T>
{
    fn from_iter<I: IntoIterator<Item = ScheduledTick<&'a T>>>(iter: I) -> Self {
        let scheduler = Self::default();
        for (sub_tick_order, tick) in iter.into_iter().enumerate() {
            scheduler.schedule_tick(
                &tick,
                u64::try_from(sub_tick_order).expect("iterator index must fit into u64"),
            );
        }
        scheduler
    }
}

impl<T> Default for ChunkTickScheduler<T> {
    fn default() -> Self {
        Self {
            inner: Mutex::new(None),
            current_tick: AtomicI64::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::Block;
    use pumpkin_util::math::position::BlockPos;

    use super::ChunkTickScheduler;
    use crate::tick::{ScheduledTick, TickPriority};

    #[test]
    fn delay_larger_than_timing_wheel_does_not_run_early() {
        let scheduler = ChunkTickScheduler::default();
        let tick = ScheduledTick {
            delay: 300,
            priority: TickPriority::Normal,
            position: BlockPos::new(0, 0, 0),
            value: &Block::STONE,
        };
        scheduler.schedule_tick(&tick, 0);

        for _ in 0..299 {
            assert!(scheduler.step_tick().is_empty());
        }
        assert_eq!(scheduler.step_tick().len(), 1);
    }

    #[test]
    fn loaded_tick_list_order_is_preserved_for_equal_priority() {
        let ticks = [
            ScheduledTick {
                delay: 1,
                priority: TickPriority::Normal,
                position: BlockPos::new(2, 0, 0),
                value: &Block::STONE,
            },
            ScheduledTick {
                delay: 1,
                priority: TickPriority::Normal,
                position: BlockPos::new(1, 0, 0),
                value: &Block::DIRT,
            },
        ];
        let scheduler = ticks.into_iter().collect::<ChunkTickScheduler<_>>();

        let due = scheduler.step_tick();
        assert_eq!(due.len(), 2);
        assert_eq!(due[0].position, BlockPos::new(2, 0, 0));
        assert_eq!(due[1].position, BlockPos::new(1, 0, 0));
    }

    #[test]
    fn saved_ticks_retain_long_delay_and_sub_tick_order() {
        let scheduler = ChunkTickScheduler::default();
        for (sub_tick_order, position) in [BlockPos::new(1, 0, 0), BlockPos::new(2, 0, 0)]
            .into_iter()
            .enumerate()
        {
            scheduler.schedule_tick(
                &ScheduledTick {
                    delay: 300,
                    priority: TickPriority::Normal,
                    position,
                    value: &Block::STONE,
                },
                u64::try_from(sub_tick_order).expect("test index must fit into u64"),
            );
        }

        let saved = scheduler.to_vec();
        assert_eq!(
            saved.iter().map(|tick| tick.delay).collect::<Vec<_>>(),
            [300, 300]
        );
        assert_eq!(saved[0].position, BlockPos::new(1, 0, 0));
        assert_eq!(saved[1].position, BlockPos::new(2, 0, 0));
    }
}
