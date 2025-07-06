use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockProperties, HorizontalFacing},
    item::Item,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::{BlockStateId, chunk::TickPriority, world::BlockFlags};

use crate::{
    block::pumpkin_block::{
        BrokenArgs, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs, OnPlaceArgs,
        OnScheduledTickArgs, OnStateReplacedArgs, PlacedArgs, PumpkinBlock,
    },
    world::World,
};

use super::tripwire_hook::TripwireHookBlock;

type TripwireProperties = pumpkin_data::block_properties::TripwireLikeProperties;
type TripwireHookProperties = pumpkin_data::block_properties::TripwireHookLikeProperties;

#[pumpkin_block("minecraft:tripwire")]
pub struct TripwireBlock;

#[async_trait]
impl PumpkinBlock for TripwireBlock {
    async fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let mut props = TripwireProperties::from_state_id(args.state.id, args.block);
        if props.powered {
            return;
        }
        props.powered = true;

        let state_id = props.to_state_id(args.block);
        args.world
            .set_block_state(args.location, state_id, BlockFlags::NOTIFY_ALL)
            .await;

        Self::update(args.world, args.location, state_id).await;

        args.world
            .schedule_block_tick(args.block, *args.location, 10, TickPriority::Normal)
            .await;
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let [connect_north, connect_east, connect_south, connect_west] = [
            BlockDirection::North,
            BlockDirection::East,
            BlockDirection::South,
            BlockDirection::West,
        ]
        .map(async |dir| {
            let current_pos = args.location.offset(dir.to_offset());
            let state_id = args.world.get_block_state_id(&current_pos).await;
            Self::should_connect_to(state_id, dir)
        });

        let mut props = TripwireProperties::from_state_id(args.block.default_state.id, args.block);

        props.north = connect_north.await;
        props.south = connect_south.await;
        props.west = connect_west.await;
        props.east = connect_east.await;

        props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        if let (Some(old_block), Some(new_block)) = (
            Block::from_state_id(args.old_state_id),
            Block::from_state_id(args.state_id),
        ) {
            if old_block == new_block {
                return;
            }
        }
        Self::update(args.world, args.location, args.state_id).await;
    }

    async fn broken(&self, args: BrokenArgs<'_>) {
        let has_shears = {
            let main_hand_item_stack = args.player.inventory().held_item();
            main_hand_item_stack
                .lock()
                .await
                .get_item()
                .eq(&Item::SHEARS)
        };
        if has_shears {
            let mut props = TripwireProperties::from_state_id(args.state.id, args.block);
            props.disarmed = true;
            args.world
                .set_block_state(
                    args.location,
                    props.to_state_id(args.block),
                    BlockFlags::empty(),
                )
                .await;
            // TODO world.emitGameEvent(player, GameEvent.SHEAR, pos);
        }
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        args.direction
            .to_horizontal_facing()
            .map_or(args.state_id, |facing| {
                let mut props = TripwireProperties::from_state_id(args.state_id, args.block);
                *match facing {
                    HorizontalFacing::North => &mut props.north,
                    HorizontalFacing::South => &mut props.south,
                    HorizontalFacing::West => &mut props.west,
                    HorizontalFacing::East => &mut props.east,
                } = Self::should_connect_to(args.neighbor_state_id, args.direction);
                props.to_state_id(args.block)
            })
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.location).await;

        let mut props = TripwireProperties::from_state_id(state_id, args.block);
        if !props.powered {
            return;
        }

        let aabb = BoundingBox::from_block(args.location);
        // TODO entity.canAvoidTraps()
        if args.world.get_entities_at_box(&aabb).await.is_empty()
            && args.world.get_players_at_box(&aabb).await.is_empty()
        {
            props.powered = false;
            let state_id = props.to_state_id(args.block);
            args.world
                .set_block_state(args.location, state_id, BlockFlags::NOTIFY_ALL)
                .await;
            Self::update(args.world, args.location, state_id).await;
        } else {
            args.world
                .schedule_block_tick(args.block, *args.location, 10, TickPriority::Normal)
                .await;
        }
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if args.moved
            || Block::from_state_id(args.old_state_id)
                .is_some_and(|old_block| old_block == args.block)
        {
            return;
        }
        let state_id = args.world.get_block_state_id(args.location).await;
        Self::update(args.world, args.location, state_id).await;
    }
}

impl TripwireBlock {
    async fn update(world: &Arc<World>, pos: &BlockPos, state_id: BlockStateId) {
        for dir in [BlockDirection::South, BlockDirection::West] {
            for i in 1..42 {
                let current_pos = pos.offset_dir(dir.to_offset(), i);
                let (current_block, current_state) =
                    world.get_block_and_block_state(&current_pos).await;
                if current_block == &Block::TRIPWIRE_HOOK {
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
                if current_block != &Block::TRIPWIRE {
                    break;
                }
            }
        }
    }

    #[must_use]
    pub fn should_connect_to(state_id: BlockStateId, facing: BlockDirection) -> bool {
        Block::from_state_id(state_id).is_some_and(|block| {
            if block == &Block::TRIPWIRE_HOOK {
                let props = TripwireHookProperties::from_state_id(state_id, block);
                Some(props.facing) == facing.opposite().to_horizontal_facing()
            } else {
                block == &Block::TRIPWIRE
            }
        })
    }
}
