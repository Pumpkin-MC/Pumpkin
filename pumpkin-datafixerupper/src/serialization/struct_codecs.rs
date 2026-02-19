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

/// A single field object to build a struct codec, which either takes an *owned* or *borrowed* [`MapCodec`] and a getter.
///
/// - `T` is the composite type to get from.
/// - `C` is the [`MapCodec`] for serializing/deserializing the field.
pub enum Field<T, C: MapCodec + 'static> {
    Owned(C, fn(&T) -> &C::Value),
    Borrowed(&'static C, fn(&T) -> &C::Value),
}

impl<T, C: MapCodec + 'static> Field<T, C> {
    fn getter(&self) -> &fn(&T) -> &C::Value {
        match self {
            Self::Owned(_, g) => g,
            Self::Borrowed(_, g) => g,
        }
    }

    const fn map_codec(&self) -> &C {
        match self {
            Self::Owned(c, _) => c,
            Self::Borrowed(c, _) => c,
        }
    }
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
            apply_function: fn(C1::Value $(, $codec_type::Value)*) -> T
        }

        impl<T, C1: MapCodec $(, $codec_type: MapCodec)* > HasValue for $name<T, C1 $(, $codec_type)*> {
            type Value = T;
        }

        impl<T, C1: MapCodec $(, $codec_type: MapCodec)* > Keyable for $name<T, C1 $(, $codec_type)*> {
            #[allow(unused_mut)]
            fn keys(&self) -> Vec<String> {
                let mut keys = self.field_1.map_codec().keys();
                $( keys.extend(self.$field.map_codec().keys()); )*
                keys
            }
        }

        impl<T, C1: MapCodec $(, $codec_type: MapCodec)* > CompressorHolder for $name<T, C1 $(, $codec_type)*> {
            impl_compressor!();
        }

        impl<T, C1: MapCodec $(, $codec_type: MapCodec)* > MapEncoder for $name<T, C1 $(, $codec_type)*> {
            #[allow(clippy::let_and_return)]
            fn encode<U: Display + PartialEq + Clone, B: StructBuilder<Value = U>>(&self, input: &Self::Value, ops: &'static impl DynamicOps<Value=U>, prefix: B) -> B {
                let prefix =
                    self.field_1.map_codec()
                        .encode((self.field_1.getter())(input), ops, prefix);
                $(
                    let prefix =
                    self.$field.map_codec()
                        .encode((self.$field.getter())(input), ops, prefix);
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
                self.field_1.map_codec().decode(input, ops).$apply_func(
                    self.apply_function,
                    $( self.$field.map_codec().decode(input, ops), )*
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
                    apply_function: f
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
                    apply_function: f
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
    use crate::serialization::codecs::validated::ValidatedCodec;
    use crate::serialization::coders::{Decoder, Encoder};
    use crate::serialization::json_ops;
    use crate::serialization::map_codec::for_getter;
    use crate::serialization::struct_codecs::StructCodec3;
    use crate::{assert_decode, struct_codec};
    use serde_json::json;

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct Book {
        name: String,
        author: String,
        pages: u32,
    }

    pub type BookCodec = StructCodec3<
        Book,
        FieldMapCodec<StringCodec>,
        FieldMapCodec<StringCodec>,
        FieldMapCodec<UintCodec>,
    >;

    pub static BOOK_CODEC: BookCodec = struct_codec!(
        for_getter(field(&STRING_CODEC, "name"), |book: &Book| &book.name),
        for_getter(field(&STRING_CODEC, "author"), |book: &Book| &book.author),
        for_getter(field(&UINT_CODEC, "pages"), |book: &Book| &book.pages),
        |name, author, pages| Book {
            name,
            author,
            pages
        }
    );

    #[test]
    fn book_struct() {
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
    fn bookshelf_struct() {
        // A struct for a bookshelf.
        #[derive(Debug, PartialEq)]
        struct Bookshelf {
            id: u32,
            // Optional, defaults to no books.
            books: Vec<Book>,
            capacity: u32,
        }

        pub type BookshelfCodec = ValidatedCodec<
            StructCodec3<
                Bookshelf,
                FieldMapCodec<UintCodec>,
                DefaultedFieldCodec<ListCodec<BookCodec>>,
                FieldMapCodec<UintCodec>,
            >,
        >;
        pub static BOOKSHELF_CODEC: BookshelfCodec = validate(
            &struct_codec!(
                for_getter(field(&UINT_CODEC, "id"), |b: &Bookshelf| &b.id),
                for_getter(
                    optional_field_with_default(&unbounded_list(&BOOK_CODEC), "books", Vec::new),
                    |b: &Bookshelf| &b.books
                ),
                for_getter(field(&UINT_CODEC, "capacity"), |b: &Bookshelf| &b.capacity),
                |id, books, capacity| Bookshelf {
                    id,
                    books,
                    capacity
                }
            ),
            |b| {
                // The number of books on the bookshelf must be less than or equal to its capacity.
                if b.books.len() <= b.capacity as usize {
                    Ok(())
                } else {
                    Err(format!(
                        "Bookshelf cannot have {} books because its capacity is {}",
                        b.books.len(),
                        b.capacity
                    ))
                }
            },
        );

        let example = Bookshelf {
            id: 1234,
            books: vec![
                Book {
                    name: "Charlie and the Chocolate Factory".to_string(),
                    author: "Roald Dahl".to_string(),
                    pages: 192,
                },
                Book {
                    name: "Infinibook".to_string(),
                    author: "Infiniauthor".to_string(),
                    pages: 1_000_000,
                },
            ],
            capacity: 2,
        };

        assert_eq!(
            BOOKSHELF_CODEC
                .encode_start(&example, &json_ops::INSTANCE)
                .expect("Could not encode bookshelf"),
            json![{
                "id": 1234,
                "capacity": 2,
                "books": [
                    {
                        "name": "Charlie and the Chocolate Factory",
                        "author": "Roald Dahl",
                        "pages": 192,
                    },
                    {
                        "name": "Infinibook",
                        "author": "Infiniauthor",
                        "pages": 1_000_000,
                    }
                ]
            }]
        );

        let example = Bookshelf {
            id: 5678,
            books: vec![
                Book {
                    name: "The Lord of the Rings".to_string(),
                    author: "J.R.R. Tolkien".to_string(),
                    pages: 1150,
                },
                Book {
                    name: "Sherlock Holmes".to_string(),
                    author: "Arthur Conan Doyle".to_string(),
                    pages: 1320,
                },
                Book {
                    name: "Empty Book".to_string(),
                    author: String::new(),
                    pages: 0,
                },
            ],
            capacity: 2,
        };

        assert!(
            BOOKSHELF_CODEC
                .encode_start(&example, &json_ops::INSTANCE)
                // We should get an error because the bookshelf cannot handle
                // more than 2 books.
                .get_message()
                .expect("Encoding bookshelf here should be an error")
                .starts_with("Bookshelf cannot have")
        );

        assert_decode!(
            BOOKSHELF_CODEC,
            json!({"id": 36, "capacity": 6, "books": [
                {"name": "Book A", "author": "Author A", "pages": 10},
                {"name": "Book B", "author": "Author B", "pages": 20},
                {"name": "Book C", "author": "Author C", "pages": 30},
                {"name": "Book D", "author": "Author D", "pages": 40},
                {"name": "Book E", "author": "Author E", "pages": 50}
            ]}),
            &json_ops::INSTANCE,
            is_success
        );

        assert_decode!(
            BOOKSHELF_CODEC,
            json!({"id": 93273, "capacity": 4, "books": [
                {"name": "Book 1", "author": "Author 1", "pages": 100},
                {"name": "Book 2", "author": "Author 2", "pages": 200},
                {"name": "Book 3", "author": "Author 3", "pages": 300},
                {"name": "Book 4", "author": "Author 4", "pages": 400},
                // This should fail because 5 > 4.
                {"name": "Book 5", "author": "Author 5", "pages": 500}
            ]}),
            &json_ops::INSTANCE,
            is_error
        );

        assert_decode!(
            BOOKSHELF_CODEC,
            // This will work because "books" is an optional field.
            json!({"id": 254, "capacity": 10}),
            &json_ops::INSTANCE,
            is_success
        );

        assert_decode!(
            BOOKSHELF_CODEC,
            // This will not work because "books" expects an array.
            json!({"id": 6252, "capacity": 1, "books": {"name": "A Tale of Two Cities", "author": "Charles Dickens", "pages": 480}}),
            &json_ops::INSTANCE,
            is_error
        );

        assert_decode!(
            BOOKSHELF_CODEC,
            json!({"id": 6253, "capacity": 1, "books": [{"name": "A Tale of Two Cities", "author": "Charles Dickens"}]}),
            &json_ops::INSTANCE,
            is_error
        );
    }
}
