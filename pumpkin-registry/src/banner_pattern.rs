use std::borrow::Cow;

use pumpkin_protocol::codec::identifier::Identifier;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'a: 'de"))]
pub struct BannerPattern<'a> {
    #[serde(borrow)]
    asset_id: Identifier<'a>,
    translation_key: Cow<'a, str>,
}
