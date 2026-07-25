use super::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::mob::Mob;
use crate::entity::{EntityBase, mob::MobEntity};
use crate::world::World;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::{EntityType, MobCategory};
use rand::RngExt;
use std::future::Future;
use std::sync::Arc;

const DEFAULT_RECIPROCAL_CHANCE: i32 = 10;

/// What entity types this goal may acquire.
enum TargetKind {
    /// Single entity type (player / zombie / …).
    Exact(&'static EntityType),
    /// Any entity in a spawn category, minus an exclusion list.
    /// Vanilla iron golem: Monster category, exclude Creeper.
    Category {
        category: &'static MobCategory,
        exclude: &'static [&'static EntityType],
    },
}

pub struct ActiveTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    reciprocal_chance: i32,
    kind: TargetKind,
    target_predicate: TargetPredicate,
    follow_distance_multiplier: f64,
}

impl ActiveTargetGoal {
    pub fn new<F, Fut>(
        mob: &MobEntity,
        target_type: &'static EntityType,
        reciprocal_chance: i32,
        check_visibility: bool,
        check_can_navigate: bool,
        predicate: Option<F>,
    ) -> Self
    where
        F: Fn(Arc<LivingEntity>, Arc<World>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = bool> + Send + 'static,
    {
        let track_target_goal = TrackTargetGoal::new(check_visibility, check_can_navigate);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        if let Some(predicate) = predicate {
            target_predicate.set_predicate(predicate);
        }

        Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(reciprocal_chance),
            kind: TargetKind::Exact(target_type),
            target_predicate,
            follow_distance_multiplier: 1.0,
        }
    }

    #[must_use]
    pub fn with_default(
        mob: &MobEntity,
        target_type: &'static EntityType,
        check_visibility: bool,
    ) -> Box<Self> {
        let track_target_goal = TrackTargetGoal::with_default(check_visibility);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        Box::new(Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(DEFAULT_RECIPROCAL_CHANCE),
            kind: TargetKind::Exact(target_type),
            target_predicate,
            follow_distance_multiplier: 1.0,
        })
    }

    /// Vanilla iron-golem style: target every Monster-category mob except those in
    /// `exclude` (typically just creeper). Single goal so the closest living
    /// hostile wins — no corpse of type A blocking a living type B.
    ///
    /// This is the Pumpkin stand-in for Java
    /// `NearestAttackableTargetGoal(Mob.class, …, e -> e instanceof Enemy && …)`.
    #[must_use]
    pub fn for_category(
        mob: &MobEntity,
        category: &'static MobCategory,
        exclude: &'static [&'static EntityType],
        reciprocal_chance: i32,
        check_visibility: bool,
    ) -> Box<Self> {
        let track_target_goal = TrackTargetGoal::with_default(check_visibility);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        Box::new(Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(reciprocal_chance),
            kind: TargetKind::Category { category, exclude },
            target_predicate,
            follow_distance_multiplier: 1.0,
        })
    }

    /// Vanilla `Enemy` targeting: all [`MobCategory::MONSTER`] minus `exclude`.
    ///
    /// - Iron golem: `exclude = IRON_GOLEM_ENEMY_EXCLUDES` (creeper only), chance 5, no LOS gate
    /// - Snow golem: `exclude = []`, chance 10, with LOS
    #[must_use]
    pub fn for_enemies(
        mob: &MobEntity,
        exclude: &'static [&'static EntityType],
        reciprocal_chance: i32,
        check_visibility: bool,
    ) -> Box<Self> {
        Self::for_category(
            mob,
            crate::entity::ai::vanilla_enemy::ENEMY_CATEGORY,
            exclude,
            reciprocal_chance,
            check_visibility,
        )
    }

    pub fn set_target(&mut self, target: Option<Arc<dyn EntityBase>>) {
        self.target = target;
    }

    #[must_use]
    pub fn with_follow_distance_multiplier(mut self, multiplier: f64) -> Self {
        self.follow_distance_multiplier = multiplier;
        self.track_target_goal = self
            .track_target_goal
            .with_follow_distance_multiplier(multiplier);
        self
    }

    fn matches_kind(&self, entity_type: &'static EntityType) -> bool {
        match self.kind {
            TargetKind::Exact(want) => entity_type.id == want.id,
            TargetKind::Category { category, exclude } => {
                // Vanilla Enemy ≈ MONSTER + mob + attackable, minus explicit excludes.
                entity_type.category == category
                    && entity_type.mob
                    && entity_type.attackable != Some(false)
                    && !exclude.iter().any(|e| e.id == entity_type.id)
            }
        }
    }

    fn find_closest_target(&mut self, mob: &MobEntity) {
        let follow_range = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE)
            * self.follow_distance_multiplier;

        // Vanilla updates the target conditions with the current follow distance on every search
        self.target_predicate.base_max_distance = follow_range;

        let world = mob.living_entity.entity.world.load();

        // Search volume is centered on the eye (vanilla), but ranking uses feet
        // position so we don't prefer a far mob that happens to match eye height.
        let mob_pos = mob.living_entity.entity.pos.load();
        let mut search_pos = mob_pos;
        search_pos.y += mob.living_entity.entity.entity_dimension.load().eye_height as f64;

        // Minestom ClosestEntityTarget / vanilla ActiveTargetGoal: scan nearby
        // candidates and pick the closest that passes the predicate. Do NOT take
        // the raw closest entity then filter — a corpse would block retargeting.
        let mut best: Option<(f64, Arc<dyn EntityBase>)> = None;

        let looking_for_player = matches!(
            self.kind,
            TargetKind::Exact(t) if t.id == EntityType::PLAYER.id
        );

        if looking_for_player {
            for player in world.get_nearby_players(search_pos, follow_range) {
                if player.is_spectator() || player.is_creative() {
                    continue;
                }
                let entity = player.clone() as Arc<dyn EntityBase>;
                let Some(living) = entity.get_living_entity() else {
                    continue;
                };
                if !self
                    .target_predicate
                    .test(&world, Some(&mob.living_entity), living)
                {
                    continue;
                }
                let dist_sq = mob_pos.squared_distance_to_vec(&entity.get_entity().pos.load());
                if best.as_ref().is_none_or(|(d, _)| dist_sq < *d) {
                    best = Some((dist_sq, entity));
                }
            }
        } else {
            for entity in world
                .get_nearby_entities(search_pos, follow_range)
                .into_values()
            {
                let et = entity.get_entity().entity_type;
                if !self.matches_kind(et) {
                    continue;
                }
                let Some(living) = entity.get_living_entity() else {
                    continue;
                };
                if !self
                    .target_predicate
                    .test(&world, Some(&mob.living_entity), living)
                {
                    continue;
                }
                let dist_sq = mob_pos.squared_distance_to_vec(&entity.get_entity().pos.load());
                if best.as_ref().is_none_or(|(d, _)| dist_sq < *d) {
                    best = Some((dist_sq, entity));
                }
            }
        }

        self.target = best.map(|(_, e)| e);
    }
}

impl Goal for ActiveTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if self.reciprocal_chance > 0
                && mob.get_random().random_range(0..self.reciprocal_chance) != 0
            {
                return false;
            }
            self.find_closest_target(mob.get_mob_entity());
            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.track_target_goal.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.set_mob_target(self.target.clone()).await;
            self.track_target_goal.start(mob).await;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.track_target_goal.stop(mob).await;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            // Periodically re-scan for a *closer* living target. Vanilla keeps the
            // first lock until invalid; that causes "chase far zombie while near
            // one is hitting me" when Revenge hasn't fired yet. If a candidate is
            // meaningfully closer (2+ blocks), switch.
            if mob.get_random().random_range(0..10) != 0 {
                return;
            }
            let current = mob.get_mob_entity().target.lock().await.clone();
            let Some(current) = current else {
                return;
            };
            if let Some(living) = current.get_living_entity()
                && !living.is_alive()
            {
                return;
            }

            let mob_pos = mob.get_entity().pos.load();
            let current_dist = mob_pos.squared_distance_to_vec(&current.get_entity().pos.load());

            self.find_closest_target(mob.get_mob_entity());
            if let Some(best) = self.target.clone() {
                let best_dist = mob_pos.squared_distance_to_vec(&best.get_entity().pos.load());
                // Switch if closer by at least ~2 blocks, or different entity much nearer.
                if best.get_entity().entity_id != current.get_entity().entity_id
                    && best_dist + 4.0 < current_dist
                {
                    mob.set_mob_target(Some(best)).await;
                }
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}
