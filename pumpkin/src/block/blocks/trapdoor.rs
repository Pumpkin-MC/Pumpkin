use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs};
use crate::entity::player::Player;
use crate::world::World;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::{BlockHalf, BlockProperties};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{RegistryKey, Taggable, get_tag_values};
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

type TrapDoorProperties = pumpkin_data::block_properties::OakTrapdoorLikeProperties;

async fn toggle_trapdoor(player: &Player, world: &Arc<World>, block_pos: &BlockPos) {
    let (block, block_state) = world.get_block_and_state_id(block_pos).await;
    let mut trapdoor_props = TrapDoorProperties::from_state_id(block_state, block);
    trapdoor_props.open = !trapdoor_props.open;

    world
        .play_block_sound_expect(
            player,
            get_sound(block, trapdoor_props.open),
            SoundCategory::Blocks,
            *block_pos,
        )
        .await;

    world
        .set_block_state(
            block_pos,
            trapdoor_props.to_state_id(block),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;
}

fn can_open_trapdoor(block: &Block) -> bool {
    if block == &Block::IRON_TRAPDOOR {
        return false;
    }
    true
}

// Todo: The sounds should be from BlockSetType
fn get_sound(block: &Block, open: bool) -> Sound {
    if open {
        if block.has_tag(&tag::Block::MINECRAFT_WOODEN_TRAPDOORS) {
            Sound::BlockWoodenTrapdoorOpen
        } else if block == &Block::IRON_TRAPDOOR {
            Sound::BlockIronTrapdoorOpen
        } else {
            Sound::BlockCopperTrapdoorOpen
        }
    } else if block.has_tag(&tag::Block::MINECRAFT_WOODEN_TRAPDOORS) {
        Sound::BlockWoodenTrapdoorClose
    } else if block == &Block::IRON_TRAPDOOR {
        Sound::BlockIronTrapdoorClose
    } else {
        Sound::BlockCopperTrapdoorClose
    }
}

#[pumpkin_block_from_tag("minecraft:trapdoors")]
pub struct TrapDoorBlock;

impl BlockBehaviour for TrapDoorBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if !can_open_trapdoor(args.block) {
                return BlockActionResult::Pass;
            }

            toggle_trapdoor(args.player, args.world, args.position).await;

            BlockActionResult::Success
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut trapdoor_props = TrapDoorProperties::default(args.block);
            trapdoor_props.waterlogged = args.replacing.water_source();

            let powered = block_receives_redstone_power(args.world, args.position).await;
            let direction = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();

            trapdoor_props.facing = direction;
            trapdoor_props.half = match args.direction {
                BlockDirection::Up => BlockHalf::Top,
                BlockDirection::Down => BlockHalf::Bottom,
                _ => match args.use_item_on.cursor_pos.y {
                    0.0..0.5 => BlockHalf::Bottom,
                    _ => BlockHalf::Top,
                },
            };
            trapdoor_props.powered = powered;
            trapdoor_props.open = powered;

            trapdoor_props.to_state_id(args.block)
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let block_state = args.world.get_block_state(args.position).await;
            let mut trapdoor_props = TrapDoorProperties::from_state_id(block_state.id, args.block);
            let powered = block_receives_redstone_power(args.world, args.position).await;
            if powered != trapdoor_props.powered {
                trapdoor_props.powered = !trapdoor_props.powered;

                if powered != trapdoor_props.open {
                    trapdoor_props.open = trapdoor_props.powered;

                    args.world
                        .play_block_sound(
                            get_sound(args.block, powered),
                            SoundCategory::Blocks,
                            *args.position,
                        )
                        .await;
                }
            }

            args.world
                .set_block_state(
                    args.position,
                    trapdoor_props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        })
    }
}
