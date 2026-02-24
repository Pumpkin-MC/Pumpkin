use crate::HasValue;
use crate::codec::Codec;
use crate::coders::{Decoder, Encoder};
use crate::data_result::DataResult;
use crate::dynamic_ops::DynamicOps;
use crate::lifecycle::Lifecycle;
use crate::list_builder::ListBuilder;
use std::fmt::{Debug, Display};

/// A list codec type. For a type `A`, this codec serializes/deserializes a [`Vec<A>`].
/// `C` is the codec used for each element of this list.
///
/// A `ListCodec` can also specify a minimum and maximum number of elements to allow in the list.
#[derive(Debug)]
pub struct ListCodec<C>
where
    C: Codec + ?Sized + 'static,
{
    element_codec: &'static C,
    min_size: usize,
    max_size: usize,
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
    fn encode<T: Display + PartialEq + Clone>(
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
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        let iter = ops.get_iter(input).with_lifecycle(Lifecycle::Stable);
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

/// Creates a new [`ListCodec`].
pub(crate) const fn new_list_codec<C: Codec>(
    codec: &'static C,
    min_size: usize,
    max_size: usize,
) -> ListCodec<C> {
    ListCodec {
        element_codec: codec,
        min_size,
        max_size,
    }
}

#[cfg(test)]
mod test {
    use crate::codec::*;
    use crate::codecs::list::ListCodec;
    use crate::codecs::primitive::{BoolCodec, DoubleCodec, IntCodec, ShortCodec, StringCodec};
    use crate::coders::Decoder;
    use crate::coders::Encoder;
    use crate::json_ops;
    use crate::{assert_decode, assert_success};
    use serde_json::json;

    #[test]
    fn encoding() {
        {
            pub static INT_LIST_CODEC: ListCodec<IntCodec> = list(&INT_CODEC, 1, 3);

            assert_success!(
                INT_LIST_CODEC.encode_start(&vec![1, 2], &json_ops::INSTANCE),
                json!([1, 2])
            );
            assert!(
                INT_LIST_CODEC
                    .encode_start(&vec![], &json_ops::INSTANCE)
                    .is_error()
            );
            assert!(
                INT_LIST_CODEC
                    .encode_start(&vec![50, 52, 54, 56], &json_ops::INSTANCE)
                    .is_error()
            );
        };

        {
            pub static STRING_LIST_CODEC: ListCodec<StringCodec> = limited_list(&STRING_CODEC, 2);

            assert_success!(
                STRING_LIST_CODEC
                    .encode_start(&vec!["a".to_string(), "b".to_string()], &json_ops::INSTANCE),
                json!(["a", "b"])
            );
            assert_success!(
                STRING_LIST_CODEC.encode_start(&vec!["one".to_string()], &json_ops::INSTANCE),
                json!(["one"])
            );
            assert!(
                STRING_LIST_CODEC
                    .encode_start(
                        &vec!["1".to_string(), "2".to_string(), "3".to_string()],
                        &json_ops::INSTANCE
                    )
                    .is_error()
            );
        };

        {
            // The inner lists have a max size of 2, while the main list has a max size of 3.
            pub static BOOL_LIST_LIST_CODEC: ListCodec<ListCodec<BoolCodec>> =
                limited_list(&limited_list(&BOOL_CODEC, 2), 3);

            assert_success!(
                BOOL_LIST_LIST_CODEC.encode_start(&vec![vec![true, true]], &json_ops::INSTANCE),
                json!([[true, true]])
            );
            assert_success!(
                BOOL_LIST_LIST_CODEC
                    .encode_start(&vec![vec![], vec![false, true]], &json_ops::INSTANCE),
                json!([[], [false, true]])
            );
            assert!(
                BOOL_LIST_LIST_CODEC
                    .encode_start(&vec![vec![true, false, true, false]], &json_ops::INSTANCE)
                    .is_error()
            );
        };
    }

    #[test]
    fn decoding() {
        {
            pub static SHORT_LIST_CODEC: ListCodec<ShortCodec> = list(&SHORT_CODEC, 2, 4);

            assert_decode!(
                SHORT_LIST_CODEC,
                json!([1, 2]),
                &json_ops::INSTANCE,
                is_success
            );
            assert_decode!(
                SHORT_LIST_CODEC,
                json!([1, 2, 6, 24]),
                &json_ops::INSTANCE,
                is_success
            );
            assert_decode!(
                SHORT_LIST_CODEC,
                json!([1, 2, 6, 24, 120]),
                &json_ops::INSTANCE,
                is_error
            );
            assert_decode!(
                SHORT_LIST_CODEC,
                json!([-45, 252, 1000]),
                &json_ops::INSTANCE,
                is_success
            );
            assert_decode!(
                SHORT_LIST_CODEC,
                json!(["string", "b"]),
                &json_ops::INSTANCE,
                is_error
            );
            assert_decode!(
                SHORT_LIST_CODEC,
                json!(["1", "2"]),
                &json_ops::INSTANCE,
                is_error
            );
        };

        {
            // The inner lists have a size of 3, while the main list has a max size of 2.
            pub static POS_LIST_CODEC: ListCodec<ListCodec<DoubleCodec>> =
                limited_list(&list(&DOUBLE_CODEC, 3, 3), 2);

            assert_decode!(
                POS_LIST_CODEC,
                json!([[0, 0.5, 1.0]]),
                &json_ops::INSTANCE,
                is_success
            );
            assert_decode!(
                POS_LIST_CODEC,
                json!([0, 0.5, 1.0]),
                &json_ops::INSTANCE,
                is_error
            );
            assert_decode!(
                POS_LIST_CODEC,
                json!([[3.56, 123.4, -0.144], [12.34, 56.78]]),
                &json_ops::INSTANCE,
                is_error
            );
            assert_decode!(POS_LIST_CODEC, json!([]), &json_ops::INSTANCE, is_success);
        }
    }
}
