use crate::HasValue;
use crate::codec::Codec;
use crate::coders::{Decoder, Encoder};
use crate::data_result::DataResult;
use crate::dynamic_ops::DynamicOps;
use crate::lifecycle::Lifecycle;
use crate::map_like::MapLike;
use crate::struct_builder::StructBuilder;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;

/// A trait for which a [`DispatchedMapCodec`] can be made. In most cases,
/// using the [`crate::impl_map_dispatchable!`] macro is enough for your needs.
///
/// This trait simply serializes/deserializes a dynamically dispatched value
/// with a codec depending on its key. This trait is meant to be implemented by a
/// marker type (already handled by `impl_map_dispatchable!`), and then used
/// while creating a codec instance.
///
/// If you want to implement this trait manually, it requires implementing 2 functions,
/// which do something depending on a variant type:
/// - [`MapDispatchable::encode`]
/// - [`MapDispatchable::decode`]
pub trait MapDispatchable {
    type Key: Clone;

    /// The dispatched type.
    type Value: ?Sized;

    /// Encodes a value of this marker's value type using an [`Encoder`] (or any [`Codec`] that can encode the value).
    fn encode<T: Display + PartialEq + Clone>(
        key: &Self::Key,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<T>;

    /// Decodes a value to this marker's value type using a [`Decoder`] (or any [`Codec`] that can decode the value).
    fn decode<T: Display + PartialEq + Clone>(
        key: &Self::Key,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Box<Self::Value>>;
}

/// A macro to generate an implementation of [`MapDispatchable`].
///
/// This macro is only meant for *trait object* value types.
/// The trait (map value type) needs to extend [`Any`] and implement the `as_any` method for all its implementations,
/// and should look like this:
/// ```
/// fn as_any(&self) -> &dyn Any;
/// ```
/// This macro generates a unit struct marker type, which can be used to define the [`DispatchedMapCodec`].
///
/// This macro has two ways of matching a key for each value:
/// - Using a *String* to pattern match `&str`s (string slices).
/// - Using any other type (*preferably a unit enum*), whose reference will be used for pattern matching.
///
/// In the macro, you place a branch for each key. Here's an example:
/// ```txt
/// ("a", TraitAImpl) => A_CODEC
/// ```
/// Here:
/// - `"a"` is the key (a string slice here).
/// - `TraitImplA` is an implementation of the trait (which is the value type of the map).
/// - `A_CODEC` is the codec that will be used for encoding/decoding the specific fields of `TraitImplA`.
///
/// For using unit enums, it would look like:
/// ```txt
/// (Enum::A, TraitAImpl) => A_CODEC
/// ```
///
/// You don't have to add a branch for every key, but any left keys will be considered invalid,
/// so if *any encoding or decoding* is attempted with an unimplemented key, it will give an error [`DataResult`].
///
/// Here is an example:
///
/// ```rust
/// # use pumpkin_codecs::{struct_codec};
/// # use pumpkin_codecs::codec::*;
/// # use pumpkin_codecs::codecs::primitive::*;
/// # use pumpkin_codecs::map_codec::{for_getter};
/// # use pumpkin_codecs::struct_codecs::StructCodec3;
///
/// use std::any::Any;
/// use pumpkin_codecs::{impl_map_dispatchable};
/// use pumpkin_codecs::codecs::dispatched_map::*;
/// use pumpkin_codecs::coders::{Encoder, Decoder};
///
/// /// A certain modifier for text.
/// pub trait TextModifier: Any {
///     fn as_any(&self) -> &dyn Any;
/// }
///
/// pub struct FontSizeTextModifier {
///     size: u32
/// }
/// impl TextModifier for FontSizeTextModifier {
///     fn as_any(&self) -> &dyn Any { self }
/// }
///
/// static FONT_SIZE_MODIFIER_CODEC: XmapCodec<FontSizeTextModifier, UintCodec> =
///     xmap(&UINT_CODEC, |size| FontSizeTextModifier { size }, |m| m.size);
///
/// pub struct FontStyleTextModifier {
///     bold: bool, underlined: bool, italic: bool
/// }
/// impl TextModifier for FontStyleTextModifier {
///     fn as_any(&self) -> &dyn Any { self }
/// }
///
/// static FONT_STYLE_MODIFIER_CODEC: StructCodec3<FontStyleTextModifier, FieldMapCodec<BoolCodec>, FieldMapCodec<BoolCodec>, FieldMapCodec<BoolCodec>> = struct_codec!(
///     for_getter(field(&BOOL_CODEC, "bold"), |f| &f.bold),
///     for_getter(field(&BOOL_CODEC, "underlined"), |f| &f.underlined),
///     for_getter(field(&BOOL_CODEC, "italic"), |f| &f.italic),
///     |bold, underlined, italic| FontStyleTextModifier { bold, underlined, italic }
/// );
///
/// pub struct ColorTextModifier {
///     r: f32, g: f32, b: f32
/// }
/// impl TextModifier for ColorTextModifier {
///     fn as_any(&self) -> &dyn Any { self }
/// }
///
/// static COLOR_MODIFIER_CODEC: StructCodec3<ColorTextModifier, FieldMapCodec<FloatCodec>, FieldMapCodec<FloatCodec>, FieldMapCodec<FloatCodec>> = struct_codec!(
///     for_getter(field(&FLOAT_CODEC, "r"), |f| &f.r),
///     for_getter(field(&FLOAT_CODEC, "g"), |f| &f.g),
///     for_getter(field(&FLOAT_CODEC, "b"), |f| &f.b),
///     |r, g, b| ColorTextModifier { r, g, b }
/// );
///
/// impl_map_dispatchable!(
///     // `TextModifierDispatch` is our marker type, which is implicitly declared here.
///     // `TextModifier` is our trait we want to store objects of in a map based on the key.
///     pub TextModifierDispatch, TextModifier,
///     ("font_size", FontSizeTextModifier) => FONT_SIZE_MODIFIER_CODEC,
///     ("font_style", FontStyleTextModifier) => FONT_STYLE_MODIFIER_CODEC,
///     ("color", ColorTextModifier) => COLOR_MODIFIER_CODEC,
/// );
///
/// pub static TEXT_MODIFIER_CODEC: DispatchedMapCodec<TextModifierDispatch, StringCodec> = dispatched_map::<TextModifierDispatch, StringCodec>(&STRING_CODEC);
/// ```
///
/// [`Any`]: std::any::Any
#[macro_export]
macro_rules! impl_map_dispatchable {
    // Specifically for strings
    (
        $vis:vis $marker:ident, $trait_ty:ident,
        $(
            ($key:pat, $ty:ty) => $codec:ident
        ),+ $(,)?
    ) => {

        $vis struct $marker;
        impl $crate::codecs::dispatched_map::MapDispatchable for $marker {
            type Key = String;
            type Value = dyn $trait_ty;

            fn encode<T: std::fmt::Display + PartialEq + Clone>(
                key: &Self::Key,
                input: &Self::Value,
                ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>,
            ) -> $crate::data_result::DataResult<T> {
                match key.as_str() {
                    $(
                        $key => input.as_any().downcast_ref::<$ty>().map_or_else(
                            || $crate::data_result::DataResult::new_error(format!("Invalid type for key {key}")),
                            |o| $codec.encode_start(o, ops)
                        ),
                    )+
                    _ => $crate::data_result::DataResult::new_error(format!("Invalid key for map {key}"))
                }
            }

            fn decode<T: std::fmt::Display + PartialEq + Clone>(
                key: &Self::Key,
                input: T,
                ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>,
            ) -> $crate::data_result::DataResult<Box<Self::Value>> {
                match key.as_str() {
                    $(
                        $key => $codec.parse(input, ops).map(|d| Box::new(d) as Box<dyn $trait_ty>),
                    )+
                    _ => $crate::data_result::DataResult::new_error(format!("Invalid key for map {key}"))
                }
            }
        }
    };

    (
        $vis:vis $marker:ident, $trait_ty:ident, $key_ty:ty,
        $(
            ($key:pat, $ty:ty) => $codec:ident
        ),+ $(,)?
    ) => {

        $vis struct $marker;
        impl $crate::codecs::dispatched_map::MapDispatchable for $marker {
            type Key = $key_ty;
            type Value = dyn $trait_ty;

            fn encode<T: std::fmt::Display + PartialEq + Clone>(
                key: &Self::Key,
                input: &Self::Value,
                ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>,
            ) -> $crate::data_result::DataResult<T> {
                match key {
                    $(
                        $key => input.as_any().downcast_ref::<$ty>().map_or_else(
                            || $crate::data_result::DataResult::new_error(format!("Invalid type for key {key}")),
                            |o| $codec.encode_start(o, ops)
                        ),
                    )+
                   _ => $crate::data_result::DataResult::new_error(format!("Invalid key for map {key}"))
                }
            }

            fn decode<T: std::fmt::Display + PartialEq + Clone>(
                key: &Self::Key,
                input: T,
                ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>,
            ) -> $crate::data_result::DataResult<Box<Self::Value>> {
                match key {
                    $(
                        $key => $codec.parse(input, ops).map(|d| Box::new(d) as Box<dyn $trait_ty>),
                    )+
                    _ => $crate::data_result::DataResult::new_error(format!("Invalid key for map {key}"))
                }
            }
        }
    };
}

pub struct DispatchedMapCodec<T: MapDispatchable, K: Codec<Value = T::Key> + 'static>
where
    K::Value: Display + Eq + Hash,
{
    key_codec: &'static K,
    phantom: PhantomData<T>,
}

impl<T: MapDispatchable, K: Codec<Value = T::Key>> HasValue for DispatchedMapCodec<T, K>
where
    K::Value: Display + Eq + Hash,
{
    type Value = HashMap<K::Value, Box<T::Value>>;
}

impl<T: MapDispatchable, K: Codec<Value = T::Key>> Encoder for DispatchedMapCodec<T, K>
where
    K::Value: Display + Eq + Hash,
{
    fn encode<U: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = U>,
        prefix: U,
    ) -> DataResult<U> {
        let mut builder = ops.map_builder();
        for (k, v) in input {
            builder = builder.add_key_result_value_result(
                self.key_codec.encode_start(k, ops),
                T::encode(k, v, ops),
            );
        }
        builder.build(prefix)
    }
}

impl<T: MapDispatchable, K: Codec<Value = T::Key>> Decoder for DispatchedMapCodec<T, K>
where
    K::Value: Display + Eq + Hash,
{
    fn decode<U: Display + PartialEq + Clone>(
        &self,
        input: U,
        ops: &'static impl DynamicOps<Value = U>,
    ) -> DataResult<(Self::Value, U)> {
        ops.get_map(&input.clone()).flat_map(|m| {
            let mut entries = HashMap::new();
            let mut failed: Vec<(U, U)> = Vec::new();

            let final_result = m.iter().fold(
                DataResult::new_success_with_lifecycle((), Lifecycle::Stable),
                |r, (k, v)| self.parse_entry(r, ops, (k, v.clone()), &mut entries, &mut failed),
            );

            let errors = ops.create_map(failed);

            final_result
                .with_complete_or_partial((entries, input))
                .map_error(|e| format!("{e}, missed input: {errors}"))
        })
    }
}

/// Parses a single entry.
impl<T: MapDispatchable, K: Codec<Value = T::Key>> DispatchedMapCodec<T, K>
where
    K::Value: Display + Eq + Hash,
{
    fn parse_entry<U: Display + PartialEq + Clone>(
        &self,
        result: DataResult<()>,
        ops: &'static impl DynamicOps<Value = U>,
        entry: (U, U),
        entries: &mut HashMap<K::Value, Box<T::Value>>,
        failed: &mut Vec<(U, U)>,
    ) -> DataResult<()> {
        let duplicated_entry = entry.clone();

        let key_result = self.key_codec.parse(duplicated_entry.0, ops);
        let value_result = key_result
            .clone()
            .flat_map(|k| T::decode(&k, duplicated_entry.1, ops));

        let entry_result = key_result.apply_2_and_make_stable(|k, v| (k, v), value_result);

        let returned_result = result.clone().add_message(&entry_result);

        if entry_result.is_error() {
            failed.push(entry);
        }

        if let Some((k, v)) = entry_result.into_result_or_partial() {
            match entries.entry(k.clone()) {
                Entry::Occupied(_) => {
                    return result.add_message(&DataResult::<()>::new_error(format!(
                        "Duplicate entry for key: {k}"
                    )));
                }
                Entry::Vacant(_) => {
                    entries.insert(k, v);
                }
            }
        }

        returned_result
    }
}

/// Creates a new [`DispatchedMapCodec`].
pub(crate) const fn new_dispatched_map_codec<T: MapDispatchable, K: Codec<Value = T::Key>>(
    key_codec: &'static K,
) -> DispatchedMapCodec<T, K>
where
    <K as HasValue>::Value: Display + Eq + Hash,
{
    DispatchedMapCodec {
        key_codec,
        phantom: PhantomData,
    }
}

#[cfg(test)]
mod test {
    use crate::codec::{
        BOOL_CODEC, FLOAT_CODEC, FieldMapCodec, STRING_CODEC, UINT_CODEC, UintCodec, XmapCodec,
        dispatched_map, field, xmap,
    };
    use crate::codecs::dispatched_map::DispatchedMapCodec;
    use crate::codecs::primitive::{BoolCodec, FloatCodec, StringCodec};
    use crate::coders::{Decoder, Encoder};
    use crate::map_codec::for_getter;
    use crate::struct_codecs::StructCodec3;
    use crate::{assert_decode, json_ops, struct_codec};
    use serde_json::json;
    use std::any::Any;
    use std::collections::HashMap;
    use std::fmt::Debug;

    pub trait TextModifier: Any + Debug {
        fn as_any(&self) -> &dyn Any;
    }

    impl_map_dispatchable!(
        pub TextModifierDispatch, TextModifier,
        ("font_size", FontSizeTextModifier) => FONT_SIZE_MODIFIER_CODEC,
        ("font_style", FontStyleTextModifier) => FONT_STYLE_MODIFIER_CODEC,
        ("color", ColorTextModifier) => COLOR_MODIFIER_CODEC,
    );

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub struct FontSizeTextModifier {
        size: u32,
    }
    impl TextModifier for FontSizeTextModifier {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    static FONT_SIZE_MODIFIER_CODEC: XmapCodec<FontSizeTextModifier, UintCodec> = xmap(
        &UINT_CODEC,
        |size| FontSizeTextModifier { size },
        |m| m.size,
    );

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub struct FontStyleTextModifier {
        bold: bool,
        underlined: bool,
        italic: bool,
    }
    impl TextModifier for FontStyleTextModifier {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    static FONT_STYLE_MODIFIER_CODEC: StructCodec3<
        FontStyleTextModifier,
        FieldMapCodec<BoolCodec>,
        FieldMapCodec<BoolCodec>,
        FieldMapCodec<BoolCodec>,
    > = struct_codec!(
        for_getter(field(&BOOL_CODEC, "bold"), |f| &f.bold),
        for_getter(field(&BOOL_CODEC, "underlined"), |f| &f.underlined),
        for_getter(field(&BOOL_CODEC, "italic"), |f| &f.italic),
        |bold, underlined, italic| FontStyleTextModifier {
            bold,
            underlined,
            italic
        }
    );

    #[derive(Copy, Clone, PartialEq, Debug)]
    pub struct ColorTextModifier {
        r: f32,
        g: f32,
        b: f32,
    }
    impl TextModifier for ColorTextModifier {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    static COLOR_MODIFIER_CODEC: StructCodec3<
        ColorTextModifier,
        FieldMapCodec<FloatCodec>,
        FieldMapCodec<FloatCodec>,
        FieldMapCodec<FloatCodec>,
    > = struct_codec!(
        for_getter(field(&FLOAT_CODEC, "r"), |f| &f.r),
        for_getter(field(&FLOAT_CODEC, "g"), |f| &f.g),
        for_getter(field(&FLOAT_CODEC, "b"), |f| &f.b),
        |r, g, b| ColorTextModifier { r, g, b }
    );

    pub static TEXT_MODIFIER_CODEC: DispatchedMapCodec<TextModifierDispatch, StringCodec> =
        dispatched_map::<TextModifierDispatch, StringCodec>(&STRING_CODEC);

    #[test]
    fn encoding() {
        let mut map: HashMap<String, Box<dyn TextModifier + 'static>> = HashMap::new();

        map.insert(
            "font_style".to_string(),
            Box::new(FontStyleTextModifier {
                bold: true,
                underlined: true,
                italic: false,
            }),
        );

        map.insert(
            "font_size".to_string(),
            Box::new(FontSizeTextModifier { size: 16 }),
        );

        map.insert(
            "color".to_string(),
            Box::new(ColorTextModifier {
                r: 0.5,
                g: 0.0,
                b: 1.0,
            }),
        );

        assert_eq!(
            TEXT_MODIFIER_CODEC
                .encode_start(&map, &json_ops::INSTANCE)
                .expect("Encoding should succeed"),
            json!({
                "font_style": {
                    "bold": true,
                    "underlined": true,
                    "italic": false
                },
                "font_size": 16,
                "color": {
                    "r": 0.5,
                    "g": 0.0,
                    "b": 1.0
                },
            })
        );

        // Unlike struct codecs with extra fields, unknown keys give an error.
        map.insert(
            "shadow_color".to_string(),
            Box::new(ColorTextModifier {
                r: 0.5,
                g: 0.0,
                b: 1.0,
            }),
        );

        assert!(
            TEXT_MODIFIER_CODEC
                .encode_start(&map, &json_ops::INSTANCE)
                .is_error(),
            "Encoding should fail"
        );

        let mut map: HashMap<String, Box<dyn TextModifier + 'static>> = HashMap::new();

        // There is a mismatch in the key and implementation.
        map.insert(
            "font_style".to_string(),
            Box::new(ColorTextModifier {
                r: 0.0,
                g: 1.0,
                b: 1.0,
            }),
        );

        assert!(
            TEXT_MODIFIER_CODEC
                .encode_start(&map, &json_ops::INSTANCE)
                .is_error(),
            "Encoding should fail"
        );
    }

    #[test]
    fn decoding() {
        let map = TEXT_MODIFIER_CODEC
            .parse(
                json!({
                    "font_size": 120,
                    "color": {
                        "r": 0.0,
                        "g": 0.2,
                        "b": 0.4
                    }
                }),
                &json_ops::INSTANCE,
            )
            .expect("Decoding should succeed");

        assert_eq!(map.len(), 2);

        assert_eq!(
            map.get("font_size")
                .expect("Font size should be present")
                .as_any()
                .downcast_ref::<FontSizeTextModifier>()
                .expect("Font size should be FontSizeTextModifier"),
            &FontSizeTextModifier { size: 120 }
        );

        assert_eq!(
            map.get("color")
                .expect("Color should be present")
                .as_any()
                .downcast_ref::<ColorTextModifier>()
                .expect("Color should be ColorTextModifier"),
            &ColorTextModifier {
                r: 0.0,
                g: 0.2,
                b: 0.4
            }
        );

        assert!(!map.contains_key("font_style"));

        assert_decode!(
            TEXT_MODIFIER_CODEC,
            json!({"font_size": -3}),
            &json_ops::INSTANCE,
            is_error
        );

        assert_decode!(
            TEXT_MODIFIER_CODEC,
            json!({
                // There is no key called "shadow_color".
                "shadow_color": {
                    "r": 0.0,
                    "g": 0.2,
                    "b": 0.4
                }
            }),
            &json_ops::INSTANCE,
            is_error
        );

        assert_decode!(
            TEXT_MODIFIER_CODEC,
            json!({
                "font_style": {
                    "bold": false,
                    // The key is "underlined", not "underline".
                    "underline": false,
                    "italic": false
                }
            }),
            &json_ops::INSTANCE,
            is_error
        );
    }
}
