use pumpkin_data::block_properties::SnifferEggLikeProperties;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use uuid::Uuid;

use crate::block::{
    BlockBehaviour, OnPlaceArgs, OnScheduledTickArgs, PathComputationType, PlacedArgs,
};
use crate::entity::r#type::from_type;

#[pumpkin_block("minecraft:sniffer_egg")]
pub struct SnifferEggBlock;

impl SnifferEggBlock {
    fn hatch_boost(
        world: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        world
            .get_block(&pos.down())
            .has_tag(&tag::Block::MINECRAFT_SNIFFER_EGG_HATCH_BOOST)
    }

    const fn get_hatch_delay(boosted: bool) -> u8 {
        if boosted { 100 } else { 200 }
    }
}

impl BlockBehaviour for SnifferEggBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let props = SnifferEggLikeProperties::default(args.block);
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let boosted = Self::hatch_boost(args.world.as_ref(), args.position);
        let delay = Self::get_hatch_delay(boosted);
        args.world
            .schedule_block_tick(args.block, *args.position, delay, TickPriority::Normal);
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = SnifferEggLikeProperties::from_state_id(state_id);

        if props.hatch < 2 {
            props.hatch += 1;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );

            args.world.play_sound(
                Sound::BlockSnifferEggCrack,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );

            let boosted = Self::hatch_boost(args.world.as_ref(), args.position);
            let delay = Self::get_hatch_delay(boosted);
            args.world
                .schedule_block_tick(args.block, *args.position, delay, TickPriority::Normal);
        } else {
            args.world
                .break_block(args.position, None, BlockFlags::SKIP_DROPS);

            args.world.play_sound(
                Sound::BlockSnifferEggHatch,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );

            let sniffer = from_type(
                &EntityType::SNIFFER,
                args.position.to_centered_f64(),
                args.world,
                Uuid::new_v4(),
            );
            sniffer.get_entity().set_age(-24000);
            args.world.spawn_entity(sniffer);
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
