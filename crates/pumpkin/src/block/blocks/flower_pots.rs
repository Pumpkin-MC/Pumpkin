use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, PathComputationType, RandomTickArgs, UseWithItemArgs};
use pumpkin_data::dimension::Dimension;
use pumpkin_data::flower_pot_transformations::get_potted_item;
use pumpkin_data::{Block, BlockId, BlockState};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block_from_tag("minecraft:flower_pots")]
pub struct FlowerPotBlock;

impl BlockBehaviour for FlowerPotBlock {
    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        {
            let item = args.item_stack.item;
            //Place the flower inside the pot
            let potted_block_id = get_potted_item(item.id);
            if args.block.eq(&Block::FLOWER_POT) {
                if potted_block_id != BlockId::AIR {
                    args.world.set_block_state(
                        args.position,
                        Block::from_id(potted_block_id).default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    );
                    args.player.increment_stat(
                        pumpkin_data::statistic::StatisticCategory::Custom,
                        pumpkin_data::statistic::CustomStatistic::PotFlower as i32,
                        1,
                    );
                }
                return BlockActionResult::Success;
            } else if potted_block_id != BlockId::AIR {
                //if the player have an item that can be potted in his hand, nothing happens
                return BlockActionResult::Consume;
            }

            //get the flower + empty the pot
            args.world.set_block_state(
                args.position,
                Block::FLOWER_POT.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
            BlockActionResult::Success
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if (args.world.dimension.eq(&Dimension::OVERWORLD)
            || args.world.dimension.eq(&Dimension::OVERWORLD_CAVES))
            && args.block.eq(&Block::POTTED_CLOSED_EYEBLOSSOM)
            && args.world.get_time_of_day() % 24000 > 14500
        {
            args.world.set_block_state(
                args.position,
                Block::POTTED_OPEN_EYEBLOSSOM.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
        }
        if args.block.eq(&Block::POTTED_OPEN_EYEBLOSSOM)
            && args.world.get_time_of_day() % 24000 <= 14500
        {
            args.world.set_block_state(
                args.position,
                Block::POTTED_CLOSED_EYEBLOSSOM.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
