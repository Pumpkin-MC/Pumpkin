use crate::{
    BlockState, BlockStateId,
    tag::{RegistryKey, Tag, Taggable},
};
use pumpkin_util::{
    loot_table::LootTable,
    math::{experience::Experience, position::BlockPos, vector3::Vector3},
    random::hash_block_pos,
    resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation},
};
use std::hash::{Hash, Hasher};

/// Represents the static definition of a Minecraft block type.
///
/// This struct contains the base properties shared by all instances of a block
/// Data-driven attributes like `hardness` and `blast_resistance` are defined here,
/// while specific orientations or variations are stored in the associated `BlockState`.
#[derive(Debug, Clone)]
pub struct Block {
    /// The numeric ID used for internal registry mapping.
    pub id: BlockId,
    /// The unique namespaced ID (e.g., "`diamond_ore`").
    pub name: &'static str,
    /// How hard the block is to break. A value of -1.0 indicates an unbreakable block (e.g., Bedrock).
    pub hardness: f32,
    /// The block's resistance to explosions.
    pub blast_resistance: f32,
    pub map_color: u8,
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
    /// Defines the amount of XP dropped when the block is mined (e.g., Coal or Diamond).
    pub experience: Option<Experience>,
}

/// Helper struct to ensure the validity of BlockIds parsed from external sources.
/// Every [`BlockId`] is guaranteed to correspond to a valid [`Block`].
///
/// Also enables [`Block`]-type pattern matching, even in const contexts:
/// ```rs
/// const fn to_waxed(block: &'static Block) -> Option<&'static Block> {
///     match block.id {
///         BlockId::COPPER_BLOCK => Some(Block::WAXED_COPPER_BLOCK),
///         //...
///         _ => None
///     }
/// }
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BlockId(u16);

impl PartialEq<BlockId> for Block {
    fn eq(&self, other: &BlockId) -> bool {
        self.id == *other
    }
}

impl PartialEq<Block> for BlockId {
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
        self.id.as_u16()
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
    pub const fn get_speed_factor(&self) -> f32 {
        self.velocity_multiplier
    }

    #[must_use]
    pub const fn get_jump_velocity_multiplier(&self) -> f32 {
        self.jump_velocity_multiplier
    }

    pub(crate) fn shape_offset_delta(&self, pos: &BlockPos) -> Vector3<f64> {
        let Some(shape_offset) = self.shape_offset() else {
            return Vector3::new(0.0, 0.0, 0.0);
        };

        let seed = hash_block_pos(pos.0.x, 0, pos.0.z) as u64;
        let max_horizontal = f64::from(shape_offset.max_horizontal);
        let x = (f64::from((seed & 15) as f32 / 15.0) - 0.5) * 0.5;
        let x = x.clamp(-max_horizontal, max_horizontal);
        let z = (f64::from(((seed >> 8) & 15) as f32 / 15.0) - 0.5) * 0.5;
        let z = z.clamp(-max_horizontal, max_horizontal);
        let y = match shape_offset.offset_type {
            ShapeOffsetType::Xz => 0.0,
            ShapeOffsetType::Xyz => {
                (f64::from(((seed >> 4) & 15) as f32 / 15.0) - 1.0)
                    * f64::from(shape_offset.max_vertical)
            }
        };

        // Extracted shapes are sampled at BlockPos::ZERO, where vanilla uses the
        // negative horizontal limits and, for XYZ offsets, the negative vertical
        // limit. Return only the delta from that sample.
        Vector3::new(
            x + max_horizontal,
            y + shape_offset.offset_type.origin_y(shape_offset.max_vertical),
            z + max_horizontal,
        )
    }

    #[must_use]
    pub fn is_waterlogged(&self, id: BlockStateId) -> bool {
        self.properties(id).is_some_and(|properties| {
            properties
                .to_props()
                .into_iter()
                .any(|(key, value)| key == "waterlogged" && value == "true")
        })
    }

    #[must_use]
    pub fn is_waterloggable(&self) -> bool {
        self.properties(self.default_state.id)
            .is_some_and(|props| props.to_props().iter().any(|p| p.0 == "waterlogged"))
    }

    /// Returns a new [`BlockStateId`] for the given [`BlockStateId`] with the
    /// `waterlogged` property forced to `value`. If the state already has
    /// waterlogged set to `value` or the  block does not expose a `waterlogged`
    /// property then `None` is returned.
    #[must_use]
    pub fn set_waterlogged(&self, id: BlockStateId, value: bool) -> Option<BlockStateId> {
        self.properties(id).and_then(|props| {
            let mut props = props.to_props();
            let waterlogged = &mut props.iter_mut().find(|p| p.0 == "waterlogged")?.1;
            let new_waterlogged = value.to_string();
            if new_waterlogged.as_str() == *waterlogged {
                return None;
            }

            *waterlogged = &new_waterlogged;
            Some(self.from_properties(&props).to_state_id(self))
        })
    }

    #[must_use]
    pub fn state_from_properties(
        &'static self,
        properties: &[(&str, &str)],
    ) -> Option<&'static BlockState> {
        self.states.iter().find(|state| {
            let Some(state_properties) = self.properties(state.id) else {
                return properties.is_empty();
            };
            let state_properties = state_properties.to_props();

            state_properties.len() == properties.len()
                && properties.iter().all(|(name, value)| {
                    state_properties.iter().any(|(state_name, state_value)| {
                        *state_name == *name && *state_value == *value
                    })
                })
        })
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

    #[must_use]
    pub fn mirror(
        &self,
        id: BlockStateId,
        mirror: crate::block_rotation::Mirror,
    ) -> &'static BlockState {
        if mirror == crate::block_rotation::Mirror::None || self.states.len() <= 1 {
            return BlockState::from_id(id);
        }
        if let Some(props) = self.properties(id) {
            let props_vec = props.to_props();
            let transformed = crate::block_rotation::transform_block_properties(
                self.name,
                &props_vec,
                crate::block_rotation::Rotation::None,
                mirror,
            );
            let transformed_refs: Vec<(&str, &str)> = transformed
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            let new_props = self.from_properties(&transformed_refs);
            let new_state_id = new_props.to_state_id(self);
            BlockState::from_id(new_state_id)
        } else {
            BlockState::from_id(id)
        }
    }

    #[must_use]
    pub fn rotate(
        &self,
        id: BlockStateId,
        rotation: crate::block_rotation::Rotation,
    ) -> &'static BlockState {
        if rotation == crate::block_rotation::Rotation::None || self.states.len() <= 1 {
            return BlockState::from_id(id);
        }
        if let Some(props) = self.properties(id) {
            let props_vec = props.to_props();
            let transformed = crate::block_rotation::transform_block_properties(
                self.name,
                &props_vec,
                rotation,
                crate::block_rotation::Mirror::None,
            );
            let transformed_refs: Vec<(&str, &str)> = transformed
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            let new_props = self.from_properties(&transformed_refs);
            let new_state_id = new_props.to_state_id(self);
            BlockState::from_id(new_state_id)
        } else {
            BlockState::from_id(id)
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ShapeOffsetType {
    Xz,
    Xyz,
}

#[derive(Clone, Copy)]
pub(crate) struct ShapeOffset {
    pub offset_type: ShapeOffsetType,
    pub max_horizontal: f32,
    pub max_vertical: f32,
}

impl ShapeOffsetType {
    fn origin_y(self, max_vertical: f32) -> f64 {
        match self {
            Self::Xz => 0.0,
            Self::Xyz => f64::from(max_vertical),
        }
    }
}

impl BlockId {
    // depends on generated impl:
    // pub(crate) const BLOCK_COUNT: u16;

    /// The total count of all registered blocks.
    pub const COUNT: u16 = Self::BLOCK_COUNT;

    // SAFETY: There must never be a BlockId where self.0 >= BlockId::BLOCK_COUNT

    #[inline]
    #[must_use]
    pub const fn new(inner: u16) -> Option<Self> {
        if inner < Self::BLOCK_COUNT {
            return Some(Self(inner));
        }
        None
    }

    #[inline]
    #[must_use]
    pub const fn new_or_air(inner: u16) -> Self {
        if inner < Self::BLOCK_COUNT {
            return Self(inner);
        }
        Self::AIR
    }

    #[inline]
    #[must_use]
    pub const fn to_block(self) -> &'static Block {
        Block::from_id(self)
    }

    #[inline(always)]
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.0
    }

    #[inline]
    #[must_use]
    pub fn has_tag(self, tag: Tag) -> bool {
        tag.1.contains(&self.0)
    }
}

impl From<BlockId> for u16 {
    #[inline]
    fn from(value: BlockId) -> Self {
        value.as_u16()
    }
}

impl Default for BlockId {
    #[inline]
    fn default() -> Self {
        Self::AIR
    }
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BlockId({} = \"{}\")",
            self.0,
            Block::from_id(*self).name
        )
    }
}

#[derive(Clone, Debug)]
pub struct Flammable {
    pub spread_chance: u8,
    pub burn_chance: u8,
}

#[cfg(test)]
mod tests {
    use super::{Block, BlockId, ShapeOffsetType};
    use pumpkin_util::math::position::BlockPos;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1.0e-6, "{actual} != {expected}");
    }

    #[test]
    fn shape_offset_registry_matches_vanilla_26_2() {
        let mut xz = 0;
        let mut xyz = 0;

        for raw_id in 0..BlockId::BLOCK_COUNT {
            match Block::from_id(BlockId::new(raw_id).unwrap())
                .shape_offset()
                .map(|offset| offset.offset_type)
            {
                Some(ShapeOffsetType::Xz) => xz += 1,
                Some(ShapeOffsetType::Xyz) => xyz += 1,
                None => {}
            }
        }

        assert_eq!(xz, 34);
        assert_eq!(xyz, 5);
    }

    #[test]
    fn shape_offset_limits_match_vanilla_26_2() {
        let bamboo = Block::BAMBOO.shape_offset().unwrap();
        assert_eq!(bamboo.max_horizontal, 0.25);
        assert_eq!(bamboo.max_vertical, 0.2);

        let pointed_dripstone = Block::POINTED_DRIPSTONE.shape_offset().unwrap();
        assert_eq!(pointed_dripstone.max_horizontal, 0.125);

        let small_dripleaf = Block::SMALL_DRIPLEAF.shape_offset().unwrap();
        assert_eq!(small_dripleaf.max_vertical, 0.1);
    }

    #[test]
    fn shape_offset_delta_matches_vanilla_coordinate_hash() {
        let origin = BlockPos::new(0, 64, 0);
        let positive_extreme = BlockPos::new(-18, 64, -7);

        let origin_delta = Block::BAMBOO.shape_offset_delta(&origin);
        assert_eq!(origin_delta.x, 0.0);
        assert_eq!(origin_delta.y, 0.0);
        assert_eq!(origin_delta.z, 0.0);

        let bamboo_delta = Block::BAMBOO.shape_offset_delta(&positive_extreme);
        assert_eq!(bamboo_delta.x, 0.5);
        assert_eq!(bamboo_delta.y, 0.0);
        assert_eq!(bamboo_delta.z, 0.5);

        let speleothem_delta = Block::POINTED_DRIPSTONE.shape_offset_delta(&positive_extreme);
        assert_eq!(speleothem_delta.x, 0.25);
        assert_eq!(speleothem_delta.y, 0.0);
        assert_eq!(speleothem_delta.z, 0.25);
        assert_eq!(
            Block::SULFUR_SPIKE.shape_offset_delta(&positive_extreme),
            speleothem_delta
        );

        let xyz_delta = Block::SHORT_GRASS.shape_offset_delta(&positive_extreme);
        assert_eq!(xyz_delta.x, 0.5);
        assert_close(xyz_delta.y, 0.08);
        assert_eq!(xyz_delta.z, 0.5);

        assert_eq!(Block::STONE.shape_offset_delta(&positive_extreme).x, 0.0);
    }
}
