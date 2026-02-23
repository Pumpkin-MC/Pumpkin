use crate::impl_compressor;
use crate::HasValue;
use crate::data_result::DataResult;
use crate::dynamic_ops::DynamicOps;
use crate::key_compressor::KeyCompressor;
use crate::keyable::Keyable;
use crate::map_codec::MapCodec;
use crate::map_coders::{CompressorHolder, MapDecoder, MapEncoder};
use crate::map_like::MapLike;
use crate::struct_builder::StructBuilder;
use std::fmt::Display;
use std::marker::PhantomData;

/// The key used to hold the map containing specific fields to encode for a variant
/// when maps are compressed.
static COMPRESSED_VALUE_KEY: &str = "value";

/// A trait for which a [`KeyDispatchMapCodec`] can be made. In most cases,
/// using the [`crate::impl_key_dispatchable!`] macro is enough for your needs.
///
/// If you want to implement this trait manually, it requires implementing 3 functions,
/// which do something depending on a variant type:
/// - [`KeyDispatchable::key`]
/// - [`KeyDispatchable::encode`]
/// - [`KeyDispatchable::decode`]
/// - [`KeyDispatchable::map_encode`]
/// - [`KeyDispatchable::map_decode`]
pub trait KeyDispatchable: Sized {
    type Key;

    /// Gets the unique key of the variant stored.
    fn key(&self) -> Self::Key;

    /// Encodes a value of this type using an [`Encoder`] implementation of a [`MapEncoder`].
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<T>;

    /// Decodes a value to this type using a [`Decoder`] implementation of a [`MapDecoder`].
    fn decode<T: Display + PartialEq + Clone>(
        key: Self::Key,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self>;

    /// Encodes a value of this type using a [`MapEncoder`].
    fn map_encode<T: Display + PartialEq + Clone, B: StructBuilder<Value = T>>(
        &self,
        input: &Self,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: B,
    ) -> B;

    /// Decodes a value to this type using a [`MapDecoder`].
    fn map_decode<T: Display + PartialEq + Clone>(
        key: Self::Key,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self>;
}

/// Convenience macro to only implement a getter for some variant.
///
/// # Example
/// ```rust
/// # use pumpkin_codecs::{struct_map_codec, impl_getter_variant, impl_key_dispatchable};
/// # use pumpkin_codecs::codec::{field, FieldMapCodec, INT_CODEC, STRING_CODEC};
/// # use pumpkin_codecs::codecs::primitive::*;
/// # use pumpkin_codecs::coders::{Encoder, Decoder};
/// # use pumpkin_codecs::map_codec::for_getter;
/// # use pumpkin_codecs::map_coders::{MapEncoder, MapDecoder};
/// # use pumpkin_codecs::struct_codecs::StructMapCodec1;
/// #
/// # use pumpkin_codecs::map_codecs::key_dispatch::*;
///
/// pub enum Example {
///     A(String),
///     B { field: i32 }
/// }
///
/// /// A [`MapCodec`] for specific fields of `Example::A`.
/// pub static A_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<StringCodec>> = struct_map_codec!(
///     for_getter(field(&STRING_CODEC, "a_field"), impl_getter_variant!(Example::A(x) => x)),
///     Example::A
/// );
///
/// /// A [`MapCodec`] for specific fields of `Example::B`.
/// pub static B_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<IntCodec>> = struct_map_codec!(
///     for_getter(field(&INT_CODEC, "b_field"),  impl_getter_variant!(Example::B {field: x} => x)),
///     |i| Example::B { field: i }
/// );
/// ```
#[macro_export]
macro_rules! impl_getter_variant {
    ($pattern:pat => $result:expr) => {
        |ty|
            if let $pattern = ty {
                $result
            } else {
                unreachable!("Tried to encode a value of an enum type using a MapCodec of a different enum type")
            }
    };
}

/// A macro to generate an implementation of [`KeyDispatchable`].
///
/// This macro has two ways of using it:
/// - Using a *String* to differentiate between variants.
/// - Using a *enum* to differentiate between variants, whose variants are **all units**.
///   The enum **must** implement `Display` for errors.
///
/// Place this in an `impl KeyDispatchable for ...` block.
///
/// In the macro, you place a branch for each variant. Here's an example:
/// ```txt
/// (Self::A(..), "a") => A_MAP_CODEC
/// ```
/// Here, `Self::A(...)` is a pattern and `"a"` is the differentiator pattern (a string slice here).
/// `A_MAP_CODEC` is the map codec that will be used for encoding/decoding the specific fields of `A`.
///
/// You don't have to add a branch for every variant, but any left variant `MapCodec`s will be considered `todo!()`,
/// so if *map* encoding is attempted with an unimplemented variant, it will **panic**.
///
/// # Examples
///
/// ```rust
/// # use pumpkin_codecs::{struct_map_codec, impl_getter_variant, impl_key_dispatchable};
/// # use pumpkin_codecs::codec::{field, FieldMapCodec, INT_CODEC, STRING_CODEC};
/// # use pumpkin_codecs::codecs::primitive::*;
/// # use pumpkin_codecs::coders::{Encoder, Decoder};
/// # use pumpkin_codecs::map_codec::for_getter;
/// # use pumpkin_codecs::map_coders::{MapEncoder, MapDecoder};
/// # use pumpkin_codecs::struct_codecs::StructMapCodec1;
///
/// use pumpkin_codecs::map_codecs::key_dispatch::*;
///
/// /// Our example `KeyDispatchable`.
/// pub enum Example {
///     A(String),
///     B { field: i32 }
/// }
///
/// // `MapCodec`s for specific variants will probably not be used anywhere else, so it's
/// // fine to keep their types as is without using `pub type ...`.
///
/// // pub static A_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<StringCodec>> = ...
/// // pub static B_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<IntCodec>> = ...
/// #
/// # pub static A_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<StringCodec>> = struct_map_codec!(
/// #     for_getter(field(&STRING_CODEC, "a_field"), impl_getter_variant!(Example::A(x) => x)),
/// #     Example::A
/// # );
/// #
/// # pub static B_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<IntCodec>> = struct_map_codec!(
/// #     for_getter(field(&INT_CODEC, "b_field"),  impl_getter_variant!(Example::B {field: x} => x)),
/// #     |i| Example::B { field: i }
/// # );
///
/// impl KeyDispatchable for Example {
///     // Our macro.
///     impl_key_dispatchable!(
///         // We use a String to differentiate our enum.
///         string,
///         (Self::A(..), "a") => A_MAP_CODEC,
///         (Self::B {..}, "b") => B_MAP_CODEC,
///     );
/// }
///
/// ```
///
/// An enum implementation would look something like this:
///
/// ```rust
/// # use pumpkin_codecs::{struct_map_codec, impl_getter_variant, impl_key_dispatchable};
/// # use pumpkin_codecs::codec::{field, FieldMapCodec, INT_CODEC, STRING_CODEC};
/// # use pumpkin_codecs::codecs::primitive::*;
/// # use pumpkin_codecs::coders::{Encoder, Decoder};
/// # use pumpkin_codecs::map_codec::for_getter;
/// # use pumpkin_codecs::map_coders::{MapEncoder, MapDecoder};
/// # use pumpkin_codecs::struct_codecs::StructMapCodec1;
/// #
/// # use pumpkin_codecs::map_codecs::key_dispatch::*;
/// #
/// # pub static A_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<StringCodec>> = struct_map_codec!(
/// #     for_getter(field(&STRING_CODEC, "a_field"), impl_getter_variant!(Example::A(x) => x)),
/// #     Example::A
/// # );
/// #
/// # pub static B_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<IntCodec>> = struct_map_codec!(
/// #     for_getter(field(&INT_CODEC, "b_field"),  impl_getter_variant!(Example::B {field: x} => x)),
/// #     |i| Example::B { field: i }
/// # );
/// use std::fmt;
///
/// /// The enum for which we want to make serialization possible.
/// pub enum Example {
///     A(String),
///     B { field: i32 }
/// }
///
/// /// Our differentiator enum.
/// #[derive(Debug, PartialEq, Eq, Copy, Clone)]
/// pub enum ExampleType {
///     A,
///     B
/// }
///
/// // `Display` implementation for our differentiator enum.
/// impl fmt::Display for ExampleType {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         // We just delegate to the `Debug` implementation.
///         write!(f, "{:?}", self)
///     }
/// }
///
/// impl KeyDispatchable for Example {
///     // Our macro.
///     impl_key_dispatchable!(
///         // We use a String to differentiate our enum.
///         enum ExampleType,
///         (Self::A(..), ExampleType::A) => A_MAP_CODEC,
///         (Self::B {..}, ExampleType::B) => B_MAP_CODEC,
///     );
/// }
/// ```
#[macro_export]
macro_rules! impl_key_dispatchable {
    (
        @internal,

        $(
            ( $variant:pat ) => $map_codec:ident
        ),+
    ) => {
        // Used to silence the warning for a complete `KeyDispatchable` implementation.
        #[allow(unreachable_patterns)]
        fn encode<T: std::fmt::Display + PartialEq + Clone>(
            &self,
            input: &Self,
            ops: &'static impl $crate::dynamic_ops::DynamicOps<Value = T>,
        ) -> $crate::data_result::DataResult<T> {
            match self {
                $(
                    $variant => $crate::map_coders::new_map_encoder_encoder(&$map_codec).encode_start(input, ops),
                )+

                _ => todo!("Map encode not implemented yet"),
            }
        }

        #[allow(unreachable_patterns)]
        fn map_encode<T: std::fmt::Display + PartialEq + Clone, B: $crate::struct_builder::StructBuilder<Value=T>>(&self, input: &Self, ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>, prefix: B) -> B {
            match self {
                $(
                    $variant => $map_codec.encode(input, ops, prefix),
                )+

                _ => todo!("Map encode not implemented yet"),
            }
        }
    };

    // For `KeyDispatchable`s that differentiate using a string.
    (
        string,

        // (Enum::FOO, "foo") => FOO_MAP_CODEC
        $(
            ( $variant:pat, $matched:literal ) => $map_codec:ident
        ),+

        $(,)?
    ) => {
        type Key = String;

        fn decode<T: std::fmt::Display + PartialEq + Clone>(key: Self::Key, input: T, ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>) -> $crate::data_result::DataResult<Self> {
            match key.as_str() {
                $(
                    $matched => $crate::map_coders::new_map_decoder_decoder(&$map_codec).parse(input, ops),
                )+

                &_ => $crate::data_result::DataResult::error(format!("Invalid differentiator value {key}")),
            }
        }

        fn map_decode<T: std::fmt::Display + PartialEq + Clone>(key: Self::Key, input: &impl $crate::map_like::MapLike<Value=T>, ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>) -> $crate::data_result::DataResult<Self> {
            match key.as_str() {
                $(
                    $matched => $map_codec.decode(input, ops),
                )+

                &_ => $crate::data_result::DataResult::error(format!("Invalid differentiator value {key}")),
            }
        }

        fn key(&self) -> Self::Key {
            match self {
                $( $variant => $matched.to_string(), )+
            }
        }

        impl_key_dispatchable!(@internal $(, ( $variant ) => $map_codec)+);
    };

    // For `KeyDispatchable`s that differentiate using an enum.
    (
        // Differentiator value enum (must be unit)
        enum $e:ty,

        // (Enum::FOO, Key::Foo) => FOO_MAP_CODEC
        $(
            ( $variant:pat, $matched:ident ) => $map_codec:ident
        ),+

        $(,)?
    ) => {
        type Key = $e;

        #[allow(unreachable_patterns)]
        fn decode<T: std::fmt::Display + PartialEq + Clone>(key: Self::Key, input: T, ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>) -> $crate::data_result::DataResult<Self> {
            match key {
                $( $matched => $crate::map_coders::new_map_decoder_decoder(&$map_codec).parse(input, ops), )+

                _ => $crate::data_result::DataResult::error(format!("Invalid differentiator value {key}")),
            }
        }

        #[allow(unreachable_patterns)]
        fn map_decode<T: std::fmt::Display + PartialEq + Clone>(key: Self::Key, input: &impl $crate::map_like::MapLike<Value=T>, ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>) -> $crate::data_result::DataResult<Self> {
            match key {
                $( $matched => $map_codec.decode(input, ops), )+

                _ => $crate::data_result::DataResult::error(format!("Invalid differentiator value {key}")),
            }
        }

        fn key(&self) -> Self::Key {
            match self {
                $( $variant => $matched, )+
            }
        }

        impl_key_dispatchable!(@internal $(, ( $variant ) => $map_codec)+);
    };
}

/// A type of [`MapCodec`] to handle encoding a struct-like type whose fields
/// can differ by variant. A very common example of this would be an *enum*.
///
/// This is only implementable for a type which implements [`KeyDispatchable`].
///
/// - `T` is the type to encode to/decode from.
/// - `M` is the `MapCodec` of the key value to differentiate between variants.
///   A common example would be `field(&STRING_CODEC, "type")`.
pub struct KeyDispatchMapCodec<T: KeyDispatchable, M: MapCodec<Value = T::Key> + 'static> {
    key_codec: M,
    phantom: PhantomData<T>,
}

impl<T: KeyDispatchable, M: MapCodec<Value = T::Key>> HasValue for KeyDispatchMapCodec<T, M> {
    type Value = T;
}

impl<T: KeyDispatchable, M: MapCodec<Value = T::Key>> Keyable for KeyDispatchMapCodec<T, M> {
    fn keys(&self) -> Vec<String> {
        let mut vec = self.key_codec.keys();
        vec.push(COMPRESSED_VALUE_KEY.to_string());
        vec
    }
}

impl<T: KeyDispatchable, M: MapCodec<Value = T::Key>> CompressorHolder
    for KeyDispatchMapCodec<T, M>
{
    impl_compressor!();
}

impl<T: KeyDispatchable, M: MapCodec<Value = T::Key>> MapEncoder for KeyDispatchMapCodec<T, M> {
    fn encode<U: Display + PartialEq + Clone, B: StructBuilder<Value = U>>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = U>,
        prefix: B,
    ) -> B {
        if ops.compress_maps() {
            input
                .map_encode(input, ops, prefix)
                .add_string_key_value_result(COMPRESSED_VALUE_KEY, input.encode(input, ops))
        } else {
            self.key_codec
                .encode(&input.key(), ops, input.map_encode(input, ops, prefix))
        }
    }
}

impl<T: KeyDispatchable, M: MapCodec<Value = T::Key>> MapDecoder for KeyDispatchMapCodec<T, M> {
    fn decode<U: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = U>,
        ops: &'static impl DynamicOps<Value = U>,
    ) -> DataResult<Self::Value> {
        self.key_codec.decode(input, ops).flat_map(|key| {
            if ops.compress_maps() {
                // If the ops we're using compresses maps, we decode
                // the data in the individual "value" key using a normal `Codec`.
                input.get_str(COMPRESSED_VALUE_KEY).map_or_else(
                    || DataResult::error("Input doesn't have a 'value' key".to_string()),
                    |v| T::decode(key, v.clone(), ops),
                )
            } else {
                // We decode the fields in our map directly.
                T::map_decode(key, input, ops)
            }
        })
    }
}

pub(crate) const fn new_key_dispatch_map_codec<T: KeyDispatchable, C: MapCodec<Value = T::Key>>(
    key_codec: C,
) -> KeyDispatchMapCodec<T, C> {
    KeyDispatchMapCodec {
        key_codec,
        phantom: PhantomData,
    }
}

#[cfg(test)]
mod test {
    use crate::codec::{
        DOUBLE_CODEC, FieldMapCodec, FieldedKeyDispatchCodec, STRING_CODEC, XmapCodec, dispatch,
        field, list, xmap,
    };
    use crate::codecs::primitive::*;
    use crate::coders::{Decoder, Encoder};
    use crate::map_codec::for_getter;
    use crate::map_coders::{MapDecoder, MapEncoder};
    use crate::struct_codecs::{StructMapCodec1, StructMapCodec2};
    use crate::struct_map_codec;

    use crate::map_codecs::key_dispatch::*;

    use crate::codecs::list::ListCodec;
    use crate::json_ops;
    use serde_json::json;

    /// A shape, which can be a circle, rectangle or triangle.
    #[derive(Debug, PartialEq)]
    pub enum Shape {
        Circle { radius: f64 },
        Rectangle { width: f64, height: f64 },
        Triangle { sides: [f64; 3] },
    }

    pub type CircleMapCodec = StructMapCodec1<Shape, FieldMapCodec<DoubleCodec>>;
    pub static CIRCLE_MAP_CODEC: CircleMapCodec = struct_map_codec!(
        for_getter(
            field(&DOUBLE_CODEC, "radius"),
            impl_getter_variant!(Shape::Circle { radius } => radius)
        ),
        |radius| Shape::Circle { radius },
    );

    pub type RectangleMapCodec =
        StructMapCodec2<Shape, FieldMapCodec<DoubleCodec>, FieldMapCodec<DoubleCodec>>;
    pub static RECTANGLE_MAP_CODEC: RectangleMapCodec = struct_map_codec!(
        for_getter(
            field(&DOUBLE_CODEC, "width"),
            impl_getter_variant!(Shape::Rectangle { width, .. } => width)
        ),
        for_getter(
            field(&DOUBLE_CODEC, "height"),
            impl_getter_variant!(Shape::Rectangle { height, .. } => height)
        ),
        |width, height| Shape::Rectangle { width, height },
    );

    pub type TriangleMapCodec =
        StructMapCodec1<Shape, FieldMapCodec<XmapCodec<[f64; 3], ListCodec<DoubleCodec>>>>;
    pub static TRIANGLE_MAP_CODEC: TriangleMapCodec = struct_map_codec!(
        for_getter(
            field(
                &xmap(
                    &list(&DOUBLE_CODEC, 3, 3),
                    // The list only allows exactly 3 elements, so this should be fine.
                    |s| s.try_into().unwrap(),
                    |s| Vec::from(s)
                ),
                "sides"
            ),
            impl_getter_variant!(Shape::Triangle { sides, .. } => sides)
        ),
        |sides| Shape::Triangle { sides },
    );

    impl KeyDispatchable for Shape {
        impl_key_dispatchable!(
            string,
            (Self::Circle {..}, "circle") => CIRCLE_MAP_CODEC,
            (Self::Rectangle {..}, "rectangle") => RECTANGLE_MAP_CODEC,
            (Self::Triangle {..}, "triangle") => TRIANGLE_MAP_CODEC,
        );
    }

    pub static SHAPE_CODEC: FieldedKeyDispatchCodec<Shape, StringCodec> =
        dispatch::<Shape, StringCodec>(&STRING_CODEC);

    #[test]
    fn encoding() {
        // Encoding a circle
        assert_eq!(
            SHAPE_CODEC
                .encode_start(&Shape::Circle { radius: 50.0 }, &json_ops::INSTANCE)
                .expect("Encoding should succeed"),
            json!({
                "type": "circle",
                "radius": 50.0,
            })
        );

        // Encoding a square
        assert_eq!(
            SHAPE_CODEC
                .encode_start(
                    &Shape::Rectangle {
                        width: 8.0,
                        height: 12.5
                    },
                    &json_ops::INSTANCE
                )
                .expect("Encoding should succeed"),
            json!({
                "type": "rectangle",
                "width": 8.0,
                "height": 12.5
            })
        );

        // Encoding a triangle
        assert_eq!(
            SHAPE_CODEC
                .encode_start(
                    &Shape::Triangle {
                        sides: [2.0, 3.0, 4.0],
                    },
                    &json_ops::INSTANCE
                )
                .expect("Encoding should succeed"),
            json!({
                "type": "triangle",
                "sides": [2.0, 3.0, 4.0]
            })
        );
    }

    #[test]
    fn decoding() {
        assert_eq!(
            SHAPE_CODEC
                .parse(
                    json!({
                        "type": "circle",
                        "radius": 10.0
                    }),
                    &json_ops::INSTANCE
                )
                .expect("Decoding should succeed"),
            Shape::Circle { radius: 10.0 }
        );

        assert_eq!(
            SHAPE_CODEC
                .parse(
                    json!({
                        "type": "rectangle",
                        "width": 12.3,
                        "height": 45.6
                    }),
                    &json_ops::INSTANCE
                )
                .expect("Decoding should succeed"),
            Shape::Rectangle {
                width: 12.3,
                height: 45.6
            }
        );

        assert_eq!(
            SHAPE_CODEC
                .parse(
                    json!({
                        "type": "triangle",
                        "sides": [12, 15, 18]
                    }),
                    &json_ops::INSTANCE
                )
                .expect("Decoding should succeed"),
            Shape::Triangle {
                sides: [12.0, 15.0, 18.0]
            }
        );

        assert!(
            SHAPE_CODEC
                .parse(
                    json!({
                        // There is no `Square` shape.
                        "type": "square",
                        "side": 4
                    }),
                    &json_ops::INSTANCE
                )
                .get_message()
                .expect("Decoding should fail")
                .starts_with("Invalid differentiator value")
        );
    }
}
