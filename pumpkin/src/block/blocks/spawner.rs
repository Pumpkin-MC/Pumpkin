use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::block::entities::mob_spawner::MobSpawnerBlockEntity;

use crate::block::{BlockBehaviour, PlacedArgs};

#[pumpkin_block("minecraft:spawner")]
pub struct SpawnerBlock;

#[async_trait]
impl BlockBehaviour for SpawnerBlock {
    async fn placed(&self, args: PlacedArgs<'_>) {
        let hopper_block_entity = MobSpawnerBlockEntity::new(*args.position);
        args.world
            .add_block_entity(Arc::new(hopper_block_entity))
            .await;
    }
}
