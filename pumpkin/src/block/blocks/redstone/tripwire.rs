use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{BlockProperties, HorizontalFacing},
    item::Item,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::{BlockStateId, chunk::TickPriority, world::BlockFlags};

use crate::{
    block::{BlockIsReplacing, pumpkin_block::PumpkinBlock},
    entity::{EntityBase, player::Player},
    server::Server,
    world::World,
};

use super::tripwire_hook::TripwireHookBlock;

type TripwireProperties = pumpkin_data::block_properties::TripwireLikeProperties;
type TripwireHookProperties = pumpkin_data::block_properties::TripwireHookLikeProperties;

#[pumpkin_block("minecraft:tripwire")]
pub struct TripwireBlock;

#[async_trait]
impl PumpkinBlock for TripwireBlock {
    async fn on_entity_collision(
        &self,
        world: &Arc<World>,
        _entity: &dyn EntityBase,
        pos: BlockPos,
        block: Block,
        state: BlockState,
        _server: &Server,
    ) {
        let mut props = TripwireProperties::from_state_id(state.id, &block);
        if props.powered {
            return;
        }
        props.powered = true;

        let state_id = props.to_state_id(&block);
        world
            .set_block_state(&pos, state_id, BlockFlags::NOTIFY_ALL)
            .await;

        Self::update(world, &pos, state_id).await;

        world
            .schedule_block_tick(&block, pos, 10, TickPriority::Normal)
            .await;
    }

    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        _player: &Player,
        block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let [connect_north, connect_east, connect_south, connect_west] = [
            BlockDirection::North,
            BlockDirection::East,
            BlockDirection::South,
            BlockDirection::West,
        ]
        .map(async |dir| {
            let current_pos = block_pos.offset(dir.to_offset());
            let state_id = world.get_block_state_id(&current_pos).await;
            Self::should_connect_to(state_id, dir)
        });

        let mut props = TripwireProperties::from_state_id(block.default_state.id, block);

        props.north = connect_north.await;
        props.south = connect_south.await;
        props.west = connect_west.await;
        props.east = connect_east.await;

        props.to_state_id(block)
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        _block: &Block,
        state_id: BlockStateId,
        pos: &BlockPos,
        old_state_id: BlockStateId,
        _notify: bool,
    ) {
        if let (Some(old_block), Some(new_block)) = (
            Block::from_state_id(old_state_id),
            Block::from_state_id(state_id),
        ) {
            if old_block == new_block {
                return;
            }
        }
        Self::update(world, pos, state_id).await;
    }

    async fn broken(
        &self,
        block: &Block,
        player: &Arc<Player>,
        location: BlockPos,
        _server: &Server,
        world: Arc<World>,
        state: BlockState,
    ) {
        let has_shears = {
            let main_hand_item_stack = player.inventory().held_item();
            main_hand_item_stack
                .lock()
                .await
                .get_item()
                .eq(&Item::SHEARS)
        };
        if has_shears {
            let mut props = TripwireProperties::from_state_id(state.id, block);
            props.disarmed = true;
            world
                .set_block_state(&location, props.to_state_id(block), BlockFlags::empty())
                .await;
            // TODO world.emitGameEvent(player, GameEvent.SHEAR, pos);
        }
    }

    async fn get_state_for_neighbor_update(
        &self,
        _world: &World,
        block: &Block,
        state: BlockStateId,
        _pos: &BlockPos,
        direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        neighbor_state: BlockStateId,
    ) -> BlockStateId {
        direction.to_horizontal_facing().map_or(state, |facing| {
            let mut props = TripwireProperties::from_state_id(state, block);
            *match facing {
                HorizontalFacing::North => &mut props.north,
                HorizontalFacing::South => &mut props.south,
                HorizontalFacing::West => &mut props.west,
                HorizontalFacing::East => &mut props.east,
            } = Self::should_connect_to(neighbor_state, direction);
            props.to_state_id(block)
        })
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, block: &Block, pos: &BlockPos) {
        let state_id = world.get_block_state_id(pos).await;

        let mut props = TripwireProperties::from_state_id(state_id, block);
        if !props.powered {
            return;
        }

        let aabb = BoundingBox::from_block(pos);
        // TODO entity.canAvoidTraps()
        if world.get_entities_at_box(&aabb).await.is_empty()
            && world.get_players_at_box(&aabb).await.is_empty()
        {
            props.powered = false;
            let state_id = props.to_state_id(block);
            world
                .set_block_state(pos, state_id, BlockFlags::NOTIFY_ALL)
                .await;
            Self::update(world, pos, state_id).await;
        } else {
            world
                .schedule_block_tick(block, *pos, 10, TickPriority::Normal)
                .await;
        }
    }

    async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        block: &Block,
        location: BlockPos,
        old_state_id: BlockStateId,
        moved: bool,
    ) {
        if moved || Block::from_state_id(old_state_id).is_some_and(|old_block| old_block == *block)
        {
            return;
        }
        let state_id = world.get_block_state_id(&location).await;
        Self::update(world, &location, state_id).await;
    }
}

impl TripwireBlock {
    async fn update(world: &Arc<World>, pos: &BlockPos, state_id: BlockStateId) {
        for dir in [BlockDirection::South, BlockDirection::West] {
            for i in 1..42 {
                let current_pos = pos.offset_dir(dir.to_offset(), i);
                let (current_block, current_state) =
                    world.get_block_and_block_state(&current_pos).await;
                if current_block == Block::TRIPWIRE_HOOK {
                    let current_props = TripwireHookProperties::from_state_id(
                        current_state.id,
                        &Block::TRIPWIRE_HOOK,
                    );
                    if current_props.facing == dir.opposite().to_horizontal_facing().unwrap() {
                        TripwireHookBlock::update(
                            world,
                            current_pos,
                            current_state.id,
                            false,
                            true,
                            i,
                            Some(state_id),
                        )
                        .await;
                    }
                    break;
                }
                if current_block != Block::TRIPWIRE {
                    break;
                }
            }
        }
    }

    #[must_use]
    pub fn should_connect_to(state_id: BlockStateId, facing: BlockDirection) -> bool {
        Block::from_state_id(state_id).is_some_and(|block| {
            if block == Block::TRIPWIRE_HOOK {
                let props = TripwireHookProperties::from_state_id(state_id, &block);
                Some(props.facing) == facing.opposite().to_horizontal_facing()
            } else {
                block == Block::TRIPWIRE
            }
        })
    }
}
