use crate::{
    BlockState, BlockStateRef,
    block_properties::get_state_by_state_id,
    tag::{RegistryKey, Tagable},
};
use pumpkin_util::{
    loot_table::LootTable,
    math::experience::Experience,
    resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation},
};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Block {
    pub id: u16,
    pub name: &'static str,
    pub translation_key: &'static str,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub slipperiness: f32,
    pub velocity_multiplier: f32,
    pub jump_velocity_multiplier: f32,
    pub item_id: u16,
    pub default_state: &'static BlockState,
    pub states: &'static [BlockState],
    pub flammable: Option<Flammable>,
    pub loot_table: Option<LootTable>,
    pub experience: Option<Experience>,
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Block {}

impl Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Tagable for Block {
    #[inline]
    fn tag_key() -> RegistryKey {
        RegistryKey::Block
    }

    #[inline]
    fn registry_key(&self) -> &str {
        self.name
    }
}

impl ToResourceLocation for &'static Block {
    fn to_resource_location(&self) -> ResourceLocation {
        ResourceLocation::vanilla(self.name)
    }
}

impl FromResourceLocation for &'static Block {
    fn from_resource_location(resource_location: &ResourceLocation) -> Option<Self> {
        Block::from_registry_key(&resource_location.path)
    }
}

impl Block {
    pub fn is_waterlogged(&self, state_id: u16) -> bool {
        self.properties(state_id).is_some_and(|properties| {
            properties
                .to_props()
                .iter()
                .any(|(key, value)| key == "waterlogged" && value == "true")
        })
    }
}

#[derive(Clone, Debug)]
pub struct Flammable {
    pub spread_chance: u8,
    pub burn_chance: u8,
}
