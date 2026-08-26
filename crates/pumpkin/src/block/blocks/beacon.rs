use std::sync::Arc;
use std::sync::Mutex;

use crate::block::entities::BlockEntity;
use pumpkin_data::translation;
use pumpkin_inventory::beacon_screen_handler::create_beacon_handler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::Inventory;

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, NormalUseArgs};

// Create the factory just like ChestScreenFactory
struct BeaconScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for BeaconScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let concrete_handler = create_beacon_handler(sync_id, player_inventory, self.0.clone());
        let concrete_arc = Arc::new(Mutex::new(concrete_handler));

        Some(concrete_arc as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_BEACON,
            translation::bedrock::CONTAINER_BEACON
        )
    }
}

#[pumpkin_block("minecraft:beacon")]
pub struct BeaconBlock;

impl BlockBehaviour for BeaconBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let block_entity = args.world.get_block_entity(args.position);

        // Extract the inventory from the entity
        let Some(inventory) = block_entity.and_then(BlockEntity::get_inventory) else {
            return BlockActionResult::Fail;
        };

        args.player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            pumpkin_data::statistic::CustomStatistic::InteractWithBeacon as i32,
            1,
        );

        // Open the screen using the factory
        args.player
            .open_handled_screen(&BeaconScreenFactory(inventory), Some(*args.position));

        BlockActionResult::Success
    }
}
