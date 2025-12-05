use std::sync::Arc;

use pumpkin_data::{
    Block,
    item::Item,
    tag::{RegistryKey, get_tag_values},
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::{GameMode, math::position::BlockPos};
use pumpkin_world::{item::ItemStack, world::BlockFlags};

use crate::{
    block::{
        BlockBehaviour, BlockFuture, NormalUseArgs, UseWithItemArgs, blocks::cake::CakeBlock,
        registry::BlockActionResult,
    },
    entity::player::Player,
    world::World,
};

const CANDLE_MAP: [(&Item, &Block); 17] = [
    (&Item::CANDLE, &Block::CANDLE_CAKE),
    (&Item::WHITE_CANDLE, &Block::WHITE_CANDLE_CAKE),
    (&Item::ORANGE_CANDLE, &Block::ORANGE_CANDLE_CAKE),
    (&Item::MAGENTA_CANDLE, &Block::MAGENTA_CANDLE_CAKE),
    (&Item::LIGHT_BLUE_CANDLE, &Block::LIGHT_BLUE_CANDLE_CAKE),
    (&Item::YELLOW_CANDLE, &Block::YELLOW_CANDLE_CAKE),
    (&Item::LIME_CANDLE, &Block::LIME_CANDLE_CAKE),
    (&Item::PINK_CANDLE, &Block::PINK_CANDLE_CAKE),
    (&Item::GRAY_CANDLE, &Block::GRAY_CANDLE_CAKE),
    (&Item::LIGHT_GRAY_CANDLE, &Block::LIGHT_GRAY_CANDLE_CAKE),
    (&Item::CYAN_CANDLE, &Block::CYAN_CANDLE_CAKE),
    (&Item::PURPLE_CANDLE, &Block::PURPLE_CANDLE_CAKE),
    (&Item::BLUE_CANDLE, &Block::BLUE_CANDLE_CAKE),
    (&Item::BROWN_CANDLE, &Block::BROWN_CANDLE_CAKE),
    (&Item::GREEN_CANDLE, &Block::GREEN_CANDLE_CAKE),
    (&Item::RED_CANDLE, &Block::RED_CANDLE_CAKE),
    (&Item::BLACK_CANDLE, &Block::BLACK_CANDLE_CAKE),
];

#[must_use]
pub fn cake_from_candle(item: &Item) -> &'static Block {
    CANDLE_MAP
        .binary_search_by_key(&item.id, |(key, _)| key.id)
        .map_or_else(
            |_| panic!("Expected a candle item, got {}", item.id),
            |index| CANDLE_MAP[index].1,
        )
}

#[must_use]
pub fn candle_from_cake(block: &Block) -> &'static Item {
    CANDLE_MAP
        .binary_search_by_key(&block.id, |(_, value)| value.id)
        .map_or_else(
            |_| panic!("Expected a candle cake block, got {}", block.id),
            |index| CANDLE_MAP[index].0,
        )
}

#[pumpkin_block_from_tag("minecraft:candle_cakes")]
pub struct CandleCakeBlock;

impl CandleCakeBlock {
    async fn consume_and_drop_candle(
        block: &Block,
        player: &Player,
        location: &BlockPos,
        world: &Arc<World>,
    ) -> BlockActionResult {
        match player.gamemode.load() {
            GameMode::Survival | GameMode::Adventure => {
                if player.hunger_manager.level.load() >= 20 {
                    return BlockActionResult::Pass;
                }
            }
            GameMode::Creative => {}
            GameMode::Spectator => return BlockActionResult::Pass,
        }

        let candle_item = candle_from_cake(block);

        let item_stack = ItemStack::new(1, candle_item);

        world.drop_stack(location, item_stack).await;

        world
            .set_block_state(
                location,
                Block::CAKE.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;

        let (block, state) = world.get_block_and_state_id(location).await;

        CakeBlock::consume_if_hungry(world, player, block, location, state).await
    }
}

impl BlockBehaviour for CandleCakeBlock {
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let item_id = args.item_stack.lock().await.item.id;
            match item_id {
                id if id == Item::FIRE_CHARGE.id || id == Item::FLINT_AND_STEEL.id => {
                    BlockActionResult::Pass
                } // Item::FIRE_CHARGE | Item::FLINT_AND_STEEL
                _ => BlockActionResult::PassToDefaultBlockAction,
            }
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            Self::consume_and_drop_candle(args.block, args.player, args.position, args.world).await
        })
    }
}
