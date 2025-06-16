use pumpkin_protocol::codec::resource_location::ResourceLocation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannerPattern {
    asset_id: ResourceLocation,
    translation_key: String,
}
