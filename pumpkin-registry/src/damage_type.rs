use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageType<'a> {
    exhaustion: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    death_message_type: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effects: Option<Cow<'a, str>>,
    message_id: Cow<'a, str>,
    scaling: Cow<'a, str>,
}
