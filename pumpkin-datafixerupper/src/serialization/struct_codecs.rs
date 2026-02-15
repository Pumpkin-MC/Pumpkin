use crate::impl_compressor;
use crate::serialization::HasValue;
use crate::serialization::codecs::map_codec::MapCodecCodec;
use crate::serialization::codecs::map_codec::new_map_codec_codec;
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::key_compressor::KeyCompressor;
use crate::serialization::keyable::Keyable;
use crate::serialization::map_codec::MapCodec;
use crate::serialization::map_coders::{CompressorHolder, MapDecoder, MapEncoder};
use crate::serialization::map_like::MapLike;
use crate::serialization::struct_builder::StructBuilder;
use std::fmt::Display;
use std::sync::OnceLock;

/// A single field object to build a struct codec, which takes a [`MapCodec`] and a getter.
///
/// - `T` is the composite type to get from.
/// - `C` is the [`MapCodec`] for serializing/deserializing the field.
pub struct Field<T, C: MapCodec> {
    map_codec: C,
    getter: fn(&T) -> &C::Value,
}

/// Creates a new [`Field`] from a [`MapCodec`].
pub(crate) const fn new_field<T, C: MapCodec>(
    map_codec: C,
    getter: fn(&T) -> &C::Value,
) -> Field<T, C> {
    Field { map_codec, getter }
}

/// Macro to generate a `StructMapCodecN` struct (structure codec of `N` arguments).
/// This also creates a function to get a normal [`Codec`] from `N` fields.
macro_rules! impl_struct_map_codec {
    (@internal_start $n:literal $name:ident $alias:ident $apply_func:ident $func_name:ident $($codec_type:ident, $field:ident),*) => {
        #[doc = concat!("A [`MapCodec`] for a map with ", stringify!($n) , " rigid field(s).")]
        ///
        /// A [`Codec`] can then be made from this object.
        pub struct $name<T, C1: MapCodec + 'static $(, $codec_type: MapCodec + 'static)* > {
            field_1: Field<T, C1>,
            $( $field: Field<T, $codec_type> ,)*
            apply_function: fn(C1::Value $(, $codec_type::Value)*) -> T,

            compressor: OnceLock<KeyCompressor>,
        }

        impl<T, C1: MapCodec $(, $codec_type: MapCodec)* > HasValue for $name<T, C1 $(, $codec_type)*> {
            type Value = T;
        }

        impl<T, C1: MapCodec $(, $codec_type: MapCodec)* > Keyable for $name<T, C1 $(, $codec_type)*> {
            #[allow(unused_mut)]
            fn keys(&self) -> Vec<String> {
                let mut keys = self.field_1.map_codec.keys();
                $( keys.extend(self.$field.map_codec.keys()); )*
                keys
            }
        }

        impl<T, C1: MapCodec $(, $codec_type: MapCodec)* > CompressorHolder for $name<T, C1 $(, $codec_type)*> {
            impl_compressor!(compressor);
        }

        impl<T, C1: MapCodec $(, $codec_type: MapCodec)* > MapEncoder for $name<T, C1 $(, $codec_type)*> {
            #[allow(clippy::let_and_return)]
            fn encode<U: Display + PartialEq + Clone, B: StructBuilder<Value = U>>(&self, input: &Self::Value, ops: &'static impl DynamicOps<Value=U>, prefix: B) -> B {
                let prefix =
                    self.field_1
                        .map_codec
                        .encode((self.field_1.getter)(input), ops, prefix);
                $(
                    let prefix =
                    self.$field
                        .map_codec
                        .encode((self.$field.getter)(input), ops, prefix);
                )*
                prefix
            }
        }

        impl<T, C1: MapCodec $(, $codec_type: MapCodec)* > MapDecoder for $name<T, C1 $(, $codec_type)*> {
            fn decode<U: Display + PartialEq + Clone>(
                &self,
                input: &impl MapLike<Value = U>,
                ops: &'static impl DynamicOps<Value = U>,
            ) -> DataResult<Self::Value> {
                self.field_1.map_codec.decode(input, ops).$apply_func(
                    self.apply_function,
                    $( self.$field.map_codec.decode(input, ops), )*
                )
            }
        }

        #[doc = concat!("A type alias of a struct [`Codec`] with ", stringify!($n), " field(s).")]
        pub type $alias<T, C1 $(, $codec_type)* > = MapCodecCodec<$name<T, C1 $(, $codec_type)*>>;
    };

    ($n:literal, $name:ident, $alias:ident, $apply_func:ident, $func_name:ident $(,)? $($codec_type:ident, $field:ident),*) => {

        impl_struct_map_codec!(@internal_start $n $name $alias $apply_func $func_name $($codec_type, $field),*);

        #[doc = concat!("Returns a struct [`Codec`] with ", stringify!($n), " field(s).")]
        pub const fn $func_name<T, C1: MapCodec $(, $codec_type: MapCodec)*>(
            field_1: Field<T, C1>,
            $($field: Field<T, $codec_type>,)*
            f: fn(C1::Value $(, $codec_type::Value)*) -> T,
        ) -> $alias<T, C1 $(, $codec_type)*> {
            new_map_codec_codec(
                $name {
                    field_1,
                    $( $field, )*
                    apply_function: f,
                    compressor: OnceLock::new(),
                }
            )
        }
    };

    (expect $n:literal, $name:ident, $alias:ident, $apply_func:ident, $func_name:ident $(,)? $($codec_type:ident, $field:ident),*) => {

        impl_struct_map_codec!(@internal_start $n $name $alias $apply_func $func_name $($codec_type, $field),*);

        #[doc = concat!("Returns a struct [`Codec`] with ", stringify!($n), " field(s).")]
        #[expect(clippy::too_many_arguments)]
        pub const fn $func_name<T, C1: MapCodec $(, $codec_type: MapCodec)*>(
            field_1: Field<T, C1>,
            $($field: Field<T, $codec_type>,)*
            f: fn(C1::Value $(, $codec_type::Value)*) -> T,
        ) -> $alias<T, C1 $(, $codec_type)*> {
            new_map_codec_codec(
                $name {
                    field_1,
                    $( $field, )*
                    apply_function: f,
                    compressor: OnceLock::new(),
                }
            )
        }
    };
}

impl_struct_map_codec!(1, StructMapCodec1, StructCodec1, map, struct_1,);
impl_struct_map_codec!(
    2,
    StructMapCodec2,
    StructCodec2,
    apply_2,
    struct_2,
    C2,
    field_2
);
impl_struct_map_codec!(
    3,
    StructMapCodec3,
    StructCodec3,
    apply_3,
    struct_3,
    C2,
    field_2,
    C3,
    field_3
);
impl_struct_map_codec!(
    4,
    StructMapCodec4,
    StructCodec4,
    apply_4,
    struct_4,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4
);
impl_struct_map_codec!(
    5,
    StructMapCodec5,
    StructCodec5,
    apply_5,
    struct_5,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4,
    C5,
    field_5
);
impl_struct_map_codec!(
    6,
    StructMapCodec6,
    StructCodec6,
    apply_6,
    struct_6,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4,
    C5,
    field_5,
    C6,
    field_6
);
impl_struct_map_codec!(
    expect 7,
    StructMapCodec7,
    StructCodec7,
    apply_7,
    struct_7,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4,
    C5,
    field_5,
    C6,
    field_6,
    C7,
    field_7
);
impl_struct_map_codec!(
    expect 8,
    StructMapCodec8,
    StructCodec8,
    apply_8,
    struct_8,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4,
    C5,
    field_5,
    C6,
    field_6,
    C7,
    field_7,
    C8,
    field_8
);
impl_struct_map_codec!(
    expect 9,
    StructMapCodec9,
    StructCodec9,
    apply_9,
    struct_9,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4,
    C5,
    field_5,
    C6,
    field_6,
    C7,
    field_7,
    C8,
    field_8,
    C9,
    field_9
);
impl_struct_map_codec!(
    expect 10,
    StructMapCodec10,
    StructCodec10,
    apply_10,
    struct_10,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4,
    C5,
    field_5,
    C6,
    field_6,
    C7,
    field_7,
    C8,
    field_8,
    C9,
    field_9,
    C10,
    field_10
);
impl_struct_map_codec!(
    expect 11,
    StructMapCodec11,
    StructCodec11,
    apply_11,
    struct_11,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4,
    C5,
    field_5,
    C6,
    field_6,
    C7,
    field_7,
    C8,
    field_8,
    C9,
    field_9,
    C10,
    field_10,
    C11,
    field_11
);
impl_struct_map_codec!(
    expect 12,
    StructMapCodec12,
    StructCodec12,
    apply_12,
    struct_12,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4,
    C5,
    field_5,
    C6,
    field_6,
    C7,
    field_7,
    C8,
    field_8,
    C9,
    field_9,
    C10,
    field_10,
    C11,
    field_11,
    C12,
    field_12
);
impl_struct_map_codec!(
    expect 13,
    StructMapCodec13,
    StructCodec13,
    apply_13,
    struct_13,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4,
    C5,
    field_5,
    C6,
    field_6,
    C7,
    field_7,
    C8,
    field_8,
    C9,
    field_9,
    C10,
    field_10,
    C11,
    field_11,
    C12,
    field_12,
    C13,
    field_13
);
impl_struct_map_codec!(
    expect 14,
    StructMapCodec14,
    StructCodec14,
    apply_14,
    struct_14,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4,
    C5,
    field_5,
    C6,
    field_6,
    C7,
    field_7,
    C8,
    field_8,
    C9,
    field_9,
    C10,
    field_10,
    C11,
    field_11,
    C12,
    field_12,
    C13,
    field_13,
    C14,
    field_14
);
impl_struct_map_codec!(
    expect 15,
    StructMapCodec15,
    StructCodec15,
    apply_15,
    struct_15,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4,
    C5,
    field_5,
    C6,
    field_6,
    C7,
    field_7,
    C8,
    field_8,
    C9,
    field_9,
    C10,
    field_10,
    C11,
    field_11,
    C12,
    field_12,
    C13,
    field_13,
    C14,
    field_14,
    C15,
    field_15
);
impl_struct_map_codec!(
    expect 16,
    StructMapCodec16,
    StructCodec16,
    apply_16,
    struct_16,
    C2,
    field_2,
    C3,
    field_3,
    C4,
    field_4,
    C5,
    field_5,
    C6,
    field_6,
    C7,
    field_7,
    C8,
    field_8,
    C9,
    field_9,
    C10,
    field_10,
    C11,
    field_11,
    C12,
    field_12,
    C13,
    field_13,
    C14,
    field_14,
    C15,
    field_15,
    C16,
    field_16
);

#[cfg(test)]
mod test {
    use crate::serialization::codec::*;
    use crate::serialization::codecs::list::ListCodec;
    use crate::serialization::codecs::primitive::StringCodec;
    use crate::serialization::coders::{Decoder, Encoder};
    use crate::serialization::json_ops;
    use crate::serialization::struct_codecs::{StructCodec2, StructCodec3, StructCodec5};
    use crate::{assert_decode, struct_codec};
    use serde_json::json;

    #[test]
    fn book_struct() {
        #[derive(Debug, PartialEq)]
        struct Book {
            name: String,
            author: String,
            pages: u32,
        }

        pub static BOOK_CODEC: StructCodec3<
            Book,
            FieldMapCodec<StringCodec>,
            FieldMapCodec<StringCodec>,
            FieldMapCodec<UnsignedIntCodec>,
        > = struct_codec!(
            for_getter(field(&STRING_CODEC, "name"), |book: &Book| &book.name),
            for_getter(field(&STRING_CODEC, "author"), |book: &Book| &book.author),
            for_getter(field(&UNSIGNED_INT_CODEC, "pages"), |book: &Book| &book
                .pages),
            |name, author, pages| Book {
                name,
                author,
                pages
            }
        );

        let object = Book {
            name: "Sample Book".to_string(),
            author: "Sample Author".to_string(),
            pages: 16,
        };

        assert_eq!(
            BOOK_CODEC
                .encode_start(&object, &json_ops::INSTANCE)
                .expect("Could not encode book"),
            json![{
                "name": "Sample Book",
                "author": "Sample Author",
                "pages": 16
            }]
        );

        assert_eq!(BOOK_CODEC.parse(json!({"name": "The Great Gatsby", "author": "F. Scott Fitzgerald", "pages": 180}), &json_ops::INSTANCE).expect("Parsing book object failed"),
                   Book {
                       name: "The Great Gatsby".to_string(),
                       author: "F. Scott Fitzgerald".to_string(),
                       pages: 180
                   }
        );

        assert_decode!(
            BOOK_CODEC,
            json!({"name": "Untitled Book", "pages": 345}),
            &json_ops::INSTANCE,
            is_error
        );
        assert_decode!(
            BOOK_CODEC,
            json!({"name": "Untitled Book 2", "author": "Untitled Author", "pages": "98"}),
            &json_ops::INSTANCE,
            is_error
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn recipe_struct() {
        // A struct for some arbitrary recipe.
        #[derive(Debug, PartialEq)]
        struct Recipe {
            id: String,
            // This must have at least 1 ingredient.
            ingredients: Vec<ItemStack>,
            result: ItemStack,
            crafting_time: u32,
            experience: u32,
        }

        // A struct for storing some items at 1 slot.
        #[derive(Debug, PartialEq)]
        struct ItemStack {
            item: String,
            // Optional field, defaults to 1
            count: u8,
        }

        pub type ItemStackCodec = StructCodec2<
            ItemStack,
            FieldMapCodec<StringCodec>,
            OptionalFieldWithDefaultMapCodec<UnsignedByteCodec>,
        >;
        pub static ITEM_STACK_CODEC: ItemStackCodec = struct_codec!(
            for_getter(field(&STRING_CODEC, "item"), |i: &ItemStack| &i.item),
            for_getter(
                optional_field_with_default(&UNSIGNED_BYTE_CODEC, "count", || 1),
                |i| &i.count
            ),
            |item, count| ItemStack { item, count }
        );

        pub type RecipeCodec = StructCodec5<
            Recipe,
            FieldMapCodec<StringCodec>,
            FieldMapCodec<ListCodec<ItemStackCodec>>,
            FieldMapCodec<ItemStackCodec>,
            FieldMapCodec<UnsignedIntCodec>,
            FieldMapCodec<UnsignedIntCodec>,
        >;
        pub static RECIPE_CODEC: RecipeCodec = struct_codec!(
            for_getter(field(&STRING_CODEC, "id"), |i: &Recipe| &i.id),
            for_getter(
                field(&list(&ITEM_STACK_CODEC, 1, usize::MAX), "ingredients"),
                |i: &Recipe| &i.ingredients
            ),
            for_getter(field(&ITEM_STACK_CODEC, "result"), |i: &Recipe| &i.result),
            for_getter(field(&UNSIGNED_INT_CODEC, "crafting_time"), |i: &Recipe| &i
                .crafting_time),
            for_getter(field(&UNSIGNED_INT_CODEC, "experience"), |i: &Recipe| &i
                .experience),
            |id, ingredients, result, crafting_time, experience| Recipe {
                id,
                ingredients,
                result,
                crafting_time,
                experience
            }
        );

        // Encoding

        let example = Recipe {
            id: String::from("flint_and_steel_recipe"),
            ingredients: vec![
                ItemStack {
                    item: String::from("flint"),
                    count: 1,
                },
                ItemStack {
                    item: String::from("iron_ingot"),
                    count: 1,
                },
            ],
            result: ItemStack {
                item: String::from("flint_and_steel"),
                count: 1,
            },
            crafting_time: 2,
            experience: 5,
        };

        assert_eq!(
            RECIPE_CODEC
                .encode_start(&example, &json_ops::INSTANCE)
                .expect("Encoding panicked"),
            json!(
                {
                    "id": "flint_and_steel_recipe",
                    "ingredients": [
                        // Since the counts of each are 1, the count will be omitted.
                        { "item": "flint" },
                        { "item": "iron_ingot" }
                    ],
                    // Same thing here.
                    "result": { "item": "flint_and_steel" },
                    "crafting_time": 2,
                    "experience": 5
                }
            )
        );

        let example = Recipe {
            id: String::from("combine_air"),
            ingredients: vec![],
            result: ItemStack {
                item: String::from("bigger_air"),
                count: 1,
            },
            crafting_time: 1,
            experience: 1,
        };

        // This should error because there are no ingredients in the recipe.
        assert!(
            RECIPE_CODEC
                .encode_start(&example, &json_ops::INSTANCE)
                .get_message()
                .expect("This DataResult should be an error")
                .starts_with("List is too short")
        );

        // Decoding

        assert_eq!(
            RECIPE_CODEC
                .parse(
                    json!({
                        "id": "orange_dye_recipe",
                        "ingredients": [
                            // The codec will be able to substitute the default
                            // count value for these items, which is 1.
                            { "item": "red_dye" },
                            { "item": "yellow_dye" }
                        ],
                        "result": { "item": "orange_dye", "count": 2 },
                        "crafting_time": 10,
                        "experience": 10
                    }),
                    &json_ops::INSTANCE
                )
                .expect("Decoding panicked"),
            Recipe {
                id: String::from("orange_dye_recipe"),
                ingredients: vec![
                    ItemStack {
                        item: String::from("red_dye"),
                        count: 1
                    },
                    ItemStack {
                        item: String::from("yellow_dye"),
                        count: 1
                    }
                ],
                result: ItemStack {
                    item: String::from("orange_dye"),
                    count: 2
                },
                crafting_time: 10,
                experience: 10,
            }
        );

        assert!(
            RECIPE_CODEC
                .parse(
                    json!({
                        "id": "bedrock_recipe",
                        "ingredients": [
                            { "item": "obsidian", "count": 64 },
                            { "item": "netherite_block", "count": 64 }
                        ],
                        "result": { "item": "bedrock" },
                        "crafting_time": 10000,
                        // Oops, omitted "experience"!
                    }),
                    &json_ops::INSTANCE
                )
                .is_error()
        );
    }
}
