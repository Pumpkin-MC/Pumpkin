use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolfVariant<'a> {
    assets: WolfAssetInfo<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolfAssetInfo<'a> {
    wild: Cow<'a, str>,
    tame: Cow<'a, str>,
    angry: Cow<'a, str>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolfSoundVariant<'a> {
    hurt_sound: Cow<'a, str>,
    pant_sound: Cow<'a, str>,
    whine_sound: Cow<'a, str>,
    ambient_sound: Cow<'a, str>,
    death_sound: Cow<'a, str>,
    growl_sound: Cow<'a, str>,
}
