use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolfVariant {
    assets: WolfAssetInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolfAssetInfo {
    wild: String,
    tame: String,
    angry: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolfSoundVariant {
    hurt_sound: String,
    pant_sound: String,
    whine_sound: String,
    ambient_sound: String,
    death_sound: String,
    growl_sound: String,
}
