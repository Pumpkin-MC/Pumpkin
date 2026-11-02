use std::sync::Arc;

use super::{Controls, Goal};
use crate::entity::ai::goal::revenge::MobFilter;
use crate::entity::ai::pathfinder::path::Path;
use crate::entity::ai::target_match::{SearchArea, TargetMatch};
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::ai::util::default_random_pos;
use crate::entity::predicate::EntityPredicate;
use crate::entity::{EntityBase, mob::Mob};
use crate::plugin::api::events::entity::entity_target::TargetReason;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;

const FAST_DISTANCE_SQ: f64 = 49.0;
const HORIZONTAL_RANGE: i32 = 16;
const VERTICAL_RANGE: i32 = 7;
/// Vanilla only looks this far above and below for threats.
const THREAT_SEARCH_HEIGHT: f64 = 3.0;

/// A check on the threat that was found.
pub type ThreatFilter = fn(&dyn EntityBase) -> bool;

/// Vanilla's `predicateOnAvoidEntity`, checked on every candidate: fleeing mob, candidate.
pub type ThreatSelector = fn(&dyn Mob, &dyn EntityBase) -> bool;

/// The default selector: no creative or spectator players.
pub fn not_creative_or_spectator(_mob: &dyn Mob, candidate: &dyn EntityBase) -> bool {
    EntityPredicate::ExceptCreativeOrSpectator.test(candidate.get_entity())
}

/// Fish and pandas: creative players count too.
pub fn not_spectator(_mob: &dyn Mob, candidate: &dyn EntityBase) -> bool {
    !candidate.get_entity().is_spectator()
}

/// Threat types, each with the distance it starts the flee at.
pub type ThreatDistances = &'static [(&'static EntityType, f64)];

pub struct AvoidEntityGoal {
    goal_control: Controls,
    flee_from: TargetMatch,
    flee_distance: f64,
    threat_predicate: TargetPredicate,
    slow_speed: f64,
    fast_speed: f64,
    selector: ThreatSelector,
    target: Option<Arc<dyn EntityBase>>,
    path: Option<Path>,
    when: Option<MobFilter>,
    keep_when: bool,
    confirm: Option<ThreatFilter>,
    drop_target: bool,
    per_type: Option<ThreatDistances>,
}

impl AvoidEntityGoal {
    #[must_use]
    pub fn new(
        flee_from: impl Into<TargetMatch>,
        flee_distance: f64,
        slow_speed: f64,
        fast_speed: f64,
    ) -> Self {
        Self {
            goal_control: Controls::MOVE,
            flee_from: flee_from.into(),
            flee_distance,
            // Vanilla `forCombat`: line of sight, can attack, not allied, visibility-scaled range.
            threat_predicate: TargetPredicate::create_attackable()
                .set_base_max_distance(flee_distance),
            slow_speed,
            fast_speed,
            selector: not_creative_or_spectator,
            target: None,
            path: None,
            when: None,
            keep_when: false,
            confirm: None,
            drop_target: false,
            per_type: None,
        }
    }

    /// Vanilla brain sensors (`VillagerHostilesSensor`): the nearest visible threat
    /// within its own type's distance, seen like any entity, no height limit.
    #[must_use]
    pub fn nearest_of(threats: ThreatDistances, slow_speed: f64, fast_speed: f64) -> Self {
        let reach = threats
            .iter()
            .fold(0.0, |max, &(_, dist)| f64::max(max, dist));
        Self {
            threat_predicate: TargetPredicate::create_non_attackable(),
            per_type: Some(threats),
            ..Self::new(TargetMatch::Any, reach, slow_speed, fast_speed)
        }
    }

    /// Only flees while the mob passes `filter`
    #[must_use]
    pub const fn when(mut self, filter: MobFilter) -> Self {
        self.when = Some(filter);
        self
    }

    /// Like [`Self::when`], also checked while fleeing. e.g. a cat tamed mid-flight stops.
    #[must_use]
    pub const fn as_long_as(mut self, filter: MobFilter) -> Self {
        self.when = Some(filter);
        self.keep_when = true;
        self
    }

    /// Replaces the default creative and spectator check on every candidate.
    #[must_use]
    pub const fn matching(mut self, selector: ThreatSelector) -> Self {
        self.selector = selector;
        self
    }

    /// The nearest threat must also pass `filter`, else no flee
    #[must_use]
    pub const fn confirmed_by(mut self, filter: ThreatFilter) -> Self {
        self.confirm = Some(filter);
        self
    }

    /// Clears the mob's target while fleeing.
    #[must_use]
    pub const fn dropping_target(mut self) -> Self {
        self.drop_target = true;
        self
    }

    fn find_threat(&self, mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let living = &mob.get_mob_entity().living_entity;
        let entity = &living.entity;
        let pos = entity.pos.load();
        let world = entity.world.load();
        let height = if self.per_type.is_some() {
            self.flee_distance
        } else {
            THREAT_SEARCH_HEIGHT
        };
        let area = SearchArea::from_feet(entity, self.flee_distance, height);
        // Sensors see as far as the follow range; the per-type distance cuts it down after.
        let sensor_predicate;
        let predicate = if self.per_type.is_some() {
            sensor_predicate = self
                .threat_predicate
                .copy()
                .set_base_max_distance(living.get_attribute_value(&Attributes::FOLLOW_RANGE));
            &sensor_predicate
        } else {
            &self.threat_predicate
        };

        self.flee_from.find_nearest(&world, &area, |candidate| {
            let candidate_entity = candidate.get_entity();
            self.per_type.is_none_or(|threats| {
                threats.iter().any(|&(threat, dist)| {
                    threat == candidate_entity.entity_type
                        && candidate_entity.pos.load().squared_distance_to_vec(&pos) <= dist * dist
                })
            }) && (self.selector)(mob, candidate)
                && predicate.test(&world, Some(mob), candidate)
        })
    }
}

impl Goal for AvoidEntityGoal {
    fn tick_reach(&self, _mob: &dyn Mob, _running: bool) -> Option<super::GoalReach> {
        Some(super::GoalReach {
            entities: self.flee_distance,
            players: if self.flee_from.matches(&EntityType::PLAYER) {
                self.flee_distance
            } else {
                0.0
            },
            speed: self.fast_speed,
            links: self
                .target
                .iter()
                .map(|target| target.get_entity().entity_id)
                .collect(),
            ..super::GoalReach::default()
        })
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.when.is_some_and(|when| !when(mob)) {
            return false;
        }
        let Some(target) = self.find_threat(mob) else {
            return false;
        };
        if self
            .confirm
            .is_some_and(|confirm| !confirm(target.as_ref()))
        {
            return false;
        }

        let threat_pos = target.get_entity().pos.load();
        let Some(flee_pos) =
            default_random_pos::get_pos_away(mob, HORIZONTAL_RANGE, VERTICAL_RANGE, threat_pos)
        else {
            return false;
        };

        // Give up when the escape route does not gain any distance.
        let mob_pos = mob.get_entity().pos.load();
        if threat_pos.squared_distance_to_vec(&flee_pos)
            < threat_pos.squared_distance_to_vec(&mob_pos)
        {
            return false;
        }

        // Vanilla `createPath(pos, 0)`, no route, no flee, so the current goal keeps running.
        // A rider plans for the mob it steers, like vanilla `getNavigation`.
        let steered = mob.get_mob_entity().steered();
        let path =
            steered
                .navigator()
                .create_path(&steered.mob_entity().living_entity, flee_pos, 0);
        let Some(path) = path else {
            return false;
        };

        self.target = Some(target);
        self.path = Some(path);
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        if self.keep_when && self.when.is_some_and(|when| !when(mob)) {
            return false;
        }
        !mob.is_navigator_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if self.drop_target {
            mob.set_mob_target(None, TargetReason::Fleeing);
        }
        let steered = mob.get_mob_entity().steered();
        steered.navigator().move_to_path(
            self.path.take(),
            self.slow_speed,
            &steered.mob_entity().living_entity,
        );
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if self.drop_target {
            mob.set_mob_target(None, TargetReason::Fleeing);
        }
        if let Some(target) = &self.target {
            let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
            let threat_pos = target.get_entity().pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&threat_pos);
            let speed = if dist_sq < FAST_DISTANCE_SQ {
                self.fast_speed
            } else {
                self.slow_speed
            };
            mob.get_mob_entity().steered().navigator().set_speed(speed);
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target = None;
        self.path = None;
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use crate::entity::mob::creeper::CreeperEntity;
    use crate::entity::mob::skeleton::SkeletonEntityBase;
    use crate::entity::mob::spider::SpiderEntity;
    use crate::entity::passive::ocelot::OcelotEntity;
    use crate::entity::passive::wolf::WolfEntity;
    use crate::entity::{Entity, EntityBase};
    use pumpkin_data::Block;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector2::Vector2;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_world::chunk::ChunkData;
    use pumpkin_world::world::BlockFlags;
    use std::sync::atomic::Ordering;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn stone_floor(world: &std::sync::Arc<crate::world::World>) {
        for chunk_x in -2..=1 {
            for chunk_z in -2..=1 {
                world.level.loaded_chunks.insert(
                    Vector2::new(chunk_x, chunk_z),
                    ChunkData::empty_sync(chunk_x, chunk_z),
                );
            }
        }
        let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
        for x in -20..=20 {
            for z in -20..=20 {
                world.set_block_state(
                    &BlockPos::new(x, 63, z),
                    Block::STONE.default_state.id,
                    flags,
                );
            }
        }
    }

    #[tokio::test]
    async fn creeper_starts_fleeing_a_spawned_ocelot_within_vanilla_retry_window() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);

        let creeper = CreeperEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::CREEPER,
        ));
        creeper
            .get_entity()
            .on_ground
            .store(true, Ordering::Relaxed);
        world.spawn_entity(creeper.clone());

        let ocelot = OcelotEntity::new(Entity::new(
            world.clone(),
            Vector3::new(2.5, 64.0, 0.5),
            &EntityType::OCELOT,
        ));
        ocelot.get_entity().on_ground.store(true, Ordering::Relaxed);
        world.spawn_entity(ocelot);

        let mut started_at = None;
        for tick in 1..=30 {
            creeper
                .get_entity()
                .tick_count
                .store(tick, Ordering::Relaxed);
            creeper.mob_entity.server_ai_step(&*creeper, &*creeper);
            if !creeper.is_navigator_idle() {
                started_at = Some(tick);
                break;
            }
        }
        world.level.shutdown().await;

        let started_at = started_at.ok_or("creeper never started fleeing the ocelot")?;
        assert!(
            started_at <= 20,
            "flee started on tick {started_at}, later than vanilla's retry window"
        );
        Ok(())
    }

    #[tokio::test]
    async fn jockey_skeleton_flees_a_wolf_on_its_spider() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);

        let pos = Vector3::new(0.5, 64.0, 0.5);
        let spider = SpiderEntity::new(Entity::new(world.clone(), pos, &EntityType::SPIDER));
        let skeleton =
            SkeletonEntityBase::new(Entity::new(world.clone(), pos, &EntityType::SKELETON));
        for entity in [spider.get_entity(), skeleton.get_entity()] {
            entity.on_ground.store(true, Ordering::Relaxed);
        }
        world.spawn_entity(spider.clone());
        world.spawn_entity(skeleton.clone());
        spider
            .get_entity()
            .add_passenger(spider.clone(), skeleton.clone());

        let wolf = WolfEntity::new(Entity::new(
            world.clone(),
            Vector3::new(2.5, 64.0, 0.5),
            &EntityType::WOLF,
        ));
        wolf.get_entity().on_ground.store(true, Ordering::Relaxed);
        world.spawn_entity(wolf);

        // The flee route belongs to the spider, so it survives the next continue check.
        let mut fleeing_ticks = 0;
        for tick in 1..=40 {
            skeleton
                .get_entity()
                .tick_count
                .store(tick, Ordering::Relaxed);
            skeleton.mob_entity.server_ai_step(&*skeleton, &*skeleton);
            if spider.is_navigator_idle() {
                fleeing_ticks = 0;
            } else {
                fleeing_ticks += 1;
            }
            if fleeing_ticks > 4 {
                break;
            }
        }
        world.level.shutdown().await;

        assert!(
            fleeing_ticks > 4,
            "the jockey never kept a flee route on its spider"
        );
        assert!(
            skeleton
                .mob_entity
                .navigator
                .lock()
                .map_err(|e| e.to_string())?
                .is_idle(),
            "the rider planned the flee on its own navigator"
        );
        Ok(())
    }
}
