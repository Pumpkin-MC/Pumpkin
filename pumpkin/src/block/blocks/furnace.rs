use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, FurnaceLikeProperties};
use pumpkin_inventory::{
    furnace::furnace_screen_handler::FurnaceScreenHandler, screen_handler::ScreenHandlerFactory,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use pumpkin_world::{
    block::entities::{BlockEntity, furnace::FurnaceBlockEntity},
    inventory::Inventory,
};
use tokio::sync::Mutex;

use crate::block::pumpkin_block::PumpkinBlock;

struct FurnaceScreenFactory {
    inventory: Arc<dyn Inventory>,
    furnace_block_entity: Arc<dyn BlockEntity>,
}

impl FurnaceScreenFactory {
    fn new(inventory: Arc<dyn Inventory>, furnace_block_entity: Arc<dyn BlockEntity>) -> Self {
        Self {
            inventory,
            furnace_block_entity,
        }
    }
}

#[async_trait]
impl ScreenHandlerFactory for FurnaceScreenFactory {
    async fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<pumpkin_inventory::player::player_inventory::PlayerInventory>,
        _player: &dyn pumpkin_inventory::screen_handler::InventoryPlayer,
    ) -> Option<Arc<Mutex<dyn pumpkin_inventory::screen_handler::ScreenHandler>>> {
        Some(Arc::new(Mutex::new(FurnaceScreenHandler::new(
            sync_id,
            player_inventory,
            self.inventory.clone(),
            self.furnace_block_entity.clone(),
        ))))
    }

    fn get_display_name(&self) -> pumpkin_util::text::TextComponent {
        TextComponent::translate("container.furnace", &[])
    }
}

#[pumpkin_block("minecraft:furnace")]
pub struct FurnaceBlock;

#[async_trait]
impl PumpkinBlock for FurnaceBlock {
    async fn normal_use(
        &self,
        args: crate::block::pumpkin_block::NormalUseArgs<'_>,
    ) -> crate::block::registry::BlockActionResult {
        if let Some(block_entity) = args.world.get_block_entity(args.position).await {
            if let Some(inventory) = block_entity.clone().get_inventory() {
                let furnace_screen_factory = FurnaceScreenFactory::new(inventory, block_entity);
                args.player
                    .open_handled_screen(&furnace_screen_factory)
                    .await;
            }
        }
        crate::block::registry::BlockActionResult::Consume
    }

    //Same to normal_use
    async fn use_with_item(
        &self,
        args: crate::block::pumpkin_block::UseWithItemArgs<'_>,
    ) -> crate::block::registry::BlockActionResult {
        if let Some(block_entity) = args.world.get_block_entity(args.position).await {
            if let Some(inventory) = block_entity.clone().get_inventory() {
                let furnace_screen_factory = FurnaceScreenFactory::new(inventory, block_entity);
                args.player
                    .open_handled_screen(&furnace_screen_factory)
                    .await;
            }
        }
        crate::block::registry::BlockActionResult::Consume
    }

    async fn on_entity_collision(
        &self,
        _args: crate::block::pumpkin_block::OnEntityCollisionArgs<'_>,
    ) {
    }

    fn should_drop_items_on_explosion(&self) -> bool {
        true
    }

    async fn explode(&self, _args: crate::block::pumpkin_block::ExplodeArgs<'_>) {}

    async fn on_synced_block_event(
        &self,
        _args: crate::block::pumpkin_block::OnSyncedBlockEventArgs<'_>,
    ) -> bool {
        false
    }

    async fn on_place(
        &self,
        args: crate::block::pumpkin_block::OnPlaceArgs<'_>,
    ) -> pumpkin_world::BlockStateId {
        let mut props = FurnaceLikeProperties::default(args.block);
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();

        props.to_state_id(args.block)
    }

    async fn random_tick(&self, _args: crate::block::pumpkin_block::RandomTickArgs<'_>) {}

    async fn can_place_at(&self, _args: crate::block::pumpkin_block::CanPlaceAtArgs<'_>) -> bool {
        true
    }

    async fn can_update_at(&self, _args: crate::block::pumpkin_block::CanUpdateAtArgs<'_>) -> bool {
        false
    }

    async fn placed(&self, args: crate::block::pumpkin_block::PlacedArgs<'_>) {
        let furnace_block_entity = FurnaceBlockEntity::new(*args.position);
        args.world
            .add_block_entity(Arc::new(furnace_block_entity))
            .await;
    }

    async fn player_placed(&self, _args: crate::block::pumpkin_block::PlayerPlacedArgs<'_>) {}

    async fn broken(&self, args: crate::block::pumpkin_block::BrokenArgs<'_>) {
        args.world.remove_block_entity(args.position).await;
    }

    async fn on_neighbor_update(
        &self,
        _args: crate::block::pumpkin_block::OnNeighborUpdateArgs<'_>,
    ) {
    }

    async fn prepare(&self, _args: crate::block::pumpkin_block::PrepareArgs<'_>) {}

    async fn get_state_for_neighbor_update(
        &self,
        args: crate::block::pumpkin_block::GetStateForNeighborUpdateArgs<'_>,
    ) -> pumpkin_world::BlockStateId {
        args.state_id
    }

    async fn on_scheduled_tick(&self, _args: crate::block::pumpkin_block::OnScheduledTickArgs<'_>) {
    }

    async fn on_state_replaced(&self, _args: crate::block::pumpkin_block::OnStateReplacedArgs<'_>) {
    }

    async fn emits_redstone_power(
        &self,
        _args: crate::block::pumpkin_block::EmitsRedstonePowerArgs<'_>,
    ) -> bool {
        false
    }

    async fn get_weak_redstone_power(
        &self,
        _args: crate::block::pumpkin_block::GetRedstonePowerArgs<'_>,
    ) -> u8 {
        0
    }

    async fn get_strong_redstone_power(
        &self,
        _args: crate::block::pumpkin_block::GetRedstonePowerArgs<'_>,
    ) -> u8 {
        0
    }

    async fn get_comparator_output(
        &self,
        _args: crate::block::pumpkin_block::GetComparatorOutputArgs<'_>,
    ) -> Option<u8> {
        None
    }
}
