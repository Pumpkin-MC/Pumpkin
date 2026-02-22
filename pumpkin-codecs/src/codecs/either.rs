use std::fmt::Display;
use crate::codec::Codec;
use crate::coders::{Decoder, Encoder};
use crate::data_result::DataResult;
use crate::dynamic_ops::DynamicOps;
use crate::HasValue;
use crate::util::either::Either;

/// A codec that can serialize/deserialize one of two types, with a codec for each one.
///
/// This evaluates the left codec first, and if the [`DataResult`] for it is invalid,
/// it evaluates the right codec.
pub struct EitherCodec<L: Codec + 'static, R: Codec + 'static> {
    left_codec: &'static L,
    right_codec: &'static R,
}

impl<L: Codec, R: Codec> HasValue for EitherCodec<L, R> {
    type Value = Either<L::Value, R::Value>;
}

impl<L: Codec, R: Codec> Encoder for EitherCodec<L, R> {
    fn encode<T: Display + PartialEq + Clone>(&self, input: &Self::Value, ops: &'static impl DynamicOps<Value=T>, prefix: T) -> DataResult<T> {
        match &input {
            Either::Left(l) => self.left_codec.encode(l, ops, prefix),
            Either::Right(r) => self.right_codec.encode(r, ops, prefix)
        }
    }
}

impl<L: Codec, R: Codec> Decoder for EitherCodec<L, R> {
    fn decode<T: Display + PartialEq + Clone>(&self, input: T, ops: &'static impl DynamicOps<Value=T>) -> DataResult<(Self::Value, T)> {
        let left = self.left_codec.decode(input.clone(), ops).map(
            |(l, t)| (Either::Left(l), t)
        );

        // If the left result is a success, return that.
        if left.is_success() { return left; }

        let right = self.right_codec.decode(input, ops).map(
            |(r, t)| (Either::Right(r), t)
        );

        // If the right result is a success, return that.
        if right.is_success() { return right; }

        // Since no result is a complete success by this point, we look for partial results.

        if left.has_result_or_partial() {
            return left;
        }

        if right.has_result_or_partial() {
            return right;
        }

        DataResult::error(format!("Failed to parse either. First: {}; Second: {}", left.get_message().unwrap(), right.get_message().unwrap()))
    }
}

/// Creates a new `EitherCodec` with the provided left and right codecs for serializing/deserializing both possible types.
pub(crate) const fn new_either_codec<L: Codec, R: Codec>(left_codec: &'static L, right_codec: &'static R) -> EitherCodec<L, R> {
    EitherCodec {
        left_codec,
        right_codec
    }
}
