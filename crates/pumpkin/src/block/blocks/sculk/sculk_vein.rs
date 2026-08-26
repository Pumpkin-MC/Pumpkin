use std::collections::HashSet;

use crate::block::{
    BlockBehaviour, BlockIsReplacing, BlockMetadata, CanPlaceAtArgs, CanUpdateAtArgs,
    GetStateForNeighborUpdateArgs, OnPlaceArgs, UseWithItemArgs, registry::BlockActionResult,
};
use crate::entity::{EntityBase, player::Player};
use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockStateId, FacingExt,
    block_properties::{BlockProperties, GlowLichenLikeProperties},
    item::Item,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

pub struct SculkVeinBlock;

impl BlockMetadata for SculkVeinBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::SCULK_VEIN].into()
    }
}

impl BlockBehaviour for SculkVeinBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if let BlockIsReplacing::Itself(state_id) = args.replacing {
            let (Some(direction), _) = get_attach_direction(
                args.world,
                args.position,
                Some(args.player),
                args.direction,
                true,
            ) else {
                return Block::AIR.default_state.id;
            };
            let mut props = GlowLichenLikeProperties::from_state_id(state_id, args.block);
            set_face(&mut props, direction);
            props.waterlogged = args.replacing.water_source();
            return props.to_state_id(args.block);
        }
        let (Some(direction), _) = get_attach_direction(
            args.world,
            args.position,
            Some(args.player),
            args.direction,
            false,
        ) else {
            return Block::AIR.default_state.id;
        };
        let mut props = GlowLichenLikeProperties::default(args.block);
        set_face(&mut props, direction);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        get_attach_direction(
            args.block_accessor,
            args.position,
            args.player,
            args.direction.unwrap_or(BlockDirection::Down),
            false,
        )
        .0
        .is_some()
    }

    fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        get_attach_direction(
            args.world,
            args.position,
            Some(args.player),
            args.direction,
            true,
        )
        .0
        .is_some()
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let old_props = GlowLichenLikeProperties::from_state_id(args.state_id, args.block);
        let mut new_directions = active_directions(old_props);

        let support = args
            .world
            .get_block(&args.position.offset(args.direction.to_offset()));
        if !is_solid_face(support) {
            new_directions.remove(&args.direction);
        }

        if new_directions.is_empty() {
            return Block::AIR.default_state.id;
        }
        let mut new_props = GlowLichenLikeProperties::default(args.block);
        for dir in new_directions {
            set_face(&mut new_props, dir);
        }
        new_props.to_state_id(args.block)
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        {
            if args.item_stack.item.id != Item::SCULK_VEIN.id {
                return BlockActionResult::Pass;
            }
            let state = args.world.get_block_state(args.position);
            let mut props = GlowLichenLikeProperties::from_state_id(state.id, args.block);

            let (Some(accurate_dir), _) = get_attach_direction(
                args.world.as_ref(),
                args.position,
                Some(args.player),
                *args.hit.face,
                true,
            ) else {
                return BlockActionResult::Fail;
            };
            set_face(&mut props, accurate_dir);

            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
            BlockActionResult::Consume
        }
    }
}

fn get_attach_direction(
    block_accessor: &dyn BlockAccessor,
    block_pos: &BlockPos,
    player_wrapper: Option<&Player>,
    click_direction: BlockDirection,
    replacing: bool,
) -> (Option<BlockDirection>, bool) {
    let clicked_block = block_accessor.get_block(&block_pos.offset(click_direction.to_offset()));

    if !replacing && clicked_block == &Block::SCULK_VEIN {
        return (None, false);
    }

    if is_solid_face(clicked_block) {
        return (Some(click_direction), false);
    }

    let (replacing_block, replacing_block_state) = block_accessor.get_block_and_state(block_pos);
    let already_active = if replacing_block == &Block::SCULK_VEIN {
        active_directions(GlowLichenLikeProperties::from_state_id(
            replacing_block_state.id,
            replacing_block,
        ))
    } else {
        HashSet::new()
    };

    if let Some(player) = player_wrapper {
        let fs = player.get_entity().get_entity_facing_order();
        let directions = [
            fs[0].to_block_direction(),
            fs[1].to_block_direction(),
            fs[2].to_block_direction(),
            fs[3].to_block_direction(),
            fs[4].to_block_direction(),
            fs[5].to_block_direction(),
        ];
        for dir in directions {
            if !already_active.contains(&dir) {
                let support = block_accessor.get_block(&block_pos.offset(dir.to_offset()));
                if is_solid_face(support) {
                    return (Some(dir), false);
                }
            }
        }
    }
    (None, false)
}

const fn is_solid_face(block: &Block) -> bool {
    block.default_state.is_full_cube()
}

fn active_directions(props: GlowLichenLikeProperties) -> HashSet<BlockDirection> {
    let mut set = HashSet::new();
    if props.down {
        set.insert(BlockDirection::Down);
    }
    if props.up {
        set.insert(BlockDirection::Up);
    }
    if props.north {
        set.insert(BlockDirection::North);
    }
    if props.south {
        set.insert(BlockDirection::South);
    }
    if props.east {
        set.insert(BlockDirection::East);
    }
    if props.west {
        set.insert(BlockDirection::West);
    }
    set
}

const fn set_face(props: &mut GlowLichenLikeProperties, direction: BlockDirection) {
    match direction {
        BlockDirection::Down => props.down = true,
        BlockDirection::Up => props.up = true,
        BlockDirection::North => props.north = true,
        BlockDirection::South => props.south = true,
        BlockDirection::West => props.west = true,
        BlockDirection::East => props.east = true,
    }
}
