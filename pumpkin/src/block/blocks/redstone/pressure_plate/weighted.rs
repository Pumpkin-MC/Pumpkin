use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{BlockProperties, EnumVariants, Integer0To15},
};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::{BlockStateId, world::BlockFlags};

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, EmitsRedstonePowerArgs,
        GetRedstonePowerArgs, OnEntityCollisionArgs, OnNeighborUpdateArgs, OnScheduledTickArgs,
        OnStateReplacedArgs,
    },
    world::World,
};

use super::PressurePlate;

/// This is for Gold and Iron Pressure Plate
pub struct WeightedPressurePlateBlock;

type PressurePlateProps = pumpkin_data::block_properties::LightWeightedPressurePlateLikeProperties;

impl BlockMetadata for WeightedPressurePlateBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        // light = Gold
        // heavy = Iron
        &[
            "light_weighted_pressure_plate",
            "heavy_weighted_pressure_plate",
        ]
    }
}

impl BlockBehaviour for WeightedPressurePlateBlock {
    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            self.on_entity_collision_pp(args).await;
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            self.on_scheduled_tick_pp(args).await;
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            self.on_state_replaced_pp(args).await;
        })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move { self.get_redstone_output(args.block, args.state.id) })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            if args.direction == BlockDirection::Up {
                return self.get_redstone_output(args.block, args.state.id);
            }
            0
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !Self::can_pressure_plate_place_at(args.world, args.position).await {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            Self::can_pressure_plate_place_at(args.world.unwrap(), args.position).await
        })
    }
}

impl PressurePlate for WeightedPressurePlateBlock {
    fn get_redstone_output(&self, block: &Block, state: BlockStateId) -> u8 {
        let props = PressurePlateProps::from_state_id(state, block);
        props.power.to_index() as u8
    }

    async fn calculate_redstone_output(&self, world: &World, block: &Block, pos: &BlockPos) -> u8 {
        // light = Gold
        // heavy = Iron
        let weight = if block == &Block::LIGHT_WEIGHTED_PRESSURE_PLATE {
            // Gold
            15
        } else {
            // Iron
            150
        };
        // TODO: this is bad use real box
        let aabb = BoundingBox::from_block(pos);
        let len = world.get_entities_at_box(&aabb).await.len()
            + world.get_players_at_box(&aabb).await.len();
        let len = len.min(weight);
        if len > 0 {
            let f = (weight.min(len) / weight) as f32;
            return (f * 15.0).ceil() as u8;
        }
        0
    }

    fn set_redstone_output(
        &self,
        block: &Block,
        state: &BlockState,
        output: u8,
    ) -> pumpkin_world::BlockStateId {
        let mut props = PressurePlateProps::from_state_id(state.id, block);
        props.power = Integer0To15::from_index(u16::from(output));
        props.to_state_id(block)
    }

    fn tick_rate(&self) -> u8 {
        10
    }
}
