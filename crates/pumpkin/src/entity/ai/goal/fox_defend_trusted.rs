use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::mob::Mob;
use crate::entity::passive::fox::FoxEntity;
use pumpkin_data::sound::{Sound, SoundCategory};

/// `Fox.DefendTrustedTargetGoal`: retaliates against whoever last hurt a trusted entity, as long
/// as that hurt happened since the last time this goal reacted to one.
///
/// Vanilla's `TRUSTED_TARGET_SELECTOR` additionally requires the trusted entity's last-hurt
/// timestamp to be within 600 ticks; this codebase's `LivingEntity` doesn't record *when* the
/// trusted entity was last hurt (only its own `last_attacked_time`, a tick counter compared for
/// change-detection, mirroring how `RevengeGoal` already uses that same field for the
/// self-defense case) -- so the 600-tick recency window is dropped, matching how far
/// `RevengeGoal` already simplifies the equivalent vanilla check.
///
/// Delegates `should_continue`/`stop`/`controls` to a `TrackTargetGoal`, exactly like
/// `RevengeGoal` does -- without it, the base `NearestAttackableTargetGoal` vanilla builds this
/// on (`TargetGoal.canContinueToUse`, which drops the target on distance or long
/// unseen-time) has no analog here, and the fox would keep `Controls::TARGET` locked
/// permanently after its first defend trigger, forever blocking the land/fish target goals from
/// ever running again.
pub struct DefendTrustedTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    last_seen_attack_time: i32,
}

impl DefendTrustedTargetGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            track_target_goal: TrackTargetGoal::with_default(false),
            target: None,
            last_seen_attack_time: 0,
        })
    }
}

impl Goal for DefendTrustedTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() else {
                return false;
            };
            let world = mob.get_entity().world.load();

            for uuid in fox.trusted_uuids() {
                let Some(trusted) = world.get_entity_by_uuid(uuid) else {
                    continue;
                };
                let Some(trusted_living) = trusted.get_living_entity() else {
                    continue;
                };

                let attacked_time = trusted_living.last_attacked_time.load(Relaxed);
                if attacked_time == self.last_seen_attack_time {
                    continue;
                }

                let attacker_id = trusted_living.last_attacker_id.load(Relaxed);
                if attacker_id == 0 {
                    continue;
                }
                let Some(attacker) = world.get_entity_by_id(attacker_id) else {
                    continue;
                };
                let Some(attacker_living) = attacker.get_living_entity() else {
                    continue;
                };
                if !attacker_living.is_part_of_game() {
                    continue;
                }
                if fox.trusts(attacker.get_entity().entity_uuid) {
                    continue;
                }

                self.last_seen_attack_time = attacked_time;
                self.target = Some(attacker);
                return true;
            }

            false
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.track_target_goal.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.set_mob_target(self.target.clone()).await;
            self.track_target_goal.start(mob).await;

            if let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() {
                let world = mob.get_entity().world.load();
                let pos = mob.get_entity().pos.load();
                world.play_sound(Sound::EntityFoxAggro, SoundCategory::Neutral, &pos);
                fox.set_defending(true);
                fox.wake_up();
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
            self.track_target_goal.stop(mob).await;
        })
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}
