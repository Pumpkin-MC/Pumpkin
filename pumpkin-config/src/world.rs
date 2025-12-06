use serde::{Deserialize, Serialize};

use crate::chunk::ChunkConfig;

#[derive(Deserialize, Serialize, Default)]
pub struct LevelConfig {
    pub chunk: ChunkConfig,
    // TODO: More options
}
