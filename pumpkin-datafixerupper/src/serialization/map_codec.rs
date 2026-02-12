use crate::impl_compressor;
use crate::serialization::HasValue;
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::key_compressor::KeyCompressor;
use crate::serialization::keyable::Keyable;
use crate::serialization::map_coders::{CompressorHolder, MapDecoder, MapEncoder};
use crate::serialization::map_like::MapLike;
use crate::serialization::struct_builder::StructBuilder;
use std::fmt::Display;
use std::sync::OnceLock;

/// A type of *codec* which encodes/decodes fields of a map.
/// `Value` is the type of field/value this is responsible for.
/// This is functionally different from [`Codec`].
pub trait MapCodec: MapEncoder + MapDecoder {}

// Any struct implementing MapEncoder<Value = A> and MapDecoder<Value = A> will also implement MapCodec<Value = A>.
impl<T> MapCodec for T where T: MapEncoder + MapDecoder {}

/// A map codec allowing an arbitrary encoder and decoder.
pub struct ComposedMapCodec<
    A,
    E: MapEncoder<Value = A> + 'static,
    D: MapDecoder<Value = A> + 'static,
> {
    pub(crate) encoder: E,
    pub(crate) decoder: D,
    pub(crate) compressor: OnceLock<KeyCompressor>,
}

impl<A, E: MapEncoder<Value = A>, D: MapDecoder<Value = A>> HasValue for ComposedMapCodec<A, E, D> {
    type Value = A;
}

impl<A, E: MapEncoder<Value = A>, D: MapDecoder<Value = A>> Keyable for ComposedMapCodec<A, E, D> {
    fn keys(&self) -> Vec<String> {
        let mut vec = self.encoder.keys();
        vec.extend(self.decoder.keys());
        vec
    }
}

impl<A, E: MapEncoder<Value = A>, D: MapDecoder<Value = A>> CompressorHolder
    for ComposedMapCodec<A, E, D>
{
    impl_compressor!(compressor);
}

impl<A, E: MapEncoder<Value = A>, D: MapDecoder<Value = A>> MapEncoder
    for ComposedMapCodec<A, E, D>
{
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: impl StructBuilder<Value = T>,
    ) -> impl StructBuilder<Value = T> {
        self.encoder.encode(input, ops, prefix)
    }
}

impl<A, E: MapEncoder<Value = A>, D: MapDecoder<Value = A>> MapDecoder
    for ComposedMapCodec<A, E, D>
{
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value> {
        self.decoder.decode(input, ops)
    }
}
