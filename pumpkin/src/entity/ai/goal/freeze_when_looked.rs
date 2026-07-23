//! Creaking freeze: stop moving while a nearby player looks at us.

use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

const STARE_RANGE: f64 = 24.0;
const PLAYER_EYE: f64 = 1.62;

pub struct FreezeWhenLookedGoal {
    was_frozen: bool,
}

impl FreezeWhenLookedGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self { was_frozen: false })
    }

    fn player_looking_at(mob: &dyn Mob, player_yaw: f32, player_pitch: f32, player_eye: Vector3<f64>) -> bool {
        let mob_pos = mob.get_entity().pos.load();
        let mob_eye = Vector3::new(
            mob_pos.x,
            mob_pos.y + f64::from(mob.get_entity().entity_dimension.load().eye_height),
            mob_pos.z,
        );

        let pitch = player_pitch.to_radians();
        let yaw = -player_yaw.to_radians();
        let cos_pitch = pitch.cos();
        let look = Vector3::new(
            f64::from(yaw.sin() * cos_pitch),
            f64::from(-pitch.sin()),
            f64::from(yaw.cos() * cos_pitch),
        );

        let dx = mob_eye.x - player_eye.x;
        let dy = mob_eye.y - player_eye.y;
        let dz = mob_eye.z - player_eye.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist < 0.1 {
            return false;
        }
        let dir = Vector3::new(dx / dist, dy / dist, dz / dist);
        let dot = look.x * dir.x + look.y * dir.y + look.z * dir.z;
        dot > 1.0 - 0.025 / dist
    }
}

impl Goal for FreezeWhenLookedGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let world = mob.get_entity().world.load();
            let pos = mob.get_entity().pos.load();
            for player in world.get_nearby_players(pos, STARE_RANGE) {
                if player.is_spectator() {
                    continue;
                }
                let pe = player.get_entity();
                let ppos = pe.pos.load();
                let eye = Vector3::new(ppos.x, ppos.y + PLAYER_EYE, ppos.z);
                if Self::player_looking_at(mob, pe.yaw.load(), pe.pitch.load(), eye) {
                    return true;
                }
            }
            false
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        // Re-check stare each tick via can_start logic.
        Box::pin(async {
            let world = mob.get_entity().world.load();
            let pos = mob.get_entity().pos.load();
            for player in world.get_nearby_players(pos, STARE_RANGE) {
                if player.is_spectator() {
                    continue;
                }
                let pe = player.get_entity();
                let ppos = pe.pos.load();
                let eye = Vector3::new(ppos.x, ppos.y + PLAYER_EYE, ppos.z);
                if FreezeWhenLookedGoal::player_looking_at(
                    mob,
                    pe.yaw.load(),
                    pe.pitch.load(),
                    eye,
                ) {
                    return true;
                }
            }
            false
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            if !self.was_frozen {
                let world = mob.get_entity().world.load();
                world.play_sound(
                    Sound::EntityCreakingFreeze,
                    SoundCategory::Hostile,
                    &mob.get_entity().pos.load(),
                );
                self.was_frozen = true;
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.was_frozen = false;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            // Stay frozen.
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            // Zero horizontal velocity.
            let mut vel = mob.get_entity().velocity.load();
            vel.x = 0.0;
            vel.z = 0.0;
            mob.get_entity().set_velocity(vel);
        })
    }

    fn controls(&self) -> Controls {
        // Block MOVE + JUMP so melee cannot run while stared at.
        Controls::MOVE | Controls::JUMP
    }
}
