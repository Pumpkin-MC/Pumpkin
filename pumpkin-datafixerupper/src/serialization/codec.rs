use crate::serialization::{
    coders::{Decoder, Encoder},
    data_result::DataResult,
    dynamic_ops::DynamicOps,
};

/// A trait describing the way to **encode from and decode into** something of a type `Value`  (`Value -> ?` and `?` -> `Value`).
pub trait Codec: Encoder + Decoder {}

// Any struct implementing Encoder<A> and Decoder<A> will also implement Codec<A>.
impl<T> Codec for T where T: Encoder + Decoder {}

/*

/// Creates a new `Codec` with a provided [`Encoder`] and [`Decoder`].
fn new<A: 'static, E, D>(encoder: E, decoder: D) -> impl Codec<Value = A>
where
    E: Encoder<Value = A>,
    D: Decoder<Value = A>,
{
    BaseCodec { encoder, decoder }
}

/// A base codec type, which accepts an [`Encoder`] and [`Decoder`] of the same type.
/// This is used by more complex codec types.
#[derive(Debug)]
struct BaseCodec<E, D> {
    encoder: E,
    decoder: D,
}

impl<E, D> Encoder for BaseCodec<E, D>
where
    E: Encoder,
    D: Decoder,
{
    type Value = E::Value;

    fn encode<T: PartialEq>(
        &self,
        input: &Self::Value,
        ops: &impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        self.encoder.encode(input, ops, prefix)
    }
}

impl<E, D> Decoder for BaseCodec<E, D>
where
    E: Encoder,
    D: Decoder,
{
    type Value = E::Value;

    fn decode<T: PartialEq>(
        &self,
        input: T,
        ops: &impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.decoder.decode(input, ops)
    }
}

*/
