use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FrogVariant {
    asset_id: String,
}
