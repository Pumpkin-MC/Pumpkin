use std::sync::Arc;

use crate::{
    block::{
        BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnLandedUponArgs,
        OnPlaceArgs, PathComputationType, RandomTickArgs,
    },
    entity::{EntityBase, player::Player},
    world::World,
};
use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId,
    block_properties::{PointedDripstoneLikeProperties, SpeleothemThickness, VerticalDirection},
    fluid::Fluid,
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::{
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};
use rand::RngExt;

const GROWTH_PROBABILITY_PER_RANDOM_TICK: f32 = 0.011_377_778;
const MAX_GROWTH_LENGTH: i32 = 7;
// `SulfurSpikeBlock.MAX_GROWING_LENGTH`
const MAX_GROWING_LENGTH: i32 = 2;
const MAX_STALAGMITE_SEARCH_RANGE_WHEN_GROWING: i32 = 10;
const REQUIRED_SPACE_TO_DRIP_THROUGH_NON_SOLID_BLOCK: BoundingBox =
    BoundingBox::new_array([0.375, 0.0, 0.375], [0.625, 1.0, 0.625]);

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
        dripstone_props.thickness = calculate_speleothem_thickness(
            args.world,
            args.block,
            args.position,
            dripstone_props.vertical_direction,
            !args.player.get_entity().is_sneaking(),
        );
        dripstone_props.to_state_id(args.block)
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
        let Some(neighbor_dir) = block_direction_to_vertical_direction(args.direction) else {
            return args.state_id;
        };
        if neighbor_dir == flip_dir(dripstone_props.vertical_direction)
            && !can_place_at_pos(args.world, args.block, args.position, None, None)
        {
            return Block::AIR.default_state.id;
        }
        let merge_opposing_tips = dripstone_props.thickness == SpeleothemThickness::TipMerge;
        dripstone_props.thickness = calculate_speleothem_thickness(
            args.world,
            args.block,
            args.position,
            dripstone_props.vertical_direction,
            merge_opposing_tips,
        );
        dripstone_props.to_state_id(args.block)
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

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if rand::rng().random::<f32>() >= GROWTH_PROBABILITY_PER_RANDOM_TICK {
            return;
        }
        let state_id = args.world.get_block_state_id(args.position);
        if is_stalactite_start_pos(args.world, args.position, state_id) {
            grow_stalactite_or_stalagmite_if_possible(
                args.world,
                args.block,
                args.position,
                state_id,
            );
        }
    }
}

fn calculate_speleothem_thickness(
    block_accessor: &dyn BlockAccessor,
    speleothem: &Block,
    position: &BlockPos,
    tip_dir: VerticalDirection,
    merge_opposing_tips: bool,
) -> SpeleothemThickness {
    let base_dir = flip_dir(tip_dir);
    let (in_front_block, in_front_state) =
        block_accessor.get_block_and_state(&relative_vertical(position, tip_dir));
    if in_front_block == speleothem && is_speleothem_with_direction(in_front_state.id, base_dir) {
        let in_front_thickness =
            PointedDripstoneLikeProperties::from_state_id(in_front_state.id).thickness;
        return if !merge_opposing_tips && in_front_thickness != SpeleothemThickness::TipMerge {
            SpeleothemThickness::Tip
        } else {
            SpeleothemThickness::TipMerge
        };
    }
    // Unlike the opposing tip check above, vanilla accepts any speleothem from here on.
    if !is_speleothem_with_direction(in_front_state.id, tip_dir) {
        return SpeleothemThickness::Tip;
    }
    let in_front_thickness =
        PointedDripstoneLikeProperties::from_state_id(in_front_state.id).thickness;
    if matches!(
        in_front_thickness,
        SpeleothemThickness::Tip | SpeleothemThickness::TipMerge
    ) {
        return SpeleothemThickness::Frustum;
    }
    let behind_state_id = block_accessor.get_block_state_id(&relative_vertical(position, base_dir));
    if is_speleothem_with_direction(behind_state_id, tip_dir) {
        SpeleothemThickness::Middle
    } else {
        SpeleothemThickness::Base
    }
}

fn is_speleothem_with_direction(state_id: BlockStateId, tip_dir: VerticalDirection) -> bool {
    Block::from_state_id(state_id).has_tag(&tag::Block::MINECRAFT_SPELEOTHEMS)
        && PointedDripstoneLikeProperties::from_state_id(state_id).vertical_direction == tip_dir
}

fn is_tip(state_id: BlockStateId, include_merged_tip: bool) -> bool {
    if !Block::from_state_id(state_id).has_tag(&tag::Block::MINECRAFT_SPELEOTHEMS) {
        return false;
    }
    let thickness = PointedDripstoneLikeProperties::from_state_id(state_id).thickness;
    thickness == SpeleothemThickness::Tip
        || (include_merged_tip && thickness == SpeleothemThickness::TipMerge)
}

fn is_stalactite_start_pos(world: &World, position: &BlockPos, state_id: BlockStateId) -> bool {
    is_speleothem_with_direction(state_id, VerticalDirection::Down)
        && world.get_block(&position.up()) != Block::from_state_id(state_id)
}

fn is_free_hanging_stalactite(state_id: BlockStateId) -> bool {
    if !is_speleothem_with_direction(state_id, VerticalDirection::Down) {
        return false;
    }
    let props = PointedDripstoneLikeProperties::from_state_id(state_id);
    props.thickness == SpeleothemThickness::Tip && !props.waterlogged
}

fn is_unmerged_tip_with_direction(
    speleothem: &Block,
    state_id: BlockStateId,
    tip_dir: VerticalDirection,
) -> bool {
    is_tip(state_id, false)
        && PointedDripstoneLikeProperties::from_state_id(state_id).vertical_direction == tip_dir
        && Block::from_state_id(state_id) == speleothem
}

fn grow_stalactite_or_stalagmite_if_possible(
    world: &Arc<World>,
    speleothem: &Block,
    stalactite_start_pos: &BlockPos,
    stalactite_start_state_id: BlockStateId,
) {
    if !can_grow(world, speleothem, stalactite_start_pos) {
        return;
    }
    let Some(stalactite_tip_pos) = find_tip(
        world,
        stalactite_start_pos,
        stalactite_start_state_id,
        get_max_growth_length(speleothem),
        false,
    ) else {
        return;
    };
    let stalactite_tip_state_id = world.get_block_state_id(&stalactite_tip_pos);
    if !is_free_hanging_stalactite(stalactite_tip_state_id)
        || !can_tip_grow(
            world,
            speleothem,
            stalactite_tip_state_id,
            &stalactite_tip_pos,
        )
    {
        return;
    }
    if rand::rng().random::<bool>() {
        grow(
            world,
            speleothem,
            &stalactite_tip_pos,
            VerticalDirection::Down,
        );
    } else {
        grow_stalagmite_below(world, speleothem, &stalactite_tip_pos);
    }
}

// Mirrors the `blockToGrowOn` constructor arguments in `Blocks`.
fn get_block_to_grow_on(speleothem: &Block) -> &'static Block {
    if speleothem == &Block::SULFUR_SPIKE {
        &Block::SULFUR
    } else {
        &Block::DRIPSTONE_BLOCK
    }
}

fn get_max_growth_length(speleothem: &Block) -> i32 {
    if speleothem == &Block::SULFUR_SPIKE {
        MAX_GROWING_LENGTH
    } else {
        MAX_GROWTH_LENGTH
    }
}

fn can_grow(world: &World, speleothem: &Block, position: &BlockPos) -> bool {
    if world.get_block(&position.up()) != get_block_to_grow_on(speleothem) {
        return false;
    }
    if speleothem != &Block::POINTED_DRIPSTONE {
        return true;
    }
    let (fluid, fluid_state) = world.get_fluid_and_fluid_state(&position.up_height(2));
    fluid.matches_type(&Fluid::WATER) && fluid_state.is_source
}

fn find_tip(
    world: &World,
    speleothem_pos: &BlockPos,
    speleothem_state_id: BlockStateId,
    max_search_length: i32,
    include_merged_tip: bool,
) -> Option<BlockPos> {
    if is_tip(speleothem_state_id, include_merged_tip) {
        return Some(*speleothem_pos);
    }
    let speleothem = Block::from_state_id(speleothem_state_id);
    let search_dir =
        PointedDripstoneLikeProperties::from_state_id(speleothem_state_id).vertical_direction;
    find_block_vertical(
        world,
        speleothem_pos,
        search_dir,
        |state_id| {
            Block::from_state_id(state_id) == speleothem
                && PointedDripstoneLikeProperties::from_state_id(state_id).vertical_direction
                    == search_dir
        },
        |state_id| is_tip(state_id, include_merged_tip),
        max_search_length,
    )
}

fn find_block_vertical(
    world: &World,
    position: &BlockPos,
    direction: VerticalDirection,
    path_predicate: impl Fn(BlockStateId) -> bool,
    target_predicate: impl Fn(BlockStateId) -> bool,
    max_steps: i32,
) -> Option<BlockPos> {
    let mut pos = *position;
    for _ in 1..max_steps {
        pos = relative_vertical(&pos, direction);
        let state_id = world.get_block_state_id(&pos);
        if target_predicate(state_id) {
            return Some(pos);
        }
        if !world.is_in_height_limit(pos.0.y) || !path_predicate(state_id) {
            return None;
        }
    }
    None
}

fn can_tip_grow(
    world: &World,
    speleothem: &Block,
    tip_state_id: BlockStateId,
    tip_pos: &BlockPos,
) -> bool {
    let grow_dir = PointedDripstoneLikeProperties::from_state_id(tip_state_id).vertical_direction;
    let state_at_grow_pos = world.get_block_state(&relative_vertical(tip_pos, grow_dir));
    if *World::get_fluid_from_state_id(state_at_grow_pos.id) != Fluid::EMPTY {
        return false;
    }
    state_at_grow_pos.is_air()
        || is_unmerged_tip_with_direction(speleothem, state_at_grow_pos.id, flip_dir(grow_dir))
}

fn grow(
    world: &Arc<World>,
    speleothem: &Block,
    grow_from_pos: &BlockPos,
    grow_to_dir: VerticalDirection,
) {
    let target_pos = relative_vertical(grow_from_pos, grow_to_dir);
    let (existing_block, existing_state) = world.get_block_and_state(&target_pos);
    if is_unmerged_tip_with_direction(speleothem, existing_state.id, flip_dir(grow_to_dir)) {
        create_merged_tips(world, speleothem, existing_state.id, &target_pos);
    } else if existing_state.is_air() || existing_block == &Block::WATER {
        create_speleothem(
            world,
            speleothem,
            &target_pos,
            grow_to_dir,
            SpeleothemThickness::Tip,
        );
    }
}

fn create_speleothem(
    world: &Arc<World>,
    speleothem: &Block,
    position: &BlockPos,
    tip_dir: VerticalDirection,
    thickness: SpeleothemThickness,
) {
    let (fluid, fluid_state) = world.get_fluid_and_fluid_state(position);
    let props = PointedDripstoneLikeProperties {
        thickness,
        vertical_direction: tip_dir,
        waterlogged: fluid.matches_type(&Fluid::WATER) && fluid_state.is_source,
    };
    world.set_block_state(
        position,
        props.to_state_id(speleothem),
        BlockFlags::NOTIFY_ALL,
    );
}

fn create_merged_tips(
    world: &Arc<World>,
    speleothem: &Block,
    tip_state_id: BlockStateId,
    tip_pos: &BlockPos,
) {
    let (stalactite_pos, stalagmite_pos) =
        match PointedDripstoneLikeProperties::from_state_id(tip_state_id).vertical_direction {
            VerticalDirection::Up => (tip_pos.up(), *tip_pos),
            VerticalDirection::Down => (*tip_pos, tip_pos.down()),
        };
    create_speleothem(
        world,
        speleothem,
        &stalactite_pos,
        VerticalDirection::Down,
        SpeleothemThickness::TipMerge,
    );
    create_speleothem(
        world,
        speleothem,
        &stalagmite_pos,
        VerticalDirection::Up,
        SpeleothemThickness::TipMerge,
    );
}

fn grow_stalagmite_below(world: &Arc<World>, speleothem: &Block, pos_above_stalagmite: &BlockPos) {
    let mut pos = *pos_above_stalagmite;
    for _ in 0..MAX_STALAGMITE_SEARCH_RANGE_WHEN_GROWING {
        pos = pos.down();
        let state_id = world.get_block_state_id(&pos);
        if *World::get_fluid_from_state_id(state_id) != Fluid::EMPTY {
            return;
        }
        if is_unmerged_tip_with_direction(speleothem, state_id, VerticalDirection::Up)
            && can_tip_grow(world, speleothem, state_id, &pos)
        {
            grow(world, speleothem, &pos, VerticalDirection::Up);
            return;
        }
        if can_support_dripstone(
            world.as_ref(),
            speleothem,
            &pos.down(),
            VerticalDirection::Down,
        ) && !world.get_fluid(&pos.down()).matches_type(&Fluid::WATER)
        {
            grow(world, speleothem, &pos.down(), VerticalDirection::Up);
            return;
        }
        if blocks_stalagmite_scan(speleothem, &pos, state_id) {
            return;
        }
    }
}

fn blocks_stalagmite_scan(speleothem: &Block, position: &BlockPos, state_id: BlockStateId) -> bool {
    speleothem == &Block::POINTED_DRIPSTONE && !can_drip_through(position, state_id)
}

fn can_drip_through(position: &BlockPos, state_id: BlockStateId) -> bool {
    let state = BlockState::from_id(state_id);
    if state.is_air() {
        return true;
    }
    if state.is_solid_render() || *World::get_fluid_from_state_id(state_id) != Fluid::EMPTY {
        return false;
    }
    !state
        .get_block_collision_shapes_at(position)
        .any(|shape| shape.intersects(&REQUIRED_SPACE_TO_DRIP_THROUGH_NON_SOLID_BLOCK))
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

fn relative_vertical(pos: &BlockPos, dir: VerticalDirection) -> BlockPos {
    match dir {
        VerticalDirection::Up => pos.up(),
        VerticalDirection::Down => pos.down(),
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
