use serde::{Deserialize, Serialize};

use crate::entity::MobCommon;

#[derive(Serialize, Deserialize)]
pub struct ZombieCommon {
    #[serde(flatten)]
    pub mob_common: MobCommon,
    #[serde(rename = "IsBaby", default)]
    pub is_baby: bool,
    // TODO: Others
}
