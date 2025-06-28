use pumpkin_util::resource_location::ResourceLocation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimPattern {
    asset_id: ResourceLocation,
    //  description: TextComponent<'static>,
    decal: bool,
}
