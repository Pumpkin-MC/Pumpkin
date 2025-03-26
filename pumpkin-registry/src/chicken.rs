use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChickenVariant {
    asset_id: String,
}
