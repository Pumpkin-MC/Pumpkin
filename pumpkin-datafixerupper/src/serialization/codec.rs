use crate::serialization::HasValue;
use crate::serialization::codecs::lazy::LazyCodec;
use crate::serialization::codecs::list::ListCodec;
use crate::serialization::codecs::map_codec::MapCodecCodec;
use crate::serialization::codecs::primitive::{
    BoolCodec, ByteBufferCodec, ByteCodec, DoubleCodec, FloatCodec, IntCodec, IntStreamCodec,
    LongCodec, LongStreamCodec, ShortCodec, StringCodec,
};
use crate::serialization::codecs::range::RangeCodec;
use crate::serialization::codecs::range::new_range_codec;
use crate::serialization::coders::{
    ComappedEncoderImpl, Decoder, Encoder, FlatComappedEncoderImpl, FlatMappedDecoderImpl,
    MappedDecoderImpl, comap, decoder_field_of, encoder_field_of, flat_comap, flat_map, map,
};
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::keyable::Keyable;
use crate::serialization::map_codec::{ComposedMapCodec, MapCodec};
use crate::serialization::map_codecs::field_coders::{FieldDecoder, FieldEncoder};
use crate::serialization::map_codecs::simple::SimpleMapCodec;
use crate::serialization::struct_codec_builder::Field;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::{LazyLock, OnceLock};

/// A type of *codec* describing the way to **encode from and decode into** something of a type `Value`  (`Value` -> `?` and `?` -> `Value`).
pub trait Codec: Encoder + Decoder {}

// Any struct implementing Encoder<Value = A> and Decoder<Value = A> will also implement Codec<Value = A>.
impl<T> Codec for T where T: Encoder + Decoder {}

/// A codec allowing an arbitrary encoder and decoder.
pub struct ComposedCodec<E: Encoder + 'static, D: Decoder<Value = E::Value> + 'static> {
    encoder: E,
    decoder: D,
}

impl<E: Encoder, D: Decoder<Value = E::Value>> HasValue for ComposedCodec<E, D> {
    type Value = E::Value;
}

impl<E: Encoder, D: Decoder<Value = E::Value>> Encoder for ComposedCodec<E, D> {
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        self.encoder.encode(input, ops, prefix)
    }
}

impl<E: Encoder, D: Decoder<Value = E::Value>> Decoder for ComposedCodec<E, D> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.decoder.decode(input, ops)
    }
}

// Primitive codecs

macro_rules! define_const_codec {
    ($name:ident, $codec_ty:ident, $ty:ident, $java_ty:ident) => {
        #[doc = concat!("A primitive codec for Java's `", stringify!($java_ty), "` (`", stringify!($ty), "` in Rust).")]
        pub const $name: $codec_ty = $codec_ty;
    };
    (vec $name:ident, $codec_ty:ident, $vec_ty:ident, $java_ty:ident) => {
        #[doc = concat!("A primitive codec for Java's `", stringify!($java_ty), "`.")]
        #[doc = concat!("Here, this actually stores a [`Vec<", stringify!($vec_ty), ">`].")]
        pub const $name: $codec_ty = $codec_ty;
    };
}

define_const_codec!(BOOL_CODEC, BoolCodec, bool, boolean);

define_const_codec!(BYTE_CODEC, ByteCodec, i8, byte);
define_const_codec!(SHORT_CODEC, ShortCodec, i16, short);
define_const_codec!(INT_CODEC, IntCodec, i32, int);
define_const_codec!(LONG_CODEC, LongCodec, i64, long);
define_const_codec!(FLOAT_CODEC, FloatCodec, f32, float);
define_const_codec!(DOUBLE_CODEC, DoubleCodec, f64, double);

define_const_codec!(STRING_CODEC, StringCodec, String, String);

define_const_codec!(vec BYTE_BUFFER_CODEC, ByteBufferCodec, i8, ByteBuffer);
define_const_codec!(vec INT_STREAM_CODEC, IntStreamCodec, i16, IntStream);
define_const_codec!(vec LONG_STREAM_CODEC, LongStreamCodec, i32, LongStream);

// Modifier methods

/// Creates a [`LazyCodec`] with a *function pointer* that returns a new [`Codec`], which will be called on first use.
pub const fn lazy<C: Codec>(f: fn() -> C) -> LazyCodec<C> {
    LazyCodec {
        codec: LazyLock::new(f),
    }
}

/// Creates a [`ListCodec`] of another [`Codec`]with the provided minimum and maximum size.
pub const fn list<C: Codec>(codec: &'static C, min_size: usize, max_size: usize) -> ListCodec<C> {
    ListCodec {
        element_codec: codec,
        min_size,
        max_size,
    }
}

/// Creates a [`ListCodec`] of another [`Codec`] with the provided maximum size.
pub const fn limited_list<C: Codec>(codec: &'static C, max_size: usize) -> ListCodec<C> {
    ListCodec {
        element_codec: codec,
        min_size: 0,
        max_size,
    }
}

/// Creates a [`ListCodec`] of another [`Codec`], which allows any size.
pub const fn unbounded_list<C: Codec>(codec: &'static C) -> ListCodec<C> {
    ListCodec {
        element_codec: codec,
        min_size: 0,
        max_size: usize::MAX,
    }
}

/// Helper macro to generate the shorthand types and functions of the transformer [`Codec`] methods.
macro_rules! make_codec_transformation_function {
    ($name:ident, $short_type:ident, $encoder_type:ident, $decoder_type:ident, $encoder_func:ident, $decoder_func:ident, $to_func_result:ty, $from_func_result:ty, $a_equivalency:literal, $s_equivalency:literal) => {
        pub type $short_type<S, C> = ComposedCodec<$encoder_type<<C as HasValue>::Value, S, C>, $decoder_type<<C as HasValue>::Value, S, C>>;

        #[doc = "Transforms a [`Codec`] of type `A` to another [`Codec`] of type `S`. Use this if:"]
        #[doc = concat!("- `A` is **", $a_equivalency, "** to `S`.")]
        #[doc = concat!("- `S` is **", $s_equivalency, "** to `A`.")]
        #[doc = ""]
        #[doc = "A type `A` is *equivalent* to `B` if *A can always successfully be converted to B*."]
        pub const fn $name<A, C: Codec<Value = A>, S>(codec: &'static C, to: fn(&A) -> $to_func_result, from: fn(&S) -> $from_func_result) -> $short_type<S, C> {
            ComposedCodec {
                encoder: $encoder_func(codec, from),
                decoder: $decoder_func(codec, to)
            }
        }
    };
}

// Transformer functions

make_codec_transformation_function!(
    xmap,
    XmapCodec,
    ComappedEncoderImpl,
    MappedDecoderImpl,
    comap,
    map,
    S,
    A,
    "equivalent",
    "equivalent"
);

make_codec_transformation_function!(
    comap_flat_map,
    ComapFlatMapCodec,
    ComappedEncoderImpl,
    FlatMappedDecoderImpl,
    comap,
    flat_map,
    DataResult<S>,
    A,
    "equivalent",
    "partially equivalent"
);

make_codec_transformation_function!(
    flat_map_comap,
    FlatMapComapCodec,
    FlatComappedEncoderImpl,
    MappedDecoderImpl,
    flat_comap,
    map,
    S,
    DataResult<A>,
    "partially equivalent",
    "equivalent"
);

make_codec_transformation_function!(
    flat_xmap,
    FlatXmapCodec,
    FlatComappedEncoderImpl,
    FlatMappedDecoderImpl,
    flat_comap,
    flat_map,
    DataResult<S>,
    DataResult<A>,
    "partially equivalent",
    "partially equivalent"
);

// Range codec functions

macro_rules! make_codec_range_function {
    ($func_name:ident, $shorthand_name:ident, $ty:ty, $codec:ident, $singleton_codec:ident, $java_type:ident) => {
        type $shorthand_name = RangeCodec<$codec>;

        #[doc = concat!("Returns a version of [`", stringify!($singleton_codec), "`] for `", stringify!($ty), "`s (or `", stringify!($java_type), "`s in Java) constrained to a minimum *(inclusive)* and maximum *(inclusive)* value.")]
        pub const fn $func_name(min: $ty, max: $ty) -> $shorthand_name {
            new_range_codec(&$singleton_codec, min, max)
        }
    };
}

make_codec_range_function!(int_range, IntRangeCodec, i32, IntCodec, INT_CODEC, int);
make_codec_range_function!(
    float_range,
    FloatRangeCodec,
    f32,
    FloatCodec,
    FLOAT_CODEC,
    float
);
make_codec_range_function!(
    double_range,
    DoubleRangeCodec,
    f64,
    DoubleCodec,
    DOUBLE_CODEC,
    double
);

// Map codec functions

/// Converts a [`MapCodec`] to a [`Codec`].
pub const fn from_map<A, C: MapCodec<Value = A>>(map_codec: C) -> MapCodecCodec<A, C> {
    MapCodecCodec { codec: map_codec }
}

/// Creates a [`SimpleMapCodec`] with the provided key codec, value (element) codec and the possible key values.
pub const fn simple_map<KC: Codec, VC: Codec>(
    key_codec: &'static KC,
    element_codec: &'static VC,
    keyable: Box<dyn Keyable>,
) -> SimpleMapCodec<KC, VC>
where
    <KC as HasValue>::Value: Display + Eq + Hash,
{
    SimpleMapCodec {
        key_codec,
        element_codec,
        keyable,
        compressor: OnceLock::new(),
    }
}

/// Creates an [`UnboundedMapCodec`] with the provided key codec, value (element) codec and the possible key values.
pub const fn unbounded_map<KC: Codec, VC: Codec>(
    key_codec: &'static KC,
    element_codec: &'static VC,
    keyable: Box<dyn Keyable>,
) -> SimpleMapCodec<KC, VC>
where
    <KC as HasValue>::Value: Display + Eq + Hash,
{
    SimpleMapCodec {
        key_codec,
        element_codec,
        keyable,
        compressor: OnceLock::new(),
    }
}

// Struct codec functions

/// Returns a [`Field`], which knows how to get a part of a struct to serialize.
pub const fn for_getter<T, C: MapCodec>(map_codec: C, getter: fn(&T) -> &C::Value) -> Field<T, C> {
    Field { map_codec, getter }
}

/// Creates a structure [`Codec`]. This macro supports up to *16* fields.
#[macro_export]
macro_rules! struct_codec {
    ($f1:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_1($f1, $f)
    };
    ($f1:expr, $f2:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_2($f1, $f2, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_3($f1, $f2, $f3, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_4($f1, $f2, $f3, $f4, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_5($f1, $f2, $f3, $f4, $f5, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_6($f1, $f2, $f3, $f4, $f5, $f6, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_7($f1, $f2, $f3, $f4, $f5, $f6, $f7, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_8(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_9(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_10(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f11:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_11(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f11, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f11:expr, $f12:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_12(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f11, $f12, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f11:expr, $f12:expr, $f13:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_13(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f11, $f12, $f13, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f11:expr, $f12:expr, $f13:expr, $f14:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_14(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f11, $f12, $f13, $f14, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f11:expr, $f12:expr, $f13:expr, $f14:expr, $f15:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_15(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f11, $f12, $f13, $f14, $f15, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f11:expr, $f12:expr, $f13:expr, $f14:expr, $f15:expr, $f16:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codec_builder::struct_16(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f11, $f12, $f13, $f14, $f15, $f16,
            $f,
        )
    };
}

// Field functions

pub type FieldMapCodec<C> = ComposedMapCodec<
    <C as HasValue>::Value,
    FieldEncoder<<C as HasValue>::Value, C>,
    FieldDecoder<<C as HasValue>::Value, C>,
>;

/// Creates a [`MapCodec`] for a field which relies on the provided [`Codec`] for serialization/deserialization.
pub const fn field_of<C: Codec>(name: &'static str, codec: &'static C) -> FieldMapCodec<C> {
    ComposedMapCodec {
        encoder: encoder_field_of(name, codec),
        decoder: decoder_field_of(name, codec),
        compressor: OnceLock::new(),
    }
}

// Assertion functions

/// Asserts that the decoding of some value by a [`DynamicOps`] via a [`Codec`] is a success/error.
/// # Example
/// ```
/// assert_decode!(codec::INT_CODEC, json!(2), json_ops::INSTANCE, is_success);
/// assert_decode!(codec::STRING_CODEC, json!("hello"), json_ops::INSTANCE, is_success);
/// assert_decode!(codec::FLOAT_CODEC, json!(true), json_ops::INSTANCE, is_error);
/// ```
#[macro_export]
macro_rules! assert_decode {
    ($codec:expr, $value:expr, $ops:expr, $assertion:ident) => {{
        assert!($codec.decode($value, &$ops).$assertion());
    }};
}
