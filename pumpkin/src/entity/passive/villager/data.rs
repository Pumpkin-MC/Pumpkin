use pumpkin_data::item::Item;
pub use pumpkin_data::villager::{VillagerProfession, VillagerType};
use pumpkin_protocol::codec::var_int::VarInt;
use serde::Serialize;

pub const BREEDING_FOOD_THRESHOLD: i32 = 12;

#[must_use]
pub const fn get_food_points(item: &Item) -> i32 {
    match item.id {
        id if id == Item::BREAD.id => 4,
        id if id == Item::POTATO.id => 1,
        id if id == Item::CARROT.id => 1,
        id if id == Item::BEETROOT.id => 1,
        _ => 0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[repr(i32)]
pub enum GossipType {
    MajorNegative = 0,
    MinorNegative = 1,
    MajorPositive = 2,
    MinorPositive = 3,
    Trading = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VillagerData {
    pub r#type: VillagerType,
    pub profession: VillagerProfession,
    pub level: i32,
}

impl Serialize for VillagerData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("VillagerData", 3)?;
        state.serialize_field("type", &VarInt(self.r#type as i32))?;
        state.serialize_field("profession", &VarInt(self.profession as i32))?;
        state.serialize_field("level", &VarInt(self.level))?;
        state.end()
    }
}
