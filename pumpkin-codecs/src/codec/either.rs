use crate::{DataResult, Decode, DynamicOps, Encode};
use either::Either;

impl<L: Encode, R: Encode> Encode for Either<L, R> {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        match self {
            Self::Left(l) => l.encode(ops, prefix),
            Self::Right(r) => r.encode(ops, prefix),
        }
    }
}

impl<L: Decode, R: Decode> Decode for Either<L, R> {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        let left = L::decode(input.clone(), ops).map(|(l, t)| (Self::Left(l), t));

        // If the left result is a success, return that.
        if left.is_success() {
            return left;
        }

        let right = R::decode(input, ops).map(|(r, t)| (Self::Right(r), t));

        // If the right result is a success, return that.
        if right.is_success() {
            return right;
        }

        // Since no result is a complete success by this point, we look for partial results.

        if left.has_result_or_partial() {
            return left;
        }

        if right.has_result_or_partial() {
            return right;
        }

        DataResult::new_error(format!(
            "Failed to parse either. First: {}; Second: {}",
            left.get_message().unwrap(),
            right.get_message().unwrap()
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::json_ops::JsonOps;
    use crate::{assert_decode, assert_encode_success};
    use either::Either;
    use serde_json::json;

    #[test]
    fn simple() {
        assert_encode_success!(Either::<i32, String>::Left(5), JsonOps, json!(5));

        assert_encode_success!(
            Either::<i32, String>::Right("I am some text.".to_string()),
            JsonOps,
            json!("I am some text.")
        );

        // Decoding

        assert_decode!(
            Either<i32, String>,
            json!(-238),
            JsonOps,
            is_success
        );
        assert_decode!(
            Either<u32, String>,
            json!(-238),
            JsonOps,
            is_error
        );
        assert_decode!(
            Either<u32, String>,
            json!("hello"),
            JsonOps,
            is_success
        );
        assert_decode!(
            Either<u32, String>,
            json!(true),
            JsonOps,
            is_error
        );
    }
}
