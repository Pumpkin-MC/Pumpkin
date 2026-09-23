use std::{fmt, sync::Arc};

use tokio::sync::watch;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriverError {
    message: Arc<str>,
}

impl DriverError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: Arc::from(message.into()),
        }
    }
}

impl fmt::Display for DriverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for DriverError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DriverState {
    Starting,
    Accepting,
    Draining,
    Stopping,
    Stopped,
    Failed(Arc<DriverError>),
}

impl DriverState {
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped | Self::Failed(_))
    }
}

pub struct Lifecycle {
    state: watch::Sender<DriverState>,
}

impl Lifecycle {
    pub fn new() -> Arc<Self> {
        let (state, _receiver) = watch::channel(DriverState::Starting);
        Arc::new(Self { state })
    }

    pub fn transition(&self, state: DriverState) {
        self.state.send_replace(state);
    }

    pub fn state(&self) -> DriverState {
        self.state.borrow().clone()
    }

    pub fn subscribe(&self) -> DriverJoin {
        DriverJoin {
            state: self.state.subscribe(),
        }
    }
}

pub struct DriverJoin {
    state: watch::Receiver<DriverState>,
}

impl DriverJoin {
    pub async fn wait(&mut self) -> Result<(), Arc<DriverError>> {
        loop {
            let state = self.state.borrow().clone();
            match state {
                DriverState::Stopped => return Ok(()),
                DriverState::Failed(error) => return Err(error),
                _ => {}
            }

            if self.state.changed().await.is_err() {
                return Err(Arc::new(DriverError::new(
                    "Wasm plugin store lifecycle observer closed before a terminal state",
                )));
            }
        }
    }
}
