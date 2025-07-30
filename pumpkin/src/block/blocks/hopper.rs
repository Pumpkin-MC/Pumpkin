use std::sync::Arc;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::{OnNeighborUpdateArgs, OnPlaceArgs, PlacedArgs};
use crate::block::{
    registry::BlockActionResult,
    {BlockBehaviour, NormalUseArgs},
};
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, HopperFacing};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_inventory::generic_container_screen_handler::create_hopper;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerFactory};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::hopper::HopperBlockEntity;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::world::BlockFlags;
use tokio::sync::Mutex;

struct HopperBlockScreenFactory(Arc<dyn Inventory>);

#[async_trait]
impl ScreenHandlerFactory for HopperBlockScreenFactory {
    async fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<Arc<Mutex<dyn ScreenHandler>>> {
        Some(Arc::new(Mutex::new(create_hopper(
            sync_id,
            player_inventory,
            self.0.clone(),
        ).await)))
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate("container.hopper", &[])
    }
}

#[pumpkin_block("minecraft:hopper")]
pub struct HopperBlock;

type HopperLikeProperties = pumpkin_data::block_properties::HopperLikeProperties;

#[async_trait]
impl BlockBehaviour for HopperBlock {
    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if let Some(block_entity) = args.world.get_block_entity(args.position).await {
            if let Some(inventory) = block_entity.get_inventory() {
                args.player
                    .open_handled_screen(&HopperBlockScreenFactory(inventory))
                    .await;
            }
        }

        BlockActionResult::Success
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = HopperLikeProperties::default(args.block);
        props.facing = match args.direction {
            BlockDirection::North => HopperFacing::North,
            BlockDirection::East => HopperFacing::East,
            BlockDirection::South => HopperFacing::South,
            BlockDirection::West => HopperFacing::West,
            BlockDirection::Up | BlockDirection::Down => HopperFacing::Down,
        };
        props.enabled = true;
        props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let props = HopperLikeProperties::from_state_id(args.state_id, args.block);
        let hopper_block_entity = HopperBlockEntity::new(*args.position, props.facing);
        args.world
            .add_block_entity(Arc::new(hopper_block_entity))
            .await;
        if Block::from_state_id(args.old_state_id) != Block::from_state_id(args.state_id) {
            check_powered_state(args.world, args.position, args.state_id, args.block).await;
        }
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        check_powered_state(
            args.world,
            args.position,
            args.world.get_block_state_id(args.position).await,
            args.block,
        )
        .await;
    }
}

async fn check_powered_state(
    world: &Arc<World>,
    pos: &BlockPos,
    state_id: BlockStateId,
    block: &Block,
) {
    let signal = !block_receives_redstone_power(world, pos).await;
    let mut state = HopperLikeProperties::from_state_id(state_id, block);
    if signal != state.enabled {
        state.enabled = signal;
        world
            .set_block_state(pos, state.to_state_id(block), BlockFlags::NOTIFY_LISTENERS)
            .await;
    }
}
