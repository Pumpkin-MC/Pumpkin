use std::{collections::HashMap, sync::Arc};

use crate::{block::blocks::redstone::is_emitting_redstone_power, entity::player::Player};
use async_trait::async_trait;
use pumpkin_data::block::{
    Block, BlockProperties, BlockState, Boolean, MovingPistonLikeProperties,
    PistonHeadLikeProperties, PistonType, get_block_by_state_id, get_state_by_state_id,
};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    block::{
        BlockDirection, FacingExt,
        entities::{BlockEntity, piston::PistonBlockEntity},
    },
    world::BlockFlags,
};

use crate::{
    block::pumpkin_block::{BlockMetadata, PumpkinBlock},
    server::Server,
    world::World,
};

use super::{PistonHandler, piston_extension::MovingPistonProps};

pub(crate) type PistonProps = pumpkin_data::block::StickyPistonLikeProperties;

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
    pub fn is_movable(block: &Block, state: &BlockState) -> bool {
        // TODO: more checks
        if state.air {
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
        true
    }
}

#[async_trait]
impl PumpkinBlock for PistonBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        block: &Block,
        _face: &BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        player: &Player,
        _other: bool,
    ) -> BlockStateId {
        let mut props = PistonProps::default(block);
        props.extended = Boolean::False;
        props.facing = player.living_entity.entity.get_facing().opposite();
        props.to_state_id(block)
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        pos: &BlockPos,
        old_state_id: BlockStateId,
        _notify: bool,
    ) {
        if old_state_id == state_id {
            return;
        }
        try_move(world, block, pos).await;
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        try_move(world, block, block_pos).await;
    }

    async fn on_synced_block_event(
        &self,
        block: &Block,
        world: &Arc<World>,
        pos: &BlockPos,
        r#type: u8,
        _data: u8,
    ) -> bool {
        let state = world.get_block_state(pos).await.unwrap();
        let mut props = PistonProps::from_state_id(state.id, block);
        let dir = props.facing.to_block_direction();

        // I don't think this is optimal ?
        let sticky = block == &Block::STICKY_PISTON;

        let should_extend = should_extend(world, block, &state, pos, dir).await;
        if should_extend && (r#type == 1 || r#type == 2) {
            props.extended = Boolean::True;
            world
                .set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_LISTENERS)
                .await;
            return false;
        }

        // This may prevents when something happend in the one tick before this function got called
        if !should_extend && r#type == 0 {
            return false;
        }

        if r#type == 0 {
            if !move_piston(world, dir, pos, true, sticky).await {
                return false;
            }
            props.extended = Boolean::True;
            world
                .set_block_state(
                    pos,
                    props.to_state_id(block),
                    BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
                )
                .await;
            return true;
        } else {
            let extended_pos = pos.offset(dir.to_offset());

            if let Some((block_entity_nbt, _block_entity)) =
                world.get_block_entity(&extended_pos).await
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

            let state = props.to_state_id(&Block::MOVING_PISTON);

            world
                .set_block_state(&pos, state, BlockFlags::FORCE_STATE)
                .await;

            world
                .add_block_entity(Arc::new(PistonBlockEntity {
                    position: *pos,
                    facing: dir.to_facing().to_block_direction(),
                    pushed_block_state: get_state_by_state_id(state).unwrap(),
                    current_progress: 0.0.into(),
                    last_progress: 0.0.into(),
                    extending: false,
                    source: true,
                }))
                .await;
            world.update_neighbors(pos, None).await;
            if sticky {
            } else {
                world
                    .set_block_state(
                        &extended_pos,
                        Block::AIR.default_state_id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        }
        return true;
    }
}

async fn should_extend(
    world: &Arc<World>,
    block: &Block,
    state: &BlockState,
    block_pos: &BlockPos,
    piston_dir: BlockDirection,
) -> bool {
    // Pistons can't be powered from the same direction as they are facing
    for dir in BlockDirection::all() {
        if dir == piston_dir
            || !is_emitting_redstone_power(
                block,
                &state,
                world,
                &block_pos.offset(dir.to_offset()),
                &dir,
            )
            .await
        {
            continue;
        }
        return true;
    }
    if is_emitting_redstone_power(block, &state, world, block_pos, &BlockDirection::Down).await {
        return true;
    }
    for dir in BlockDirection::all() {
        if dir == BlockDirection::Down
            || !is_emitting_redstone_power(
                block,
                &state,
                world,
                &block_pos.up().offset(dir.to_offset()),
                &dir,
            )
            .await
        {
            continue;
        }
        return true;
    }
    false
}

async fn try_move(world: &Arc<World>, block: &Block, block_pos: &BlockPos) {
    let state = world.get_block_state(block_pos).await.unwrap();
    let props = PistonProps::from_state_id(state.id, block);
    let dir = props.facing.to_block_direction();
    let should_extent = should_extend(world, block, &state, block_pos, dir).await;

    if should_extent && !props.extended.to_bool() {
        if PistonHandler::new(&world, *block_pos, dir, true)
            .calculate_push()
            .await
        {
            world
                .add_synced_block_event(*block_pos, 0, dir.to_index() as u8)
                .await;
        }
    } else if !should_extent && props.extended.to_bool() {
        let new_pos = block_pos.offset_dir(dir.to_offset(), 2);
        let (new_block, new_state) = world.get_block_and_block_state(&new_pos).await.unwrap();
        let mut r#type = 1;
        if new_block == Block::MOVING_PISTON {
            let new_props = MovingPistonProps::from_state_id(new_state.id, &new_block);
            // TODO: check more things
            if new_props.facing == props.facing {
                r#type = 2;
            }
        }
        world
            .add_synced_block_event(*block_pos, r#type, dir.to_index() as u8)
            .await;
    }
}

async fn move_piston(
    world: &Arc<World>,
    dir: BlockDirection,
    block_pos: &BlockPos,
    extend: bool,
    sticky: bool,
) -> bool {
    let extended_pos = block_pos.offset(dir.to_offset());
    if !extend && world.get_block(&extended_pos).await.unwrap() == Block::PISTON_HEAD {
        world
            .set_block_state(
                &extended_pos,
                Block::AIR.default_state_id,
                BlockFlags::FORCE_STATE,
            )
            .await;
    }
    let mut handler = PistonHandler::new(world, *block_pos, dir, extend);
    if !handler.calculate_push().await {
        return false;
    }

    let mut moved_blocks_map: HashMap<BlockPos, BlockState> = HashMap::new();
    let moved_blocks: Vec<BlockPos> = handler.moved_blocks;
    dbg!(&moved_blocks);

    let mut moved_block_states: Vec<BlockState> = Vec::new();

    for &block_pos in &moved_blocks {
        let block_state = world.get_block_state(&block_pos).await.unwrap();
        dbg!(block_state.id);
        moved_block_states.push(block_state.clone());
        moved_blocks_map.insert(block_pos, block_state);
    }

    let broken_blocks: Vec<BlockPos> = handler.broken_blocks;
    dbg!(&broken_blocks);
    let mut affected_block_states: Vec<BlockState> =
        Vec::with_capacity(moved_blocks.len() + broken_blocks.len());
    let move_direction = if extend { dir } else { dir.opposite() };

    for &broken_block_pos in broken_blocks.iter().rev() {
        let block_state = world.get_block_state(&broken_block_pos).await.unwrap();
        world
            .break_block(&broken_block_pos, None, BlockFlags::empty())
            .await;
        affected_block_states.push(block_state);
    }

    for (index, &moved_block_pos) in moved_blocks.iter().rev().enumerate() {
        let block_state = world.get_block_state(&moved_block_pos).await.unwrap();
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
                    pushed_block_state: moved_state.clone(),
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

    let air_state = Block::AIR.default_state_id;
    for &pos in moved_blocks_map.keys() {
        world
            .set_block_state(
                &pos,
                air_state,
                BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE | BlockFlags::MOVED,
            )
            .await;
    }

    for (pos, state) in moved_blocks_map.iter() {
        // state.prepare(world, pos, BlockFlags::NOTIFY_LISTENERS);
        world
            .block_registry
            .prepare(
                world,
                pos,
                &get_block_by_state_id(state.id).unwrap(),
                state.id,
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;
        world.update_neighbors(&pos, None).await;
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
        if let Some(block_state) = affected_block_states.get(i).cloned() {
            world
                .block_registry
                .on_state_replaced(
                    world,
                    &get_block_by_state_id(block_state.id).unwrap(),
                    broken_block_pos,
                    block_state.id, // ?
                    false,
                )
                .await;
            world
                .block_registry
                .prepare(
                    world,
                    &broken_block_pos,
                    &get_block_by_state_id(block_state.id).unwrap(),
                    block_state.id,
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
            world.update_neighbors(&broken_block_pos, None).await;
        }
    }

    for (i, &moved_block_pos) in moved_blocks.iter().rev().enumerate() {
        world.update_neighbors(&moved_block_pos, None).await;
    }

    // world.update_neighbors(&extended_pos, None).await;

    true
}
