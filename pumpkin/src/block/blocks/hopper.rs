use std::sync::Arc;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::{BlockFuture, OnNeighborUpdateArgs, OnPlaceArgs, PlacedArgs};
use crate::block::{
    registry::BlockActionResult,
    {BlockBehaviour, NormalUseArgs},
};
use crate::world::World;

use pumpkin_data::block_properties::{BlockProperties, HopperFacing};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_inventory::generic_container_screen_handler::create_hopper;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::hopper::HopperBlockEntity;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::world::BlockFlags;
use tokio::sync::Mutex;

struct HopperBlockScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for HopperBlockScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let concrete_handler = create_hopper(sync_id, player_inventory, self.0.clone()).await;

            let concrete_arc = Arc::new(Mutex::new(concrete_handler));

            Some(concrete_arc as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate("container.hopper", &[])
    }
}

#[pumpkin_block("minecraft:hopper")]
pub struct HopperBlock;

type HopperLikeProperties = pumpkin_data::block_properties::HopperLikeProperties;

impl BlockBehaviour for HopperBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(inventory) = block_entity.get_inventory()
            {
                args.player
                    .open_handled_screen(&HopperBlockScreenFactory(inventory))
                    .await;
            }

            BlockActionResult::Success
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
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
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let props = HopperLikeProperties::from_state_id(args.state_id, args.block);
            let hopper_block_entity = HopperBlockEntity::new(*args.position, props.facing);
            args.world
                .add_block_entity(Arc::new(hopper_block_entity))
                .await;
            if Block::from_state_id(args.old_state_id) != Block::from_state_id(args.state_id) {
                check_powered_state(args.world, args.position, args.state_id, args.block).await;
            }
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            check_powered_state(
                args.world,
                args.position,
                args.world.get_block_state_id(args.position).await,
                args.block,
            )
            .await;
        })
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
