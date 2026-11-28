use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, GetStateForNeighborUpdateArgs, NormalUseArgs, OnNeighborUpdateArgs,
    OnPlaceArgs, PathComputationType,
};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::world::World;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::Half;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockState, BlockStateId, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

type TrapDoorProperties = pumpkin_data::block_properties::OakTrapdoorLikeProperties;

fn toggle_trapdoor(player: &Player, world: &Arc<World>, block_pos: &BlockPos) {
    let (block, block_state) = world.get_block_and_state_id(block_pos);
    let mut trapdoor_props = TrapDoorProperties::from_state_id(block_state);
    trapdoor_props.open = !trapdoor_props.open;

    world.play_block_sound_expect(
        player,
        get_sound(block, trapdoor_props.open),
        SoundCategory::Blocks,
        *block_pos,
    );

    let game_event = if trapdoor_props.open {
        pumpkin_data::game_event::GameEvent::BlockOpen
    } else {
        pumpkin_data::game_event::GameEvent::BlockClose
    };
    world.emit_game_event(game_event.name(), block_pos.to_centered_f64());

    world.set_block_state(
        block_pos,
        trapdoor_props.to_state_id(block),
        BlockFlags::NOTIFY_LISTENERS,
    );
}

fn can_open_trapdoor(block: &Block) -> bool {
    block != &Block::IRON_TRAPDOOR
}

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

/// Facing and half of a placed trapdoor, as in vanilla `TrapDoorBlock#getStateForPlacement`.
fn trapdoor_placement(
    attached_face: BlockDirection,
    player_facing: HorizontalFacing,
    cursor_y: f32,
    replaces_clicked_block: bool,
) -> (HorizontalFacing, Half) {
    match attached_face.to_horizontal_facing() {
        // placed against a side of a full block: face the clicked side, and let
        // the cursor height pick the half
        Some(face) if !replaces_clicked_block => (
            face.opposite(),
            if cursor_y > 0.5 {
                Half::Top
            } else {
                Half::Bottom
            },
        ),
        // on the ground, under a ceiling, or inside a replaceable block such as
        // a snow layer: face the player, and be TOP unless placed upwards
        _ => (
            player_facing.opposite(),
            if attached_face == BlockDirection::Down {
                Half::Bottom
            } else {
                Half::Top
            },
        ),
    }
}

#[pumpkin_block_from_tag("minecraft:trapdoors")]
pub struct TrapDoorBlock;

impl BlockBehaviour for TrapDoorBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        {
            if !can_open_trapdoor(args.block) {
                return BlockActionResult::Pass;
            }

            toggle_trapdoor(args.player, args.world, args.position);

            BlockActionResult::Success
        }
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut trapdoor_props = TrapDoorProperties::default(args.block);
        trapdoor_props.waterlogged = args.replacing.water_source();

        let powered = block_receives_redstone_power(args.world, args.position);

        let player_facing = args.player.get_entity().get_horizontal_facing();
        // placing into the clicked position means the clicked block is replaceable
        let replaces_clicked_block = *args.position == args.use_item_on.position;

        let (facing, half) = trapdoor_placement(
            args.direction,
            player_facing,
            args.use_item_on.cursor_pos.y,
            replaces_clicked_block,
        );
        trapdoor_props.facing = facing;
        trapdoor_props.half = half;

        trapdoor_props.powered = powered;
        trapdoor_props.open = powered;

        trapdoor_props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let trapdoor_props = TrapDoorProperties::from_state_id(args.state_id);
        if trapdoor_props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                pumpkin_data::fluid::Fluid::WATER.flow.tick_delay,
                TickPriority::Normal,
            );
        }
        args.state_id
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let block_state = args.world.get_block_state(args.position);
        let mut trapdoor_props = TrapDoorProperties::from_state_id(block_state.id);
        let powered = block_receives_redstone_power(args.world, args.position);

        if powered != trapdoor_props.powered {
            if powered != trapdoor_props.open {
                trapdoor_props.open = powered;

                args.world.play_block_sound(
                    get_sound(args.block, powered),
                    SoundCategory::Blocks,
                    *args.position,
                );
                let game_event = if powered {
                    pumpkin_data::game_event::GameEvent::BlockOpen
                } else {
                    pumpkin_data::game_event::GameEvent::BlockClose
                };
                args.world
                    .emit_game_event(game_event.name(), args.position.to_centered_f64());
            }

            trapdoor_props.powered = powered;

            args.world.set_block_state(
                args.position,
                trapdoor_props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );
        }
    }

    fn is_pathfindable(&self, state: &BlockState, computation_type: PathComputationType) -> bool {
        let props = TrapDoorProperties::from_state_id(state.id);
        match computation_type {
            PathComputationType::Land | PathComputationType::Air => props.open,
            PathComputationType::Water => props.waterlogged,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_matches_vanilla() {
        let south = HorizontalFacing::South;
        // against the side of a full block: face the clicked side, half follows
        // the cursor height
        assert_eq!(
            trapdoor_placement(BlockDirection::West, south, 0.25, false),
            (HorizontalFacing::East, Half::Bottom)
        );
        assert_eq!(
            trapdoor_placement(BlockDirection::West, south, 0.75, false),
            (HorizontalFacing::East, Half::Top)
        );
        // clicking the side of a replaceable block (snow layer, short grass)
        // puts the trapdoor into that block: it faces the player and is TOP
        // whatever the cursor height is
        assert_eq!(
            trapdoor_placement(BlockDirection::West, south, 0.75, true),
            (HorizontalFacing::North, Half::Top)
        );
        // on the ground: face the player, bottom half
        assert_eq!(
            trapdoor_placement(BlockDirection::Down, south, 0.9, false),
            (HorizontalFacing::North, Half::Bottom)
        );
        // under a ceiling: face the player, top half
        assert_eq!(
            trapdoor_placement(BlockDirection::Up, south, 0.1, false),
            (HorizontalFacing::North, Half::Top)
        );
    }
}
