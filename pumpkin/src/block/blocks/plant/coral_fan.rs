use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, FacingExt, HorizontalFacingExt,
    block_properties::{
        BlockProperties, Facing, LadderLikeProperties, MangroveRootsLikeProperties,
    },
    tag::{self, Taggable},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};
use rand::RngExt;

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockIsReplacing, BlockMetadata, CanPlaceAtArgs,
        GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
        blocks::plant::coral::scan_for_water,
    },
    entity::EntityBase,
    world::World,
};
pub struct CoralFanBlock;
impl BlockMetadata for CoralFanBlock {
    fn ids() -> Box<[u16]> {
        let alive_fans = [
            Block::BRAIN_CORAL_FAN.id,
            Block::BUBBLE_CORAL_FAN.id,
            Block::FIRE_CORAL_FAN.id,
            Block::HORN_CORAL_FAN.id,
            Block::TUBE_CORAL_FAN.id,
            Block::BRAIN_CORAL_WALL_FAN.id,
            Block::BUBBLE_CORAL_WALL_FAN.id,
            Block::FIRE_CORAL_WALL_FAN.id,
            Block::HORN_CORAL_WALL_FAN.id,
            Block::TUBE_CORAL_WALL_FAN.id,
        ];
        let mut plants = Vec::new();
        for alive_fan_id in alive_fans {
            let clone = alive_fan_id;
            plants.push(clone);
            plants.push(get_dead_type(Block::from_id(clone)).id);
        }
        plants.into()
    }
}

pub type CoralWallFanLikeProperties = LadderLikeProperties;
pub type CoralFanLikeProperties = MangroveRootsLikeProperties;

impl BlockBehaviour for CoralFanBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.direction == BlockDirection::Down {
                let support_block = args.world.get_block_state(&args.position.down()).await;
                if support_block.is_center_solid(BlockDirection::Up) {
                    return get_default_coral_fan_state_id(
                        args.block,
                        args.replacing.water_source(),
                    );
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
                let support_block = args.world.get_block_state(&args.position.down()).await;
                if support_block.is_center_solid(BlockDirection::Up) {
                    return get_default_coral_fan_state_id(
                        args.block,
                        args.replacing.water_source(),
                    );
                }
            }

            for dir in directions {
                if dir != Facing::Up
                    && dir != Facing::Down
                    && can_place_at(args.world, args.position, dir.to_block_direction()).await
                {
                    let Some(wall_block) = get_corresponding_wall_fan_type(args.block) else {
                        return 0;
                    };
                    let mut coral_wall_fan_props = CoralWallFanLikeProperties::default(&wall_block);
                    coral_wall_fan_props.waterlogged = args.replacing.water_source();
                    coral_wall_fan_props.facing = dir
                        .opposite()
                        .to_block_direction()
                        .to_horizontal_facing()
                        .unwrap();
                    return coral_wall_fan_props.to_state_id(&wall_block);
                }
            }

            let support_block = args.world.get_block_state(&args.position.down()).await;
            if support_block.is_center_solid(BlockDirection::Up) {
                return get_default_coral_fan_state_id(args.block, args.replacing.water_source());
            }
            0
        })
    }
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !scan_for_water(args.world, args.position).await {
                try_schedule_die_tick(args.block, args.world, args.position).await;
            }
        })
    }
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !scan_for_water(args.world, args.position).await && !is_dead(args.block) {
                let current_state = args.world.get_block_state(args.position).await;
                let dead_block_state_id = if is_wall_fan(args.block) {
                    let props =
                        CoralWallFanLikeProperties::from_state_id(current_state.id, args.block);
                    props.to_state_id(&get_dead_type(args.block))
                } else {
                    let props = CoralFanLikeProperties::from_state_id(current_state.id, args.block);
                    props.to_state_id(&get_dead_type(args.block))
                };
                args.world
                    .set_block_state(args.position, dead_block_state_id, BlockFlags::empty())
                    .await;
            }
        })
    }
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let replacing_block = args.block_accessor.get_block(&args.position).await;
            if replacing_block != &Block::WATER && replacing_block != args.block {
                return false;
            }
            let support_block = args
                .block_accessor
                .get_block_state(&args.position.down())
                .await;
            if support_block.is_center_solid(BlockDirection::Up) && !is_wall_fan(args.block) {
                return true;
            }
            for dir in BlockDirection::horizontal() {
                if can_place_at(args.block_accessor, args.position, dir).await {
                    return true;
                }
            }
            false
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if is_wall_fan(args.block) {
                let props = CoralWallFanLikeProperties::from_state_id(args.state_id, args.block);
                if props.facing.to_block_direction().opposite() == args.direction
                    && !can_place_at(
                        args.world,
                        args.position,
                        props.facing.to_block_direction().opposite(),
                    )
                    .await
                {
                    return 0;
                }
            } else if args.direction == BlockDirection::Down {
                let support_block = args.world.get_block_state(&args.position.down()).await;
                if !support_block.is_center_solid(BlockDirection::Up) {
                    return 0;
                }
            }
            args.state_id
        })
    }
}
fn get_default_coral_fan_state_id(block: &Block, waterlogged: bool) -> BlockStateId {
    let mut props = CoralFanLikeProperties::default(block);
    props.waterlogged = waterlogged;
    props.to_state_id(block)
}
async fn try_schedule_die_tick(block: &Block, world: &Arc<World>, pos: &BlockPos) {
    let tick_delay = 60 + rand::rng().random_range(0..40);
    world
        .schedule_block_tick(
            block,
            *pos,
            tick_delay as u8,
            pumpkin_world::tick::TickPriority::Normal,
        )
        .await;
}
fn is_wall_fan(block: &Block) -> bool {
    block.has_tag(&tag::Block::MINECRAFT_WALL_CORALS)
        || block == &Block::DEAD_BRAIN_CORAL_WALL_FAN
        || block == &Block::DEAD_BUBBLE_CORAL_WALL_FAN
        || block == &Block::DEAD_FIRE_CORAL_WALL_FAN
        || block == &Block::DEAD_HORN_CORAL_WALL_FAN
        || block == &Block::DEAD_TUBE_CORAL_WALL_FAN
}
fn is_dead(block: &Block) -> bool {
    block == &Block::DEAD_BRAIN_CORAL_FAN
        || block == &Block::DEAD_BRAIN_CORAL_WALL_FAN
        || block == &Block::DEAD_BUBBLE_CORAL_FAN
        || block == &Block::DEAD_BUBBLE_CORAL_WALL_FAN
        || block == &Block::DEAD_FIRE_CORAL_FAN
        || block == &Block::DEAD_FIRE_CORAL_WALL_FAN
        || block == &Block::DEAD_HORN_CORAL_FAN
        || block == &Block::DEAD_HORN_CORAL_WALL_FAN
        || block == &Block::DEAD_TUBE_CORAL_FAN
        || block == &Block::DEAD_TUBE_CORAL_WALL_FAN
}
fn get_dead_type(block: &Block) -> Block {
    if block == &Block::BRAIN_CORAL_FAN {
        Block::DEAD_BRAIN_CORAL_FAN
    } else if block == &Block::BRAIN_CORAL_WALL_FAN {
        Block::DEAD_BRAIN_CORAL_WALL_FAN
    } else if block == &Block::BUBBLE_CORAL_FAN {
        Block::DEAD_BUBBLE_CORAL_FAN
    } else if block == &Block::BUBBLE_CORAL_WALL_FAN {
        Block::DEAD_BUBBLE_CORAL_WALL_FAN
    } else if block == &Block::FIRE_CORAL_FAN {
        Block::DEAD_FIRE_CORAL_FAN
    } else if block == &Block::FIRE_CORAL_WALL_FAN {
        Block::DEAD_FIRE_CORAL_WALL_FAN
    } else if block == &Block::HORN_CORAL_FAN {
        Block::DEAD_HORN_CORAL_FAN
    } else if block == &Block::HORN_CORAL_WALL_FAN {
        Block::DEAD_HORN_CORAL_WALL_FAN
    } else if block == &Block::TUBE_CORAL_FAN {
        Block::DEAD_TUBE_CORAL_FAN
    } else {
        Block::DEAD_TUBE_CORAL_WALL_FAN
    }
}
fn get_corresponding_wall_fan_type(block: &Block) -> Option<Block> {
    if block == &Block::TUBE_CORAL_FAN {
        Some(Block::TUBE_CORAL_WALL_FAN)
    } else if block == &Block::BRAIN_CORAL_FAN {
        Some(Block::BRAIN_CORAL_WALL_FAN)
    } else if block == &Block::BUBBLE_CORAL_FAN {
        Some(Block::BUBBLE_CORAL_WALL_FAN)
    } else if block == &Block::FIRE_CORAL_FAN {
        Some(Block::FIRE_CORAL_WALL_FAN)
    } else if block == &Block::HORN_CORAL_FAN {
        Some(Block::HORN_CORAL_WALL_FAN)
    } else {
        None
    }
}
async fn can_place_at(
    world: &dyn BlockAccessor,
    block_pos: &BlockPos,
    facing: BlockDirection,
) -> bool {
    world
        .get_block_state(&block_pos.offset(facing.to_offset()))
        .await
        .is_side_solid(facing.opposite())
}
