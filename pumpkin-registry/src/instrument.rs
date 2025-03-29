use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument<'a> {
    sound_event: Cow<'a, str>,
    use_duration: f32,
    range: f32,
    //  description: TextComponent<'static>,
}
