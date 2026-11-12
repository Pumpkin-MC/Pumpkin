use pumpkin_data::effect::StatusEffect;
use pumpkin_data::tag::{self, Tag, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalReach};
use crate::entity::ai::goal::try_find_water::TryFindWaterGoal;
use crate::entity::ai::util::default_random_pos;
use crate::entity::passive::abstract_horse::AbstractHorse;
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};

const WATER_SEARCH_XZ: i32 = 5;
const WATER_SEARCH_Y: i32 = 1;
const HORIZONTAL_RANGE: i32 = 5;
const VERTICAL_RANGE: i32 = 4;

/// Vanilla `PanicGoal`'s function from the mob to the damage type tag that makes it panic.
pub type PanicTag = fn(&dyn Mob) -> &'static Tag;

/// Ends a panic route early, as vanilla `PandaPanicGoal.canContinueToUse` does for a sitting panda.
pub type GiveUpCheck = fn(&dyn Mob) -> bool;

fn panic_causes(_mob: &dyn Mob) -> &'static Tag {
    &tag::DamageType::MINECRAFT_PANIC_CAUSES
}

pub struct EscapeDangerGoal {
    speed: f64,
    panic_tag: PanicTag,
    give_up_when: Option<GiveUpCheck>,
    goal_control: Controls,
    target: Option<Vector3<f64>>,
    running: bool,
    calm_while_mob_controlled: bool,
    teleport_to_owner: bool,
    water_search_xz: Option<i32>,
}

impl EscapeDangerGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            speed,
            panic_tag: panic_causes,
            give_up_when: None,
            goal_control: Controls::MOVE,
            target: None,
            running: false,
            calm_while_mob_controlled: false,
            teleport_to_owner: false,
            water_search_xz: None,
        })
    }

    #[must_use]
    pub fn tamable(speed: f64) -> Box<Self> {
        let mut goal = Self::new(speed);
        goal.teleport_to_owner = true;
        goal
    }

    /// Horses don't panic while they count as mob controlled.
    #[must_use]
    pub fn mount(speed: f64) -> Box<Self> {
        let mut goal = Self::new(speed);
        goal.calm_while_mob_controlled = true;
        goal
    }

    /// The damage type tag that makes this mob panic, from the mob itself.
    #[must_use]
    pub const fn with_panic_tag(mut self: Box<Self>, panic_tag: PanicTag) -> Box<Self> {
        self.panic_tag = panic_tag;
        self
    }

    /// The mob stops its route and the goal ends as soon as `give_up_when` holds.
    #[must_use]
    pub const fn giving_up_when(mut self: Box<Self>, give_up_when: GiveUpCheck) -> Box<Self> {
        self.give_up_when = Some(give_up_when);
        self
    }

    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.running
    }

    /// Only damage of the mob's panic tag (`#minecraft:panic_causes` by default) makes it flee,
    /// and only while the source is still remembered.
    fn should_panic(&self, mob: &dyn Mob) -> bool {
        mob.get_mob_entity()
            .living_entity
            .get_last_damage_type()
            .is_some_and(|damage_type| damage_type.has_tag((self.panic_tag)(mob)))
    }

    /// Nearest water within `xz_dist` blocks horizontally, only when the mob is not stuck in a block.
    fn look_for_water(mob: &dyn Mob, xz_dist: i32) -> Option<BlockPos> {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let mob_pos = entity.block_pos.load();

        if world
            .get_block_state(&mob_pos)
            .get_block_collision_shapes_at(&mob_pos)
            .next()
            .is_some()
        {
            return None;
        }

        let mut best: Option<(i32, BlockPos)> = None;
        for dx in -xz_dist..=xz_dist {
            for dy in -WATER_SEARCH_Y..=WATER_SEARCH_Y {
                for dz in -xz_dist..=xz_dist {
                    let pos = BlockPos::new(mob_pos.0.x + dx, mob_pos.0.y + dy, mob_pos.0.z + dz);
                    if !TryFindWaterGoal::is_water(&world, &pos) {
                        continue;
                    }
                    let distance = dx.abs() + dy.abs() + dz.abs();
                    if best.is_none_or(|(best_distance, _)| distance < best_distance) {
                        best = Some((distance, pos));
                    }
                }
            }
        }

        best.map(|(_, pos)| pos)
    }

    fn find_random_position(mob: &dyn Mob) -> Option<Vector3<f64>> {
        default_random_pos::get_pos(mob, HORIZONTAL_RANGE, VERTICAL_RANGE)
    }
}

impl Goal for EscapeDangerGoal {
    fn tick_reach(&self, mob: &dyn Mob, running: bool) -> Option<GoalReach> {
        // Fleeing is the only fast part. Self-inflicted panic before the goals run comes from
        // fire, lava, freezing or effects; hits from others already widen the reach.
        let living = &mob.get_mob_entity().living_entity;
        let entity = &living.entity;
        let may_flee = running
            || self.should_panic(mob)
            || entity.fire_ticks.load(Relaxed) > 0
            || entity.frozen_ticks.load(Relaxed) > 0
            || entity.touching_lava.load(Relaxed)
            || living.has_effect(&StatusEffect::POISON)
            || living.has_effect(&StatusEffect::WITHER);
        Some(GoalReach {
            speed: if may_flee { self.speed } else { 0.0 },
            ..GoalReach::default()
        })
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !self.should_panic(mob)
            || (self.calm_while_mob_controlled
                && mob
                    .as_abstract_horse()
                    .is_some_and(AbstractHorse::is_mob_controlled))
        {
            return false;
        }

        let water_search_xz = self
            .water_search_xz
            .or_else(|| (mob.get_entity().fire_ticks.load(Relaxed) > 0).then_some(WATER_SEARCH_XZ));
        let water = water_search_xz.and_then(|xz_dist| Self::look_for_water(mob, xz_dist));
        if let Some(water) = water {
            self.target = Some(water.to_f64());
            return true;
        }

        self.target = Self::find_random_position(mob);
        self.target.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        if self.give_up_when.is_some_and(|give_up| give_up(mob)) {
            mob.get_mob_entity().steered().navigator().stop();
            return false;
        }
        !mob.is_navigator_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target {
            let pos = mob.get_mob_entity().living_entity.entity.pos.load();
            mob.get_mob_entity()
                .steered()
                .navigator()
                .set_progress(NavigatorGoal::new(pos, target, self.speed));
        }
        self.running = true;
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target = None;
        self.running = false;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if self.teleport_to_owner {
            super::follow_owner::FollowOwnerGoal::teleport_if_needed(mob);
        }
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
