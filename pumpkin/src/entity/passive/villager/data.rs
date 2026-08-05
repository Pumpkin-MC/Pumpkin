use pumpkin_data::chunk::Biome;
use pumpkin_data::item::Item;
pub use pumpkin_data::villager::{VillagerProfession, VillagerType};
use pumpkin_protocol::codec::var_int::VarInt;
use serde::Serialize;

pub const BREEDING_FOOD_THRESHOLD: i32 = 12;

/// Vanilla `VillagerType#byBiome` / `BY_BIOME`
/// (`net/minecraft/world/entity/npc/villager/VillagerType.java`). Biomes absent from the
/// table fall back to `VillagerData.DEFAULT_TYPE`, which is `plains`.
///
/// If the position's biome cannot be resolved at all, this also yields `Plains` - but that is
/// a deliberate local default, not the vanilla rule above. Vanilla's `plains` default covers
/// biomes *absent from* `BY_BIOME`, which presupposes a known biome; vanilla has no case for a
/// position whose biome is unresolvable. The default is unavoidable rather than preferred:
/// both callers (`villager/mod.rs`, `zombie_villager.rs`) call this once from
/// `init_data_tracker`, which is never retried, and `VillagerData` has no "type unknown"
/// representation to carry forward.
#[must_use]
pub fn villager_type_at(entity: &crate::entity::Entity) -> VillagerType {
    entity
        .world
        .load()
        .get_biome(&pumpkin_util::math::position::BlockPos::floored_v(
            entity.pos.load(),
        ))
        .map_or(VillagerType::Plains, villager_type_by_biome)
}

#[must_use]
pub fn villager_type_by_biome(biome: &Biome) -> VillagerType {
    match biome.registry_id {
        "badlands" | "desert" | "eroded_badlands" | "wooded_badlands" => VillagerType::Desert,
        "bamboo_jungle" | "jungle" | "sparse_jungle" => VillagerType::Jungle,
        "savanna_plateau" | "savanna" | "windswept_savanna" => VillagerType::Savanna,
        "deep_frozen_ocean" | "frozen_ocean" | "frozen_river" | "ice_spikes" | "snowy_beach"
        | "snowy_taiga" | "snowy_plains" | "grove" | "snowy_slopes" | "frozen_peaks"
        | "jagged_peaks" => VillagerType::Snow,
        "swamp" | "mangrove_swamp" => VillagerType::Swamp,
        "old_growth_spruce_taiga"
        | "old_growth_pine_taiga"
        | "windswept_gravelly_hills"
        | "windswept_hills"
        | "taiga"
        | "windswept_forest" => VillagerType::Taiga,
        _ => VillagerType::Plains,
    }
}

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

#[cfg(test)]
mod tests {
    use super::{VillagerType, villager_type_by_biome};
    use pumpkin_data::chunk::Biome;

    #[test]
    fn maps_vanilla_by_biome_table() {
        for (biome, expected) in [
            (&Biome::DESERT, VillagerType::Desert),
            (&Biome::BADLANDS, VillagerType::Desert),
            (&Biome::BAMBOO_JUNGLE, VillagerType::Jungle),
            (&Biome::WINDSWEPT_SAVANNA, VillagerType::Savanna),
            (&Biome::FROZEN_RIVER, VillagerType::Snow),
            (&Biome::JAGGED_PEAKS, VillagerType::Snow),
            (&Biome::MANGROVE_SWAMP, VillagerType::Swamp),
            (&Biome::WINDSWEPT_FOREST, VillagerType::Taiga),
            (&Biome::OLD_GROWTH_PINE_TAIGA, VillagerType::Taiga),
        ] {
            assert_eq!(villager_type_by_biome(biome), expected);
        }
    }

    #[test]
    fn unlisted_biomes_fall_back_to_the_default_type() {
        assert_eq!(villager_type_by_biome(&Biome::PLAINS), VillagerType::Plains);
        assert_eq!(villager_type_by_biome(&Biome::FOREST), VillagerType::Plains);
        assert_eq!(
            villager_type_by_biome(&Biome::NETHER_WASTES),
            VillagerType::Plains
        );
    }
}
