use pumpkin_codecs::{DataResult, Decode, DynamicOps, Encode, json_ops::JsonOps};
use pumpkin_util::{identifier::Identifier, text::TextComponent};
use serde_json::{Map, Value};

fn encode_json<O: DynamicOps>(
    value: Value,
    ops: &'static O,
    prefix: O::Value,
) -> DataResult<O::Value> {
    if prefix != ops.empty() {
        return DataResult::new_error("environment attribute value only supports an empty prefix");
    }
    DataResult::new_success(JsonOps.convert_to(ops, value))
}

fn decode_json<O: DynamicOps, T>(
    input: O::Value,
    ops: &'static O,
    decode: impl FnOnce(Value) -> Result<T, String>,
) -> DataResult<(T, O::Value)> {
    match decode(ops.convert_to(&JsonOps, input)) {
        Ok(value) => DataResult::new_success((value, ops.empty())),
        Err(error) => DataResult::new_error(error),
    }
}

fn parse_identifier(value: &Value) -> Result<Identifier, String> {
    let value = value
        .as_str()
        .ok_or_else(|| "expected a resource location string".to_string())?;
    Identifier::parse(value).map_err(|error| error.to_string())
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleOptions {
    pub particle_type: Identifier,
    /// Particle-type-specific fields. The schema depends on `particle_type`.
    pub options: Map<String, Value>,
}

impl ParticleOptions {
    #[must_use]
    pub fn simple(particle_type: Identifier) -> Self {
        Self {
            particle_type,
            options: Map::new(),
        }
    }

    fn from_json(value: Value) -> Result<Self, String> {
        let Value::Object(mut object) = value else {
            return Err("particle options must be an object".to_string());
        };
        let particle_type = object
            .remove("type")
            .ok_or_else(|| "particle options are missing `type`".to_string())?;
        Ok(Self {
            particle_type: parse_identifier(&particle_type)?,
            options: object,
        })
    }

    fn to_json(&self) -> Value {
        let mut object = self.options.clone();
        object.insert(
            "type".to_string(),
            Value::String(self.particle_type.to_string()),
        );
        Value::Object(object)
    }
}

impl Encode for ParticleOptions {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        encode_json(self.to_json(), ops, prefix)
    }
}

impl Decode for ParticleOptions {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        decode_json(input, ops, Self::from_json)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AmbientParticle {
    pub particle: ParticleOptions,
    pub probability: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AmbientParticles(pub Vec<AmbientParticle>);

impl Encode for AmbientParticles {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        encode_json(
            Value::Array(
                self.0
                    .iter()
                    .map(|entry| {
                        serde_json::json!({
                            "particle": entry.particle.to_json(),
                            "probability": entry.probability,
                        })
                    })
                    .collect(),
            ),
            ops,
            prefix,
        )
    }
}

impl Decode for AmbientParticles {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        decode_json(input, ops, |value| {
            let Value::Array(values) = value else {
                return Err("ambient_particles must be a list".to_string());
            };
            let mut particles = Vec::with_capacity(values.len());
            for value in values {
                let Value::Object(mut object) = value else {
                    return Err("ambient particle must be an object".to_string());
                };
                let particle = ParticleOptions::from_json(
                    object
                        .remove("particle")
                        .ok_or_else(|| "ambient particle is missing `particle`".to_string())?,
                )?;
                let probability = object
                    .remove("probability")
                    .and_then(|value| value.as_f64())
                    .ok_or_else(|| {
                        "ambient particle is missing numeric `probability`".to_string()
                    })? as f32;
                if !(0.0..=1.0).contains(&probability) {
                    return Err("ambient particle probability must be between 0 and 1".to_string());
                }
                particles.push(AmbientParticle {
                    particle,
                    probability,
                });
            }
            Ok(Self(particles))
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundEvent {
    pub id: Identifier,
    pub fixed_range: Option<f32>,
}

impl SoundEvent {
    #[must_use]
    pub fn new(id: Identifier) -> Self {
        Self {
            id,
            fixed_range: None,
        }
    }

    fn from_json(value: Value) -> Result<Self, String> {
        match value {
            Value::String(id) => Ok(Self::new(
                Identifier::parse(&id).map_err(|error| error.to_string())?,
            )),
            Value::Object(mut object) => {
                let id = object
                    .remove("sound_id")
                    .ok_or_else(|| "sound event is missing `sound_id`".to_string())?;
                let fixed_range = object
                    .remove("range")
                    .map(|value| {
                        value
                            .as_f64()
                            .map(|value| value as f32)
                            .ok_or_else(|| "sound event `range` must be numeric".to_string())
                    })
                    .transpose()?;
                Ok(Self {
                    id: parse_identifier(&id)?,
                    fixed_range,
                })
            }
            _ => Err("sound event must be a resource location or object".to_string()),
        }
    }

    fn to_json(&self) -> Value {
        self.fixed_range.map_or_else(
            || Value::String(self.id.to_string()),
            |range| serde_json::json!({"sound_id": self.id.to_string(), "range": range}),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MusicSound {
    pub sound: SoundEvent,
    pub min_delay: i32,
    pub max_delay: i32,
    pub replace_current_music: bool,
}

impl MusicSound {
    fn from_json(value: Value) -> Result<Self, String> {
        let Value::Object(mut object) = value else {
            return Err("music sound must be an object".to_string());
        };
        let sound = SoundEvent::from_json(
            object
                .remove("sound")
                .ok_or_else(|| "music sound is missing `sound`".to_string())?,
        )?;
        let min_delay = object
            .remove("min_delay")
            .and_then(|value| value.as_i64())
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| "music sound is missing integer `min_delay`".to_string())?;
        let max_delay = object
            .remove("max_delay")
            .and_then(|value| value.as_i64())
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| "music sound is missing integer `max_delay`".to_string())?;
        if min_delay > max_delay {
            return Err("music sound `min_delay` cannot exceed `max_delay`".to_string());
        }
        let replace_current_music = object
            .remove("replace_current_music")
            .map(|value| {
                value
                    .as_bool()
                    .ok_or_else(|| "`replace_current_music` must be boolean".to_string())
            })
            .transpose()?
            .unwrap_or(false);
        Ok(Self {
            sound,
            min_delay,
            max_delay,
            replace_current_music,
        })
    }

    fn to_json(&self) -> Value {
        let mut object = Map::new();
        object.insert("sound".to_string(), self.sound.to_json());
        object.insert("min_delay".to_string(), Value::from(self.min_delay));
        object.insert("max_delay".to_string(), Value::from(self.max_delay));
        if self.replace_current_music {
            object.insert("replace_current_music".to_string(), Value::Bool(true));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BackgroundMusic {
    pub default: Option<MusicSound>,
    pub creative: Option<MusicSound>,
    pub underwater: Option<MusicSound>,
}

impl Encode for BackgroundMusic {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        let mut object = Map::new();
        if let Some(value) = &self.default {
            object.insert("default".to_string(), value.to_json());
        }
        if let Some(value) = &self.creative {
            object.insert("creative".to_string(), value.to_json());
        }
        if let Some(value) = &self.underwater {
            object.insert("underwater".to_string(), value.to_json());
        }
        encode_json(Value::Object(object), ops, prefix)
    }
}

impl Decode for BackgroundMusic {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        decode_json(input, ops, |value| {
            let Value::Object(mut object) = value else {
                return Err("background_music must be an object".to_string());
            };
            let parse = |value: Option<Value>| value.map(MusicSound::from_json).transpose();
            Ok(Self {
                default: parse(object.remove("default"))?,
                creative: parse(object.remove("creative"))?,
                underwater: parse(object.remove("underwater"))?,
            })
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BedCondition {
    Always,
    WhenDark,
    Never,
}

impl BedCondition {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::WhenDark => "when_dark",
            Self::Never => "never",
        }
    }

    fn from_json(value: Value) -> Result<Self, String> {
        match value.as_str() {
            Some("always") => Ok(Self::Always),
            Some("when_dark") => Ok(Self::WhenDark),
            Some("never") => Ok(Self::Never),
            _ => Err("bed condition must be `always`, `when_dark`, or `never`".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BedRule {
    pub can_sleep: BedCondition,
    pub can_set_spawn: BedCondition,
    pub explodes: bool,
    pub error_message: Option<TextComponent>,
}

impl BedRule {
    #[must_use]
    pub fn overworld() -> Self {
        Self {
            can_sleep: BedCondition::WhenDark,
            can_set_spawn: BedCondition::Always,
            explodes: false,
            error_message: Some(pumpkin_macros::translate_cross!(
                "block.minecraft.bed.no_sleep",
                "tile.bed.noSleep"
            )),
        }
    }
}

impl Encode for BedRule {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        let mut object = Map::new();
        object.insert(
            "can_sleep".to_string(),
            Value::String(self.can_sleep.as_str().to_string()),
        );
        object.insert(
            "can_set_spawn".to_string(),
            Value::String(self.can_set_spawn.as_str().to_string()),
        );
        if self.explodes {
            object.insert("explodes".to_string(), Value::Bool(true));
        }
        if let Some(message) = &self.error_message {
            match serde_json::to_value(message) {
                Ok(value) => {
                    object.insert("error_message".to_string(), value);
                }
                Err(error) => return DataResult::new_error(error.to_string()),
            }
        }
        encode_json(Value::Object(object), ops, prefix)
    }
}

impl Decode for BedRule {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        decode_json(input, ops, |value| {
            let Value::Object(mut object) = value else {
                return Err("bed_rule must be an object".to_string());
            };
            let can_sleep = BedCondition::from_json(
                object
                    .remove("can_sleep")
                    .ok_or_else(|| "bed_rule is missing `can_sleep`".to_string())?,
            )?;
            let can_set_spawn = BedCondition::from_json(
                object
                    .remove("can_set_spawn")
                    .ok_or_else(|| "bed_rule is missing `can_set_spawn`".to_string())?,
            )?;
            let explodes = object
                .remove("explodes")
                .map(|value| {
                    value
                        .as_bool()
                        .ok_or_else(|| "`explodes` must be boolean".to_string())
                })
                .transpose()?
                .unwrap_or(false);
            let error_message = object
                .remove("error_message")
                .map(|value| serde_json::from_value(value).map_err(|error| error.to_string()))
                .transpose()?;
            Ok(Self {
                can_sleep,
                can_set_spawn,
                explodes,
                error_message,
            })
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AmbientMoodSettings {
    pub sound: SoundEvent,
    pub tick_delay: i32,
    pub block_search_extent: i32,
    pub offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AmbientAdditionsSettings {
    pub sound: SoundEvent,
    pub tick_chance: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AmbientSounds {
    pub loop_sound: Option<SoundEvent>,
    pub mood: Option<AmbientMoodSettings>,
    pub additions: Vec<AmbientAdditionsSettings>,
}

impl Encode for AmbientSounds {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        let mut object = Map::new();
        if let Some(sound) = &self.loop_sound {
            object.insert("loop".to_string(), sound.to_json());
        }
        if let Some(mood) = &self.mood {
            object.insert(
                "mood".to_string(),
                serde_json::json!({
                    "sound": mood.sound.to_json(), "tick_delay": mood.tick_delay,
                    "block_search_extent": mood.block_search_extent, "offset": mood.offset,
                }),
            );
        }
        if !self.additions.is_empty() {
            object.insert("additions".to_string(), Value::Array(self.additions.iter().map(|addition| serde_json::json!({
                "sound": addition.sound.to_json(), "tick_chance": addition.tick_chance,
            })).collect()));
        }
        encode_json(Value::Object(object), ops, prefix)
    }
}

impl Decode for AmbientSounds {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        decode_json(input, ops, |value| {
            let Value::Object(mut object) = value else {
                return Err("ambient_sounds must be an object".to_string());
            };
            let loop_sound = object
                .remove("loop")
                .map(SoundEvent::from_json)
                .transpose()?;
            let mood = object
                .remove("mood")
                .map(|value| {
                    let Value::Object(mut mood) = value else {
                        return Err("ambient mood must be an object".to_string());
                    };
                    Ok(AmbientMoodSettings {
                        sound: SoundEvent::from_json(
                            mood.remove("sound")
                                .ok_or_else(|| "ambient mood is missing `sound`".to_string())?,
                        )?,
                        tick_delay: mood
                            .remove("tick_delay")
                            .and_then(|value| value.as_i64())
                            .and_then(|value| i32::try_from(value).ok())
                            .ok_or_else(|| {
                                "ambient mood is missing integer `tick_delay`".to_string()
                            })?,
                        block_search_extent: mood
                            .remove("block_search_extent")
                            .and_then(|value| value.as_i64())
                            .and_then(|value| i32::try_from(value).ok())
                            .ok_or_else(|| {
                                "ambient mood is missing integer `block_search_extent`".to_string()
                            })?,
                        offset: mood
                            .remove("offset")
                            .and_then(|value| value.as_f64())
                            .ok_or_else(|| "ambient mood is missing numeric `offset`".to_string())?
                            as f32,
                    })
                })
                .transpose()?;
            let additions = match object.remove("additions") {
                None => Vec::new(),
                Some(Value::Array(values)) => values
                    .into_iter()
                    .map(|value| {
                        let Value::Object(mut addition) = value else {
                            return Err("ambient addition must be an object".to_string());
                        };
                        let tick_chance = addition
                            .remove("tick_chance")
                            .and_then(|value| value.as_f64())
                            .ok_or_else(|| {
                                "ambient addition is missing numeric `tick_chance`".to_string()
                            })? as f32;
                        if !(0.0..=1.0).contains(&tick_chance) {
                            return Err("ambient addition `tick_chance` must be between 0 and 1"
                                .to_string());
                        }
                        Ok(AmbientAdditionsSettings {
                            sound: SoundEvent::from_json(addition.remove("sound").ok_or_else(
                                || "ambient addition is missing `sound`".to_string(),
                            )?)?,
                            tick_chance,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
                Some(_) => return Err("ambient_sounds `additions` must be a list".to_string()),
            };
            Ok(Self {
                loop_sound,
                mood,
                additions,
            })
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriState {
    True,
    False,
    Default,
}

impl Encode for TriState {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        match self {
            Self::True => true.encode(ops, prefix),
            Self::False => false.encode(ops, prefix),
            Self::Default => "default".to_string().encode(ops, prefix),
        }
    }
}

impl Decode for TriState {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        if let Some(value) = ops.get_bool(&input).into_result() {
            return DataResult::new_success((
                if value { Self::True } else { Self::False },
                ops.empty(),
            ));
        }

        String::parse(input, ops).flat_map(|value| match value.as_str() {
            "default" => DataResult::new_success((Self::Default, ops.empty())),
            _ => DataResult::new_error("tri_state must be true, false, or \"default\""),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonPhase {
    FullMoon,
    WaningGibbous,
    ThirdQuarter,
    WaningCrescent,
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
}

impl MoonPhase {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FullMoon => "full_moon",
            Self::WaningGibbous => "waning_gibbous",
            Self::ThirdQuarter => "third_quarter",
            Self::WaningCrescent => "waning_crescent",
            Self::NewMoon => "new_moon",
            Self::WaxingCrescent => "waxing_crescent",
            Self::FirstQuarter => "first_quarter",
            Self::WaxingGibbous => "waxing_gibbous",
        }
    }
}

impl Encode for MoonPhase {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.as_str().to_string().encode(ops, prefix)
    }
}

impl Decode for MoonPhase {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        String::parse(input, ops).flat_map(|value| {
            let phase = match value.as_str() {
                "full_moon" => Self::FullMoon,
                "waning_gibbous" => Self::WaningGibbous,
                "third_quarter" => Self::ThirdQuarter,
                "waning_crescent" => Self::WaningCrescent,
                "new_moon" => Self::NewMoon,
                "waxing_crescent" => Self::WaxingCrescent,
                "first_quarter" => Self::FirstQuarter,
                "waxing_gibbous" => Self::WaxingGibbous,
                _ => return DataResult::new_error(format!("Unknown moon phase: {value}")),
            };
            DataResult::new_success((phase, ops.empty()))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity(pub Identifier);

impl Encode for Activity {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.0.to_string().encode(ops, prefix)
    }
}

impl Decode for Activity {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        String::parse(input, ops).flat_map(|value| {
            Identifier::parse(&value).map_or_else(
                |error| DataResult::new_error(error.to_string()),
                |identifier| DataResult::new_success((Self(identifier), ops.empty())),
            )
        })
    }
}
