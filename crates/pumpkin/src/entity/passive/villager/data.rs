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
    pub r#type: VarInt,
    pub profession: VarInt,
    pub level: VarInt,
}

impl pumpkin_protocol::java::client::play::MetadataSerializer for VillagerData {
    fn write_metadata(
        &self,
        writer: &mut impl std::io::Write,
    ) -> Result<(), pumpkin_protocol::ser::WritingError> {
        use pumpkin_protocol::ser::NetworkWriteExt;
        writer.write_var_int(&self.r#type)?;
        writer.write_var_int(&self.profession)?;
        writer.write_var_int(&self.level)
    }
}

impl VillagerData {
    #[must_use]
    pub const fn new(r#type: VillagerType, profession: VillagerProfession, level: i32) -> Self {
        Self {
            r#type: VarInt(r#type as i32),
            profession: VarInt(profession as i32),
            level: VarInt(level),
        }
    }

    #[must_use]
    pub fn type_enum(&self) -> VillagerType {
        VillagerType::from_i32(self.r#type.0).unwrap_or(VillagerType::Plains)
    }

    #[must_use]
    pub fn profession_enum(&self) -> VillagerProfession {
        VillagerProfession::from_i32(self.profession.0).unwrap_or(VillagerProfession::None)
    }
}
