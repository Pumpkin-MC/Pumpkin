use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, BubbleColumnLikeProperties},
    fluid::{
        Falling, FlowingWaterLikeFluidProperties as FlowingFluidProperties, Fluid, FluidProperties,
        Level,
    },
};

use crate::BlockStateId;

/// Helper utilities for working with bubble column block states.
pub struct BubbleColumn;

impl BubbleColumn {
    /// Returns `Some(drag)` if the provided block can act as the base of a bubble column.
    /// `drag = true` corresponds to downward columns (magma), `false` for upward soul sand columns.
    #[must_use]
    pub fn drag_for_base(block: &Block) -> Option<bool> {
        if block == &Block::SOUL_SAND {
            Some(false)
        } else if block == &Block::MAGMA_BLOCK {
            Some(true)
        } else {
            None
        }
    }

    /// Returns the drag direction encoded in a bubble column block state.
    #[must_use]
    pub fn drag_from_state(state_id: BlockStateId) -> Option<bool> {
        if Block::from_state_id(state_id) != &Block::BUBBLE_COLUMN {
            return None;
        }

        let props = BubbleColumnLikeProperties::from_state_id(state_id, &Block::BUBBLE_COLUMN);
        Some(props.drag)
    }

    /// Returns the block state ID for a bubble column with the provided drag direction.
    #[must_use]
    pub fn state_id(drag: bool) -> BlockStateId {
        let mut props = BubbleColumnLikeProperties::default(&Block::BUBBLE_COLUMN);
        props.drag = drag;
        props.to_state_id(&Block::BUBBLE_COLUMN)
    }

    /// Returns the block state ID for a still water source.
    #[must_use]
    pub fn water_source_state_id() -> BlockStateId {
        let mut props = FlowingFluidProperties::default(&Fluid::FLOWING_WATER);
        props.level = Level::L8;
        props.falling = Falling::False;
        props.to_state_id(&Fluid::FLOWING_WATER)
    }

    /// Returns true if the provided state ID represents a still source water block.
    #[must_use]
    pub fn is_source_water_state(state_id: BlockStateId) -> bool {
        if let Some(fluid) = Fluid::from_state_id(state_id)
            && fluid == &Fluid::FLOWING_WATER
        {
            let props = FlowingFluidProperties::from_state_id(state_id, &Fluid::FLOWING_WATER);
            return props.level == Level::L8 && props.falling == Falling::False;
        }
        false
    }

    /// Returns true if a block/state combination can be replaced or converted into a bubble column.
    #[must_use]
    pub fn can_host_column(block: &Block, state_id: BlockStateId) -> bool {
        block == &Block::BUBBLE_COLUMN
            || (block == &Block::WATER && Self::is_source_water_state(state_id))
    }
}
