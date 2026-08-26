use crate::{
    block::{
        BlockBehaviour, BlockIsReplacing, BlockMetadata, CanPlaceAtArgs,
        GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
        blocks::coral::{is_dead_coral, scan_for_water, try_schedule_die_tick},
    },
    entity::EntityBase,
};
use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockStateId, FacingExt, HorizontalFacingExt,
    block_properties::{
        BlockProperties, Facing, HorizontalFacing, LadderLikeProperties,
        MangroveRootsLikeProperties,
    },
    tag::{self, Taggable},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

pub struct CoralFanBlock;

impl BlockMetadata for CoralFanBlock {
    fn ids() -> Box<[BlockId]> {
        let alive_wall_fans: Vec<BlockId> = tag::Block::MINECRAFT_WALL_CORALS
            .1
            .iter()
            .map(|v| BlockId::new_or_air(*v))
            .collect();
        let alive_coral_fans: &[BlockId] = &[
            BlockId::BRAIN_CORAL_FAN,
            BlockId::BUBBLE_CORAL_FAN,
            BlockId::FIRE_CORAL_FAN,
            BlockId::HORN_CORAL_FAN,
            BlockId::TUBE_CORAL_FAN,
        ];
        let alive_fans = [alive_wall_fans.as_slice(), alive_coral_fans].concat();
        let mut plants = Vec::new();
        for alive_fan_id in alive_fans {
            plants.push(alive_fan_id);
            plants.push(get_dead_type(alive_fan_id).map_or(BlockId::AIR, |b| b.id));
        }
        plants.into()
    }
}

pub type CoralWallFanLikeProperties = LadderLikeProperties;
pub type CoralFanLikeProperties = MangroveRootsLikeProperties;

impl BlockBehaviour for CoralFanBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if args.direction == BlockDirection::Down {
            let support_block = args.world.get_block_state(&args.position.down());
            if support_block.is_center_solid(BlockDirection::Up) {
                return get_default_coral_fan_state_id(args.block, args.replacing.water_source());
            }
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
        } else if directions[0] == Facing::Down {
            let support_block = args.world.get_block_state(&args.position.down());
            if support_block.is_center_solid(BlockDirection::Up) {
                return get_default_coral_fan_state_id(args.block, args.replacing.water_source());
            }
        }

        for dir in directions {
            if let (Some(h_facing), Some(opp_facing)) = (
                dir.to_horizontal_facing(),
                dir.opposite().to_horizontal_facing(),
            ) && can_place_at(args.world, args.position, h_facing)
            {
                let Some(wall_block) = get_corresponding_wall_fan_type(args.block.id) else {
                    return BlockStateId::AIR;
                };
                let mut coral_wall_fan_props = CoralWallFanLikeProperties::default(wall_block);
                coral_wall_fan_props.waterlogged = args.replacing.water_source();
                coral_wall_fan_props.facing = opp_facing;
                return coral_wall_fan_props.to_state_id(wall_block);
            }
        }

        let support_block = args.world.get_block_state(&args.position.down());
        if support_block.is_center_solid(BlockDirection::Up) {
            return get_default_coral_fan_state_id(args.block, args.replacing.water_source());
        }
        BlockStateId::AIR
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            if !scan_for_water(args.world, args.position) {
                try_schedule_die_tick(args.block, args.world, args.position);
            }
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !scan_for_water(args.world, args.position) && !is_dead_coral(args.block) {
            let current_state = args.world.get_block_state(args.position);

            let Some(dead_block) = get_dead_type(args.block.id) else {
                return;
            };

            // VANILLA FIX: Explicitly set waterlogged to false when dying
            let dead_block_state_id = if is_wall_fan(args.block) {
                let mut props =
                    CoralWallFanLikeProperties::from_state_id(current_state.id, args.block);
                props.waterlogged = false;
                props.to_state_id(dead_block)
            } else {
                let mut props = CoralFanLikeProperties::from_state_id(current_state.id, args.block);
                props.waterlogged = false;
                props.to_state_id(dead_block)
            };

            args.world
                .set_block_state(args.position, dead_block_state_id, BlockFlags::empty());
        }
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let support_block = args.block_accessor.get_block_state(&args.position.down());
        if support_block.is_center_solid(BlockDirection::Up) && !is_wall_fan(args.block) {
            return true;
        }
        for dir in BlockDirection::horizontal() {
            if can_place_at(args.block_accessor, args.position, dir) {
                return true;
            }
        }
        false
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if is_wall_fan(args.block) {
            let props = CoralWallFanLikeProperties::from_state_id(args.state_id, args.block);
            if props.facing.to_block_direction().opposite() == args.direction
                && !can_place_at(args.world, args.position, props.facing.opposite())
            {
                return BlockStateId::AIR;
            }
        } else if args.direction == BlockDirection::Down {
            let support_block = args.world.get_block_state(&args.position.down());
            if !support_block.is_center_solid(BlockDirection::Up) {
                return BlockStateId::AIR;
            }
        }

        args.state_id
    }
}

fn get_default_coral_fan_state_id(block: &Block, waterlogged: bool) -> BlockStateId {
    let mut props = CoralFanLikeProperties::default(block);
    props.waterlogged = waterlogged;
    props.to_state_id(block)
}

fn is_wall_fan(block: &Block) -> bool {
    block.has_tag(&tag::Block::MINECRAFT_WALL_CORALS)
        || block == &Block::DEAD_BRAIN_CORAL_WALL_FAN
        || block == &Block::DEAD_BUBBLE_CORAL_WALL_FAN
        || block == &Block::DEAD_FIRE_CORAL_WALL_FAN
        || block == &Block::DEAD_HORN_CORAL_WALL_FAN
        || block == &Block::DEAD_TUBE_CORAL_WALL_FAN
}

const fn get_dead_type(id: BlockId) -> Option<&'static Block> {
    match id {
        BlockId::BRAIN_CORAL_FAN => Some(&Block::DEAD_BRAIN_CORAL_FAN),
        BlockId::BRAIN_CORAL_WALL_FAN => Some(&Block::DEAD_BRAIN_CORAL_WALL_FAN),
        BlockId::BUBBLE_CORAL_FAN => Some(&Block::DEAD_BUBBLE_CORAL_FAN),
        BlockId::BUBBLE_CORAL_WALL_FAN => Some(&Block::DEAD_BUBBLE_CORAL_WALL_FAN),
        BlockId::FIRE_CORAL_FAN => Some(&Block::DEAD_FIRE_CORAL_FAN),
        BlockId::FIRE_CORAL_WALL_FAN => Some(&Block::DEAD_FIRE_CORAL_WALL_FAN),
        BlockId::HORN_CORAL_FAN => Some(&Block::DEAD_HORN_CORAL_FAN),
        BlockId::HORN_CORAL_WALL_FAN => Some(&Block::DEAD_HORN_CORAL_WALL_FAN),
        BlockId::TUBE_CORAL_FAN => Some(&Block::DEAD_TUBE_CORAL_FAN),
        BlockId::TUBE_CORAL_WALL_FAN => Some(&Block::DEAD_TUBE_CORAL_WALL_FAN),
        _ => None,
    }
}

const fn get_corresponding_wall_fan_type(id: BlockId) -> Option<&'static Block> {
    match id {
        BlockId::TUBE_CORAL_FAN => Some(&Block::TUBE_CORAL_WALL_FAN),
        BlockId::BRAIN_CORAL_FAN => Some(&Block::BRAIN_CORAL_WALL_FAN),
        BlockId::BUBBLE_CORAL_FAN => Some(&Block::BUBBLE_CORAL_WALL_FAN),
        BlockId::FIRE_CORAL_FAN => Some(&Block::FIRE_CORAL_WALL_FAN),
        BlockId::HORN_CORAL_FAN => Some(&Block::HORN_CORAL_WALL_FAN),
        BlockId::DEAD_TUBE_CORAL_FAN => Some(&Block::DEAD_TUBE_CORAL_WALL_FAN),
        BlockId::DEAD_BRAIN_CORAL_FAN => Some(&Block::DEAD_BRAIN_CORAL_WALL_FAN),
        BlockId::DEAD_BUBBLE_CORAL_FAN => Some(&Block::DEAD_BUBBLE_CORAL_WALL_FAN),
        BlockId::DEAD_FIRE_CORAL_FAN => Some(&Block::DEAD_FIRE_CORAL_WALL_FAN),
        BlockId::DEAD_HORN_CORAL_FAN => Some(&Block::DEAD_HORN_CORAL_WALL_FAN),
        _ => None,
    }
}

fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos, facing: HorizontalFacing) -> bool {
    world
        .get_block_state(&block_pos.offset(facing.to_offset()))
        .is_side_solid(facing.opposite().to_block_direction())
}
