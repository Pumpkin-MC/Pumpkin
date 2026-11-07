use crate::{
    BlockState, BlockStateId, Property, PropertyKey,
    block_properties::MAX_BLOCK_PROPERTIES,
    block_rotation::{Mirror, Rotation, transform_block_properties},
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
    /// Loot table identifier, `None` when the block has no loot table.
    pub loot_table: Option<&'static str>,
    /// State layout of each property, in vanilla order.
    pub property_slots: &'static [PropertySlot],
}

/// One property of a block: `index = (state - first) / stride % values.len()`.
#[derive(Debug, Clone, Copy)]
pub struct PropertySlot {
    pub key: PropertyKey,
    pub stride: u16,
    /// Values in index order, same variants as `to_props`.
    pub values: &'static [Property],
}

impl PropertySlot {
    #[must_use]
    pub const fn count(&self) -> u16 {
        self.values.len() as u16
    }

    #[must_use]
    pub fn index_of(&self, value: Property) -> Option<u16> {
        self.values
            .iter()
            .position(|slot_value| slot_value.matches_property(value))
            .map(|index| index as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnFloorPredicate {
    Default,
    Never,
    Always,
    OcelotOrParrot,
    PolarBear,
    FireImmune,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuffocationPredicate {
    Default,
    Never,
    Always,
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
    pub const fn is_waterlogged(&self, id: BlockStateId) -> bool {
        id.is_waterlogged()
    }

    #[must_use]
    pub fn is_waterloggable(&self) -> bool {
        self.property_slot(PropertyKey::Waterlogged).is_some()
    }

    /// Returns a new [`BlockStateId`] for the given [`BlockStateId`] with the
    /// `waterlogged` property forced to `value`. If the state already has
    /// waterlogged set to `value` or the  block does not expose a `waterlogged`
    /// property then `None` is returned.
    #[must_use]
    pub fn set_waterlogged(&self, id: BlockStateId, value: bool) -> Option<BlockStateId> {
        if id.is_waterlogged() == value {
            return None;
        }
        self.with_property(id, Property::Waterlogged(value))
    }

    #[must_use]
    pub fn property_slot(&self, key: PropertyKey) -> Option<&'static PropertySlot> {
        self.property_slots.iter().find(|slot| slot.key == key)
    }

    // state offset
    const fn state_offset(&self, id: BlockStateId) -> Option<u16> {
        let offset = id.as_u16().wrapping_sub(self.states[0].id.as_u16());
        if (offset as usize) < self.states.len() {
            Some(offset)
        } else {
            None
        }
    }

    /// Value index of `key` in `id`; `None` if `id` is not a state of this block.
    #[must_use]
    pub fn property_index(&self, id: BlockStateId, key: PropertyKey) -> Option<u16> {
        let offset = self.state_offset(id)?;
        let slot = self.property_slot(key)?;
        Some(offset / slot.stride % slot.count())
    }

    /// `id` with `key` set to value `index`, by arithmetic.
    #[must_use]
    pub fn with_property_index(
        &self,
        id: BlockStateId,
        key: PropertyKey,
        index: u16,
    ) -> Option<BlockStateId> {
        let offset = self.state_offset(id)?;
        let slot = self.property_slot(key)?;
        if index >= slot.count() {
            return None;
        }
        let current = offset / slot.stride % slot.count();
        let offset = offset - current * slot.stride + index * slot.stride;
        // SAFETY: offset < states.len(), mixed radix of a valid state.
        Some(unsafe { BlockStateId::new_unchecked(self.states[0].id.as_u16() + offset) })
    }

    #[must_use]
    pub fn get_property(&self, id: BlockStateId, key: PropertyKey) -> Option<Property> {
        let index = self.property_index(id, key)?;
        Some(self.property_slot(key)?.values[index as usize])
    }

    /// `id` with `value` applied; `None` if the block has no such value.
    #[must_use]
    pub fn with_property(&self, id: BlockStateId, value: Property) -> Option<BlockStateId> {
        let slot = self.property_slot(value.key())?;
        self.with_property_index(id, slot.key, slot.index_of(value)?)
    }

    /// All property values of `id`, no allocation.
    pub fn property_values(&self, id: BlockStateId) -> impl Iterator<Item = Property> {
        let offset = self.state_offset(id).unwrap_or(0);
        self.property_slots
            .iter()
            .map(move |slot| slot.values[(offset / slot.stride % slot.count()) as usize])
    }

    #[must_use]
    pub fn state_from_properties(
        &'static self,
        properties: &[Property],
    ) -> Option<&'static BlockState> {
        if properties.len() != self.property_slots.len() {
            return None;
        }
        let mut offset = 0;
        for slot in self.property_slots {
            let value = properties.iter().find_map(|value| slot.index_of(*value))?;
            offset += value * slot.stride;
        }
        self.states.get(offset as usize)
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
    pub fn mirror(&self, id: BlockStateId, mirror: Mirror) -> &'static BlockState {
        self.transform(id, Rotation::None, mirror)
    }

    #[must_use]
    pub fn rotate(&self, id: BlockStateId, rotation: Rotation) -> &'static BlockState {
        self.transform(id, rotation, Mirror::None)
    }

    fn transform(
        &self,
        id: BlockStateId,
        rotation: Rotation,
        mirror: Mirror,
    ) -> &'static BlockState {
        if (rotation == Rotation::None && mirror == Mirror::None) || self.states.len() <= 1 {
            return BlockState::from_id(id);
        }
        let mut values = [Property::Waterlogged(false); MAX_BLOCK_PROPERTIES];
        let count = self.property_slots.len();
        for (slot, value) in values.iter_mut().zip(self.property_values(id)) {
            *slot = value;
        }
        let values = &values[..count];
        let mut new_id = id;
        for value in transform_block_properties(self.name, values, rotation, mirror) {
            // changed only
            if !values.contains(&value) {
                new_id = self.with_property(new_id, value).unwrap_or(new_id);
            }
        }
        BlockState::from_id(new_id)
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

impl From<Block> for BlockId {
    #[inline]
    fn from(value: Block) -> Self {
        value.id
    }
}

impl From<&Block> for BlockId {
    #[inline]
    fn from(value: &Block) -> Self {
        value.id
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
