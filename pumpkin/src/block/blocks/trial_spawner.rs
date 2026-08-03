use std::sync::Arc;

use pumpkin_macros::pumpkin_block;

use crate::block::entities::trial_spawner::TrialSpawnerBlockEntity;
use crate::block::{BlockBehaviour, BlockFuture, PlacedArgs};

#[pumpkin_block("minecraft:trial_spawner")]
pub struct TrialSpawnerBlock;

impl BlockBehaviour for TrialSpawnerBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let block_entity = TrialSpawnerBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(block_entity));
        })
    }
}
