//! Entity pushes on a player, one path for Java and Bedrock.
//!
//! The server pushes every player like vanilla. Java clients predict the same pushes,
//! Bedrock clients don't predict entity pushes, so they get them as `SetActorMotion`.

use std::sync::atomic::Ordering;

use pumpkin_util::math::vector3::Vector3;

use super::Player;
use crate::entity::EntityBase;

/// Most ticks a Bedrock motion and the answering `PlayerAuthInput` take together.
const MAX_ROUND_TRIP_TICKS: u32 = 9;
const RESIDUAL_HISTORY: usize = MAX_ROUND_TRIP_TICKS as usize + 1;

/// Push velocity a Bedrock client carries from the server's push motions.
///
/// `SetActorMotion` replaces the velocity with a ping old `PlayerAuthInput.delta`, so the
/// push part in that delta is swapped for the current one. Pushes still in flight count
/// too, so every tick can send its own push like Java applies it.
#[derive(Default)]
pub struct PushResidual {
    /// Pushes sent so far, braked to the current tick.
    carried: Vector3<f64>,
    /// `carried` at the start of each recent tick, before that tick's push.
    history: [Vector3<f64>; RESIDUAL_HISTORY],
    now: usize,
}

impl PushResidual {
    /// One tick of the client's horizontal friction.
    pub fn tick(&mut self, friction: f64) {
        self.carried = Vector3::new(self.carried.x * friction, 0.0, self.carried.z * friction);
        self.now = (self.now + 1) % RESIDUAL_HISTORY;
        self.history[self.now] = self.carried;
    }

    /// Motion that adds `push` to the client's velocity. `lookback` is how many ticks
    /// older the push part of `delta` is.
    fn layer(&mut self, delta: Vector3<f64>, push: Vector3<f64>, lookback: usize) -> Vector3<f64> {
        let lookback = lookback.min(RESIDUAL_HISTORY - 1);
        let in_delta = self.history[(self.now + RESIDUAL_HISTORY - lookback) % RESIDUAL_HISTORY];
        let motion = delta - in_delta + self.carried + push;
        self.carried += push;
        motion
    }

    /// Another own motion replaced the client's velocity.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Ticks a Bedrock motion and the answering `PlayerAuthInput` take together.
/// `ping_ms == 0` is unsampled and keeps the cap.
fn bedrock_round_trip_ticks(ping_ms: u32) -> u32 {
    if ping_ms == 0 {
        return MAX_ROUND_TRIP_TICKS;
    }
    ping_ms.div_ceil(50).clamp(1, MAX_ROUND_TRIP_TICKS)
}

impl Player {
    /// Vanilla `LivingEntity.pushEntities` for a player, run after all player ticks.
    pub fn push_entities(&self) {
        let living = &self.living_entity;
        if !living.dead.load(Ordering::Relaxed) && living.health.load() > 0.0 {
            living.push_entities(self);
        }
    }

    /// Server `travel` friction, which the Bedrock client applies to pushes it carries.
    pub(crate) fn tick_push_residual(&self, friction: f64) {
        if let Some(client) = self.client.bedrock() {
            client
                .push_residual
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .tick(friction);
        }
    }

    /// Velocity part of vanilla `ServerEntity.sendChanges`. Runs after the entity pass,
    /// so this tick's knockback goes out before the next packets and friction.
    pub fn send_velocity_changes(&self) {
        let entity = self.get_entity();
        if entity.sync_velocity.swap(false, Ordering::SeqCst) {
            // The server velocity already holds the pushes
            entity.push_impulse.store(Vector3::default());
            entity.velocity_dirty.store(false, Ordering::SeqCst);
            self.send_hurt_velocity();
            return;
        }
        if entity.velocity_dirty.swap(false, Ordering::SeqCst)
            && entity.velocity_changed_since_sent()
        {
            // Vanilla `needsSync`: watchers only, Java clients predict their own pushes
            entity.send_velocity_to_watchers();
        }
        let Some(client) = self.client.bedrock() else {
            entity.push_impulse.store(Vector3::default());
            return;
        };
        // `SetActorMotion` replaces velocity, there is no add-impulse packet. Until an
        // own motion like knockback shows up in the deltas, a push would overwrite it,
        // so pushes wait at full size.
        let round_trip = bedrock_round_trip_ticks(self.ping.load(Ordering::Relaxed));
        let since_motion = self
            .world()
            .get_world_age()
            .saturating_sub(client.own_motion_age.load());
        let input_caught_up = client.input_tick.load(Ordering::Relaxed)
            > client.own_motion_tick.load(Ordering::Relaxed);
        if since_motion <= i64::from(round_trip) || !input_caught_up {
            return;
        }
        let push = entity.push_impulse.swap(Vector3::default());
        if push == Vector3::default() {
            return;
        }
        // The delta left half a round trip ago, the motion lands half a round trip later
        let lookback = round_trip as usize - 1;
        let motion = client
            .push_residual
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .layer(client.client_delta.load(), push, lookback);
        self.send_own_motion(motion);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUSH: f64 = 0.05;
    const PUSH_TICKS: usize = 20;
    const TICKS: usize = 300;

    /// Java client positions: pushes add to its own velocity after each move.
    fn java_positions(friction: f64) -> Vec<f64> {
        let (mut v, mut x) = (0.0, 0.0);
        let mut positions = Vec::new();
        for t in 0..TICKS {
            x += v;
            v *= friction;
            if t < PUSH_TICKS {
                v += PUSH;
            }
            positions.push(x);
        }
        positions
    }

    /// Bedrock client `one_way` ticks away that walks with `walk` and only gets pushes
    /// as motions replacing its velocity. Returns how far the pushes moved it, per tick.
    fn bedrock_push_positions(friction: f64, one_way: usize, walk: f64) -> Vec<f64> {
        let walk_speed = walk / (1.0 - friction);
        let round_trip = bedrock_round_trip_ticks(one_way as u32 * 100);
        let mut residual = PushResidual::default();
        let (mut v, mut x) = (walk_speed, 0.0);
        let mut deltas = Vec::new();
        let mut positions = Vec::new();
        let mut inbox = vec![None; TICKS + one_way];
        for t in 0..TICKS {
            if let Some(motion) = inbox[t] {
                v = motion;
            }
            x += v;
            v = v * friction + walk;
            deltas.push(v);
            positions.push(x - walk_speed * (t + 1) as f64);

            residual.tick(friction);
            if t < PUSH_TICKS {
                let delta = t.checked_sub(one_way).map_or(walk_speed, |i| deltas[i]);
                let push = Vector3::new(PUSH, 0.0, 0.0);
                let motion =
                    residual.layer(Vector3::new(delta, 0.0, 0.0), push, round_trip as usize - 1);
                inbox[t + one_way] = Some(motion.x);
            }
        }
        positions
    }

    #[test]
    fn bedrock_round_trip_uses_ping_and_caps() {
        assert_eq!(bedrock_round_trip_ticks(0), 9);
        assert_eq!(bedrock_round_trip_ticks(1), 1);
        assert_eq!(bedrock_round_trip_ticks(50), 1);
        assert_eq!(bedrock_round_trip_ticks(100), 2);
        assert_eq!(bedrock_round_trip_ticks(250), 5);
        assert_eq!(bedrock_round_trip_ticks(500), 9);
        assert_eq!(bedrock_round_trip_ticks(2000), 9);
    }

    #[test]
    fn bedrock_is_pushed_like_java_one_way_later() {
        // Ground, air and ice friction; 100 to 400 ms ping; standing and walking
        for friction in [0.546, 0.91, 0.8918] {
            let java = java_positions(friction);
            for one_way in 1..=4 {
                for walk in [0.0, 0.05] {
                    let bedrock = bedrock_push_positions(friction, one_way, walk);
                    for t in 0..TICKS - one_way {
                        let diff = bedrock[t + one_way - 1] - java[t];
                        assert!(
                            diff.abs() < 1e-9,
                            "friction {friction}, {one_way} ticks one way, walk {walk}, \
                             tick {t}: {diff}"
                        );
                    }
                }
            }
        }
    }
}
