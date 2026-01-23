use std::sync::Arc;

use crate::block::{BlockFuture, BlockMetadata, OnPlaceArgs, OnSyncedBlockEventArgs, PlacedArgs};
use crate::block::{
    registry::BlockActionResult,
    {BlockBehaviour, NormalUseArgs},
};

use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::shulker_box::ShulkerBoxBlockEntity;
use pumpkin_world::inventory::Inventory;
use tokio::sync::Mutex;

struct ShulkerBoxScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for ShulkerBoxScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler = create_generic_9x3(sync_id, player_inventory, self.0.clone()).await;
            let screen_handler_arc = Arc::new(Mutex::new(handler));

            Some(screen_handler_arc as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate("container.shulkerBox", &[])
    }
}

pub struct ShulkerBoxBlock;

impl BlockMetadata for ShulkerBoxBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:shulker_boxes").unwrap()
    }
}

type EndRodLikeProperties = pumpkin_data::block_properties::EndRodLikeProperties;

impl BlockBehaviour for ShulkerBoxBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = EndRodLikeProperties::default(args.block);
            props.facing = args.direction.to_facing().opposite();
            props.to_state_id(args.block)
        })
    }

    fn on_synced_block_event<'a>(
        &'a self,
        args: OnSyncedBlockEventArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            // On the server, we don't need the Animation steps for now, because the client is responsible for that.
            // TODO: Do not open the shulker box when it is currently closing
            args.r#type == Self::OPEN_ANIMATION_EVENT_TYPE
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let barrel_block_entity = ShulkerBoxBlockEntity::new(*args.position);
            args.world
                .add_block_entity(Arc::new(barrel_block_entity))
                .await;
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(inventory) = block_entity.get_inventory()
            {
                args.player
                    .open_handled_screen(&ShulkerBoxScreenFactory(inventory))
                    .await;
            }

            BlockActionResult::Success
        })
    }
}

impl ShulkerBoxBlock {
    pub const OPEN_ANIMATION_EVENT_TYPE: u8 = 1;
}
