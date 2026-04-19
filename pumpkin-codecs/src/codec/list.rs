use crate::list_builder::ListBuilder;
use crate::{DataResult, Decode, DynamicOps, Encode, Lifecycle};

/// A wrapped [`Vec`] that can only contain a size of elements between `MIN` and `MAX` (inclusive).
pub struct BoundedVec<T, const MIN: usize, const MAX: usize>(Vec<T>);

impl<T, const MIN: usize, const MAX: usize> From<BoundedVec<T, MIN, MAX>> for Vec<T> {
    fn from(value: BoundedVec<T, MIN, MAX>) -> Self {
        value.0
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
