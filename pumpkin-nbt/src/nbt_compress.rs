use crate::{Error, Nbt, NbtCompound, deserializer, serializer};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::io::{Read, Write};

/// Reads a GZipped NBT compound tag.
///
/// This function takes a byte slice containing a GZipped NBT compound tag and returns the deserialized compound.
///
/// # Arguments
///
/// * `compressed_data` - A byte slice containing the GZipped NBT data
///
/// # Returns
///
/// A Result containing either the parsed NbtCompound or an Error
pub fn read_gzip_compound_tag(compressed_data: &[u8]) -> Result<NbtCompound, Error> {
    // Create a GZip decoder
    let mut decoder = GzDecoder::new(compressed_data);

    // Decompress the data
    let mut decompressed_data = Vec::new();
    decoder
        .read_to_end(&mut decompressed_data)
        .map_err(Error::Incomplete)?;

    // Read the NBT data
    let nbt = Nbt::read(&mut deserializer::ReadAdaptor::new(&decompressed_data[..]))?;
    Ok(nbt.root_tag)
}

/// Writes an NBT compound tag with GZip compression.
///
/// This function takes an NbtCompound and writes it as a GZipped byte vector.
///
/// # Arguments
///
/// * `compound` - The NbtCompound to serialize and compress
///
/// # Returns
///
/// A Result containing either the compressed data as a byte vector or an Error
pub fn write_gzip_compound_tag(compound: &NbtCompound) -> Result<Vec<u8>, Error> {
    // First serialize the NBT data
    let nbt = Nbt::new(String::new(), compound.clone());
    let serialized = nbt.write();

    // Then compress it with GZip
    let mut compressed_data = Vec::new();
    {
        let mut encoder = GzEncoder::new(&mut compressed_data, Compression::default());
        encoder.write_all(&serialized).map_err(Error::Incomplete)?;
        encoder.finish().map_err(Error::Incomplete)?;
    }

    Ok(compressed_data)
}

/// Reads a GZipped NBT structure.
///
/// This function takes a byte slice containing a GZipped NBT structure and deserializes it into a user-provided type.
///
/// # Arguments
///
/// * `compressed_data` - A byte slice containing the GZipped NBT data
///
/// # Returns
///
/// A Result containing either the parsed structure or an Error
pub fn from_gzip_bytes<'a, T>(compressed_data: &[u8]) -> Result<T, Error>
where
    T: serde::Deserialize<'a>,
{
    // Create a GZip decoder
    let mut decoder = GzDecoder::new(compressed_data);

    // Decompress the data
    let mut decompressed_data = Vec::new();
    decoder
        .read_to_end(&mut decompressed_data)
        .map_err(Error::Incomplete)?;

    // Deserialize the NBT data
    deserializer::from_bytes(&decompressed_data[..])
}

/// Writes a GZipped NBT structure.
///
/// This function takes a serializable structure and writes it as a GZipped byte vector.
///
/// # Arguments
///
/// * `value` - The value to serialize and compress
///
/// # Returns
///
/// A Result containing either the compressed data as a byte vector or an Error
pub fn to_gzip_bytes<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: serde::Serialize,
{
    // First serialize the NBT data
    let mut uncompressed_data = Vec::new();
    serializer::to_bytes(value, &mut uncompressed_data)?;

    // Then compress it with GZip
    let mut compressed_data = Vec::new();
    {
        let mut encoder = GzEncoder::new(&mut compressed_data, Compression::default());
        encoder
            .write_all(&uncompressed_data)
            .map_err(Error::Incomplete)?;
        encoder.finish().map_err(Error::Incomplete)?;
    }

    Ok(compressed_data)
}

#[cfg(test)]
mod tests {
    use crate::{
        NbtCompound,
        nbt_compress::{
            from_gzip_bytes, read_gzip_compound_tag, to_gzip_bytes, write_gzip_compound_tag,
        },
        tag::NbtTag,
    };
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[test]
    fn test_gzip_read_write_compound() {
        // Create a test compound
        let mut compound = NbtCompound::new();
        compound.put_byte("byte_value", 123);
        compound.put_short("short_value", 12345);
        compound.put_int("int_value", 1234567);
        compound.put_long("long_value", 123456789);
        compound.put_float("float_value", 123.456);
        compound.put_double("double_value", 123456.789);
        compound.put_bool("bool_value", true);
        compound.put("string_value", NbtTag::String("test string".to_string()));

        // Create a nested compound
        let mut nested = NbtCompound::new();
        nested.put_int("nested_int", 42);
        compound.put_component("nested_compound", nested);

        // Write to GZip
        let compressed = write_gzip_compound_tag(&compound).expect("Failed to compress compound");

        // Read from GZip
        let read_compound =
            read_gzip_compound_tag(&compressed).expect("Failed to decompress compound");

        // Verify values
        assert_eq!(read_compound.get_byte("byte_value"), Some(123));
        assert_eq!(read_compound.get_short("short_value"), Some(12345));
        assert_eq!(read_compound.get_int("int_value"), Some(1234567));
        assert_eq!(read_compound.get_long("long_value"), Some(123456789));
        assert_eq!(read_compound.get_float("float_value"), Some(123.456));
        assert_eq!(read_compound.get_double("double_value"), Some(123456.789));
        assert_eq!(read_compound.get_bool("bool_value"), Some(true));
        assert_eq!(
            read_compound.get_string("string_value").map(String::as_str),
            Some("test string")
        );

        // Verify nested compound
        if let Some(nested) = read_compound.get_compound("nested_compound") {
            assert_eq!(nested.get_int("nested_int"), Some(42));
        } else {
            panic!("Failed to retrieve nested compound");
        }
    }

    #[test]
    fn test_gzip_empty_compound() {
        let compound = NbtCompound::new();
        let compressed =
            write_gzip_compound_tag(&compound).expect("Failed to compress empty compound");
        let read_compound =
            read_gzip_compound_tag(&compressed).expect("Failed to decompress empty compound");

        assert_eq!(read_compound.child_tags.len(), 0);
    }

    #[test]
    fn test_gzip_large_compound() {
        let mut compound = NbtCompound::new();

        // Add 1000 integer entries
        for i in 0..1000 {
            compound.put_int(&format!("value_{}", i), i);
        }

        let compressed =
            write_gzip_compound_tag(&compound).expect("Failed to compress large compound");
        let read_compound =
            read_gzip_compound_tag(&compressed).expect("Failed to decompress large compound");

        assert_eq!(read_compound.child_tags.len(), 1000);

        // Verify a few entries
        assert_eq!(read_compound.get_int("value_0"), Some(0));
        assert_eq!(read_compound.get_int("value_500"), Some(500));
        assert_eq!(read_compound.get_int("value_999"), Some(999));
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        string_field: String,
        int_field: i32,
        bool_field: bool,
        float_field: f32,
        string_list: Vec<String>,
        nested: NestedStruct,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct NestedStruct {
        value: i64,
        name: String,
    }

    #[test]
    fn test_gzip_serialize_deserialize() {
        let test_struct = TestStruct {
            string_field: "test string".to_string(),
            int_field: 12345,
            bool_field: true,
            float_field: 123.456,
            string_list: vec!["one".to_string(), "two".to_string(), "three".to_string()],
            nested: NestedStruct {
                value: 9876543210,
                name: "nested_test".to_string(),
            },
        };

        // Serialize to GZip
        let compressed =
            to_gzip_bytes(&test_struct).expect("Failed to serialize and compress struct");

        // Deserialize from GZip
        let read_struct: TestStruct =
            from_gzip_bytes(&compressed).expect("Failed to decompress and deserialize struct");

        assert_eq!(read_struct, test_struct);
    }

    #[test]
    fn test_gzip_compression_ratio() {
        let mut compound = NbtCompound::new();

        // Create a compound with repetitive data (should compress well)
        for _i in 0..1000 {
            compound.put("repeated_key", NbtTag::String("this is a test string that will be repeated many times to demonstrate compression".to_string()));
        }

        let uncompressed = compound.child_tags.len() * 100; // rough estimate
        let compressed = write_gzip_compound_tag(&compound).expect("Failed to compress compound");

        println!("Uncompressed size (est): {} bytes", uncompressed);
        println!("Compressed size: {} bytes", compressed.len());
        println!(
            "Compression ratio: {:.2}x",
            uncompressed as f64 / compressed.len() as f64
        );

        // Just ensure we can read it back - actual compression ratio will vary
        let _ = read_gzip_compound_tag(&compressed).expect("Failed to decompress compound");
    }

    #[test]
    fn test_gzip_invalid_data() {
        // Try to read from invalid data
        let invalid_data = vec![1, 2, 3, 4, 5]; // Not valid GZip data
        let result = read_gzip_compound_tag(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_with_arrays() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct ArrayTest {
            byte_array: Vec<u8>,
            int_array: Vec<i32>,
            string_array: Vec<String>,
        }

        let test_struct = ArrayTest {
            byte_array: vec![1, 2, 3, 4, 5],
            int_array: vec![100, 200, 300, 400, 500],
            string_array: vec!["one".to_string(), "two".to_string(), "three".to_string()],
        };

        let compressed = to_gzip_bytes(&test_struct).expect("Failed to serialize and compress");
        let read_struct: ArrayTest =
            from_gzip_bytes(&compressed).expect("Failed to decompress and deserialize");

        assert_eq!(read_struct, test_struct);
    }

    #[test]
    fn test_roundtrip_with_map() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct MapTest {
            string_map: HashMap<String, String>,
            int_map: HashMap<String, i32>,
        }

        let mut string_map = HashMap::new();
        string_map.insert("key1".to_string(), "value1".to_string());
        string_map.insert("key2".to_string(), "value2".to_string());

        let mut int_map = HashMap::new();
        int_map.insert("one".to_string(), 1);
        int_map.insert("two".to_string(), 2);

        let test_struct = MapTest {
            string_map,
            int_map,
        };

        let compressed = to_gzip_bytes(&test_struct).expect("Failed to serialize and compress");
        let read_struct: MapTest =
            from_gzip_bytes(&compressed).expect("Failed to decompress and deserialize");

        assert_eq!(read_struct, test_struct);
    }
}
