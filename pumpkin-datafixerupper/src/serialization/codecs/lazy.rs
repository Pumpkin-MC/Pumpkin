use crate::serialization::HasValue;
use crate::serialization::codec::Codec;
use crate::serialization::coders::{Decoder, Encoder};
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use std::fmt::Display;
use std::sync::LazyLock;

/// A type of [`Codec`] that initializes an inner [`Codec`] on first use.
pub struct LazyCodec<C>
where
    C: Codec,
{
    codec: LazyLock<C>,
}

impl<C: Codec> HasValue for LazyCodec<C> {
    type Value = C::Value;
}

impl<C: Codec> Encoder for LazyCodec<C> {
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        self.codec.encode(input, ops, prefix)
    }
}

impl<C: Codec> Decoder for LazyCodec<C> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.codec.decode(input, ops)
    }
}

/// Creates a new [`LazyCodec`].
pub(crate) const fn new_lazy_codec<C: Codec>(f: fn() -> C) -> LazyCodec<C> {
    LazyCodec {
        codec: LazyLock::new(f),
    }
}
