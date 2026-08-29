use std::str::FromStr;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    math::position::BlockPos,
    resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation},
};

pub mod scheduler;

/// Pumpkin `ChunkTickScheduler` ring width.
///
/// Vanilla `ScheduledTick.triggerTick` is an unbounded `long`; remaining delay here is `u8`
/// (`SavedTick.delay` NBT `t` clamps to this).
const MAX_TICK_DELAY: usize = 1 << 8;

/// Vanilla `TickPriority`. NBT field `p`.
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

    #[must_use]
    pub const fn from_i32(value: i32) -> Option<Self> {
        match value {
            -3 => Some(Self::ExtremelyHigh),
            -2 => Some(Self::VeryHigh),
            -1 => Some(Self::High),
            0 => Some(Self::Normal),
            1 => Some(Self::Low),
            2 => Some(Self::VeryLow),
            3 => Some(Self::ExtremelyLow),
            _ => None,
        }
    }

    /// Vanilla `TickPriority.byValue`: out-of-range `p` clamps to a neighbor, not drop.
    #[must_use]
    pub const fn by_value(value: i32) -> Self {
        match Self::from_i32(value) {
            Some(priority) => priority,
            None if value < Self::ExtremelyHigh as i32 => Self::ExtremelyHigh,
            None => Self::ExtremelyLow,
        }
    }
}

#[derive(Debug)]
pub struct TickPriorityNotFound;

impl TryFrom<i32> for TickPriority {
    type Error = TickPriorityNotFound;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_i32(value).ok_or(TickPriorityNotFound)
    }
}

/// Vanilla `SavedTick`: pending `(delay, priority, pos, type)` before `unpack`.
///
/// Live due-this-tick entries are `OrderedTick` (`ScheduledTick` without `triggerTick`).
#[derive(Clone)]
pub struct ScheduledTick<T> {
    /// Remaining ticks until due. Vanilla `SavedTick.delay` / NBT `t`; ring-clamped to `u8`.
    pub delay: u8,
    pub priority: TickPriority,
    pub position: BlockPos,
    pub value: T,
}

/// Vanilla `ScheduledTick` minus `triggerTick` (the ring slot is that).
///
/// `Ord` is vanilla `INTRA_TICK_DRAIN_ORDER` (`priority`, then `subTickOrder`); pos/type are ignored.
#[derive(Clone)]
pub struct OrderedTick<T> {
    pub priority: TickPriority,
    /// Vanilla `ScheduledTick.subTickOrder`. Live: `LevelTicks.nextSubTickCount` (0..).
    /// Restored NBT: negative, `LevelChunkTicks.unpack` / `ChunkTickScheduler::from_iter`.
    pub sub_tick_order: i64,

    pub position: BlockPos,
    pub value: T,
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

impl<T> ScheduledTick<T>
where
    T: ToResourceLocation,
{
    /// Vanilla `SavedTick` NBT: `x`/`y`/`z`, `t` delay, `p` priority, `i` type id.
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.put_int("x", self.position.0.x);
        nbt.put_int("y", self.position.0.y);
        nbt.put_int("z", self.position.0.z);
        nbt.put_int("t", i32::from(self.delay));
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
        // Overdue (`t` < 0) fires next `step_tick`. `t` > 255 would wrap to delay 0 as `u8`.
        let delay = nbt.get_int("t")?.clamp(0, (MAX_TICK_DELAY - 1) as i32) as u8;
        let priority = TickPriority::by_value(nbt.get_int("p")?);
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
