use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block_properties::{
    BlockProperties, ChestLikeProperties, ChestType, HorizontalFacing,
};
use pumpkin_data::entity::EntityPose;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::chest::ChestBlockEntity;
use pumpkin_world::world::BlockFlags;

use crate::block::pumpkin_block::{
    BrokenArgs, OnPlaceArgs, OnStateReplacedArgs, PlacedArgs, UseWithItemArgs,
};
use crate::entity::EntityBase;
use crate::world::World;
use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    entity::player::Player,
};

#[pumpkin_block("minecraft:chest")]
pub struct ChestBlock;

#[async_trait]
impl PumpkinBlock for ChestBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut chest_props = ChestLikeProperties::default(args.block);

        chest_props.waterlogged = args.replacing.water_source();

        let (r#type, facing) = compute_chest_props(
            args.world,
            args.player,
            args.block,
            args.location,
            args.direction,
        )
        .await;
        chest_props.facing = facing;
        chest_props.r#type = r#type;

        chest_props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let chest = ChestBlockEntity::new(*args.location);
        args.world.add_block_entity(Arc::new(chest)).await;

        let chest_props = ChestLikeProperties::from_state_id(args.state_id, args.block);
        let connected_towards = match chest_props.r#type {
            ChestType::Single => return,
            ChestType::Left => chest_props.facing.rotate_clockwise(),
            ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
        };

        if let Some(mut neighbor_props) = get_chest_properties_if_can_connect(
            args.world,
            args.block,
            args.location,
            chest_props.facing,
            connected_towards,
            ChestType::Single,
        )
        .await
        {
            neighbor_props.r#type = chest_props.r#type.opposite();

            args.world
                .set_block_state(
                    &args.location.offset(connected_towards.to_offset()),
                    neighbor_props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        }
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        args.world.remove_block_entity(args.location).await;
    }

    async fn use_with_item(&self, _args: UseWithItemArgs<'_>) -> BlockActionResult {
        BlockActionResult::Consume
    }

    async fn broken(&self, args: BrokenArgs<'_>) {
        let chest_props = ChestLikeProperties::from_state_id(args.state.id, args.block);
        let connected_towards = match chest_props.r#type {
            ChestType::Single => return,
            ChestType::Left => chest_props.facing.rotate_clockwise(),
            ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
        };

        if let Some(mut neighbor_props) = get_chest_properties_if_can_connect(
            args.world,
            args.block,
            args.location,
            chest_props.facing,
            connected_towards,
            chest_props.r#type.opposite(),
        )
        .await
        {
            neighbor_props.r#type = ChestType::Single;

            args.world
                .set_block_state(
                    &args.location.offset(connected_towards.to_offset()),
                    neighbor_props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        }
    }
}

async fn compute_chest_props(
    world: &World,
    player: &Player,
    block: &Block,
    block_pos: &BlockPos,
    face: BlockDirection,
) -> (ChestType, HorizontalFacing) {
    let chest_facing = player.get_entity().get_horizontal_facing().opposite();

    if player.get_entity().pose.load() == EntityPose::Crouching {
        let Some(face) = face.to_horizontal_facing() else {
            return (ChestType::Single, chest_facing);
        };

        let (clicked_block, clicked_block_state) = world
            .get_block_and_block_state(&block_pos.offset(face.to_offset()))
            .await;

        if clicked_block == block {
            let clicked_props =
                ChestLikeProperties::from_state_id(clicked_block_state.id, clicked_block);

            if clicked_props.r#type != ChestType::Single {
                return (ChestType::Single, chest_facing);
            }

            if clicked_props.facing.rotate_clockwise() == face {
                return (ChestType::Left, clicked_props.facing);
            } else if clicked_props.facing.rotate_counter_clockwise() == face {
                return (ChestType::Right, clicked_props.facing);
            }
        }

        return (ChestType::Single, chest_facing);
    }

    if get_chest_properties_if_can_connect(
        world,
        block,
        block_pos,
        chest_facing,
        chest_facing.rotate_clockwise(),
        ChestType::Single,
    )
    .await
    .is_some()
    {
        (ChestType::Left, chest_facing)
    } else if get_chest_properties_if_can_connect(
        world,
        block,
        block_pos,
        chest_facing,
        chest_facing.rotate_counter_clockwise(),
        ChestType::Single,
    )
    .await
    .is_some()
    {
        (ChestType::Right, chest_facing)
    } else {
        (ChestType::Single, chest_facing)
    }
}

async fn get_chest_properties_if_can_connect(
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
    facing: HorizontalFacing,
    direction: HorizontalFacing,
    wanted_type: ChestType,
) -> Option<ChestLikeProperties> {
    let (neighbor_block, neighbor_block_state) = world
        .get_block_and_block_state(&block_pos.offset(direction.to_offset()))
        .await;

    if neighbor_block != block {
        return None;
    }

    let neighbor_props =
        ChestLikeProperties::from_state_id(neighbor_block_state.id, neighbor_block);
    if neighbor_props.facing == facing && neighbor_props.r#type == wanted_type {
        return Some(neighbor_props);
    }

    None
}

trait ChestTypeExt {
    fn opposite(&self) -> ChestType;
}

impl ChestTypeExt for ChestType {
    fn opposite(&self) -> Self {
        match self {
            Self::Single => Self::Single,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}
