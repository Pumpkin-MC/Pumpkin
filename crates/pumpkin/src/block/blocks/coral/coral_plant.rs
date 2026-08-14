use pumpkin_data::{
    BlockDirection, BlockId, BlockStateId,
    block_properties::{BlockProperties, MangroveRootsLikeProperties},
    tag,
};
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
    blocks::coral::{is_dead_coral, scan_for_water, try_schedule_die_tick},
};
pub struct CoralPlantBlock;
impl BlockMetadata for CoralPlantBlock {
    fn ids() -> Box<[BlockId]> {
        let alive_plants = tag::Block::MINECRAFT_CORAL_PLANTS.1;
        let mut plants = Vec::new();
        for alive_plant_id in alive_plants {
            let block_id = BlockId::new_or_air(*alive_plant_id);
            plants.push(block_id);
            plants.push(get_dead_type(block_id).unwrap_or_default());
        }
        plants.into()
    }
}
pub type CoralPlantLikeProperties = MangroveRootsLikeProperties;

impl BlockBehaviour for CoralPlantBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = CoralPlantLikeProperties::default(args.block);
            props.waterlogged = args.replacing.water_source();
            props.to_state_id(args.block)
        })
    }
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !scan_for_water(args.world, args.position).await && !is_dead_coral(args.block) {
                try_schedule_die_tick(args.block, args.world, args.position).await;
            }
        })
    }
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !scan_for_water(args.world, args.position).await && !is_dead_coral(args.block) {
                let current_state = args.world.get_block_state(args.position);
                let dead_block_state_id = {
                    let props =
                        CoralPlantLikeProperties::from_state_id(current_state.id, args.block);
                    props.to_state_id(get_dead_type(args.block.id).unwrap_or_default().to_block())
                };
                args.world
                    .set_block_state(args.position, dead_block_state_id, BlockFlags::empty())
                    .await;
            }
        })
    }
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> bool {
        let support_block = args.block_accessor.get_block_state(&args.position.down());
        if support_block.is_center_solid(BlockDirection::Up) {
            return true;
        }
        false
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.direction == BlockDirection::Down {
                let support_block = args.world.get_block_state(&args.position.down());
                if !support_block.is_center_solid(BlockDirection::Up) {
                    return BlockStateId::AIR;
                }
            }
            args.state_id
        })
    }
}
const fn get_dead_type(id: BlockId) -> Option<BlockId> {
    match id {
        BlockId::BRAIN_CORAL => Some(BlockId::DEAD_BRAIN_CORAL),
        BlockId::BUBBLE_CORAL => Some(BlockId::DEAD_BUBBLE_CORAL),
        BlockId::FIRE_CORAL => Some(BlockId::DEAD_FIRE_CORAL),
        BlockId::HORN_CORAL => Some(BlockId::DEAD_HORN_CORAL),
        BlockId::TUBE_CORAL => Some(BlockId::DEAD_TUBE_CORAL),
        _ => None,
    }
}
