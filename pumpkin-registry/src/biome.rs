use std::borrow::Cow;

use pumpkin_protocol::codec::var_int::VarInt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Biome<'a> {
    has_precipitation: bool,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature_modifier: Option<Cow<'a, str>>,
    downfall: f32,
    effects: BiomeEffects<'a>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BiomeEffects<'a> {
    fog_color: i32,
    water_color: i32,
    water_fog_color: i32,
    sky_color: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    foliage_color: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    grass_color: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    grass_color_modifier: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    particle: Option<Particle<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ambient_sound: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mood_sound: Option<MoodSound<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    additions_sound: Option<AdditionsSound<'a>>,
    //   #[serde(skip_serializing_if = "Option::is_none")]
    //   music: Option<Vec<DataPool<Music>>>,
    music_volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Particle<'a> {
    options: ParticleOptions<'a>,
    probability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParticleOptions<'a> {
    r#type: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<VarInt>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MoodSound<'a> {
    sound: Cow<'a, str>,
    tick_delay: i32,
    block_search_extent: i32,
    offset: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdditionsSound<'a> {
    sound: Cow<'a, str>,
    tick_chance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Music<'a> {
    sound: Cow<'a, str>,
    min_delay: i32,
    max_delay: i32,
    replace_current_music: bool,
}
