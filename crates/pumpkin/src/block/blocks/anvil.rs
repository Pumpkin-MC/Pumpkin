use crate::block::blocks::falling::FallingBlock;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, GetStateForNeighborUpdateArgs, NormalUseArgs, OnPlaceArgs, OnScheduledTickArgs,
    PlacedArgs,
};

use pumpkin_data::block_properties::WallTorchLikeProperties;
use pumpkin_data::translation;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_inventory::anvil::AnvilScreenHandler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::SimpleInventory;
use std::sync::Arc;
use std::sync::Mutex;

#[pumpkin_block_from_tag("minecraft:anvil")]
pub struct AnvilBlock;

impl AnvilBlock {
    #[must_use]
    pub fn damage(state_id: BlockStateId) -> Option<BlockStateId> {
        let block = Block::from_state_id(state_id);
        let next_block = if block == &Block::ANVIL {
            &Block::CHIPPED_ANVIL
        } else if block == &Block::CHIPPED_ANVIL {
            &Block::DAMAGED_ANVIL
        } else {
            return None;
        };
        let props = WallTorchLikeProperties::from_state_id(state_id);
        Some(props.to_state_id(next_block))
    }
}

impl BlockBehaviour for AnvilBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        args.player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            pumpkin_data::statistic::CustomStatistic::InteractWithAnvil as i32,
            1,
        );
        args.player
            .open_handled_screen(&AnvilScreenFactory, Some(*args.position));

        BlockActionResult::Success
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        FallingBlock::placed(&FallingBlock, args);
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let dir = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .rotate_clockwise();

        let mut props = WallTorchLikeProperties::default(args.block);

        props.facing = dir;
        props.to_state_id(args.block)
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        FallingBlock::on_scheduled_tick(&FallingBlock, args);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        FallingBlock::get_state_for_neighbor_update(&FallingBlock, args)
    }
}

struct AnvilScreenFactory;

impl ScreenHandlerFactory for AnvilScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let inventory = Arc::new(SimpleInventory::new(3));
        let handler = AnvilScreenHandler::new(sync_id, player_inventory, inventory);
        let concrete_arc = Arc::new(Mutex::new(handler));

        Some(concrete_arc as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_REPAIR,
            translation::bedrock::CONTAINER_REPAIR
        )
    }
}
