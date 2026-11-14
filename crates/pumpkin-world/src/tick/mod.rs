use std::{cmp::Reverse, collections::BinaryHeap, str::FromStr};

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    math::position::BlockPos,
    resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation},
};

pub mod order;
pub mod scheduler;

const MAX_TICK_DELAY: usize = 1 << 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
#[repr(i32)]
pub enum TickPriority {
    ExtremelyHigh = -3,
    VeryHigh = -2,
    High = -1,
    Normal = 0,
    Low = 1,
    VeryLow = 2,
    ExtremelyLow = 3,
}

impl TickPriority {
    #[must_use]
    pub const fn values() -> [Self; 7] {
        [
            Self::ExtremelyHigh,
            Self::VeryHigh,
            Self::High,
            Self::Normal,
            Self::Low,
            Self::VeryLow,
            Self::ExtremelyLow,
        ]
    }
}

#[derive(Debug)]
pub struct TickPriorityNotFound;

impl TryFrom<i32> for TickPriority {
    type Error = TickPriorityNotFound;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            -3 => Ok(Self::ExtremelyHigh),
            -2 => Ok(Self::VeryHigh),
            -1 => Ok(Self::High),
            0 => Ok(Self::Normal),
            1 => Ok(Self::Low),
            2 => Ok(Self::VeryLow),
            3 => Ok(Self::ExtremelyLow),
            _ => Err(TickPriorityNotFound),
        }
    }
}

#[derive(Clone)]
pub struct ScheduledTick<T> {
    pub delay: u8,
    pub priority: TickPriority,
    pub position: BlockPos,
    pub value: T,
}

#[derive(Clone)]
pub struct OrderedTick<T> {
    pub priority: TickPriority,
    pub sub_tick_order: i64,

    pub position: BlockPos,
    pub value: T,
}

impl<T> OrderedTick<T> {
    pub const fn new(position: BlockPos, value: T) -> Self {
        Self {
            priority: TickPriority::Normal,
            sub_tick_order: 0,
            position,
            value,
        }
    }
}

impl<T> PartialEq for OrderedTick<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sub_tick_order == other.sub_tick_order
    }
}

impl<T> Eq for OrderedTick<T> {}

impl<T> PartialOrd for OrderedTick<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl<T> Ord for OrderedTick<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| self.sub_tick_order.cmp(&other.sub_tick_order))
    }
}

/// Drains chunk queues by comparing only their next due ticks, like vanilla's `LevelTicks`.
/// Each queue is in deadline order, with priority and sub-tick order breaking ties.
#[must_use]
pub(crate) fn drain_containers<T>(containers: Vec<Vec<OrderedTick<T>>>) -> Vec<OrderedTick<T>> {
    let mut containers: Vec<_> = containers.into_iter().map(Vec::into_iter).collect();
    let mut heads: BinaryHeap<_> = containers
        .iter_mut()
        .enumerate()
        .filter_map(|(index, ticks)| ticks.next().map(|tick| Reverse((tick, index))))
        .collect();
    let mut drained = Vec::new();
    while let Some(Reverse((tick, index))) = heads.pop() {
        drained.push(tick);
        for next in containers[index].by_ref() {
            if heads.peek().is_some_and(|Reverse((head, _))| next > *head) {
                heads.push(Reverse((next, index)));
                break;
            }
            drained.push(next);
        }
    }
    drained
}

impl<T> ScheduledTick<T>
where
    T: ToResourceLocation,
{
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.put_int("x", self.position.0.x);
        nbt.put_int("y", self.position.0.y);
        nbt.put_int("z", self.position.0.z);
        nbt.put_int("t", self.delay as i32);
        nbt.put_int("p", self.priority as i32);
        nbt.put_string("i", self.value.to_resource_location());
        nbt
    }
}

impl<T> ScheduledTick<T>
where
    T: FromResourceLocation,
{
    #[must_use]
    pub fn from_nbt_compound(nbt: &NbtCompound) -> Option<Self> {
        let x = nbt.get_int("x")?;
        let y = nbt.get_int("y")?;
        let z = nbt.get_int("z")?;
        let delay = nbt.get_int("t")? as u8;
        let priority = TickPriority::try_from(nbt.get_int("p")?).ok()?;
        let res_loc_str = nbt.get_string("i")?;
        let res_loc = ResourceLocation::from_str(res_loc_str).ok()?;
        let value = T::from_resource_location(&res_loc)?;

        Some(Self {
            delay,
            priority,
            position: BlockPos::new(x, y, z),
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_chunk_heads_continue_draining_the_current_chunk() {
        let waiting = OrderedTick::new(BlockPos::new(0, 64, 0), ());
        let mut urgent = OrderedTick::new(BlockPos::new(16, 64, 0), ());
        urgent.priority = TickPriority::High;
        let following = OrderedTick::new(BlockPos::new(17, 64, 0), ());

        let drained = drain_containers(vec![vec![waiting], vec![urgent, following]]);
        assert_eq!(
            drained
                .iter()
                .map(|tick| tick.position.0.x)
                .collect::<Vec<_>>(),
            [16, 17, 0]
        );
    }
}
