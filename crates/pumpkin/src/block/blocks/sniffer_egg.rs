use pumpkin_data::block_properties::{BlockProperties, SnifferEggLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
};

#[pumpkin_block("minecraft:sniffer_egg")]
pub struct SnifferEggBlock;

impl SnifferEggBlock {
    fn is_on_moss(
        world: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        let below_pos = pos.down();
        let state = world.get_block_state(&below_pos);
        let block = Block::from_state_id(state.id);
        block.name == "moss_block"
    }

    const fn get_hatch_delay(on_moss: bool) -> u8 {
        if on_moss { 100 } else { 200 }
    }
}

impl BlockBehaviour for SnifferEggBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let props = SnifferEggLikeProperties::default(args.block);
            props.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world.play_sound(
                Sound::BlockSnifferEggPlop,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );

            let on_moss = Self::is_on_moss(args.world.as_ref(), args.position);
            let delay = Self::get_hatch_delay(on_moss);
            args.world
                .schedule_block_tick(args.block, *args.position, delay, TickPriority::Normal);
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position);
            let mut props = SnifferEggLikeProperties::from_state_id(state_id, args.block);

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
                    Sound::BlockSnifferEggCrack,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );

                let on_moss = Self::is_on_moss(args.world.as_ref(), args.position);
                let delay = Self::get_hatch_delay(on_moss);
                args.world.schedule_block_tick(
                    args.block,
                    *args.position,
                    delay,
                    TickPriority::Normal,
                );
            } else {
                args.world
                    .break_block(args.position, None, BlockFlags::SKIP_DROPS)
                    .await;

                args.world.play_sound(
                    Sound::BlockSnifferEggHatch,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
            }
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world.play_sound(
                Sound::BlockSnifferEggCrack,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );
            args.world
                .drop_stack(args.position, ItemStack::new(1, &Item::SNIFFER_EGG))
                .await;
        })
    }
}
