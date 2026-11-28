use std::sync::Arc;

use crate::{
    block::{
        BlockBehaviour, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
        OnLandedUponArgs, OnPlaceArgs, OnProjectileHitArgs, PathComputationType, PlacedArgs,
        RandomTickArgs,
    },
    entity::{
        player::Player,
        projectile::{may_break, may_interact},
    },
    world::World,
};
use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockState, BlockStateId,
    block_properties::{PointedDripstoneLikeProperties, SpeleothemThickness, VerticalDirection},
    entity::EntityType,
    fluid::Fluid,
    game_event::GameEvent,
    tag::{self, Taggable},
    world::WorldEvent,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::{
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};
use rand::RngExt;

const MAX_SEARCH_LENGTH_WHEN_CHECKING_DRIP_TYPE: i32 = 11;
const MAX_SEARCH_LENGTH_BETWEEN_STALACTITE_TIP_AND_CAULDRON: i32 = 11;
const WATER_TRANSFER_PROBABILITY_PER_RANDOM_TICK: f32 = 0.175_781_25;
const LAVA_TRANSFER_PROBABILITY_PER_RANDOM_TICK: f32 = 0.058_593_75;
const GROWTH_PROBABILITY_PER_RANDOM_TICK: f32 = 0.011_377_778;
const MAX_STALAGMITE_SEARCH_RANGE_WHEN_GROWING: i32 = 10;
const POINTED_DRIPSTONE_MAX_GROWTH_LENGTH: i32 = 7;
const SULFUR_SPIKE_MAX_GROWTH_LENGTH: i32 = 2;
const REQUIRED_SPACE_TO_DRIP_THROUGH_NON_SOLID_BLOCK: BoundingBox =
    BoundingBox::new_array([0.375, 0.0, 0.375], [0.625, 1.0, 0.625]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CauldronDripFluid {
    Water,
    Lava,
}

// TODO: randomTick, maybeTransferFluid (mud to clay, cauldron fill) and growth
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
                pumpkin_data::fluid::Fluid::WATER.flow.tick_delay,
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
                args.entity
                    .cause_fall_damage(args.entity, args.fall_distance + 2.5, 2.0);
                return;
            }
        }
        args.entity
            .cause_fall_damage(args.entity, args.fall_distance, 1.0);
    }

    fn on_projectile_hit(&self, args: OnProjectileHitArgs<'_>) {
        const TRIDENT_BREAK_MIN_SPEED: f64 = 0.6;

        let projectile_entity = args.projectile.get_entity();
        if args.block == &Block::POINTED_DRIPSTONE
            && projectile_entity.entity_type == &EntityType::TRIDENT
            && projectile_entity.velocity.load().length() > TRIDENT_BREAK_MIN_SPEED
            && may_interact(args.projectile, args.world, args.position)
            && may_break(args.projectile, args.world)
        {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let mut rng = rand::rng();
        if args.block == &Block::POINTED_DRIPSTONE {
            maybe_transfer_fluid(args.world, args.position, state_id, rng.random::<f32>());
        }
        if rng.random::<f32>() < GROWTH_PROBABILITY_PER_RANDOM_TICK
            && is_stalactite_start_pos(args.world, args.block, args.position, state_id)
        {
            grow_stalactite_or_stalagmite_if_possible(
                args.world,
                args.block,
                args.position,
                state_id,
                rng.random::<bool>(),
            );
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

#[derive(Clone, Copy)]
struct FluidAboveStalactite {
    pos: BlockPos,
    fluid: CauldronDripFluid,
    from_mud: bool,
}

pub(crate) fn cauldron_drip_fluid(
    world: &World,
    cauldron_pos: &BlockPos,
) -> Option<CauldronDripFluid> {
    let tip_pos = find_stalactite_tip_above_cauldron(world, cauldron_pos)?;
    let (tip_block, tip_state_id) = world.get_block_and_state_id(&tip_pos);
    get_fluid_above_stalactite(world, tip_block, &tip_pos, tip_state_id).map(|info| info.fluid)
}

pub(crate) fn maybe_transfer_fluid(
    world: &Arc<World>,
    pos: &BlockPos,
    state_id: BlockStateId,
    random_value: f32,
) {
    if random_value >= WATER_TRANSFER_PROBABILITY_PER_RANDOM_TICK {
        return;
    }
    if !is_stalactite_start_pos(world, &Block::POINTED_DRIPSTONE, pos, state_id) {
        return;
    }
    let Some(fluid_info) =
        get_fluid_above_stalactite(world, &Block::POINTED_DRIPSTONE, pos, state_id)
    else {
        return;
    };
    let transfer_probability = match fluid_info.fluid {
        CauldronDripFluid::Water => WATER_TRANSFER_PROBABILITY_PER_RANDOM_TICK,
        CauldronDripFluid::Lava => LAVA_TRANSFER_PROBABILITY_PER_RANDOM_TICK,
    };
    if random_value >= transfer_probability {
        return;
    }
    let Some(tip_pos) = find_tip(
        world,
        &Block::POINTED_DRIPSTONE,
        pos,
        state_id,
        MAX_SEARCH_LENGTH_WHEN_CHECKING_DRIP_TYPE,
        false,
    ) else {
        return;
    };

    if fluid_info.from_mud && fluid_info.fluid == CauldronDripFluid::Water {
        world.set_block_state(
            &fluid_info.pos,
            Block::CLAY.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        world.emit_game_event(
            GameEvent::BlockChange.name(),
            fluid_info.pos.to_centered_f64(),
        );
        world.sync_world_event(WorldEvent::DripstoneDrip, tip_pos, 0);
        return;
    }

    if let Some((cauldron_pos, cauldron_block)) =
        find_fillable_cauldron_below_stalactite_tip(world, &tip_pos, fluid_info.fluid)
    {
        world.sync_world_event(WorldEvent::DripstoneDrip, tip_pos, 0);
        let fall_distance = (tip_pos.0.y - cauldron_pos.0.y).max(0);
        let delay = (50 + fall_distance).min(i32::from(u8::MAX)) as u8;
        world.schedule_block_tick(cauldron_block, cauldron_pos, delay, TickPriority::Normal);
    }
}

fn grow_stalactite_or_stalagmite_if_possible(
    world: &Arc<World>,
    speleothem: &Block,
    stalactite_start_pos: &BlockPos,
    stalactite_start_state_id: BlockStateId,
    grow_downward: bool,
) {
    if !can_speleothem_grow(world, speleothem, stalactite_start_pos) {
        return;
    }
    let max_growth_length = if speleothem == &Block::SULFUR_SPIKE {
        SULFUR_SPIKE_MAX_GROWTH_LENGTH
    } else {
        POINTED_DRIPSTONE_MAX_GROWTH_LENGTH
    };
    let Some(tip_pos) = find_tip(
        world,
        speleothem,
        stalactite_start_pos,
        stalactite_start_state_id,
        max_growth_length,
        false,
    ) else {
        return;
    };
    let (tip_block, tip_state_id) = world.get_block_and_state_id(&tip_pos);
    if !is_free_hanging_stalactite(tip_block, tip_state_id)
        || !can_tip_grow(world, speleothem, &tip_pos, VerticalDirection::Down)
    {
        return;
    }

    if grow_downward {
        grow_speleothem(world, speleothem, &tip_pos, VerticalDirection::Down);
    } else {
        grow_stalagmite_below(world, speleothem, &tip_pos);
    }
}

fn is_still_fluid_source(world: &World, pos: &BlockPos, expected: &Fluid) -> bool {
    let (fluid, fluid_state) = world.get_fluid_and_fluid_state(pos);
    fluid_state.is_source && fluid.matches_type(expected)
}

fn can_speleothem_grow(world: &World, speleothem: &Block, start_pos: &BlockPos) -> bool {
    if speleothem == &Block::SULFUR_SPIKE {
        return world.get_block(&start_pos.up()) == &Block::SULFUR;
    }
    if world.get_block(&start_pos.up()) != &Block::DRIPSTONE_BLOCK {
        return false;
    }
    let above_two = start_pos.up_height(2);
    is_still_fluid_source(world, &above_two, &Fluid::WATER)
}

fn can_tip_grow(
    world: &World,
    speleothem: &Block,
    tip_pos: &BlockPos,
    grow_dir: VerticalDirection,
) -> bool {
    let grow_pos = step_vertical(tip_pos, grow_dir);
    if world.get_fluid(&grow_pos).id != Fluid::EMPTY.id {
        return false;
    }
    let (block, state) = world.get_block_and_state(&grow_pos);
    state.is_air()
        || is_unmerged_tip_with_direction(block, state.id, speleothem, flip_dir(grow_dir))
}

fn grow_speleothem(
    world: &Arc<World>,
    speleothem: &Block,
    grow_from_pos: &BlockPos,
    grow_to_dir: VerticalDirection,
) {
    let target_pos = step_vertical(grow_from_pos, grow_to_dir);
    let (existing_block, existing_state) = world.get_block_and_state(&target_pos);
    if is_unmerged_tip_with_direction(
        existing_block,
        existing_state.id,
        speleothem,
        flip_dir(grow_to_dir),
    ) {
        let (stalactite_pos, stalagmite_pos) = if grow_to_dir == VerticalDirection::Down {
            (target_pos.up(), target_pos)
        } else {
            (target_pos, target_pos.down())
        };
        place_grown_speleothem(
            world,
            speleothem,
            &stalactite_pos,
            VerticalDirection::Down,
            SpeleothemThickness::TipMerge,
        );
        place_grown_speleothem(
            world,
            speleothem,
            &stalagmite_pos,
            VerticalDirection::Up,
            SpeleothemThickness::TipMerge,
        );
    } else if existing_state.is_air() || existing_block == &Block::WATER {
        place_grown_speleothem(
            world,
            speleothem,
            &target_pos,
            grow_to_dir,
            SpeleothemThickness::Tip,
        );
    }
}

fn place_grown_speleothem(
    world: &Arc<World>,
    speleothem: &Block,
    pos: &BlockPos,
    direction: VerticalDirection,
    thickness: SpeleothemThickness,
) {
    let mut props = PointedDripstoneLikeProperties::default(speleothem);
    props.vertical_direction = direction;
    props.thickness = thickness;
    props.waterlogged = is_still_fluid_source(world, pos, &Fluid::WATER);
    let state_id = props.to_state_id(speleothem);
    world.set_block_state(pos, state_id, BlockFlags::NOTIFY_ALL);
    let (len, ver_dir) =
        get_stalagmite_or_stalactite_len_and_dir_from_tip_pos(world, speleothem, pos, state_id);
    match ver_dir {
        VerticalDirection::Up => update_stalagmite(world, speleothem, len, pos),
        VerticalDirection::Down => update_stalactite(world, speleothem, len, pos),
    }
}

fn grow_stalagmite_below(world: &Arc<World>, speleothem: &Block, pos_above_stalagmite: &BlockPos) {
    let mut scan_pos = *pos_above_stalagmite;
    for _ in 0..MAX_STALAGMITE_SEARCH_RANGE_WHEN_GROWING {
        scan_pos = scan_pos.down();
        if world.get_fluid(&scan_pos).id != Fluid::EMPTY.id {
            return;
        }
        let (block, state) = world.get_block_and_state(&scan_pos);
        if is_unmerged_tip_with_direction(block, state.id, speleothem, VerticalDirection::Up)
            && can_tip_grow(world, speleothem, &scan_pos, VerticalDirection::Up)
        {
            grow_speleothem(world, speleothem, &scan_pos, VerticalDirection::Up);
            return;
        }
        let below_scan = scan_pos.down();
        if can_support_dripstone(
            world.as_ref(),
            speleothem,
            &below_scan,
            VerticalDirection::Down,
        ) && !world
            .get_fluid(&below_scan)
            .has_tag(&tag::Fluid::MINECRAFT_WATER)
        {
            grow_speleothem(world, speleothem, &below_scan, VerticalDirection::Up);
            return;
        }
        if speleothem == &Block::POINTED_DRIPSTONE && !can_drip_through(world, &scan_pos, state) {
            return;
        }
    }
}

fn get_fluid_above_stalactite(
    world: &World,
    speleothem: &Block,
    stalactite_pos: &BlockPos,
    stalactite_state_id: BlockStateId,
) -> Option<FluidAboveStalactite> {
    if !is_stalactite(speleothem, stalactite_state_id) {
        return None;
    }
    let root_pos = find_root_block(
        world,
        speleothem,
        stalactite_pos,
        stalactite_state_id,
        MAX_SEARCH_LENGTH_WHEN_CHECKING_DRIP_TYPE,
    )?;
    let above_pos = root_pos.up();
    let above_block = world.get_block(&above_pos);
    if above_block == &Block::MUD && !world.dimension.water_evaporates {
        return Some(FluidAboveStalactite {
            pos: above_pos,
            fluid: CauldronDripFluid::Water,
            from_mud: true,
        });
    }
    let (fluid, fluid_state) = world.get_fluid_and_fluid_state(&above_pos);
    let drip_fluid = if fluid_state.is_source && fluid.matches_type(&Fluid::WATER) {
        CauldronDripFluid::Water
    } else if fluid_state.is_source && fluid.matches_type(&Fluid::LAVA) {
        CauldronDripFluid::Lava
    } else {
        return None;
    };
    Some(FluidAboveStalactite {
        pos: above_pos,
        fluid: drip_fluid,
        from_mud: false,
    })
}

fn find_root_block(
    world: &World,
    speleothem: &Block,
    pos: &BlockPos,
    state_id: BlockStateId,
    max_steps: i32,
) -> Option<BlockPos> {
    let tip_dir = PointedDripstoneLikeProperties::from_state_id(state_id).vertical_direction;
    let step_dir = flip_dir(tip_dir);
    let mut current = *pos;
    for _ in 1..max_steps {
        current = step_vertical(&current, step_dir);
        if is_outside_build_height(world, current.0.y) {
            return None;
        }
        let (block, cur_state_id) = world.get_block_and_state_id(&current);
        if block != speleothem {
            return Some(current);
        }
        if PointedDripstoneLikeProperties::from_state_id(cur_state_id).vertical_direction != tip_dir
        {
            return None;
        }
    }
    None
}

fn find_tip(
    world: &World,
    speleothem: &Block,
    pos: &BlockPos,
    state_id: BlockStateId,
    max_steps: i32,
    include_merged: bool,
) -> Option<BlockPos> {
    if is_tip(speleothem, state_id, include_merged) {
        return Some(*pos);
    }
    let search_dir = PointedDripstoneLikeProperties::from_state_id(state_id).vertical_direction;
    let mut current = *pos;
    for _ in 1..max_steps {
        current = step_vertical(&current, search_dir);
        if is_outside_build_height(world, current.0.y) {
            return None;
        }
        let (block, cur_state_id) = world.get_block_and_state_id(&current);
        if is_tip(block, cur_state_id, include_merged) && block == speleothem {
            let props = PointedDripstoneLikeProperties::from_state_id(cur_state_id);
            if props.vertical_direction == search_dir {
                return Some(current);
            }
        }
        if block != speleothem
            || PointedDripstoneLikeProperties::from_state_id(cur_state_id).vertical_direction
                != search_dir
        {
            return None;
        }
    }
    None
}

fn find_fillable_cauldron_below_stalactite_tip(
    world: &World,
    stalactite_tip_pos: &BlockPos,
    fluid: CauldronDripFluid,
) -> Option<(BlockPos, &'static Block)> {
    let mut current = *stalactite_tip_pos;
    for _ in 1..MAX_SEARCH_LENGTH_BETWEEN_STALACTITE_TIP_AND_CAULDRON {
        current = current.down();
        if is_outside_build_height(world, current.0.y) {
            return None;
        }
        let (block, state) = world.get_block_and_state(&current);
        if can_cauldron_receive_drip(block.id, state.id, fluid) {
            return Some((current, block));
        }
        if !can_drip_through(world, &current, state) {
            return None;
        }
    }
    None
}

fn find_stalactite_tip_above_cauldron(world: &World, cauldron_pos: &BlockPos) -> Option<BlockPos> {
    let mut current = *cauldron_pos;
    for _ in 1..MAX_SEARCH_LENGTH_BETWEEN_STALACTITE_TIP_AND_CAULDRON {
        current = current.up();
        if is_outside_build_height(world, current.0.y) {
            return None;
        }
        let (block, state) = world.get_block_and_state(&current);
        if is_free_hanging_stalactite(block, state.id) {
            return Some(current);
        }
        if !can_drip_through(world, &current, state) {
            return None;
        }
    }
    None
}

fn can_cauldron_receive_drip(
    block_id: BlockId,
    _state_id: BlockStateId,
    fluid: CauldronDripFluid,
) -> bool {
    match block_id {
        BlockId::CAULDRON => true,
        // Vanilla `LayeredCauldronBlock.canReceiveStalactiteDrip`: water cauldron accepts water
        // regardless of fullness when scheduling; `receiveStalactiteDrip` checks `!isFull`.
        BlockId::WATER_CAULDRON => fluid == CauldronDripFluid::Water,
        _ => false,
    }
}

fn can_drip_through(world: &World, pos: &BlockPos, state: &BlockState) -> bool {
    if state.is_air() {
        return true;
    }
    if state.is_solid_render() || world.get_fluid(pos).id != Fluid::EMPTY.id {
        return false;
    }
    !state
        .get_block_collision_shapes()
        .any(|shape| shape.intersects(&REQUIRED_SPACE_TO_DRIP_THROUGH_NON_SOLID_BLOCK))
}

fn is_stalactite_start_pos(
    world: &World,
    speleothem: &Block,
    pos: &BlockPos,
    state_id: BlockStateId,
) -> bool {
    is_stalactite(speleothem, state_id) && world.get_block(&pos.up()) != speleothem
}

fn is_stalactite(block: &Block, state_id: BlockStateId) -> bool {
    block.has_tag(&tag::Block::MINECRAFT_SPELEOTHEMS)
        && PointedDripstoneLikeProperties::from_state_id(state_id).vertical_direction
            == VerticalDirection::Down
}

fn is_free_hanging_stalactite(block: &Block, state_id: BlockStateId) -> bool {
    if !block.has_tag(&tag::Block::MINECRAFT_SPELEOTHEMS) {
        return false;
    }
    let props = PointedDripstoneLikeProperties::from_state_id(state_id);
    props.vertical_direction == VerticalDirection::Down
        && props.thickness == SpeleothemThickness::Tip
        && !props.waterlogged
}

fn is_tip(block: &Block, state_id: BlockStateId, include_merged: bool) -> bool {
    if !block.has_tag(&tag::Block::MINECRAFT_SPELEOTHEMS) {
        return false;
    }
    let thickness = PointedDripstoneLikeProperties::from_state_id(state_id).thickness;
    thickness == SpeleothemThickness::Tip
        || (include_merged && thickness == SpeleothemThickness::TipMerge)
}

fn is_unmerged_tip_with_direction(
    block: &Block,
    state_id: BlockStateId,
    expected_block: &Block,
    tip_dir: VerticalDirection,
) -> bool {
    block == expected_block
        && is_tip(block, state_id, false)
        && PointedDripstoneLikeProperties::from_state_id(state_id).vertical_direction == tip_dir
}

fn step_vertical(pos: &BlockPos, dir: VerticalDirection) -> BlockPos {
    match dir {
        VerticalDirection::Up => pos.up(),
        VerticalDirection::Down => pos.down(),
    }
}

const fn is_outside_build_height(world: &World, y: i32) -> bool {
    y < world.dimension.min_y || y >= world.dimension.min_y + world.dimension.height
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
