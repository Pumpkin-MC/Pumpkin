use std::marker::PhantomData;

use crate::serialization::{data_result::DataResult, dynamic_ops::DynamicOps};

/// A trait describing the way to encode something of a type `A` into something else  (`A -> ?`).
pub trait Encoder<A>: 'static {
    /// Encodes an input of this encoder's type (`A`) into an output of type `T`,
    /// along with a prefix (already encoded data).
    fn encode<T: PartialEq>(&self, input: &A, ops: &impl DynamicOps<Value = T>, prefix: T) -> DataResult<T>;

    /// Encodes an input of this encoder's type (`A`) into an output of type `T`
    /// with no prefix (no already encoded data).
    fn encode_start<T: PartialEq>(&self, input: &A, ops: &impl DynamicOps<Value = T>) -> DataResult<T> {
        self.encode(input, ops, ops.empty())
    }

    /// Transforms this `Encoder<A>` into another `Encoder<B>` by using a function
    /// that maps a value of `B` to another of `A` for encoding.
    fn comap<B, F>(&self, f: F) -> ComappedEncoder<'_, A, B, F, Self>
    where
        B: 'static,
        F: Fn(&B) -> A + 'static,
        Self: Sized,
    {
        ComappedEncoder {
            encoder: self,
            f,
            phantom: PhantomData,
        }
    }
}

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
    fn encode<T: PartialEq>(&self, input: &B, ops: &impl DynamicOps<Value = T>, prefix: T) -> DataResult<T> {
        self.encoder.encode(&(self.f)(input), ops, prefix)
    }
}
