use serde::Deserialize;

pub use pumpkin_data::chunk_gen_settings::{GenerationSettings, GenerationShapeConfig};

// TODO: Re-enable this once generation settings are loaded from JSON again.
#[derive(Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GeneratorSetting {
    Overworld,
    LargeBiomes,
    Amplified,
    Nether,
    End,
    Caves,
    FloatingIslands,
}
