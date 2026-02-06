use std::marker::PhantomData;

use crate::serialization::{
    data_result::DataResult, decoder::Decoder, dynamic_ops::DynamicOps, encoder::Encoder,
};

/// A trait describing the way to **encode and decode** something of a type `A` into something else  (`A -> ?` and `?` -> `A`).
pub trait Codec<A, Kind>: Encoder<A> + Decoder<A> {}

// Any struct implementing Encoder<A> and Decoder<A> will also implement Codec<A>.
impl<A, T, Kind> Codec<A, Kind> for T
where
    T: Encoder<A> + Decoder<A> + 'static,
    A: 'static,
{
}

fn new<A: 'static, Kind, E, D>(encoder: E, decoder: D) -> impl Codec<A, Kind>
where
    E: Encoder<A>,
    D: Decoder<A>,
{
    BaseCodec {
        encoder,
        decoder,
        phantom: PhantomData,
    }
}

/// A base codec type, which accepts an [`Encoder`] and [`Decoder`] of the same type.
/// This is used by more complex codec types.
#[derive(Debug)]
struct BaseCodec<A, E, D> {
    encoder: E,
    decoder: D,
    phantom: PhantomData<A>,
}

impl<A, E, D> Encoder<A> for BaseCodec<A, E, D>
where
    A: 'static,
    E: Encoder<A>,
    D: Decoder<A>,
{
    fn encode<T: PartialEq>(
        &self,
        input: &A,
        ops: &impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        self.encoder.encode(input, ops, prefix)
    }
}

impl<A, E, D> Decoder<A> for BaseCodec<A, E, D>
where
    A: 'static,
    E: Encoder<A>,
    D: Decoder<A>,
{
    fn decode<T: PartialEq>(
        &self,
        input: T,
        ops: &impl DynamicOps<Value = T>,
    ) -> DataResult<(A, T)> {
        self.decoder.decode(input, ops)
    }
}
