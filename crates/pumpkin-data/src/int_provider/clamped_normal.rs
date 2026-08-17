use std::any::Any;

use pumpkin_codecs::{
    DataResult, Decode, DynamicOps, Encode,
    codec::{FieldDecode, FieldEncode},
    struct_builder::StructBuilder as _,
};
use pumpkin_util::{
    identifier::Identifier,
    random::{RandomGenerator, RandomImpl},
};

use super::{IntProvider, Pipe};

#[derive(Clone, Debug)]
pub struct ClampedNormalIntProvider {
    pub mean: f32,
    pub deviation: f32,
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

impl ClampedNormalIntProvider {
    #[must_use]
    pub const fn new(mean: f32, deviation: f32, min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            mean,
            deviation,
            min_inclusive,
            max_inclusive,
        }
    }
}

impl IntProvider for ClampedNormalIntProvider {
    fn get_min(&self) -> i32 {
        self.min_inclusive
    }
    fn get(&self, random: &mut RandomGenerator) -> i32 {
        let value = (random.next_gaussian() as f32)
            .mul_add(self.deviation, self.mean)
            .round() as i32;
        value.clamp(self.min_inclusive, self.max_inclusive)
    }
    fn get_max(&self) -> i32 {
        self.max_inclusive
    }
    fn provider_type(&self) -> Identifier {
        Identifier::vanilla_static("clamped_normal")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Encode for ClampedNormalIntProvider {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.mean
            .encode_field("mean", ops, ops.map_builder())
            .pipe(|builder| self.deviation.encode_field("deviation", ops, builder))
            .pipe(|builder| {
                self.min_inclusive
                    .encode_field("min_inclusive", ops, builder)
            })
            .pipe(|builder| {
                self.max_inclusive
                    .encode_field("max_inclusive", ops, builder)
            })
            .build(prefix)
    }
}

impl Decode for ClampedNormalIntProvider {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            f32::decode_field::<O>("mean", &map, ops).apply_4(
                |mean, deviation, min_inclusive, max_inclusive| {
                    (
                        Self {
                            mean,
                            deviation,
                            min_inclusive,
                            max_inclusive,
                        },
                        ops.empty(),
                    )
                },
                f32::decode_field::<O>("deviation", &map, ops),
                i32::decode_field::<O>("min_inclusive", &map, ops),
                i32::decode_field::<O>("max_inclusive", &map, ops),
            )
        })
    }
}
