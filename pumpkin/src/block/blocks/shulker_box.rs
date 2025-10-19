use std::sync::Arc;

use crate::block::{BlockMetadata, OnPlaceArgs, OnSyncedBlockEventArgs, PlacedArgs};
use crate::block::{
    registry::BlockActionResult,
    {BlockBehaviour, NormalUseArgs},
};
use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerFactory};
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::shulker_box::ShulkerBoxBlockEntity;
use pumpkin_world::inventory::Inventory;
use tokio::sync::Mutex;

struct ShulkerBoxScreenFactory(Arc<dyn Inventory>);

#[async_trait]
impl ScreenHandlerFactory for ShulkerBoxScreenFactory {
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

#[async_trait]
impl BlockBehaviour for ShulkerBoxBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = EndRodLikeProperties::default(args.block);
        props.facing = args.direction.to_facing().opposite();
        props.to_state_id(args.block)
    }

    async fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        // On the server, we don't need the Animation steps for now, because the client is responsible for that.
        // TODO: Do not open the shulker box when it is currently closing
        args.r#type == Self::OPEN_ANIMATION_EVENT_TYPE
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let barrel_block_entity = ShulkerBoxBlockEntity::new(*args.position);
        args.world
            .add_block_entity(Arc::new(barrel_block_entity))
            .await;
    }

    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if let Some(block_entity) = args.world.get_block_entity(args.position).await
            && let Some(inventory) = block_entity.get_inventory()
        {
            args.player
                .open_handled_screen(&ShulkerBoxScreenFactory(inventory))
                .await;
        }

        BlockActionResult::Success
    }
}

impl ShulkerBoxBlock {
    pub const OPEN_ANIMATION_EVENT_TYPE: u8 = 1;
}
