use std::sync::Arc;

use pumpkin_data::block_properties::SuspiciousSandLikeProperties;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockId, BlockStateId};
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::falling::FallingBlock;
use crate::block::entities::brushable_block::BrushableBlockBlockEntity;
use crate::block::{
    BlockBehaviour, BlockMetadata, GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs,
    PlacedArgs,
};
use crate::entity::falling::FallingEntity;

pub struct BrushableBlock;

impl BlockMetadata for BrushableBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::SUSPICIOUS_SAND, BlockId::SUSPICIOUS_GRAVEL].into()
    }
}

impl BrushableBlock {
    pub fn brush(
        world: &Arc<crate::world::World>,
        pos: &pumpkin_util::math::position::BlockPos,
        block: &Block,
    ) {
        let state = world.get_block_state(pos);
        let mut props = SuspiciousSandLikeProperties::from_state_id(state.id);

        let is_gravel = block.id == BlockId::SUSPICIOUS_GRAVEL;

        if let Some(be) = world.get_block_entity(pos)
            && let Some(brush_be) = be.as_any().downcast_ref::<BrushableBlockBlockEntity>()
        {
            let mut hits = brush_be
                .hits
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *hits += 1;

            if *hits >= 4 {
                let item = brush_be
                    .item
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take();
                if let Some(item_stack) = item {
                    world.drop_stack(pos, item_stack);
                }

                let target_block = if is_gravel {
                    &Block::GRAVEL
                } else {
                    &Block::SAND
                };

                world.set_block_state(pos, target_block.default_state.id, BlockFlags::NOTIFY_ALL);

                let sound = if is_gravel {
                    Sound::ItemBrushBrushingGravelComplete
                } else {
                    Sound::ItemBrushBrushingSandComplete
                };

                world.play_sound(sound, SoundCategory::Blocks, &pos.to_f64());
            } else {
                props.dusted = (*hits as u8).min(3);
                world.set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL);

                let sound = if is_gravel {
                    Sound::ItemBrushBrushingGravel
                } else {
                    Sound::ItemBrushBrushingSand
                };

                world.play_sound(sound, SoundCategory::Blocks, &pos.to_f64());
            }
        }
    }
}

impl BlockBehaviour for BrushableBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let props = SuspiciousSandLikeProperties::default(args.block);
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = BrushableBlockBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));
        FallingBlock::placed_with_delay(&args, 2);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        FallingBlock::get_state_for_neighbor_update_with_delay(&args, 2)
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if let Some(be) = args.world.get_block_entity(args.position)
            && let Some(brush_be) = be.as_any().downcast_ref::<BrushableBlockBlockEntity>()
        {
            let mut hits = brush_be
                .hits
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if *hits > 0 {
                *hits -= 1;
                let state = args.world.get_block_state(args.position);
                let mut props = SuspiciousSandLikeProperties::from_state_id(state.id);
                let new_dusted = (*hits as u8).min(3);
                if props.dusted != new_dusted {
                    props.dusted = new_dusted;
                    args.world.set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    );
                }
            }
        }

        let (block, state) = args.world.get_block_and_state(&args.position.down());
        if !FallingBlock::can_fall_through(state, block) || args.position.0.y < args.world.min_y {
            return;
        }
        let state = args.world.get_block_state(args.position);
        FallingEntity::replace_spawn(args.world, *args.position, state.id);
    }
}
