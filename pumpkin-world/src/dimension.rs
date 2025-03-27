#[cfg(not(target_family = "wasm"))]
use std::path::PathBuf;

use serde::Deserialize;

#[cfg(not(target_family = "wasm"))]
use crate::level::Level;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    Overworld,
    Nether,
    End,
}

impl Dimension {
    #[cfg(not(target_family = "wasm"))]
    pub fn into_level(&self, mut base_directory: PathBuf) -> Level {
        match self {
            Dimension::Overworld => {}
            Dimension::Nether => base_directory.push("DIM-1"),
            Dimension::End => base_directory.push("DIM1"),
        }
        Level::from_root_folder(base_directory)
    }
}
