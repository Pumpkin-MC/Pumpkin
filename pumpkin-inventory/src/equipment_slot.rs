use std::borrow::Cow;

#[derive(Debug, Clone, Copy)]
pub enum EquipmentType {
    Hand,
    HumanoidArmor,
    AnimalArmor,
    Saddle,
}

#[derive(Debug, Clone)]
pub struct EquipmentSlot {
    pub slot_type: EquipmentType,
    pub entity_id: i32,
    pub max_count: i32,
    pub index: i32,
    pub name: Cow<'static, str>,
}

impl EquipmentSlot {
    pub const MAIN_HAND: Self = Self {
        slot_type: EquipmentType::Hand,
        entity_id: 0,
        index: 0,
        max_count: 0,
        name: Cow::Borrowed("mainhand"),
    };
    pub const OFF_HAND: Self = Self {
        slot_type: EquipmentType::Hand,
        entity_id: 1,
        index: 5,
        max_count: 0,
        name: Cow::Borrowed("offhand"),
    };
    pub const FEET: Self = Self {
        slot_type: EquipmentType::HumanoidArmor,
        entity_id: 0,
        index: 1,
        max_count: 1,
        name: Cow::Borrowed("feet"),
    };
    pub const LEGS: Self = Self {
        slot_type: EquipmentType::HumanoidArmor,
        entity_id: 1,
        index: 2,
        max_count: 1,
        name: Cow::Borrowed("legs"),
    };
    pub const CHEST: Self = Self {
        slot_type: EquipmentType::HumanoidArmor,
        entity_id: 2,
        index: 3,
        max_count: 1,
        name: Cow::Borrowed("chest"),
    };
    pub const HEAD: Self = Self {
        slot_type: EquipmentType::HumanoidArmor,
        entity_id: 3,
        index: 4,
        max_count: 1,
        name: Cow::Borrowed("head"),
    };
    pub const BODY: Self = Self {
        slot_type: EquipmentType::AnimalArmor,
        entity_id: 0,
        index: 6,
        max_count: 1,
        name: Cow::Borrowed("body"),
    };
    pub const SADDLE: Self = Self {
        slot_type: EquipmentType::Saddle,
        entity_id: 0,
        index: 7,
        max_count: 1,
        name: Cow::Borrowed("saddle"),
    };

    pub fn get_entity_slot_id(&self) -> i32 {
        self.entity_id
    }

    pub fn get_offset_entity_slot_id(&self, offset: i32) -> i32 {
        self.entity_id + offset
    }

    pub fn is_armor_slot(&self) -> bool {
        matches!(
            self.slot_type,
            EquipmentType::HumanoidArmor | EquipmentType::AnimalArmor
        )
    }
}
