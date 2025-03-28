use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CowVariant<'a> {
    // Cow in cow variant, think about it...
    asset_id: Cow<'a, str>,
}
