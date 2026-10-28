use crate::block::{
    BlockBehaviour, BlockIsReplacing, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    PlacedArgs,
};
use crate::entity::EntityBase;
use pumpkin_data::block_properties::{Facing, WallTorchLikeProperties, WhiteBannerLikeProperties};
use pumpkin_data::{Block, BlockDirection, BlockStateId, FacingExt, HorizontalFacingExt};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

use crate::block::entities::banner::BannerBlockEntity;
use std::sync::Arc;

#[pumpkin_block_from_tag("minecraft:banners")]
pub struct BannerBlock;

fn is_wall_banner(block: &Block) -> bool {
    block.name.ends_with("_wall_banner")
}

fn get_wall_banner_block(standing_block: &Block) -> Option<&Block> {
    if is_wall_banner(standing_block) {
        return Some(standing_block);
    }
    let color = standing_block.name.strip_suffix("_banner")?;
    Block::from_name(&format!("{color}_wall_banner"))
}

impl BlockBehaviour for BannerBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = BannerBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if args.direction == BlockDirection::Down && can_place_standing(args.world, args.position) {
            let mut props = WhiteBannerLikeProperties::default(args.block);
            props.rotation = args.player.get_entity().get_flipped_rotation_16();
            return props.to_state_id(args.block);
        }

        let mut directions = args.player.get_entity().get_entity_facing_order();
        if args.replacing == BlockIsReplacing::None {
            let face = args.direction.to_facing();
            let mut i = 0;
            while i < directions.len() && directions[i] != face {
                i += 1;
            }
            if i > 0 {
                directions.copy_within(0..i, 1);
                directions[0] = face;
            }
        }

        for dir in directions {
            if dir == Facing::Down {
                if can_place_standing(args.world, args.position) {
                    let mut props = WhiteBannerLikeProperties::default(args.block);
                    props.rotation = args.player.get_entity().get_flipped_rotation_16();
                    return props.to_state_id(args.block);
                }
            } else if dir != Facing::Up
                && can_place_wall(args.world, args.position, dir.to_block_direction())
                && let Some(wall_block) = get_wall_banner_block(args.block)
                && let Some(facing) = dir.opposite().to_horizontal_facing()
            {
                let mut props = WallTorchLikeProperties::default(wall_block);
                props.facing = facing;
                return props.to_state_id(wall_block);
            }
        }

        Block::AIR.default_state.id
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        if args.player.is_none() {
            if is_wall_banner(args.block) {
                let props = WallTorchLikeProperties::from_state_id(args.state.id);
                return can_place_wall(
                    args.block_accessor,
                    args.position,
                    props.facing.to_block_direction().opposite(),
                );
            }
            return can_place_standing(args.block_accessor, args.position);
        }

        if can_place_standing(args.block_accessor, args.position) {
            return true;
        }
        for dir in BlockDirection::horizontal() {
            if can_place_wall(args.block_accessor, args.position, dir.to_block_direction()) {
                return true;
            }
        }
        false
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if is_wall_banner(args.block) {
            let props = WallTorchLikeProperties::from_state_id(args.state_id);
            let support_dir = props.facing.to_block_direction().opposite();
            if args.direction == support_dir
                && !can_place_wall(args.world, args.position, support_dir)
            {
                return Block::AIR.default_state.id;
            }
        } else if args.direction == BlockDirection::Down
            && !can_place_standing(args.world, args.position)
        {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }
}

fn can_place_standing(world: &dyn BlockAccessor, position: &BlockPos) -> bool {
    world.get_block_state(&position.down()).is_solid()
}

fn can_place_wall(
    world: &dyn BlockAccessor,
    position: &BlockPos,
    support_dir: BlockDirection,
) -> bool {
    world
        .get_block_state(&position.offset(support_dir.to_offset()))
        .is_solid()
}
