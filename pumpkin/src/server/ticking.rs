use super::Server;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::task::JoinSet;
use tracing::error;

impl Server {
    /// Main server tick method. This now handles both player/network ticking (which always runs)
    /// and world/game logic ticking (which is affected by freeze state).
    pub async fn tick(self: &Arc<Self>) {
        if self.tick_rate_manager.runs_normally() || self.tick_rate_manager.is_sprinting() {
            self.tick_worlds().await;
            // Always run player and network ticking, even when game is frozen
        } else {
            self.tick_players_and_network().await;
        }
    }

    /// Ticks essential server functions that must run even when the game is frozen.
    /// This includes player ticking (network, keep-alives) and flushing world updates to clients.
    pub async fn tick_players_and_network(self: &Arc<Self>) {
        let worlds = self.worlds.load();

        for world in worlds.iter() {
            world.flush_block_updates().await;
            world.flush_synced_block_events().await;
        }

        let mut set = JoinSet::new();
        for world in worlds.iter() {
            let players = world.players.load();
            for player in players.iter() {
                let player_clone = player.clone();
                let server_clone = self.clone();
                set.spawn(async move {
                    player_clone.tick(&server_clone).await;
                });
            }
        }
        set.join_all().await;
    }
    /// Ticks the game logic for all worlds. This is the part that is affected by `/tick freeze`.
    pub async fn tick_worlds(self: &Arc<Self>) {
        self.task_scheduler.tick(self).await;

        let mut set = JoinSet::new();

        for world in self.worlds.load().iter() {
            let world = world.clone();
            let server = self.clone();

            set.spawn(async move {
                world.tick(server).await;
            });
        }

        set.join_all().await;

        // Global tasks
        if let Err(e) = self.player_data_storage.tick(self).await {
            error!("Error ticking player data: {e}");
        }
    }

    /// Updates the tick time statistics with the duration of the last tick.
    pub async fn update_tick_times(&self, tick_duration_nanos: i64) {
        let tick_count = self.tick_count.fetch_add(1, Ordering::Relaxed);
        let index = (tick_count % 100) as usize;

        let mut tick_times = self.tick_times_nanos.lock().await;
        let old_time = tick_times[index];
        tick_times[index] = tick_duration_nanos;
        drop(tick_times);

        self.aggregated_tick_times_nanos
            .fetch_add(tick_duration_nanos - old_time, Ordering::Relaxed);
    }

    /// Gets the rolling average tick time over the last 100 ticks, in nanoseconds.
    pub fn get_average_tick_time_nanos(&self) -> i64 {
        let tick_count = self.tick_count.load(Ordering::Relaxed);
        let sample_size = (tick_count as usize).min(100);
        if sample_size == 0 {
            return 0;
        }
        self.aggregated_tick_times_nanos.load(Ordering::Relaxed) / sample_size as i64
    }

    /// Returns the average Milliseconds Per Tick (MSPT).
    pub fn get_mspt(&self) -> f64 {
        let avg_nanos = self.get_average_tick_time_nanos();
        // Convert nanoseconds to decimal milliseconds
        avg_nanos as f64 / 1_000_000.0
    }

    /// Returns the Ticks Per Second (TPS).
    pub fn get_tps(&self) -> f64 {
        let mspt = self.get_mspt();
        if mspt <= 0.0 {
            return 0.0;
        }
        1000.0 / mspt
    }

    /// Returns a copy of the last 100 tick times.
    pub async fn get_tick_times_nanos_copy(&self) -> [i64; 100] {
        *self.tick_times_nanos.lock().await
    }
}
