use super::{
    World,
    weather_profile::{ClientEffect, PhaseTiming, WeatherCycle, WeatherProfile},
};
use crate::plugin::api::events::world::weather_change::{
    ThunderChangeEvent, WeatherCause, WeatherChangeEvent, WeatherPhaseChangeEvent,
};
use pumpkin_data::dimension::Dimension;
use pumpkin_protocol::bedrock::client::level_event::{CLevelEvent, LevelEvent};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{CGameEvent, GameEvent};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world_info::{
    LevelData,
    data_files::{SavedWeatherPhase, read_weather, write_weather},
};
use std::sync::{Arc, Mutex};
use tracing::error;

const WEATHER_TRANSITION_SPEED: f32 = 0.01;

/// Rain level above which a world counts as raining.
const RAINING_LEVEL: f32 = 0.2;
/// Thunder level above which a world counts as thundering.
const THUNDERING_LEVEL: f32 = 0.9;

/// Vanilla `Level.canHaveWeather`.
#[must_use]
pub fn can_have_weather(dimension: &Dimension) -> bool {
    dimension.has_skylight
        && !dimension.has_ceiling
        && dimension.minecraft_name != Dimension::THE_END.minecraft_name
}

/// A custom phase's running state.
#[derive(Clone, Copy, Default)]
pub struct PhaseState {
    pub time: i32,
    pub active: bool,
}

/// A weather cycle (vanilla `WeatherData`), plus the custom phases of its profile.
#[derive(Clone, Default)]
pub struct WeatherData {
    pub clear_weather_time: i32,
    pub raining: bool,
    pub rain_time: i32,
    pub thundering: bool,
    pub thunder_time: i32,
    pub profile: Arc<WeatherProfile>,
    /// The state of each of `profile.phases`.
    pub phases: Vec<PhaseState>,
}

/// The weather a world is loaded with.
#[derive(Default)]
pub struct WeatherSetup {
    pub data: SharedWeatherData,
    pub cycle: WeatherCycle,
    /// Overrides whether the world has weather; the dimension decides if unset.
    pub enabled: Option<bool>,
}

pub type SharedWeatherData = Arc<Mutex<WeatherData>>;

impl WeatherData {
    #[must_use]
    pub fn from_level_data(data: &LevelData) -> Self {
        Self {
            clear_weather_time: data.clear_weather_time,
            raining: data.raining,
            rain_time: data.rain_time,
            thundering: data.thundering,
            thunder_time: data.thunder_time,
            ..Self::default()
        }
    }

    /// Switches to another profile; custom phases start inactive.
    #[must_use]
    pub fn with_profile(mut self, profile: Arc<WeatherProfile>) -> Self {
        self.phases = vec![PhaseState::default(); profile.phases.len()];
        self.profile = profile;
        self
    }

    /// Copies the weather into `data`, for the world to be saved with it.
    pub fn store_in(&self, data: &mut LevelData) {
        data.weather_phases = self.saved_phases();
        data.clear_weather_time = self.clear_weather_time;
        data.raining = self.raining;
        data.rain_time = self.rain_time;
        data.thundering = self.thundering;
        data.thunder_time = self.thunder_time;
    }

    /// Vanilla `MinecraftServer.setWeatherParameters`.
    pub const fn set_parameters(
        &mut self,
        clear_time: i32,
        rain_time: i32,
        raining: bool,
        thundering: bool,
    ) {
        self.clear_weather_time = clear_time;
        self.rain_time = rain_time;
        self.thunder_time = rain_time;
        self.raining = raining;
        self.thundering = thundering;
    }

    /// Vanilla `ServerLevel.resetWeatherCycle`; the clear time is kept.
    pub fn reset_cycle(&mut self) {
        self.rain_time = 0;
        self.raining = false;
        self.thunder_time = 0;
        self.thundering = false;
        self.phases.fill(PhaseState::default());
    }

    /// Starts or ends a custom phase. Without `ticks` it lasts its profile's duration,
    /// or waits its delay when ended.
    pub fn set_phase(&mut self, id: &str, active: bool, ticks: Option<i32>) -> Result<(), String> {
        let index = self
            .profile
            .phases
            .iter()
            .position(|phase| phase.id == id)
            .ok_or_else(|| format!("unknown weather phase '{id}'"))?;
        let timing = self.profile.phases[index].timing();
        let range = if active {
            timing.duration
        } else {
            timing.delay
        };
        self.phases[index] = PhaseState {
            time: ticks.map_or_else(|| range.sample(&mut rand::rng()), |ticks| ticks.max(0)),
            active,
        };
        Ok(())
    }

    /// What clients should see: the cycle's rain and storm, plus any active phase's effect.
    #[must_use]
    pub fn effective(&self) -> (bool, bool) {
        let (mut raining, mut thundering) = (self.raining, self.thundering);
        for (phase, state) in self.profile.phases.iter().zip(&self.phases) {
            if !state.active {
                continue;
            }
            match phase.client_effect {
                ClientEffect::None => {}
                ClientEffect::Rain => raining = true,
                ClientEffect::Thunder => {
                    raining = true;
                    thundering = true;
                }
            }
        }
        (raining, thundering)
    }

    /// Copies the cycle into a `weather.dat`.
    pub fn store_in_file(&self, file: &mut pumpkin_world::world_info::data_files::WeatherData) {
        file.clear_weather_time = self.clear_weather_time;
        file.raining = self.raining;
        file.rain_time = self.rain_time;
        file.thundering = self.thundering;
        file.thunder_time = self.thunder_time;
        file.custom_phases = self.saved_phases();
    }

    fn saved_phases(&self) -> Vec<SavedWeatherPhase> {
        self.profile
            .phases
            .iter()
            .zip(&self.phases)
            .map(|(phase, state)| SavedWeatherPhase {
                id: phase.id.clone(),
                time: state.time,
                active: state.active,
            })
            .collect()
    }

    /// Takes the saved state of the phases the profile still has.
    pub fn restore_phases(&mut self, saved: &[SavedWeatherPhase]) {
        for (phase, state) in self.profile.phases.iter().zip(&mut self.phases) {
            if let Some(saved) = saved.iter().find(|saved| saved.id == phase.id) {
                *state = PhaseState {
                    time: saved.time.max(0),
                    active: saved.active,
                };
            }
        }
    }

    /// A cycle read back from a `weather.dat`.
    #[must_use]
    pub fn from_file(file: &pumpkin_world::world_info::data_files::WeatherData) -> Self {
        Self {
            clear_weather_time: file.clear_weather_time,
            raining: file.raining,
            rain_time: file.rain_time,
            thundering: file.thundering,
            thunder_time: file.thunder_time,
            ..Self::default()
        }
    }

    /// One tick of the cycle (vanilla `ServerLevel.advanceWeatherCycle`).
    pub fn advance_cycle(&mut self) {
        let mut rng = rand::rng();
        if self.clear_weather_time > 0 {
            self.clear_weather_time -= 1;
            self.thunder_time = i32::from(!self.thundering);
            self.rain_time = i32::from(!self.raining);
            self.thundering = false;
            self.raining = false;
        } else {
            Self::advance_phase(
                &mut self.thunder_time,
                &mut self.thundering,
                self.profile.thunder,
                &mut rng,
            );
            Self::advance_phase(
                &mut self.rain_time,
                &mut self.raining,
                self.profile.rain,
                &mut rng,
            );
        }

        for (phase, state) in self.profile.phases.iter().zip(&mut self.phases) {
            Self::advance_phase(&mut state.time, &mut state.active, phase.timing(), &mut rng);
        }
    }

    /// Counts a timer down and flips the state when it runs out; an expired timer is re-rolled.
    fn advance_phase(
        time: &mut i32,
        active: &mut bool,
        timing: PhaseTiming,
        rng: &mut impl rand::Rng,
    ) {
        if *time > 0 {
            *time -= 1;
            if *time == 0 {
                *active = !*active;
            }
        } else if *active {
            *time = timing.duration.sample(rng);
        } else {
            *time = timing.delay.sample(rng);
        }
    }
}

/// A snapshot of a world's weather.
pub struct WeatherState {
    pub clear_weather_time: i32,
    pub rain_time: i32,
    pub thunder_time: i32,
    pub raining: bool,
    pub thundering: bool,
    pub rain_level: f32,
    pub thunder_level: f32,
    /// The custom phases, by id.
    pub phases: Vec<(String, PhaseState)>,
}

/// Bedrock's intensity for a level from 0 to 1.
fn bedrock_intensity(level: f32) -> i32 {
    (level.clamp(0.0, 1.0) * 65535.0) as i32
}

/// A rain level as both editions send it.
fn rain_level_packets(level: f32) -> (CGameEvent, CLevelEvent) {
    let event = if level > 0.0 {
        LevelEvent::StartRaining
    } else {
        LevelEvent::StopRaining
    };
    (
        CGameEvent::new(GameEvent::RainLevelChange, level),
        weather_level_event(event, level),
    )
}

/// A thunder level as both editions send it.
fn thunder_level_packets(level: f32) -> (CGameEvent, CLevelEvent) {
    let event = if level > 0.0 {
        LevelEvent::StartThunderstorm
    } else {
        LevelEvent::StopThunderstorm
    };
    (
        CGameEvent::new(GameEvent::ThunderLevelChange, level),
        weather_level_event(event, level),
    )
}

fn weather_level_event(event: LevelEvent, level: f32) -> CLevelEvent {
    CLevelEvent {
        event_id: VarInt(event as i32),
        position: Vector3::new(0.0, 0.0, 0.0),
        data: VarInt(bedrock_intensity(level)),
    }
}

/// What a tick of a world's weather levels changed.
pub struct WeatherTick {
    pub rain_level: f32,
    pub thunder_level: f32,
    pub rain_changed: bool,
    pub thunder_changed: bool,
    pub raining_changed: bool,
}

/// A world's rain and thunder levels, which follow the shared [`WeatherData`].
#[derive(Clone, Copy, Default)]
pub struct Weather {
    pub rain_level: f32,
    pub old_rain_level: f32,
    pub thunder_level: f32,
    pub old_thunder_level: f32,
}

impl Weather {
    /// The levels a saved world resumes with (vanilla `prepareWeather`): a rain or
    /// storm already under way is at full strength.
    #[must_use]
    pub fn prepared(data: &WeatherData) -> Self {
        let (raining, thundering) = data.effective();
        let rain_level = if raining { 1.0 } else { 0.0 };
        let thunder_level = if raining && thundering { 1.0 } else { 0.0 };
        Self {
            rain_level,
            old_rain_level: rain_level,
            thunder_level,
            old_thunder_level: thunder_level,
        }
    }

    #[must_use]
    pub fn is_raining(&self) -> bool {
        self.rain_level > RAINING_LEVEL
    }

    #[must_use]
    pub fn is_thundering(&self) -> bool {
        self.thunder_level > THUNDERING_LEVEL
    }

    fn tick(&mut self, data: &WeatherData) -> WeatherTick {
        let was_raining = self.is_raining();
        self.old_rain_level = self.rain_level;
        self.old_thunder_level = self.thunder_level;

        let step = |level: f32, on: bool| {
            if on {
                (level + WEATHER_TRANSITION_SPEED).min(1.0)
            } else {
                (level - WEATHER_TRANSITION_SPEED).max(0.0)
            }
        };
        let (raining, thundering) = data.effective();
        self.rain_level = step(self.rain_level, raining);
        self.thunder_level = step(self.thunder_level, thundering);

        WeatherTick {
            rain_level: self.rain_level,
            thunder_level: self.thunder_level,
            rain_changed: self.old_rain_level != self.rain_level,
            thunder_changed: self.old_thunder_level != self.thunder_level,
            raining_changed: was_raining != self.is_raining(),
        }
    }
}

/// What changed in a weather cycle: rain, thunder or one custom phase.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Flag {
    Rain,
    Thunder,
    Phase(usize),
}

/// The on/off states of a weather cycle, to see what a change flipped.
struct Flags {
    raining: bool,
    thundering: bool,
    phases: Vec<bool>,
}

impl WeatherData {
    fn flags(&self) -> Flags {
        Flags {
            raining: self.raining,
            thundering: self.thundering,
            phases: self.phases.iter().map(|phase| phase.active).collect(),
        }
    }

    fn flips_since(&self, before: &Flags) -> Vec<(Flag, bool)> {
        let mut flips = Vec::new();
        if self.raining != before.raining {
            flips.push((Flag::Rain, self.raining));
        }
        if self.thundering != before.thundering {
            flips.push((Flag::Thunder, self.thundering));
        }
        for (i, (phase, was)) in self.phases.iter().zip(&before.phases).enumerate() {
            if phase.active != *was {
                flips.push((Flag::Phase(i), phase.active));
            }
        }
        flips
    }

    /// Puts a flag back to how it was before a cancelled change.
    fn restore(&mut self, flag: Flag, before: &Flags) {
        match flag {
            Flag::Rain => self.raining = before.raining,
            Flag::Thunder => self.thundering = before.thundering,
            Flag::Phase(i) => {
                if let (Some(phase), Some(was)) = (self.phases.get_mut(i), before.phases.get(i)) {
                    phase.active = *was;
                }
            }
        }
    }
}

impl World {
    #[must_use]
    pub const fn can_have_weather(&self) -> bool {
        self.weather_enabled
    }

    fn lock_weather_data(&self) -> std::sync::MutexGuard<'_, WeatherData> {
        self.weather_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn lock_weather(&self) -> std::sync::MutexGuard<'_, Weather> {
        self.weather
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Vanilla `Level.isRaining`.
    #[must_use]
    pub fn is_raining(&self) -> bool {
        self.can_have_weather() && self.lock_weather().is_raining()
    }

    /// Vanilla `Level.isThundering`.
    #[must_use]
    pub fn is_thundering(&self) -> bool {
        self.can_have_weather() && self.lock_weather().is_thundering()
    }

    /// The current rain and thunder levels.
    #[must_use]
    pub fn weather_levels(&self) -> (f32, f32) {
        let weather = self.lock_weather();
        (weather.rain_level, weather.thunder_level)
    }

    /// Changes the weather cycle and fires an event for everything that flipped. A
    /// cancelled flip is put back. Flips caused by a `/weather clear` countdown can not be
    /// cancelled, or they would flip again every tick.
    pub fn change_weather(
        self: &Arc<Self>,
        cause: WeatherCause,
        edit: impl FnOnce(&mut WeatherData),
    ) {
        let (before, flips, profile, cancellable) = {
            let mut data = self.lock_weather_data();
            let cancellable = cause != WeatherCause::Natural || data.clear_weather_time <= 0;
            let before = data.flags();
            edit(&mut data);
            let flips = data.flips_since(&before);
            (before, flips, data.profile.clone(), cancellable)
        };
        let Some(server) = self.server.upgrade().filter(|_| !flips.is_empty()) else {
            return;
        };

        for (flag, to) in flips {
            let cancelled = match flag {
                Flag::Rain => {
                    let mut event = WeatherChangeEvent::new(self.clone(), to, cause);
                    server.plugin_manager.fire_blocking(&server, &mut event);
                    event.cancelled
                }
                Flag::Thunder => {
                    let mut event = ThunderChangeEvent::new(self.clone(), to, cause);
                    server.plugin_manager.fire_blocking(&server, &mut event);
                    event.cancelled
                }
                Flag::Phase(i) => {
                    let Some(phase) = profile.phases.get(i) else {
                        continue;
                    };
                    let mut event =
                        WeatherPhaseChangeEvent::new(self.clone(), phase.id.clone(), to, cause);
                    server.plugin_manager.fire_blocking(&server, &mut event);
                    event.cancelled
                }
            };
            if cancelled && cancellable {
                self.lock_weather_data().restore(flag, &before);
            }
        }
    }

    /// Starts or stops rain in this world's weather cycle.
    pub fn set_raining(self: &Arc<Self>, raining: bool) {
        self.change_weather(WeatherCause::Plugin, |data| {
            if data.raining != raining {
                let thundering = data.thundering;
                data.set_parameters(0, 0, raining, thundering);
            }
        });
    }

    /// Starts or stops thunder in this world's weather cycle.
    pub fn set_thundering(self: &Arc<Self>, thundering: bool) {
        self.change_weather(WeatherCause::Plugin, |data| {
            if data.thundering != thundering {
                let raining = data.raining;
                data.set_parameters(0, 0, raining, thundering);
            }
        });
    }

    /// Sets the weather like `/weather` does.
    pub fn set_weather(
        self: &Arc<Self>,
        cause: WeatherCause,
        clear_time: i32,
        rain_time: i32,
        raining: bool,
        thundering: bool,
    ) {
        self.change_weather(cause, |data| {
            data.set_parameters(clear_time, rain_time, raining, thundering);
        });
    }

    /// Starts or ends one of the weather profile's custom phases.
    pub fn set_weather_phase(
        self: &Arc<Self>,
        id: &str,
        active: bool,
        ticks: Option<i32>,
    ) -> Result<(), String> {
        let mut result = Ok(());
        self.change_weather(WeatherCause::Plugin, |data| {
            result = data.set_phase(id, active, ticks);
        });
        result
    }

    /// Sets the ticks left until the clear sky ends, the rain flips or the thunder flips.
    pub fn set_weather_durations(
        &self,
        clear: Option<i32>,
        rain: Option<i32>,
        thunder: Option<i32>,
    ) {
        let mut data = self.lock_weather_data();
        if let Some(clear) = clear {
            data.clear_weather_time = clear.max(0);
        }
        if let Some(rain) = rain {
            data.rain_time = rain.max(0);
        }
        if let Some(thunder) = thunder {
            data.thunder_time = thunder.max(0);
        }
    }

    /// A snapshot of this world's weather.
    #[must_use]
    pub fn weather_state(&self) -> WeatherState {
        let (rain_level, thunder_level) = self.weather_levels();
        let data = self.lock_weather_data();
        WeatherState {
            clear_weather_time: data.clear_weather_time,
            rain_time: data.rain_time,
            thunder_time: data.thunder_time,
            raining: data.raining,
            thundering: data.thundering,
            rain_level,
            thunder_level,
            phases: data
                .profile
                .phases
                .iter()
                .zip(&data.phases)
                .map(|(phase, state)| (phase.id.clone(), *state))
                .collect(),
        }
    }

    /// Advances the cycle and moves this world's levels towards it
    /// (vanilla `ServerLevel.advanceWeatherCycle`).
    pub fn tick_weather(self: &Arc<Self>) {
        if !self.can_have_weather() {
            return;
        }

        if self.level_info.load().game_rules.advance_weather
            && !matches!(self.weather_cycle, WeatherCycle::Fixed { .. })
        {
            self.change_weather(WeatherCause::Natural, WeatherData::advance_cycle);
        }
        let tick = {
            let data = self.lock_weather_data();
            self.lock_weather().tick(&data)
        };

        if tick.rain_changed {
            let (je, be) = rain_level_packets(tick.rain_level);
            self.broadcast_editioned(&je, &be);
        }
        if tick.thunder_changed {
            let (je, be) = thunder_level_packets(tick.thunder_level);
            self.broadcast_editioned(&je, &be);
        }
        if tick.raining_changed {
            self.announce_raining(&tick);
        }
    }

    /// Runs `send` for every world following the same weather cycle as this one. Like
    /// vanilla's single cycle, all of them hear about rain starting or stopping.
    fn for_each_cycle_world(&self, send: impl Fn(&Self)) {
        let Some(server) = self.server.upgrade() else {
            send(self);
            return;
        };
        for world in server.worlds.load().iter() {
            if Arc::ptr_eq(&world.weather_data, &self.weather_data) {
                send(world);
            }
        }
    }

    fn announce_raining(&self, tick: &WeatherTick) {
        let event = if self.is_raining() {
            GameEvent::BeginRaining
        } else {
            GameEvent::EndRaining
        };
        let begin_or_end = CGameEvent::new(event, 0.0);
        let (rain_je, rain_be) = rain_level_packets(tick.rain_level);
        let (thunder_je, thunder_be) = thunder_level_packets(tick.thunder_level);
        // Bedrock has no begin or end; its level events carry both.
        self.for_each_cycle_world(|world| {
            world.broadcast_packet_all(&begin_or_end);
            world.broadcast_editioned(&rain_je, &rain_be);
            world.broadcast_editioned(&thunder_je, &thunder_be);
        });
    }

    /// Vanilla's wake-up weather reset: sleeping through a night ends the rain.
    pub fn reset_weather_after_sleep(self: &Arc<Self>) {
        if self.level_info.load().game_rules.advance_weather
            && !matches!(self.weather_cycle, WeatherCycle::Fixed { .. })
            && self.is_raining()
        {
            self.change_weather(WeatherCause::Sleep, WeatherData::reset_cycle);
        }
    }

    /// Saves an own cycle with the world; shared cycles are saved with the server.
    pub(super) fn save_own_weather(&self) {
        if self.weather_cycle != WeatherCycle::Own {
            return;
        }
        let folder = &self.level.level_folder.root_folder;
        let mut file = read_weather(folder);
        self.lock_weather_data().store_in_file(&mut file);
        if let Err(e) = write_weather(folder, &file) {
            error!(
                "Failed to save the weather of {}: {e}",
                self.get_world_name()
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clear_time_stops_the_weather() {
        let mut data = WeatherData {
            clear_weather_time: 2,
            raining: true,
            thundering: true,
            ..WeatherData::default()
        };
        data.advance_cycle();
        assert_eq!(data.clear_weather_time, 1);
        assert!(!data.raining && !data.thundering);
        // The timers are set so the weather flips back once the clear time is over.
        assert_eq!((data.rain_time, data.thunder_time), (0, 0));
    }

    #[test]
    fn expired_timers_flip_and_reroll() {
        let mut data = WeatherData {
            rain_time: 1,
            thunder_time: 1,
            ..WeatherData::default()
        };
        data.advance_cycle();
        assert!(data.raining && data.thundering);
        data.advance_cycle();
        assert!((12_000..=24_000).contains(&data.rain_time));
        assert!((3_600..=15_600).contains(&data.thunder_time));
    }

    #[test]
    fn reset_keeps_the_clear_time() {
        let mut data = WeatherData::default();
        data.set_parameters(50, 100, true, true);
        assert_eq!(data.thunder_time, 100);
        data.reset_cycle();
        assert_eq!(data.clear_weather_time, 50);
        assert!(!data.raining && !data.thundering);
    }

    #[test]
    fn prepared_levels_need_rain_for_thunder() {
        let storm = Weather::prepared(&WeatherData {
            raining: true,
            thundering: true,
            ..WeatherData::default()
        });
        assert!(storm.is_raining() && storm.is_thundering());
        let dry_thunder = Weather::prepared(&WeatherData {
            thundering: true,
            ..WeatherData::default()
        });
        assert_eq!(
            (dry_thunder.rain_level, dry_thunder.thunder_level),
            (0.0, 0.0)
        );
    }

    #[test]
    fn levels_ease_towards_the_cycle() {
        let data = WeatherData {
            raining: true,
            ..WeatherData::default()
        };
        let mut weather = Weather::default();
        let tick = weather.tick(&data);
        assert!(tick.rain_changed && !tick.thunder_changed && !tick.raining_changed);
        for _ in 0..20 {
            weather.tick(&data);
        }
        assert!(weather.is_raining());
    }

    fn blood_moon() -> WeatherData {
        let profile = WeatherProfile::from_json(
            r#"{"phases":[{"id":"blood_moon","delay":[5,5],"duration":[3,3],"client_effect":"thunder"}]}"#,
        )
        .unwrap();
        WeatherData::default().with_profile(Arc::new(profile))
    }

    #[test]
    fn custom_phase_runs_its_own_cycle() {
        let mut data = blood_moon();
        data.advance_cycle(); // rolls the delay
        assert_eq!(data.phases[0].time, 5);
        for _ in 0..5 {
            data.advance_cycle();
        }
        assert!(data.phases[0].active);
        // The storm shows to clients although the vanilla cycle is dry.
        assert_eq!(data.effective(), (true, true));
        assert!(!data.raining && !data.thundering);
        for _ in 0..4 {
            data.advance_cycle();
        }
        assert!(!data.phases[0].active);
    }

    #[test]
    fn phases_can_be_set_by_hand() {
        let mut data = blood_moon();
        data.set_phase("blood_moon", true, Some(100)).unwrap();
        assert_eq!((data.phases[0].active, data.phases[0].time), (true, 100));
        assert!(data.set_phase("nope", true, None).is_err());
        data.reset_cycle();
        assert!(!data.phases[0].active);
    }

    #[test]
    fn flips_are_seen_and_restored() {
        let mut data = blood_moon();
        let before = data.flags();
        data.raining = true;
        data.phases[0].active = true;
        let flips = data.flips_since(&before);
        assert_eq!(flips.len(), 2);
        for (flag, _) in flips {
            data.restore(flag, &before);
        }
        assert!(!data.raining && !data.phases[0].active);
    }

    #[test]
    fn phases_survive_a_save() {
        let mut data = blood_moon();
        data.set_phase("blood_moon", true, Some(77)).unwrap();
        let mut file = pumpkin_world::world_info::data_files::WeatherData::default();
        data.store_in_file(&mut file);

        let mut loaded = WeatherData::from_file(&file).with_profile(data.profile.clone());
        loaded.restore_phases(&file.custom_phases);
        assert_eq!((loaded.phases[0].active, loaded.phases[0].time), (true, 77));
        // A phase the profile no longer has is dropped quietly.
        let mut other = WeatherData::default();
        other.restore_phases(&file.custom_phases);
        assert!(other.phases.is_empty());
    }

    #[test]
    fn bedrock_levels_use_start_and_stop_events() {
        let (_, start) = rain_level_packets(0.5);
        assert_eq!(start.event_id.0, LevelEvent::StartRaining as i32);
        assert_eq!(start.data.0, 32767);
        let (_, stop) = rain_level_packets(0.0);
        assert_eq!(stop.event_id.0, LevelEvent::StopRaining as i32);
        assert_eq!(stop.data.0, 0);
        let (_, storm) = thunder_level_packets(1.0);
        assert_eq!(storm.event_id.0, LevelEvent::StartThunderstorm as i32);
        assert_eq!(storm.data.0, 65535);
    }

    #[test]
    fn only_skylit_open_dimensions_have_weather() {
        assert!(can_have_weather(&Dimension::OVERWORLD));
        assert!(!can_have_weather(&Dimension::THE_NETHER));
        assert!(!can_have_weather(&Dimension::THE_END));
    }
}
