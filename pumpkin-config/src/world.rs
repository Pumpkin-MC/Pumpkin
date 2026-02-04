use serde::{Deserialize, Serialize};

use crate::{chunk::ChunkConfig, lighting::LightingEngineConfig};

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct LevelConfig {
    pub chunk: ChunkConfig,
    #[serde(default)]
    pub lighting: LightingEngineConfig,
    // TODO: More options
}
