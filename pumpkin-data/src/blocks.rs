use crate::{
    BlockState, BlockStateRef,
    tag::{RegistryKey, Taggable},
};
use pumpkin_util::{
    loot_table::LootTable,
    math::experience::Experience,
    resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation},
};
use std::hash::{Hash, Hasher};

/// Represents the static definition of a Minecraft block type.
///
/// This struct contains the base properties shared by all instances of a block
/// Data-driven attributes like `hardness` and `blast_resistance` are defined here,
/// while specific orientations or variations are stored in the associated `BlockState`.
#[derive(Debug)]
pub struct Block {
    /// The numeric ID used for internal registry mapping.
    pub id: u16,
    /// The unique namespaced ID (e.g., "`diamond_ore`").
    pub name: &'static str,
    /// The key used for client-side localization (e.g., "`block.minecraft.diamond_ore`").
    pub translation_key: &'static str,
    /// How hard the block is to break. A value of -1.0 indicates an unbreakable block (e.g., Bedrock).
    pub hardness: f32,
    /// The block's resistance to explosions.
    pub blast_resistance: f32,
    /// The friction coefficient. Default is 0.6; Ice is 0.98.
    pub slipperiness: f32,
    /// How much this block affects the speed of an entity walking on it (e.g., Soul Sand).
    pub velocity_multiplier: f32,
    /// How much this block affects an entity's jump height (e.g., Honey Blocks).
    pub jump_velocity_multiplier: f32,
    /// The ID of the item form of this block, used for inventory and drops.
    pub item_id: u16,
    /// The initial state of the block when placed without extra data.
    pub default_state: &'static BlockState,
    /// A list of all possible valid states (properties like rotation, waterlogged, etc.) for this block.
    pub states: &'static [BlockState],
    /// Fire behavior settings. If `None`, the block is not flammable.
    pub flammable: Option<Flammable>,
    /// Defines the items dropped when this block is destroyed.
    pub loot_table: Option<LootTable>,
    /// Defines the amount of XP dropped when the block is mined (e.g., Coal or Diamond).
    pub experience: Option<Experience>,
}

impl PartialEq<u16> for Block {
    fn eq(&self, other: &u16) -> bool {
        self.id == *other
    }
}

impl PartialEq<Block> for u16 {
    fn eq(&self, other: &Block) -> bool {
        *self == other.id
    }
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

impl Taggable for Block {
    #[inline]
    fn tag_key() -> RegistryKey {
        RegistryKey::Block
    }

    #[inline]
    fn registry_key(&self) -> &str {
        self.name
    }

    #[inline]
    fn registry_id(&self) -> u16 {
        self.id
    }
}

impl ToResourceLocation for &'static Block {
    fn to_resource_location(&self) -> ResourceLocation {
        format!("minecraft:{}", self.name)
    }
}

impl FromResourceLocation for &'static Block {
    fn from_resource_location(resource_location: &ResourceLocation) -> Option<Self> {
        Block::from_registry_key(
            resource_location
                .strip_prefix("minecraft:")
                .unwrap_or(resource_location),
        )
    }
}

impl Block {
    #[must_use]
    pub fn is_waterlogged(&self, state_id: u16) -> bool {
        self.properties(state_id).is_some_and(|properties| {
            properties
                .to_props()
                .into_iter()
                .any(|(key, value)| key == "waterlogged" && value == "true")
        })
    }

    /// Returns a new [`BlockState`] reference for the given `state_id` with the
    /// `waterlogged` property forced to `true` if the block supports that
    /// property.  If the state is already waterlogged or the block does not
    /// expose a `waterlogged` property then `None` is returned.
    #[must_use]
    pub fn with_waterlogged(&self, state_id: u16) -> Option<&'static BlockState> {
        // Check if already waterlogged
        if self.is_waterlogged(state_id) {
            return Some(BlockState::from_id(state_id));
        }

        // Modify the property list if available
        if let Some(props_source) = self.properties(state_id) {
            let mut props: Vec<(&str, &str)> = props_source
                .to_props()
                .iter()
                .map(|(k, v)| (*k, *v))
                .collect();

            // Only try to change states that actually have the key
            if props.iter().any(|(k, _)| *k == "waterlogged") {
                if let Some(idx) = props.iter().position(|(k, _)| *k == "waterlogged") {
                    props[idx] = ("waterlogged", "true");
                } else {
                    props.push(("waterlogged", "true"));
                }

                let new_state_id = self.from_properties(&props).to_state_id(self);
                return Some(BlockState::from_id(new_state_id));
            }
        }

        None
    }

    /// Returns whether this block is solid (based on default state)
    #[must_use]
    pub const fn is_solid(&self) -> bool {
        self.default_state.is_solid()
    }

    /// Returns whether this block is air (based on default state)
    #[must_use]
    pub const fn is_air(&self) -> bool {
        self.default_state.is_air()
    }
}

#[derive(Clone, Debug)]
pub struct Flammable {
    pub spread_chance: u8,
    pub burn_chance: u8,
}
