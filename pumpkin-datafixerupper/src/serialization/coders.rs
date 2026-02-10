use crate::serialization::{HasValue, data_result::DataResult, dynamic_ops::DynamicOps};
use std::marker::PhantomData;

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

    /// Returns a *contramapped* (*comapped*) transformation of a provided [`Encoder`].
    /// A *comapped* encoder transforms the input before encoding.
    fn comap<A, B, F>(&self, f: F) -> impl Encoder<Value = B>
    where
        F: Fn(&B) -> A,
        Self: Encoder<Value = A> + Sized,
    {
        struct ComappedEncoderImpl<'a, B, E, F> {
            encoder: &'a E,
            function: F,
            phantom: PhantomData<B>,
        }

        impl<B, E, F> HasValue for ComappedEncoderImpl<'_, B, E, F> {
            type Value = B;
        }

        impl<A, B, E: Encoder<Value = A>, F: Fn(&B) -> A> Encoder for ComappedEncoderImpl<'_, B, E, F> {
            fn encode<T: PartialEq + Clone>(
                &self,
                input: &Self::Value,
                ops: &'static impl DynamicOps<Value = T>,
                prefix: T,
            ) -> DataResult<T> {
                self.encoder.encode(&(self.function)(input), ops, prefix)
            }
        }

        ComappedEncoderImpl {
            encoder: self,
            function: f,
            phantom: PhantomData,
        }
    }

    /// Returns a *flat contramapped* (*flat-comapped*) transformation of a provided [`Encoder`].
    /// A *flat comapped* encoder transforms the input before encoding, but the transformation can fail.
    fn flat_comap<A, B, F>(&self, f: F) -> impl Encoder<Value = B>
    where
        F: Fn(&B) -> DataResult<A>,
        Self: Encoder<Value = A> + Sized,
    {
        struct FlatComappedEncoderImpl<'a, B, E, F> {
            encoder: &'a E,
            function: F,
            phantom: PhantomData<B>,
        }

        impl<B, E, F> HasValue for FlatComappedEncoderImpl<'_, B, E, F> {
            type Value = B;
        }

        impl<A, B, E: Encoder<Value = A>, F: Fn(&B) -> DataResult<A>> Encoder
            for FlatComappedEncoderImpl<'_, B, E, F>
        {
            fn encode<T: PartialEq + Clone>(
                &self,
                input: &Self::Value,
                ops: &'static impl DynamicOps<Value = T>,
                prefix: T,
            ) -> DataResult<T> {
                (self.function)(input).flat_map(|a| self.encoder.encode(&a, ops, prefix))
            }
        }

        FlatComappedEncoderImpl {
            encoder: self,
            function: f,
            phantom: PhantomData,
        }
    }
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

    /// Returns a *covariant mapped* transformation of a provided [`Decoder`].
    /// A *mapped* decoder transforms the output after decoding.
    fn map<A, B, F>(&self, f: F) -> impl Decoder<Value = B>
    where
        F: Fn(&A) -> B,
        Self: Decoder<Value = A> + Sized,
    {
        struct MappedDecoderImpl<'a, B, D, F> {
            decoder: &'a D,
            function: F,
            phantom: PhantomData<B>,
        }

        impl<B, D, F> HasValue for MappedDecoderImpl<'_, B, D, F> {
            type Value = B;
        }

        impl<A, B, D: Decoder<Value = A>, F: Fn(&A) -> B> Decoder for MappedDecoderImpl<'_, B, D, F> {
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

        MappedDecoderImpl {
            decoder: self,
            function: f,
            phantom: PhantomData,
        }
    }

    /// Returns a *covariant flat-mapped* transformation of a provided [`Decoder`].
    /// A *flat-mapped* decoder transforms the output after decoding, but the transformation can fail.
    fn flat_map<A, B, F>(&self, f: F) -> impl Decoder<Value = B>
    where
        F: Fn(&A) -> DataResult<B>,
        Self: Decoder<Value = A> + Sized,
    {
        struct FlatMappedDecoderImpl<'a, B, D, F> {
            decoder: &'a D,
            function: F,
            phantom: PhantomData<B>,
        }

        impl<B, D, F> HasValue for FlatMappedDecoderImpl<'_, B, D, F> {
            type Value = B;
        }

        impl<A, B, D: Decoder<Value = A>, F: Fn(&A) -> DataResult<B>> Decoder
            for FlatMappedDecoderImpl<'_, B, D, F>
        {
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

        FlatMappedDecoderImpl {
            decoder: self,
            function: f,
            phantom: PhantomData,
        }
    }
}
