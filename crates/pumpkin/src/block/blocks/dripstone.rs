use std::sync::Arc;

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
        OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
    },
    entity::{falling::FallingEntity, player::Player},
    world::World,
};
use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    block_properties::{
        BlockProperties, PointedDripstoneLikeProperties, SpeleothemThickness, VerticalDirection,
    },
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

/// Ticks between a stalactite losing its support and starting to fall. This matches the delay
/// this repository already uses for every other falling block (`FallingBlock` and
/// `ConcretePowderBlock` both schedule 2 ticks); vanilla's own constant could not be verified
/// from an available source, and the wiki does not state one.
const DELAY_BEFORE_FALLING: u8 = 2;

/// Cap on the damage a falling stalactite can deal. minecraft.wiki, "Pointed Dripstone":
/// the damage is capped no matter how far the stalactite falls.
const STALACTITE_MAX_DAMAGE: i32 = 40;

/// Damage per block of fall distance for a falling stalactite.
///
/// minecraft.wiki, "Pointed Dripstone" (Java Edition): "the amount of damage is 1HP per pointed
/// dripstone falling (less than 6 will be counted as 6) per each block of falling distance".
#[must_use]
pub fn stalactite_damage_per_distance(block_count: usize) -> f32 {
    block_count.max(6) as f32
}

/// A downward-pointing pointed dripstone, i.e. part of a stalactite.
#[must_use]
pub fn is_stalactite(block: &Block, state_id: BlockStateId) -> bool {
    block == &Block::POINTED_DRIPSTONE
        && PointedDripstoneLikeProperties::from_state_id(state_id, block).vertical_direction
            == VerticalDirection::Down
}

/// The lowest block of a stalactite, including the merged-tip form.
const fn is_tip_thickness(thickness: SpeleothemThickness) -> bool {
    matches!(
        thickness,
        SpeleothemThickness::Tip | SpeleothemThickness::TipMerge
    )
}

/// Turns the unsupported stalactite starting at `position` into falling block entities.
///
/// minecraft.wiki, "Pointed Dripstone": "If the block supporting a stalactite or any block of
/// the stalactite is broken, all of the unsupported pointed dripstone below the broken block
/// drops, causing damage to any player and mobs standing beneath it, similar to a falling
/// anvil." Only the tip segment hurts entities; the damage scales with the number of blocks
/// in the falling stalactite (see [`stalactite_damage_per_distance`]).
async fn spawn_falling_stalactite(world: &Arc<World>, position: &BlockPos) {
    let mut segments = Vec::new();
    let mut current = *position;
    let mut has_tip = false;
    loop {
        let (block, state) = world.get_block_and_state(&current);
        if !is_stalactite(block, state.id) {
            break;
        }
        segments.push((current, state.id));
        let props = PointedDripstoneLikeProperties::from_state_id(state.id, block);
        if is_tip_thickness(props.thickness) {
            has_tip = true;
            break;
        }
        current = current.down();
    }

    let last_index = segments.len().saturating_sub(1);
    let damage_per_distance = stalactite_damage_per_distance(segments.len());
    for (index, (pos, state_id)) in segments.into_iter().enumerate() {
        let hurts = (has_tip && index == last_index)
            .then_some((damage_per_distance, STALACTITE_MAX_DAMAGE));
        FallingEntity::replace_spawn_hurting(world, pos, state_id, hurts).await;
    }
}

#[pumpkin_block("minecraft:pointed_dripstone")]
pub struct DripstoneBlock;

impl BlockBehaviour for DripstoneBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at_pos(
            args.block_accessor,
            args.position,
            args.direction,
            args.player,
        )
    }
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut dripstone_props = PointedDripstoneLikeProperties::default(args.block);
            dripstone_props.waterlogged = args.replacing.water_source();
            let Some(support_block_ver_dir) = get_support_block_vertical_direction(
                args.world,
                args.position,
                Some(args.direction),
                Some(args.player),
            ) else {
                //this shouldn't happen
                return Block::AIR.default_state.id;
            };

            dripstone_props.vertical_direction = flip_dir(support_block_ver_dir);
            dripstone_props.to_state_id(&Block::POINTED_DRIPSTONE)
        })
    }
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let (len, vertical_dir) = get_stalagmite_or_stalactice_len_and_dir_from_tip_pos(
                args.world,
                args.position,
                args.state_id,
            );
            match vertical_dir {
                VerticalDirection::Up => {
                    update_stalagmite(args.world, len, args.position).await;
                }
                VerticalDirection::Down => {
                    update_stalactite(args.world, len, args.position).await;
                }
            }
        })
    }
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let broken_dripstone_props =
                PointedDripstoneLikeProperties::from_state_id(args.state.id, args.block);
            let new_tip_pos = match broken_dripstone_props.vertical_direction {
                VerticalDirection::Up => args.position.down(),
                VerticalDirection::Down => args.position.up(),
            };

            let (len, vertical_dir) = get_stalagmite_or_stalactice_len_and_dir_from_tip_pos(
                args.world,
                &new_tip_pos,
                args.state.id,
            );
            match vertical_dir {
                VerticalDirection::Up => {
                    update_stalagmite(args.world, len, &new_tip_pos).await;
                }
                VerticalDirection::Down => {
                    update_stalactite(args.world, len, &new_tip_pos).await;
                }
            }
        })
    }
    /// Fires `DELAY_BEFORE_FALLING` ticks after a stalactite lost its support. The guard is
    /// required: turning each segment into a falling entity replaces it with air and notifies
    /// its neighbors, which schedules further ticks for segments this call already consumed.
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let (block, state) = args.world.get_block_and_state(args.position);
            if !is_stalactite(block, state.id)
                || can_place_at_pos(&**args.world, args.position, None, None)
            {
                return;
            }
            spawn_falling_stalactite(args.world, args.position).await;
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !can_place_at_pos(args.world, args.position, None, None) {
                // An unsupported stalactite does not vanish: it falls (see
                // `spawn_falling_stalactite`). Stalagmites keep breaking immediately.
                if is_stalactite(args.block, args.state_id) {
                    args.world.schedule_block_tick(
                        args.block,
                        *args.position,
                        DELAY_BEFORE_FALLING,
                        TickPriority::Normal,
                    );
                    return args.state_id;
                }
                return Block::AIR.default_state.id;
            }
            let mut dripstone_props =
                PointedDripstoneLikeProperties::from_state_id(args.state_id, args.block);
            if dripstone_props.thickness != SpeleothemThickness::TipMerge {
                return args.state_id;
            }
            match dripstone_props.vertical_direction {
                VerticalDirection::Up => {
                    let block_above = args.world.get_block(&args.position.up());
                    if block_above != &Block::POINTED_DRIPSTONE {
                        dripstone_props.thickness = SpeleothemThickness::Tip;
                        return dripstone_props.to_state_id(args.block);
                    }
                }
                VerticalDirection::Down => {
                    let block_below = args.world.get_block(&args.position.down());
                    if block_below != &Block::POINTED_DRIPSTONE {
                        dripstone_props.thickness = SpeleothemThickness::Tip;
                        return dripstone_props.to_state_id(args.block);
                    }
                }
            }
            args.state_id
        })
    }
}
async fn update_stalagmite(world: &Arc<World>, stalagmite_len: u8, tip_pos: &BlockPos) {
    let block_above = world.get_block(&tip_pos.up());
    if block_above == &Block::POINTED_DRIPSTONE {
        modify_dripstone_thickness_to(world, tip_pos, SpeleothemThickness::TipMerge).await;
        modify_dripstone_thickness_to(world, &tip_pos.up(), SpeleothemThickness::TipMerge).await;
    } else {
        modify_dripstone_thickness_to(world, tip_pos, SpeleothemThickness::Tip).await;
    }
    match stalagmite_len {
        2 => {
            modify_dripstone_thickness_to(
                world,
                &tip_pos.down_height(1),
                SpeleothemThickness::Frustum,
            )
            .await;
        }
        3 => {
            modify_dripstone_thickness_to(
                world,
                &tip_pos.down_height(1),
                SpeleothemThickness::Frustum,
            )
            .await;
            modify_dripstone_thickness_to(
                world,
                &tip_pos.down_height(2),
                SpeleothemThickness::Base,
            )
            .await;
        }
        4 => {
            modify_dripstone_thickness_to(
                world,
                &tip_pos.down_height(1),
                SpeleothemThickness::Frustum,
            )
            .await;
            modify_dripstone_thickness_to(
                world,
                &tip_pos.down_height(2),
                SpeleothemThickness::Middle,
            )
            .await;
            modify_dripstone_thickness_to(
                world,
                &tip_pos.down_height(3),
                SpeleothemThickness::Base,
            )
            .await;
        }
        5 => {
            modify_dripstone_thickness_to(
                world,
                &tip_pos.down_height(1),
                SpeleothemThickness::Frustum,
            )
            .await;
            modify_dripstone_thickness_to(
                world,
                &tip_pos.down_height(2),
                SpeleothemThickness::Middle,
            )
            .await;
            modify_dripstone_thickness_to(
                world,
                &tip_pos.down_height(3),
                SpeleothemThickness::Middle,
            )
            .await;
        }
        _ => {}
    }
}

async fn update_stalactite(world: &Arc<World>, stalagmite_len: u8, tip_pos: &BlockPos) {
    let block_below = world.get_block(&tip_pos.down());
    if block_below == &Block::POINTED_DRIPSTONE {
        modify_dripstone_thickness_to(world, tip_pos, SpeleothemThickness::TipMerge).await;
        modify_dripstone_thickness_to(world, &tip_pos.down(), SpeleothemThickness::TipMerge).await;
    } else {
        modify_dripstone_thickness_to(world, tip_pos, SpeleothemThickness::Tip).await;
    }
    match stalagmite_len {
        2 => {
            modify_dripstone_thickness_to(
                world,
                &tip_pos.up_height(1),
                SpeleothemThickness::Frustum,
            )
            .await;
        }
        3 => {
            modify_dripstone_thickness_to(
                world,
                &tip_pos.up_height(1),
                SpeleothemThickness::Frustum,
            )
            .await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(2), SpeleothemThickness::Base)
                .await;
        }
        4 => {
            modify_dripstone_thickness_to(
                world,
                &tip_pos.up_height(1),
                SpeleothemThickness::Frustum,
            )
            .await;
            modify_dripstone_thickness_to(
                world,
                &tip_pos.up_height(2),
                SpeleothemThickness::Middle,
            )
            .await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(3), SpeleothemThickness::Base)
                .await;
        }
        5 => {
            modify_dripstone_thickness_to(
                world,
                &tip_pos.up_height(1),
                SpeleothemThickness::Frustum,
            )
            .await;
            modify_dripstone_thickness_to(
                world,
                &tip_pos.up_height(2),
                SpeleothemThickness::Middle,
            )
            .await;
            modify_dripstone_thickness_to(
                world,
                &tip_pos.up_height(3),
                SpeleothemThickness::Middle,
            )
            .await;
        }
        _ => {}
    }
}
fn get_stalagmite_or_stalactice_len_and_dir_from_tip_pos(
    world: &Arc<World>,
    position: &BlockPos,
    block_state_id: BlockStateId,
) -> (u8, VerticalDirection) {
    let props =
        PointedDripstoneLikeProperties::from_state_id(block_state_id, &Block::POINTED_DRIPSTONE);

    let mut dripstone_len = 1;
    let mut next_dripstone_pos = offset_pos_by_vertical_dir(position, props.vertical_direction);
    //We dont care if it's longer than 5 blocks because of how thickness system works.
    while dripstone_len < 5 {
        if world.get_block(&next_dripstone_pos) != &Block::POINTED_DRIPSTONE {
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
    position: &BlockPos,
    placing_direction: Option<BlockDirection>,
    player_option: Option<&Player>,
) -> bool {
    // Determine support block
    let Some(support_block_vertical_direction) = get_support_block_vertical_direction(
        block_accessor,
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
    let support_block = block_accessor.get_block(&support_pos);
    if can_support_dripstone(support_block) {
        return true;
    }
    false
}

fn get_support_block_vertical_direction(
    block_accessor: &dyn BlockAccessor,
    position: &BlockPos,
    placing_direction_wrapper: Option<BlockDirection>,
    player_option: Option<&Player>,
) -> Option<VerticalDirection> {
    let Some(placing_direction) = placing_direction_wrapper else {
        //then this is basically called by a neighbor update check
        let (block, state) = block_accessor.get_block_and_state(position);
        if block != &Block::POINTED_DRIPSTONE {
            return None;
        }
        let props = PointedDripstoneLikeProperties::from_state_id(state.id, block);
        return Some(flip_dir(props.vertical_direction));
    };
    match block_direction_to_vertical_direction(placing_direction) {
        Some(ver_dir) => match ver_dir {
            VerticalDirection::Up => {
                let block_above = block_accessor.get_block(&position.up());
                let block_below = block_accessor.get_block(&position.down());
                if can_support_dripstone(block_above) {
                    return Some(VerticalDirection::Up);
                } else if can_support_dripstone(block_below) {
                    return Some(VerticalDirection::Down);
                }
                None
            }
            VerticalDirection::Down => {
                let block_above = block_accessor.get_block(&position.up());
                let block_below = block_accessor.get_block(&position.down());
                if can_support_dripstone(block_below) {
                    return Some(VerticalDirection::Down);
                } else if can_support_dripstone(block_above) {
                    return Some(VerticalDirection::Up);
                }
                None
            }
        },
        None => player_option.map_or(Some(VerticalDirection::Up), |player| {
            let (_, pitch) = player.rotation();
            let (can_place_above, can_place_below) = {
                let block_above = block_accessor.get_block(&position.up());
                let block_below = block_accessor.get_block(&position.down());
                (
                    can_support_dripstone(block_above),
                    can_support_dripstone(block_below),
                )
            };
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
fn can_support_dripstone(support_block: &Block) -> bool {
    if support_block == &Block::POINTED_DRIPSTONE {
        return true;
    }
    if support_block.default_state.is_full_cube() && support_block.default_state.is_solid_block() {
        return true;
    }
    false
}
async fn modify_dripstone_thickness_to(
    world: &Arc<World>,
    pos: &BlockPos,
    new_thickness: SpeleothemThickness,
) {
    let (block, support_block_state_id) = world.get_block_and_state_id(pos);

    if block != &Block::POINTED_DRIPSTONE {
        //this shouldn't happen
        return;
    }
    let mut support_props =
        PointedDripstoneLikeProperties::from_state_id(support_block_state_id, block);
    if support_props.thickness == new_thickness {
        return;
    }
    support_props.thickness = new_thickness;
    world
        .set_block_state(
            pos,
            support_props.to_state_id(&Block::POINTED_DRIPSTONE),
            BlockFlags::empty(),
        )
        .await;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stalactite_damage_clamps_to_six() {
        assert!((stalactite_damage_per_distance(1) - 6.0).abs() < f32::EPSILON);
        assert!((stalactite_damage_per_distance(5) - 6.0).abs() < f32::EPSILON);
        assert!((stalactite_damage_per_distance(6) - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stalactite_damage_scales_past_six() {
        assert!((stalactite_damage_per_distance(7) - 7.0).abs() < f32::EPSILON);
        assert!((stalactite_damage_per_distance(20) - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stalactite_damage_feeds_the_shared_fall_damage_formula() {
        // A 6-block stalactite falling 10 blocks: 6 * 10 = 60, capped at 40.
        let per_distance = stalactite_damage_per_distance(6);
        let damage = crate::block::blocks::falling::fall_damage_amount(
            10,
            per_distance,
            STALACTITE_MAX_DAMAGE,
        );
        assert!((damage - 40.0).abs() < f32::EPSILON);
        // The same stalactite falling 3 blocks: 6 * 3 = 18, under the cap.
        let damage = crate::block::blocks::falling::fall_damage_amount(
            3,
            per_distance,
            STALACTITE_MAX_DAMAGE,
        );
        assert!((damage - 18.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tip_thickness_covers_merged_tips() {
        assert!(is_tip_thickness(SpeleothemThickness::Tip));
        assert!(is_tip_thickness(SpeleothemThickness::TipMerge));
        assert!(!is_tip_thickness(SpeleothemThickness::Frustum));
        assert!(!is_tip_thickness(SpeleothemThickness::Middle));
        assert!(!is_tip_thickness(SpeleothemThickness::Base));
    }

    #[test]
    fn is_stalactite_only_matches_downward_dripstone() {
        let mut props = PointedDripstoneLikeProperties::default(&Block::POINTED_DRIPSTONE);
        props.vertical_direction = VerticalDirection::Down;
        let down = props.to_state_id(&Block::POINTED_DRIPSTONE);
        props.vertical_direction = VerticalDirection::Up;
        let up = props.to_state_id(&Block::POINTED_DRIPSTONE);

        assert!(is_stalactite(&Block::POINTED_DRIPSTONE, down));
        assert!(!is_stalactite(&Block::POINTED_DRIPSTONE, up));
        assert!(!is_stalactite(&Block::STONE, Block::STONE.default_state.id));
    }
}
