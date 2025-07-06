use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockHalf;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::block_properties::StairShape;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::Tagable;
use pumpkin_data::tag::get_tag_values;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

use crate::block::pumpkin_block::OnNeighborUpdateArgs;
use crate::block::pumpkin_block::OnPlaceArgs;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::world::World;

type StairsProperties = pumpkin_data::block_properties::OakStairsLikeProperties;

pub struct StairBlock;

impl BlockMetadata for StairBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:stairs").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for StairBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut stair_props = StairsProperties::default(args.block);
        stair_props.waterlogged = args.replacing.water_source();

        stair_props.facing = args.player.living_entity.entity.get_horizontal_facing();
        stair_props.half = match args.direction {
            BlockDirection::Up => BlockHalf::Top,
            BlockDirection::Down => BlockHalf::Bottom,
            _ => match args.use_item_on.cursor_pos.y {
                0.0...0.5 => BlockHalf::Bottom,
                0.5...1.0 => BlockHalf::Top,

                // This cannot happen normally
                #[allow(clippy::match_same_arms)]
                _ => BlockHalf::Bottom,
            },
        };

        stair_props.shape = compute_stair_shape(
            args.world,
            args.location,
            stair_props.facing,
            stair_props.half,
        )
        .await;

        stair_props.to_state_id(args.block)
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.location).await;
        let mut stair_props = StairsProperties::from_state_id(state_id, args.block);

        let new_shape = compute_stair_shape(
            args.world,
            args.location,
            stair_props.facing,
            stair_props.half,
        )
        .await;

        if stair_props.shape != new_shape {
            stair_props.shape = new_shape;
            args.world
                .set_block_state(
                    args.location,
                    stair_props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
    }
}

async fn compute_stair_shape(
    world: &World,
    block_pos: &BlockPos,
    facing: HorizontalFacing,
    half: BlockHalf,
) -> StairShape {
    let right_locked = get_stair_properties_if_exists(
        world,
        &block_pos.offset(facing.rotate_clockwise().to_offset()),
    )
    .await
    .is_some_and(|other_stair_props| {
        other_stair_props.half == half && other_stair_props.facing == facing
    });

    let left_locked = get_stair_properties_if_exists(
        world,
        &block_pos.offset(facing.rotate_counter_clockwise().to_offset()),
    )
    .await
    .is_some_and(|other_stair_props| {
        other_stair_props.half == half && other_stair_props.facing == facing
    });

    if left_locked && right_locked {
        return StairShape::Straight;
    }

    if let Some(other_stair_props) =
        get_stair_properties_if_exists(world, &block_pos.offset(facing.to_offset())).await
    {
        if other_stair_props.half == half {
            if !left_locked && other_stair_props.facing == facing.rotate_clockwise() {
                return StairShape::OuterRight;
            } else if !right_locked && other_stair_props.facing == facing.rotate_counter_clockwise()
            {
                return StairShape::OuterLeft;
            }
        }
    }

    if let Some(other_stair_props) =
        get_stair_properties_if_exists(world, &block_pos.offset(facing.opposite().to_offset()))
            .await
    {
        if other_stair_props.half == half {
            if !right_locked && other_stair_props.facing == facing.rotate_clockwise() {
                return StairShape::InnerRight;
            } else if !left_locked && other_stair_props.facing == facing.rotate_counter_clockwise()
            {
                return StairShape::InnerLeft;
            }
        }
    }

    StairShape::Straight
}

async fn get_stair_properties_if_exists(
    world: &World,
    block_pos: &BlockPos,
) -> Option<StairsProperties> {
    let (block, block_state) = world.get_block_and_block_state(block_pos).await;
    block
        .is_tagged_with("#minecraft:stairs")
        .unwrap()
        .then(|| StairsProperties::from_state_id(block_state.id, block))
}
