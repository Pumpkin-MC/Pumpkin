use std::{collections::HashMap, sync::LazyLock};

use pumpkin_util::read_data_from_file;
use serde::Deserialize;

use crate::generation::structure::placement::StructurePlacement;

pub mod placement;
pub mod structures;

#[derive(Deserialize)]
pub struct StructureSet {
    pub placement: StructurePlacement,
}

pub static STRUCTURE_SETS: LazyLock<HashMap<String, StructureSet>> =
    LazyLock::new(|| read_data_from_file!("../../../assets/structure_set.json"));
