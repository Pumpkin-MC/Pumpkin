use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::BlockProperties,
    tag::{RegistryKey, get_tag_values},
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

/// This is for Normal Pressure plates, so not Gold or Iron
pub struct PressurePlateBlock;

type PressurePlateProps = pumpkin_data::block_properties::StonePressurePlateLikeProperties;

impl BlockMetadata for PressurePlateBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        let mut combined = Vec::new();
        combined.extend_from_slice(
            get_tag_values(RegistryKey::Block, "minecraft:wooden_pressure_plates").unwrap(),
        );
        combined.extend_from_slice(
            get_tag_values(RegistryKey::Block, "minecraft:stone_pressure_plates").unwrap(),
        );
        combined.leak()
    }
}

impl BlockBehaviour for PressurePlateBlock {
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

impl PressurePlate for PressurePlateBlock {
    fn get_redstone_output(&self, block: &Block, state: BlockStateId) -> u8 {
        let props = PressurePlateProps::from_state_id(state, block);
        if props.powered { 15 } else { 0 }
    }

    async fn calculate_redstone_output(&self, world: &World, _block: &Block, pos: &BlockPos) -> u8 {
        // TODO: this is bad use real box
        let aabb = BoundingBox::from_block(pos);
        if !world.get_entities_at_box(&aabb).await.is_empty()
            || !world.get_players_at_box(&aabb).await.is_empty()
        {
            return 15;
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
        props.powered = output > 0;
        props.to_state_id(block)
    }
}
