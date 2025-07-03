use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::BlockFace;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::chunk::TickPriority;
use pumpkin_world::world::BlockFlags;

type ButtonLikeProperties = pumpkin_data::block_properties::LeverLikeProperties;

use crate::block::blocks::abstruct_wall_mounting::WallMountedBlock;
use crate::block::blocks::redstone::lever::LeverLikePropertiesExt;
use crate::block::pumpkin_block::CanPlaceAtArgs;
use crate::block::pumpkin_block::EmitsRedstonePowerArgs;
use crate::block::pumpkin_block::GetRedstonePowerArgs;
use crate::block::pumpkin_block::GetStateForNeighborUpdateArgs;
use crate::block::pumpkin_block::OnPlaceArgs;
use crate::block::pumpkin_block::OnScheduledTickArgs;
use crate::block::pumpkin_block::OnStateReplacedArgs;
use crate::block::pumpkin_block::UseWithItemArgs;
use crate::block::pumpkin_block::{BlockMetadata, NormalUseArgs, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::world::World;

async fn click_button(world: &Arc<World>, block_pos: &BlockPos) {
    let (block, state) = world.get_block_and_block_state(block_pos).await;

    let mut button_props = ButtonLikeProperties::from_state_id(state.id, block);
    if !button_props.powered {
        button_props.powered = true;
        world
            .set_block_state(
                block_pos,
                button_props.to_state_id(block),
                BlockFlags::NOTIFY_ALL,
            )
            .await;
        let delay = if block == &Block::STONE_BUTTON {
            20
        } else {
            30
        };
        world
            .schedule_block_tick(block, *block_pos, delay, TickPriority::Normal)
            .await;
        ButtonBlock::update_neighbors(world, block_pos, &button_props).await;
    }
}

pub struct ButtonBlock;

impl BlockMetadata for ButtonBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:buttons").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for ButtonBlock {
    async fn normal_use(&self, args: NormalUseArgs<'_>) {
        click_button(args.world, args.location).await;
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        click_button(args.world, args.location).await;
        BlockActionResult::Consume
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.location).await;
        let mut props = ButtonLikeProperties::from_state_id(state.id, args.block);
        props.powered = false;
        args.world
            .set_block_state(
                args.location,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            )
            .await;
        Self::update_neighbors(args.world, args.location, &props).await;
    }

    async fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    async fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let button_props = ButtonLikeProperties::from_state_id(args.state.id, args.block);
        if button_props.powered { 15 } else { 0 }
    }

    async fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let button_props = ButtonLikeProperties::from_state_id(args.state.id, args.block);
        if button_props.powered && button_props.get_direction() == args.direction {
            15
        } else {
            0
        }
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if !args.moved {
            let button_props = ButtonLikeProperties::from_state_id(args.old_state_id, args.block);
            if button_props.powered {
                Self::update_neighbors(args.world, args.location, &button_props).await;
            }
        }
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props =
            ButtonLikeProperties::from_state_id(args.block.default_state.id, args.block);
        (props.face, props.facing) =
            WallMountedBlock::get_placement_face(self, args.player, args.direction);

        props.to_state_id(args.block)
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        WallMountedBlock::can_place_at(self, args.block_accessor, args.location, args.direction)
            .await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        WallMountedBlock::get_state_for_neighbor_update(self, args).await
    }
}

#[async_trait]
impl WallMountedBlock for ButtonBlock {
    fn get_direction(&self, state_id: BlockStateId, block: &Block) -> BlockDirection {
        let props = ButtonLikeProperties::from_state_id(state_id, block);
        match props.face {
            BlockFace::Floor => BlockDirection::Up,
            BlockFace::Ceiling => BlockDirection::Down,
            BlockFace::Wall => props.facing.to_block_direction(),
        }
    }
}

impl ButtonBlock {
    async fn update_neighbors(
        world: &Arc<World>,
        block_pos: &BlockPos,
        props: &ButtonLikeProperties,
    ) {
        let direction = props.get_direction().opposite();
        world.update_neighbors(block_pos, None).await;
        world
            .update_neighbors(&block_pos.offset(direction.to_offset()), None)
            .await;
    }
}
