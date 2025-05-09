use serde::{Deserialize, Serialize};

use crate::entity::NonPlayerCommon;

#[derive(Serialize, Deserialize)]
pub struct Projectile {
    #[serde(flatten)]
    pub common: NonPlayerCommon,
}
