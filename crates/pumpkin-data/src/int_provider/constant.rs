use std::any::Any;

use pumpkin_codecs::{
    DataResult, Decode, DynamicOps, Encode,
    codec::{FieldDecode, FieldEncode},
    struct_builder::StructBuilder as _,
};
use pumpkin_util::{identifier::Identifier, random::RandomGenerator};

use super::IntProvider;

#[derive(Clone, Debug)]
pub struct ConstantIntProvider {
    pub value: i32,
}

impl ConstantIntProvider {
    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self { value }
    }
}

impl IntProvider for ConstantIntProvider {
    fn get_min(&self) -> i32 {
        self.value
    }
    fn get(&self, _random: &mut RandomGenerator) -> i32 {
        self.value
    }
    fn get_max(&self) -> i32 {
        self.value
    }
    fn provider_type(&self) -> Identifier {
        Identifier::vanilla_static("constant")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Encode for ConstantIntProvider {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.value
            .encode_field("value", ops, ops.map_builder())
            .build(prefix)
    }
}

impl Decode for ConstantIntProvider {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            i32::decode_field::<O>("value", &map, ops).map(|value| (Self { value }, ops.empty()))
        })
    }
}
