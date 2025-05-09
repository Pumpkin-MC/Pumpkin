use serde::{Deserialize, Serialize};

use crate::entity::NonPlayerCommon;

#[derive(Serialize, Deserialize)]
pub struct ExpOrb {
    #[serde(flatten)]
    pub common: NonPlayerCommon,
    #[serde(rename = "Age")]
    pub age_ticks: i16,
    #[serde(rename = "Count")]
    pub merge_count: i32,
    #[serde(rename = "Health")]
    pub health: i16,
    #[serde(rename = "Value")]
    pub value: i16,
}
