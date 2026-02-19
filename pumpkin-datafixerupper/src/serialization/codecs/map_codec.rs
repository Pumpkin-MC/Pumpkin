use crate::serialization::HasValue;
use crate::serialization::coders::{Decoder, Encoder};
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::map_codec::MapCodec;
use crate::serialization::struct_builder::StructBuilder;
use std::fmt::Display;

/// A [`Codec`] implementation for a [`MapCodec`].
///
/// The `MapCodec` held by this `Codec` can either be *owned* or a static reference (*borrowed*).
pub enum MapCodecCodec<C: MapCodec + 'static> {
    Owned(C),
    Borrowed(&'static C),
}

impl<C: MapCodec> MapCodecCodec<C> {
    const fn codec(&self) -> &C {
        match self {
            Self::Owned(c) => c,
            Self::Borrowed(c) => c,
        }
    }
}

impl<C: MapCodec> HasValue for MapCodecCodec<C> {
    type Value = C::Value;
}

impl<C: MapCodec> Encoder for MapCodecCodec<C> {
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        self.codec()
            .encode(input, ops, self.codec().builder(ops))
            .build(prefix)
    }
}

impl<C: MapCodec> Decoder for MapCodecCodec<C> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.codec()
            .compressed_decode(input.clone(), ops)
            .map(|a| (a, input))
    }
}
