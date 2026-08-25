use std::sync::Arc;

use pumpkin_data::block_properties::{BlockProperties, SuspiciousSandLikeProperties};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockId, BlockStateId};
use pumpkin_world::world::BlockFlags;

use crate::block::entities::brushable_block::BrushableBlockBlockEntity;
use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, BrokenArgs, OnPlaceArgs, PlacedArgs,
};

pub struct BrushableBlock;

impl BlockMetadata for BrushableBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::SUSPICIOUS_SAND, BlockId::SUSPICIOUS_GRAVEL].into()
    }
}

impl BrushableBlock {
    pub async fn brush(
        world: &Arc<crate::world::World>,
        pos: &pumpkin_util::math::position::BlockPos,
        block: &Block,
    ) {
        let state = world.get_block_state(pos);
        let mut props = SuspiciousSandLikeProperties::from_state_id(state.id, block);

        let is_gravel = block.id == BlockId::SUSPICIOUS_GRAVEL;

        if let Some(be) = world.get_block_entity(pos)
            && let Some(brush_be) = be.as_any().downcast_ref::<BrushableBlockBlockEntity>()
        {
            let mut hits = brush_be.hits.lock().await;
            *hits += 1;

            if *hits >= 4 {
                let item = brush_be.item.lock().await.take();
                if let Some(item_stack) = item {
                    world.drop_stack(pos, item_stack).await;
                }

                let target_block = if is_gravel {
                    &Block::GRAVEL
                } else {
                    &Block::SAND
                };

                world
                    .set_block_state(pos, target_block.default_state.id, BlockFlags::NOTIFY_ALL)
                    .await;

                let sound = if is_gravel {
                    Sound::ItemBrushBrushingGravelComplete
                } else {
                    Sound::ItemBrushBrushingSandComplete
                };

                world.play_sound(sound, SoundCategory::Blocks, &pos.to_f64());
            } else {
                props.dusted = (*hits as u8).min(3);
                world
                    .set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                    .await;

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
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let props = SuspiciousSandLikeProperties::default(args.block);
            props.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let entity = BrushableBlockBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(be) = args.world.get_block_entity(args.position)
                && let Some(brush_be) = be.as_any().downcast_ref::<BrushableBlockBlockEntity>()
                && let Some(contained) = brush_be.item.lock().await.take()
            {
                args.world.drop_stack(args.position, contained).await;
            }
        })
    }
}
