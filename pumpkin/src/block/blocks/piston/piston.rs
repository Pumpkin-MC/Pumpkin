use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection, BlockState, FacingExt,
    block_properties::{
        BlockProperties, MovingPistonLikeProperties, PistonHeadLikeProperties, PistonType,
        get_block_by_state_id, get_state_by_state_id,
    },
    block_state::PistonBehavior,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    block::entities::{BlockEntity, has_block_block_entity, piston::PistonBlockEntity},
    world::BlockFlags,
};

use crate::{
    block::{
        blocks::redstone::is_emitting_redstone_power,
        pumpkin_block::{
            BlockMetadata, OnNeighborUpdateArgs, OnPlaceArgs, OnSyncedBlockEventArgs, PlacedArgs,
            PumpkinBlock,
        },
    },
    world::World,
};

use super::PistonHandler;

pub(crate) type PistonProps = pumpkin_data::block_properties::StickyPistonLikeProperties;

pub struct PistonBlock;

impl BlockMetadata for PistonBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::PISTON.name, Block::STICKY_PISTON.name]
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
            let props = PistonProps::from_state_id(state.id, block);
            if props.extended {
                return false;
            }
        } else {
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
        }
        !has_block_block_entity(block)
    }
}

#[async_trait]
impl PumpkinBlock for PistonBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = PistonProps::default(args.block);
        props.extended = false;
        props.facing = args.player.living_entity.entity.get_facing().opposite();
        props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        if args.old_state_id == args.state_id {
            return;
        }
        try_move(args.world, args.block, args.location).await;
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        try_move(args.world, args.block, args.location).await;
    }

    #[expect(clippy::too_many_lines)]
    async fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        let (block, world, pos, r#type, data) = (
            args.block,
            args.world,
            args.location,
            args.r#type,
            args.data,
        );

        let state = world.get_block_state(pos).await;
        let mut props = PistonProps::from_state_id(state.id, block);
        let dir = props.facing.to_block_direction();

        // I don't think this is optimal ?
        let sticky = block == &Block::STICKY_PISTON;

        let should_extend = should_extend(world, pos, dir).await;
        if should_extend && (r#type == 1 || r#type == 2) {
            props.extended = true;
            world
                .set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_LISTENERS)
                .await;
            return false;
        }

        // This may prevents when something happens in the one tick before this function got called
        if !should_extend && r#type == 0 {
            return false;
        }

        // Extend Piston
        if r#type == 0 {
            if !move_piston(world, dir, pos, true, sticky).await {
                return false;
            }
            props.extended = true;
            world
                .set_block_state(
                    pos,
                    props.to_state_id(block),
                    BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
                )
                .await;
            return true;
        }
        // Reduce Piston

        let extended_pos = pos.offset(dir.to_offset());

        if let Some((block_entity_nbt, _block_entity)) = world.get_block_entity(&extended_pos).await
        {
            PistonBlockEntity::from_nbt(&block_entity_nbt, extended_pos)
                .finish(world.clone())
                .await;
        }

        let mut props = MovingPistonLikeProperties::default(&Block::MOVING_PISTON);
        props.facing = dir.to_facing();
        props.r#type = if sticky {
            PistonType::Sticky
        } else {
            PistonType::Normal
        };

        world
            .set_block_state(
                pos,
                props.to_state_id(&Block::MOVING_PISTON),
                BlockFlags::FORCE_STATE,
            )
            .await;

        let mut props = PistonProps::default(block);
        props.facing = BlockDirection::by_index((data & 7) as usize)
            .unwrap()
            .to_facing();

        world
            .add_block_entity(Arc::new(PistonBlockEntity {
                position: *pos,
                facing: dir,
                pushed_block_state: get_state_by_state_id(props.to_state_id(block)).unwrap(),
                current_progress: 0.0.into(),
                last_progress: 0.0.into(),
                extending: false,
                source: true,
            }))
            .await;

        world.update_neighbors(pos, None).await;
        if sticky {
            let pos = pos.offset_dir(dir.to_offset(), 2);
            let (block, state) = world.get_block_and_block_state(&pos).await;
            let mut bl2 = false;
            if block == &Block::MOVING_PISTON {
                if let Some(entity) = world.get_block_entity(&pos).await {
                    let piston = PistonBlockEntity::from_nbt(&entity.0, pos);
                    if piston.facing == dir && piston.extending {
                        piston.finish(world.clone()).await;
                        bl2 = true;
                    }
                }
            }
            if !bl2 {
                if r#type == 1
                    && !state.is_air()
                    && Self::is_movable(block, state, dir, false, dir)
                    && (state.piston_behavior == PistonBehavior::Normal
                        || block == &Block::PISTON
                        || block == &Block::STICKY_PISTON)
                {
                    move_piston(world, dir, &pos, false, sticky).await;
                } else {
                    // remove
                    world
                        .set_block_state(
                            &extended_pos,
                            Block::AIR.default_state.id,
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                }
            }
        } else {
            // remove
            world
                .set_block_state(
                    &extended_pos,
                    Block::AIR.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
        return true;
    }
}

async fn should_extend(world: &World, block_pos: &BlockPos, piston_dir: BlockDirection) -> bool {
    for dir in BlockDirection::all() {
        let neighbor_pos = block_pos.offset(dir.to_offset());
        let (block, state) = world.get_block_and_block_state(&neighbor_pos).await;
        // Pistons can't be powered from the same direction as they are facing
        if dir == piston_dir
            || !is_emitting_redstone_power(block, state, world, &neighbor_pos, dir).await
        {
            continue;
        }
        return true;
    }
    let neighbor_pos = block_pos.offset(BlockDirection::Down.to_offset());
    let (block, state) = world.get_block_and_block_state(&neighbor_pos).await;
    if is_emitting_redstone_power(block, state, world, block_pos, BlockDirection::Down).await {
        return true;
    }
    for dir in BlockDirection::all() {
        let neighbor_pos = block_pos.up().offset(dir.to_offset());
        let (block, state) = world.get_block_and_block_state(&neighbor_pos).await;
        if dir == BlockDirection::Down
            || !is_emitting_redstone_power(block, state, world, &neighbor_pos, dir).await
        {
            continue;
        }
        return true;
    }
    false
}

async fn try_move(world: &Arc<World>, block: &Block, block_pos: &BlockPos) {
    let state = world.get_block_state(block_pos).await;
    let props = PistonProps::from_state_id(state.id, block);
    let dir = props.facing.to_block_direction();
    let should_extent = should_extend(world, block_pos, dir).await;

    if should_extent && !props.extended {
        if PistonHandler::new(world, *block_pos, dir, true)
            .calculate_push()
            .await
        {
            world
                .add_synced_block_event(*block_pos, 0, dir.to_index())
                .await;
        }
    } else if !should_extent && props.extended {
        let new_pos = block_pos.offset_dir(dir.to_offset(), 2);
        let (new_block, new_state) = world.get_block_and_block_state(&new_pos).await;
        let mut r#type = 1;

        if new_block == &Block::MOVING_PISTON {
            let new_props = MovingPistonLikeProperties::from_state_id(new_state.id, new_block);
            if new_props.facing == props.facing {
                if let Some(entity) = world.get_block_entity(&new_pos).await {
                    let piston = PistonBlockEntity::from_nbt(&entity.0, new_pos);
                    if piston.extending && piston.current_progress.load() < 0.5
                    // TODO: more stuff...
                    {
                        // Piston reduced too quickly, if its a stick piston no blocks will be dragged
                        r#type = 2;
                    }
                }
            }
        }
        world
            .add_synced_block_event(*block_pos, r#type, dir.to_index())
            .await;
    }
}

#[expect(clippy::too_many_lines)]
async fn move_piston(
    world: &Arc<World>,
    dir: BlockDirection,
    block_pos: &BlockPos,
    extend: bool,
    sticky: bool,
) -> bool {
    let extended_pos = block_pos.offset(dir.to_offset());
    if !extend && world.get_block(&extended_pos).await == &Block::PISTON_HEAD {
        world
            .set_block_state(
                &extended_pos,
                Block::AIR.default_state.id,
                BlockFlags::FORCE_STATE,
            )
            .await;
    }
    let mut handler = PistonHandler::new(world, *block_pos, dir, extend);
    if !handler.calculate_push().await {
        return false;
    }

    let mut moved_blocks_map: HashMap<BlockPos, &BlockState> = HashMap::new();
    let moved_blocks: Vec<BlockPos> = handler.moved_blocks;

    let mut moved_block_states: Vec<&BlockState> = Vec::new();

    for &block_pos in &moved_blocks {
        let block_state = world.get_block_state(&block_pos).await;
        moved_block_states.push(block_state);
        moved_blocks_map.insert(block_pos, block_state);
    }

    let broken_blocks: Vec<BlockPos> = handler.broken_blocks;
    let mut affected_block_states: Vec<&BlockState> =
        Vec::with_capacity(moved_blocks.len() + broken_blocks.len());
    let move_direction = if extend { dir } else { dir.opposite() };

    for &broken_block_pos in broken_blocks.iter().rev() {
        let block_state = world.get_block_state(&broken_block_pos).await;
        world
            .break_block(
                &broken_block_pos,
                None,
                BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE,
            )
            .await;
        affected_block_states.push(block_state);
    }

    for (index, &moved_block_pos) in moved_blocks.iter().rev().enumerate() {
        let block_state = world.get_block_state(&moved_block_pos).await;
        let target_pos = moved_block_pos.offset(move_direction.to_offset());
        moved_blocks_map.remove(&target_pos);

        let mut props = MovingPistonLikeProperties::default(&Block::MOVING_PISTON);
        props.facing = dir.to_facing();
        let state = props.to_state_id(&Block::MOVING_PISTON);

        world
            .set_block_state(&target_pos, state, BlockFlags::MOVED)
            .await;

        if let Some(moved_state) = moved_block_states.get(index) {
            world
                .add_block_entity(Arc::new(PistonBlockEntity {
                    position: extended_pos,
                    facing: dir.to_facing().to_block_direction(),
                    pushed_block_state: moved_state,
                    current_progress: 0.0.into(),
                    last_progress: 0.0.into(),
                    extending: extend,
                    source: false,
                }))
                .await;
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
        world
            .set_block_state(
                &extended_pos,
                props.to_state_id(&Block::MOVING_PISTON),
                BlockFlags::MOVED,
            )
            .await;
        let mut props = PistonHeadLikeProperties::default(&Block::PISTON_HEAD);
        props.facing = dir.to_facing();
        props.r#type = pistion_type;
        world
            .add_block_entity(Arc::new(PistonBlockEntity {
                position: extended_pos,
                facing: dir.to_facing().to_block_direction(),
                pushed_block_state: get_state_by_state_id(props.to_state_id(&Block::PISTON_HEAD))
                    .unwrap(),
                current_progress: 0.0.into(),
                last_progress: 0.0.into(),
                extending: true,
                source: true,
            }))
            .await;
    }

    let air_state = Block::AIR.default_state.id;
    for &pos in moved_blocks_map.keys() {
        world
            .set_block_state(
                &pos,
                air_state,
                BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE | BlockFlags::MOVED,
            )
            .await;
    }

    for (pos, state) in &moved_blocks_map {
        world
            .block_registry
            .prepare(
                world,
                pos,
                get_block_by_state_id(state.id).unwrap(),
                state.id,
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;
        world.update_neighbors(pos, None).await;
        world
            .block_registry
            .prepare(
                world,
                pos,
                &Block::AIR,
                air_state,
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;
    }

    for (i, &broken_block_pos) in broken_blocks.iter().rev().enumerate() {
        if let Some(block_state) = affected_block_states.get(i) {
            world
                .block_registry
                .on_state_replaced(
                    world,
                    get_block_by_state_id(block_state.id).unwrap(),
                    &broken_block_pos,
                    block_state.id, // ?
                    false,
                )
                .await;
            world
                .block_registry
                .prepare(
                    world,
                    &broken_block_pos,
                    get_block_by_state_id(block_state.id).unwrap(),
                    block_state.id,
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
            world.update_neighbors(&broken_block_pos, None).await;
        }
    }
    for &moved_block_pos in moved_blocks.iter().rev() {
        world.update_neighbors(&moved_block_pos, None).await;
    }

    if extend {
        world.update_neighbors(&extended_pos, None).await;
    }

    true
}
