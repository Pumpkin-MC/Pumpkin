use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JukeboxSong<'a> {
    sound_event: Cow<'a, str>,
    description: Description<'a>,
    length_in_seconds: f32,
    comparator_output: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Description<'a> {
    translate: Cow<'a, str>,
}
