use std::collections::HashSet;

use pumpkin_util::math::position::BlockPos;

use crate::tick::{MAX_TICK_DELAY, OrderedTick, ScheduledTick};

#[derive(Debug, Clone)]
pub struct ChunkTickScheduler<T> {
    tick_queue: [Vec<OrderedTick<T>>; MAX_TICK_DELAY],
    queued_ticks: HashSet<(BlockPos, T)>,
    offset: usize,
}

impl<'a, T: std::hash::Hash + Eq> ChunkTickScheduler<&'a T> {
    pub fn step_tick(&mut self) -> Vec<OrderedTick<&'a T>> {
        self.offset += 1;
        self.offset %= MAX_TICK_DELAY;
        let mut res = Vec::new();
        std::mem::swap(&mut res, &mut self.tick_queue[self.offset]);
        for next_tick in &res {
            self.queued_ticks
                .remove(&(next_tick.position, next_tick.value));
        }
        res
    }

    pub fn schedule_tick(&mut self, tick: ScheduledTick<&'a T>, sub_tick_order: u64) {
        if self.queued_ticks.insert((tick.position, tick.value)) {
            let index = (self.offset + tick.delay as usize) % MAX_TICK_DELAY;
            self.tick_queue[index].push(OrderedTick {
                priority: tick.priority,
                sub_tick_order,
                position: tick.position,
                value: tick.value,
            });
        }
    }

    pub fn is_scheduled(&self, pos: BlockPos, value: &'a T) -> bool {
        self.queued_ticks.contains(&(pos, value))
    }
}

impl<T> Default for ChunkTickScheduler<T> {
    fn default() -> Self {
        Self {
            tick_queue: std::array::from_fn(|_| Vec::new()),
            queued_ticks: HashSet::new(),

            offset: 0,
        }
    }
}
