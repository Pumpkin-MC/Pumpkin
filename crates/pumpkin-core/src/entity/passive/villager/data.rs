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
    MinorPositive = 2,
    MajorPositive = 3,
    Trading = 4,
}

impl GossipType {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MajorNegative => "major_negative",
            Self::MinorNegative => "minor_negative",
            Self::MinorPositive => "minor_positive",
            Self::MajorPositive => "major_positive",
            Self::Trading => "trading",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "major_negative" => Some(Self::MajorNegative),
            "minor_negative" => Some(Self::MinorNegative),
            "major_positive" => Some(Self::MajorPositive),
            "minor_positive" => Some(Self::MinorPositive),
            "trading" => Some(Self::Trading),
            _ => None,
        }
    }

    #[must_use]
    pub const fn from_legacy_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::MajorNegative),
            1 => Some(Self::MinorNegative),
            2 => Some(Self::MinorPositive),
            3 => Some(Self::MajorPositive),
            4 => Some(Self::Trading),
            _ => None,
        }
    }

    #[must_use]
    pub const fn weight(self) -> i32 {
        match self {
            Self::MajorNegative => -5,
            Self::MinorNegative => -1,
            Self::MajorPositive => 5,
            Self::MinorPositive | Self::Trading => 1,
        }
    }

    #[must_use]
    pub const fn max_value(self) -> i32 {
        match self {
            Self::MajorNegative => 100,
            Self::MinorNegative => 200,
            Self::MajorPositive => 20,
            Self::MinorPositive | Self::Trading => 25,
        }
    }

    #[must_use]
    pub const fn daily_decay(self) -> i32 {
        match self {
            Self::MajorNegative => 10,
            Self::MinorNegative => 20,
            Self::MajorPositive => 0,
            Self::MinorPositive => 1,
            Self::Trading => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GossipType;

    #[test]
    fn gossip_types_use_vanilla_names_and_values() {
        let types = [
            (GossipType::MajorNegative, "major_negative", -5, 100, 10),
            (GossipType::MinorNegative, "minor_negative", -1, 200, 20),
            (GossipType::MinorPositive, "minor_positive", 1, 25, 1),
            (GossipType::MajorPositive, "major_positive", 5, 20, 0),
            (GossipType::Trading, "trading", 1, 25, 2),
        ];

        for (index, (gossip_type, name, weight, max, decay)) in types.into_iter().enumerate() {
            assert_eq!(gossip_type.name(), name);
            assert_eq!(GossipType::from_name(name), Some(gossip_type));
            assert_eq!(GossipType::from_legacy_id(index as i32), Some(gossip_type));
            assert_eq!(gossip_type.weight(), weight);
            assert_eq!(gossip_type.max_value(), max);
            assert_eq!(gossip_type.daily_decay(), decay);
        }
    }
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
