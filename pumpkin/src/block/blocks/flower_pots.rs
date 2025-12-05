use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, RandomTickArgs, UseWithItemArgs};
use pumpkin_data::Block;
use pumpkin_data::flower_pot_transformations::get_potted_item;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_registry::VanillaDimensionType;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block_from_tag("minecraft:flower_pots")]
pub struct FlowerPotBlock;

impl BlockBehaviour for FlowerPotBlock {
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let item = args.item_stack.lock().await.item;
            //Place the flower inside the pot
            let potted_block_id = get_potted_item(item.id);
            if args.block.eq(&Block::FLOWER_POT) {
                if potted_block_id != 0 {
                    args.world
                        .set_block_state(
                            args.position,
                            Block::from_id(potted_block_id).default_state.id,
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                }
                return BlockActionResult::Success;
            } else if potted_block_id != 0 {
                //if the player have an item that can be potted in his hand, nothing happens
                return BlockActionResult::Consume;
            }

            //get the flower + empty the pot
            args.world
                .set_block_state(
                    args.position,
                    Block::FLOWER_POT.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            BlockActionResult::Success
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if (args
                .world
                .dimension_type
                .eq(&VanillaDimensionType::Overworld)
                || args
                    .world
                    .dimension_type
                    .eq(&VanillaDimensionType::OverworldCaves))
                && args.block.eq(&Block::POTTED_CLOSED_EYEBLOSSOM)
                && args.world.level_time.lock().await.time_of_day % 24000 > 14500
            {
                args.world
                    .set_block_state(
                        args.position,
                        Block::POTTED_OPEN_EYEBLOSSOM.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
            if args.block.eq(&Block::POTTED_OPEN_EYEBLOSSOM)
                && args.world.level_time.lock().await.time_of_day % 24000 <= 14500
            {
                args.world
                    .set_block_state(
                        args.position,
                        Block::POTTED_CLOSED_EYEBLOSSOM.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }
}
