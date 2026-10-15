use std::sync::Arc;

use crate::block::{BlockBehaviour, BlockMetadata, OnPlaceArgs, OnScheduledTickArgs, PlacedArgs};
use pumpkin_data::{
    BlockId, BlockStateId, block_properties::SculkCatalystLikeProperties,
    sound::{Sound, SoundCategory},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

pub struct SculkCatalystBlock;

impl BlockMetadata for SculkCatalystBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::SCULK_CATALYST].into()
    }
}

impl SculkCatalystBlock {
    pub fn bloom(world: &Arc<crate::world::World>, pos: &BlockPos) {
        let block = world.get_block(pos);
        if block.id != BlockId::SCULK_CATALYST {
            return;
        }
        let state = world.get_block_state(pos);
        let mut props = SculkCatalystLikeProperties::from_state_id(state.id);
        if !props.bloom {
            props.bloom = true;
            world.set_block_state(
                pos,
                props.to_state_id(block),
                BlockFlags::NOTIFY_ALL,
            );
            world.schedule_block_tick(block, *pos, 8, TickPriority::Normal);
        }
        world.play_sound(
            Sound::BlockSculkCatalystBloom,
            SoundCategory::Blocks,
            &pos.to_f64(),
        );
    }
}

impl BlockBehaviour for SculkCatalystBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        let be = crate::block::entities::sculk_catalyst::SculkCatalystBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(be));
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = SculkCatalystLikeProperties::default(args.block);
        props.bloom = false;
        props.to_state_id(args.block)
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        let mut props = SculkCatalystLikeProperties::from_state_id(state.id);
        if props.bloom {
            props.bloom = false;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }
}
