use std::sync::Arc;

use crate::block::entities::mob_spawner::MobSpawnerBlockEntity;
use crate::entity::experience_orb::ExperienceOrbEntity;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;

use crate::block::{BlockBehaviour, BlockFuture, BrokenArgs, OnSyncedBlockEventArgs, PlacedArgs};

#[pumpkin_block("minecraft:spawner")]
pub struct SpawnerBlock;

impl BlockBehaviour for SpawnerBlock {
    fn on_synced_block_event<'a>(
        &'a self,
        _args: OnSyncedBlockEventArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let spawner_block_entity = MobSpawnerBlockEntity::new(*args.position, None);
            args.world.add_block_entity(Arc::new(spawner_block_entity));
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if args.player.gamemode.load() != GameMode::Creative {
                let xp_count = 15 + rand::random_range(0..15) + rand::random_range(0..15);
                ExperienceOrbEntity::spawn(args.world, args.position.to_centered_f64(), xp_count)
                    .await;
            }
        })
    }
}
