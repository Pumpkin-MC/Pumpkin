use super::chunk_state::{Chunk, StagedChunkEnum};
use super::dag::{EdgeKey, NodeKey};
use slotmap::Key;

pub struct ChunkHolder {
    pub target_stage: StagedChunkEnum,
    /// Minimum stage required by generation dependencies (e.g. `StructureReferences` neighbors).
    /// May exceed `target_stage` when this chunk is needed by a neighbor's generation task
    /// but isn't in the player's view radius. Tasks up to this stage are scheduled but the
    /// chunk is not made public and is unloaded once the dependency is satisfied.
    pub dependency_stage: StagedChunkEnum,
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
            dependency_stage: StagedChunkEnum::None,
            current_stage: StagedChunkEnum::None,
            chunk: None,
            occupied: NodeKey::null(),
            occupied_by: EdgeKey::null(),
            public: false,
            tasks: [NodeKey::null(); 10],
        }
    }
}

impl ChunkHolder {
    #[must_use]
    pub fn is_inactive(&self) -> bool {
        self.target_stage == StagedChunkEnum::None
            && self.dependency_stage == StagedChunkEnum::None
            && self.current_stage == StagedChunkEnum::None
            && self.chunk.is_none()
            && self.occupied.is_null()
            && !self.public
            && self.tasks.iter().all(|task| task.is_null())
    }
}
