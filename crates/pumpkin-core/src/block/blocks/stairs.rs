use crate::entity::EntityBase;
use pumpkin_data::block_properties::Half;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::block_properties::StairsShape;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId, Mirror, Rotation, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;

use crate::block::{
    BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs, PathComputationType,
};
use crate::world::World;

type StairsProperties = pumpkin_data::block_properties::WhiteWoolStairsLikeProperties;

#[pumpkin_block_from_tag("minecraft:stairs")]
pub struct StairBlock;

impl BlockBehaviour for StairBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut stair_props = StairsProperties::default(args.block);
        stair_props.waterlogged = args.replacing.water_source();

        stair_props.facing = args.player.get_entity().get_horizontal_facing();
        stair_props.half = match args.direction {
            BlockDirection::Up => Half::Top,
            BlockDirection::Down => Half::Bottom,
            _ => match args.use_item_on.cursor_pos.y {
                0.0..=0.5 => Half::Bottom,
                _ => Half::Top,
            },
        };

        stair_props.shape = compute_stair_shape(
            args.world,
            args.position,
            stair_props.facing,
            stair_props.half,
        );

        stair_props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let mut stair_props = StairsProperties::from_state_id(args.state_id);
        if stair_props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }

        if args.direction.is_horizontal() {
            stair_props.shape = compute_stair_shape(
                args.world,
                args.position,
                stair_props.facing,
                stair_props.half,
            );
            return stair_props.to_state_id(args.block);
        }

        args.state_id
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        let mut stair_props = StairsProperties::from_state_id(state_id);
        stair_props.facing = rotation.rotate_horizontal(stair_props.facing);
        BlockState::from_id(stair_props.to_state_id(block))
    }

    fn mirror(&self, block: &Block, state_id: BlockStateId, mirror: Mirror) -> &'static BlockState {
        let mut stair_props = StairsProperties::from_state_id(state_id);
        let direction = stair_props.facing;
        let shape = stair_props.shape;

        match mirror {
            Mirror::LeftRight => {
                if matches!(direction, HorizontalFacing::North | HorizontalFacing::South) {
                    stair_props.facing = stair_props.facing.opposite();
                    stair_props.shape = match shape {
                        StairsShape::Straight => StairsShape::Straight,
                        StairsShape::OuterLeft => StairsShape::OuterRight,
                        StairsShape::InnerRight => StairsShape::InnerLeft,
                        StairsShape::InnerLeft => StairsShape::InnerRight,
                        StairsShape::OuterRight => StairsShape::OuterLeft,
                    };
                    return BlockState::from_id(stair_props.to_state_id(block));
                }
            }
            Mirror::FrontBack => {
                if matches!(direction, HorizontalFacing::East | HorizontalFacing::West) {
                    stair_props.facing = stair_props.facing.opposite();
                    stair_props.shape = match shape {
                        StairsShape::Straight => StairsShape::Straight,
                        StairsShape::OuterLeft => StairsShape::OuterRight,
                        StairsShape::InnerRight => StairsShape::InnerRight,
                        StairsShape::InnerLeft => StairsShape::InnerLeft,
                        StairsShape::OuterRight => StairsShape::OuterLeft,
                    };
                    return BlockState::from_id(stair_props.to_state_id(block));
                }
            }
            Mirror::None => {}
        }

        BlockState::from_id(state_id)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

fn compute_stair_shape(
    world: &World,
    block_pos: &BlockPos,
    facing: HorizontalFacing,
    half: Half,
) -> StairsShape {
    let right_locked = get_stair_properties_if_exists(
        world,
        &block_pos.offset(facing.rotate_clockwise().to_offset()),
    )
    .is_some_and(|other_stair_props| {
        other_stair_props.half == half && other_stair_props.facing == facing
    });

    let left_locked = get_stair_properties_if_exists(
        world,
        &block_pos.offset(facing.rotate_counter_clockwise().to_offset()),
    )
    .is_some_and(|other_stair_props| {
        other_stair_props.half == half && other_stair_props.facing == facing
    });

    if left_locked && right_locked {
        return StairsShape::Straight;
    }

    if let Some(other_stair_props) =
        get_stair_properties_if_exists(world, &block_pos.offset(facing.to_offset()))
        && other_stair_props.half == half
    {
        if !left_locked && other_stair_props.facing == facing.rotate_clockwise() {
            return StairsShape::OuterRight;
        } else if !right_locked && other_stair_props.facing == facing.rotate_counter_clockwise() {
            return StairsShape::OuterLeft;
        }
    }

    if let Some(other_stair_props) =
        get_stair_properties_if_exists(world, &block_pos.offset(facing.opposite().to_offset()))
        && other_stair_props.half == half
    {
        if !right_locked && other_stair_props.facing == facing.rotate_clockwise() {
            return StairsShape::InnerRight;
        } else if !left_locked && other_stair_props.facing == facing.rotate_counter_clockwise() {
            return StairsShape::InnerLeft;
        }
    }

    StairsShape::Straight
}

fn get_stair_properties_if_exists(world: &World, block_pos: &BlockPos) -> Option<StairsProperties> {
    let (block, block_state) = world.get_block_and_state_id(block_pos);
    block
        .has_tag(&tag::Block::MINECRAFT_STAIRS)
        .then(|| StairsProperties::from_state_id(block_state))
}
