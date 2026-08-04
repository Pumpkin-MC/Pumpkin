use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};
use crate::world::game_event::{GameEventContext, emit_game_event};

pub struct CreakingEntity {
    pub mob_entity: MobEntity,
    /// `HOME_POS`: position of the bound `CreakingHeartBlockEntity`, if any.
    home_pos: AtomicCell<Option<BlockPos>>,
    /// `IS_ACTIVE`.
    is_active: AtomicBool,
    /// `invulnerabilityAnimationRemainingTicks`. While positive, incoming damage that would
    /// otherwise be absorbed by the heart-bound gate is a strict no-op (double-hit within the
    /// same 8-tick window doesn't retrigger the resin-spread effect).
    invulnerability_ticks: AtomicI32,
}

impl CreakingEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let creaking = Self {
            mob_entity,
            home_pos: AtomicCell::new(None),
            is_active: AtomicBool::new(false),
            invulnerability_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(creaking);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    /// Vanilla `Creaking.setTransient`. The pathfinding-malus half (`DAMAGING`/`POWDER_SNOW`/
    /// `LAVA`/`FIRE`/`FIRE_IN_NEIGHBOR`) is deferred: Pumpkin's `PathType` enum doesn't have a
    /// clean 1:1 mapping onto vanilla's taxonomy for those five keys, and it isn't needed for
    /// the invulnerability/watched-immobility behavior this pass focuses on.
    pub fn set_transient(&self, pos: BlockPos) {
        self.home_pos.store(Some(pos));
    }

    #[must_use]
    pub fn is_heart_bound(&self) -> bool {
        self.home_pos.load().is_some()
    }

    #[must_use]
    pub fn get_home_pos(&self) -> Option<BlockPos> {
        self.home_pos.load()
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.is_active.load(Relaxed)
    }

    /// Looks up the block entity at `home_pos` and checks whether it still considers this
    /// creaking its live protector. `None` (no home, or no block entity there) is treated as
    /// "not bound", matching vanilla's `homePos == null` early-return in `tick()`.
    async fn is_still_protected(&self) -> bool {
        let Some(home_pos) = self.home_pos.load() else {
            return true;
        };
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let Some(block_entity) = world.get_block_entity(&home_pos) else {
            return false;
        };
        let Some(heart) = block_entity
            .as_any()
            .downcast_ref::<crate::block::entities::creaking_heart::CreakingHeartBlockEntity>(
        ) else {
            return false;
        };
        heart.is_protector(entity.entity_uuid).await
    }

    /// Vanilla `Creaking.checkCanMove`. Simplifications (documented, not silently dropped):
    /// no `canAttack`/`isAlliedTo` gate (every nearby player counts as a potential watcher),
    /// no carved-pumpkin disguise-item check, and the "nearby players" radius is approximated
    /// as the creaking's `FOLLOW_RANGE` attribute rather than a dedicated brain-sensor radius.
    async fn check_can_move(&self) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        let follow_range = self
            .mob_entity
            .living_entity
            .get_attribute_value(&pumpkin_data::attributes::Attributes::FOLLOW_RANGE);
        let players = world.get_nearby_players(pos, follow_range);
        let active = self.is_active();

        if players.is_empty() {
            if active {
                self.deactivate().await;
            }
            return true;
        }

        let mut has_potential_target = false;
        for player in &players {
            has_potential_target = true;
            if !is_looking_at_me(player, entity) {
                continue;
            }

            if active {
                return false;
            }

            let dist_sq = pos.squared_distance_to_vec(&player.living_entity.entity.pos.load());
            if dist_sq < 144.0 {
                self.activate(player).await;
                return false;
            }
        }

        if !has_potential_target && active {
            self.deactivate().await;
        }
        true
    }

    async fn activate(&self, player: &Arc<Player>) {
        self.is_active.store(true, Relaxed);
        self.set_mob_target(Some(player.clone())).await;
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        emit_game_event(
            &world,
            pumpkin_data::game_event::GameEvent::EntityAction,
            entity.pos.load(),
            GameEventContext::none(),
        )
        .await;
        world.play_sound(
            Sound::EntityCreakingActivate,
            SoundCategory::Hostile,
            &entity.pos.load(),
        );
    }

    async fn deactivate(&self) {
        self.is_active.store(false, Relaxed);
        self.set_mob_target(None).await;
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        emit_game_event(
            &world,
            pumpkin_data::game_event::GameEvent::EntityAction,
            entity.pos.load(),
            GameEventContext::none(),
        )
        .await;
        world.play_sound(
            Sound::EntityCreakingDeactivate,
            SoundCategory::Hostile,
            &entity.pos.load(),
        );
    }
}

/// Vanilla `LivingEntity.isLookingAtMe` (simplified: single eye-height candidate, 0.5 dot-product
/// FOV threshold, matching the doc's description of the vanilla check).
fn is_looking_at_me(player: &Arc<Player>, target: &Entity) -> bool {
    let eye_pos = player.get_eye_pos();
    let target_pos = target.pos.load() + Vector3::new(0.0, f64::from(target.height()) * 0.5, 0.0);
    let to_target = target_pos - eye_pos;
    let dist = to_target.length();
    if dist < 1.0e-4 {
        return true;
    }
    let dir = to_target.normalize();
    let look = player.get_looking_vector();
    dir.dot(&look) > 0.5
}

impl NBTStorage for CreakingEntity {}

impl Mob for CreakingEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.invulnerability_ticks.load(Relaxed) > 0 {
                self.invulnerability_ticks.fetch_sub(1, Relaxed);
            }

            // Vanilla `Creaking.tick`: a heart-bound creaking whose heart no longer claims it
            // as a live protector dies instantly.
            if self.is_heart_bound() && !self.is_still_protected().await {
                self.mob_entity.living_entity.set_health(0.0);
                return;
            }

            let was_movable = !self.is_active();
            let now_movable = self.check_can_move().await;
            if now_movable != was_movable {
                let entity = &self.mob_entity.living_entity.entity;
                let world = entity.world.load();
                let pos = entity.pos.load();
                emit_game_event(
                    &world,
                    pumpkin_data::game_event::GameEvent::EntityAction,
                    pos,
                    GameEventContext::none(),
                )
                .await;
                if now_movable {
                    world.play_sound(Sound::EntityCreakingUnfreeze, SoundCategory::Hostile, &pos);
                } else {
                    self.mob_entity.navigator.lock().unwrap().stop();
                    world.play_sound(Sound::EntityCreakingFreeze, SoundCategory::Hostile, &pos);
                }
            }
        })
    }

    /// Vanilla `Creaking.hurtServer`, ported via the `pre_damage` hook (the only per-mob
    /// damage-gate extension point `Mob` exposes -- `damage_with_context` itself lives in the
    /// blanket `EntityBase` impl and isn't overridable per mob). Unbound creakings, or damage
    /// that bypasses invulnerability, take damage normally (`pre_damage` returns `true`,
    /// falling through to the default pipeline). A heart-bound creaking instead absorbs *all*
    /// damage (never loses HP, `pre_damage` returns `false`) as long as either a living/
    /// projectile entity caused it or the direct damage source resolves to a player;
    /// anything else (e.g. unattributed explosion/fire) is a complete no-op. This is not "only
    /// the bound player can hurt it" -- any legitimate attacker is absorbed, not passed through.
    ///
    /// Deferred: `pre_damage` only receives the direct damage `source`, not vanilla's separate
    /// `cause` (e.g. the player who shot an arrow). An arrow hit is still absorbed correctly
    /// (the projectile check passes independent of attribution), but `creaking_hurt` -- which
    /// vanilla only fires when a *player* is responsible -- won't fire for indirect
    /// (projectile-shot-by-a-player) hits without a `cause` parameter threaded through, only for
    /// a player directly named as the damage source.
    fn pre_damage<'a>(
        &'a self,
        damage_type: DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let Some(home_pos) = self.home_pos.load() else {
                return true;
            };
            if damage_type.has_tag(&tag::DamageType::MINECRAFT_BYPASSES_INVULNERABILITY) {
                return true;
            }

            if self.invulnerability_ticks.load(Relaxed) > 0
                || self.mob_entity.living_entity.health.load() <= 0.0
            {
                return false;
            }

            let responsible_player = source.and_then(EntityBase::get_player);
            let is_living_or_projectile = source.is_some_and(|s| {
                s.get_living_entity().is_some()
                    || crate::entity::projectile::is_projectile(s.get_entity().entity_type)
            });
            if !is_living_or_projectile && responsible_player.is_none() {
                return false;
            }

            self.invulnerability_ticks.store(8, Relaxed);

            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load();
            let pos = entity.pos.load();
            emit_game_event(
                &world,
                pumpkin_data::game_event::GameEvent::EntityAction,
                pos,
                GameEventContext::none(),
            )
            .await;

            if let Some(block_entity) = world.get_block_entity(&home_pos)
                && let Some(heart) = block_entity
                    .as_any()
                    .downcast_ref::<crate::block::entities::creaking_heart::CreakingHeartBlockEntity>()
                && heart.is_protector(entity.entity_uuid).await
            {
                if responsible_player.is_some() {
                    heart.creaking_hurt();
                }
                world.play_sound(Sound::EntityCreakingSway, SoundCategory::Hostile, &pos);
            }

            false
        })
    }
}
