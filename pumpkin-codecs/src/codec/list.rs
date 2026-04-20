use crate::list_builder::ListBuilder;
use crate::{DataResult, Decode, DynamicOps, Encode, FlatTryFrom, Lifecycle};

/// A wrapped [`Vec`] that can only contain a size of elements between `MIN` and `MAX` (inclusive).
pub struct BoundedVec<T, const MIN: usize, const MAX: usize>(Vec<T>);

impl<T, const MIN: usize, const MAX: usize> From<BoundedVec<T, MIN, MAX>> for Vec<T> {
    fn from(value: BoundedVec<T, MIN, MAX>) -> Self {
        value.0
    }
}

impl<T, const MIN: usize, const MAX: usize> FlatTryFrom<Vec<T>> for BoundedVec<T, MIN, MAX> {
    fn flat_try_from(value: Vec<T>) -> DataResult<Self> {
        let size = value.len();
        if size < MIN {
            create_too_short_error(MIN, MAX, size)
        } else if size > MAX {
            create_too_long_error(MIN, MAX, size)
        } else {
            DataResult::new_success(Self(value))
        }
    }
}

fn create_too_short_error<T>(min: usize, max: usize, size: usize) -> DataResult<T> {
    DataResult::new_error(format!(
        "List is too short: {size}, expected range [{min}-{max}]"
    ))
}

fn create_too_long_error<T>(min: usize, max: usize, size: usize) -> DataResult<T> {
    DataResult::new_error(format!(
        "List is too long: {size}, expected range [{min}-{max}]"
    ))
}

impl<T, const MIN: usize, const MAX: usize> Encode for BoundedVec<T, MIN, MAX>
where
    T: Encode,
{
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        let size = self.0.len();
        if size < MIN {
            create_too_short_error(MIN, MAX, size)
        } else if size > MAX {
            create_too_long_error(MIN, MAX, size)
        } else {
            let mut builder = ops.list_builder();
            for e in &self.0 {
                builder = builder.add_data_result(e.encode_start(ops));
            }
            builder.build(prefix)
        }
    }
}

impl<T, const MIN: usize, const MAX: usize> Decode for BoundedVec<T, MIN, MAX>
where
    T: Decode,
{
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        let iter = ops.get_iter(input).with_lifecycle(Lifecycle::Stable);
        iter.flat_map(|i| {
            let mut total_count = 0;
            let mut elements: Vec<T> = vec![];
            let mut failed: Vec<O::Value> = vec![];
            // This is used to keep track of the overall `DataResult`.
            // If any one element has a partial result, this turns into a partial result.
            // If any one element has no result, this turns into a non-result.
            let mut result = DataResult::new_success(());

            for element in i {
                total_count += 1;
                if elements.len() >= MAX {
                    failed.push(element.clone());
                    continue;
                }
                let element_result = T::decode(element.clone(), ops);
                result = result.add_message(&element_result);
                if let Some(element) = element_result.into_result_or_partial() {
                    elements.push(element.0);
                }
            }

            if total_count < MIN {
                return create_too_short_error(MIN, MAX, total_count);
            }

            let pair = (Self(elements), ops.create_list(failed));
            if total_count > MAX {
                result = create_too_long_error(MIN, MAX, total_count);
            }
            result.with_complete_or_partial(pair)
        })
    }
}

impl<T> Encode for Vec<T>
where
    T: Encode,
{
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        let mut builder = ops.list_builder();
        for e in self {
            builder = builder.add_data_result(e.encode_start(ops));
        }
        builder.build(prefix)
    }
}

impl<T> Decode for Vec<T>
where
    T: Decode,
{
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        let iter = ops.get_iter(input).with_lifecycle(Lifecycle::Stable);
        iter.flat_map(|i| {
            let mut elements: Self = vec![];
            let mut result = DataResult::new_success(());

            for element in i {
                let element_result = T::decode(element.clone(), ops);
                result = result.add_message(&element_result);
                if let Some(element) = element_result.into_result_or_partial() {
                    elements.push(element.0);
                }
            }

            let pair = (elements, ops.create_list(Vec::new()));
            result.with_complete_or_partial(pair)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::assert_decode;
    use crate::assert_encode_success;
    use crate::json_ops::JsonOps;
    use serde_json::json;

    #[test]
    fn encoding() {
        assert_encode_success!(vec![1, 2], JsonOps, json!([1, 2]));
        let vec: Vec<i32> = vec![];
        assert_encode_success!(vec, JsonOps, json!([]));
        assert_encode_success!(vec![-3, 192, 182], JsonOps, json!([-3, 192, 182]));

        assert_encode_success!(
            vec!["a".to_string(), "b".to_string()],
            JsonOps,
            json!(["a", "b"])
        );
        assert_encode_success!(vec!["one".to_string()], JsonOps, json!(["one"]));
        assert_encode_success!(
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            JsonOps,
            json!(["1", "2", "3"])
        );

        assert_encode_success!(vec![1, 2], JsonOps, json!([1, 2]));

        assert_encode_success!(vec![true, false], JsonOps, json!([true, false]));
        assert_encode_success!(
            vec![vec![true, false], vec![true, false]],
            JsonOps,
            json!([[true, false], [true, false]])
        );
        assert_encode_success!(
            vec![vec![vec![true, true], vec![false, false]]],
            JsonOps,
            json!([[[true, true], [false, false]]])
        );
    }

    #[test]
    fn decoding() {
        type NumberGrid = Vec<Vec<f64>>;

        assert_decode!(Vec<i16>, json!([1, 2, 3]), JsonOps, is_success);
        assert_decode!(Vec<i16>, json!([1, 2, 6, 24, 120]), JsonOps, is_success);
        assert_decode!(Vec<i16>, json!(["string", "b"]), JsonOps, is_error);
        assert_decode!(Vec<i16>, json!(false), JsonOps, is_error);

        assert_decode!(NumberGrid, json!([[0, 0.5, 1.0]]), JsonOps, is_success);
        assert_decode!(
            NumberGrid,
            json!([[0, 0.5, 1.0], [1, 4, 5], [-293.4, 1, 293]]),
            JsonOps,
            is_success
        );
        assert_decode!(
            NumberGrid,
            json!([[0, 0.5, 1.0], [1, false, 5], [-293.4, 1, 293]]),
            JsonOps,
            is_error
        );
        assert_decode!(
            NumberGrid,
            json!([[1, 1.5, 2.0], [-20]]),
            JsonOps,
            is_success
        );
        assert_decode!(NumberGrid, json!([[]]), JsonOps, is_success);
        assert_decode!(NumberGrid, json!([[[[]]]]), JsonOps, is_error);
    }
}
