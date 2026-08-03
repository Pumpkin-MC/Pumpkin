// Port of net.minecraft.world.entity.monster.warden.{Warden, WardenAi} plus
// net.minecraft.world.entity.ai.behavior.warden.SonicBoom, using the anger-system port in
// `warden_anger.rs` and the GameEvent/vibration engine in `crate::world::game_event`.
//
// Pumpkin has no Brain/Memory/Activity system (see PARITY.md), so `WardenAi`'s activity
// ladder (EMERGE/DIG/ROAR/FIGHT/INVESTIGATE/SNIFF/IDLE, each a `Brain` memory-gated
// `Behavior` list) is not portable as-is. This implementation instead keeps the anger
// state and cooldowns as plain fields on `WardenEntity`, ticked directly from `mob_tick`,
// and drives targeting/attacking through the existing Goal system. Concretely ported:
// anger accumulation from vibrations and melee hits, vibration-driven investigation
// (via `mob_entity.target`), and the sonic boom ranged attack. Explicitly NOT ported,
// with reasons:
//
// - Pose-driven emerge/dig/roar animation states (Warden.java lines 275-282, 346-365,
//   517-521, `AnimationState` fields) and the corresponding `Pose::EMERGING`/`DIGGING`/
//   `ROARING` client sync: Pumpkin's `Entity` has no `Pose` enum or per-pose client
//   animation-state channel at all (unlike vanilla's `SynchedEntityData` `DATA_POSE`).
//   Digging in/out of the ground on first spawn (`EntitySpawnReason::TRIGGERED` only,
//   i.e. sculk shrieker-summoned Wardens) and its temporary invulnerability
//   (`isInvulnerableTo`, `ignoreExplosion`) are therefore not implemented; a Warden here
//   spawns immediately usable, with no digging/emerging grace period.
// - `doPush`-triggered touch anger (Warden.java lines 528-537): Pumpkin's `EntityBase`
//   has no entity-entity collision/push hook to attach this to at all (grepped
//   `pumpkin/src/entity/{mod,living}.rs` and `mob/mod.rs` — no `on_push`/`do_push`).
//   `touch_cooldown` below is kept as a field and constant for when such a hook exists,
//   but is never set.
// - The uuid-side-map anger persistence across chunk unload (`AngerManagement`'s
//   `angerByUuid`) and `addAdditionalSaveData`/`readAdditionalSaveData` NBT round-trip:
//   see `warden_anger.rs` module doc comment.
// - Projectile-specific anger scaling (Warden.java lines 618-635: `RECENT_PROJECTILE`
//   30-block/100-tick logic distinguishing a projectile's owner from the projectile
//   itself) — `GameEventContext` here only carries one `source_entity`, with no separate
//   projectile-owner field the way vanilla's `onReceiveVibration` does, so every
//   vibration is treated as the non-projectile path (`increaseAngerAt(sourceEntity,
//   DEFAULT_ANGER)`). Revisit once a projectile-owner field is added to
//   `GameEventContext`.
// - World-border containment checks (`canReceiveVibration`'s
//   `level.getWorldBorder().isWithinBounds(pos)`, `canTargetEntity`'s equivalent): omitted
//   for simplicity, not architecturally blocked.
// - `WardenAi.setDisturbanceLocation`'s two-phase "walk to a bare location, then decide"
//   investigation: there is no location-only walk-goal wired here, so a heard/seen
//   disturbance becomes an immediate soft attack-target instead (`set_disturbance` below).

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;

use crate::entity::mob::warden_anger::{self, AngerLevel, AngerManagement};
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};
use crate::world::World;
use crate::world::game_event::{
    GameEventContext, GameEventFuture, GameEventListener, PositionSource,
};

pub struct WardenEntity {
    pub mob_entity: MobEntity,
    anger: AsyncMutex<AngerManagement>,
    vibration_cooldown: AtomicI32,
    /// Never set (see module doc comment on the missing push hook); kept for when one exists.
    touch_cooldown: AtomicI32,
    sonic_boom_cooldown: AtomicI32,
    listener_registered: AtomicBool,
    listener: std::sync::Mutex<Option<Arc<WardenVibrationListener>>>,
}

impl WardenEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let warden = Self {
            mob_entity,
            anger: AsyncMutex::new(AngerManagement::new()),
            vibration_cooldown: AtomicI32::new(0),
            touch_cooldown: AtomicI32::new(0),
            sonic_boom_cooldown: AtomicI32::new(0),
            listener_registered: AtomicBool::new(false),
            listener: std::sync::Mutex::new(None),
        };
        let mob_arc = Arc::new(warden);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
        goal_selector.add_goal(0, Box::new(SwimGoal::default()));
        goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.2, true)));
        goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(0.5)));
        goal_selector.add_goal(
            6,
            LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
        );
        goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));
        drop(goal_selector);

        let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
        target_selector.add_goal(
            1,
            ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
        );
        drop(target_selector);

        let self_uuid = mob_arc.mob_entity.living_entity.entity.entity_uuid;
        let listener = Arc::new(WardenVibrationListener {
            warden: Arc::downgrade(&mob_arc),
            uuid: self_uuid,
        });
        *mob_arc.listener.lock().unwrap() = Some(listener);

        mob_arc
    }

    /// `Warden.canTargetEntity`, minus the world-border check (see module doc comment).
    #[must_use]
    fn can_target_entity(&self, entity: &dyn EntityBase) -> bool {
        let base = entity.get_entity();
        if base.entity_uuid == self.mob_entity.living_entity.entity.entity_uuid {
            return false;
        }
        if !base.is_alive() {
            return false;
        }
        if base.entity_type.id == EntityType::ARMOR_STAND.id
            || base.entity_type.id == EntityType::WARDEN.id
        {
            return false;
        }
        if let Some(player) = entity.get_player() {
            let gamemode = player.gamemode.load();
            if gamemode == GameMode::Creative || gamemode == GameMode::Spectator {
                return false;
            }
        }
        true
    }

    /// `Warden.increaseAngerAt(entity, amount, playSound)`.
    async fn increase_anger_at(&self, entity: &Arc<dyn EntityBase>, amount: i32, play_sound: bool) {
        if self.mob_entity.is_no_ai() || !self.can_target_entity(entity.as_ref()) {
            return;
        }
        let is_player = entity.get_player().is_some();
        let uuid = entity.get_entity().entity_uuid;
        let new_anger = {
            let mut anger = self.anger.lock().await;
            anger.increase_anger(uuid, is_player, amount)
        };

        let maybe_switch_target = {
            let target = self.mob_entity.target.lock().await;
            !matches!(&*target, Some(t) if t.get_player().is_some())
        };
        if is_player && maybe_switch_target && AngerLevel::by_anger(new_anger).is_angry() {
            self.set_attack_target(entity.clone()).await;
        }

        if play_sound {
            self.play_listening_sound().await;
        }
    }

    async fn play_listening_sound(&self) {
        let anger = self.active_anger().await;
        let sound = match AngerLevel::by_anger(anger) {
            AngerLevel::Angry | AngerLevel::Agitated => Sound::EntityWardenListeningAngry,
            AngerLevel::Calm => Sound::EntityWardenListening,
        };
        let pos = self.mob_entity.living_entity.entity.pos.load();
        self.mob_entity
            .living_entity
            .entity
            .world
            .load()
            .play_sound_fine(sound, SoundCategory::Hostile, &pos, 10.0, 1.0);
    }

    /// `Warden.getActiveAnger` against the current attack target, if any.
    async fn active_anger(&self) -> i32 {
        let target_uuid = self
            .mob_entity
            .target
            .lock()
            .await
            .as_ref()
            .map(|t| t.get_entity().entity_uuid);
        self.anger.lock().await.active_anger(target_uuid)
    }

    #[must_use]
    pub async fn get_anger_level(&self) -> AngerLevel {
        AngerLevel::by_anger(self.active_anger().await)
    }

    /// `Warden.setAttackTarget`; also resets the sonic-boom cooldown like vanilla does
    /// (`SonicBoom.setCooldown(this, 200)` — vanilla's `TIME_TO_USE_MELEE_UNTIL_SONIC_BOOM`).
    async fn set_attack_target(&self, target: Arc<dyn EntityBase>) {
        *self.mob_entity.target.lock().await = Some(target);
        self.sonic_boom_cooldown
            .store(warden_anger::SONIC_BOOM_COOLDOWN_TICKS, Ordering::Relaxed);
    }

    /// `WardenAi.setDisturbanceLocation`, redirected onto the existing goal system by
    /// setting the current attack target directly to the disturbance's source entity (see
    /// module doc comment on why the location-only walk phase is skipped).
    async fn set_disturbance(&self, source: &Arc<dyn EntityBase>) {
        let already_angry = self.get_anger_level().await.is_angry();
        let has_target = self.mob_entity.target.lock().await.is_some();
        if !already_angry && !has_target {
            *self.mob_entity.target.lock().await = Some(source.clone());
        }
    }

    /// `VibrationSystem.User.onReceiveVibration` (`Warden.VibrationUser`), minus the
    /// projectile-owner branch (see module doc comment).
    async fn on_vibration(&self, source_entity: Option<Arc<dyn EntityBase>>) {
        if self.mob_entity.living_entity.dead.load(Ordering::Relaxed) {
            return;
        }
        self.vibration_cooldown
            .store(warden_anger::VIBRATION_COOLDOWN_TICKS, Ordering::Relaxed);
        let pos = self.mob_entity.living_entity.entity.pos.load();
        self.mob_entity
            .living_entity
            .entity
            .world
            .load()
            .play_sound_fine(
                Sound::EntityWardenTendrilClicks,
                SoundCategory::Hostile,
                &pos,
                5.0,
                1.0,
            );

        if let Some(source) = source_entity {
            self.increase_anger_at(&source, warden_anger::DEFAULT_ANGER, false)
                .await;
            if !self.get_anger_level().await.is_angry() {
                self.set_disturbance(&source).await;
            }
        }
    }

    /// `Warden.doHurtTarget` cooldown reset (called on a successful melee attack).
    fn on_successful_melee_attack(&self) {
        self.sonic_boom_cooldown
            .store(warden_anger::SONIC_BOOM_COOLDOWN_TICKS, Ordering::Relaxed);
    }

    /// `SonicBoom` behavior: a ranged attack used once the current target is too far for
    /// melee but still within 15 (xz) / 20 (y) blocks, and off cooldown. Simplified to a
    /// single-tick effect (no `Behavior` start/tick/stop phase, no delayed warm-up sound —
    /// `SonicBoom.TICKS_BEFORE_PLAYING_SOUND` / `DURATION` windows collapse to immediate
    /// execution here).
    async fn try_sonic_boom(&self) {
        if !self.get_anger_level().await.is_angry() {
            return;
        }
        if self.sonic_boom_cooldown.load(Ordering::Relaxed) > 0 {
            return;
        }
        let target = self.mob_entity.target.lock().await.clone();
        let Some(target) = target else {
            return;
        };
        if !self.can_target_entity(target.as_ref()) {
            return;
        }

        let source = self.mob_entity.living_entity.entity.pos.load();
        let target_eye = target.get_eye_pos();
        let delta = target_eye - source;
        let distance_xz = delta.x.mul_add(delta.x, delta.z * delta.z).sqrt();
        if distance_xz > warden_anger::SONIC_BOOM_DISTANCE_XZ
            || delta.y.abs() > warden_anger::SONIC_BOOM_DISTANCE_Y
        {
            return;
        }

        self.sonic_boom_cooldown
            .store(warden_anger::SONIC_BOOM_COOLDOWN_TICKS, Ordering::Relaxed);

        let world = self.mob_entity.living_entity.entity.world.load();
        world.play_sound_fine(
            Sound::EntityWardenSonicBoom,
            SoundCategory::Hostile,
            &source,
            3.0,
            1.0,
        );

        let self_entity: &dyn EntityBase = self;
        let damaged = target
            .damage_with_context(
                target.as_ref(),
                warden_anger::SONIC_BOOM_DAMAGE,
                DamageType::SONIC_BOOM,
                Some(source),
                Some(self_entity),
                Some(self_entity),
            )
            .await;
        if damaged {
            let normalize = if delta.length_squared() > 1.0e-6 {
                delta.normalize()
            } else {
                Vector3::new(0.0, 0.0, 0.0)
            };
            target.get_entity().add_velocity(Vector3::new(
                normalize.x * warden_anger::SONIC_BOOM_KNOCKBACK_HORIZONTAL,
                normalize.y * warden_anger::SONIC_BOOM_KNOCKBACK_VERTICAL,
                normalize.z * warden_anger::SONIC_BOOM_KNOCKBACK_HORIZONTAL,
            ));
        }
    }

    async fn register_listener_once(&self) {
        if self.listener_registered.swap(true, Ordering::Relaxed) {
            return;
        }
        let listener = self.listener.lock().unwrap().clone();
        let Some(listener) = listener else {
            return;
        };
        let world = self.mob_entity.living_entity.entity.world.load();
        world.register_game_event_listener(listener).await;
    }
}

impl NBTStorage for WardenEntity {}

impl Mob for WardenEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.register_listener_once().await;

            if self.vibration_cooldown.load(Ordering::Relaxed) > 0 {
                self.vibration_cooldown.fetch_sub(1, Ordering::Relaxed);
            }
            if self.touch_cooldown.load(Ordering::Relaxed) > 0 {
                self.touch_cooldown.fetch_sub(1, Ordering::Relaxed);
            }
            if self.sonic_boom_cooldown.load(Ordering::Relaxed) > 0 {
                self.sonic_boom_cooldown.fetch_sub(1, Ordering::Relaxed);
            }

            let age = self
                .mob_entity
                .living_entity
                .entity
                .age
                .load(Ordering::Relaxed);
            if age % warden_anger::ANGERMANAGEMENT_TICK_DELAY == 0 {
                let self_uuid = self.mob_entity.living_entity.entity.entity_uuid;
                let world = self.mob_entity.living_entity.entity.world.load_full();
                let valid = |uuid: Uuid| {
                    uuid != self_uuid
                        && world
                            .get_entity_by_uuid(uuid)
                            .is_some_and(|e| self.can_target_entity(e.as_ref()))
                };
                self.anger.lock().await.tick(valid);
            }

            self.try_sonic_boom().await;
        })
    }

    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.mob_entity.is_no_ai() {
                return;
            }
            let Some(source) = source else {
                return;
            };
            let world = self.mob_entity.living_entity.entity.world.load();
            let Some(attacker) = world.get_entity_by_uuid(source.get_entity().entity_uuid) else {
                return;
            };
            drop(world);

            self.increase_anger_at(
                &attacker,
                AngerLevel::Angry.minimum_anger() + warden_anger::ON_HURT_ANGER_BOOST,
                false,
            )
            .await;

            let has_target = self.mob_entity.target.lock().await.is_some();
            if !has_target {
                let self_pos = self.mob_entity.living_entity.entity.pos.load();
                let attacker_pos = attacker.get_entity().pos.load();
                let close = (self_pos - attacker_pos).length_squared() <= 25.0;
                if close {
                    self.set_attack_target(attacker).await;
                }
            }
        })
    }

    fn on_successful_attack<'a>(&'a self, _target: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.on_successful_melee_attack();
        })
    }
}

/// `Warden.VibrationUser` (`VibrationSystem.User`), collapsed onto the simpler
/// `GameEventListener` trait this project's engine actually has (see `game_event/mod.rs`).
struct WardenVibrationListener {
    warden: Weak<WardenEntity>,
    uuid: Uuid,
}

impl GameEventListener for WardenVibrationListener {
    fn listener_source(&self) -> PositionSource {
        PositionSource::Entity(self.uuid)
    }

    fn listener_radius(&self) -> i32 {
        // Warden.VibrationUser.GAME_EVENT_LISTENER_RANGE
        16
    }

    fn handle_game_event<'a>(
        &'a self,
        _world: &'a Arc<World>,
        event: &'a GameEvent,
        context: &'a GameEventContext,
        _source_position: Vector3<f64>,
    ) -> GameEventFuture<'a> {
        Box::pin(async move {
            let Some(warden) = self.warden.upgrade() else {
                return false;
            };
            if !warden_can_listen(event) {
                return false;
            }
            if warden.mob_entity.is_no_ai()
                || warden.mob_entity.living_entity.dead.load(Ordering::Relaxed)
                || warden.vibration_cooldown.load(Ordering::Relaxed) > 0
            {
                return false;
            }
            // Suppresses vibrations sourced from an entity Warden would refuse to target
            // (e.g. another Warden, or a creative/spectator player) — mirrors
            // `canReceiveVibration`'s `!(context.sourceEntity() instanceof LivingEntity &&
            // !canTargetEntity(...))`.
            if let Some(source) = &context.source_entity
                && !warden.can_target_entity(source.as_ref())
            {
                return false;
            }
            warden.on_vibration(context.source_entity.clone()).await;
            true
        })
    }
}

/// `GameEventTags.WARDEN_CAN_LISTEN` (`pumpkin-data/src/generated/tag.rs`,
/// `GameEvent::MINECRAFT_WARDEN_CAN_LISTEN`). `GameEvent` derives no `Taggable` impl (see
/// `game_event/vibration.rs` doc comment on why it can't hold ad hoc derives), so the tag's
/// event list is matched directly here instead of via `has_tag`.
const fn warden_can_listen(event: &GameEvent) -> bool {
    matches!(
        event,
        GameEvent::BlockAttach
            | GameEvent::BlockChange
            | GameEvent::BlockClose
            | GameEvent::BlockDestroy
            | GameEvent::BlockDetach
            | GameEvent::BlockOpen
            | GameEvent::BlockPlace
            | GameEvent::BlockActivate
            | GameEvent::BlockDeactivate
            | GameEvent::Bounce
            | GameEvent::ContainerClose
            | GameEvent::ContainerOpen
            | GameEvent::Drink
            | GameEvent::Eat
            | GameEvent::ElytraGlide
            | GameEvent::EntityDamage
            | GameEvent::EntityDie
            | GameEvent::EntityDismount
            | GameEvent::EntityInteract
            | GameEvent::EntityMount
            | GameEvent::EntityPlace
            | GameEvent::EntityAction
            | GameEvent::Equip
            | GameEvent::Explode
            | GameEvent::FluidPickup
            | GameEvent::FluidPlace
            | GameEvent::HitGround
            | GameEvent::InstrumentPlay
            | GameEvent::ItemInteractFinish
            | GameEvent::LightningStrike
            | GameEvent::NoteBlockPlay
            | GameEvent::PrimeFuse
            | GameEvent::ProjectileLand
            | GameEvent::ProjectileShoot
            | GameEvent::Shear
            | GameEvent::Splash
            | GameEvent::Step
            | GameEvent::Swim
            | GameEvent::Teleport
            | GameEvent::Unequip
    )
}

#[cfg(test)]
mod tests {
    use super::warden_can_listen;
    use pumpkin_data::game_event::GameEvent;

    #[test]
    fn warden_can_listen_matches_the_generated_tag_membership() {
        assert!(warden_can_listen(&GameEvent::BlockDestroy));
        assert!(warden_can_listen(&GameEvent::Step));
        assert!(warden_can_listen(&GameEvent::NoteBlockPlay));
        assert!(!warden_can_listen(&GameEvent::JukeboxPlay));
        assert!(!warden_can_listen(&GameEvent::SculkSensorTendrilsClicking));
    }
}
