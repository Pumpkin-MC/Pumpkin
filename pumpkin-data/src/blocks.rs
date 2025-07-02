use crate::{
    BlockState, BlockStateRef,
    block_properties::get_state_by_state_id,
    tag::{RegistryKey, Tagable},
};
use pumpkin_util::{loot_table::LootTable, math::experience::Experience};

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

#[derive(Clone, Debug)]
pub struct Flammable {
    pub spread_chance: u8,
    pub burn_chance: u8,
}
