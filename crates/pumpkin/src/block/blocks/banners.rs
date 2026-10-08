use crate::block::{BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs, PlacedArgs};
use crate::entity::EntityBase;
use pumpkin_data::block_properties::{WhiteBannerLikeProperties, WhiteWallBannerProperties};
use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId, FacingExt, HorizontalFacingExt,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

use crate::block::entities::banner::BannerBlockEntity;
use std::sync::Arc;

#[pumpkin_block_from_tag("minecraft:banners")]
pub struct BannerBlock;

impl BlockBehaviour for BannerBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let entity = BannerBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        }
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut directions = args
            .player
            .get_entity()
            .get_entity_facing_order()
            .map(|d| d.to_block_direction());
        // Vanilla prioritizes the clicked face only when placing beside the clicked block.
        if args.position != &args.use_item_on.position {
            let index = directions
                .iter()
                .position(|d| *d == args.direction)
                .unwrap();
            directions[..=index].rotate_right(1);
        }
        let Some(direction) = select_support(&directions, |direction| {
            has_support(args.world, args.position, direction)
        }) else {
            return BlockStateId::AIR;
        };
        if direction.is_horizontal() {
            let color = args
                .block
                .name
                .strip_suffix("_wall_banner")
                .or_else(|| args.block.name.strip_suffix("_banner"))
                .unwrap();
            let wall_block = Block::from_name(&format!("{color}_wall_banner")).unwrap();
            let mut props = WhiteWallBannerProperties::default(wall_block);
            props.facing = direction.opposite().to_cardinal_direction();
            return props.to_state_id(wall_block);
        }
        let mut props = WhiteBannerLikeProperties::default(args.block);
        props.rotation = args.player.get_entity().get_flipped_rotation_16();
        props.to_state_id(args.block)
    }

    fn can_place_at(&self, args: crate::block::CanPlaceAtArgs<'_>) -> bool {
        // The registry validates the item before on_place selects its standing or wall state.
        if args.player.is_some() && args.use_item_on.is_some() {
            return has_support(args.block_accessor, args.position, BlockDirection::Down)
                || BlockDirection::horizontal().iter().any(|d| {
                    has_support(args.block_accessor, args.position, d.to_block_direction())
                });
        }
        has_support(
            args.block_accessor,
            args.position,
            support_direction(args.block, args.state.id),
        )
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        state_after_neighbor_update(
            args.block,
            args.state_id,
            args.direction,
            args.neighbor_state_id,
        )
    }
}

fn select_support(
    directions: &[BlockDirection; 6],
    has_support: impl Fn(BlockDirection) -> bool,
) -> Option<BlockDirection> {
    // WallBannerBlock computes its first valid horizontal state before the item
    // chooses between that state and the standing banner in direction order.
    let wall = directions
        .iter()
        .copied()
        .find(|d| d.is_horizontal() && has_support(*d));
    for direction in directions {
        if *direction == BlockDirection::Down {
            if has_support(*direction) {
                return Some(*direction);
            }
        } else if direction.is_horizontal() && wall.is_some() {
            return wall;
        }
    }
    None
}

fn support_direction(block: &Block, state_id: BlockStateId) -> BlockDirection {
    if block.name.ends_with("_wall_banner") {
        WhiteWallBannerProperties::from_state_id(state_id)
            .facing
            .to_block_direction()
            .opposite()
    } else {
        BlockDirection::Down
    }
}

fn state_after_neighbor_update(
    block: &Block,
    state_id: BlockStateId,
    direction: BlockDirection,
    neighbor_state_id: BlockStateId,
) -> BlockStateId {
    if direction == support_direction(block, state_id)
        && !BlockState::from_id(neighbor_state_id).is_solid()
    {
        BlockStateId::AIR
    } else {
        state_id
    }
}

fn has_support(world: &dyn BlockAccessor, position: &BlockPos, direction: BlockDirection) -> bool {
    let state = world.get_block_state(&position.offset(direction.to_offset()));
    state.is_solid()
}

#[cfg(test)]
mod tests {
    use super::*;
    use BlockDirection::{Down, East, North, South, Up, West};

    #[test]
    fn wall_banners_use_their_matching_standing_banner_loot_table() {
        for color in [
            "white",
            "orange",
            "magenta",
            "light_blue",
            "yellow",
            "lime",
            "pink",
            "gray",
            "light_gray",
            "cyan",
            "purple",
            "blue",
            "brown",
            "green",
            "red",
            "black",
        ] {
            let wall = Block::from_name(&format!("{color}_wall_banner")).unwrap();
            let standing = Block::from_name(&format!("{color}_banner")).unwrap();
            assert_eq!(crate::block::loot_table_block(wall), standing);
            assert_eq!(crate::block::loot_table_block(standing), standing);
            assert!(
                pumpkin_data::loot_table::get_loot_table(&format!(
                    "minecraft:blocks/{}",
                    standing.name
                ))
                .is_some()
            );
        }
        assert_eq!(crate::block::loot_table_block(&Block::STONE), &Block::STONE);
    }

    #[test]
    fn floor_or_wall_follows_placement_order() {
        let supports = |d| matches!(d, Down | North);
        assert_eq!(
            select_support(&[Down, North, South, East, West, Up], supports),
            Some(Down)
        );
        assert_eq!(
            select_support(&[North, Down, South, East, West, Up], supports),
            Some(North)
        );
    }

    #[test]
    fn wall_candidate_is_computed_before_item_selects_variant() {
        // The first horizontal direction has no support, but Vanilla still uses
        // the wall state found later rather than the intervening floor candidate.
        assert_eq!(
            select_support(&[North, Down, South, East, West, Up], |d| matches!(
                d,
                Down | South
            )),
            Some(South)
        );
    }

    #[test]
    fn ceiling_is_ignored_and_missing_support_rejects_placement() {
        let directions = [Up, North, Down, South, East, West];
        assert_eq!(select_support(&directions, |d| d == Up), None);
        assert_eq!(select_support(&directions, |_| false), None);
        assert_eq!(select_support(&directions, |d| d == Down), Some(Down));
    }

    #[test]
    fn losing_support_removes_banner_immediately_but_other_updates_do_not() {
        let standing = &Block::WHITE_BANNER;
        let state_id = standing.default_state.id;
        assert_eq!(
            state_after_neighbor_update(standing, state_id, Down, BlockStateId::AIR),
            BlockStateId::AIR
        );
        assert_eq!(
            state_after_neighbor_update(standing, state_id, North, BlockStateId::AIR),
            state_id
        );
        assert_eq!(
            state_after_neighbor_update(standing, state_id, Down, Block::STONE.default_state.id),
            state_id
        );
        let wall = &Block::WHITE_WALL_BANNER;
        for support in [North, South, East, West] {
            let mut props = WhiteWallBannerProperties::default(wall);
            props.facing = support.opposite().to_cardinal_direction();
            let state_id = props.to_state_id(wall);
            assert_eq!(
                state_after_neighbor_update(wall, state_id, support, BlockStateId::AIR),
                BlockStateId::AIR
            );
            assert_eq!(
                state_after_neighbor_update(wall, state_id, Down, BlockStateId::AIR),
                state_id
            );
            assert_eq!(
                state_after_neighbor_update(wall, state_id, support, Block::STONE.default_state.id),
                state_id
            );
        }
    }

    #[test]
    fn every_banner_color_faces_away_from_its_wall_support() {
        for color in [
            "white",
            "orange",
            "magenta",
            "light_blue",
            "yellow",
            "lime",
            "pink",
            "gray",
            "light_gray",
            "cyan",
            "purple",
            "blue",
            "brown",
            "green",
            "red",
            "black",
        ] {
            let standing = Block::from_name(&format!("{color}_banner")).unwrap();
            assert_eq!(support_direction(standing, standing.default_state.id), Down);
            let wall = Block::from_name(&format!("{color}_wall_banner")).unwrap();
            for direction in [North, South, East, West] {
                let mut props = WhiteWallBannerProperties::default(wall);
                props.facing = direction.opposite().to_cardinal_direction();
                assert_eq!(support_direction(wall, props.to_state_id(wall)), direction);
            }
        }
    }
}
