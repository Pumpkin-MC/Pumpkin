use std::sync::Arc;

use crate::block::{
    BlockBehaviour, BlockFuture, NormalUseArgs, OnPlaceArgs, OnSyncedBlockEventArgs, PlacedArgs,
    registry::BlockActionResult,
};
use pumpkin_data::block_properties::{BlockProperties, LadderLikeProperties};
use pumpkin_inventory::{
    generic_container_screen_handler::create_generic_9x3,
    player::player_inventory::PlayerInventory,
    screen_handler::{BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use pumpkin_world::{
    BlockStateId, block::entities::ender_chest::EnderChestBlockEntity, inventory::Inventory,
};
use tokio::sync::Mutex;

struct EnderChestScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for EnderChestScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler = create_generic_9x3(sync_id, player_inventory, self.0.clone()).await;
            let concrete_arc = Arc::new(Mutex::new(handler));

            Some(concrete_arc as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate("container.enderchest", &[])
    }
}

#[pumpkin_block("minecraft:ender_chest")]
pub struct EnderChestBlock;

impl BlockBehaviour for EnderChestBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = LadderLikeProperties::default(args.block);
            props.facing = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();
            props.to_state_id(args.block)
        })
    }

    fn on_synced_block_event<'a>(
        &'a self,
        args: OnSyncedBlockEventArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            // On the server, we don't need to do more because the client is responsible for that.
            args.r#type == Self::LID_ANIMATION_EVENT_TYPE
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(block_entity) = block_entity
                    .as_any()
                    .downcast_ref::<EnderChestBlockEntity>()
            {
                let inventory = args.player.ender_chest_inventory();
                inventory.set_tracker(block_entity.get_tracker()).await;
                args.player
                    .open_handled_screen(&EnderChestScreenFactory(inventory.clone()))
                    .await;

                // TODO: player.incrementStat(Stats.OPEN_ENDERCHEST);
                // TODO: PiglinBrain.onGuardedBlockInteracted(serverWorld, player, true);
            }

            BlockActionResult::Success
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let block_entity = EnderChestBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(block_entity)).await;
        })
    }
}

impl EnderChestBlock {
    pub const LID_ANIMATION_EVENT_TYPE: u8 = 1;
}
