use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CatVariant {
    asset_id: String,
}
