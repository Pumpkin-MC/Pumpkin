use std::sync::Arc;

use crate::{
    block::{
        BlockBehaviour, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
        OnLandedUponArgs, OnPlaceArgs, PathComputationType, PlacedArgs,
    },
    entity::player::Player,
    world::World,
};
use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId,
    block_properties::{PointedDripstoneLikeProperties, SpeleothemThickness, VerticalDirection},
    fluid::Fluid,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};

#[pumpkin_block_from_tag("minecraft:speleothems")]
pub struct DripstoneBlock;

impl BlockBehaviour for DripstoneBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at_pos(
            args.block_accessor,
            args.block,
            args.position,
            args.direction,
            args.player,
        )
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut dripstone_props = PointedDripstoneLikeProperties::default(args.block);
        dripstone_props.waterlogged = args.replacing.water_source();
        let Some(support_block_ver_dir) = get_support_block_vertical_direction(
            args.world,
            args.block,
            args.position,
            Some(args.direction),
            Some(args.player),
        ) else {
            return Block::AIR.default_state.id;
        };

        dripstone_props.vertical_direction = flip_dir(support_block_ver_dir);
        dripstone_props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let (len, vertical_dir) = get_stalagmite_or_stalactite_len_and_dir_from_tip_pos(
            args.world,
            args.block,
            args.position,
            args.state_id,
        );
        match vertical_dir {
            VerticalDirection::Up => {
                update_stalagmite(args.world, args.block, len, args.position);
            }
            VerticalDirection::Down => {
                update_stalactite(args.world, args.block, len, args.position);
            }
        }
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        let broken_dripstone_props = PointedDripstoneLikeProperties::from_state_id(args.state.id);
        let new_tip_pos = match broken_dripstone_props.vertical_direction {
            VerticalDirection::Up => args.position.down(),
            VerticalDirection::Down => args.position.up(),
        };

        let (len, vertical_dir) = get_stalagmite_or_stalactite_len_and_dir_from_tip_pos(
            args.world,
            args.block,
            &new_tip_pos,
            args.state.id,
        );
        match vertical_dir {
            VerticalDirection::Up => {
                update_stalagmite(args.world, args.block, len, &new_tip_pos);
            }
            VerticalDirection::Down => {
                update_stalactite(args.world, args.block, len, &new_tip_pos);
            }
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let mut dripstone_props = PointedDripstoneLikeProperties::from_state_id(args.state_id);
        if dripstone_props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        if !can_place_at_pos(args.world, args.block, args.position, None, None) {
            return Block::AIR.default_state.id;
        }
        if dripstone_props.thickness != SpeleothemThickness::TipMerge {
            return args.state_id;
        }
        match dripstone_props.vertical_direction {
            VerticalDirection::Up => {
                let block_above = args.world.get_block(&args.position.up());
                if block_above != args.block {
                    dripstone_props.thickness = SpeleothemThickness::Tip;
                    return dripstone_props.to_state_id(args.block);
                }
            }
            VerticalDirection::Down => {
                let block_below = args.world.get_block(&args.position.down());
                if block_below != args.block {
                    dripstone_props.thickness = SpeleothemThickness::Tip;
                    return dripstone_props.to_state_id(args.block);
                }
            }
        }
        args.state_id
    }

    fn on_landed_upon(&self, args: OnLandedUponArgs<'_>) {
        let entity_pos = args.entity.get_entity().pos.load();
        let pos = BlockPos::floored(entity_pos.x, entity_pos.y, entity_pos.z);
        let (block, state_id) = args.world.get_block_and_state_id(&pos);
        if block == &Block::POINTED_DRIPSTONE {
            let props = PointedDripstoneLikeProperties::from_state_id(state_id);
            if props.vertical_direction == VerticalDirection::Up
                && props.thickness == SpeleothemThickness::Tip
            {
                if let Some(living) = args.entity.get_living_entity() {
                    living.handle_fall_damage(args.entity, args.fall_distance + 2.5, 2.0);
                }
                return;
            }
        }
        if let Some(living) = args.entity.get_living_entity() {
            living.handle_fall_damage(args.entity, args.fall_distance, 1.0);
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

fn update_stalagmite(
    world: &Arc<World>,
    speleothem: &Block,
    stalagmite_len: u8,
    tip_pos: &BlockPos,
) {
    let block_above = world.get_block(&tip_pos.up());
    if block_above == speleothem {
        modify_dripstone_thickness_to(world, speleothem, tip_pos, SpeleothemThickness::TipMerge);
        modify_dripstone_thickness_to(
            world,
            speleothem,
            &tip_pos.up(),
            SpeleothemThickness::TipMerge,
        );
    } else {
        modify_dripstone_thickness_to(world, speleothem, tip_pos, SpeleothemThickness::Tip);
    }
    match stalagmite_len {
        2 => {
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.down_height(1),
                SpeleothemThickness::Frustum,
            );
        }
        3 => {
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.down_height(1),
                SpeleothemThickness::Frustum,
            );
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.down_height(2),
                SpeleothemThickness::Base,
            );
        }
        4 => {
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.down_height(1),
                SpeleothemThickness::Frustum,
            );
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.down_height(2),
                SpeleothemThickness::Middle,
            );
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.down_height(3),
                SpeleothemThickness::Base,
            );
        }
        5 => {
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.down_height(1),
                SpeleothemThickness::Frustum,
            );
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.down_height(2),
                SpeleothemThickness::Middle,
            );
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.down_height(3),
                SpeleothemThickness::Middle,
            );
        }
        _ => {}
    }
}

fn update_stalactite(
    world: &Arc<World>,
    speleothem: &Block,
    stalactite_len: u8,
    tip_pos: &BlockPos,
) {
    let block_below = world.get_block(&tip_pos.down());
    if block_below == speleothem {
        modify_dripstone_thickness_to(world, speleothem, tip_pos, SpeleothemThickness::TipMerge);
        modify_dripstone_thickness_to(
            world,
            speleothem,
            &tip_pos.down(),
            SpeleothemThickness::TipMerge,
        );
    } else {
        modify_dripstone_thickness_to(world, speleothem, tip_pos, SpeleothemThickness::Tip);
    }
    match stalactite_len {
        2 => {
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.up_height(1),
                SpeleothemThickness::Frustum,
            );
        }
        3 => {
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.up_height(1),
                SpeleothemThickness::Frustum,
            );
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.up_height(2),
                SpeleothemThickness::Base,
            );
        }
        4 => {
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.up_height(1),
                SpeleothemThickness::Frustum,
            );
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.up_height(2),
                SpeleothemThickness::Middle,
            );
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.up_height(3),
                SpeleothemThickness::Base,
            );
        }
        5 => {
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.up_height(1),
                SpeleothemThickness::Frustum,
            );
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.up_height(2),
                SpeleothemThickness::Middle,
            );
            modify_dripstone_thickness_to(
                world,
                speleothem,
                &tip_pos.up_height(3),
                SpeleothemThickness::Middle,
            );
        }
        _ => {}
    }
}

fn get_stalagmite_or_stalactite_len_and_dir_from_tip_pos(
    world: &Arc<World>,
    speleothem: &Block,
    position: &BlockPos,
    block_state_id: BlockStateId,
) -> (u8, VerticalDirection) {
    let props = PointedDripstoneLikeProperties::from_state_id(block_state_id);

    let mut dripstone_len = 1;
    let mut next_dripstone_pos = offset_pos_by_vertical_dir(position, props.vertical_direction);
    // We don't care if it's longer than 5 blocks because of how the thickness system works.
    while dripstone_len < 5 {
        if world.get_block(&next_dripstone_pos) != speleothem {
            break;
        }
        next_dripstone_pos =
            offset_pos_by_vertical_dir(&next_dripstone_pos, props.vertical_direction);
        dripstone_len += 1;
    }
    (dripstone_len, props.vertical_direction)
}

fn can_place_at_pos(
    block_accessor: &dyn BlockAccessor,
    speleothem: &Block,
    position: &BlockPos,
    placing_direction: Option<BlockDirection>,
    player_option: Option<&Player>,
) -> bool {
    let Some(support_block_vertical_direction) = get_support_block_vertical_direction(
        block_accessor,
        speleothem,
        position,
        placing_direction,
        player_option,
    ) else {
        return false;
    };
    let support_pos = match support_block_vertical_direction {
        VerticalDirection::Up => position.up(),
        VerticalDirection::Down => position.down(),
    };
    can_support_dripstone(
        block_accessor,
        speleothem,
        &support_pos,
        support_block_vertical_direction,
    )
}

fn get_support_block_vertical_direction(
    block_accessor: &dyn BlockAccessor,
    speleothem: &Block,
    position: &BlockPos,
    placing_direction_wrapper: Option<BlockDirection>,
    player_option: Option<&Player>,
) -> Option<VerticalDirection> {
    let Some(placing_direction) = placing_direction_wrapper else {
        let (block, state) = block_accessor.get_block_and_state(position);
        if block != speleothem {
            return None;
        }
        let props = PointedDripstoneLikeProperties::from_state_id(state.id);
        return Some(flip_dir(props.vertical_direction));
    };
    match block_direction_to_vertical_direction(placing_direction) {
        Some(ver_dir) => match ver_dir {
            VerticalDirection::Up => {
                if can_support_dripstone(
                    block_accessor,
                    speleothem,
                    &position.up(),
                    VerticalDirection::Up,
                ) {
                    return Some(VerticalDirection::Up);
                } else if can_support_dripstone(
                    block_accessor,
                    speleothem,
                    &position.down(),
                    VerticalDirection::Down,
                ) {
                    return Some(VerticalDirection::Down);
                }
                None
            }
            VerticalDirection::Down => {
                if can_support_dripstone(
                    block_accessor,
                    speleothem,
                    &position.down(),
                    VerticalDirection::Down,
                ) {
                    return Some(VerticalDirection::Down);
                } else if can_support_dripstone(
                    block_accessor,
                    speleothem,
                    &position.up(),
                    VerticalDirection::Up,
                ) {
                    return Some(VerticalDirection::Up);
                }
                None
            }
        },
        None => player_option.map_or(Some(VerticalDirection::Up), |player| {
            let (_, pitch) = player.rotation();
            let (can_place_above, can_place_below) = (
                can_support_dripstone(
                    block_accessor,
                    speleothem,
                    &position.up(),
                    VerticalDirection::Up,
                ),
                can_support_dripstone(
                    block_accessor,
                    speleothem,
                    &position.down(),
                    VerticalDirection::Down,
                ),
            );
            match (can_place_above, can_place_below) {
                (true, true) => {
                    if pitch > 0.0 {
                        Some(VerticalDirection::Down)
                    } else {
                        Some(VerticalDirection::Up)
                    }
                }
                (false, false) => None,
                (true, false) => Some(VerticalDirection::Up),
                (false, true) => Some(VerticalDirection::Down),
            }
        }),
    }
}

fn can_support_dripstone(
    block_accessor: &dyn BlockAccessor,
    speleothem: &Block,
    support_pos: &BlockPos,
    support_dir: VerticalDirection,
) -> bool {
    let (support_block, support_state) = block_accessor.get_block_and_state(support_pos);
    if support_block == speleothem {
        let props = PointedDripstoneLikeProperties::from_state_id(support_state.id);
        return props.vertical_direction == flip_dir(support_dir);
    }
    let face = match support_dir {
        VerticalDirection::Up => BlockDirection::Down,
        VerticalDirection::Down => BlockDirection::Up,
    };
    support_state.is_side_solid(face)
}

fn modify_dripstone_thickness_to(
    world: &Arc<World>,
    speleothem: &Block,
    pos: &BlockPos,
    new_thickness: SpeleothemThickness,
) {
    let (block, support_block_state_id) = world.get_block_and_state_id(pos);

    if block != speleothem {
        return;
    }
    let mut support_props = PointedDripstoneLikeProperties::from_state_id(support_block_state_id);
    if support_props.thickness == new_thickness {
        return;
    }
    support_props.thickness = new_thickness;
    world.set_block_state(
        pos,
        support_props.to_state_id(speleothem),
        BlockFlags::empty(),
    );
}

fn offset_pos_by_vertical_dir(pos: &BlockPos, ver_dir: VerticalDirection) -> BlockPos {
    match ver_dir {
        VerticalDirection::Up => pos.down(),
        VerticalDirection::Down => pos.up(),
    }
}

const fn block_direction_to_vertical_direction(dir: BlockDirection) -> Option<VerticalDirection> {
    match dir {
        BlockDirection::Up => Some(VerticalDirection::Up),
        BlockDirection::Down => Some(VerticalDirection::Down),
        _ => None,
    }
}

fn flip_dir(dir: VerticalDirection) -> VerticalDirection {
    if dir == VerticalDirection::Up {
        return VerticalDirection::Down;
    }
    VerticalDirection::Up
}
