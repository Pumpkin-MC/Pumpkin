use std::sync::Arc;

use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::block::{
    Block, BlockProperties, BlockState, Boolean, MovingPistonLikeProperties, PistonType,
    get_state_by_state_id,
};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    block::{BlockDirection, FacingExt, entities::piston::PistonBlockEntity},
    world::BlockFlags,
};

use crate::{
    block::pumpkin_block::{BlockMetadata, PumpkinBlock},
    server::Server,
    world::World,
};

use super::is_emitting_redstone_power;

type PistonProps = pumpkin_data::block::StickyPistonLikeProperties;

pub struct PistonBlock;

impl BlockMetadata for PistonBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::PISTON.name, Block::STICKY_PISTON.name]
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
    let mut props = PistonProps::from_state_id(state.id, block);
    // I don't think this is optimal ?
    let sticky = block == &Block::STICKY_PISTON;
    let dir = props.facing.to_block_direction();
    let should_extent = should_extend(world, block, &state, block_pos, dir).await;
    dbg!(should_extent);

    if should_extent && !props.extended.to_bool() {
        if !move_piston(world, dir, block_pos, true, sticky).await {
            return;
        }
        props.extended = Boolean::True;
        world
            .set_block_state(
                block_pos,
                props.to_state_id(block),
                BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
            )
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
    if extend {
        let mut props = MovingPistonLikeProperties::default(&Block::MOVING_PISTON);
        props.facing = dir.to_facing();
        props.r#type = if sticky {
            PistonType::Sticky
        } else {
            PistonType::Normal
        };
        world
            .set_block_state(
                &extended_pos,
                props.to_state_id(&Block::MOVING_PISTON),
                BlockFlags::MOVED,
            )
            .await;
        //world.add_block_entity(PistonBlockEntity).await;
    }
    true
}
