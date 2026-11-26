use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use crate::block::{
    GetComparatorOutputArgs, GetScreenHandlerFactoryArgs, OnPlaceArgs, OnSyncedBlockEventArgs,
    PlacedArgs,
};
use crate::block::{
    registry::BlockActionResult,
    {BlockBehaviour, NormalUseArgs},
};

use crate::block::entities::shulker_box::ShulkerBoxBlockEntity;
use crate::world::World;
use pumpkin_data::block_properties::ShulkerBoxProperties;
use pumpkin_data::translation;
use pumpkin_data::{BlockDirection, BlockStateId, FacingExt};
use pumpkin_inventory::Inventory;
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;

struct ShulkerBoxScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for ShulkerBoxScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let handler = create_generic_9x3(sync_id, player_inventory, self.0.clone(), player);
        let screen_handler_arc = Arc::new(Mutex::new(handler));

        Some(screen_handler_arc as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_SHULKERBOX,
            translation::bedrock::CONTAINER_SHULKERBOX
        )
    }
}

#[pumpkin_block_from_tag("minecraft:shulker_boxes")]
pub struct ShulkerBoxBlock;

type EndRodLikeProperties = pumpkin_data::block_properties::EndRodLikeProperties;

impl BlockBehaviour for ShulkerBoxBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = EndRodLikeProperties::default(args.block);
        props.facing = args.direction.to_facing().opposite();
        props.to_state_id(args.block)
    }

    fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        // On the server, we don't need the Animation steps for now, because the client is responsible for that.
        // TODO: Do not open the shulker box when it is currently closing
        args.r#type == Self::OPEN_ANIMATION_EVENT_TYPE
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let barrel_block_entity = ShulkerBoxBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(barrel_block_entity));
        }
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if !Self::can_open(args.world, args.position) {
            return BlockActionResult::Success;
        }

        if let Some(factory) = self.get_screen_handler_factory(GetScreenHandlerFactoryArgs {
            server: args.server,
            world: args.world,
            block: args.block,
            position: args.position,
            player: args.player,
        }) {
            args.player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::OpenShulkerBox as i32,
                1,
            );
            args.player
                .open_handled_screen(factory.as_ref(), Some(*args.position));
        }

        BlockActionResult::Success
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        let block_entity = args.world.get_block_entity(args.position)?;
        let inventory = block_entity.get_inventory()?;
        Some(Box::new(ShulkerBoxScreenFactory(inventory)))
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        crate::block::container_comparator_output(&args)
    }
}

impl ShulkerBoxBlock {
    pub const OPEN_ANIMATION_EVENT_TYPE: u8 = 1;

    const LID_OPEN_PROGRESS: f64 = 0.5;
    const LID_OPEN_INSET: f64 = 1.0E-6;

    /// Vanilla `ShulkerBoxBlock.canOpen`. Pumpkin has no server-side animation status;
    /// a box with viewers is treated as open, and a closing box as closed.
    fn can_open(world: &World, position: &BlockPos) -> bool {
        if let Some(block_entity) = world.get_block_entity(position)
            && let Some(shulker_box) = block_entity
                .as_any()
                .downcast_ref::<ShulkerBoxBlockEntity>()
            && shulker_box.viewers.current.load(Ordering::Relaxed) > 0
        {
            return true;
        }

        let state = world.get_block_state(position);
        let facing = ShulkerBoxProperties::from_state_id(state.id)
            .facing
            .to_block_direction();
        let lid_open_bounding_box = Self::lid_open_bounding_box(position, facing);

        world.no_block_collision(lid_open_bounding_box)
            && world.no_entity_collision(None, lid_open_bounding_box)
    }

    /// Vanilla `Shulker.getProgressDeltaAabb(1.0, facing, 0.0, 0.5, bottom center)`:
    /// the half of the neighbouring block the lid opens into.
    fn lid_open_bounding_box(position: &BlockPos, facing: BlockDirection) -> BoundingBox {
        let mut bounds = BoundingBox::from_block(&position.offset(facing.to_offset()));
        match facing {
            BlockDirection::Down => bounds.min.y += Self::LID_OPEN_PROGRESS,
            BlockDirection::Up => bounds.max.y -= Self::LID_OPEN_PROGRESS,
            BlockDirection::North => bounds.min.z += Self::LID_OPEN_PROGRESS,
            BlockDirection::South => bounds.max.z -= Self::LID_OPEN_PROGRESS,
            BlockDirection::West => bounds.min.x += Self::LID_OPEN_PROGRESS,
            BlockDirection::East => bounds.max.x -= Self::LID_OPEN_PROGRESS,
        }
        bounds.contract_all(Self::LID_OPEN_INSET)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lid_open_bounding_box_matches_vanilla() {
        let position = BlockPos::new(0, 64, 0);
        let cases = [
            (BlockDirection::Up, [0.0, 65.0, 0.0], [1.0, 65.5, 1.0]),
            (BlockDirection::Down, [0.0, 63.5, 0.0], [1.0, 64.0, 1.0]),
            (BlockDirection::North, [0.0, 64.0, -0.5], [1.0, 65.0, 0.0]),
            (BlockDirection::South, [0.0, 64.0, 1.0], [1.0, 65.0, 1.5]),
            (BlockDirection::West, [-0.5, 64.0, 0.0], [0.0, 65.0, 1.0]),
            (BlockDirection::East, [1.0, 64.0, 0.0], [1.5, 65.0, 1.0]),
        ];

        for (facing, min, max) in cases {
            let bounds = ShulkerBoxBlock::lid_open_bounding_box(&position, facing);
            let inset = ShulkerBoxBlock::LID_OPEN_INSET;
            let close = |a: f64, b: f64| (a - b).abs() < 1.0e-9;
            assert!(close(bounds.min.x, min[0] + inset));
            assert!(close(bounds.min.y, min[1] + inset));
            assert!(close(bounds.min.z, min[2] + inset));
            assert!(close(bounds.max.x, max[0] - inset));
            assert!(close(bounds.max.y, max[1] - inset));
            assert!(close(bounds.max.z, max[2] - inset));
        }
    }
}
