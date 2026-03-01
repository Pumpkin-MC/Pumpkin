use std::collections::hash_map::Entry;
use crate::map_like::MapLike;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;
use crate::codec::Codec;
use crate::coders::{Decoder, Encoder};
use crate::data_result::DataResult;
use crate::dynamic_ops::DynamicOps;
use crate::HasValue;
use crate::lifecycle::Lifecycle;
use crate::struct_builder::StructBuilder;

/// A trait for which a [`DispatchedMapCodec`] can be made. In most cases,
/// using the [`crate::impl_map_dispatchable!`] macro is enough for your needs.
///
/// This trait simply serializes/deserializes a variant with a codec depending on that variant's key.
///
/// If you want to implement this trait manually, it requires implementing 2 functions,
/// which do something depending on a variant type:
/// - [`KeyDispatchable::encode`]
/// - [`KeyDispatchable::decode`]
pub trait MapDispatchable {
    type Key: Clone;

    /// The dispatched type.
    type Value: ?Sized;

    /// Encodes a value of this type using an [`Encoder`].
    fn encode<T: Display + PartialEq + Clone>(
        key: &Self::Key,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value=T>,
    ) -> DataResult<T>;

    /// Decodes a value to this type using a [`Decoder`].
    fn decode<T: Display + PartialEq + Clone>(
        key: &Self::Key,
        input: T,
        ops: &'static impl DynamicOps<Value=T>,
    ) -> DataResult<Box<Self::Value>>;
}

#[macro_export]
macro_rules! impl_map_dispatchable {
    (
        $vis:vis $marker:ident, $trait_ty:ident, $key_ty:ty,
        $(
            $key:pat, $ty:ty => $codec:ident
        ),+ $(,)?
    ) => {
        $vis struct $marker;
        impl $crate::codecs::dispatched_map::MapDispatchable for $marker {

            type Key = $key_ty;
            type Value = dyn $trait_ty;
        }
    };

    // Specifically for strings
    (
        $vis:vis $marker:ident, $trait_ty:ident,
        $(
            $key:pat, $ty:ty => $codec:ident
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
                            || $crate::data_result::DataResult::error(format!("Invalid type for key {key}")),
                            |o| $codec.encode_start(o, ops)
                        ),
                    )+
                    _ => todo!("Encode not implemented yet")
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
                    _ => todo!("Decode not implemented yet")
                }
            }
        }
    };
}

pub struct DispatchedMapCodec<T: MapDispatchable, K: Codec<Value = T::Key> + 'static> where K::Value: Display + Eq + Hash {
    key_codec: &'static K,
    phantom: PhantomData<T>
}

impl<T: MapDispatchable, K: Codec<Value = T::Key>> HasValue for DispatchedMapCodec<T, K> where K::Value: Display + Eq + Hash {
    type Value = HashMap<K::Value, Box<T::Value>>;
}

impl<T: MapDispatchable, K: Codec<Value = T::Key>> Encoder for DispatchedMapCodec<T, K> where K::Value: Display + Eq + Hash {
    fn encode<U: Display + PartialEq + Clone>(&self, input: &Self::Value, ops: &'static impl DynamicOps<Value=U>, prefix: U) -> DataResult<U> {
        let mut builder = ops.map_builder();
        for (k, v) in input {
            builder = builder.add_key_result_value_result(
                self.key_codec.encode_start(k, ops),
                T::encode(k, v, ops)
            );
        }
        builder.build(prefix)
    }
}

impl<T: MapDispatchable, K: Codec<Value = T::Key>> Decoder for DispatchedMapCodec<T, K> where K::Value: Display + Eq + Hash {
    fn decode<U: Display + PartialEq + Clone>(&self, input: U, ops: &'static impl DynamicOps<Value=U>) -> DataResult<(Self::Value, U)> {
        ops.get_map(&input.clone()).flat_map(|m| {
            let mut entries = HashMap::new();
            let mut failed: Vec<(U, U)> = Vec::new();

            let final_result = m.iter().fold(
                DataResult::success_with_lifecycle((), Lifecycle::Stable),
                |r, (k, v)| self.parse_entry(r, ops, (k, v.clone()), &mut entries, &mut failed)
            );

            let errors = ops.create_map(failed);

            final_result
                .with_complete_or_partial((entries, input))
                .map_error(|e| format!("{e}, missed input: {errors}"))
        })
    }
}

/// Parses a single entry.
impl<T: MapDispatchable, K: Codec<Value = T::Key>> DispatchedMapCodec<T, K> where K::Value: Display + Eq + Hash {
    fn parse_entry<U: Display + PartialEq + Clone>(
        &self,
        result: DataResult<()>, ops: &'static impl DynamicOps<Value=U>, entry: (U, U),
        entries: &mut HashMap<K::Value, Box<T::Value>>, failed: &mut Vec<(U, U)>
    ) -> DataResult<()> {
        let duplicated_entry = entry.clone();

        let key_result = self.key_codec.parse(duplicated_entry.0, ops);
        let value_result = key_result.clone().flat_map(|k| T::decode(&k, duplicated_entry.1, ops));

        let entry_result = key_result.apply_2_and_make_stable(|k, v| (k, v), value_result);

        let returned_result = result.clone().add_message(&entry_result);

        if entry_result.is_error() {
            failed.push(entry);
        }

        if let Some((k, v)) = entry_result.into_result_or_partial() {
            match entries.entry(k.clone()) {
                Entry::Occupied(_) => {
                    return result.add_message(&DataResult::<()>::error(format!("Duplicate entry for key: {k}")));
                },
                Entry::Vacant(_) => {
                    entries.insert(k, v);
                },
            }
        }

        returned_result
    }
}

/// Creates a new [`DispatchedMapCodec`].
pub(crate) const fn dispatched_map_codec<T: MapDispatchable, K: Codec<Value = T::Key>>(
    key_codec: &'static K
) -> DispatchedMapCodec<T, K>
where
    <K as HasValue>::Value: Display + Eq + Hash,
{
    DispatchedMapCodec {
        key_codec,
        phantom: PhantomData
    }
}

#[cfg(test)]
mod test {
    use std::any::Any;
    use std::collections::HashMap;
    use crate::{json_ops, struct_codec};
    use crate::codec::{field, xmap, FieldMapCodec, UintCodec, XmapCodec, BOOL_CODEC, FLOAT_CODEC, STRING_CODEC, UINT_CODEC};
    use crate::codecs::dispatched_map::{dispatched_map_codec, DispatchedMapCodec};
    use crate::codecs::map_codec::MapCodecCodec;
    use crate::codecs::primitive::{BoolCodec, FloatCodec, StringCodec};
    use crate::coders::{Decoder, Encoder};
    use crate::map_codec::for_getter;
    use crate::struct_codecs::StructMapCodec3;

    pub trait TextModifier: Any {
        fn as_any(&self) -> &dyn Any;
    }

    impl_map_dispatchable!(
        TextModifierDispatch, TextModifier,
        "font_size", FontSizeTextModifier => FONT_SIZE_MODIFIER_CODEC,
        "font_style", FontStyleTextModifier => FONT_STYLE_MODIFIER_CODEC,
        "color", ColorTextModifier => COLOR_MODIFIER_CODEC,
    );

    pub struct FontSizeTextModifier {
        size: u32
    }
    impl TextModifier for FontSizeTextModifier {
        fn as_any(&self) -> &dyn Any { self }
    }

    static FONT_SIZE_MODIFIER_CODEC: XmapCodec<FontSizeTextModifier, UintCodec> =
        xmap(&UINT_CODEC, |size| FontSizeTextModifier { size }, |m| m.size);

    pub struct FontStyleTextModifier {
        bold: bool, underlined: bool, italic: bool
    }
    impl TextModifier for FontStyleTextModifier {
        fn as_any(&self) -> &dyn Any { self }
    }

    static FONT_STYLE_MODIFIER_CODEC: MapCodecCodec<StructMapCodec3<FontStyleTextModifier, FieldMapCodec<BoolCodec>, FieldMapCodec<BoolCodec>, FieldMapCodec<BoolCodec>>> = struct_codec!(
        for_getter(field(&BOOL_CODEC, "bold"), |f| &f.bold),
        for_getter(field(&BOOL_CODEC, "underlined"), |f| &f.underlined),
        for_getter(field(&BOOL_CODEC, "italic"), |f| &f.italic),
        |bold, underlined, italic| FontStyleTextModifier { bold, underlined, italic }
    );

    pub struct ColorTextModifier {
        r: f32, g: f32, b: f32
    }
    impl TextModifier for ColorTextModifier {
        fn as_any(&self) -> &dyn Any { self }
    }

    static COLOR_MODIFIER_CODEC: MapCodecCodec<StructMapCodec3<ColorTextModifier, FieldMapCodec<FloatCodec>, FieldMapCodec<FloatCodec>, FieldMapCodec<FloatCodec>>> = struct_codec!(
        for_getter(field(&FLOAT_CODEC, "r"), |f| &f.r),
        for_getter(field(&FLOAT_CODEC, "g"), |f| &f.g),
        for_getter(field(&FLOAT_CODEC, "b"), |f| &f.b),
        |r, g, b| ColorTextModifier { r, g, b }
    );

    pub static TEXT_MODIFIER_CODEC: DispatchedMapCodec<TextModifierDispatch, StringCodec> = dispatched_map_codec::<TextModifierDispatch, StringCodec>(&STRING_CODEC);

    #[test]
    fn encoding() {
        let mut map: HashMap<String, Box<dyn TextModifier>> = HashMap::new();

        map.insert("font_style".to_string(), Box::new(FontStyleTextModifier {
            bold: true,
            underlined: true,
            italic: false
        }));

        map.insert("font_size".to_string(), Box::new(FontSizeTextModifier {
            size: 16
        }));

        map.insert("color".to_string(), Box::new(ColorTextModifier {
            r: 0.5, g: 0.0, b: 1.0
        }));

        println!("{:#?}", TEXT_MODIFIER_CODEC.encode_start(&map, &json_ops::INSTANCE));
    }
}
