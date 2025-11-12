use pumpkin_data::tag::{Taggable, get_tag_values};
use std::sync::Arc;

use crate::block::blocks::chests::{
    ChestTypeExt, calculate_comparator_output, chest_broken, chest_normal_use,
    player_crouching_behaviour,
};
use crate::block::{
    BrokenArgs, GetComparatorOutputArgs, NormalUseArgs, OnPlaceArgs, OnSyncedBlockEventArgs,
    PlacedArgs,
};
use crate::entity::EntityBase;
use crate::world::World;
use crate::{
    block::{BlockBehaviour, registry::BlockActionResult},
    entity::player::Player,
};
use async_trait::async_trait;
use pumpkin_data::block_properties::{
    BlockProperties, ChestLikeProperties, ChestType, HorizontalFacing,
};
use pumpkin_data::entity::EntityPose;
use pumpkin_data::tag::Block::MINECRAFT_COPPER_CHESTS;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::chest::ChestBlockEntity;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block_from_tag("minecraft:copper_chests")]
pub struct CopperChestBlock;

#[async_trait]
impl BlockBehaviour for CopperChestBlock {
    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        chest_normal_use(&args).await
    }

    async fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        // On the server, we don't need to do more because the client is responsible for that.
        args.r#type == Self::LID_ANIMATION_EVENT_TYPE
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut chest_props = ChestLikeProperties::default(args.block);

        chest_props.waterlogged = args.replacing.water_source();

        let (r#type, facing) = compute_copper_chest_props(
            args.world,
            args.player,
            args.block,
            args.position,
            args.direction,
        )
        .await;
        chest_props.facing = facing;
        chest_props.r#type = r#type;

        chest_props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let chest = ChestBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(chest)).await;
        let chest_props = ChestLikeProperties::from_state_id(args.state_id, args.block);
        let connected_towards = match chest_props.r#type {
            ChestType::Single => return,
            ChestType::Left => chest_props.facing.rotate_clockwise(),
            ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
        };

        if let Some(mut neighbor_props) = get_copper_chest_properties_if_can_connect(
            args.world,
            args.block,
            args.position,
            chest_props.facing,
            connected_towards,
            ChestType::Single,
        )
        .await
        {
            let neighbor_block = args
                .world
                .get_block(&args.position.offset(connected_towards.to_offset()))
                .await;
            let block_id = if args.block.id < neighbor_block.id {
                args.block.id
            } else {
                neighbor_block.id
            };
            neighbor_props.r#type = chest_props.r#type.opposite();

            args.world
                .set_block_state(
                    &args.position.offset(connected_towards.to_offset()),
                    neighbor_props.to_state_id(Block::from_id(block_id)),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
            if args.block.id != block_id {
                args.world
                    .set_block_state(
                        args.position,
                        chest_props.to_state_id(Block::from_id(block_id)),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        }
    }

    async fn broken(&self, args: BrokenArgs<'_>) {
        chest_broken(args).await;
    }

    async fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        calculate_comparator_output(args).await
    }
}

impl CopperChestBlock {
    pub const LID_ANIMATION_EVENT_TYPE: u8 = 1;
}

async fn compute_copper_chest_props(
    world: &World,
    player: &Player,
    block: &Block,
    block_pos: &BlockPos,
    face: BlockDirection,
) -> (ChestType, HorizontalFacing) {
    let chest_facing = player.get_entity().get_horizontal_facing().opposite();

    if player.get_entity().pose.load() == EntityPose::Crouching {
        return player_crouching_behaviour(world, block, block_pos, face, chest_facing).await;
    }

    if get_copper_chest_properties_if_can_connect(
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
    } else if get_copper_chest_properties_if_can_connect(
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

async fn get_copper_chest_properties_if_can_connect(
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
    facing: HorizontalFacing,
    direction: HorizontalFacing,
    wanted_type: ChestType,
) -> Option<ChestLikeProperties> {
    let (neighbor_block, neighbor_block_state) = world
        .get_block_and_state_id(&block_pos.offset(direction.to_offset()))
        .await;

    if neighbor_block != block && !neighbor_block.has_tag(&MINECRAFT_COPPER_CHESTS) {
        return None;
    }
    let neighbor_props = ChestLikeProperties::from_state_id(neighbor_block_state, neighbor_block);
    if neighbor_props.facing == facing && neighbor_props.r#type == wanted_type {
        return Some(neighbor_props);
    }

    None
}
