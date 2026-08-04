use std::sync::Weak;
use std::sync::atomic::Ordering::Relaxed;

use crossbeam::atomic::AtomicCell;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::Difficulty;

use crate::entity::ai::goal::spellcaster::{IllagerSpell, SpellCastTimer};
use crate::entity::ai::goal::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::entity::mob::illusioner::IllusionerEntity;

/// Vanilla: `Illusioner.registerGoals`'s `SpellcasterIllager.SpellcasterCastingSpellGoal`. Stops
/// navigation and keeps the illusioner looking at its target while a spell is active.
pub struct IllusionerCastingSpellGoal {
    illusioner: Weak<IllusionerEntity>,
}

impl IllusionerCastingSpellGoal {
    #[must_use]
    pub const fn new(illusioner: Weak<IllusionerEntity>) -> Self {
        Self { illusioner }
    }
}

impl Goal for IllusionerCastingSpellGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.illusioner
                .upgrade()
                .is_some_and(|illusioner| illusioner.spellcaster.is_casting_spell())
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.illusioner
                .upgrade()
                .is_some_and(|illusioner| illusioner.spellcaster.is_casting_spell())
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(illusioner) = self.illusioner.upgrade() {
                illusioner.mob_entity.navigator.lock().unwrap().stop();
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(illusioner) = self.illusioner.upgrade() {
                illusioner.spellcaster.set_current_spell(IllagerSpell::None);
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(illusioner) = self.illusioner.upgrade() else {
                return;
            };
            let target = illusioner.mob_entity.target.lock().await.clone();
            if let Some(target) = target {
                illusioner
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
/// gate each concrete goal adds on top.
async fn generic_use_spell_can_use(
    illusioner: &IllusionerEntity,
    next_attack_tick_count: i32,
) -> bool {
    let target = illusioner.mob_entity.target.lock().await.clone();
    let Some(target) = target else {
        return false;
    };
    if !target.get_entity().is_alive() || illusioner.spellcaster.is_casting_spell() {
        return false;
    }
    illusioner.mob_entity.living_entity.entity.age.load(Relaxed) >= next_attack_tick_count
}

/// Vanilla: `Illusioner.IllusionerMirrorSpellGoal`. Turns the illusioner invisible.
pub struct IllusionerMirrorSpellGoal {
    illusioner: Weak<IllusionerEntity>,
    timer: SpellCastTimer,
}

impl IllusionerMirrorSpellGoal {
    const CASTING_TIME: i32 = 20;
    const CASTING_INTERVAL: i32 = 340;

    #[must_use]
    pub const fn new(illusioner: Weak<IllusionerEntity>) -> Self {
        Self {
            illusioner,
            timer: SpellCastTimer::new(),
        }
    }
}

impl Goal for IllusionerMirrorSpellGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(illusioner) = self.illusioner.upgrade() else {
                return false;
            };
            if !generic_use_spell_can_use(&illusioner, self.timer.next_attack_tick_count).await {
                return false;
            }
            illusioner
                .mob_entity
                .living_entity
                .get_effect(&StatusEffect::INVISIBILITY)
                .await
                .is_none()
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(illusioner) = self.illusioner.upgrade() else {
                return false;
            };
            let target = illusioner.mob_entity.target.lock().await.clone();
            target.is_some_and(|target| target.get_entity().is_alive())
                && self.timer.attack_warmup_delay > 0
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(illusioner) = self.illusioner.upgrade() else {
                return;
            };
            let tick_count = illusioner.mob_entity.living_entity.entity.age.load(Relaxed);
            self.timer.start(
                tick_count,
                20,
                Self::CASTING_TIME,
                Self::CASTING_INTERVAL,
                &illusioner.spellcaster,
                IllagerSpell::Disappear,
            );
            illusioner
                .mob_entity
                .living_entity
                .entity
                .world
                .load()
                .play_sound(
                    Sound::EntityIllusionerPrepareMirror,
                    SoundCategory::Hostile,
                    &illusioner.mob_entity.living_entity.entity.pos.load(),
                );
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(illusioner) = self.illusioner.upgrade() else {
                return;
            };
            if self.timer.tick() {
                illusioner
                    .mob_entity
                    .living_entity
                    .add_effect(Effect {
                        effect_type: &StatusEffect::INVISIBILITY,
                        duration: 1200,
                        amplifier: 0,
                        ambient: false,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    })
                    .await;
                illusioner
                    .mob_entity
                    .living_entity
                    .entity
                    .world
                    .load()
                    .play_sound(
                        Sound::EntityIllusionerCastSpell,
                        SoundCategory::Hostile,
                        &illusioner.mob_entity.living_entity.entity.pos.load(),
                    );
            }
        })
    }
}

/// Vanilla: `Illusioner.IllusionerBlindnessSpellGoal`. Blinds the current target, but only once
/// per new target and only on `Hard` local difficulty (`isHarderThan(NORMAL)`).
pub struct IllusionerBlindnessSpellGoal {
    illusioner: Weak<IllusionerEntity>,
    timer: SpellCastTimer,
    last_target_id: AtomicCell<i32>,
}

impl IllusionerBlindnessSpellGoal {
    const CASTING_TIME: i32 = 20;
    const CASTING_INTERVAL: i32 = 180;

    #[must_use]
    pub const fn new(illusioner: Weak<IllusionerEntity>) -> Self {
        Self {
            illusioner,
            timer: SpellCastTimer::new(),
            last_target_id: AtomicCell::new(0),
        }
    }
}

impl Goal for IllusionerBlindnessSpellGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(illusioner) = self.illusioner.upgrade() else {
                return false;
            };
            if !generic_use_spell_can_use(&illusioner, self.timer.next_attack_tick_count).await {
                return false;
            }
            let Some(target) = illusioner.mob_entity.target.lock().await.clone() else {
                return false;
            };
            if target.get_entity().entity_id == self.last_target_id.load() {
                return false;
            }
            let world = illusioner.mob_entity.living_entity.entity.world.load();
            // Vanilla: `DifficultyInstance.isHarderThan(Difficulty.NORMAL.ordinal())`; `Hard` is
            // the only `Difficulty` variant ordered after `Normal`.
            world.level_info.load().difficulty == Difficulty::Hard
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(illusioner) = self.illusioner.upgrade() else {
                return false;
            };
            let target = illusioner.mob_entity.target.lock().await.clone();
            target.is_some_and(|target| target.get_entity().is_alive())
                && self.timer.attack_warmup_delay > 0
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(illusioner) = self.illusioner.upgrade() else {
                return;
            };
            let tick_count = illusioner.mob_entity.living_entity.entity.age.load(Relaxed);
            self.timer.start(
                tick_count,
                20,
                Self::CASTING_TIME,
                Self::CASTING_INTERVAL,
                &illusioner.spellcaster,
                IllagerSpell::Blindness,
            );
            if let Some(target) = illusioner.mob_entity.target.lock().await.as_ref() {
                self.last_target_id.store(target.get_entity().entity_id);
            }
            illusioner
                .mob_entity
                .living_entity
                .entity
                .world
                .load()
                .play_sound(
                    Sound::EntityIllusionerPrepareBlindness,
                    SoundCategory::Hostile,
                    &illusioner.mob_entity.living_entity.entity.pos.load(),
                );
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(illusioner) = self.illusioner.upgrade() else {
                return;
            };
            if self.timer.tick() {
                let target = illusioner.mob_entity.target.lock().await.clone();
                if let Some(target) = target
                    && let Some(living) = target.get_living_entity()
                {
                    living
                        .add_effect(Effect {
                            effect_type: &StatusEffect::BLINDNESS,
                            duration: 400,
                            amplifier: 0,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                            blend: false,
                        })
                        .await;
                }
                illusioner
                    .mob_entity
                    .living_entity
                    .entity
                    .world
                    .load()
                    .play_sound(
                        Sound::EntityIllusionerCastSpell,
                        SoundCategory::Hostile,
                        &illusioner.mob_entity.living_entity.entity.pos.load(),
                    );
            }
        })
    }
}
