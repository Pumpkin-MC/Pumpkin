use pumpkin_data::block_properties::{BlockProperties, TurtleEggLikeProperties};
use pumpkin_data::entity::EntityPose;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{BlockDirection, BlockStateId, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::{
    BlockBehaviour, BlockFuture, BlockIsReplacing, BrokenArgs, CanPlaceAtArgs, CanUpdateAtArgs,
    GetStateForNeighborUpdateArgs, OnLandedUponArgs, OnPlaceArgs, OnScheduledTickArgs,
    RandomTickArgs,
};
use crate::entity::EntityBase;

type TurtleEggProperties = TurtleEggLikeProperties;

#[pumpkin_block("minecraft:turtle_egg")]
pub struct TurtleEggBlock;

impl BlockBehaviour for TurtleEggBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.player.get_entity().pose.load() != EntityPose::Crouching
                && let BlockIsReplacing::Itself(state_id) = args.replacing
            {
                let mut properties = TurtleEggProperties::from_state_id(state_id, args.block);
                if properties.eggs < 4 {
                    properties.eggs += 1;
                }
                return properties.to_state_id(args.block);
            }

            let properties = TurtleEggProperties::default(args.block);
            properties.to_state_id(args.block)
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position)
    }

    fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        let b = BlockAccessor::get_block(args.world, args.position);
        args.player.get_entity().pose.load() != EntityPose::Crouching
            && TurtleEggProperties::from_state_id(args.state_id, args.block).eggs < 4
            && args.block.id == b.id
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
            if !can_place_at(args.world.as_ref(), args.position) {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Turtle eggs can only hatch when placed on sand
            if !args
                .world
                .get_block(&args.position.down())
                .has_tag(&tag::Block::MINECRAFT_SAND)
            {
                return;
            }

            let state_id = args.world.get_block_state_id(args.position);
            let mut props = TurtleEggProperties::from_state_id(state_id, args.block);

            if props.hatch < 2 {
                props.hatch += 1;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;

                args.world.play_sound(
                    Sound::EntityTurtleEggCrack,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
            } else {
                args.world
                    .break_block(args.position, None, BlockFlags::SKIP_DROPS)
                    .await;

                args.world.play_sound(
                    Sound::EntityTurtleEggHatch,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
            }
        })
    }

    fn on_landed_upon<'a>(&'a self, args: OnLandedUponArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(living) = args.entity.get_living_entity() {
                living
                    .handle_fall_damage(args.entity, args.fall_distance, 1.0)
                    .await;
            }

            // Entity falling onto turtle egg breaks an egg
            if args.fall_distance > 0.5 {
                args.world.play_sound(
                    Sound::EntityTurtleEggBreak,
                    SoundCategory::Blocks,
                    &args.entity.get_entity().pos.load(),
                );
            }
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world.play_sound(
                Sound::EntityTurtleEggBreak,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );
        })
    }
}

fn can_place_at(block_accessor: &dyn BlockAccessor, position: &BlockPos) -> bool {
    let (support_block, state) = block_accessor.get_block_and_state(&position.down());
    support_block.has_tag(&tag::Block::MINECRAFT_SAND) || state.is_center_solid(BlockDirection::Up)
}
