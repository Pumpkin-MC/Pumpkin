use crate::serialization::HasValue;
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::key_compressor::KeyCompressor;
use crate::serialization::keyable::Keyable;
use crate::serialization::map_codec::MapCodec;
use crate::serialization::map_coders::{CompressorHolder, MapDecoder, MapEncoder};
use crate::serialization::map_like::MapLike;
use crate::serialization::struct_builder::StructBuilder;
use std::fmt::Display;

/// A validator [`MapCodec`] that validates any values before encoding and after decoding.
pub struct ValidatedMapCodec<C: MapCodec + 'static> {
    codec: &'static C,
    /// The validator function used.
    validator: fn(&C::Value) -> Result<(), String>,
}

impl<C: MapCodec> HasValue for ValidatedMapCodec<C> {
    type Value = C::Value;
}

impl<C: MapCodec> Keyable for ValidatedMapCodec<C> {
    fn keys(&self) -> Vec<String> {
        self.codec.keys()
    }
}

impl<C: MapCodec> CompressorHolder for ValidatedMapCodec<C> {
    fn compressor(&self) -> &KeyCompressor {
        self.codec.compressor()
    }
}

impl<C: MapCodec> MapEncoder for ValidatedMapCodec<C> {
    fn encode<T: Display + PartialEq + Clone, B: StructBuilder<Value = T>>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: B,
    ) -> B {
        match (self.validator)(input) {
            Ok(()) => self.codec.encode(input, ops, prefix),
            Err(s) => prefix.with_errors_from(&DataResult::<()>::error(s)),
        }
    }
}

impl<C: MapCodec> MapDecoder for ValidatedMapCodec<C> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value> {
        let result = self.codec.decode(input, ops);
        if let Some(v) = result.result_or_partial_as_ref() {
            (self.validator)(v).map_or_else(DataResult::error, |()| result)
        } else {
            result
        }
    }
}

/// Creates a new [`ValidatedMapCodec`].
pub(crate) const fn new_validated_map_codec<C: MapCodec>(
    codec: &'static C,
    validator: fn(&C::Value) -> Result<(), String>,
) -> ValidatedMapCodec<C> {
    ValidatedMapCodec { codec, validator }
}
