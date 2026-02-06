use crate::serialization::{
    data_result::DataResult, decoder::Decoder, dynamic_ops::DynamicOps, encoder::Encoder,
};

/// Helper macro to generate the struct & encode function for a primitive codec.
macro_rules! impl_primitive_codec_common {
    ($name:ident, $prim:ty, $create_func:ident) => {
        pub struct $name;
        impl Encoder<$prim> for $name {
            fn encode<T: PartialEq>(
                &self,
                input: &$prim,
                ops: &impl DynamicOps<Value = T>,
                prefix: T,
            ) -> DataResult<T> {
                ops.merge_into_primitive(prefix, ops.$create_func(input))
            }
        }
    };
}

macro_rules! impl_primitive_codec {
    ($name:ident, $prim:ty, $create_func:ident, $get_func:ident) => {
        impl_primitive_codec_common!($name, $prim, $create_func);

        impl Decoder<$prim> for $name {
            fn decode<T>(
                &self,
                input: T,
                ops: &impl DynamicOps<Value = T>,
            ) -> DataResult<($prim, T)> {
                ops.$get_func(&input).map(|r| (r, ops.empty()))
            }
        }
    };
}

macro_rules! impl_primitive_number_codec {
    ($name:ident, $prim:ty, $create_func:ident) => {
        impl_primitive_codec_common!($name, $prim, $create_func);

        impl Decoder<$prim> for $name {
            fn decode<T>(
                &self,
                input: T,
                ops: &impl DynamicOps<Value = T>,
            ) -> DataResult<($prim, T)> {
                ops.get_number(&input)
                    .map(|n| <$prim>::from(n))
                    .map(|r| (r, ops.empty()))
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

impl_primitive_codec!(ByteBufferCodec, Vec<i8>, create_byte_buffer, get_bytes);

//impl_primitive_codec!(IntStreamCodec, Vec<i8>, create_bytes);
//impl_primitive_codec!(LongStreamCodec, Vec<i8>, create_bytes);

/// A primitive codec for Java's `boolean` (`bool` in Rust).
pub const BOOL: BoolCodec = BoolCodec;

/// A primitive codec for Java's `byte` (or `i8` in Rust).
pub const BYTE: ByteCodec = ByteCodec;
/// A primitive codec for Java's `short` (or `i16` in Rust).
pub const SHORT: ShortCodec = ShortCodec;
/// A primitive codec for Java's `int` (or `i32` in Rust).
pub const INT: IntCodec = IntCodec;
/// A primitive codec for Java's `long` (or `i64` in Rust).
pub const LONG: LongCodec = LongCodec;
/// A primitive codec for Java's `float` (or `f32` in Rust).
pub const FLOAT: FloatCodec = FloatCodec;
/// A primitive codec for Java's `double` (or `f64` in Rust).
pub const DOUBLE: DoubleCodec = DoubleCodec;

/// A primitive codec for Java's `String` (also `String` in Rust).
pub const STRING: StringCodec = StringCodec;

/// A primitive codec for Java's `String` (also `String` in Rust).
pub const BYTE_BUFFER: ByteBufferCodec = ByteBufferCodec;
