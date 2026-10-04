use std::sync::Arc;

use crate::block::entities::mob_spawner::MobSpawnerBlockEntity;
use crate::entity::experience_orb::ExperienceOrbEntity;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;

use crate::block::{BlockBehaviour, BrokenArgs, OnSyncedBlockEventArgs, PlacedArgs};

#[pumpkin_block("minecraft:spawner")]
pub struct SpawnerBlock;

impl BlockBehaviour for SpawnerBlock {
    fn on_synced_block_event(&self, _args: OnSyncedBlockEventArgs<'_>) -> bool {
        true
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let spawner_block_entity = MobSpawnerBlockEntity::new(*args.position, None);
            args.world.add_block_entity(Arc::new(spawner_block_entity));
        }
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        {
            if args.player.gamemode.load() != GameMode::Creative {
                let xp_count = 15 + rand::random_range(0..15) + rand::random_range(0..15);
                ExperienceOrbEntity::spawn(args.world, args.position.to_centered_f64(), xp_count);
            }
        }
    }
}
