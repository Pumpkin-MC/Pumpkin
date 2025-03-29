use pumpkin_protocol::codec::identifier::Identifier;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'a: 'de"))]
pub struct Painting<'a> {
    #[serde(borrow)]
    asset_id: Identifier<'a>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    //  title: Option<TextComponent<'static>>,
    //  #[serde(skip_serializing_if = "Option::is_none")]
    //  author: Option<TextComponent<'static>>,
    height: i32,
    width: i32,
}
