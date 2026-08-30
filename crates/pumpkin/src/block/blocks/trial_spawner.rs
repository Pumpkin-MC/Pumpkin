use std::sync::Arc;

use pumpkin_macros::pumpkin_block;

use crate::block::entities::trial_spawner::TrialSpawnerBlockEntity;
use crate::block::{BlockBehaviour, PlacedArgs};

#[pumpkin_block("minecraft:trial_spawner")]
pub struct TrialSpawnerBlock;

impl BlockBehaviour for TrialSpawnerBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = TrialSpawnerBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));
    }
}
