use crate::serialization::map_codecs::field_coders::FieldDecoder;
use crate::serialization::{
    HasValue, data_result::DataResult, dynamic_ops::DynamicOps,
    map_codecs::field_coders::FieldEncoder,
};

/// A trait describing the way to encode something of a type `Value` into something else  (`Value -> ?`).
pub trait Encoder: HasValue {
    /// Encodes an input of this encoder's type (`A`) into an output of type `T`,
    /// along with a prefix (already encoded data).
    fn encode<T: PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T>;

    /// Encodes an input of this encoder's type (`A`) into an output of type `T`
    /// with no prefix (no already encoded data).
    fn encode_start<T: PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<T> {
        self.encode(input, ops, ops.empty())
    }
}

pub struct ComappedEncoderImpl<A, B, E: 'static> {
    encoder: &'static E,
    function: fn(&B) -> A,
}

impl<A, B, E> HasValue for ComappedEncoderImpl<A, B, E> {
    type Value = B;
}

impl<A, B, E: Encoder<Value = A>> Encoder for ComappedEncoderImpl<A, B, E> {
    fn encode<T: PartialEq + Clone>(
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
pub(crate) const fn comap<A, B, E: Encoder<Value = A>>(
    encoder: &'static E,
    f: fn(&B) -> A,
) -> ComappedEncoderImpl<A, B, E> {
    ComappedEncoderImpl {
        encoder,
        function: f,
    }
}

pub struct FlatComappedEncoderImpl<A, B, E: 'static> {
    encoder: &'static E,
    function: fn(&B) -> DataResult<A>,
}

impl<A, B, E> HasValue for FlatComappedEncoderImpl<A, B, E> {
    type Value = B;
}

impl<A, B, E: Encoder<Value = A>> Encoder for FlatComappedEncoderImpl<A, B, E> {
    fn encode<T: PartialEq + Clone>(
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
pub(crate) const fn flat_comap<A, B, E: Encoder<Value = A>>(
    encoder: &'static E,
    f: fn(&B) -> DataResult<A>,
) -> FlatComappedEncoderImpl<A, B, E> {
    FlatComappedEncoderImpl {
        encoder,
        function: f,
    }
}

pub(crate) const fn encoder_field_of<A, E: Encoder<Value = A>>(
    name: &'static str,
    encoder: &'static E,
) -> FieldEncoder<A, E> {
    FieldEncoder::new(name, encoder)
}

/// A trait describing the way to decode something of type into something of type `Value` (`? -> Value`).
pub trait Decoder: HasValue {
    /// Decodes an input of this decoder's type (`A`) into an output of type `T`,
    /// keeping the remaining undecoded data as another element of the tuple.
    fn decode<T: PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)>;

    /// Decodes an input of this decoder's type (`A`) into an output of type `T`,
    /// ignoring any remaining undecoded data (of type `A`).
    fn parse<T: PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value> {
        self.decode(input, ops).map(|r| r.0)
    }
}

pub struct MappedDecoderImpl<A, B, D: 'static> {
    decoder: &'static D,
    function: fn(&A) -> B,
}

impl<A, B, D> HasValue for MappedDecoderImpl<A, B, D> {
    type Value = B;
}

impl<A, B, D: Decoder<Value = A>> Decoder for MappedDecoderImpl<A, B, D> {
    fn decode<T: PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.decoder
            .decode(input, ops)
            .map(|(a, t)| ((self.function)(&a), t))
    }
}

/// Returns a *covariant mapped* transformation of a provided [`Decoder`].
/// A *mapped* decoder transforms the output after decoding.
pub(crate) const fn map<A, B, D: Decoder<Value = A>>(
    decoder: &'static D,
    f: fn(&A) -> B,
) -> MappedDecoderImpl<A, B, D> {
    MappedDecoderImpl {
        decoder,
        function: f,
    }
}

pub struct FlatMappedDecoderImpl<A, B, D: 'static> {
    decoder: &'static D,
    function: fn(&A) -> DataResult<B>,
}

impl<A, B, D> HasValue for FlatMappedDecoderImpl<A, B, D> {
    type Value = B;
}

impl<A, B, D: Decoder<Value = A>> Decoder for FlatMappedDecoderImpl<A, B, D> {
    fn decode<T: PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.decoder
            .decode(input, ops)
            .flat_map(|(a, t)| (self.function)(&a).map(|b| (b, t)))
    }
}

/// Returns a *covariant flat-mapped* transformation of a provided [`Decoder`].
/// A *flat-mapped* decoder transforms the output after decoding, but the transformation can fail.
pub(crate) const fn flat_map<A, B, D: Decoder<Value = A>>(
    decoder: &'static D,
    f: fn(&A) -> DataResult<B>,
) -> FlatMappedDecoderImpl<A, B, D> {
    FlatMappedDecoderImpl {
        decoder,
        function: f,
    }
}

pub(crate) const fn decoder_field_of<A, D: Decoder<Value = A>>(
    name: &'static str,
    decoder: &'static D,
) -> FieldDecoder<A, D> {
    FieldDecoder::new(name, decoder)
}
