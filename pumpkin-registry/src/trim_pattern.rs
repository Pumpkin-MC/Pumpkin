use pumpkin_protocol::codec::resource_location::ResourceLocation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimPattern {
    asset_id: ResourceLocation,
    //  description: TextComponent<'static>,
    decal: bool,
}
