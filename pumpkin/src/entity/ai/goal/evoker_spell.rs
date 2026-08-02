use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Weak};

use pumpkin_data::BlockDirection;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::block::entities::sign::DyeColor;
use crate::entity::ai::goal::spellcaster::{IllagerSpell, SpellCastTimer};
use crate::entity::ai::goal::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::entity::mob::evoker::EvokerEntity;
use crate::entity::mob::vex::VexEntity;
use crate::entity::passive::sheep::SheepEntity;
use crate::entity::projectile::evoker_fangs::EvokerFangsEntity;
use crate::entity::{EntityBase, r#type::from_type};

/// Vanilla: `Evoker.EvokerCastingSpellGoal` (extends `SpellcasterIllager.SpellcasterCastingSpellGoal`).
/// Stops navigation and keeps the evoker looking at whatever it's casting at while a spell is
/// active.
pub struct EvokerCastingSpellGoal {
    evoker: Weak<EvokerEntity>,
}

impl EvokerCastingSpellGoal {
    #[must_use]
    pub const fn new(evoker: Weak<EvokerEntity>) -> Self {
        Self { evoker }
    }
}

impl Goal for EvokerCastingSpellGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.evoker
                .upgrade()
                .is_some_and(|evoker| evoker.spellcaster.is_casting_spell())
        })
    }

    // Vanilla's base `Goal#canContinueToUse` defaults to `canUse()` when not overridden, and
    // `SpellcasterCastingSpellGoal` doesn't override it.
    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.evoker
                .upgrade()
                .is_some_and(|evoker| evoker.spellcaster.is_casting_spell())
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(evoker) = self.evoker.upgrade() {
                evoker.mob_entity.navigator.lock().unwrap().stop();
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(evoker) = self.evoker.upgrade() {
                evoker.spellcaster.set_current_spell(IllagerSpell::None);
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return;
            };
            let look_target = evoker.mob_entity.target.lock().await.clone();
            let look_target = match look_target {
                Some(target) => Some(target),
                None => evoker.wololo_target.lock().await.clone(),
            };
            if let Some(target) = look_target {
                evoker
                    .mob_entity
                    .look_control
                    .lock()
                    .unwrap()
                    .look_at_entity_with_range(
                        &target,
                        mob.get_max_look_yaw_change(),
                        mob.get_max_look_pitch_change(),
                    );
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

/// Vanilla: `SpellcasterIllager.SpellcasterUseSpellGoal#canUse`, minus the spell-specific extra
/// gate (e.g. the vex-count check `EvokerSummonSpellGoal` adds on top).
async fn generic_use_spell_can_use(evoker: &EvokerEntity, next_attack_tick_count: i32) -> bool {
    let target = evoker.mob_entity.target.lock().await.clone();
    let Some(target) = target else {
        return false;
    };
    if !target.get_entity().is_alive() || evoker.spellcaster.is_casting_spell() {
        return false;
    }
    evoker.mob_entity.living_entity.entity.age.load(Relaxed) >= next_attack_tick_count
}

/// Vanilla: `Evoker.EvokerAttackSpellGoal`.
pub struct EvokerAttackSpellGoal {
    evoker: Weak<EvokerEntity>,
    timer: SpellCastTimer,
}

impl EvokerAttackSpellGoal {
    const CASTING_TIME: i32 = 40;
    const CASTING_INTERVAL: i32 = 100;

    #[must_use]
    pub const fn new(evoker: Weak<EvokerEntity>) -> Self {
        Self {
            evoker,
            timer: SpellCastTimer::new(),
        }
    }

    /// Vanilla: `EvokerAttackSpellGoal.performSpellCasting`, geometry only. Returns
    /// `(x, z, angle_radians, delay_ticks)` for every fang to spawn.
    #[must_use]
    fn fang_layout(
        evoker_x: f64,
        evoker_z: f64,
        target_x: f64,
        target_z: f64,
        distance_sq: f64,
    ) -> Vec<(f64, f64, f32, i32)> {
        let angle_towards_target = (target_z - evoker_z).atan2(target_x - evoker_x) as f32;
        let mut fangs = Vec::new();

        if distance_sq < 9.0 {
            for i in 0..5 {
                let angle = angle_towards_target + i as f32 * std::f32::consts::PI * 0.4;
                fangs.push((
                    evoker_x + f64::from(angle.cos()) * 1.5,
                    evoker_z + f64::from(angle.sin()) * 1.5,
                    angle,
                    0,
                ));
            }
            for i in 0..8 {
                let angle = angle_towards_target
                    + i as f32 * std::f32::consts::PI * 2.0 / 8.0
                    + std::f32::consts::PI * 2.0 / 5.0;
                fangs.push((
                    evoker_x + f64::from(angle.cos()) * 2.5,
                    evoker_z + f64::from(angle.sin()) * 2.5,
                    angle,
                    3,
                ));
            }
        } else {
            for i in 0..16 {
                let reach = 1.25 * f64::from(i + 1);
                fangs.push((
                    evoker_x + f64::from(angle_towards_target.cos()) * reach,
                    evoker_z + f64::from(angle_towards_target.sin()) * reach,
                    angle_towards_target,
                    i,
                ));
            }
        }

        fangs
    }

    /// Vanilla: `EvokerAttackSpellGoal.createSpellEntity`. Scans downward from `max_y` for the
    /// first sturdy floor and spawns a fang on top of it.
    ///
    /// Scope reduction: vanilla additionally raises the fang by the target block's collision-shape
    /// top Y (e.g. a slab or stair beneath it); Pumpkin has no generic per-block collision-shape
    /// query reachable here, so the fang always sits flush on the sturdy block's top face.
    async fn create_spell_entity(
        evoker: &EvokerEntity,
        x: f64,
        z: f64,
        min_y: f64,
        max_y: f64,
        angle: f32,
        delay_ticks: i32,
    ) {
        let world = evoker.mob_entity.living_entity.entity.world.load();
        let mut pos = BlockPos::new(x.floor() as i32, max_y.floor() as i32, z.floor() as i32);
        let min_y_floor = min_y.floor() as i32;
        let mut success = false;

        loop {
            let below = BlockPos::new(pos.0.x, pos.0.y - 1, pos.0.z);
            if world
                .get_block_state(&below)
                .is_side_solid(BlockDirection::Up)
            {
                success = true;
                break;
            }
            pos = below;
            if pos.0.y < min_y_floor - 1 {
                break;
            }
        }

        if success {
            let fangs = EvokerFangsEntity::new_spell(
                &world,
                Vector3::new(x, f64::from(pos.0.y), z),
                angle,
                delay_ticks,
                &evoker.mob_entity.living_entity.entity,
            );
            world.spawn_entity(Arc::new(fangs)).await;
        }
    }

    async fn perform_spell_casting(evoker: &EvokerEntity) {
        let Some(target) = evoker.mob_entity.target.lock().await.clone() else {
            return;
        };
        let evoker_pos = evoker.mob_entity.living_entity.entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        let min_y = evoker_pos.y.min(target_pos.y);
        let max_y = evoker_pos.y.max(target_pos.y) + 1.0;
        let distance_sq = evoker_pos.squared_distance_to_vec(&target_pos);

        let fangs = Self::fang_layout(
            evoker_pos.x,
            evoker_pos.z,
            target_pos.x,
            target_pos.z,
            distance_sq,
        );
        for (x, z, angle, delay) in fangs {
            Self::create_spell_entity(evoker, x, z, min_y, max_y, angle, delay).await;
        }
    }
}

impl Goal for EvokerAttackSpellGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return false;
            };
            generic_use_spell_can_use(&evoker, self.timer.next_attack_tick_count).await
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return false;
            };
            let target = evoker.mob_entity.target.lock().await.clone();
            target.is_some_and(|target| target.get_entity().is_alive())
                && self.timer.attack_warmup_delay > 0
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return;
            };
            let tick_count = evoker.mob_entity.living_entity.entity.age.load(Relaxed);
            self.timer.start(
                tick_count,
                20,
                Self::CASTING_TIME,
                Self::CASTING_INTERVAL,
                &evoker.spellcaster,
                IllagerSpell::Fangs,
            );
            evoker
                .mob_entity
                .living_entity
                .entity
                .world
                .load()
                .play_sound(
                    Sound::EntityEvokerPrepareAttack,
                    SoundCategory::Hostile,
                    &evoker.mob_entity.living_entity.entity.pos.load(),
                );
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return;
            };
            if self.timer.tick() {
                Self::perform_spell_casting(&evoker).await;
                evoker
                    .mob_entity
                    .living_entity
                    .entity
                    .world
                    .load()
                    .play_sound(
                        Sound::EntityEvokerCastSpell,
                        SoundCategory::Hostile,
                        &evoker.mob_entity.living_entity.entity.pos.load(),
                    );
            }
        })
    }
}

/// Vanilla: `Evoker.EvokerSummonSpellGoal`.
pub struct EvokerSummonSpellGoal {
    evoker: Weak<EvokerEntity>,
    timer: SpellCastTimer,
}

impl EvokerSummonSpellGoal {
    const CASTING_TIME: i32 = 100;
    const CASTING_INTERVAL: i32 = 340;

    #[must_use]
    pub const fn new(evoker: Weak<EvokerEntity>) -> Self {
        Self {
            evoker,
            timer: SpellCastTimer::new(),
        }
    }

    /// Vanilla: `EvokerSummonSpellGoal.canUse`'s vex-count gate: `random.nextInt(8) + 1 > vexes`.
    #[must_use]
    const fn vex_summon_allowed(nearby_vexes: usize, roll_1_to_8: i32) -> bool {
        roll_1_to_8 > nearby_vexes as i32
    }

    fn nearby_vex_count(evoker: &EvokerEntity) -> usize {
        let entity = &evoker.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let bb = entity.bounding_box.load().expand(16.0, 16.0, 16.0);
        world
            .get_entities_at_box(&bb)
            .into_iter()
            .filter(|e| e.get_entity().entity_type.id == EntityType::VEX.id)
            .count()
    }

    async fn perform_spell_casting(evoker: &EvokerEntity) {
        let entity = &evoker.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let origin = entity.block_pos.load();

        for _ in 0..3 {
            let offset = BlockPos::new(
                origin.0.x - 2 + rand::rng().random_range(0..5),
                origin.0.y + 1,
                origin.0.z - 2 + rand::rng().random_range(0..5),
            );
            let spawn_pos = Vector3::new(
                f64::from(offset.0.x) + 0.5,
                f64::from(offset.0.y),
                f64::from(offset.0.z) + 0.5,
            );
            let vex_base = from_type(&EntityType::VEX, spawn_pos, &world, uuid::Uuid::new_v4());
            if let Some(vex) = vex_base.cast_any().downcast_ref::<VexEntity>() {
                vex.set_owner(entity);
                vex.set_bound_origin(offset);
                // Vanilla: `20 * (30 + random.nextInt(90))`.
                vex.set_limited_life(20 * (30 + rand::rng().random_range(0..90)));
            }
            // Scope reduction: vanilla also adds the vex to the evoker's scoreboard team
            // (`serverLevel.getScoreboard().addPlayerToTeam`); Pumpkin has no team system to
            // mirror that into.
            world.spawn_entity(vex_base).await;
        }
    }
}

impl Goal for EvokerSummonSpellGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return false;
            };
            if !generic_use_spell_can_use(&evoker, self.timer.next_attack_tick_count).await {
                return false;
            }
            let nearby_vexes = Self::nearby_vex_count(&evoker);
            Self::vex_summon_allowed(nearby_vexes, rand::rng().random_range(1..=8))
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return false;
            };
            let target = evoker.mob_entity.target.lock().await.clone();
            target.is_some_and(|target| target.get_entity().is_alive())
                && self.timer.attack_warmup_delay > 0
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return;
            };
            let tick_count = evoker.mob_entity.living_entity.entity.age.load(Relaxed);
            self.timer.start(
                tick_count,
                20,
                Self::CASTING_TIME,
                Self::CASTING_INTERVAL,
                &evoker.spellcaster,
                IllagerSpell::SummonVex,
            );
            evoker
                .mob_entity
                .living_entity
                .entity
                .world
                .load()
                .play_sound(
                    Sound::EntityEvokerPrepareSummon,
                    SoundCategory::Hostile,
                    &evoker.mob_entity.living_entity.entity.pos.load(),
                );
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return;
            };
            if self.timer.tick() {
                Self::perform_spell_casting(&evoker).await;
                evoker
                    .mob_entity
                    .living_entity
                    .entity
                    .world
                    .load()
                    .play_sound(
                        Sound::EntityEvokerCastSpell,
                        SoundCategory::Hostile,
                        &evoker.mob_entity.living_entity.entity.pos.load(),
                    );
            }
        })
    }
}

/// Vanilla: `Evoker.EvokerWololoSpellGoal`. Recolors a nearby blue sheep to red. This is a real
/// vanilla easter egg (a reference to the "wololo" mind-control trope), not an invented feature.
pub struct EvokerWololoSpellGoal {
    evoker: Weak<EvokerEntity>,
    timer: SpellCastTimer,
}

impl EvokerWololoSpellGoal {
    const CAST_WARMUP_TIME: i32 = 40;
    const CASTING_TIME: i32 = 60;
    const CASTING_INTERVAL: i32 = 140;

    #[must_use]
    pub const fn new(evoker: Weak<EvokerEntity>) -> Self {
        Self {
            evoker,
            timer: SpellCastTimer::new(),
        }
    }

    fn find_blue_sheep(evoker: &EvokerEntity) -> Option<Arc<dyn EntityBase>> {
        let entity = &evoker.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let bb = entity.bounding_box.load().expand(16.0, 4.0, 16.0);

        let candidates: Vec<Arc<dyn EntityBase>> = world
            .get_entities_at_box(&bb)
            .into_iter()
            .filter(|e| {
                e.get_entity().is_alive()
                    && e.cast_any()
                        .downcast_ref::<SheepEntity>()
                        .is_some_and(|sheep| sheep.get_color() == DyeColor::Blue as u8)
            })
            .collect();

        if candidates.is_empty() {
            return None;
        }
        let idx = rand::rng().random_range(0..candidates.len());
        Some(candidates[idx].clone())
    }
}

impl Goal for EvokerWololoSpellGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return false;
            };
            if evoker.mob_entity.target.lock().await.is_some()
                || evoker.spellcaster.is_casting_spell()
            {
                return false;
            }
            let tick_count = evoker.mob_entity.living_entity.entity.age.load(Relaxed);
            if tick_count < self.timer.next_attack_tick_count {
                return false;
            }
            let world = evoker.mob_entity.living_entity.entity.world.load();
            if !world.level_info.load().game_rules.mob_griefing {
                return false;
            }

            let Some(sheep) = Self::find_blue_sheep(&evoker) else {
                return false;
            };
            *evoker.wololo_target.lock().await = Some(sheep);
            true
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return false;
            };
            evoker.wololo_target.lock().await.is_some() && self.timer.attack_warmup_delay > 0
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return;
            };
            let tick_count = evoker.mob_entity.living_entity.entity.age.load(Relaxed);
            self.timer.start(
                tick_count,
                Self::CAST_WARMUP_TIME,
                Self::CASTING_TIME,
                Self::CASTING_INTERVAL,
                &evoker.spellcaster,
                IllagerSpell::Wololo,
            );
            evoker
                .mob_entity
                .living_entity
                .entity
                .world
                .load()
                .play_sound(
                    Sound::EntityEvokerPrepareWololo,
                    SoundCategory::Hostile,
                    &evoker.mob_entity.living_entity.entity.pos.load(),
                );
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(evoker) = self.evoker.upgrade() {
                *evoker.wololo_target.lock().await = None;
            }
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(evoker) = self.evoker.upgrade() else {
                return;
            };
            if self.timer.tick() {
                let target = evoker.wololo_target.lock().await.clone();
                if let Some(target) = target
                    && target.get_entity().is_alive()
                    && let Some(sheep) = target.cast_any().downcast_ref::<SheepEntity>()
                {
                    sheep.set_color(DyeColor::Red as u8);
                }
                evoker
                    .mob_entity
                    .living_entity
                    .entity
                    .world
                    .load()
                    .play_sound(
                        Sound::EntityEvokerCastSpell,
                        SoundCategory::Hostile,
                        &evoker.mob_entity.living_entity.entity.pos.load(),
                    );
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{EvokerAttackSpellGoal, EvokerSummonSpellGoal};

    #[test]
    fn fang_layout_switches_between_close_and_far_patterns() {
        // Vanilla: `distanceToSqr(target) < 9.0` picks the 5+8 "burst" pattern, otherwise the
        // 16-fang "line" pattern.
        let close = EvokerAttackSpellGoal::fang_layout(0.0, 0.0, 1.0, 1.0, 2.0);
        assert_eq!(close.len(), 13);
        let far = EvokerAttackSpellGoal::fang_layout(0.0, 0.0, 10.0, 0.0, 100.0);
        assert_eq!(far.len(), 16);
        // The far pattern's delay ticks stagger 0..16 in spawn order.
        assert_eq!(far[0].3, 0);
        assert_eq!(far[15].3, 15);
    }

    #[test]
    fn vex_summon_gate_matches_vanilla_roll() {
        // Vanilla: `random.nextInt(8) + 1 > vexes` (roll is uniform in 1..=8).
        assert!(EvokerSummonSpellGoal::vex_summon_allowed(0, 1));
        assert!(!EvokerSummonSpellGoal::vex_summon_allowed(8, 8));
        assert!(EvokerSummonSpellGoal::vex_summon_allowed(3, 8));
        assert!(!EvokerSummonSpellGoal::vex_summon_allowed(3, 3));
    }
}
