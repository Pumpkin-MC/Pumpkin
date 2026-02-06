use serde::{Deserialize, Serialize};

use crate::chunk::ChunkConfig;

/// Configuration for world and level-specific settings.
///
/// Currently, it includes chunk-related options; more settings may be added later.
#[derive(Deserialize, Serialize, Default, Clone)]
pub struct LevelConfig {
    /// Configuration for chunk behaviour and management.
    pub chunk: ChunkConfig,
    // TODO: More options
}
