use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId,
    block_properties::CandleCakeProperties,
    game_event::GameEvent,
    item::Item,
    item_stack::ItemStack,
    sound::{Sound, SoundCategory},
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::{
    block::{
        BlockBehaviour, CanPlaceAtArgs, GetComparatorOutputArgs, GetStateForNeighborUpdateArgs,
        NormalUseArgs, PathComputationType, UseWithItemArgs,
        blocks::cake::{CakeBlock, FULL_CAKE_SIGNAL},
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
        .map_or(&Block::CAKE, |index| CANDLE_MAP[index].1)
}

#[must_use]
pub fn candle_from_cake(block: &Block) -> &'static Item {
    CANDLE_MAP
        .binary_search_by_key(&block.id, |(_, value)| value.id)
        .map_or(&Item::CANDLE, |index| CANDLE_MAP[index].0)
}

#[pumpkin_block_from_tag("minecraft:candle_cakes")]
pub struct CandleCakeBlock;

impl CandleCakeBlock {
    fn consume_and_drop_candle(
        block: &Block,
        player: &Player,
        location: &BlockPos,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let result = CakeBlock::consume_if_hungry(
            world,
            player,
            &Block::CAKE,
            location,
            Block::CAKE.default_state.id,
        );
        if result.consumes_action() {
            let candle_item = candle_from_cake(block);
            world.drop_stack(location, ItemStack::new(1, candle_item));
        }
        result
    }
}

impl BlockBehaviour for CandleCakeBlock {
    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let item_id = args.item_stack.item.id;
        if item_id == Item::FIRE_CHARGE.id || item_id == Item::FLINT_AND_STEEL.id {
            return BlockActionResult::Pass;
        }
        let state_id = args.world.get_block_state_id(args.position);
        let mut properties = CandleCakeProperties::from_state_id(state_id);
        if args.hit.cursor_pos.y > 0.5 && args.item_stack.is_empty() && properties.lit {
            properties.lit = false;
            args.world.set_block_state(
                args.position,
                properties.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
            args.world.play_sound(
                Sound::BlockCandleExtinguish,
                SoundCategory::Blocks,
                &args.position.to_centered_f64(),
            );
            args.world.emit_game_event(
                GameEvent::BlockChange.name(),
                args.position.to_centered_f64(),
            );
            return BlockActionResult::Success;
        }
        BlockActionResult::PassToDefaultBlockAction
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        Self::consume_and_drop_candle(args.block, args.player, args.position, args.world)
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Down && !can_place_at(args.world, args.position) {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }

    /// A candle cake is always uneaten, so it reads a full cake.
    fn get_comparator_output(&self, _args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        Some(FULL_CAKE_SIGNAL)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

fn can_place_at(world: &dyn BlockAccessor, position: &BlockPos) -> bool {
    let state = world.get_block_state(&position.down());
    state.is_solid()
}
