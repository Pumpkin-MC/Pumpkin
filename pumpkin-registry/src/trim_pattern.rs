use pumpkin_protocol::codec::identifier::Identifier;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimPattern {
    asset_id: Identifier,
    //  description: TextComponent<'static>,
    decal: bool,
}
