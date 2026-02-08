/*
TODO
1. add proto chunk dirty flag
2. better priority
5. add lifetime to loading ticket
6. solve entity not unload problem
*/

pub type HashMapType<K, V> = rustc_hash::FxHashMap<K, V>;
pub type HashSetType<K> = rustc_hash::FxHashSet<K>;
pub type ChunkPos = pumpkin_util::math::vector2::Vector2<i32>;
pub type ChunkLevel = HashMapType<ChunkPos, i8>;
pub type IOLock = std::sync::Arc<(
    std::sync::Mutex<HashMapType<ChunkPos, u8>>,
    std::sync::Condvar,
)>;

pub mod channel;
pub mod chunk_holder;
pub mod chunk_listener;
pub mod chunk_loading;
pub mod chunk_state;
pub mod dag;
pub mod generation_cache;
pub mod generation;
pub mod schedule;
pub mod worker_logic;

pub use channel::LevelChannel;
pub use chunk_holder::ChunkHolder;
pub use chunk_listener::ChunkListener;
pub use chunk_loading::ChunkLoading;
pub use chunk_state::{Chunk, StagedChunkEnum};
pub use dag::DAG;
pub use generation_cache::Cache;
pub use generation::generate_single_chunk;
pub use schedule::GenerationSchedule;
