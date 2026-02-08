use super::chunk_state::{Chunk, StagedChunkEnum};
use super::dag::{EdgeKey, NodeKey};
use slotmap::Key;

pub struct ChunkHolder {
    pub target_stage: StagedChunkEnum,
    pub current_stage: StagedChunkEnum,
    pub chunk: Option<Chunk>,
    pub occupied: NodeKey,
    pub occupied_by: EdgeKey,
    pub public: bool,
    pub tasks: [NodeKey; 10],
}

impl Default for ChunkHolder {
    fn default() -> Self {
        Self {
            target_stage: StagedChunkEnum::None,
            current_stage: StagedChunkEnum::None,
            chunk: None,
            occupied: NodeKey::null(),
            occupied_by: EdgeKey::null(),
            public: false,
            tasks: [NodeKey::null(); 10],
        }
    }
}
