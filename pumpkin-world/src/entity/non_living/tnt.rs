use serde::{Deserialize, Serialize};

use crate::entity::NonPlayerCommon;

#[derive(Serialize, Deserialize)]
pub struct Tnt {
    #[serde(flatten)]
    pub common: NonPlayerCommon,
    #[serde(rename = "fuse")]
    pub fuse_ticks: i16,
    pub explosion_power: f32,
    // TODO: block state
}
