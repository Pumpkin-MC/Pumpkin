use crate::serialization::map_codecs::field_coders::FieldDecoder;
use crate::serialization::{
    HasValue, data_result::DataResult, dynamic_ops::DynamicOps,
    map_codecs::field_coders::FieldEncoder,
};
use std::fmt::Display;

/// A trait describing the way to encode something of a type `Value` into something else  (`Value -> ?`).
pub trait Encoder: HasValue {
    /// Encodes an input of this encoder's type (`A`) into an output of type `T`,
    /// along with a prefix (already encoded data).
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T>;

    /// Encodes an input of this encoder's type (`A`) into an output of type `T`
    /// with no prefix (no already encoded data).
    fn encode_start<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<T> {
        self.encode(input, ops, ops.empty())
    }
}

pub struct ComappedEncoderImpl<B, E: Encoder + 'static> {
    encoder: &'static E,
    function: fn(&B) -> E::Value,
}

impl<B, E: Encoder> HasValue for ComappedEncoderImpl<B, E> {
    type Value = B;
}

impl<B, E: Encoder> Encoder for ComappedEncoderImpl<B, E> {
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        self.encoder.encode(&(self.function)(input), ops, prefix)
    }
}

/// Returns a *contramapped* (*comapped*) transformation of a provided [`Encoder`].
/// A *comapped* encoder transforms the input before encoding.
pub(crate) const fn comap<B, E: Encoder>(
    encoder: &'static E,
    f: fn(&B) -> E::Value,
) -> ComappedEncoderImpl<B, E> {
    ComappedEncoderImpl {
        encoder,
        function: f,
    }
}

pub struct FlatComappedEncoderImpl<B, E: Encoder + 'static> {
    encoder: &'static E,
    function: fn(&B) -> DataResult<E::Value>,
}

impl<B, E: Encoder> HasValue for FlatComappedEncoderImpl<B, E> {
    type Value = B;
}

impl<B, E: Encoder> Encoder for FlatComappedEncoderImpl<B, E> {
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        (self.function)(input).flat_map(|a| self.encoder.encode(&a, ops, prefix))
    }
}

/// Returns a *flat contramapped* (*flat-comapped*) transformation of a provided [`Encoder`].
/// A *flat comapped* encoder transforms the input before encoding, but the transformation can fail.
pub(crate) const fn flat_comap<B, E: Encoder>(
    encoder: &'static E,
    f: fn(&B) -> DataResult<E::Value>,
) -> FlatComappedEncoderImpl<B, E> {
    FlatComappedEncoderImpl {
        encoder,
        function: f,
    }
}

pub(crate) const fn encoder_field<A, E: Encoder<Value = A>>(
    name: &'static str,
    encoder: &'static E,
) -> FieldEncoder<A, E> {
    FieldEncoder::new(name, encoder)
}

/// A trait describing the way to decode something of some type to something of type `Value` (`? -> Value`).
pub trait Decoder: HasValue {
    /// Decodes an input of this decoder's type (`A`) into an output of type `T`,
    /// keeping the remaining undecoded data as another element of the tuple.
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)>;

    /// Decodes an input of this decoder's type (`A`) into an output of type `T`.
    fn parse<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value> {
        self.decode(input, ops).map(|r| r.0)
    }
}

pub struct MappedDecoderImpl<B, D: Decoder + 'static> {
    decoder: &'static D,
    function: fn(D::Value) -> B,
}

impl<B, D: Decoder> HasValue for MappedDecoderImpl<B, D> {
    type Value = B;
}

impl<B, D: Decoder> Decoder for MappedDecoderImpl<B, D> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.decoder
            .decode(input, ops)
            .map(|(a, t)| ((self.function)(a), t))
    }
}

/// Returns a *covariant mapped* transformation of a provided [`Decoder`].
/// A *mapped* decoder transforms the output after decoding.
pub(crate) const fn map<B, D: Decoder>(
    decoder: &'static D,
    f: fn(D::Value) -> B,
) -> MappedDecoderImpl<B, D> {
    MappedDecoderImpl {
        decoder,
        function: f,
    }
}

pub struct FlatMappedDecoderImpl<B, D: Decoder + 'static> {
    decoder: &'static D,
    function: fn(D::Value) -> DataResult<B>,
}

impl<B, D: Decoder> HasValue for FlatMappedDecoderImpl<B, D> {
    type Value = B;
}

impl<B, D: Decoder> Decoder for FlatMappedDecoderImpl<B, D> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.decoder
            .decode(input, ops)
            .flat_map(|(a, t)| (self.function)(a).map(|b| (b, t)))
    }
}

/// Returns a *covariant flat-mapped* transformation of a provided [`Decoder`].
/// A *flat-mapped* decoder transforms the output after decoding, but the transformation can fail.
pub(crate) const fn flat_map<B, D: Decoder>(
    decoder: &'static D,
    f: fn(D::Value) -> DataResult<B>,
) -> FlatMappedDecoderImpl<B, D> {
    FlatMappedDecoderImpl {
        decoder,
        function: f,
    }
}

pub(crate) const fn decoder_field<A, D: Decoder<Value = A>>(
    name: &'static str,
    decoder: &'static D,
) -> FieldDecoder<A, D> {
    FieldDecoder::new(name, decoder)
}
