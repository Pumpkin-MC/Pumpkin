use std::sync::Arc;

use pumpkin_macros::pumpkin_block;

use crate::block::{
    BlockBehaviour, BlockFuture, PlacedArgs, entities::mob_spawner::MobSpawnerBlockEntity,
};

#[pumpkin_block("minecraft:spawner")]
pub struct SpawnerBlock;

impl BlockBehaviour for SpawnerBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let hopper_block_entity = MobSpawnerBlockEntity::new(*args.position);
            args.world
                .add_block_entity(Arc::new(hopper_block_entity))
                .await;
        })
    }
}
