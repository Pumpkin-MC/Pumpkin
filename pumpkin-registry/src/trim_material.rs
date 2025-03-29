use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimMaterial<'a> {
    asset_name: Cow<'a, str>,
    //  description: TextComponent<'static>,
}
