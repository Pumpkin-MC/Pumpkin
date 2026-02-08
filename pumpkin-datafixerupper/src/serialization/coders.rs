use crate::serialization::{HasValue, data_result::DataResult, dynamic_ops::DynamicOps};

/// A trait describing the way to encode something of a type `Value` into something else  (`Value -> ?`).
pub trait Encoder: HasValue + 'static {
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

/// A trait describing the way to decode something of type into something of type `Value` (`? -> Value`).
pub trait Decoder: HasValue + 'static {
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

/*
pub struct ComappedEncoder<'a, A, B, F, E: ?Sized> {
    encoder: &'a E,
    f: F,
    phantom: PhantomData<(A, B)>,
}

impl<A, B, F, E> Encoder<B> for ComappedEncoder<'static, A, B, F, E>
where
    A: 'static,
    B: 'static,
    F: Fn(&B) -> A + 'static,
    E: Encoder<A>,
{
    fn encode<T: PartialEq>(
        &self,
        input: &B,
        ops: &impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        self.encoder.encode(&(self.f)(input), ops, prefix)
    }
}

*/
