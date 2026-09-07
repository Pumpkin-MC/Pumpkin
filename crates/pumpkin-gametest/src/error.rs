use pumpkin_util::math::position::BlockPos;
use thiserror::Error;

pub type GameTestResult<T> = Result<T, GameTestError>;

#[derive(Debug, Error)]
pub enum GameTestError {
    #[error("assertion failed at tick {tick}: {message}")]
    Assertion {
        tick: u32,
        position: Option<BlockPos>,
        message: String,
    },

    #[error("test exceeded its maximum of {max_ticks} ticks")]
    Timeout { max_ticks: u32 },

    #[error(
        "test exhausted {attempts} attempts with {successes} successes; {required_successes} successes required: {last_error}"
    )]
    ExhaustedAttempts {
        attempts: u32,
        successes: u32,
        required_successes: u32,
        last_error: String,
    },

    #[error("{0}")]
    InvalidStructure(String),

    #[error("{0}")]
    World(String),
}
