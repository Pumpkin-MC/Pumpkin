use serde::{Deserialize, Serialize};

use crate::chunk::ChunkConfig;

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct LevelConfig {
    pub chunk: ChunkConfig,
    // TODO: More options
}
