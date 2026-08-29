use std::sync::Arc;

use crate::block::entities::piston::PistonBlockEntity;
use crate::entity::EntityBase;
use pumpkin_data::BlockId;
use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId, FacingExt,
    block_properties::{
        BlockProperties, MovingPistonLikeProperties, PistonHeadLikeProperties, PistonType,
    },
    block_state::PistonBehavior,
    sound::{Sound, SoundCategory},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;
use rustc_hash::FxHashMap;

use crate::{
    block::{
        BlockBehaviour, BlockMetadata, BrokenArgs, OnNeighborUpdateArgs, OnPlaceArgs,
        OnSyncedBlockEventArgs, PlacedArgs,
        blocks::{piston::piston_head::PistonHeadProperties, redstone::is_emitting_redstone_power},
    },
    world::World,
};

use super::PistonHandler;

pub(crate) type PistonProps = pumpkin_data::block_properties::StickyPistonLikeProperties;

pub struct PistonBlock;

impl BlockMetadata for PistonBlock {
    fn ids() -> Box<[BlockId]> {
        [Block::PISTON.id, Block::STICKY_PISTON.id].into()
    }
}

impl PistonBlock {
    /// Vanilla piston `blockEvent` type: extend (`0`), retract/pull (`1`), drop (`2`).
    pub const TRIGGER_EXTEND: u8 = 0;
    pub const TRIGGER_CONTRACT: u8 = 1;
    pub const TRIGGER_DROP: u8 = 2;

    /// Dest/arm `moving_piston`: vanilla 324
    /// (`UPDATE_INVISIBLE | MOVE_BY_PISTON | SKIP_BLOCK_ENTITY_SIDEEFFECTS`).
    /// No `NOTIFY_LISTENERS` (clients), no `FORCE_STATE` (shape updates still run).
    const DEST_PLACEHOLDER_FLAGS: BlockFlags =
        BlockFlags::MOVED.union(BlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK);

    /// Retracting body: vanilla 276
    /// (`UPDATE_INVISIBLE | UPDATE_KNOWN_SHAPE | SKIP_BLOCK_ENTITY_SIDEEFFECTS`).
    const RETRACT_BODY_FLAGS: BlockFlags =
        BlockFlags::FORCE_STATE.union(BlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK);

    /// Vanilla `state.getBlock() instanceof PistonBaseBlock`.
    #[must_use]
    pub fn is_base(block: &Block) -> bool {
        block == &Block::PISTON || block == &Block::STICKY_PISTON
    }

    #[must_use]
    pub fn is_sticky(block: &Block) -> bool {
        block == &Block::STICKY_PISTON
    }

    #[must_use]
    const fn type_from_sticky(sticky: bool) -> PistonType {
        if sticky {
            PistonType::Sticky
        } else {
            PistonType::Normal
        }
    }

    #[must_use]
    pub fn piston_type(block: &Block) -> PistonType {
        Self::type_from_sticky(Self::is_sticky(block))
    }

    #[must_use]
    pub fn is_movable(
        world: &World,
        pos: &BlockPos,
        block: &Block,
        state: &BlockState,
        dir: BlockDirection,
        can_break: bool,
        piston_dir: BlockDirection,
    ) -> bool {
        // Vanilla `PistonBaseBlock.isPushable`: outside the height range first (not the
        // world-edge `dir` tests). Air at minY/maxY is pushable so a down/up piston can
        // extend into the bottom/top layer; a non-air cell there is not.
        if !world.is_in_height_limit(pos.0.y) {
            return false;
        }
        if state.is_air() {
            return true;
        }
        // Vanilla hardcodes these four. `MOVING_PISTON` is not on that list; it fails
        // `hasBlockEntity` below. Listed here because `newBlockEntity` is null.
        if block == &Block::OBSIDIAN
            || block == &Block::CRYING_OBSIDIAN
            || block == &Block::RESPAWN_ANCHOR
            || block == &Block::REINFORCED_DEEPSLATE
            || block == &Block::MOVING_PISTON
        {
            return false;
        }
        if (dir == BlockDirection::Down && pos.0.y == world.get_bottom_y())
            || (dir == BlockDirection::Up && pos.0.y == world.get_top_y())
        {
            return false;
        }
        if Self::is_base(block) {
            let props = PistonProps::from_state_id(state.id, block);
            return !props.extended;
        }
        #[expect(clippy::float_cmp)]
        if state.hardness == -1.0 {
            return false;
        }
        match state.piston_behavior {
            PistonBehavior::Destroy => return can_break,
            PistonBehavior::Block => return false,
            PistonBehavior::PushOnly => return dir == piston_dir,
            _ => {}
        }
        // Vanilla `BlockState.hasBlockEntity()`.
        state.block_entity_type == u16::MAX
    }
}

impl BlockBehaviour for PistonBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = PistonProps::default(args.block);
        props.extended = false;
        props.facing = args.player.get_entity().get_facing().opposite();
        props.to_state_id(args.block)
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        let props = PistonProps::from_state_id(args.state.id, args.block);
        let pos = args
            .position
            .offset(props.facing.to_block_direction().to_offset());
        let (block_to_check, block_to_check_state_id) = args.world.get_block_and_state_id(&pos);
        if &Block::PISTON_HEAD == block_to_check {
            let head_props =
                PistonHeadProperties::from_state_id(block_to_check_state_id, block_to_check);

            if head_props.facing.to_block_direction() != props.facing.to_block_direction() {
                return;
            }

            args.world.break_block(&pos, None, BlockFlags::SKIP_DROPS);
        } else if &Block::MOVING_PISTON == block_to_check {
            args.world.break_block(&pos, None, BlockFlags::SKIP_DROPS);
        }
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        if args.old_state_id == args.state_id {
            return;
        }
        check_if_extend(args.world, args.block, args.position);
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        check_if_extend(args.world, args.block, args.position);
    }

    fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        let block_id = args.block.id;
        let block = Block::from_id(block_id);
        Self::handle_synced_block_event(block, args.world, args.position, args.r#type, args.data)
    }
}

impl PistonBlock {
    #[expect(clippy::too_many_lines)]
    fn handle_synced_block_event(
        block: &Block,
        world: &Arc<World>,
        pos: &BlockPos,
        r#type: u8,
        data: u8,
    ) -> bool {
        let state = world.get_block_state(pos);
        let mut props = PistonProps::from_state_id(state.id, block);
        let dir = props.facing.to_block_direction();

        let sticky = Self::is_sticky(block);

        let should_extend = should_extend(world, pos, dir);
        if should_extend && (r#type == Self::TRIGGER_CONTRACT || r#type == Self::TRIGGER_DROP) {
            props.extended = true;
            world.set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_LISTENERS);
            return false;
        }

        // Signal dropped in the tick between `checkIfExtend` and this event.
        if !should_extend && r#type == Self::TRIGGER_EXTEND {
            return false;
        }

        if r#type == Self::TRIGGER_EXTEND {
            let mut event =
                crate::plugin::api::events::block::block_piston::BlockPistonExtendEvent::new(
                    *pos,
                    format!("{dir:?}"),
                );
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            if event.cancelled {
                return false;
            }

            if !move_blocks(world, dir, pos, true, sticky) {
                return false;
            }
            props.extended = true;
            world.set_block_state(
                pos,
                props.to_state_id(block),
                BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
            );
            world.defer_live_block_change(*pos);
            // Play piston extend sound
            let pitch = rand::rng().random_range(0.6f32..0.85);
            world.play_sound_fine(
                Sound::BlockPistonExtend,
                SoundCategory::Blocks,
                &pos.to_centered_f64(),
                0.5,
                pitch,
            );
            return true;
        }

        let mut event =
            crate::plugin::api::events::block::block_piston::BlockPistonRetractEvent::new(
                *pos,
                format!("{dir:?}"),
            );
        if let Some(server) = world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            return false;
        }

        let extended_pos = pos.offset(dir.to_offset());

        // Live instance only: `get_block_entity` can rebuild from NBT at progress 0.
        if let Some(block_entity) = world.get_live_block_entity(&extended_pos)
            && let Some(piston) = block_entity.as_any().downcast_ref::<PistonBlockEntity>()
        {
            piston.finish(world);
        }

        world.set_block_state(
            pos,
            moving_piston_placeholder(dir, Some(Self::piston_type(block))),
            Self::RETRACT_BODY_FLAGS,
        );

        let mut props = PistonProps::default(block);
        props.facing = BlockDirection::by_index((data & 7) as usize)
            .unwrap_or(BlockDirection::North)
            .to_facing();

        world.add_block_entity(Arc::new(PistonBlockEntity::new(
            *pos,
            dir,
            BlockState::from_id(props.to_state_id(block)),
            false,
            true,
        )));

        // Vanilla `updateNeighborsAt` then `updateNeighbourShapes(..., 2)` after 276.
        // `FORCE_STATE` skipped shape updates on the write.
        world.update_neighbors(pos, None);
        world
            .block_registry
            .update_neighbors(world, pos, BlockFlags::NOTIFY_LISTENERS);
        if sticky {
            let pull_pos = pos.offset_dir(dir.to_offset(), 2);
            let (block, state) = world.get_block_and_state(&pull_pos);
            // Vanilla `pistonPiece`: finish an extending placeholder two cells out;
            // this retract does not pull (0-tick drop).
            let piston_piece = if block == &Block::MOVING_PISTON
                && let Some(entity) = world.get_live_block_entity(&pull_pos)
                && let Some(piston) = entity.as_any().downcast_ref::<PistonBlockEntity>()
                && piston.facing == dir
                && piston.extending
            {
                piston.finish(world);
                true
            } else {
                false
            };
            if !piston_piece {
                // Vanilla `b0 != 1` (`TRIGGER_DROP`). Event type is `r#type`;
                // `data` is facing (`b1`).
                if r#type != Self::TRIGGER_CONTRACT
                    || state.is_air()
                    || !Self::is_movable(world, &pull_pos, block, state, dir.opposite(), false, dir)
                    || (state.piston_behavior != PistonBehavior::Normal && !Self::is_base(block))
                {
                    world.set_block_state(
                        &extended_pos,
                        Block::AIR.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    );
                    world.defer_live_block_change(extended_pos);
                } else {
                    move_blocks(world, dir, pos, false, sticky);
                }
            }
        } else {
            world.set_block_state(
                &extended_pos,
                Block::AIR.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
            world.defer_live_block_change(extended_pos);
        }
        // Play piston contract sound
        let pitch = rand::rng().random_range(0.6f32..0.75);
        world.play_sound_fine(
            Sound::BlockPistonContract,
            SoundCategory::Blocks,
            &pos.to_centered_f64(),
            0.5,
            pitch,
        );
        true
    }
}

fn should_extend(world: &World, block_pos: &BlockPos, piston_dir: BlockDirection) -> bool {
    for dir in BlockDirection::all() {
        let neighbor_pos = block_pos.offset(dir.to_offset());
        let (block, state) = world.get_block_and_state(&neighbor_pos);
        // Pistons can't be powered from the same direction as they are facing
        if dir == piston_dir || !is_emitting_redstone_power(block, state, world, &neighbor_pos, dir)
        {
            continue;
        }
        return true;
    }
    // Vanilla `getNeighborSignal`: `level.hasSignal(pos, Direction.DOWN)` on this cell.
    // Solid cells fold in `getDirectSignalTo` (quasi-connectivity from the skip face).
    let (block, state) = world.get_block_and_state(block_pos);
    if is_emitting_redstone_power(block, state, world, block_pos, BlockDirection::Down) {
        return true;
    }
    for dir in BlockDirection::all() {
        let neighbor_pos = block_pos.up().offset(dir.to_offset());
        let (block, state) = world.get_block_and_state(&neighbor_pos);
        if dir == BlockDirection::Down
            || !is_emitting_redstone_power(block, state, world, &neighbor_pos, dir)
        {
            continue;
        }
        return true;
    }
    false
}

fn check_if_extend(world: &Arc<World>, block: &Block, block_pos: &BlockPos) {
    let state = world.get_block_state(block_pos);
    let props = PistonProps::from_state_id(state.id, block);
    let dir = props.facing.to_block_direction();
    let should_extend = should_extend(world, block_pos, dir);

    if should_extend && !props.extended {
        if PistonHandler::new(world, *block_pos, dir, true).calculate_push() {
            world.add_synced_block_event(*block_pos, PistonBlock::TRIGGER_EXTEND, dir.to_index());
        }
    } else if !should_extend && props.extended {
        let new_pos = block_pos.offset_dir(dir.to_offset(), 2);
        let (new_block, new_state) = world.get_block_and_state_id(&new_pos);
        let mut r#type = PistonBlock::TRIGGER_CONTRACT;

        if new_block == &Block::MOVING_PISTON {
            let new_props = MovingPistonLikeProperties::from_state_id(new_state, new_block);
            if new_props.facing == props.facing
                && let Some(entity) = world.get_live_block_entity(&new_pos)
                && let Some(piston) = entity.as_any().downcast_ref::<PistonBlockEntity>()
                && piston.should_drop_instead_of_pull(world)
            {
                r#type = PistonBlock::TRIGGER_DROP;
            }
        }
        world.add_synced_block_event(*block_pos, r#type, dir.to_index());
    }
}

/// `MOVING_PISTON` placeholder. Dest cells leave `TYPE` at default; arm/body set sticky/normal.
fn moving_piston_placeholder(dir: BlockDirection, piston_type: Option<PistonType>) -> BlockStateId {
    let mut props = MovingPistonLikeProperties::default(&Block::MOVING_PISTON);
    props.facing = dir.to_facing();
    if let Some(piston_type) = piston_type {
        props.r#type = piston_type;
    }
    props.to_state_id(&Block::MOVING_PISTON)
}

#[expect(clippy::too_many_lines)]
fn move_blocks(
    world: &Arc<World>,
    dir: BlockDirection,
    block_pos: &BlockPos,
    extending: bool,
    sticky: bool,
) -> bool {
    let extended_pos = block_pos.offset(dir.to_offset());
    if !extending && world.get_block(&extended_pos) == &Block::PISTON_HEAD {
        // Vanilla `setBlock(..., 276)`: `UPDATE_INVISIBLE`.
        world.set_block_state(
            &extended_pos,
            Block::AIR.default_state.id,
            PistonBlock::RETRACT_BODY_FLAGS,
        );
        world.defer_live_block_change(extended_pos);
    }
    let mut handler = PistonHandler::new(world, *block_pos, dir, extending);
    if !handler.calculate_push() {
        return false;
    }

    let mut moved_blocks_map: FxHashMap<BlockPos, &BlockState> = FxHashMap::default();
    let moved_blocks: Vec<BlockPos> = handler.moved_blocks;

    let mut moved_block_states: Vec<&BlockState> = Vec::new();

    for &block_pos in &moved_blocks {
        let block_state = world.get_block_state(&block_pos);
        moved_block_states.push(block_state);
        moved_blocks_map.insert(block_pos, block_state);
    }

    let broken_blocks: Vec<BlockPos> = handler.broken_blocks;
    let mut affected_block_states: Vec<&BlockState> =
        Vec::with_capacity(moved_blocks.len() + broken_blocks.len());
    let move_direction = if extending { dir } else { dir.opposite() };

    for &broken_block_pos in broken_blocks.iter().rev() {
        let block_state = world.get_block_state(&broken_block_pos);
        world.break_block(
            &broken_block_pos,
            None,
            BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE,
        );
        affected_block_states.push(block_state);
    }

    for (index, &moved_block_pos) in moved_blocks.iter().rev().enumerate() {
        let block_state = world.get_block_state(&moved_block_pos);
        let target_pos = moved_block_pos.offset(move_direction.to_offset());
        moved_blocks_map.remove(&target_pos);

        world.set_block_state(
            &target_pos,
            moving_piston_placeholder(dir, None),
            PistonBlock::DEST_PLACEHOLDER_FLAGS,
        );

        if let Some(moved_state) = moved_block_states.get(moved_blocks.len() - 1 - index) {
            world.add_block_entity(Arc::new(PistonBlockEntity::new(
                target_pos,
                dir.to_facing().to_block_direction(),
                moved_state,
                extending,
                false,
            )));
        }
        affected_block_states.push(block_state);
    }

    if extending {
        let piston_type = PistonBlock::type_from_sticky(sticky);
        moved_blocks_map.remove(&extended_pos);
        world.set_block_state(
            &extended_pos,
            moving_piston_placeholder(dir, Some(piston_type)),
            PistonBlock::DEST_PLACEHOLDER_FLAGS,
        );
        let mut props = PistonHeadLikeProperties::default(&Block::PISTON_HEAD);
        props.facing = dir.to_facing();
        props.r#type = piston_type;
        world.add_block_entity(Arc::new(PistonBlockEntity::new(
            extended_pos,
            dir.to_facing().to_block_direction(),
            BlockState::from_id(props.to_state_id(&Block::PISTON_HEAD)),
            true,
            true,
        )));
    }

    let air_state = Block::AIR.default_state.id;
    for &pos in moved_blocks_map.keys() {
        // Vanilla `setBlock(pos, AIR, 82)`. The client must not see this air before
        // `CBlockEvent` runs `moveBlocks` (dest `moving_piston` BE + offset collision).
        // `set_block_state` always queues a `CBlockUpdate`; drop it. The event packet
        // is the client write. `ChunkHolder.broadcastChanges` would send 82 next tick.
        world.set_block_state(
            &pos,
            air_state,
            BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE | BlockFlags::MOVED,
        );
        world.discard_queued_block_change(pos);
    }

    for (pos, state) in &moved_blocks_map {
        world.block_registry.prepare(
            world,
            pos,
            Block::from_state_id(state.id),
            state.id,
            BlockFlags::NOTIFY_LISTENERS,
        );
        // Vanilla `air.updateNeighbourShapes(level, pos, 2)`: shape only. Block updates
        // wait for the `moved_blocks` loop after every placeholder and air fill is written.
        world
            .block_registry
            .update_neighbors(world, pos, BlockFlags::NOTIFY_LISTENERS);
        world.block_registry.prepare(
            world,
            pos,
            &Block::AIR,
            air_state,
            BlockFlags::NOTIFY_LISTENERS,
        );
    }

    for (i, &broken_block_pos) in broken_blocks.iter().rev().enumerate() {
        if let Some(block_state) = affected_block_states.get(i) {
            world.block_registry.on_state_replaced(
                world,
                Block::from_state_id(block_state.id),
                &broken_block_pos,
                block_state.id, // ?
                false,
            );
            world.block_registry.prepare(
                world,
                &broken_block_pos,
                Block::from_state_id(block_state.id),
                block_state.id,
                BlockFlags::NOTIFY_LISTENERS,
            );
            // Vanilla `updateNeighborsAt(pos, toUpdate[i].getBlock())`: the captured pre-break
            // block, not air.
            world.update_neighbors_from(
                &broken_block_pos,
                Block::from_state_id(block_state.id),
                None,
            );
        }
    }
    for (index, &moved_block_pos) in moved_blocks.iter().rev().enumerate() {
        // Captured pre-move state; the cell is air or another placeholder by now.
        if let Some(block_state) = affected_block_states.get(broken_blocks.len() + index) {
            world.update_neighbors_from(
                &moved_block_pos,
                Block::from_state_id(block_state.id),
                None,
            );
        }
    }

    if extending {
        // Vanilla `level.updateNeighborsAt(armPos, Blocks.PISTON_HEAD)`: the arm is still
        // MOVING_PISTON, so the source is explicit.
        world.update_neighbors_from(&extended_pos, &Block::PISTON_HEAD, None);
    }

    for &pos in moved_blocks_map.keys() {
        world.discard_queued_block_change(pos);
    }
    for &pos in &broken_blocks {
        world.defer_live_block_change(pos);
    }

    true
}
