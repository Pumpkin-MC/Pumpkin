use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimMaterial {
    asset_name: String,
    //  description: TextComponent<'static>,
}
