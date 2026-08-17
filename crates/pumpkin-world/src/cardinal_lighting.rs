use pumpkin_codecs::{DataResult, Decode, DynamicOps, Encode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardinalLighting {
    pub down: f32,
    pub up: f32,
    pub north: f32,
    pub south: f32,
    pub west: f32,
    pub east: f32,
}

impl CardinalLighting {
    #[must_use]
    pub const fn new(down: f32, up: f32, north: f32, south: f32, west: f32, east: f32) -> Self {
        Self {
            down,
            up,
            north,
            south,
            west,
            east,
        }
    }
}

pub const DEFAULT: CardinalLighting =
    CardinalLighting::new(0.5f32, 1.0f32, 0.8f32, 0.8f32, 0.6f32, 0.6f32);

pub const NETHER: CardinalLighting =
    CardinalLighting::new(0.9f32, 0.9f32, 0.8f32, 0.8f32, 0.6f32, 0.6f32);

impl Encode for CardinalLighting {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        let name = if *self == DEFAULT {
            "default"
        } else if *self == NETHER {
            "nether"
        } else {
            return DataResult::new_error("cardinal lighting has no serialized vanilla profile");
        };
        name.to_string().encode(ops, prefix)
    }
}

impl Decode for CardinalLighting {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        String::decode(input, ops).flat_map(|(value, remaining)| {
            let lighting = match value.as_str() {
                "default" => DEFAULT,
                "nether" => NETHER,
                _ => return DataResult::new_error(format!("unknown cardinal lighting: {value}")),
            };
            DataResult::new_success((lighting, remaining))
        })
    }
}
