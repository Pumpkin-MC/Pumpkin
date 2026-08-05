use crate::block::blocks::copper_weathering;
use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, RandomTickArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_data::{BlockDirection, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block_from_tag("minecraft:lanterns")]
pub struct LanternBlock;

impl BlockBehaviour for LanternBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props =
                pumpkin_data::block_properties::LanternLikeProperties::default(args.block);
            props.r#waterlogged = args.replacing.water_source();

            props.r#hanging = hanging_for_placement(
                args.direction,
                args.use_item_on.cursor_pos.y,
                floor_supports(args.world, args.position),
                ceiling_supports(args.world, args.position),
            );

            props.to_state_id(args.block)
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        args.world
            .is_some_and(|world| can_place_at(world, args.position))
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position) {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
            }
            args.state_id
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position) {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // No tag gate needed: the oxidation_stages table below only contains the
            // copper lantern family, so this is a no-op for every other lantern type.

            let current_state_id = args.world.get_block_state_id(args.position);
            let lantern_props =
                pumpkin_data::block_properties::LanternLikeProperties::from_state_id(
                    current_state_id,
                    args.block,
                );

            let oxidation_stages = [
                &Block::COPPER_LANTERN,
                &Block::EXPOSED_COPPER_LANTERN,
                &Block::WEATHERED_COPPER_LANTERN,
                &Block::OXIDIZED_COPPER_LANTERN,
            ];

            copper_weathering::try_oxidize_copper(
                args.world,
                args.position,
                args.block,
                &oxidation_stages,
                |next_block| {
                    let mut new_props =
                        pumpkin_data::block_properties::LanternLikeProperties::default(next_block);
                    new_props.r#hanging = lantern_props.r#hanging;
                    new_props.r#waterlogged = lantern_props.r#waterlogged;
                    new_props.to_state_id(next_block)
                },
            )
            .await;
        })
    }
}

/// Orientation of a freshly placed lantern.
///
/// minecraft.wiki "Lantern": "To place a lantern on top of a block, aim at the block's top
/// face, and press use. To hang a lantern from the bottom of a block, aim at the block's
/// bottom face, and press use." and "Pressing use on the top half of the adjacent block will
/// hang the lantern from the bottom of an above block, and pressing use on the bottom half of
/// the adjacent block will place the lantern down on the top face of a bottom block."
///
/// `direction` is the placement face handed to `on_place`, i.e. the clicked face inverted
/// (see `BlockRegistry::on_use_with_item`, which passes `face.opposite()`), so `Down` means
/// the player aimed at a top face and `Up` means a bottom face. Slabs use the same
/// direction-then-cursor-half shape.
fn hanging_for_placement(
    direction: BlockDirection,
    cursor_y: f32,
    floor_supports: bool,
    ceiling_supports: bool,
) -> bool {
    match direction {
        BlockDirection::Up => true,
        BlockDirection::Down => false,
        _ => {
            let hanging = cursor_y >= 0.5;
            // Side clicks only pick a preference; if that side has no support but the other
            // one does, use the supported one so the lantern does not immediately pop off.
            if hanging && !ceiling_supports && floor_supports {
                false
            } else if !hanging && !floor_supports && ceiling_supports {
                true
            } else {
                hanging
            }
        }
    }
}

fn floor_supports(world: &World, position: &BlockPos) -> bool {
    //idk why this don't update with .is_center_solid so this is a 'temporary patch'
    if world
        .get_block(&position.down())
        .has_tag(&tag::Block::C_FENCE_GATES)
    {
        let fence_gate_props =
            pumpkin_data::block_properties::OakFenceGateLikeProperties::from_state_id(
                world.get_block_state_id(&position.down()),
                world.get_block(&position.down()),
            );

        if fence_gate_props.open {
            return false;
        }
    }
    let (block_down, block_down_state) = world.get_block_and_state(&position.down());
    block_down_state.is_center_solid(BlockDirection::Up)
        || block_down.has_tag(&tag::Block::MINECRAFT_UNSTABLE_BOTTOM_CENTER)
}

fn ceiling_supports(world: &World, position: &BlockPos) -> bool {
    world
        .get_block_state(&position.up())
        .is_center_solid(BlockDirection::Down)
}

fn can_place_at(world: &World, position: &BlockPos) -> bool {
    //idk why this don't update with .is_center_solid so this is a 'temporary patch'
    if world
        .get_block(&position.down())
        .has_tag(&tag::Block::C_FENCE_GATES)
    {
        let fence_gate_props =
            pumpkin_data::block_properties::OakFenceGateLikeProperties::from_state_id(
                world.get_block_state_id(&position.down()),
                world.get_block(&position.down()),
            );

        if fence_gate_props.open {
            return false;
        }
    }
    floor_supports(world, position) || ceiling_supports(world, position)
}

#[cfg(test)]
mod tests {
    use super::hanging_for_placement;
    use pumpkin_data::BlockDirection;

    #[test]
    fn clicked_top_face_places_standing_lantern() {
        // Aiming at a block's top face => placement direction Down.
        assert!(!hanging_for_placement(
            BlockDirection::Down,
            0.5,
            true,
            false
        ));
    }

    #[test]
    fn clicked_bottom_face_places_hanging_lantern() {
        // Aiming at a block's bottom face => placement direction Up.
        assert!(hanging_for_placement(BlockDirection::Up, 0.5, false, true));
    }

    #[test]
    fn one_block_gap_top_face_click_is_standing() {
        // Falsifier: solid blocks both above and below, player aims at the floor's top face.
        // The old rule keyed off the block above and produced a hanging lantern.
        assert!(!hanging_for_placement(
            BlockDirection::Down,
            0.5,
            true,
            true
        ));
    }

    #[test]
    fn one_block_gap_bottom_face_click_is_hanging() {
        assert!(hanging_for_placement(BlockDirection::Up, 0.5, true, true));
    }

    #[test]
    fn side_click_uses_cursor_half() {
        for dir in [
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::East,
            BlockDirection::West,
        ] {
            assert!(hanging_for_placement(dir, 0.75, true, true));
            assert!(!hanging_for_placement(dir, 0.25, true, true));
        }
    }

    #[test]
    fn side_click_falls_back_to_the_supported_side() {
        assert!(!hanging_for_placement(
            BlockDirection::North,
            0.75,
            true,
            false
        ));
        assert!(hanging_for_placement(
            BlockDirection::North,
            0.25,
            false,
            true
        ));
    }
}
