use std::sync::Arc;

use crate::block::entities::{has_block_block_entity, piston::PistonBlockEntity};
use crate::entity::EntityBase;
use pumpkin_data::BlockId;
use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId, FacingExt,
    block_properties::{MovingPistonLikeProperties, PistonHeadLikeProperties, PistonType},
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
        OnSyncedBlockEventArgs, PathComputationType, PlacedArgs,
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
    #[must_use]
    pub fn is_movable(
        block: &Block,
        state: &BlockState,
        dir: BlockDirection,
        can_break: bool,
        piston_dir: BlockDirection,
    ) -> bool {
        // TODO: more checks
        if state.is_air() {
            return true;
        }
        // Vanilla hardcoded them aswell
        if block == &Block::OBSIDIAN
            || block == &Block::CRYING_OBSIDIAN
            || block == &Block::RESPAWN_ANCHOR
            || block == &Block::REINFORCED_DEEPSLATE
        {
            return false;
        }
        if block == &Block::PISTON || block == &Block::STICKY_PISTON {
            let props = PistonProps::from_state_id(state.id);
            // Extended pistons are immovable. Non-extended pistons are movable
            return !props.extended;
        }
        #[expect(clippy::float_cmp)]
        if state.hardness == -1.0 {
            return false;
        }
        match state.piston_behavior {
            pumpkin_data::block_state::PistonBehavior::Destroy => return can_break,
            pumpkin_data::block_state::PistonBehavior::Block => return false,
            pumpkin_data::block_state::PistonBehavior::PushOnly => return dir == piston_dir,
            _ => {}
        }
        !has_block_block_entity(block)
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
        let props = PistonProps::from_state_id(args.state.id);
        let pos = args
            .position
            .offset(props.facing.to_block_direction().to_offset());
        let (block_to_check, block_to_check_state_id) = args.world.get_block_and_state_id(&pos);
        if &Block::PISTON_HEAD == block_to_check {
            let head_props = PistonHeadProperties::from_state_id(block_to_check_state_id);

            if (head_props.facing.to_block_direction() != props.facing.to_block_direction())
                && &Block::PISTON_HEAD == block_to_check
            {
                //Then this is a head of some other piston.
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
        try_move(args.world, args.block, args.position);
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        try_move(args.world, args.block, args.position);
    }

    fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        let block_id = args.block.id;
        let block = Block::from_id(block_id);
        Self::handle_synced_block_event(block, args.world, args.position, args.r#type, args.data)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
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
        let mut props = PistonProps::from_state_id(state.id);
        let dir = props.facing.to_block_direction();

        // I don't think this is optimal ?
        let sticky = block == &Block::STICKY_PISTON;

        let should_extend = should_extend(world, pos, dir);
        if should_extend && (r#type == 1 || r#type == 2) {
            props.extended = true;
            world.set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_LISTENERS);
            return false;
        }

        // This may prevents when something happens in the one tick before this function got called
        if !should_extend && r#type == 0 {
            return false;
        }

        // Extend Piston
        if r#type == 0 {
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

            if !move_piston(world, dir, pos, true, sticky) {
                return false;
            }
            props.extended = true;
            world.set_block_state(
                pos,
                props.to_state_id(block),
                BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
            );
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
        // Reduce Piston

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

        if let Some(block_entity) = world.get_block_entity(&extended_pos)
            && let Some(piston) = block_entity.as_any().downcast_ref::<PistonBlockEntity>()
        {
            piston.finish(world);
        }

        let mut props = MovingPistonLikeProperties::default(&Block::MOVING_PISTON);
        props.facing = dir.to_facing();
        props.r#type = if sticky {
            PistonType::Sticky
        } else {
            PistonType::Normal
        };

        world.set_block_state(
            pos,
            props.to_state_id(&Block::MOVING_PISTON),
            BlockFlags::NOTIFY_ALL | BlockFlags::FORCE_STATE,
        );

        let mut props = PistonProps::default(block);
        props.facing = BlockDirection::by_index((data & 7) as usize)
            .unwrap_or(BlockDirection::North)
            .to_facing();

        world.add_block_entity(Arc::new(PistonBlockEntity {
            position: *pos,
            facing: dir,
            pushed_block_state: BlockState::from_id(props.to_state_id(block)),
            current_progress: 0.0.into(),
            last_progress: 0.0.into(),
            extending: false,
            source: true,
        }));

        world.set_block_state(
            &extended_pos,
            Block::AIR.default_state.id,
            BlockFlags::NOTIFY_ALL | BlockFlags::FORCE_STATE,
        );

        world.update_neighbors(pos, None);
        if sticky {
            let pull_pos = pos.offset_dir(dir.to_offset(), 2);
            let (block, state) = world.get_block_and_state(&pull_pos);
            if data == 2 {
                world.set_block_state(
                    &extended_pos,
                    Block::AIR.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );
            } else {
                let is_air = state.is_air();
                if !is_air
                    && (Self::is_movable(block, state, dir, false, dir.opposite())
                        || Self::is_movable(block, state, dir, false, dir))
                    && (state.piston_behavior == PistonBehavior::Normal
                        || block == &Block::PISTON
                        || block == &Block::STICKY_PISTON)
                {
                    move_piston(world, dir, pos, false, sticky);
                } else {
                    // remove
                    world.set_block_state(
                        &extended_pos,
                        Block::AIR.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    );
                }
            }
        } else {
            // remove
            world.set_block_state(
                &extended_pos,
                Block::AIR.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
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
    let neighbor_pos = block_pos.offset(BlockDirection::Down.to_offset());
    let (block, state) = world.get_block_and_state(&neighbor_pos);
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

pub fn try_move(world: &Arc<World>, _block: &Block, block_pos: &BlockPos) {
    let state = world.get_block_state(block_pos);
    let props = PistonProps::from_state_id(state.id);
    let dir = props.facing.to_block_direction();
    let should_extent = should_extend(world, block_pos, dir);

    if should_extent && !props.extended {
        if PistonHandler::new(world, *block_pos, dir, true).calculate_push() {
            world.add_synced_block_event(*block_pos, 0, dir.to_index());
        }
    } else if !should_extent && props.extended {
        let new_pos = block_pos.offset_dir(dir.to_offset(), 2);
        let (new_block, new_state) = world.get_block_and_state_id(&new_pos);
        let mut r#type = 1;

        if new_block == &Block::MOVING_PISTON {
            let new_props = MovingPistonLikeProperties::from_state_id(new_state);
            if new_props.facing == props.facing
                && let Some(entity) = world.get_block_entity(&new_pos)
            {
                let Some(piston) = entity.as_any().downcast_ref::<PistonBlockEntity>() else {
                    return;
                };
                if piston.extending && piston.current_progress.load() < 0.5
                // TODO: more stuff...
                {
                    // Piston reduced too quickly, if its a stick piston no blocks will be dragged
                    r#type = 2;
                }
            }
        }
        world.add_synced_block_event(*block_pos, r#type, dir.to_index());
    }
}

#[expect(clippy::too_many_lines)]
fn move_piston(
    world: &Arc<World>,
    dir: BlockDirection,
    block_pos: &BlockPos,
    extend: bool,
    sticky: bool,
) -> bool {
    let extended_pos = block_pos.offset(dir.to_offset());
    if !extend && world.get_block(&extended_pos) == &Block::PISTON_HEAD {
        world.set_block_state(
            &extended_pos,
            Block::AIR.default_state.id,
            BlockFlags::FORCE_STATE,
        );
    }
    let mut handler = PistonHandler::new(world, *block_pos, dir, extend);
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
    let move_direction = if extend { dir } else { dir.opposite() };

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

        let mut props = MovingPistonLikeProperties::default(&Block::MOVING_PISTON);
        props.facing = dir.to_facing();
        let state = props.to_state_id(&Block::MOVING_PISTON);

        world.set_block_state(
            &target_pos,
            state,
            BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
        );

        if let Some(moved_state) = moved_block_states.get(moved_blocks.len() - 1 - index) {
            world.add_block_entity(Arc::new(PistonBlockEntity {
                position: target_pos,
                facing: dir.to_facing().to_block_direction(),
                pushed_block_state: moved_state,
                current_progress: 0.0.into(),
                last_progress: 0.0.into(),
                extending: extend,
                source: false,
            }));
        }
        affected_block_states.push(block_state);
    }

    if extend {
        let pistion_type = if sticky {
            PistonType::Sticky
        } else {
            PistonType::Normal
        };
        let mut props = MovingPistonLikeProperties::default(&Block::MOVING_PISTON);
        props.facing = dir.to_facing();
        props.r#type = pistion_type;
        moved_blocks_map.remove(&extended_pos);
        world.set_block_state(
            &extended_pos,
            props.to_state_id(&Block::MOVING_PISTON),
            BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
        );
        let mut props = PistonHeadLikeProperties::default(&Block::PISTON_HEAD);
        props.facing = dir.to_facing();
        props.r#type = pistion_type;
        world.add_block_entity(Arc::new(PistonBlockEntity {
            position: extended_pos,
            facing: dir.to_facing().to_block_direction(),
            pushed_block_state: BlockState::from_id(props.to_state_id(&Block::PISTON_HEAD)),
            current_progress: 0.0.into(),
            last_progress: 0.0.into(),
            extending: true,
            source: true,
        }));
    }

    let air_state = Block::AIR.default_state.id;
    for &pos in moved_blocks_map.keys() {
        world.set_block_state(
            &pos,
            air_state,
            BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE | BlockFlags::MOVED,
        );
    }

    for (pos, state) in &moved_blocks_map {
        world.block_registry.prepare(
            world,
            pos,
            Block::from_state_id(state.id),
            state.id,
            BlockFlags::NOTIFY_LISTENERS,
        );
        world.update_neighbors(pos, None);
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
            world.update_neighbors(&broken_block_pos, None);
        }
    }
    for &moved_block_pos in moved_blocks.iter().rev() {
        world.update_neighbors(&moved_block_pos, None);
    }

    if extend {
        world.update_neighbors(&extended_pos, None);
    }

    true
}
