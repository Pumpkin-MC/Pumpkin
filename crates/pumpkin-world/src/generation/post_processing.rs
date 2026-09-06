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
//! `StructurePiece.placeBlock` marks a second family of positions: after `level.setBlock`, a
//! block in `SHAPE_CHECK_BLOCKS` (the fences, torches, ladder and iron bars) marks *itself* on
//! the chunk that holds it. Those take the other branch of `postProcessGeneration`,
//! `Block.updateFromNeighbourShapes`, which is what gives worldgen fences their connections.

use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockState, BlockStateId,
    block_properties::{
        Axis, BubbleColumnLikeProperties, OakFenceGateLikeProperties, OakFenceLikeProperties,
    },
    tag,
    tag::Taggable,
};
use pumpkin_util::math::vector3::Vector3;

use super::proto_chunk::GenerationCache;

/// `StructurePiece.SHAPE_CHECK_BLOCKS` (`StructurePiece.java:43`): the blocks whose shape
/// `StructurePiece.placeBlock` re-derives once the chunk is complete.
const SHAPE_CHECK_BLOCKS: [BlockId; 12] = [
    Block::NETHER_BRICK_FENCE.id,
    Block::TORCH.id,
    Block::WALL_TORCH.id,
    Block::OAK_FENCE.id,
    Block::SPRUCE_FENCE.id,
    Block::DARK_OAK_FENCE.id,
    Block::PALE_OAK_FENCE.id,
    Block::ACACIA_FENCE.id,
    Block::BIRCH_FENCE.id,
    Block::JUNGLE_FENCE.id,
    Block::LADDER.id,
    Block::IRON_BARS.id,
];

/// `SHAPE_CHECK_BLOCKS.contains(blockState.getBlock())`.
#[must_use]
pub fn needs_shape_check(state: BlockStateId) -> bool {
    SHAPE_CHECK_BLOCKS.contains(&state.to_block_id())
}

/// `BlockBehaviour.UPDATE_SHAPE_ORDER` (`BlockBehaviour.java:85`).
const UPDATE_SHAPE_ORDER: [BlockDirection; 6] = [
    BlockDirection::West,
    BlockDirection::East,
    BlockDirection::North,
    BlockDirection::South,
    BlockDirection::Down,
    BlockDirection::Up,
];

/// `Block.updateFromNeighbourShapes` (`Block.java:200`): fold `updateShape` over
/// `UPDATE_SHAPE_ORDER`, reading every neighbour from the unmodified world.
#[must_use]
fn update_from_neighbour_shapes<C: GenerationCache + ?Sized>(
    state: BlockStateId,
    pos: Vector3<i32>,
    cache: &C,
) -> BlockStateId {
    let mut new_state = state;
    for direction in UPDATE_SHAPE_ORDER {
        let offset = direction.to_offset();
        let neighbour_pos = Vector3::new(pos.x + offset.x, pos.y + offset.y, pos.z + offset.z);
        let neighbour = GenerationCache::get_block_state(cache, &neighbour_pos);
        new_state = update_shape(new_state, pos, direction, neighbour, cache);
    }
    new_state
}

/// `BlockState.updateShape`, restricted to the overrides worldgen can reach.
///
/// `FenceBlock.updateShape` (`FenceBlock.java:99`) and, for the two mushrooms,
/// `VegetationBlock.updateShape` (`VegetationBlock.java:28`) are ported; the other
/// `SHAPE_CHECK_BLOCKS` (torch, wall torch, ladder, iron bars) keep the placed state, which is
/// what Pumpkin did before this pass existed.
#[must_use]
fn update_shape<C: GenerationCache + ?Sized>(
    state: BlockStateId,
    pos: Vector3<i32>,
    direction: BlockDirection,
    neighbour: BlockStateId,
    cache: &C,
) -> BlockStateId {
    let block = state.to_block_id();
    if block == Block::BROWN_MUSHROOM || block == Block::RED_MUSHROOM {
        // `!state.canSurvive(level, pos) ? Blocks.AIR.defaultBlockState() : super.updateShape(..)`
        return if mushroom_can_survive(cache, pos) {
            state
        } else {
            Block::AIR.default_state.id
        };
    }
    fence_update_shape(state, direction, neighbour)
}

/// `MushroomBlock.canSurvive` (`MushroomBlock.java:83`):
/// `below.is(OVERRIDES_MUSHROOM_LIGHT_REQUIREMENT) ? true
///  : level.getRawBrightness(pos, 0) < 13 && this.mayPlaceOn(below, level, belowPos)`,
/// with `MushroomBlock.mayPlaceOn` = `state.isSolidRender()`.
#[must_use]
fn mushroom_can_survive<C: GenerationCache + ?Sized>(cache: &C, pos: Vector3<i32>) -> bool {
    let below_pos = Vector3::new(pos.x, pos.y - 1, pos.z);
    let below = GenerationCache::get_block_state(cache, &below_pos);
    if below
        .to_block()
        .has_tag(&tag::Block::MINECRAFT_OVERRIDES_MUSHROOM_LIGHT_REQUIREMENT)
    {
        return true;
    }
    raw_brightness(cache, pos) < 13 && BlockState::from_id(below).is_solid_render()
}

/// `LevelReader.getRawBrightness(pos, 0)` = `max(blockLight, skyLight)`.
///
/// Vanilla answers this from the light engine, which has run by the time a chunk reaches
/// `FULL`. Pumpkin has no light at the feature stage, so only the one value the sky light
/// engine reaches without propagating is modelled: a column of nothing but air above `pos`
/// is sky light 15, exactly. Every other position answers 0, which keeps the block — the
/// state Pumpkin had before this pass, never a removal vanilla did not make.
#[must_use]
fn raw_brightness<C: GenerationCache + ?Sized>(cache: &C, pos: Vector3<i32>) -> u8 {
    let top = i32::from(cache.top_y());
    for y in pos.y + 1..top {
        if !BlockState::from_id(GenerationCache::get_block_state(
            cache,
            &Vector3::new(pos.x, y, pos.z),
        ))
        .is_air()
        {
            return 0;
        }
    }
    15
}

/// `FenceBlock.updateShape` (`FenceBlock.java:99`).
#[must_use]
fn fence_update_shape(
    state: BlockStateId,
    direction: BlockDirection,
    neighbour: BlockStateId,
) -> BlockStateId {
    let block = state.to_block();
    if !block.has_tag(&tag::Block::MINECRAFT_FENCES) || direction.to_axis() == Axis::Y {
        return state;
    }

    // `state.setValue(PROPERTY_BY_DIRECTION.get(directionToNeighbour), this.connectsTo(
    //     neighbourState, neighbourState.isFaceSturdy(level, neighbourPos, direction.getOpposite()),
    //     direction.getOpposite()))`
    let facing = direction.opposite();
    let face_sturdy = BlockState::from_id(neighbour).is_side_solid(facing);
    let connected = connects_to(block, neighbour, face_sturdy, facing);

    let mut properties = OakFenceLikeProperties::from_state_id(state);
    match direction {
        BlockDirection::North => properties.north = connected,
        BlockDirection::South => properties.south = connected,
        BlockDirection::West => properties.west = connected,
        BlockDirection::East => properties.east = connected,
        BlockDirection::Up | BlockDirection::Down => unreachable!("filtered by the axis check"),
    }
    properties.to_state_id(block)
}

/// `FenceBlock.connectsTo` (`FenceBlock.java:59`):
/// `!isExceptionForConnection(state) && faceSolid || sameFence || gate`.
#[must_use]
fn connects_to(
    fence: &'static Block,
    neighbour: BlockStateId,
    face_solid: bool,
    direction: BlockDirection,
) -> bool {
    let block = neighbour.to_block();

    // `isSameFence`: `state.is(FENCES) && state.is(WOODEN_FENCES) == this.defaultBlockState()
    // .is(WOODEN_FENCES)`.
    let same_fence = block.has_tag(&tag::Block::MINECRAFT_FENCES)
        && block.has_tag(&tag::Block::MINECRAFT_WOODEN_FENCES)
            == fence.has_tag(&tag::Block::MINECRAFT_WOODEN_FENCES);

    // `block instanceof FenceGateBlock && FenceGateBlock.connectsToDirection(state, direction)`,
    // which is `state.getValue(FACING).getAxis() == direction.getClockWise().getAxis()`.
    let gate = block.has_tag(&tag::Block::MINECRAFT_FENCE_GATES) && {
        let facing = OakFenceGateLikeProperties::from_state_id(neighbour).facing;
        BlockDirection::from_cardinal_direction(facing).to_axis()
            == direction.rotate_clockwise().to_axis()
    };

    !is_exception_for_connection(neighbour) && face_solid || same_fence || gate
}

/// `Block.isExceptionForConnection` (`Block.java:251`).
#[must_use]
fn is_exception_for_connection(state: BlockStateId) -> bool {
    let block = state.to_block();
    block.has_tag(&tag::Block::MINECRAFT_LEAVES)
        || block.id == Block::BARRIER.id
        || block.id == Block::CARVED_PUMPKIN.id
        || block.id == Block::JACK_O_LANTERN.id
        || block.id == Block::MELON.id
        || block.id == Block::PUMPKIN.id
        || block.has_tag(&tag::Block::MINECRAFT_SHULKER_BOXES)
}

/// The position `WorldGenRegion.setBlock` marks for post-processing after writing `state`,
/// mirroring `BlockBehaviour.Properties.postProcess`.
#[must_use]
pub fn post_process_pos(state: BlockStateId, pos: Vector3<i32>) -> Option<Vector3<i32>> {
    let block = state.to_block_id();
    // `Blocks::postProcessAbove`, registered on `MAGMA_BLOCK` and `SOUL_SAND`.
    if block == Block::MAGMA_BLOCK || block == Block::SOUL_SAND {
        return Some(Vector3::new(pos.x, pos.y + 1, pos.z));
    }
    // `Blocks::postProcessSelf`, registered on `BROWN_MUSHROOM` (`Blocks.java:1015`) and
    // `RED_MUSHROOM` (`Blocks.java:1027`).
    if block == Block::BROWN_MUSHROOM || block == Block::RED_MUSHROOM {
        return Some(pos);
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
fn update_column<C: GenerationCache + ?Sized>(cache: &mut C, pos: Vector3<i32>) {
    let occupy = GenerationCache::get_block_state(cache, &pos);
    if !can_occupy(occupy) {
        return;
    }

    let below = GenerationCache::get_block_state(cache, &Vector3::new(pos.x, pos.y - 1, pos.z));
    let column = column_state(below, occupy);
    let column_state = BlockState::from_id(column);
    cache.set_block_state(&pos, column_state);

    let top = cache.bottom_y() as i32 + cache.height() as i32;
    let mut y = pos.y + 1;
    while y < top
        && can_occupy(GenerationCache::get_block_state(
            cache,
            &Vector3::new(pos.x, y, pos.z),
        ))
    {
        // `if (!level.setBlock(pos, columnState, 2)) return;` — the only way that fails here is
        // leaving the build height, which the loop bound already covers.
        cache.set_block_state(&Vector3::new(pos.x, y, pos.z), column_state);
        y += 1;
    }
}

/// Whether all four horizontal neighbours of `pos` sit in chunks this cache can read.
///
/// A `SHAPE_CHECK_BLOCKS` position is only re-derived from a window that actually holds its
/// neighbours; from any other window the missing side would read as `AIR`.
#[must_use]
fn shape_check_readable<C: GenerationCache + ?Sized>(cache: &C, pos: Vector3<i32>) -> bool {
    UPDATE_SHAPE_ORDER
        .iter()
        .filter(|direction| direction.to_axis() != Axis::Y)
        .all(|direction| {
            let offset = direction.to_offset();
            cache.contains_chunk((pos.x + offset.x) >> 4, (pos.z + offset.z) >> 4)
        })
}

/// `LevelChunk.postProcessGeneration` (`LevelChunk.java:565`) for one chunk's marked positions.
///
/// ```text
/// if (blockState.getBlock() instanceof LiquidBlock) blockState.tick(level, pos, random);
/// else { BlockState s = Block.updateFromNeighbourShapes(blockState, level, pos);
///        if (s != blockState) level.setBlock(pos, s, 276); }
/// ```
///
/// Vanilla runs this once, when the chunk reaches `FULL` and therefore after every one of its
/// eight neighbours finished `FEATURES`. Pumpkin decorates a chunk from a 3x3 window and has no
/// `FULL` transition, so the shape half is instead run again every time a chunk of the window
/// finishes its features: `Block.updateFromNeighbourShapes` recomputes all four sides from the
/// world, so the last run — the one after the neighbour on that side was decorated — is the one
/// that decides, and it sees exactly what vanilla saw. The liquid half is not idempotent in
/// principle and stays a one-shot.
pub fn post_process_generation<C: GenerationCache + ?Sized>(
    cache: &mut C,
    chunk_x: i32,
    chunk_z: i32,
) {
    let marked = match cache.get_chunk_mut(chunk_x, chunk_z) {
        Some(chunk) if !chunk.post_processing.is_empty() => {
            std::mem::take(&mut chunk.post_processing)
        }
        _ => return,
    };

    let mut pending = Vec::new();
    for pos in marked {
        let state = GenerationCache::get_block_state(cache, &pos);
        let block = state.to_block_id();
        if block == Block::WATER.id || block == Block::LAVA.id {
            // `LiquidBlock.tick` -> `shouldBubbleColumnOccupy(state)`, the same predicate as
            // `canOccupy` minus the "already a bubble column" shortcut.
            if state == Block::WATER.default_state.id {
                update_column(cache, pos);
            }
            continue;
        }
        pending.push(pos);
        if !shape_check_readable(cache, pos) {
            continue;
        }
        let new_state = update_from_neighbour_shapes(state, pos, cache);
        if new_state != state {
            cache.set_block_state(&pos, BlockState::from_id(new_state));
        }
    }

    if let Some(chunk) = cache.get_chunk_mut(chunk_x, chunk_z) {
        chunk.post_processing = pending;
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

    /// `Blocks.BROWN_MUSHROOM` (`Blocks.java:1015`) and `Blocks.RED_MUSHROOM`
    /// (`Blocks.java:1027`) are registered with `.postProcess(Blocks::postProcessSelf)`, which
    /// returns the position itself.
    #[test]
    fn the_mushrooms_mark_themselves() {
        let pos = Vector3::new(-127, 64, 33);
        assert_eq!(
            post_process_pos(Block::BROWN_MUSHROOM.default_state.id, pos),
            Some(pos)
        );
        assert_eq!(
            post_process_pos(Block::RED_MUSHROOM.default_state.id, pos),
            Some(pos)
        );
    }

    /// `MushroomBlock.canSurvive` (`MushroomBlock.java:83`) short-circuits to `true` only for
    /// `BlockTags.OVERRIDES_MUSHROOM_LIGHT_REQUIREMENT` — mycelium, podzol and the two nyliums.
    /// On anything else, `mayPlaceOn` (`state.isSolidRender()`) passes for the sand these seven
    /// worldgen mushrooms stand on, so `getRawBrightness(pos, 0) < 13` is what decides, and
    /// under open sky it does not hold.
    #[test]
    fn only_the_light_requirement_can_remove_a_mushroom_from_sand() {
        for block in [Block::MYCELIUM, Block::PODZOL] {
            assert!(block.has_tag(&tag::Block::MINECRAFT_OVERRIDES_MUSHROOM_LIGHT_REQUIREMENT));
        }
        assert!(!Block::SAND.has_tag(&tag::Block::MINECRAFT_OVERRIDES_MUSHROOM_LIGHT_REQUIREMENT));
        assert!(BlockState::from_id(Block::SAND.default_state.id).is_solid_render());
        assert!(!BlockState::from_id(Block::AIR.default_state.id).is_solid_render());
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

    /// `StructurePiece.SHAPE_CHECK_BLOCKS` (`StructurePiece.java:43`) holds the six wood
    /// fences, the nether brick fence, both torches, the ladder and iron bars — and nothing else.
    #[test]
    fn shape_check_blocks_are_the_vanilla_set() {
        for block in [
            Block::NETHER_BRICK_FENCE,
            Block::TORCH,
            Block::WALL_TORCH,
            Block::OAK_FENCE,
            Block::SPRUCE_FENCE,
            Block::DARK_OAK_FENCE,
            Block::PALE_OAK_FENCE,
            Block::ACACIA_FENCE,
            Block::BIRCH_FENCE,
            Block::JUNGLE_FENCE,
            Block::LADDER,
            Block::IRON_BARS,
        ] {
            assert!(needs_shape_check(block.default_state.id), "{}", block.name);
        }
        for block in [
            Block::CRIMSON_FENCE,
            Block::MANGROVE_FENCE,
            Block::OAK_FENCE_GATE,
            Block::DARK_OAK_PLANKS,
            Block::RAIL,
            Block::COBWEB,
            Block::STONE,
        ] {
            assert!(!needs_shape_check(block.default_state.id), "{}", block.name);
        }
    }

    /// `BlockBehaviour.UPDATE_SHAPE_ORDER = {WEST, EAST, NORTH, SOUTH, DOWN, UP}`
    /// (`BlockBehaviour.java:85`).
    #[test]
    fn update_shape_order_is_west_east_north_south_down_up() {
        assert_eq!(
            UPDATE_SHAPE_ORDER,
            [
                BlockDirection::West,
                BlockDirection::East,
                BlockDirection::North,
                BlockDirection::South,
                BlockDirection::Down,
                BlockDirection::Up,
            ]
        );
    }

    #[expect(clippy::fn_params_excessive_bools)]
    fn fence(north: bool, east: bool, south: bool, west: bool) -> BlockStateId {
        OakFenceLikeProperties {
            north,
            east,
            south,
            west,
            waterlogged: false,
        }
        .to_state_id(&Block::DARK_OAK_FENCE)
    }

    /// `FenceBlock.updateShape` (`FenceBlock.java:99`) writes
    /// `PROPERTY_BY_DIRECTION.get(directionToNeighbour)` from `connectsTo(neighbourState,
    /// neighbourState.isFaceSturdy(level, neighbourPos, direction.getOpposite()),
    /// direction.getOpposite())`, and leaves the state alone for a vertical neighbour.
    #[test]
    fn fence_update_shape_follows_the_neighbour() {
        // A sturdy face connects.
        assert_eq!(
            fence_update_shape(
                fence(false, false, false, false),
                BlockDirection::East,
                Block::ORANGE_TERRACOTTA.default_state.id,
            ),
            fence(false, true, false, false)
        );
        // Cave air does not, and it also clears a connection the piece placed.
        assert_eq!(
            fence_update_shape(
                fence(false, true, false, false),
                BlockDirection::East,
                Block::CAVE_AIR.default_state.id,
            ),
            fence(false, false, false, false)
        );
        // `isSameFence`: another wooden fence connects even though its face is not sturdy...
        assert_eq!(
            fence_update_shape(
                fence(false, false, false, false),
                BlockDirection::North,
                Block::OAK_FENCE.default_state.id,
            ),
            fence(true, false, false, false)
        );
        // ...but the nether brick fence is not in `WOODEN_FENCES`, so it does not.
        assert_eq!(
            fence_update_shape(
                fence(false, false, false, false),
                BlockDirection::North,
                Block::NETHER_BRICK_FENCE.default_state.id,
            ),
            fence(false, false, false, false)
        );
        // `isExceptionForConnection`: a pumpkin has a sturdy face and still does not connect.
        assert_eq!(
            fence_update_shape(
                fence(false, false, false, false),
                BlockDirection::South,
                Block::PUMPKIN.default_state.id,
            ),
            fence(false, false, false, false)
        );
        // `directionToNeighbour.getAxis().isHorizontal()` gates the whole thing.
        assert_eq!(
            fence_update_shape(
                fence(false, false, false, false),
                BlockDirection::Up,
                Block::ORANGE_TERRACOTTA.default_state.id,
            ),
            fence(false, false, false, false)
        );
        // Non-fences take the `super.updateShape` identity branch here.
        assert_eq!(
            fence_update_shape(
                Block::DARK_OAK_PLANKS.default_state.id,
                BlockDirection::East,
                Block::ORANGE_TERRACOTTA.default_state.id,
            ),
            Block::DARK_OAK_PLANKS.default_state.id
        );
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
