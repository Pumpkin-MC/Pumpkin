use crate::serialization::{
    data_result::DataResult, decoder::Decoder, dynamic_ops::DynamicOps,
    encoder::Encoder,
};

/// Helper macro to generate the struct & encode function for a primitive codec.
macro_rules! impl_primitive_codec_common {
    ($name:ident, $prim:ty, $create_func:ident) => {
        struct $name;
        impl Encoder<$prim> for $name {
            fn encode<T: PartialEq>(
                &self,
                input: &$prim,
                ops: &impl DynamicOps<Value = T>,
                prefix: T,
            ) -> DataResult<T> {
                ops.merge_into_primitive(prefix, ops.$create_func(input.clone()))
            }
        }
    };
}

macro_rules! impl_primitive_codec {
    ($name:ident, $prim:ty, $create_func:ident, $get_func:ident) => {
        impl_primitive_codec_common!($name, $prim, $create_func);

        impl Decoder<$prim> for $name {
            fn decode<T>(&self, input: T, ops: &impl DynamicOps<Value = T>) -> DataResult<($prim, T)> {
                ops.$get_func(&input).map(|r| (r, ops.empty()))
            }
        }
    };
}

macro_rules! impl_primitive_number_codec {
    ($name:ident, $prim:ty, $create_func:ident) => {
        struct $name;
        impl Encoder<$prim> for $name {
            fn encode<T: PartialEq>(
                &self,
                input: &$prim,
                ops: &impl DynamicOps<Value = T>,
                prefix: T,
            ) -> DataResult<T> {
                ops.merge_into_primitive(prefix, ops.$create_func(*input))
            }
        }

        impl Decoder<$prim> for $name {
            fn decode<T>(&self, input: T, ops: &impl DynamicOps<Value = T>) -> DataResult<($prim, T)> {
                ops.get_number(&input).map(|n| <$prim>::from(n)).map(|r| (r, ops.empty()))
            }
        }
    };
}

impl_primitive_codec!(BoolCodec, bool, create_bool, get_bool);

impl_primitive_number_codec!(ByteCodec, i8, create_byte);
impl_primitive_number_codec!(ShortCodec, i16, create_short);
impl_primitive_number_codec!(IntCodec, i32, create_int);
impl_primitive_number_codec!(LongCodec, i64, create_long);
impl_primitive_number_codec!(FloatCodec, f32, create_float);
impl_primitive_number_codec!(DoubleCodec, f64, create_double);

impl_primitive_codec!(StringCodec, String, create_string, get_string);