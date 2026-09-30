use crate::block::{
    BlockBehaviour, BlockMetadata, OnScheduledTickArgs, PlacedArgs,
    blocks::coral::{is_dead_coral, scan_for_water, try_schedule_die_tick},
};
use pumpkin_data::{Block, BlockId, tag};
use pumpkin_world::world::BlockFlags;
pub struct CoralBlock;
impl BlockMetadata for CoralBlock {
    fn ids() -> Box<[BlockId]> {
        let alive_plants = tag::Block::MINECRAFT_CORAL_BLOCKS.1;
        let mut plants = Vec::new();
        for alive_plant_id in alive_plants {
            let block_id = BlockId::new_or_air(*alive_plant_id);
            plants.push(block_id);
            plants.push(get_dead_coral_block_type(block_id).map_or(BlockId::AIR, |b| b.id));
        }
        plants.into()
    }
}
impl BlockBehaviour for CoralBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        {
            if !scan_for_water(args.world, args.position) && !is_dead_coral(args.block) {
                try_schedule_die_tick(args.block, args.world, args.position);
            }
        }
    }
    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !scan_for_water(args.world, args.position) && !is_dead_coral(args.block) {
            let Some(dead_block) = get_dead_coral_block_type(args.block.id) else {
                return;
            };
            let dead_block_state_id = dead_block.default_state.id;
            args.world
                .set_block_state(args.position, dead_block_state_id, BlockFlags::NOTIFY_ALL);
        }
    }
}
const fn get_dead_coral_block_type(id: BlockId) -> Option<&'static Block> {
    match id {
        BlockId::BRAIN_CORAL_BLOCK => Some(&Block::DEAD_BRAIN_CORAL_BLOCK),
        BlockId::BUBBLE_CORAL_BLOCK => Some(&Block::DEAD_BUBBLE_CORAL_BLOCK),
        BlockId::FIRE_CORAL_BLOCK => Some(&Block::DEAD_FIRE_CORAL_BLOCK),
        BlockId::HORN_CORAL_BLOCK => Some(&Block::DEAD_HORN_CORAL_BLOCK),
        BlockId::TUBE_CORAL_BLOCK => Some(&Block::DEAD_TUBE_CORAL_BLOCK),
        _ => None,
    }
}
