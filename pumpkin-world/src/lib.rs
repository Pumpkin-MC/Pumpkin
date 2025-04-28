use generation::settings::GenerationSettings;
use pumpkin_util::math::vector2::Vector2;

pub mod biome;
pub mod block;
pub mod chunk;
pub mod cylindrical_chunk_iterator;
pub mod data;
pub mod dimension;
pub mod entity;
mod generation;
pub mod item;
pub mod level;
mod lock;
pub mod world_info;

pub type BlockId = u16;
pub type BlockStateId = u16;

#[macro_export]
macro_rules! global_path {
    ($path:expr) => {{
        use std::path::Path;
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(file!())
            .parent()
            .unwrap()
            .join($path)
    }};
}

#[macro_export]
macro_rules! read_data_from_file {
    ($path:expr) => {{
        use std::fs;
        use $crate::global_path;
        serde_json::from_str(&fs::read_to_string(global_path!($path)).expect("no data file"))
            .expect("failed to decode data")
    }};
}

// TODO: is there a way to do in-file benches?
pub use generation::{
    GlobalRandomConfig, noise_router::proto_noise_router::ProtoNoiseRouters,
    proto_chunk::ProtoChunk, settings::GENERATION_SETTINGS, settings::GeneratorSetting,
};

pub fn bench_create_and_populate_noise(
    base_router: &ProtoNoiseRouters,
    random_config: &GlobalRandomConfig,
    settings: &GenerationSettings,
) {
    let mut chunk = ProtoChunk::new(Vector2::new(0, 0), base_router, random_config, settings);
    chunk.populate_noise();
}

pub fn bench_create_and_populate_biome(
    base_router: &ProtoNoiseRouters,
    random_config: &GlobalRandomConfig,
    settings: &GenerationSettings,
) {
    let mut chunk = ProtoChunk::new(Vector2::new(0, 0), base_router, random_config, settings);
    chunk.populate_biomes();
}

pub fn bench_create_and_populate_noise_with_surface(
    base_router: &ProtoNoiseRouters,
    random_config: &GlobalRandomConfig,
    settings: &GenerationSettings,
) {
    let mut chunk = ProtoChunk::new(Vector2::new(0, 0), base_router, random_config, settings);
    chunk.populate_biomes();
    chunk.populate_noise();
    chunk.build_surface();
}
