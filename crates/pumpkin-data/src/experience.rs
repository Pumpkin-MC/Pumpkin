use pumpkin_codecs::{
    DataResult, Decode, DynamicOps, Encode,
    codec::{FieldDecode, FieldEncode},
    struct_builder::StructBuilder as _,
};

use crate::int_provider::{IntProvider, IntProviderValue};

#[derive(Clone, Debug)]
pub struct Experience {
    pub experience: IntProviderValue,
}

impl Encode for Experience {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        self.experience
            .encode_field("experience", ops, ops.map_builder())
            .build(prefix)
    }
}

impl Decode for Experience {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            IntProviderValue::decode_field::<O>("experience", &map, ops)
                .map(|experience| (Self { experience }, ops.empty()))
        })
    }
}
