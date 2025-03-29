use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FrogVariant<'a> {
    asset_id: Cow<'a, str>,
}
