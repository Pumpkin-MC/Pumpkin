use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, OnScheduledTickArgs, PlacedArgs,
    blocks::coral::{is_dead_coral, scan_for_water, try_schedule_die_tick},
};
use pumpkin_data::{Block, tag};
use pumpkin_world::world::BlockFlags;
pub struct CoralBlock;
impl BlockMetadata for CoralBlock {
    fn ids() -> Box<[u16]> {
        let alive_plants = tag::Block::MINECRAFT_CORAL_BLOCKS.1;
        let mut plants = Vec::new();
        for alive_plant_id in alive_plants {
            let clone = *alive_plant_id;
            plants.push(clone);
            plants.push(get_dead_coral_block_type(Block::from_id(clone)).id);
        }
        plants.into()
    }
}
impl BlockBehaviour for CoralBlock {
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
                let dead_block_state_id = get_dead_coral_block_type(args.block).default_state.id;
                args.world
                    .set_block_state(args.position, dead_block_state_id, BlockFlags::empty())
                    .await;
            }
        })
    }
}
fn get_dead_coral_block_type(block: &Block) -> Block {
    if block == &Block::BRAIN_CORAL_BLOCK {
        Block::DEAD_BRAIN_CORAL_BLOCK
    } else if block == &Block::BUBBLE_CORAL_BLOCK {
        Block::DEAD_BUBBLE_CORAL_BLOCK
    } else if block == &Block::FIRE_CORAL_BLOCK {
        Block::DEAD_FIRE_CORAL_BLOCK
    } else if block == &Block::HORN_CORAL_BLOCK {
        Block::DEAD_HORN_CORAL_BLOCK
    } else {
        Block::DEAD_TUBE_CORAL_BLOCK
    }
}
