use std::any::Any;

use pumpkin_codecs::{
    DataResult, Decode, DynamicOps, Encode,
    codec::{FieldDecode, FieldEncode},
    struct_builder::StructBuilder as _,
};
use pumpkin_util::{identifier::Identifier, random::RandomGenerator};

use super::{IntProvider, IntProviderValue, Pipe};

#[derive(Clone, Debug)]
pub struct ClampedIntProvider {
    pub source: IntProviderValue,
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

impl ClampedIntProvider {
    #[must_use]
    pub fn new(source: IntProviderValue, min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            source,
            min_inclusive,
            max_inclusive,
        }
    }
}

impl IntProvider for ClampedIntProvider {
    fn get_min(&self) -> i32 {
        self.min_inclusive.max(self.source.get_min())
    }
    fn get(&self, random: &mut RandomGenerator) -> i32 {
        self.source
            .get(random)
            .clamp(self.min_inclusive, self.max_inclusive)
    }
    fn get_max(&self) -> i32 {
        self.max_inclusive.min(self.source.get_max())
    }
    fn provider_type(&self) -> Identifier {
        Identifier::vanilla_static("clamped")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Encode for ClampedIntProvider {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.source
            .encode_field("source", ops, ops.map_builder())
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

impl Decode for ClampedIntProvider {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            IntProviderValue::decode_field::<O>("source", &map, ops).apply_3(
                |source, min_inclusive, max_inclusive| {
                    (
                        Self {
                            source,
                            min_inclusive,
                            max_inclusive,
                        },
                        ops.empty(),
                    )
                },
                i32::decode_field::<O>("min_inclusive", &map, ops),
                i32::decode_field::<O>("max_inclusive", &map, ops),
            )
        })
    }
}
