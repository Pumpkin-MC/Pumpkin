use crate::impl_compressor;
use crate::serialization::HasValue;
use crate::serialization::coders::{Decoder, Encoder};
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::key_compressor::KeyCompressor;
use crate::serialization::keyable::Keyable;
use crate::serialization::map_coders::{CompressorHolder, MapDecoder, MapEncoder};
use crate::serialization::map_like::MapLike;
use crate::serialization::struct_builder::StructBuilder;
use std::fmt::Display;
use std::sync::OnceLock;

/// A [`MapEncoder`] that knows how to encode an entire field (key + value).
/// `A` is the type of value encoded.
pub struct FieldEncoder<A, E: Encoder<Value = A> + 'static> {
    /// The name of the key.
    name: &'static str,
    /// The [`Encoder`] for encoding the value.
    element_encoder: &'static E,
    /// The [`KeyCompressor`] for this encoder.
    compressor: OnceLock<KeyCompressor>,
}

impl<A, E: Encoder<Value = A>> HasValue for FieldEncoder<A, E> {
    type Value = A;
}

impl<A, E: Encoder<Value = A>> Keyable for FieldEncoder<A, E> {
    fn keys(&self) -> Vec<String> {
        vec![self.name.to_string()]
    }
}

impl<A, E: Encoder<Value = A>> CompressorHolder for FieldEncoder<A, E> {
    impl_compressor!(compressor);
}

impl<A, E: Encoder<Value = A>> MapEncoder for FieldEncoder<A, E> {
    fn encode<T: Display + PartialEq + Clone, B: StructBuilder<Value = T>>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: B,
    ) -> B {
        prefix.add_string_key_value_result(self.name, self.element_encoder.encode_start(input, ops))
    }
}

impl<A, E: Encoder<Value = A>> FieldEncoder<A, E> {
    /// Returns a new [`FieldEncoder`] with the provided name and [`Encoder`].
    pub(crate) const fn new(name: &'static str, element_encoder: &'static E) -> Self {
        Self {
            name,
            element_encoder,
            compressor: OnceLock::new(),
        }
    }
}

/// A [`MapDecoder`] that knows how to decode an entire field (key + value).
/// `A` is the type of value that the decoder can decode into.
pub struct FieldDecoder<A, D: Decoder<Value = A> + 'static> {
    /// The name of the key.
    name: &'static str,
    /// The [`Decoder`] for encoding the value.
    element_decoder: &'static D,
    /// The [`KeyCompressor`] for this encoder.
    compressor: OnceLock<KeyCompressor>,
}

impl<A, D: Decoder<Value = A>> HasValue for FieldDecoder<A, D> {
    type Value = A;
}

impl<A, D: Decoder<Value = A>> Keyable for FieldDecoder<A, D> {
    fn keys(&self) -> Vec<String> {
        vec![self.name.to_string()]
    }
}

impl<A, D: Decoder<Value = A>> CompressorHolder for FieldDecoder<A, D> {
    impl_compressor!(compressor);
}

impl<A, D: Decoder<Value = A>> MapDecoder for FieldDecoder<A, D> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value> {
        input.get_str(self.name).map_or_else(
            || DataResult::error(format!("No key {} in map", self.name)),
            |v| self.element_decoder.parse(v.clone(), ops),
        )
    }
}

impl<A, D: Decoder<Value = A>> FieldDecoder<A, D> {
    /// Returns a new [`FieldDecoder`] with the provided name and [`Decoder`].
    pub(crate) const fn new(name: &'static str, element_decoder: &'static D) -> Self {
        Self {
            name,
            element_decoder,
            compressor: OnceLock::new(),
        }
    }
}
