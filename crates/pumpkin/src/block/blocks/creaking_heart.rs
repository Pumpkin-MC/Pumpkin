use std::sync::Arc;

use pumpkin_data::block_properties::{
    Axis, BlockProperties, CreakingHeartLikeProperties, CreakingHeartState,
};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::entities::creaking_heart::CreakingHeartBlockEntity;
use crate::block::{BlockBehaviour, BrokenArgs, OnNeighborUpdateArgs, OnPlaceArgs, PlacedArgs};

#[pumpkin_block("minecraft:creaking_heart")]
pub struct CreakingHeartBlock;

impl CreakingHeartBlock {
    fn is_pale_oak_log(block: &Block) -> bool {
        block.name == "pale_oak_log" || block.name == "stripped_pale_oak_log"
    }

    fn check_active_logs(
        world: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
        axis: Axis,
    ) -> bool {
        let (pos_a, pos_b) = match axis {
            Axis::X => (pos.west(), pos.east()),
            Axis::Y => (pos.down(), pos.up()),
            Axis::Z => (pos.north(), pos.south()),
        };

        let state_a = world.get_block_state(&pos_a);
        let state_b = world.get_block_state(&pos_b);

        let block_a = Block::from_state_id(state_a.id);
        let block_b = Block::from_state_id(state_b.id);

        Self::is_pale_oak_log(block_a) && Self::is_pale_oak_log(block_b)
    }
}

impl BlockBehaviour for CreakingHeartBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props =
            CreakingHeartLikeProperties::from_state_id(args.block.default_state.id, args.block);
        props.axis = match args.direction {
            pumpkin_data::BlockDirection::North | pumpkin_data::BlockDirection::South => Axis::Z,
            pumpkin_data::BlockDirection::East | pumpkin_data::BlockDirection::West => Axis::X,
            pumpkin_data::BlockDirection::Up | pumpkin_data::BlockDirection::Down => Axis::Y,
        };
        props.creaking_heart_state = CreakingHeartState::Uprooted;
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let entity = CreakingHeartBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));

            let state_id = args.world.get_block_state_id(args.position);
            let mut props = CreakingHeartLikeProperties::from_state_id(state_id, args.block);

            if Self::check_active_logs(args.world.as_ref(), args.position, props.axis) {
                props.creaking_heart_state = CreakingHeartState::Dormant;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );

                args.world.play_sound(
                    Sound::BlockCreakingHeartSpawn,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
            }
        }
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        {
            let state_id = args.world.get_block_state_id(args.position);
            let mut props = CreakingHeartLikeProperties::from_state_id(state_id, args.block);

            let active_logs =
                Self::check_active_logs(args.world.as_ref(), args.position, props.axis);
            let new_state = if active_logs {
                if props.creaking_heart_state == CreakingHeartState::Uprooted {
                    args.world.play_sound(
                        Sound::BlockCreakingHeartSpawn,
                        SoundCategory::Blocks,
                        &args.position.to_f64(),
                    );
                }
                CreakingHeartState::Dormant
            } else {
                CreakingHeartState::Uprooted
            };

            if props.creaking_heart_state != new_state {
                props.creaking_heart_state = new_state;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        {
            args.world.play_sound(
                Sound::BlockCreakingHeartBreak,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );
            args.world
                .drop_stack(args.position, ItemStack::new(1, &Item::CREAKING_HEART));
        }
    }
}
