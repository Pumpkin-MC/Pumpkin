use std::{borrow::Cow, collections::HashMap};
use pumpkin_world::item::ItemStack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlotType {
    Hand,
    HumanoidArmor,
    AnimalArmor,
    Saddle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Feet,
    Legs,
    Chest,
    Head,
    Body,
    Saddle,
}

impl EquipmentSlot {
    pub const NO_MAX_COUNT: i32 = 0;

    pub fn slot_type(&self) -> EquipmentSlotType {
        match self {
            EquipmentSlot::MainHand | EquipmentSlot::OffHand => EquipmentSlotType::Hand,
            EquipmentSlot::Feet | EquipmentSlot::Legs | EquipmentSlot::Chest | EquipmentSlot::Head => {
                EquipmentSlotType::HumanoidArmor
            }
            EquipmentSlot::Body => EquipmentSlotType::AnimalArmor,
            EquipmentSlot::Saddle => EquipmentSlotType::Saddle,
        }
    }

    pub fn entity_slot_id(&self) -> i32 {
        match self {
            EquipmentSlot::MainHand => 0,
            EquipmentSlot::OffHand => 1,
            EquipmentSlot::Feet => 0,
            EquipmentSlot::Legs => 1,
            EquipmentSlot::Chest => 2,
            EquipmentSlot::Head => 3,
            EquipmentSlot::Body => 0,
            EquipmentSlot::Saddle => 0,
        }
    }

    pub fn max_count(&self) -> i32 {
        match self {
            EquipmentSlot::MainHand => Self::NO_MAX_COUNT,
            EquipmentSlot::OffHand => Self::NO_MAX_COUNT,
            EquipmentSlot::Feet | EquipmentSlot::Legs | EquipmentSlot::Chest | EquipmentSlot::Head => 1,
            EquipmentSlot::Body => 1,
            EquipmentSlot::Saddle => 1,
        }
    }

    pub fn index(&self) -> i32 {
        match self {
            EquipmentSlot::MainHand => 0,
            EquipmentSlot::OffHand => 5,
            EquipmentSlot::Feet => 1,
            EquipmentSlot::Legs => 2,
            EquipmentSlot::Chest => 3,
            EquipmentSlot::Head => 4,
            EquipmentSlot::Body => 6,
            EquipmentSlot::Saddle => 7,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            EquipmentSlot::MainHand => "mainhand",
            EquipmentSlot::OffHand => "offhand",
            EquipmentSlot::Feet => "feet",
            EquipmentSlot::Legs => "legs",
            EquipmentSlot::Chest => "chest",
            EquipmentSlot::Head => "head",
            EquipmentSlot::Body => "body",
            EquipmentSlot::Saddle => "saddle",
        }
    }

    pub fn offset_entity_slot_id(&self, offset: i32) -> i32 {
        offset + self.entity_slot_id()
    }

    pub fn offset_index(&self, offset: i32) -> i32 {
        self.index() + offset
    }

    pub fn is_armor_slot(&self) -> bool {
        matches!(
            self.slot_type(),
            EquipmentSlotType::HumanoidArmor | EquipmentSlotType::AnimalArmor
        )
    }

    pub fn increases_dropped_experience(&self) -> bool {
        self.slot_type() != EquipmentSlotType::Saddle
    }

    pub fn split_item_stack<'a>(&self, stack: &'a mut ItemStack) -> Cow<'a, ItemStack> {
        let max_count = self.max_count() as u8;

        if max_count > 0 {
            Cow::Owned(stack.split(max_count))
        } else {
            Cow::Borrowed(stack)
        }
    }

    pub fn from_index(index: i32) -> Option<Self> {
        match index {
            0 => Some(EquipmentSlot::MainHand),
            1 => Some(EquipmentSlot::Feet),
            2 => Some(EquipmentSlot::Legs),
            3 => Some(EquipmentSlot::Chest),
            4 => Some(EquipmentSlot::Head),
            5 => Some(EquipmentSlot::OffHand),
            6 => Some(EquipmentSlot::Body),
            7 => Some(EquipmentSlot::Saddle),
            _ => None,
        }
    }

    pub fn by_name(name: &str) -> Result<Self, String> {
        match name {
            "mainhand" => Ok(EquipmentSlot::MainHand),
            "offhand" => Ok(EquipmentSlot::OffHand),
            "feet" => Ok(EquipmentSlot::Feet),
            "legs" => Ok(EquipmentSlot::Legs),
            "chest" => Ok(EquipmentSlot::Chest),
            "head" => Ok(EquipmentSlot::Head),
            "body" => Ok(EquipmentSlot::Body),
            "saddle" => Ok(EquipmentSlot::Saddle),
            _ => Err(format!("Invalid slot '{}'", name)),
        }
    }

    pub fn all_values() -> &'static [EquipmentSlot] {
        &[
            EquipmentSlot::MainHand,
            EquipmentSlot::OffHand,
            EquipmentSlot::Feet,
            EquipmentSlot::Legs,
            EquipmentSlot::Chest,
            EquipmentSlot::Head,
            EquipmentSlot::Body,
            EquipmentSlot::Saddle,
        ]
    }
}

/// Equipment holder trait for entities that can have equipment (like a armorstand)
pub trait HasEquipment {
    fn get_equipment(&self, slot: EquipmentSlot) -> Option<&ItemStack>;
    fn set_equipment(&mut self, slot: EquipmentSlot, item: Option<ItemStack>);
    fn get_all_equipment(&self) -> HashMap<EquipmentSlot, ItemStack>;
}
