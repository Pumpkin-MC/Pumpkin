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
pub struct UniformIntProvider {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

impl UniformIntProvider {
    #[must_use]
    pub const fn new(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }
}

impl IntProvider for UniformIntProvider {
    fn get_min(&self) -> i32 {
        self.min_inclusive
    }
    fn get(&self, random: &mut RandomGenerator) -> i32 {
        random.next_inbetween_i32(self.min_inclusive, self.max_inclusive)
    }
    fn get_max(&self) -> i32 {
        self.max_inclusive
    }
    fn provider_type(&self) -> Identifier {
        Identifier::vanilla_static("uniform")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Encode for UniformIntProvider {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.min_inclusive
            .encode_field("min_inclusive", ops, ops.map_builder())
            .pipe(|builder| {
                self.max_inclusive
                    .encode_field("max_inclusive", ops, builder)
            })
            .build(prefix)
    }
}

impl Decode for UniformIntProvider {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            i32::decode_field::<O>("min_inclusive", &map, ops).apply_2(
                |min_inclusive, max_inclusive| {
                    (
                        Self {
                            min_inclusive,
                            max_inclusive,
                        },
                        ops.empty(),
                    )
                },
                i32::decode_field::<O>("max_inclusive", &map, ops),
            )
        })
    }
}
