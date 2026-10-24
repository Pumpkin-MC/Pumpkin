use crate::error::GameTestError;

#[derive(Debug)]
pub enum GameTestState {
    Queued,
    SettingUp { elapsed_ticks: u32 },
    Running { elapsed_ticks: u32 },
    Passed { tick: u32 },
    Failed { tick: u32, error: GameTestError },
}

impl GameTestState {
    #[must_use]
    pub const fn is_finished(&self) -> bool {
        matches!(self, Self::Passed { .. } | Self::Failed { .. })
    }
}
