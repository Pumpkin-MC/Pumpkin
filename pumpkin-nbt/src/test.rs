use std::io::Cursor;

use crate::Error;
use crate::compound::NbtCompound;
use crate::deserializer::from_bytes;
use crate::deserializer::from_bytes_bedrock;
use crate::nbt_byte_array;
use crate::nbt_int_array;
use crate::nbt_long_array;
use crate::serializer::NbtWriteHelperBedrock;
use crate::serializer::to_bytes_bedrock;
use crate::serializer::to_bytes_named;
use crate::serializer::to_bytes_named_bedrock;
use crate::serializer::{NbtWriteHelperJava, to_bytes};
use crate::tag::NbtTag;
use crate::{deserializer::from_bytes_unnamed, serializer::to_bytes_unnamed};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Test {
    byte: i8,
    short: i16,
    int: i32,
    long: i64,
    float: f32,
    string: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct BorrowedTest<'a> {
    byte: i8,
    string: &'a str,
}

#[test]
fn zero_alloc_from_slice() {
    let test = Test {
        byte: 123,
        short: 1342,
        int: 4313,
        long: 34,
        float: 1.00,
        string: "Zero alloc NBT".to_string(),
    };

    let mut bytes = Vec::new();
    to_bytes_unnamed(&test, &mut bytes).unwrap();
    let borrowed: BorrowedTest = crate::from_slice_unnamed(&bytes).unwrap();

    assert_eq!(borrowed.byte, 123);
    assert_eq!(borrowed.string, "Zero alloc NBT");
}

#[test]
fn simple_ser_de_unnamed() {
    let test = Test {
        byte: 123,
        short: 1342,
        int: 4313,
        long: 34,
        float: 1.00,
        string: "Hello test".to_string(),
    };

    let mut bytes = Vec::new();
    to_bytes_unnamed(&test, &mut bytes).unwrap();
    let recreated_struct: Test = from_bytes_unnamed(Cursor::new(bytes)).unwrap();

    assert_eq!(test, recreated_struct);
}

#[test]
fn simple_ser_de_unnamed_bedrock() {
    let test = Test {
        byte: 123,
        short: 1342,
        int: 4313,
        long: 34,
        float: 1.00,
        string: "Hello test".to_string(),
    };

    // Bedrock doesn't actually use unnamed NBT (AFAIK). `to_bytes_bedrock` actually encodes empty name.
    let mut bytes = Vec::new();
    to_bytes_bedrock(&test, &mut bytes).unwrap();
    let recreated_struct: Test = from_bytes_bedrock(Cursor::new(bytes)).unwrap();

    assert_eq!(test, recreated_struct);
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[expect(clippy::struct_field_names)]
struct TestArray {
    #[serde(serialize_with = "nbt_byte_array")]
    byte_array: Vec<u8>,
    #[serde(serialize_with = "nbt_int_array")]
    int_array: Vec<i32>,
    #[serde(serialize_with = "nbt_long_array")]
    long_array: Vec<i64>,
}

#[test]
fn simple_ser_de_array() {
    let test = TestArray {
        byte_array: vec![0, 3, 2],
        int_array: vec![13, 1321, 2],
        long_array: vec![1, 0, 200301, 1],
    };

    let mut bytes = Vec::new();
    to_bytes_unnamed(&test, &mut bytes).unwrap();
    let recreated_struct: TestArray = from_bytes_unnamed(Cursor::new(bytes)).unwrap();

    assert_eq!(test, recreated_struct);
}

#[test]
fn simple_ser_de_array_bedrock() {
    let test = TestArray {
        byte_array: vec![0, 3, 2],
        int_array: vec![13, 1321, 2],
        long_array: vec![1, 0, 200301, 1],
    };

    let mut bytes = Vec::new();
    to_bytes_bedrock(&test, &mut bytes).unwrap();
    let recreated_struct: TestArray = from_bytes_bedrock(Cursor::new(bytes)).unwrap();

    assert_eq!(test, recreated_struct);
}

#[test]
fn simple_ser_de_named() {
    let name = String::from("Test");
    let test = Test {
        byte: 123,
        short: 1342,
        int: 4313,
        long: 34,
        float: 1.00,
        string: "Hello test".to_string(),
    };

    let mut bytes = Vec::new();
    to_bytes_named(&test, name, &mut bytes).unwrap();
    let recreated_struct: Test = from_bytes(Cursor::new(bytes)).unwrap();

    assert_eq!(test, recreated_struct);
}

#[test]
fn simple_ser_de_named_bedrock() {
    let name = String::from("Test");
    let test = Test {
        byte: 123,
        short: 1342,
        int: 4313,
        long: 34,
        float: 1.00,
        string: "Hello test".to_string(),
    };

    let mut bytes = Vec::new();
    to_bytes_named_bedrock(&test, name, &mut bytes).unwrap();
    let recreated_struct: Test = from_bytes_bedrock(Cursor::new(bytes)).unwrap();

    assert_eq!(test, recreated_struct);
}

#[test]
fn simple_ser_de_array_named() {
    let name = String::from("Test");
    let test = TestArray {
        byte_array: vec![0, 3, 2],
        int_array: vec![13, 1321, 2],
        long_array: vec![1, 0, 200301, 1],
    };

    let mut bytes = Vec::new();
    to_bytes_named(&test, name, &mut bytes).unwrap();
    let recreated_struct: TestArray = from_bytes(Cursor::new(bytes)).unwrap();

    assert_eq!(test, recreated_struct);
}

#[test]
fn simple_ser_de_array_named_bedrock() {
    let name = String::from("Test");
    let test = TestArray {
        byte_array: vec![0, 3, 2],
        int_array: vec![13, 1321, 2],
        long_array: vec![1, 0, 200301, 1],
    };

    let mut bytes = Vec::new();
    to_bytes_named_bedrock(&test, name, &mut bytes).unwrap();
    let recreated_struct: TestArray = from_bytes_bedrock(Cursor::new(bytes)).unwrap();

    assert_eq!(test, recreated_struct);
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Egg {
    food: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Breakfast {
    food: Egg,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct TestList {
    option: Option<Egg>,
    nested_compound: Breakfast,
    compounds: Vec<Test>,
    list_string: Vec<String>,
    empty: Vec<Test>,
}

#[test]
fn list() {
    let test1 = Test {
        byte: 123,
        short: 1342,
        int: 4313,
        long: 34,
        float: 1.00,
        string: "Hello test".to_string(),
    };

    let test2 = Test {
        byte: 13,
        short: 342,
        int: -4313,
        long: -132334,
        float: -69.420,
        string: "Hello compounds".to_string(),
    };

    let list_compound = TestList {
        option: Some(Egg {
            food: "Skibid".to_string(),
        }),
        nested_compound: Breakfast {
            food: Egg {
                food: "Over easy".to_string(),
            },
        },
        compounds: vec![test1, test2],
        list_string: vec![String::new(), "abcbcbcbbc".to_string()],
        empty: vec![],
    };

    let mut bytes = Vec::new();
    to_bytes_unnamed(&list_compound, &mut bytes).unwrap();
    let recreated_struct: TestList = from_bytes_unnamed(Cursor::new(bytes)).unwrap();
    assert_eq!(list_compound, recreated_struct);
}

#[test]
fn list_bedrock() {
    let test1 = Test {
        byte: 123,
        short: 1342,
        int: 4313,
        long: 34,
        float: 1.00,
        string: "Hello test".to_string(),
    };

    let test2 = Test {
        byte: 13,
        short: 342,
        int: -4313,
        long: -132334,
        float: -69.420,
        string: "Hello compounds".to_string(),
    };

    let list_compound = TestList {
        option: Some(Egg {
            food: "Skibid".to_string(),
        }),
        nested_compound: Breakfast {
            food: Egg {
                food: "Over easy".to_string(),
            },
        },
        compounds: vec![test1, test2],
        list_string: vec![String::new(), "abcbcbcbbc".to_string()],
        empty: vec![],
    };

    let mut bytes = Vec::new();
    to_bytes_bedrock(&list_compound, &mut bytes).unwrap();
    let recreated_struct: TestList = from_bytes_bedrock(Cursor::new(bytes)).unwrap();
    assert_eq!(list_compound, recreated_struct);
}

#[test]
fn list_named() {
    let test1 = Test {
        byte: 123,
        short: 1342,
        int: 4313,
        long: 34,
        float: 1.00,
        string: "Hello test".to_string(),
    };

    let test2 = Test {
        byte: 13,
        short: 342,
        int: -4313,
        long: -132334,
        float: -69.420,
        string: "Hello compounds".to_string(),
    };

    let list_compound = TestList {
        option: None,
        nested_compound: Breakfast {
            food: Egg {
                food: "Over easy".to_string(),
            },
        },
        compounds: vec![test1, test2],
        list_string: vec![String::new(), "abcbcbcbbc".to_string()],
        empty: vec![],
    };

    let mut bytes = Vec::new();
    to_bytes_named(&list_compound, "a".to_string(), &mut bytes).unwrap();
    let recreated_struct: TestList = from_bytes(Cursor::new(bytes)).unwrap();
    assert_eq!(list_compound, recreated_struct);
}

#[test]
fn list_named_bedrock() {
    let test1 = Test {
        byte: 123,
        short: 1342,
        int: 4313,
        long: 34,
        float: 1.00,
        string: "Hello test".to_string(),
    };

    let test2 = Test {
        byte: 13,
        short: 342,
        int: -4313,
        long: -132334,
        float: -69.420,
        string: "Hello compounds".to_string(),
    };

    let list_compound = TestList {
        option: None,
        nested_compound: Breakfast {
            food: Egg {
                food: "Over easy".to_string(),
            },
        },
        compounds: vec![test1, test2],
        list_string: vec![String::new(), "abcbcbcbbc".to_string()],
        empty: vec![],
    };

    let mut bytes = Vec::new();
    to_bytes_named_bedrock(&list_compound, "a".to_string(), &mut bytes).unwrap();
    let recreated_struct: TestList = from_bytes_bedrock(Cursor::new(bytes)).unwrap();
    assert_eq!(list_compound, recreated_struct);
}

#[test]
fn wrapper_compound_lists() {
    let mut vec: Vec<NbtTag> = Vec::new();

    // These tags will be wrapped during serialization.
    vec.push(NbtTag::Int(-1823));
    vec.push(NbtTag::Int(123));
    vec.push(NbtTag::String("Not an int".into()));
    vec.push(NbtTag::Byte(2));

    // This compound will not, since the list is already a list of compound tags.
    // This compound cannot be unwrapped in any way, so it is preserved
    // on deserialization.
    vec.push(NbtTag::Compound({
        let mut compound = NbtCompound::new();
        compound.put_short("example", 1234);
        compound
    }));

    // This wrapper compound will be wrapped because we want to preserve the
    // original data during deserialization.
    //
    // Suppose we had {"": `tag`}. If we didn't wrap this, on deserialization,
    // we would get `tag`, which doesn't match the serialized compound tag.
    // Therefore, we must wrap it and serialize {"": {"": `tag`}}.
    // Then on deserialization, we get {"": `tag`}, which matches what we wanted
    // to serialize in the first place.
    //
    // This compound represents {"": 1L}.
    vec.push(NbtTag::Compound({
        let mut compound = NbtCompound::new();
        compound.put_long("", 1);
        compound
    }));

    let expected_bytes = [
        0x09, // List type
        0x0A, // This list is a compound tag list
        0x00, 0x00, 0x00, 0x06, // This list has 6 elements.
        // Now for parsing each compound tag:
        0x03, // Int type
        0x00, 0x00, // Empty key
        0xFF, 0xFF, 0xF8, 0xE1, // -1823
        0x00, // End
        0x03, // Int type
        0x00, 0x00, // Empty key
        0x00, 0x00, 0x00, 0x7B, // 123
        0x00, // End
        0x08, // String type
        0x00, 0x00, // Empty key
        0x00, 0x0A, // The string is 10 characters long.
        0x4E, 0x6F, 0x74, 0x20, 0x61, 0x6E, 0x20, 0x69, 0x6E, 0x74, // "Not an int"
        0x00, // End
        0x01, // Byte type
        0x00, 0x00, // Empty key
        0x02, // 2b
        0x00, // End
        // For the first (unwrapped) compound:
        0x02, // Short type
        0x00, 0x07, // The key is 7 characters long.
        0x65, 0x78, 0x61, 0x6D, 0x70, 0x6C, 0x65, // "example"
        0x04, 0xD2, // 1234
        0x00, // End
        // For the second (wrapped) wrapper compound:
        0x0A, // Compound type
        0x00, 0x00, // Empty key
        0x04, // Long type
        0x00, 0x00, // Empty key
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // 1L
        0x00, // End
        0x00, // End
    ];

    let mut bytes = Vec::new();
    let mut write_adaptor = NbtWriteHelperJava::new(&mut bytes);
    NbtTag::List(vec)
        .serialize(&mut write_adaptor)
        .expect("Expected serialization to succeed");

    assert_eq!(bytes, expected_bytes);
}

#[test]
fn wrapper_compound_lists_bedrock() {
    let mut vec: Vec<NbtTag> = Vec::new();

    // These tags will be wrapped during serialization.
    vec.push(NbtTag::Int(-1823));
    vec.push(NbtTag::Int(123));
    vec.push(NbtTag::String("Not an int".into()));
    vec.push(NbtTag::Byte(2));

    // This compound will not, since the list is already a list of compound tags.
    // This compound cannot be unwrapped in any way, so it is preserved
    // on deserialization.
    vec.push(NbtTag::Compound({
        let mut compound = NbtCompound::new();
        compound.put_short("example", 1234);
        compound
    }));

    // This wrapper compound will be wrapped because we want to preserve the
    // original data during deserialization.
    //
    // Suppose we had {"": `tag`}. If we didn't wrap this, on deserialization,
    // we would get `tag`, which doesn't match the serialized compound tag.
    // Therefore, we must wrap it and serialize {"": {"": `tag`}}.
    // Then on deserialization, we get {"": `tag`}, which matches what we wanted
    // to serialize in the first place.
    //
    // This compound represents {"": 1L}.
    vec.push(NbtTag::Compound({
        let mut compound = NbtCompound::new();
        compound.put_long("", 1);
        compound
    }));

    let expected_bytes = [
        0x09, // List type
        0x0A, // This list is a compound tag list
        0xC,  // This list has 6 elements.
        // Now for parsing each compound tag:
        0x03, // Int type
        0x00, // Empty key
        0xBD, 0x1C, // -1823
        0x00, // End
        0x03, // Int type
        0x00, // Empty key
        0xF6, 0x01, // 123
        0x00, // End
        0x08, // String type
        0x00, // Empty key
        0x0A, // The string is 10 characters long.
        0x4E, 0x6F, 0x74, 0x20, 0x61, 0x6E, 0x20, 0x69, 0x6E, 0x74, // "Not an int"
        0x00, // End
        0x01, // Byte type
        0x00, // Empty key
        0x02, // 2b
        0x00, // End
        // For the first (unwrapped) compound:
        0x02, // Short type
        0x07, // The key is 7 characters long.
        0x65, 0x78, 0x61, 0x6D, 0x70, 0x6C, 0x65, // "example"
        0xD2, 0x04, // 1234
        0x00, // End
        // For the second (wrapped) wrapper compound:
        0x0A, // Compound type
        0x00, // Empty key
        0x04, // Long type
        0x00, // Empty key
        0x02, // 1L
        0x00, // End
        0x00, // End
    ];

    let mut bytes = Vec::new();
    let mut write_adaptor = NbtWriteHelperBedrock::new(&mut bytes);
    NbtTag::List(vec)
        .serialize(&mut write_adaptor)
        .expect("Expected serialization to succeed");

    assert_eq!(bytes, expected_bytes);
}

#[test]
fn nbt_arrays() {
    #[derive(Serialize)]
    struct Tagged {
        #[serde(serialize_with = "nbt_long_array")]
        l: [i64; 1],
        #[serde(serialize_with = "nbt_int_array")]
        i: [i32; 1],
        #[serde(serialize_with = "nbt_byte_array")]
        b: [u8; 1],
    }
    #[derive(Serialize)]
    struct NotTagged {
        l: [i64; 1],
        i: [i32; 1],
        b: [u8; 1],
    }

    let value = Tagged {
        l: [0],
        i: [0],
        b: [0],
    };
    let expected_bytes = [
        0x0A, // Component Tag
        0x00, 0x00, // Empty root name
        0x0C, // Long Array Type
        0x00, 0x01, // Key length
        0x6C, // Key (l)
        0x00, 0x00, 0x00, 0x01, // Array Length
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Value(s)
        0x0B, // Int Array Tag
        0x00, 0x01, // Key length
        0x69, // Key (i)
        0x00, 0x00, 0x00, 0x01, // Array Length
        0x00, 0x00, 0x00, 0x00, // Value(s)
        0x07, // Byte Array Tag
        0x00, 0x01, // Key length
        0x62, // Key (b)
        0x00, 0x00, 0x00, 0x01, // Array Length
        0x00, // Value(s)
        0x00, // End Tag
    ];

    let mut bytes = Vec::new();
    to_bytes(&value, &mut bytes).unwrap();
    assert_eq!(bytes, expected_bytes);

    let value = NotTagged {
        l: [0],
        i: [0],
        b: [0],
    };
    let expected_bytes = [
        0x0A, // Component Tag
        0x00, 0x00, // Empty root name
        0x09, // List Tag
        0x00, 0x01, // Key length
        0x6C, // Key (l)
        0x04, // Array Type
        0x00, 0x00, 0x00, 0x01, // Array Length
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Value(s)
        0x09, // List Tag
        0x00, 0x01, // Key length
        0x69, // Key (i)
        0x03, // Array Type
        0x00, 0x00, 0x00, 0x01, // Array Length
        0x00, 0x00, 0x00, 0x00, // Value(s)
        0x09, // List Tag
        0x00, 0x01, // Key length
        0x62, // Key (b)
        0x01, // Array Type
        0x00, 0x00, 0x00, 0x01, // Array Length
        0x00, // Value(s)
        0x00, // End Tag
    ];

    let mut bytes = Vec::new();
    to_bytes(&value, &mut bytes).unwrap();
    assert_eq!(bytes, expected_bytes);
}

#[test]
fn nbt_arrays_bedrock() {
    #[derive(Serialize)]
    struct Tagged {
        #[serde(serialize_with = "nbt_long_array")]
        l: [i64; 1],
        #[serde(serialize_with = "nbt_int_array")]
        i: [i32; 1],
        #[serde(serialize_with = "nbt_byte_array")]
        b: [u8; 1],
    }
    #[derive(Serialize)]
    struct NotTagged {
        l: [i64; 1],
        i: [i32; 1],
        b: [u8; 1],
    }

    let value = Tagged {
        l: [0],
        i: [0],
        b: [0],
    };
    let expected_bytes = [
        0x0A, // Component Tag
        0x00, // Empty root name
        0x0C, // Long Array Type
        0x01, // Key length
        0x6C, // Key (l)
        0x02, // Array Length
        0x00, // Value(s)
        0x0B, // Int Array Tag
        0x01, // Key length
        0x69, // Key (i)
        0x02, // Array Length
        0x00, // Value(s)
        0x07, // Byte Array Tag
        0x01, // Key length
        0x62, // Key (b)
        0x02, // Array Length
        0x00, // Value(s)
        0x00, // End Tag
    ];

    let mut bytes = Vec::new();
    to_bytes_bedrock(&value, &mut bytes).unwrap();
    assert_eq!(bytes, expected_bytes);

    let value = NotTagged {
        l: [0],
        i: [0],
        b: [0],
    };
    let expected_bytes = [
        0x0A, // Component Tag
        0x00, // Empty root name
        0x09, // List Tag
        0x01, // Key length
        0x6C, // Key (l)
        0x04, // Array Type
        0x02, // Array Length
        0x00, // Value(s)
        0x09, // List Tag
        0x01, // Key length
        0x69, // Key (i)
        0x03, // Array Type
        0x02, // Array Length
        0x00, // Value(s)
        0x09, // List Tag
        0x01, // Key length
        0x62, // Key (b)
        0x01, // Array Type
        0x02, // Array Length
        0x00, // Value(s)
        0x00, // End Tag
    ];

    let mut bytes = Vec::new();
    to_bytes_bedrock(&value, &mut bytes).unwrap();
    assert_eq!(bytes, expected_bytes);
}

#[test]
fn tuple_fail() {
    #[derive(Serialize)]
    struct BadData {
        x: (i32, i64),
    }

    let value = BadData { x: (0, 0) };
    let mut bytes = Vec::new();
    let err = to_bytes(&value, &mut bytes);

    match err {
        Err(Error::SerdeError(_)) => (),
        _ => panic!("Expected to fail serialization!"),
    }
}

#[test]
fn tuple_fail_bedrock() {
    #[derive(Serialize)]
    struct BadData {
        x: (i32, i64),
    }

    let value = BadData { x: (0, 0) };
    let mut bytes = Vec::new();
    let err = to_bytes_bedrock(&value, &mut bytes);

    match err {
        Err(Error::SerdeError(_)) => (),
        _ => panic!("Expected to fail serialization!"),
    }
}

#[test]
fn tuple_ok() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct GoodData {
        x: (i32, i32),
    }

    let value = GoodData { x: (1, 2) };
    let mut bytes = Vec::new();
    to_bytes(&value, &mut bytes).unwrap();

    let reconstructed = from_bytes(Cursor::new(bytes)).unwrap();
    assert_eq!(value, reconstructed);
}

#[test]
fn tuple_ok_bedrock() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct GoodData {
        x: (i32, i32),
    }

    let value = GoodData { x: (1, 2) };
    let mut bytes = Vec::new();
    to_bytes_bedrock(&value, &mut bytes).unwrap();

    let reconstructed = from_bytes_bedrock(Cursor::new(bytes)).unwrap();
    assert_eq!(value, reconstructed);
}

// TODO: More robust tests
