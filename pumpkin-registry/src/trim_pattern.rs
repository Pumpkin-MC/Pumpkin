use pumpkin_protocol::codec::identifier::Identifier;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'a: 'de"))]
pub struct TrimPattern<'a> {
    #[serde(borrow)]
    asset_id: Identifier<'a>,
    decal: bool,
}
