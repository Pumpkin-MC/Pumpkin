use std::sync::atomic::{AtomicI32, AtomicU8, Ordering::Relaxed};

/// Vanilla: `SpellcasterIllager.IllagerSpell`. Each variant carries the RGB color used for the
/// client-side hand-particle effect in `SpellcasterIllager#tick`.
///
/// Scope reduction: Pumpkin has no per-tick client-side hand-particle spawning for mobs (that
/// `tick()` override lives entirely on the client render path in vanilla), so `color()` is kept
/// for structural completeness / a future particle implementation but is not currently consumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IllagerSpell {
    None = 0,
    SummonVex = 1,
    Fangs = 2,
    Wololo = 3,
    Disappear = 4,
    Blindness = 5,
}

impl IllagerSpell {
    #[must_use]
    pub const fn color(self) -> (f32, f32, f32) {
        match self {
            Self::None => (0.0, 0.0, 0.0),
            Self::SummonVex => (0.7, 0.7, 0.8),
            Self::Fangs => (0.4, 0.3, 0.35),
            Self::Wololo => (0.7, 0.5, 0.2),
            Self::Disappear => (0.3, 0.3, 0.8),
            Self::Blindness => (0.1, 0.1, 0.2),
        }
    }
}

/// Shared spellcasting timer state. Vanilla: `SpellcasterIllager.spellCastingTickCount` /
/// `currentSpell`.
#[derive(Default)]
pub struct SpellcasterState {
    spell_casting_tick_count: AtomicI32,
    current_spell: AtomicU8,
}

impl SpellcasterState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            spell_casting_tick_count: AtomicI32::new(0),
            current_spell: AtomicU8::new(IllagerSpell::None as u8),
        }
    }

    /// Vanilla: `isCastingSpell` (server side: `spellCastingTickCount > 0`).
    #[must_use]
    pub fn is_casting_spell(&self) -> bool {
        self.spell_casting_tick_count.load(Relaxed) > 0
    }

    #[must_use]
    pub fn casting_ticks_left(&self) -> i32 {
        self.spell_casting_tick_count.load(Relaxed)
    }

    pub fn set_casting_time(&self, ticks: i32) {
        self.spell_casting_tick_count.store(ticks, Relaxed);
    }

    /// Vanilla: `setIsCastingSpell`.
    pub fn set_current_spell(&self, spell: IllagerSpell) {
        self.current_spell.store(spell as u8, Relaxed);
    }

    /// Vanilla: `customServerAiStep` -- decrements the counter once per tick while positive.
    pub fn tick(&self) {
        if self.spell_casting_tick_count.load(Relaxed) > 0 {
            self.spell_casting_tick_count.fetch_sub(1, Relaxed);
        }
    }
}

/// Shared "warm up, then cast, then cool down" timer used by every concrete `SpellcasterUseSpellGoal` subclass in vanilla.
///
/// Each concrete goal owns one of these and supplies its own warmup/casting/interval durations.
#[derive(Default)]
pub struct SpellCastTimer {
    pub attack_warmup_delay: i32,
    pub next_attack_tick_count: i32,
}

impl SpellCastTimer {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            attack_warmup_delay: 0,
            next_attack_tick_count: 0,
        }
    }

    /// Vanilla: `SpellcasterUseSpellGoal#start`.
    pub fn start(
        &mut self,
        tick_count: i32,
        warmup_time: i32,
        casting_time: i32,
        casting_interval: i32,
        state: &SpellcasterState,
        spell: IllagerSpell,
    ) {
        self.attack_warmup_delay = warmup_time;
        state.set_casting_time(casting_time);
        self.next_attack_tick_count = tick_count + casting_interval;
        state.set_current_spell(spell);
    }

    /// Vanilla: `SpellcasterUseSpellGoal#tick`. Returns `true` on the exact tick the warmup
    /// reaches zero, i.e. when `performSpellCasting` should run.
    pub const fn tick(&mut self) -> bool {
        self.attack_warmup_delay -= 1;
        self.attack_warmup_delay == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_only_casts_while_ticks_remain() {
        let state = SpellcasterState::new();
        assert!(!state.is_casting_spell());
        state.set_casting_time(2);
        assert!(state.is_casting_spell());
        state.tick();
        assert!(state.is_casting_spell());
        state.tick();
        assert!(!state.is_casting_spell());
        // Further ticks must not underflow.
        state.tick();
        assert!(!state.is_casting_spell());
    }

    #[test]
    fn timer_signals_once_when_warmup_elapses() {
        let mut timer = SpellCastTimer::new();
        let state = SpellcasterState::new();
        timer.start(0, 3, 40, 100, &state, IllagerSpell::Fangs);
        assert_eq!(state.casting_ticks_left(), 40);
        assert_eq!(timer.next_attack_tick_count, 100);
        assert!(!timer.tick());
        assert!(!timer.tick());
        assert!(timer.tick());
    }
}
