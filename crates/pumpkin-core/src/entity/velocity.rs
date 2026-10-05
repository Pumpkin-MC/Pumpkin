//! Applying, pushing and sending entity velocity.
//!
//! Impulses land in `velocity` and are flushed once per tick by [`Entity::flush_velocity`]:
//! `push_velocity` is vanilla `needsSync` (watchers only), `mark_hurt` is `hurtMarked`
//! (watchers and the entity's own client).

use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::entity::EntityType;
use pumpkin_protocol::bedrock::client::CSetActorMotion;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::CEntityVelocity;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

use super::{Entity, EntityBase, player::Player};
use crate::plugin::api::events::entity::{EntityKnockbackByEntityEvent, EntityKnockbackEvent};
use crate::plugin::api::events::player::player_velocity::PlayerVelocityEvent;
use crate::world::scoreboard::{CollisionRule, Scoreboard, Team};

impl Entity {
    pub fn set_velocity(&self, velocity: Vector3<f64>) {
        self.velocity.store(velocity);
        self.send_velocity();
    }

    pub fn add_velocity(&self, velocity: Vector3<f64>) {
        self.set_velocity(self.velocity.load() + velocity);
    }

    /// Vanilla `Entity.push(x, 0, z)`: sent with the next flush (`needsSync`).
    pub fn push_velocity(&self, x: f64, z: f64) {
        self.velocity
            .store(self.velocity.load() + Vector3::new(x, 0.0, z));
        self.velocity_dirty.store(true, Ordering::SeqCst);
    }

    /// Vanilla `markHurt`: the next flush also reaches the own client.
    pub fn mark_hurt(&self) {
        self.sync_velocity.store(true, Ordering::SeqCst);
    }

    /// Vanilla `LivingEntity.knockback`, sent with the next flush.
    ///
    /// Knockback resistance is a `LivingEntity` attribute, so callers modelling
    /// `LivingEntity.knockback` scale `strength` with `combat::knockback_after_resistance`
    /// first; callers modelling a raw `Entity.push` (the ender dragon) pass it unscaled.
    ///
    /// Fires `EntityKnockbackByEntityEvent` (with a `source`), then `EntityKnockbackEvent`.
    pub fn apply_knockback(
        &self,
        mut strength: f64,
        mut x: f64,
        mut z: f64,
        source: Option<&Self>,
    ) {
        let server = self.world.load().server.upgrade();

        // Plugin hook: fired even at zero strength, so plugins can add knockback. A
        // cancel carries over to `EntityKnockbackEvent`, which can undo it.
        let mut cancelled = false;
        if let (Some(server), Some(source)) = (&server, source) {
            let mut event =
                EntityKnockbackByEntityEvent::new(self.entity_id, source.entity_id, strength, x, z);
            server.plugin_manager.fire_blocking(server, &mut event);
            (strength, x, z, cancelled) = (event.force, event.x, event.z, event.cancelled);
        }
        // Vanilla: no effect at zero strength.
        if strength <= 0.0 {
            return;
        }

        while x.mul_add(x, z * z) < f64::from(1.0E-5f32) {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let push = Vector3::new(x, 0.0, z).normalize() * strength;
        let velocity = self.velocity.load();
        let target = Vector3::new(
            velocity.x / 2.0 - push.x,
            if self.on_ground.load(Ordering::Relaxed) {
                (velocity.y / 2.0 + strength).min(0.4)
            } else {
                velocity.y
            },
            velocity.z / 2.0 - push.z,
        );

        // Plugin hook: the added velocity, not the result.
        let mut knockback = target - velocity;
        if let Some(server) = &server {
            let mut event = EntityKnockbackEvent {
                entity_id: self.entity_id,
                hit_by_id: source.map(|source| source.entity_id),
                knockback,
                cancelled,
            };
            server.plugin_manager.fire_blocking(server, &mut event);
            (knockback, cancelled) = (event.knockback, event.cancelled);
        }
        if cancelled {
            return;
        }

        self.velocity.store(velocity + knockback);
        self.velocity_dirty.store(true, Ordering::SeqCst);
    }

    /// Once per tick: sends what `push_velocity`, `apply_knockback` and `mark_hurt` left.
    pub fn flush_velocity(&self, player: Option<&Player>) {
        let hurt = self.sync_velocity.swap(false, Ordering::SeqCst);
        if !self.velocity_dirty.swap(false, Ordering::SeqCst) && !hurt {
            return;
        }
        match player {
            Some(player) => player.sync_velocity(hurt),
            None => self.send_velocity_to_watchers(),
        }
    }

    /// Immediate send to watchers and, for a player, its own client.
    pub fn send_velocity(&self) {
        self.send_velocity_to_watchers();
        if self.entity_type == &EntityType::PLAYER
            && let Some(player) = self.world.load().get_player_by_id(self.entity_id)
        {
            player.send_own_velocity(self.velocity.load());
        }
    }

    /// Watchers only: the own client predicts its pushes.
    pub fn send_velocity_to_watchers(&self) {
        let velocity = self.velocity.load();
        self.last_sent_velocity.store(velocity);
        self.world.load().send_to_tracking_players_editioned(
            self,
            &CEntityVelocity::new(self.entity_id.into(), velocity),
            &CSetActorMotion {
                target_runtime_id: VarULong(self.entity_id as u64),
                motion: velocity.to_f32_lossy(),
                tick: VarULong(0),
            },
        );
    }
}

impl Player {
    /// Fires `PlayerVelocityEvent`, then sends right away.
    pub fn set_velocity(&self, velocity: Vector3<f64>) {
        if let Some(velocity) = self.fire_velocity_event(velocity) {
            self.living_entity.entity.set_velocity(velocity);
        }
    }

    /// Plugin knockback: vanilla `LivingEntity.knockback` without a source.
    pub fn apply_knockback(&self, strength: f64, x: f64, z: f64) {
        let entity = &self.living_entity.entity;
        entity.apply_knockback(strength, x, z, None);
        entity.mark_hurt();
    }

    /// `PlayerVelocityEvent`: the velocity to send, `None` if cancelled.
    fn fire_velocity_event(&self, velocity: Vector3<f64>) -> Option<Vector3<f64>> {
        let world = self.world();
        let Some(server) = world.server.upgrade() else {
            return Some(velocity);
        };
        if !server.plugin_manager.has_handlers::<PlayerVelocityEvent>() {
            return Some(velocity);
        }
        let Some(player) = world.get_player_by_uuid(self.gameprofile.id) else {
            return Some(velocity);
        };
        let mut event = PlayerVelocityEvent {
            player,
            velocity,
            cancelled: false,
        };
        server.plugin_manager.fire_blocking(&server, &mut event);
        (!event.cancelled).then_some(event.velocity)
    }

    /// Flush of pushed or knocked back velocity. `hurt` is vanilla `hurtMarked`.
    /// Cancelling the event sends nothing.
    fn sync_velocity(&self, hurt: bool) {
        let entity = &self.living_entity.entity;
        let velocity = entity.velocity.load();
        let Some(new_velocity) = self.fire_velocity_event(velocity) else {
            return;
        };
        let changed = new_velocity != velocity;
        if changed {
            entity.velocity.store(new_velocity);
        }
        entity.send_velocity_to_watchers();
        // Vanilla sends pushes to watchers only. Own client pushes: see `client_push`.
        if hurt || changed {
            self.send_own_velocity(new_velocity);
        }
    }

    /// Velocity to the own client. Bedrock: tagged with the last processed input tick.
    pub fn send_own_velocity(&self, velocity: Vector3<f64>) {
        let tick = self
            .client
            .bedrock()
            .map_or(0, |client| client.input_tick.load(Ordering::Relaxed));
        self.try_enqueue_packet_editioned(
            &CEntityVelocity::new(self.entity_id().into(), velocity),
            &CSetActorMotion {
                target_runtime_id: VarULong(self.entity_id() as u64),
                motion: velocity.to_f32_lossy(),
                tick: VarULong(tick),
            },
        );
    }
}

/// Vanilla `Entity.push(Entity)`: both sides get pushed apart.
pub fn push_apart(this: &(impl EntityBase + ?Sized), other: &dyn EntityBase) {
    let this_entity = this.get_entity();
    let other_entity = other.get_entity();

    if this_entity.no_physics.load(Ordering::Relaxed)
        || other_entity.no_physics.load(Ordering::Relaxed)
        || this_entity.has_passenger(other_entity.entity_id)
        || other_entity.has_passenger(this_entity.entity_id)
    {
        return;
    }

    let mut dx = other_entity.pos.load().x - this_entity.pos.load().x;
    let mut dz = other_entity.pos.load().z - this_entity.pos.load().z;
    let mut d = dx.abs().max(dz.abs());
    if d < f64::from(0.01f32) {
        return;
    }
    d = d.sqrt();
    dx /= d;
    dz /= d;
    let d2 = (1.0 / d).min(1.0);
    dx = dx * d2 * f64::from(0.05f32);
    dz = dz * d2 * f64::from(0.05f32);

    if !this_entity.has_passengers() && this.is_pushable() {
        this_entity.push_velocity(-dx, -dz);
    }
    if !other_entity.has_passengers() && other.is_pushable() {
        other_entity.push_velocity(dx, dz);
    }
}

/// Vanilla `Level.getPushableEntities`: everything in `area` that `pusher` may push.
pub fn pushable_entities(pusher: &dyn EntityBase, area: &BoundingBox) -> Vec<Arc<dyn EntityBase>> {
    let pusher_id = pusher.get_entity().entity_id;
    pusher
        .get_entity()
        .world
        .load()
        .get_all_at_box(area)
        .into_iter()
        .filter(|entity| {
            entity.get_entity().entity_id != pusher_id && pushable_by(pusher, entity.as_ref())
        })
        .collect()
}

/// Vanilla `EntitySelector.pushableBy`, plus the plugin `collides` switch.
pub fn pushable_by(pusher: &dyn EntityBase, pushed: &dyn EntityBase) -> bool {
    pusher
        .get_living_entity()
        .is_none_or(|living| living.collides.load(Ordering::Relaxed))
        && !pushed.is_spectator()
        && pushed.is_pushable()
        && is_push_allowed_by_teams(pusher, pushed)
}

/// Team part of `pushableBy`. Both teams come from one scoreboard: the custom Java
/// one of the pushed (else pushing) player, as that client predicts it.
fn is_push_allowed_by_teams(pusher: &dyn EntityBase, pushed: &dyn EntityBase) -> bool {
    let allowed = |scoreboard: &Scoreboard| {
        scoreboard.get_teams().is_empty()
            || is_allowed_by_team_rules(
                scoreboard.get_entity_team(&pusher.get_scoreboard_name()),
                scoreboard.get_entity_team(&pushed.get_scoreboard_name()),
            )
    };
    [pushed, pusher]
        .into_iter()
        .filter_map(EntityBase::get_player)
        .find_map(|player| player.with_java_scoreboard(allowed))
        .unwrap_or_else(|| {
            let world = pusher.get_entity().world.load();
            let scoreboard = world
                .scoreboard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            allowed(&scoreboard)
        })
}

fn is_allowed_by_team_rules(own_team: Option<&Team>, their_team: Option<&Team>) -> bool {
    let own_rule = own_team.map_or(CollisionRule::Always, |team| team.collision_rule);
    let their_rule = their_team.map_or(CollisionRule::Always, |team| team.collision_rule);

    if own_rule == CollisionRule::Never || their_rule == CollisionRule::Never {
        return false;
    }

    let same_team = own_team
        .zip(their_team)
        .is_some_and(|(own, their)| own.name == their.name);

    if (own_rule == CollisionRule::PushOwnTeam || their_rule == CollisionRule::PushOwnTeam)
        && same_team
    {
        return false;
    }

    (own_rule != CollisionRule::PushOtherTeams && their_rule != CollisionRule::PushOtherTeams)
        || same_team
}
