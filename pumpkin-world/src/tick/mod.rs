use std::str::FromStr;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    math::position::BlockPos,
    resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation},
};
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
    pub fn values() -> [TickPriority; 7] {
        [
            TickPriority::ExtremelyHigh,
            TickPriority::VeryHigh,
            TickPriority::High,
            TickPriority::Normal,
            TickPriority::Low,
            TickPriority::VeryLow,
            TickPriority::ExtremelyLow,
        ]
    }
}

impl From<i32> for TickPriority {
    fn from(value: i32) -> Self {
        match value {
            -3 => TickPriority::ExtremelyHigh,
            -2 => TickPriority::VeryHigh,
            -1 => TickPriority::High,
            0 => TickPriority::Normal,
            1 => TickPriority::Low,
            2 => TickPriority::VeryLow,
            3 => TickPriority::ExtremelyLow,
            _ => panic!("Invalid tick priority: {value}"),
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
    pub fn new(position: BlockPos, value: T) -> Self {
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
        nbt.put_string("i", self.value.to_resource_location().to_string());
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
        NbtCompound::deserialize(deserializer).map(|nbt| {
            let x = nbt.get_int("x").unwrap();
            let y = nbt.get_int("y").unwrap();
            let z = nbt.get_int("z").unwrap();
            let delay = nbt.get_int("t").unwrap() as u8;
            let priority = TickPriority::from(nbt.get_int("p").unwrap());
            let value = T::from_resource_location(
                &ResourceLocation::from_str(nbt.get_string("i").unwrap()).unwrap(),
            )
            .unwrap();

            ScheduledTick {
                delay,
                priority,
                position: BlockPos::new(x, y, z),
                value,
            }
        })
    }
}
