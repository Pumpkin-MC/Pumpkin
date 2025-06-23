use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering};
use std::time::Instant;

use crossbeam::atomic::AtomicCell;
use pumpkin_config::BASIC_CONFIG;
use pumpkin_protocol::client::play::{CSystemChatMessage, CTickingState, CTickingStep};
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::entity::player::Player;
use crate::server::Server;
const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000;

#[derive(Debug)]
pub struct ServerTickRateManager {
    tickrate: AtomicCell<f32>,
    nanoseconds_per_tick: AtomicI64,
    frozen_ticks_to_run: AtomicI32,
    run_game_elements: AtomicBool,
    is_frozen: AtomicBool,

    // Sprinting state
    remaining_sprint_ticks: AtomicI64,
    sprint_tick_start_time: AtomicCell<Instant>,
    sprint_time_spend: AtomicI64,
    scheduled_current_sprint_ticks: AtomicI64,
    previous_is_frozen: AtomicBool,
}

impl Default for ServerTickRateManager {
    fn default() -> Self {
        let tickrate = BASIC_CONFIG.tps;
        Self {
            tickrate: AtomicCell::new(tickrate),
            nanoseconds_per_tick: AtomicI64::new(NANOSECONDS_PER_SECOND / tickrate as i64),
            frozen_ticks_to_run: AtomicI32::new(0),
            run_game_elements: AtomicBool::new(true),
            is_frozen: AtomicBool::new(false),
            remaining_sprint_ticks: AtomicI64::new(0),
            sprint_tick_start_time: AtomicCell::new(Instant::now()),
            sprint_time_spend: AtomicI64::new(0),
            scheduled_current_sprint_ticks: AtomicI64::new(0),
            previous_is_frozen: AtomicBool::new(false),
        }
    }
}

impl ServerTickRateManager {
    pub fn tick(&self) {
        let frozen_ticks = self.frozen_ticks_to_run.load(Ordering::Relaxed);
        let run_game = !self.is_frozen.load(Ordering::Relaxed) || frozen_ticks > 0;
        self.run_game_elements.store(run_game, Ordering::Relaxed);

        if frozen_ticks > 0 {
            self.frozen_ticks_to_run.fetch_sub(1, Ordering::Relaxed);
        }
    }

    // Getters
    pub fn tickrate(&self) -> f32 {
        self.tickrate.load()
    }

    pub fn nanoseconds_per_tick(&self) -> i64 {
        self.nanoseconds_per_tick.load(Ordering::Relaxed)
    }

    pub fn is_frozen(&self) -> bool {
        self.is_frozen.load(Ordering::Relaxed)
    }

    pub fn runs_normally(&self) -> bool {
        self.run_game_elements.load(Ordering::Relaxed)
    }

    pub fn is_sprinting(&self) -> bool {
        self.remaining_sprint_ticks.load(Ordering::Relaxed) > 0
    }

    pub fn is_stepping_forward(&self) -> bool {
        self.frozen_ticks_to_run.load(Ordering::Relaxed) > 0
    }

    pub async fn set_tick_rate(&self, server: &Server, rate: f32) {
        self.tickrate.store(rate.max(1.0));
        self.nanoseconds_per_tick.store(
            (NANOSECONDS_PER_SECOND as f64 / f64::from(self.tickrate.load())) as i64,
            Ordering::Relaxed,
        );
        // server.on_tick_rate_changed(); // Might need this hook if autosave interval depends on it
        self.update_state_to_clients(server).await;
    }

    pub async fn set_frozen(&self, server: &Server, frozen: bool) {
        self.is_frozen.store(frozen, Ordering::Relaxed);
        self.update_state_to_clients(server).await;
    }

    pub async fn step_game_if_paused(&self, server: &Server, ticks: i32) -> bool {
        if !self.is_frozen() {
            return false;
        }
        self.frozen_ticks_to_run.store(ticks, Ordering::Relaxed);
        self.update_step_ticks(server).await;
        true
    }

    pub async fn stop_stepping(&self, server: &Server) -> bool {
        if self.is_stepping_forward() {
            self.frozen_ticks_to_run.store(0, Ordering::Relaxed);
            self.update_step_ticks(server).await;
            true
        } else {
            false
        }
    }

    pub async fn request_game_to_sprint(&self, server: &Server, ticks: i64) -> bool {
        let was_sprinting = self.is_sprinting();
        self.sprint_time_spend.store(0, Ordering::Relaxed);
        self.scheduled_current_sprint_ticks
            .store(ticks, Ordering::Relaxed);
        self.remaining_sprint_ticks.store(ticks, Ordering::Relaxed);
        self.previous_is_frozen
            .store(self.is_frozen(), Ordering::Relaxed);
        self.set_frozen(server, false).await;
        was_sprinting
    }

    pub async fn stop_sprinting(&self, server: &Server) -> bool {
        if self.is_sprinting() {
            self.finish_tick_sprint(server).await;
            true
        } else {
            false
        }
    }

    /// Records the start time of a sprint tick's workload.
    pub fn start_sprint_tick_work(&self) {
        self.sprint_tick_start_time.store(Instant::now());
    }

    /// Records the time spent on the sprint tick's work and decrements the remaining sprint ticks.
    /// Returns `true` if the sprint has just finished.
    pub fn end_sprint_tick_work(&self) -> bool {
        let spent = Instant::now().duration_since(self.sprint_tick_start_time.load());
        self.sprint_time_spend
            .fetch_add(spent.as_nanos() as i64, Ordering::Relaxed);

        // fetch_sub returns the *previous* value. If it was 1, it is now 0, and the sprint is over.
        self.remaining_sprint_ticks.fetch_sub(1, Ordering::Relaxed) == 1
    }

    pub async fn finish_tick_sprint(&self, server: &Server) {
        let total_sprinted_ticks = self.scheduled_current_sprint_ticks.load(Ordering::Relaxed)
            - self.remaining_sprint_ticks.load(Ordering::Relaxed);
        let time_spent_nanos = self.sprint_time_spend.load(Ordering::Relaxed);

        let inner_message = if total_sprinted_ticks > 0 && time_spent_nanos > 0 {
            let time_spent_ms = time_spent_nanos as f64 / 1_000_000.0;
            let tps = (total_sprinted_ticks as f64 * 1000.0) / time_spent_ms;
            let mspt = time_spent_ms / total_sprinted_ticks as f64;

            TextComponent::translate(
                "commands.tick.sprint.report",
                [
                    TextComponent::text(format!("{tps:.2}")),
                    TextComponent::text(format!("{mspt:.2}")),
                ],
            )
        } else {
            // This is the message for `/tick sprint stop` or a zero-tick sprint.
            TextComponent::translate("commands.tick.sprint.stop.success", [])
        };

        // Construct the final component with the [Server: ...] wrapper
        let final_report = TextComponent::text("[Server: ")
            .add_child(inner_message)
            .add_child(TextComponent::text("]"))
            .italic()
            .color_named(NamedColor::Gray);

        // Send as a system chat message, which does not add a sender prefix.
        server
            .broadcast_packet_all(&CSystemChatMessage::new(&final_report, false))
            .await;

        // Reset state after sending the report
        self.scheduled_current_sprint_ticks
            .store(0, Ordering::Relaxed);
        self.sprint_time_spend.store(0, Ordering::Relaxed);
        self.remaining_sprint_ticks.store(0, Ordering::Relaxed);
        self.set_frozen(server, self.previous_is_frozen.load(Ordering::Relaxed))
            .await;
        // server.on_tick_rate_changed();
    }

    async fn update_state_to_clients(&self, server: &Server) {
        server
            .broadcast_packet_all(&CTickingState::new(
                self.tickrate.load(),
                self.is_frozen.load(Ordering::Relaxed),
            ))
            .await;
    }

    async fn update_step_ticks(&self, server: &Server) {
        server
            .broadcast_packet_all(&CTickingStep::new(
                self.frozen_ticks_to_run.load(Ordering::Relaxed).into(),
            ))
            .await;
    }
    pub async fn update_joining_player(&self, player: &Player) {
        player
            .client
            .send_packet_now(&CTickingState::new(self.tickrate(), self.is_frozen()))
            .await;
        player
            .client
            .send_packet_now(&CTickingStep::new(
                self.frozen_ticks_to_run.load(Ordering::Relaxed).into(),
            ))
            .await;
    }
}
