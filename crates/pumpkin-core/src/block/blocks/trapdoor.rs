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
use pumpkin_data::fluid::Fluid;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockState, BlockStateId};
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

    if let Some(sound) = get_sound(block, trapdoor_props.open) {
        world.play_block_sound_expect(player, sound, SoundCategory::Blocks, *block_pos);
    }

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
    block
        .block_set_type
        .is_some_and(|set_type| set_type.can_open_by_hand)
}

fn get_sound(block: &Block, open: bool) -> Option<Sound> {
    block.block_set_type.map(|set_type| {
        if open {
            set_type.trapdoor_open
        } else {
            set_type.trapdoor_close
        }
    })
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

        // Correct facing logic using Option unwrap
        let facing = args
            .direction
            .to_horizontal_facing()
            .unwrap_or(player_facing);

        trapdoor_props.facing = facing;

        trapdoor_props.half = match args.direction {
            BlockDirection::Up => Half::Top,
            BlockDirection::Down => Half::Bottom,
            _ => {
                if args.use_item_on.cursor_pos.y > 0.5 {
                    Half::Top
                } else {
                    Half::Bottom
                }
            }
        };

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

                if let Some(sound) = get_sound(args.block, powered) {
                    args.world
                        .play_block_sound(sound, SoundCategory::Blocks, *args.position);
                }
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
    fn can_open_by_hand_comes_from_the_block_set_type() {
        assert!(!can_open_trapdoor(&Block::IRON_TRAPDOOR));
        assert!(can_open_trapdoor(&Block::OAK_TRAPDOOR));
        assert!(can_open_trapdoor(&Block::CHERRY_TRAPDOOR));
        assert!(can_open_trapdoor(&Block::BAMBOO_TRAPDOOR));
        assert!(can_open_trapdoor(&Block::COPPER_TRAPDOOR));
    }

    #[test]
    fn trapdoor_sound_comes_from_the_block_set_type() {
        assert_eq!(
            get_sound(&Block::CHERRY_TRAPDOOR, true),
            Some(Sound::BlockCherryWoodTrapdoorOpen)
        );
        assert_eq!(
            get_sound(&Block::BAMBOO_TRAPDOOR, false),
            Some(Sound::BlockBambooWoodTrapdoorClose)
        );
        assert_eq!(
            get_sound(&Block::IRON_TRAPDOOR, true),
            Some(Sound::BlockIronTrapdoorOpen)
        );
        assert_eq!(
            get_sound(&Block::COPPER_TRAPDOOR, false),
            Some(Sound::BlockCopperTrapdoorClose)
        );
    }
}
