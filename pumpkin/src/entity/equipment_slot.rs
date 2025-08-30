use pumpkin_world::item::ItemStack;
use std::{borrow::Cow, collections::HashMap};

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

    #[must_use]
    pub fn slot_type(&self) -> EquipmentSlotType {
        match self {
            Self::MainHand | Self::OffHand => EquipmentSlotType::Hand,
            Self::Feet | Self::Legs | Self::Chest | Self::Head => EquipmentSlotType::HumanoidArmor,
            Self::Body => EquipmentSlotType::AnimalArmor,
            Self::Saddle => EquipmentSlotType::Saddle,
        }
    }

    #[must_use]
    pub fn entity_slot_id(&self) -> i32 {
        match self {
            Self::MainHand | Self::Feet | Self::Body | Self::Saddle => 0,
            Self::OffHand | Self::Legs => 1,
            Self::Chest => 2,
            Self::Head => 3,
        }
    }

    #[must_use]
    pub fn max_count(&self) -> i32 {
        match self {
            Self::OffHand | Self::MainHand => Self::NO_MAX_COUNT,
            Self::Feet | Self::Legs | Self::Chest | Self::Head | Self::Body | Self::Saddle => 1,
        }
    }

    #[must_use]
    pub fn index(&self) -> i32 {
        match self {
            Self::MainHand => 0,
            Self::OffHand => 5,
            Self::Feet => 1,
            Self::Legs => 2,
            Self::Chest => 3,
            Self::Head => 4,
            Self::Body => 6,
            Self::Saddle => 7,
        }
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::MainHand => "mainhand",
            Self::OffHand => "offhand",
            Self::Feet => "feet",
            Self::Legs => "legs",
            Self::Chest => "chest",
            Self::Head => "head",
            Self::Body => "body",
            Self::Saddle => "saddle",
        }
    }

    #[must_use]
    pub fn offset_entity_slot_id(&self, offset: i32) -> i32 {
        offset + self.entity_slot_id()
    }

    #[must_use]
    pub fn offset_index(&self, offset: i32) -> i32 {
        self.index() + offset
    }

    #[must_use]
    pub fn is_armor_slot(&self) -> bool {
        matches!(
            self.slot_type(),
            EquipmentSlotType::HumanoidArmor | EquipmentSlotType::AnimalArmor
        )
    }

    #[must_use]
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

    #[must_use]
    pub fn from_index(index: i32) -> Option<Self> {
        match index {
            0 => Some(Self::MainHand),
            1 => Some(Self::Feet),
            2 => Some(Self::Legs),
            3 => Some(Self::Chest),
            4 => Some(Self::Head),
            5 => Some(Self::OffHand),
            6 => Some(Self::Body),
            7 => Some(Self::Saddle),
            _ => None,
        }
    }

    pub fn by_name(name: &str) -> Result<Self, String> {
        match name {
            "mainhand" => Ok(Self::MainHand),
            "offhand" => Ok(Self::OffHand),
            "feet" => Ok(Self::Feet),
            "legs" => Ok(Self::Legs),
            "chest" => Ok(Self::Chest),
            "head" => Ok(Self::Head),
            "body" => Ok(Self::Body),
            "saddle" => Ok(Self::Saddle),
            _ => Err(format!("Invalid slot '{name}'")),
        }
    }

    #[must_use]
    pub fn all_values() -> &'static [Self] {
        &[
            Self::MainHand,
            Self::OffHand,
            Self::Feet,
            Self::Legs,
            Self::Chest,
            Self::Head,
            Self::Body,
            Self::Saddle,
        ]
    }
}

/// Equipment holder trait for entities that can have equipment (like a armorstand)
pub trait HasEquipment {
    fn get_equipment(&self, slot: EquipmentSlot) -> Option<&ItemStack>;
    fn set_equipment(&mut self, slot: EquipmentSlot, item: Option<ItemStack>);
    fn get_all_equipment(&self) -> HashMap<EquipmentSlot, ItemStack>;
}
