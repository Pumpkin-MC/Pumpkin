use crate::codec::primitive::sealed::Primitive;
use crate::{DataResult, Decode, DynamicOps, Encode};

mod sealed {
    use super::{DataResult, DynamicOps};

    /// Sealed trait to easily implement `Encode` and `Decode` for
    /// primitive DFU types.
    pub trait Primitive: Sized {
        fn primitive_encode<O: DynamicOps>(&self, ops: &'static O) -> O::Value;
        fn primitive_decode<O: DynamicOps>(ops: &'static O, input: O::Value) -> DataResult<Self>;
    }
}

impl<T: Primitive> Encode for T {
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        ops.merge_into_primitive(prefix, self.primitive_encode(ops))
    }
}

impl<T: Primitive> Decode for T {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        T::primitive_decode(ops, input).map(|r| (r, ops.empty()))
    }
}

macro_rules! impl_number {
    ($ty:ty, $create_func:ident) => {
        impl Primitive for $ty {
            fn primitive_encode<O: DynamicOps>(&self, ops: &'static O) -> O::Value {
                ops.$create_func(*self)
            }
            fn primitive_decode<O: DynamicOps>(
                ops: &'static O,
                input: O::Value,
            ) -> DataResult<Self> {
                ops.get_number(&input).map(|n| <$ty>::from(n))
            }
        }
    };
}

macro_rules! impl_number_and_unsigned {
    ($ty:ty, $uty:ty, $create_func:ident) => {
        impl_number!($ty, $create_func);

        // Unsigned type
        impl Encode for $uty {
            fn encode<O: DynamicOps>(
                &self,
                ops: &'static O,
                prefix: O::Value,
            ) -> DataResult<O::Value> {
                <$ty>::try_from(*self).map_or_else(
                    |_| {
                        DataResult::new_error(concat!(
                            "Could not fit ",
                            stringify!($uty),
                            " into ",
                            stringify!($ty)
                        ))
                    },
                    |i| i.encode(ops, prefix),
                )
            }
        }
        impl Decode for $uty {
            fn decode<O: DynamicOps>(
                input: O::Value,
                ops: &'static O,
            ) -> DataResult<(Self, O::Value)> {
                <$ty>::parse(input, ops).flat_map(|i| {
                    <$uty>::try_from(i).map_or_else(
                        |_| {
                            DataResult::new_error(concat!(
                                "Could not fit ",
                                stringify!($ty),
                                " into ",
                                stringify!($uty)
                            ))
                        },
                        |u| DataResult::new_success((u, ops.empty())),
                    )
                })
            }
        }
    };
}

impl_number_and_unsigned!(i8, u8, create_byte);
impl_number_and_unsigned!(i16, u16, create_short);
impl_number_and_unsigned!(i32, u32, create_int);
impl_number_and_unsigned!(i64, u64, create_long);

impl_number!(f32, create_float);
impl_number!(f64, create_double);

impl Primitive for bool {
    fn primitive_encode<O: DynamicOps>(&self, ops: &'static O) -> O::Value {
        ops.create_bool(*self)
    }

    fn primitive_decode<O: DynamicOps>(ops: &'static O, input: O::Value) -> DataResult<Self> {
        ops.get_bool(&input)
    }
}

impl Primitive for String {
    fn primitive_encode<O: DynamicOps>(&self, ops: &'static O) -> O::Value {
        ops.create_string(self.as_str())
    }

    fn primitive_decode<O: DynamicOps>(ops: &'static O, input: O::Value) -> DataResult<Self> {
        ops.get_string(&input)
    }
}

macro_rules! stream_struct {
    ($stream:ident, $ty:ty, $create_func:ident, $get_func:ident) => {
        #[doc = concat!("A [`Vec<", stringify!($ty), ">`] wrapper that has built-in DFU support for encoding and decoding.")]
        #[derive(Debug, Clone)]
        pub struct $stream(pub Vec<$ty>);

        impl From<Vec<$ty>> for $stream {
            fn from(value: Vec<$ty>) -> Self {
                Self(value)
            }
        }

        impl From<$stream> for Vec<$ty> {
            fn from(value: $stream) -> Vec<$ty> {
                value.0
            }
        }

        impl Primitive for $stream {
            fn primitive_encode<O: DynamicOps>(&self, ops: &'static O) -> O::Value {
                ops.$create_func(self.0.clone())
            }

            fn primitive_decode<O: DynamicOps>(ops: &'static O, input: O::Value) -> DataResult<Self> {
                ops.$get_func(input).map(From::from)
            }
        }
    };
}

stream_struct!(ByteBuffer, i8, create_byte_list, get_byte_list);
stream_struct!(IntStream, i32, create_int_list, get_int_list);
stream_struct!(LongStream, i64, create_long_list, get_long_list);

#[cfg(test)]
mod test {
    use crate::json_ops::JsonOps;
    use crate::{ByteBuffer, IntStream, LongStream, assert_decode, assert_encode_success};
    use serde_json::json;

    #[test]
    fn encoding() {
        assert_encode_success!(3, JsonOps, json!(3));
        assert_encode_success!(-68i8, JsonOps, json!(-68));
        assert_encode_success!(-913813743, JsonOps, json!(-913813743));
        assert_encode_success!("Hello, world!".to_string(), JsonOps, json!("Hello, world!"));
        assert_encode_success!(String::new(), JsonOps, json!(""));
        assert_encode_success!(ByteBuffer::from(vec![1, 2, 3]), JsonOps, json!([1, 2, 3]));
        assert_encode_success!(
            IntStream::from(vec![3, 6, 9, 11, 15]),
            JsonOps,
            json!([3, 6, 9, 11, 15])
        );
        assert_encode_success!(
            LongStream::from(vec![4, 6, 9, 12]),
            JsonOps,
            json!([4, 6, 9, 12])
        );

        assert_encode_success!(3u8, JsonOps, json!(3));
        assert_encode_success!(923482312u64, JsonOps, json!(923482312));
    }

    #[test]
    fn decoding() {
        assert_decode!(i32, json!(-2), JsonOps, is_success);
        assert_decode!(bool, json!("hello"), JsonOps, is_error);
        assert_decode!(bool, json!(0), JsonOps, is_error);

        assert_decode!(IntStream, json!([1, 2, 3]), JsonOps, is_success);
        assert_decode!(LongStream, json!([]), JsonOps, is_success);
        assert_decode!(ByteBuffer, json!(["not a number"]), JsonOps, is_error);

        assert_decode!(String, json!("cool"), JsonOps, is_success);
        assert_decode!(String, json!(1), JsonOps, is_error);

        assert_decode!(u32, json!(-45), JsonOps, is_error);
        assert_decode!(u64, json!(-132541235), JsonOps, is_error);
        assert_decode!(u64, json!(132541235), JsonOps, is_success);
    }
}
