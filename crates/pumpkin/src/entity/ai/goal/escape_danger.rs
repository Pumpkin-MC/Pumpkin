use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_data::damage::DamageType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

const RANGE: i32 = 5;
const TARGET_ATTEMPTS: usize = 10;

pub struct EscapeDangerGoal {
    speed: f64,
    goal_control: Controls,
    target: Option<Vector3<f64>>,
    /// Only damage types in this tag cause panic (vanilla `PanicGoal#shouldPanic`).
    panic_tag: &'static tag::Tag,
}

impl EscapeDangerGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self::with_panic_tag(
            speed,
            &tag::DamageType::MINECRAFT_PANIC_CAUSES,
        ))
    }

    /// Creates a panic goal that only reacts to environmental damage
    /// (vanilla `TamableAnimalPanicGoal`, used by tamed animals like wolves and cats).
    #[must_use]
    pub fn with_environmental_panic(speed: f64) -> Box<Self> {
        Box::new(Self::with_panic_tag(
            speed,
            &tag::DamageType::MINECRAFT_PANIC_ENVIRONMENTAL_CAUSES,
        ))
    }

    #[must_use]
    const fn with_panic_tag(speed: f64, panic_tag: &'static tag::Tag) -> Self {
        Self {
            speed,
            goal_control: Controls::MOVE,
            target: None,
            panic_tag,
        }
    }

    fn is_in_danger(&self, mob: &dyn Mob) -> bool {
        let living = &mob.get_mob_entity().living_entity;

        // Vanilla `PanicGoal#shouldPanic`: only the last damage source's type decides
        // whether the mob panics, so being punched does not scare tamed animals.
        let last_damage_type_id = living.last_damage_type_id.load(Relaxed);
        u8::try_from(last_damage_type_id)
            .ok()
            .and_then(DamageType::from_id)
            .is_some_and(|damage_type| damage_type.has_tag(self.panic_tag))
    }

    fn find_escape_target(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let mut rng = mob.get_random();

        for _ in 0..TARGET_ATTEMPTS {
            let dx = rng.random_range(-RANGE..=RANGE);
            let dz = rng.random_range(-RANGE..=RANGE);
            if dx == 0 && dz == 0 {
                continue;
            }
            return Some(Vector3::new(pos.x + dx as f64, pos.y, pos.z + dz as f64));
        }

        None
    }
}

impl Goal for EscapeDangerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !self.is_in_danger(mob) {
            return false;
        }
        self.target = Self::find_escape_target(mob);
        self.target.is_some()
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        !navigator.is_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target {
            let pos = mob.get_mob_entity().living_entity.entity.pos.load();
            let mut navigator = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            navigator.set_progress(NavigatorGoal::new(pos, target, self.speed));
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target = None;
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
