use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockProperties, MangroveRootsLikeProperties},
    tag,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockFlags};
use rand::RngExt;

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
        OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
    },
    world::World,
};
pub struct CoralPlantBlock;
impl BlockMetadata for CoralPlantBlock {
    fn ids() -> Box<[u16]> {
        let alive_plants = tag::Block::MINECRAFT_CORAL_PLANTS.1;
        let mut plants = Vec::new();
        for alive_plant_id in alive_plants {
            let clone = *alive_plant_id;
            plants.push(clone);
            plants.push(get_dead_type(Block::from_id(clone)).id);
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
            if !scan_for_water(args.world, args.position).await && !is_dead(args.block) {
                try_schedule_die_tick(args.block, args.world, args.position).await;
            }
        })
    }
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !scan_for_water(args.world, args.position).await && !is_dead(args.block) {
                let current_state = args.world.get_block_state(args.position).await;
                let dead_block_state_id = {
                    let props =
                        CoralPlantLikeProperties::from_state_id(current_state.id, args.block);
                    props.to_state_id(&get_dead_type(args.block))
                };
                args.world
                    .set_block_state(args.position, dead_block_state_id, BlockFlags::empty())
                    .await;
            }
        })
    }
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let replacing_block = args.block_accessor.get_block(&args.position).await;
            if replacing_block != &Block::WATER && replacing_block != args.block {
                return false;
            }

            let support_block = args
                .block_accessor
                .get_block_state(&args.position.down())
                .await;
            if support_block.is_center_solid(BlockDirection::Up) {
                return true;
            }
            false
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.direction == BlockDirection::Down {
                let support_block = args.world.get_block_state(&args.position.down()).await;
                if !support_block.is_center_solid(BlockDirection::Up) {
                    return 0;
                }
            }
            args.state_id
        })
    }
}
async fn try_schedule_die_tick(block: &Block, world: &Arc<World>, pos: &BlockPos) {
    let tick_delay = 60 + rand::rng().random_range(0..40);
    world
        .schedule_block_tick(
            block,
            *pos,
            tick_delay as u8,
            pumpkin_world::tick::TickPriority::Normal,
        )
        .await;
}
fn is_dead(block: &Block) -> bool {
    block == &Block::DEAD_BRAIN_CORAL
        || block == &Block::DEAD_BUBBLE_CORAL
        || block == &Block::DEAD_FIRE_CORAL
        || block == &Block::DEAD_HORN_CORAL
        || block == &Block::DEAD_TUBE_CORAL
}
fn get_dead_type(block: &Block) -> Block {
    if block == &Block::BRAIN_CORAL {
        Block::DEAD_BRAIN_CORAL
    } else if block == &Block::BUBBLE_CORAL {
        Block::DEAD_BUBBLE_CORAL
    } else if block == &Block::FIRE_CORAL {
        Block::DEAD_FIRE_CORAL
    } else if block == &Block::HORN_CORAL {
        Block::DEAD_HORN_CORAL
    } else {
        Block::DEAD_TUBE_CORAL
    }
}
pub async fn scan_for_water(world: &Arc<World>, pos: &BlockPos) -> bool {
    for direction in BlockDirection::all() {
        let neighbor_pos = pos.offset(direction.to_offset());
        let block = world.get_block(&neighbor_pos).await;
        if block == &Block::WATER {
            return true;
        }
    }
    false
}
