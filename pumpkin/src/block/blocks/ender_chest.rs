use std::sync::Arc;

use crate::block::{
    BlockBehaviour, NormalUseArgs, OnPlaceArgs, OnSyncedBlockEventArgs, PlacedArgs,
    registry::BlockActionResult,
};
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, LadderLikeProperties};
use pumpkin_inventory::{
    generic_container_screen_handler::create_generic_9x3,
    player::player_inventory::PlayerInventory,
    screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerFactory},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use pumpkin_world::{
    BlockStateId, block::entities::ender_chest::EnderChestBlockEntity, inventory::Inventory,
};
use tokio::sync::Mutex;

struct EnderChestScreenFactory(Arc<dyn Inventory>);

#[async_trait]
impl ScreenHandlerFactory for EnderChestScreenFactory {
    async fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<Arc<Mutex<dyn ScreenHandler>>> {
        Some(Arc::new(Mutex::new(
            create_generic_9x3(sync_id, player_inventory, self.0.clone()).await,
        )))
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate("container.enderchest", &[])
    }
}

#[pumpkin_block("minecraft:ender_chest")]
pub struct EnderChestBlock;

#[async_trait]
impl BlockBehaviour for EnderChestBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = LadderLikeProperties::default(args.block);
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        props.to_state_id(args.block)
    }

    async fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        // On the server, we don't need to do more because the client is responsible for that.
        args.r#type == Self::LID_ANIMATION_EVENT_TYPE
    }

    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
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
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let block_entity = EnderChestBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(block_entity)).await;
    }
}

impl EnderChestBlock {
    pub const LID_ANIMATION_EVENT_TYPE: u8 = 1;
}
