use serde::{Deserialize, Serialize};

use crate::{
    entity::{NonPlayerCommon, nbt_item_stack},
    item::ItemStack,
};

#[derive(Serialize, Deserialize)]
pub struct Item {
    #[serde(flatten)]
    pub common: NonPlayerCommon,
    #[serde(rename = "Age")]
    pub age_ticks: i16,
    #[serde(rename = "PickupDelay")]
    pub pickup_delay_ticks: i16,
    #[serde(rename = "Health")]
    pub health: i16,
    #[serde(rename = "Item", with = "nbt_item_stack")]
    pub stack: ItemStack,
    // TODO: Owners
}
