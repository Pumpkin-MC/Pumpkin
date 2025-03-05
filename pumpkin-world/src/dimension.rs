use std::path::PathBuf;

use serde::Deserialize;

use crate::level::Level;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    OverWorld,
    Nether,
    End,
}

impl Dimension {
    pub fn into_level(&self, mut base_directory: PathBuf) -> Level {
        match self {
            Dimension::OverWorld => {}
            Dimension::Nether => base_directory.push("DIM-1"),
            Dimension::End => base_directory.push("DIM1"),
        }
        Level::from_root_folder(base_directory)
    }
}
