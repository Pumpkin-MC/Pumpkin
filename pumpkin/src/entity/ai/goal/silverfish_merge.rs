//! Silverfish merge into stone (vanilla `SilverfishMergeWithStoneGoal` stand-in).
//! When idle, occasionally enter a nearby host block as an infested variant.

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::mob::Mob;
use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

const COOLDOWN: i32 = 40;
const SCAN: i32 = 2;

/// Host block → infested form (vanilla InfestedBlock hosts).
fn infested_for(host: &Block) -> Option<&'static Block> {
    if host.id == Block::STONE.id {
        Some(&Block::INFESTED_STONE)
    } else if host.id == Block::COBBLESTONE.id {
        Some(&Block::INFESTED_COBBLESTONE)
    } else if host.id == Block::STONE_BRICKS.id {
        Some(&Block::INFESTED_STONE_BRICKS)
    } else if host.id == Block::MOSSY_STONE_BRICKS.id {
        Some(&Block::INFESTED_MOSSY_STONE_BRICKS)
    } else if host.id == Block::CRACKED_STONE_BRICKS.id {
        Some(&Block::INFESTED_CRACKED_STONE_BRICKS)
    } else if host.id == Block::CHISELED_STONE_BRICKS.id {
        Some(&Block::INFESTED_CHISELED_STONE_BRICKS)
    } else if host.id == Block::DEEPSLATE.id {
        Some(&Block::INFESTED_DEEPSLATE)
    } else {
        None
    }
}

pub struct SilverfishMergeWithStoneGoal {
    cooldown: i32,
}

impl SilverfishMergeWithStoneGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            cooldown: to_goal_ticks(COOLDOWN),
        })
    }
}

impl Goal for SilverfishMergeWithStoneGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            // Only when not fighting.
            if mob.get_mob_entity().target.lock().await.is_some() {
                return false;
            }
            if self.cooldown > 0 {
                self.cooldown -= 1;
                return false;
            }
            // Vanilla rolls infrequently.
            if mob.get_random().random_range(0..10) != 0 {
                self.cooldown = to_goal_ticks(COOLDOWN);
                return false;
            }
            let world = mob.get_entity().world.load();
            if !world.level_info.load().game_rules.mob_griefing {
                self.cooldown = to_goal_ticks(COOLDOWN);
                return false;
            }
            true
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        // One-shot.
        Box::pin(async { false })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.cooldown = to_goal_ticks(COOLDOWN);
            let entity = mob.get_entity();
            let world = entity.world.load();
            let origin = entity.block_pos.load();

            for dy in -1..=1 {
                for dx in -SCAN..=SCAN {
                    for dz in -SCAN..=SCAN {
                        if dx == 0 && dy == 0 && dz == 0 {
                            continue;
                        }
                        let pos = BlockPos::new(origin.0.x + dx, origin.0.y + dy, origin.0.z + dz);
                        let block = world.get_block(&pos);
                        let Some(infested) = infested_for(block) else {
                            continue;
                        };
                        let _ = world
                            .set_block_state(
                                &pos,
                                infested.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                        // Enter the stone: remove the silverfish entity.
                        entity.remove().await;
                        return;
                    }
                }
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}
