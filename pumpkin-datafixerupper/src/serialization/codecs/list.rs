use crate::serialization::{
    HasValue,
    codec::Codec,
    coders::{Decoder, Encoder},
    data_result::DataResult,
    dynamic_ops::DynamicOps,
    lifecycle::Lifecycle,
    list_builder::ListBuilder,
};
use std::fmt::Debug;

/// A list codec type. For a type `A`, this codec serializes/deserializes a [`Vec<A>`].
/// `C` is the codec used for each element of this list.
///
/// A `ListCodec` can also specify a minimum and maximum number of elements to allow in the list.
#[derive(Debug)]
pub struct ListCodec<C>
where
    C: Codec + ?Sized + 'static,
{
    pub(crate) element_codec: &'static C,
    pub(crate) min_size: usize,
    pub(crate) max_size: usize,
}

impl<C: Codec> ListCodec<C> {
    fn create_too_short_error<T>(&self, size: usize) -> DataResult<T> {
        DataResult::error(format!(
            "List is too short: {size}, expected range [{}-{}]",
            self.min_size, self.max_size
        ))
    }

    fn create_too_long_error<T>(&self, size: usize) -> DataResult<T> {
        DataResult::error(format!(
            "List is too long: {size}, expected range [{}-{}]",
            self.min_size, self.max_size
        ))
    }
}

impl<C: Codec> HasValue for ListCodec<C> {
    type Value = Vec<C::Value>;
}

impl<C: Codec> Encoder for ListCodec<C> {
    fn encode<T: PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        let size = input.len();
        if size < self.min_size {
            self.create_too_short_error(size)
        } else if size > self.max_size {
            self.create_too_long_error(size)
        } else {
            let mut builder = ops.list_builder();
            for e in input {
                builder = builder.add_data_result(self.element_codec.encode_start(e, ops));
            }
            builder.build(prefix)
        }
    }
}

impl<C> Decoder for ListCodec<C>
where
    C: Codec,
{
    fn decode<T: PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        let iter = ops.get_iter(&input).with_lifecycle(Lifecycle::Stable);
        iter.flat_map(|i| {
            let mut total_count = 0;
            let mut elements: Self::Value = vec![];
            let mut failed: Vec<T> = vec![];
            // This is used to keep track of the overall `DataResult`.
            // If any one element has a partial result, this turns into a partial result.
            // If any one element has no result, this turns into a non-result.
            let mut result = DataResult::success(());

            for element in i {
                total_count += 1;
                if elements.len() >= self.max_size {
                    failed.push(element.clone());
                    continue;
                }
                let element_result = self.element_codec.decode(element.clone(), ops);
                result = result.add_message(&element_result);
                if let Some(element) = element_result.into_result_or_partial() {
                    elements.push(element.0);
                }
            }

            if elements.len() < self.min_size {
                return self.create_too_short_error(elements.len());
            }

            let pair = (elements, ops.create_list(failed));
            if total_count > self.max_size {
                result = self.create_too_long_error(total_count);
            }
            result.with_complete_or_partial(pair)
        })
    }
}
