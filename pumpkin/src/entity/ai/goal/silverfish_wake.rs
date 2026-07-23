//! Silverfish wake-up friends: when fighting, crack nearby infested blocks
//! (vanilla `SilverfishWakeUpFriendsGoal` stand-in).

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::mob::Mob;
use crate::entity::r#type::from_type;
use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use uuid::Uuid;

const SCAN_RADIUS: i32 = 5;
const COOLDOWN: i32 = 20;

const INFESTED: &[&Block] = &[
    &Block::INFESTED_STONE,
    &Block::INFESTED_COBBLESTONE,
    &Block::INFESTED_STONE_BRICKS,
    &Block::INFESTED_MOSSY_STONE_BRICKS,
    &Block::INFESTED_CRACKED_STONE_BRICKS,
    &Block::INFESTED_CHISELED_STONE_BRICKS,
    &Block::INFESTED_DEEPSLATE,
];

fn is_infested(block: &Block) -> bool {
    INFESTED.iter().any(|b| b.id == block.id)
}

pub struct SilverfishWakeFriendsGoal {
    /// Ticks remaining while actively searching / after a wake.
    look_ticks: i32,
}

impl SilverfishWakeFriendsGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self { look_ticks: 0 })
    }
}

impl Goal for SilverfishWakeFriendsGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            // Only when already angered / fighting.
            if mob.get_mob_entity().target.lock().await.is_none() {
                return false;
            }
            if self.look_ticks > 0 {
                return true;
            }
            // Kick off a short search window each fight.
            self.look_ticks = to_goal_ticks(COOLDOWN);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            self.look_ticks > 0 && mob.get_mob_entity().target.lock().await.is_some()
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            if self.look_ticks > 0 {
                self.look_ticks -= 1;
            }
            // Only attempt break every few ticks.
            if self.look_ticks % 4 != 0 {
                return;
            }

            let entity = mob.get_entity();
            let world = entity.world.load();
            let origin = entity.block_pos.load();

            for dy in -1..=2 {
                for dx in -SCAN_RADIUS..=SCAN_RADIUS {
                    for dz in -SCAN_RADIUS..=SCAN_RADIUS {
                        let pos =
                            BlockPos::new(origin.0.x + dx, origin.0.y + dy, origin.0.z + dz);
                        let block = world.get_block(&pos);
                        if !is_infested(block) {
                            continue;
                        }

                        // Break infested → air and spawn a friend.
                        // (InfestedBlock::broken needs a player cause; spawn manually.)
                        let _ = world
                            .set_block_state(
                                &pos,
                                Block::AIR.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                        let spawn_at = pos.0.to_f64() + Vector3::new(0.5, 0.0, 0.5);
                        let friend =
                            from_type(&EntityType::SILVERFISH, spawn_at, &world, Uuid::new_v4());
                        friend.get_entity().set_pos(spawn_at);
                        world.spawn_entity(friend).await;
                        world.play_sound(
                            Sound::EntitySilverfishAmbient,
                            SoundCategory::Hostile,
                            &spawn_at,
                        );
                        // One block per activation wave; JoinAnger picks up combat.
                        self.look_ticks = 0;
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
