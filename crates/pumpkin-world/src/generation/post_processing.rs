//! Vanilla `LevelChunk.postProcessGeneration`, the pass that runs once a chunk reaches
//! `FULL` and replays the block updates worldgen deferred.
//!
//! `WorldGenRegion.setBlock` asks every written state for
//! `blockState.getPostProcessPos(this, pos)` (unless the write carries flag `16`) and, when it
//! answers with a position, calls `chunk.markPosForPostProcessing(pos)`. Only four vanilla
//! blocks register a `PostProcess`: `SOUL_SAND` and `MAGMA_BLOCK` use `Blocks::postProcessAbove`
//! (`pos.above()`), the two mushrooms use `Blocks::postProcessSelf`.
//!
//! `LevelChunk.postProcessGeneration` then walks the marked positions and, for a
//! `LiquidBlock`, runs `blockState.tick(level, pos, random)`. `LiquidBlock.tick` grows a bubble
//! column when the fluid there is a full water source, which is how vanilla ends up with
//! `bubble_column` above worldgen magma without a single game tick having run.
//!
//! Only the magma/soul-sand -> bubble-column half is implemented here; the mushrooms take the
//! `Block.updateFromNeighbourShapes` branch instead and no worldgen mismatch is attributed to
//! them yet.

use pumpkin_data::{Block, BlockState, BlockStateId, block_properties::BubbleColumnLikeProperties};
use pumpkin_util::math::vector3::Vector3;

use super::proto_chunk::ProtoChunk;

/// The position `WorldGenRegion.setBlock` marks for post-processing after writing `state`,
/// mirroring `BlockBehaviour.Properties.postProcess`.
#[must_use]
pub fn post_process_pos(state: BlockStateId, pos: Vector3<i32>) -> Option<Vector3<i32>> {
    let block = state.to_block_id();
    // `Blocks::postProcessAbove`, registered on `MAGMA_BLOCK` and `SOUL_SAND`.
    if block == Block::MAGMA_BLOCK || block == Block::SOUL_SAND {
        return Some(Vector3::new(pos.x, pos.y + 1, pos.z));
    }
    None
}

/// `BubbleColumnBlock.canOccupy`: the state is already a bubble column, or it is a full water
/// source (`fluid.is(BUBBLE_COLUMN_CAN_OCCUPY) && block instanceof LiquidBlock &&
/// fluid.isSource() && fluid.getAmount() >= 8`).
#[must_use]
fn can_occupy(state: BlockStateId) -> bool {
    state.to_block_id() == Block::BUBBLE_COLUMN || state == Block::WATER.default_state.id
}

/// `BubbleColumnBlock.getColumnState`. `below` is what decides drag: magma is in
/// `ENABLES_BUBBLE_COLUMN_DRAG_DOWN`, soul sand in `ENABLES_BUBBLE_COLUMN_PUSH_UP`.
#[must_use]
fn column_state(below: BlockStateId, occupy: BlockStateId) -> BlockStateId {
    let below_block = below.to_block_id();
    if below_block == Block::BUBBLE_COLUMN {
        return below;
    }
    if below_block == Block::SOUL_SAND {
        return bubble_column(false);
    }
    if below_block == Block::MAGMA_BLOCK {
        return bubble_column(true);
    }
    if occupy.to_block_id() == Block::BUBBLE_COLUMN {
        Block::WATER.default_state.id
    } else {
        occupy
    }
}

#[must_use]
fn bubble_column(drag: bool) -> BlockStateId {
    BubbleColumnLikeProperties { drag }.to_state_id(&Block::BUBBLE_COLUMN)
}

/// `BubbleColumnBlock.updateColumn(bubbleColumn, level, occupyAt, occupyState, belowState)`.
///
/// The walk is strictly vertical, so it never leaves the chunk that holds `pos`.
fn update_column(chunk: &mut ProtoChunk, pos: Vector3<i32>) {
    let occupy = chunk.get_block_state(&pos);
    if !can_occupy(occupy) {
        return;
    }

    let below = chunk.get_block_state(&Vector3::new(pos.x, pos.y - 1, pos.z));
    let column = column_state(below, occupy);
    let column_state = BlockState::from_id(column);
    chunk.set_block_state(pos.x, pos.y, pos.z, column_state);

    let top = chunk.bottom_y() as i32 + chunk.height() as i32;
    let mut y = pos.y + 1;
    while y < top && can_occupy(chunk.get_block_state(&Vector3::new(pos.x, y, pos.z))) {
        // `if (!level.setBlock(pos, columnState, 2)) return;` — the only way that fails here is
        // leaving the build height, which the loop bound already covers.
        chunk.set_block_state(pos.x, y, pos.z, column_state);
        y += 1;
    }
}

/// `LevelChunk.postProcessGeneration`, restricted to the `blockState.getBlock() instanceof
/// LiquidBlock -> blockState.tick(...)` branch (`LiquidBlock.tick` ->
/// `BubbleColumnBlock.updateColumn`).
pub fn post_process_generation(chunk: &mut ProtoChunk) {
    if chunk.post_processing.is_empty() {
        return;
    }
    let marked = std::mem::take(&mut chunk.post_processing);
    for pos in marked {
        let state = chunk.get_block_state(&pos);
        // `LiquidBlock.tick` -> `shouldBubbleColumnOccupy(state)`, the same predicate as
        // `canOccupy` minus the "already a bubble column" shortcut.
        if state == Block::WATER.default_state.id {
            update_column(chunk, pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Blocks.MAGMA_BLOCK` and `Blocks.SOUL_SAND` are registered with
    /// `.postProcess(Blocks::postProcessAbove)`, which returns `blockPos.above()`.
    #[test]
    fn only_magma_and_soul_sand_mark_the_block_above() {
        let pos = Vector3::new(3, 7, -5);
        assert_eq!(
            post_process_pos(Block::MAGMA_BLOCK.default_state.id, pos),
            Some(Vector3::new(3, 8, -5))
        );
        assert_eq!(
            post_process_pos(Block::SOUL_SAND.default_state.id, pos),
            Some(Vector3::new(3, 8, -5))
        );
        assert_eq!(post_process_pos(Block::STONE.default_state.id, pos), None);
        assert_eq!(post_process_pos(Block::WATER.default_state.id, pos), None);
    }

    /// `BubbleColumnBlock` calls `registerDefaultState(stateDefinition.any().setValue(DRAG_DOWN,
    /// true))`, and `getColumnState` picks `DRAG_DOWN=true` over magma
    /// (`ENABLES_BUBBLE_COLUMN_DRAG_DOWN`) and `DRAG_DOWN=false` over soul sand
    /// (`ENABLES_BUBBLE_COLUMN_PUSH_UP`).
    #[test]
    fn column_state_follows_the_block_below() {
        let water = Block::WATER.default_state.id;
        let drag = bubble_column(true);
        let push = bubble_column(false);

        assert_eq!(drag, Block::BUBBLE_COLUMN.default_state.id);
        assert_ne!(drag, push);
        assert_eq!(
            column_state(Block::MAGMA_BLOCK.default_state.id, water),
            drag
        );
        assert_eq!(column_state(Block::SOUL_SAND.default_state.id, water), push);
        // An existing column above just propagates.
        assert_eq!(column_state(push, water), push);
        // Nothing enabling below and no column to remove: the state stays put.
        assert_eq!(column_state(Block::STONE.default_state.id, water), water);
        assert_eq!(column_state(Block::STONE.default_state.id, drag), water);
    }

    /// `canOccupy` accepts an existing bubble column and a full water source only.
    #[test]
    fn can_occupy_accepts_source_water_and_columns() {
        assert!(can_occupy(Block::WATER.default_state.id));
        assert!(can_occupy(bubble_column(true)));
        assert!(can_occupy(bubble_column(false)));
        assert!(!can_occupy(Block::LAVA.default_state.id));
        assert!(!can_occupy(Block::AIR.default_state.id));
        assert!(!can_occupy(Block::STONE.default_state.id));
    }
}
