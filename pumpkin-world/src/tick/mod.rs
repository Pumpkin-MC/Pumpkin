use std::str::FromStr;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    math::position::BlockPos,
    resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation},
};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    pub sub_tick_order: u64,

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

impl<T> Serialize for ScheduledTick<T>
where
    T: ToResourceLocation,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut nbt = NbtCompound::new();
        nbt.put_int("x", self.position.0.x);
        nbt.put_int("y", self.position.0.y);
        nbt.put_int("z", self.position.0.z);
        nbt.put_int("t", self.delay as i32);
        nbt.put_int("p", self.priority as i32);
        nbt.put_string("i", self.value.to_resource_location());
        nbt.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for ScheduledTick<T>
where
    T: FromResourceLocation,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nbt = NbtCompound::deserialize(deserializer)?;

        let get_int = |key| nbt.get_int(key).ok_or_else(|| D::Error::missing_field(key));
        let get_str = |key| {
            nbt.get_string(key)
                .ok_or_else(|| D::Error::missing_field(key))
        };

        let x = get_int("x")?;
        let y = get_int("y")?;
        let z = get_int("z")?;

        let delay = get_int("t")? as u8;

        let priority = TickPriority::try_from(get_int("p")?)
            .map_err(|_| D::Error::custom("Invalid tick priority"))?;

        let res_loc_str = get_str("i")?;
        let res_loc = ResourceLocation::from_str(res_loc_str).map_err(|e| {
            D::Error::custom(format!("Invalid ResourceLocation '{res_loc_str}': {e}"))
        })?;

        let value = T::from_resource_location(&res_loc)
            .ok_or_else(|| D::Error::custom(format!("Unknown tick type: {res_loc_str}")))?;

        Ok(Self {
            delay,
            priority,
            position: BlockPos::new(x, y, z),
            value,
        })
    }
}
