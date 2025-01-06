use serde::{Deserialize, Serialize};

// Why needed this?
// if we implement a more cool chunk optimizations we can define it here
// but now its only for rle vec

#[derive(Deserialize, Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct ChunkOptimizationConfig {
    pub rle_compression: Option<RleCompression>,
}


#[derive(Deserialize, Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct RleCompression {}
