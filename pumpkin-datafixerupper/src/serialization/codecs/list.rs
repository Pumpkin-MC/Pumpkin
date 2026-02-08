use crate::serialization::{
    HasValue,
    codec::Codec,
    coders::{Decoder, Encoder},
    data_result::DataResult,
    dynamic_ops::DynamicOps,
    lifecycle::Lifecycle,
    list_builder::ListBuilder,
};

/// A list codec type. For a type `A`, this codec serializes/deserializes a [`Vec<A>`].
/// - `C` is the codec used for each element of this list.
/// - Also, `MIN` specifies the minimum number of element this codec has (inclusive), while
/// `MIN` specifies the maximum number of element this codec has (inclusive).
#[derive(Debug)]
pub struct ListCodec<C, const MIN: usize, const MAX: usize>
where
    C: Codec,
{
    element_codec: &'static C,
}

impl<C: Codec, const MIN: usize, const MAX: usize> ListCodec<C, MIN, MAX> {
    fn create_too_short_error<T>(size: usize) -> DataResult<T> {
        DataResult::error(format!(
            "List is too short: {size}, expected range [{MIN}-{MAX}]"
        ))
    }

    fn create_too_long_error<T>(size: usize) -> DataResult<T> {
        DataResult::error(format!(
            "List is too short: {size}, expected range [{MIN}-{MAX}]"
        ))
    }
}

impl<C: Codec, const MIN: usize, const MAX: usize> HasValue for ListCodec<C, MIN, MAX> {
    type Value = Vec<C::Value>;
}

impl<C: Codec, const MIN: usize, const MAX: usize> Encoder for ListCodec<C, MIN, MAX> {
    fn encode<T: PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        let size = input.len();
        if size < MIN {
            ListCodec::<C, MIN, MAX>::create_too_short_error(size)
        } else if size > MAX {
            ListCodec::<C, MIN, MAX>::create_too_long_error(size)
        } else {
            let mut builder = ops.list_builder();
            for e in input {
                builder = builder.add_data_result(self.element_codec.encode_start(e, ops));
            }
            builder.build(prefix)
        }
    }
}

impl<C, const MIN: usize, const MAX: usize> Decoder for ListCodec<C, MIN, MAX>
where
    C: Codec,
{
    fn decode<T: PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        let mut iter = ops.get_iter(&input);
        iter.set_lifecycle(Lifecycle::Stable);
        iter.flat_map(|i| {
            let mut total_count = 0;
            let elements: Self::Value = vec![];
            let mut failed: Vec<T> = vec![];
            let mut result = DataResult::success(());

            for element in i {
                total_count += 1;
                if elements.len() > MAX {
                    return ListCodec::<C, MIN, MAX>::create_too_long_error(elements.len());
                }
                let element_result = self.element_codec.decode(element.clone(), ops);
                if element_result.is_error() {
                    failed.push(element.clone());
                }
                result = result.apply_2_and_make_stable(|r, _| r, element_result)
            }

            if elements.len() < MIN {
                return ListCodec::<C, MIN, MAX>::create_too_short_error(elements.len());
            }

            let pair = (elements, ops.create_list(failed));
            if total_count > MAX {
                result = ListCodec::<C, MIN, MAX>::create_too_long_error(total_count);
            }
            result.with_complete_or_partial(pair)
        })
    }
}
