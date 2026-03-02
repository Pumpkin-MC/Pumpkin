use crate::HasValue;
use crate::data_result::DataResult;
use crate::dynamic_ops::DynamicOps;
use crate::impl_compressor;
use crate::key_compressor::KeyCompressor;
use crate::keyable::Keyable;
use crate::map_codec::MapCodec;
use crate::map_coders::{CompressorHolder, MapDecoder, MapEncoder};
use crate::map_like::MapLike;
use crate::struct_builder::StructBuilder;
use either::Either;
use std::fmt::Display;

/// A [`MapCodec`] that can serialize/deserialize one of two types, with a map codec for each one.
///
/// This evaluates the left map codec first, and if the [`DataResult`] for it is invalid,
/// it evaluates the right map codec.
pub struct EitherMapCodec<L: MapCodec + 'static, R: MapCodec + 'static> {
    left_codec: &'static L,
    right_codec: &'static R,
}

impl<L: MapCodec, R: MapCodec> HasValue for EitherMapCodec<L, R> {
    type Value = Either<L::Value, R::Value>;
}

impl<L: MapCodec, R: MapCodec> Keyable for EitherMapCodec<L, R> {
    fn keys(&self) -> Vec<String> {
        let mut keys = self.left_codec.keys();
        keys.extend(self.right_codec.keys());
        keys
    }
}

impl<L: MapCodec, R: MapCodec> CompressorHolder for EitherMapCodec<L, R> {
    impl_compressor!();
}

impl<L: MapCodec, R: MapCodec> MapEncoder for EitherMapCodec<L, R> {
    fn encode<T: Display + PartialEq + Clone, B: StructBuilder<Value = T>>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: B,
    ) -> B {
        match &input {
            Either::Left(l) => self.left_codec.encode(l, ops, prefix),
            Either::Right(r) => self.right_codec.encode(r, ops, prefix),
        }
    }
}

impl<L: MapCodec, R: MapCodec> MapDecoder for EitherMapCodec<L, R> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self::Value> {
        let left = self.left_codec.decode(input, ops).map(Either::Left);
        if left.is_success() {
            return left;
        }
        let right = self.right_codec.decode(input, ops).map(Either::Right);
        if right.is_success() {
            return right;
        }
        left.apply_2(|_, r| r, right)
    }
}

/// Creates a new `EitherMapCodec` with the provided left and right codecs for serializing/deserializing both possible types.
pub(crate) const fn new_either_map_codec<L: MapCodec, R: MapCodec>(
    left_codec: &'static L,
    right_codec: &'static R,
) -> EitherMapCodec<L, R> {
    EitherMapCodec {
        left_codec,
        right_codec,
    }
}
