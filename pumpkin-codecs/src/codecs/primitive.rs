use crate::HasValue;
use crate::codec::Codec;
use crate::coders::{Decoder, Encoder};
use crate::data_result::DataResult;
use crate::dynamic_ops::DynamicOps;

// DFU types

/// Helper macro to generate the struct and [`HasValue`] trait implementation for a `PrimitiveCodec` struct.
macro_rules! impl_primitive_codec_start {
    ($name:ident, $prim:ty) => {
        /// A primitive [`Codec`] for the
        #[doc = concat!("[`", stringify!($prim), "`]")]
        /// data type.
        pub struct $name;

        impl HasValue for $name {
            type Value = $prim;
        }
    };
}

/// Helper macro to generate an entire implementation for a number `PrimitiveCodec`.
macro_rules! impl_primitive_number_codec {
    ($name:ident, $prim:ty, $create_func:ident) => {
        impl_primitive_codec_start!($name, $prim);
        impl PrimitiveCodec for $name {
            fn read<T>(
                &self,
                ops: &'static impl DynamicOps<Value = T>,
                input: T,
            ) -> DataResult<$prim> {
                ops.get_number(&input).map(|n| <$prim>::from(n))
            }

            fn write<T>(&self, ops: &'static impl DynamicOps<Value = T>, value: &$prim) -> T {
                ops.$create_func(*value)
            }
        }
    };
}

/// Helper macro to generate an entire implementation for a list `PrimitiveCodec`.
macro_rules! impl_primitive_list_codec {
    ($name:ident, $elem:ty, $get_func:ident, $create_func:ident) => {
        impl_primitive_codec_start!($name, Vec<$elem>);
        impl PrimitiveCodec for $name {
            fn read<T>(
                &self,
                ops: &'static impl DynamicOps<Value = T>,
                input: T,
            ) -> DataResult<Vec<$elem>> {
                ops.$get_func(input)
            }

            fn write<T>(&self, ops: &'static impl DynamicOps<Value = T>, value: &Vec<$elem>) -> T {
                ops.$create_func(value.to_vec())
            }
        }
    };
}

/// A generic primitive codec.
trait PrimitiveCodec: Codec {
    fn read<T>(
        &self,
        ops: &'static impl DynamicOps<Value = T>,
        input: T,
    ) -> DataResult<Self::Value>;

    fn write<T>(&self, ops: &'static impl DynamicOps<Value = T>, value: &Self::Value) -> T;
}

impl<C: PrimitiveCodec> Encoder for C {
    fn encode<T: PartialEq>(
        &self,
        input: &<C as HasValue>::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        ops.merge_into_primitive(prefix, self.write(ops, input))
    }
}

impl<C: PrimitiveCodec> Decoder for C {
    fn decode<T: PartialEq>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(<C as HasValue>::Value, T)> {
        self.read(ops, input).map(|r| (r, ops.empty()))
    }
}

// Implementations

impl_primitive_codec_start!(BoolCodec, bool);
impl PrimitiveCodec for BoolCodec {
    fn read<T>(&self, ops: &'static impl DynamicOps<Value = T>, input: T) -> DataResult<bool> {
        ops.get_bool(&input)
    }

    fn write<T>(&self, ops: &'static impl DynamicOps<Value = T>, value: &bool) -> T {
        ops.create_bool(*value)
    }
}

impl_primitive_number_codec!(ByteCodec, i8, create_byte);
impl_primitive_number_codec!(ShortCodec, i16, create_short);
impl_primitive_number_codec!(IntCodec, i32, create_int);
impl_primitive_number_codec!(LongCodec, i64, create_long);
impl_primitive_number_codec!(FloatCodec, f32, create_float);
impl_primitive_number_codec!(DoubleCodec, f64, create_double);

impl_primitive_codec_start!(StringCodec, String);
impl PrimitiveCodec for StringCodec {
    fn read<T>(&self, ops: &'static impl DynamicOps<Value = T>, input: T) -> DataResult<String> {
        ops.get_string(&input)
    }

    fn write<T>(&self, ops: &'static impl DynamicOps<Value = T>, value: &String) -> T {
        ops.create_string(value)
    }
}

impl_primitive_codec_start!(ByteBufferCodec, Box<[u8]>);
impl PrimitiveCodec for ByteBufferCodec {
    fn read<T>(&self, ops: &'static impl DynamicOps<Value = T>, input: T) -> DataResult<Box<[u8]>> {
        ops.get_byte_buffer(input)
    }

    fn write<T>(&self, ops: &'static impl DynamicOps<Value = T>, value: &Box<[u8]>) -> T {
        ops.create_byte_buffer(value.to_vec())
    }
}

impl_primitive_list_codec!(IntStreamCodec, i32, get_int_list, create_int_list);
impl_primitive_list_codec!(LongStreamCodec, i64, get_long_list, create_long_list);

#[cfg(test)]
mod test {
    use crate::codec::*;
    use crate::coders::*;
    use crate::json_ops;
    use crate::{assert_decode, assert_success};
    use serde_json::json;

    #[test]
    fn encoding() {
        assert_success!(INT_CODEC.encode_start(&3, &json_ops::INSTANCE), json!(3));
        assert_success!(
            BYTE_CODEC.encode_start(&-68i8, &json_ops::INSTANCE),
            json!(-68)
        );
        assert_success!(
            LONG_CODEC.encode_start(&-913813743, &json_ops::INSTANCE),
            json!(-913813743)
        );

        assert_success!(
            STRING_CODEC.encode_start(&"Hello, world!".to_string(), &json_ops::INSTANCE),
            json!("Hello, world!")
        );
        assert_success!(
            STRING_CODEC.encode_start(&String::new(), &json_ops::INSTANCE),
            json!("")
        );

        assert_success!(
            BYTE_BUFFER_CODEC.encode_start(&Box::from([1u8, 2u8, 3u8]), &json_ops::INSTANCE),
            json!([1, 2, 3])
        );
        assert_success!(
            LONG_STREAM_CODEC.encode_start(&vec![4, 6, 9, 12], &json_ops::INSTANCE),
            json!([4, 6, 9, 12])
        );
    }

    #[test]
    fn decoding() {
        assert_decode!(INT_CODEC, json!(-2), &json_ops::INSTANCE, is_success);

        assert_decode!(SHORT_CODEC, json!("hello"), &json_ops::INSTANCE, is_error);
        assert_decode!(BOOL_CODEC, json!(0), &json_ops::INSTANCE, is_error);

        assert_decode!(
            INT_STREAM_CODEC,
            json!([1, 2, 3]),
            &json_ops::INSTANCE,
            is_success
        );
        assert_decode!(
            LONG_STREAM_CODEC,
            json!([]),
            &json_ops::INSTANCE,
            is_success
        );
        assert_decode!(
            BYTE_BUFFER_CODEC,
            json!(["not a number"]),
            &json_ops::INSTANCE,
            is_error
        );

        assert_decode!(STRING_CODEC, json!("cool"), &json_ops::INSTANCE, is_success);
        assert_decode!(STRING_CODEC, json!(1), &json_ops::INSTANCE, is_error);
    }
}
