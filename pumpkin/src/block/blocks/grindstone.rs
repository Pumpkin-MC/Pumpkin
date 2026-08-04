use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockStateId, HorizontalFacingExt,
    block_properties::{AttachFace, BlockProperties, GrindstoneLikeProperties},
};
use pumpkin_inventory::grindstone_screen_handler::GrindstoneScreenHandler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture as ScreenHandlerBoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockAccessor;
use tokio::sync::Mutex;

use crate::block::CanPlaceAtArgs;
use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, NormalUseArgs};
use crate::block::{GetStateForNeighborUpdateArgs, OnPlaceArgs};

use super::abstract_wall_mounting::WallMountedBlock;

#[pumpkin_block("minecraft:grindstone")]
pub struct GrindstoneBlock;

impl BlockBehaviour for GrindstoneBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props =
                GrindstoneLikeProperties::from_state_id(args.block.default_state.id, args.block);
            (props.face, props.facing) =
                WallMountedBlock::get_placement_face(self, args.player, args.direction);

            props.to_state_id(args.block)
        })
    }

    // useWithoutItem (GrindstoneBlock.java:73-83): opens the repair/disenchant menu and awards
    // the interaction stat.
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            args.player
                .increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::InteractWithGrindstone as i32,
                    1,
                )
                .await;
            args.player
                .open_handled_screen(&GrindstoneScreenFactory, Some(*args.position))
                .await;

            BlockActionResult::Success
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        // Use the provided direction, or fallback to the current state's direction if missing
        let direction = args
            .direction
            .unwrap_or_else(|| self.get_direction(args.state.id, args.block));

        WallMountedBlock::can_place_at(self, args.block_accessor, args.position, direction)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move { WallMountedBlock::get_state_for_neighbor_update(self, args).await })
    }
}

impl WallMountedBlock for GrindstoneBlock {
    fn can_place_at<'a>(
        &'a self,
        _world: &'a dyn BlockAccessor,
        _pos: &'a BlockPos,
        _direction: BlockDirection,
    ) -> bool {
        true
    }

    fn get_direction(&self, state_id: BlockStateId, block: &Block) -> BlockDirection {
        let props = GrindstoneLikeProperties::from_state_id(state_id, block);
        match props.face {
            AttachFace::Floor => BlockDirection::Up,
            AttachFace::Ceiling => BlockDirection::Down,
            AttachFace::Wall => props.facing.to_block_direction(),
        }
    }
}

struct GrindstoneScreenFactory;

impl ScreenHandlerFactory for GrindstoneScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerBoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler: SharedScreenHandler = Arc::new(Mutex::new(GrindstoneScreenHandler::new(
                sync_id,
                player_inventory,
            )));
            Some(handler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        // No Bedrock-side `container.grindstone_title` key exists; unlike Loom/Stonecutter,
        // Bedrock has no direct equivalent to translate against.
        TextComponent::translate(
            pumpkin_data::translation::java::CONTAINER_GRINDSTONE_TITLE,
            &[],
        )
    }
}
