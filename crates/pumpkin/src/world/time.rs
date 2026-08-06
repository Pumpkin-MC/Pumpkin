use pumpkin_protocol::{bedrock::client::set_time::CSetTime, java::client::play::CUpdateTime};

use super::World;

pub struct LevelTime {
    pub world_age: i64,
    pub time_of_day: i64,
    pub rain_time: i64,
}

impl Default for LevelTime {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelTime {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            world_age: 0,
            time_of_day: 0,
            rain_time: 0,
        }
    }

    #[must_use]
    pub const fn with_time_of_day(time_of_day: i64) -> Self {
        Self {
            world_age: 0,
            time_of_day,
            rain_time: 0,
        }
    }

    pub const fn tick_time(&mut self, advance_time: bool, advance_weather: bool) {
        self.world_age += 1;
        if advance_weather {
            self.rain_time += 1;
        }
        if advance_time {
            self.time_of_day += 1;
        }
    }

    pub async fn send_time(&self, world: &World) {
        let advance_time = {
            let lock = world.level_info.load();
            lock.game_rules.advance_time
        };

        world
            .broadcast_editioned(
                &CUpdateTime::new(self.world_age, self.time_of_day, advance_time),
                &CSetTime::new(self.time_of_day as _), // TODO do we need to tell bedrock that time is frozen?
            )
            .await;
    }

    pub const fn add_time(&mut self, time: i64) {
        self.time_of_day += time;
    }

    pub const fn set_time(&mut self, time: i64) {
        self.time_of_day = time;
    }

    #[must_use]
    pub const fn query_daytime(&self) -> i64 {
        self.time_of_day % 24000
    }

    #[must_use]
    pub const fn query_gametime(&self) -> i64 {
        self.world_age
    }

    #[must_use]
    pub const fn query_day(&self) -> i64 {
        self.time_of_day / 24000
    }

    #[must_use]
    pub const fn is_night(&self) -> bool {
        (self.time_of_day % 24000) >= 12000 && (self.time_of_day % 24000) <= 23999
    }

    /// Whether this tick opens the persistent (`CREATURE`) mob-category spawn
    /// gate.
    ///
    /// Mirrors vanilla's `ServerChunkCache.tickChunks`:
    /// `boolean spawnPersistent = this.level.getGameTime() % 400L == 0L;`.
    /// `getGameTime()` is backed by the unconditionally-incrementing game
    /// time (`world_age` here), not the day/night clock (`time_of_day`),
    /// which the `doDaylightCycle` game rule can freeze and which sleep-skips
    /// / `/time set` can jump independently of elapsed ticks.
    #[must_use]
    pub const fn opens_persistent_spawn_gate(&self) -> bool {
        self.world_age % 400 == 0
    }
}

#[cfg(test)]
mod test {
    use super::LevelTime;

    #[test]
    fn restores_persisted_time_of_day() {
        let time = LevelTime::with_time_of_day(36_001);

        assert_eq!(time.time_of_day, 36_001);
        assert_eq!(time.world_age, 0);
    }

    /// `opens_persistent_spawn_gate` (the function `World::tick` calls to
    /// decide `spawn_passives`) must key off `world_age`, which keeps
    /// advancing every tick regardless of the `doDaylightCycle` game rule
    /// (`advance_time`) — unlike `time_of_day`, which freezes when that rule
    /// is off and can also jump independently via sleep-skips / `/time set`.
    ///
    /// If the gate were keyed off `time_of_day` instead, disabling the
    /// daylight cycle would permanently stop `CREATURE`-category natural
    /// spawns (cows, sheep, pigs, chickens, horses, ...) the moment
    /// `time_of_day` settled on a non-multiple of 400, since it would then
    /// never change again.
    #[test]
    fn persistent_spawn_gate_opens_on_world_age_even_when_time_of_day_is_frozen() {
        let mut time = LevelTime::new();

        for _ in 0..500 {
            time.tick_time(false, false); // advance_time = false: doDaylightCycle off
        }

        assert_eq!(time.time_of_day, 0);
        assert_eq!(time.world_age, 500);
        assert!(!time.opens_persistent_spawn_gate());

        for _ in 0..300 {
            time.tick_time(false, false);
        }

        assert_eq!(time.time_of_day, 0, "frozen regardless of world_age");
        assert_eq!(time.world_age, 800);
        assert!(time.opens_persistent_spawn_gate());
    }
}
