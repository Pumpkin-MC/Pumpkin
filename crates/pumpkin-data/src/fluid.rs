use crate::BlockStateId;
use crate::tag::{RegistryKey, Taggable};
use pumpkin_util::resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation};
use std::hash::{Hash, Hasher};

#[rustfmt::skip]
#[path = "generated/fluid.rs"]
mod generated;

/// Spread tuning of a fluid, from `FlowingFluid`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FluidFlow {
    /// `getTickDelay`.
    pub tick_delay: u8,
    /// `getDropOff`.
    pub drop_off: u8,
    /// `getSlopeFindDistance`: holes are found up to this + 1 blocks away.
    pub slope_find_distance: u8,
}

#[derive(Clone)]
pub struct FluidState {
    pub height: f32,
    pub level: i16,
    pub is_empty: bool,
    pub blast_resistance: f32,
    pub block_state_id: BlockStateId,
    pub is_still: bool,
    pub is_source: bool,
    pub falling: bool,
}

#[derive(Clone)]
pub struct Fluid {
    pub id: u16,
    pub name: &'static str,
    pub states: &'static [FluidState],
    pub default_state_index: u16,
    /// Levels without `FAST_LAVA`.
    pub flow: FluidFlow,
    /// Levels with `FAST_LAVA`.
    pub fast_lava_flow: FluidFlow,
}

impl Hash for Fluid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Fluid {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Fluid {}

impl Fluid {
    #[must_use]
    pub const fn same_fluid_type(a: u16, b: u16) -> bool {
        Self::family_id(a) == Self::family_id(b)
    }

    #[must_use]
    pub const fn matches_type(&self, other: &Self) -> bool {
        Self::same_fluid_type(self.id, other.id)
    }

    /// Spread tuning in a level with `fast_lava`.
    #[must_use]
    pub const fn flow_in(&self, fast_lava: bool) -> FluidFlow {
        if fast_lava {
            self.fast_lava_flow
        } else {
            self.flow
        }
    }

    /// Vanilla `BlockState.getFluidState().getType()`.
    #[must_use]
    #[inline]
    pub const fn of_block_state(id: BlockStateId) -> &'static Self {
        // extractor IS_LIQUID: fluid state not empty
        if !id.to_state().is_liquid() {
            return &Self::EMPTY;
        }
        match Self::from_state_id(id) {
            Some(fluid) => fluid,
            // waterlogged, kelp, seagrass, bubble column
            None => &Self::WATER,
        }
    }

    /// Vanilla `BlockState.getFluidState()`.
    #[must_use]
    #[inline]
    pub fn fluid_state_of(id: BlockStateId) -> (&'static Self, FluidState) {
        let fluid = Self::of_block_state(id);
        let state = match FlowingFluidProperties::from_state_id(id, fluid) {
            // LiquidBlock.getFluidState: falling blocks hold amount 8
            Some(FlowingFluidProperties {
                falling: Falling::True,
                ..
            }) => &fluid.states[FALLING_ROW],
            Some(props) => &fluid.states[props.row()],
            None => fluid.default_state(),
        };
        (fluid, state.clone())
    }

    #[must_use]
    pub const fn default_state(&self) -> &FluidState {
        &self.states[self.default_state_index as usize]
    }

    /// `state_id` is a source state of this fluid.
    #[must_use]
    pub fn is_source(&self, state_id: BlockStateId) -> bool {
        self.states
            .iter()
            .any(|state| state.block_state_id == state_id && state.is_source)
    }
}

impl ToResourceLocation for &'static Fluid {
    fn to_resource_location(&self) -> ResourceLocation {
        format!("minecraft:{}", self.name)
    }
}

impl FromResourceLocation for &'static Fluid {
    fn from_resource_location(resource_location: &ResourceLocation) -> Option<Self> {
        Fluid::from_registry_key(
            resource_location
                .strip_prefix("minecraft:")
                .unwrap_or(resource_location),
        )
    }
}

impl Taggable for Fluid {
    #[inline]
    fn tag_key() -> RegistryKey {
        RegistryKey::Fluid
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Falling {
    True,
    False,
}

/// Fluid amount 1..=8.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Level {
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
    L8,
}

impl Level {
    const ALL: [Self; 8] = [
        Self::L1,
        Self::L2,
        Self::L3,
        Self::L4,
        Self::L5,
        Self::L6,
        Self::L7,
        Self::L8,
    ];

    /// Amount - 1.
    #[must_use]
    pub const fn to_index(self) -> u16 {
        self as u16
    }

    /// `None` outside 1..=8.
    #[must_use]
    pub const fn from_amount(amount: i32) -> Option<Self> {
        if 1 <= amount && amount <= 8 {
            Some(Self::ALL[amount as usize - 1])
        } else {
            None
        }
    }
}

/// Amount and falling of a flowing fluid state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlowingFluidProperties {
    pub falling: Falling,
    pub level: Level,
}

/// Rows of a flowing fluid: 8 levels, falling and not.
const ROWS: usize = 16;
/// Falling, amount 8.
const FALLING_ROW: usize = Level::L8 as usize;

impl FlowingFluidProperties {
    // flowing fluid rows: level + 8 * falling
    const fn row(self) -> usize {
        self.level as usize + 8 * self.falling as usize
    }

    /// `None` if `id` is no flowing state of `fluid`.
    #[must_use]
    #[inline]
    pub const fn from_state_id(id: BlockStateId, fluid: &Fluid) -> Option<Self> {
        // flowing rows: ids descend from row 0
        let row = fluid.states[0]
            .block_state_id
            .as_u16()
            .wrapping_sub(id.as_u16()) as usize;
        if fluid.states.len() != ROWS || row >= ROWS {
            return None;
        }
        Some(Self {
            level: Level::ALL[row % 8],
            falling: if row < 8 {
                Falling::True
            } else {
                Falling::False
            },
        })
    }

    /// Flowing fluids only: their 16 rows cover every property.
    #[must_use]
    pub const fn to_state_id(self, fluid: &Fluid) -> BlockStateId {
        fluid.states[self.row()].block_state_id
    }
}

#[cfg(test)]
mod tests {
    use super::Fluid;
    use crate::Block;
    use crate::block_properties::{BlockProperties, WaterLikeProperties};

    #[test]
    fn liquid_block_states_preserve_source_flow_and_falling_depths() {
        // LiquidBlock.getFluidState: source, amounts 7..1, then falling amount 8.
        let amounts = [8, 7, 6, 5, 4, 3, 2, 1, 8, 8, 8, 8, 8, 8, 8, 8];
        for (block, source, flowing) in [
            (&Block::WATER, &Fluid::WATER, &Fluid::FLOWING_WATER),
            (&Block::LAVA, &Fluid::LAVA, &Fluid::FLOWING_LAVA),
        ] {
            for (level, amount) in amounts.into_iter().enumerate() {
                let id = WaterLikeProperties { level: level as u8 }.to_state_id(block);
                let (fluid, state) = Fluid::fluid_state_of(id);
                let expected = if level == 0 { source } else { flowing };
                assert_eq!(fluid.id, expected.id, "{} level {level}", block.name);
                assert_eq!(state.level, amount);
                assert!((state.height - f32::from(amount) / 9.0).abs() < f32::EPSILON);
                assert_eq!(state.is_source, level == 0);
                assert_eq!(state.falling, level >= 8);
                assert!(!state.is_empty);
                assert_eq!(
                    state.block_state_id,
                    WaterLikeProperties {
                        level: level.min(8) as u8
                    }
                    .to_state_id(block)
                );
            }
        }
    }

    #[test]
    fn aquatic_and_waterlogged_blocks_contain_source_water() {
        let wet_stairs = Block::OAK_STAIRS
            .set_waterlogged(Block::OAK_STAIRS.default_state.id, true)
            .unwrap();
        for id in [
            Block::KELP.default_state.id,
            Block::KELP_PLANT.default_state.id,
            Block::SEAGRASS.default_state.id,
            Block::TALL_SEAGRASS.default_state.id,
            Block::BUBBLE_COLUMN.default_state.id,
            wet_stairs,
        ] {
            let (fluid, state) = Fluid::fluid_state_of(id);
            assert_eq!(fluid.id, Fluid::WATER.id);
            assert!(state.is_source && !state.is_empty && !state.falling);
            assert_eq!(state.level, 8);
            assert!((state.height - 8.0 / 9.0).abs() < f32::EPSILON);
        }

        for block in [&Block::AIR, &Block::OAK_STAIRS, &Block::WATER_CAULDRON] {
            let (fluid, state) = Fluid::fluid_state_of(block.default_state.id);
            assert_eq!(fluid.id, Fluid::EMPTY.id);
            assert!(state.is_empty);
            assert_eq!(state.height, 0.0);
        }
    }
}
