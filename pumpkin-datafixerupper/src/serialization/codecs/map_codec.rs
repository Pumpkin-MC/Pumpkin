use crate::serialization::HasValue;
use crate::serialization::coders::{Decoder, Encoder};
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::map_codec::MapCodec;
use crate::serialization::struct_builder::StructBuilder;
use std::fmt::Display;

/// A [`Codec`] implementation for a [`MapCodec`].
pub struct MapCodecCodec<A, C: MapCodec<Value = A>> {
    pub(crate) codec: C,
}

impl<A, C: MapCodec<Value = A>> HasValue for MapCodecCodec<A, C> {
    type Value = A;
}

impl<A, C: MapCodec<Value = A>> Encoder for MapCodecCodec<A, C> {
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        self.codec
            .encode(input, ops, self.codec.builder(ops))
            .build(prefix)
    }
}

impl<A, C: MapCodec<Value = A>> Decoder for MapCodecCodec<A, C> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.codec
            .compressed_decode(input.clone(), ops)
            .map(|a| (a, input))
    }
}
