use pumpkin_data::{Block, BlockDirection, entity::EntityType, world::WorldEvent};
use pumpkin_world::{BlockStateId, world::BlockFlags};

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockMetadata, OnPlaceArgs, PlacedArgs,
        blocks::skull_block::SkullBlock,
    },
    entity::{Entity, boss::wither::WitherEntity},
};

pub struct WitherSkeletonSkullBlock;

impl BlockMetadata for WitherSkeletonSkullBlock {
    fn ids() -> Box<[u16]> {
        [Block::WITHER_SKELETON_SKULL.id].into()
    }
}

impl BlockBehaviour for WitherSkeletonSkullBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        SkullBlock::on_place(&SkullBlock, args)
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let world = args.world;
            let pos = args.position;

            let is_soul_block =
                |block: &Block| block == &Block::SOUL_SAND || block == &Block::SOUL_SOIL;
            let is_skull = |block: &Block| block == &Block::WITHER_SKELETON_SKULL;

            for dir in [BlockDirection::North, BlockDirection::West] {
                let opposite = dir.opposite();
                for offset in 0..3 {
                    let center_skull_pos = match offset {
                        0 => *pos,
                        1 => pos.offset(opposite.to_offset()),
                        2 => pos.offset(dir.to_offset()),
                        _ => unreachable!(),
                    };

                    let top_middle = center_skull_pos.down();
                    let base = top_middle.down();
                    let arm1 = top_middle.offset(dir.to_offset());
                    let arm2 = top_middle.offset(opposite.to_offset());
                    let skull1_pos = arm1.up();
                    let skull2_pos = arm2.up();

                    if is_soul_block(world.get_block(&top_middle).await)
                        && is_soul_block(world.get_block(&base).await)
                        && is_soul_block(world.get_block(&arm1).await)
                        && is_soul_block(world.get_block(&arm2).await)
                        && is_skull(world.get_block(&center_skull_pos).await)
                        && is_skull(world.get_block(&skull1_pos).await)
                        && is_skull(world.get_block(&skull2_pos).await)
                    {
                        let pattern = [
                            center_skull_pos,
                            skull1_pos,
                            skull2_pos,
                            top_middle,
                            arm1,
                            arm2,
                            base,
                        ];

                        for p in pattern {
                            world
                                .set_block_state(
                                    &p,
                                    Block::AIR.default_state.id,
                                    BlockFlags::NOTIFY_ALL,
                                )
                                .await;
                            world
                                .sync_world_event(
                                    WorldEvent::BlockBroken,
                                    p,
                                    Block::SOUL_SAND.default_state.id.into(),
                                )
                                .await;
                        }

                        let entity = Entity::new(
                            world.clone(),
                            top_middle.to_centered_f64(),
                            &EntityType::WITHER,
                        );
                        let wither = WitherEntity::new(entity).await;
                        world.spawn_entity(wither).await;
                        return;
                    }
                }
            }
        })
    }
}
