use crate::deserializer::NbtReadHelper;
use crate::{Error, Nbt, NbtCompound, deserializer, serializer};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::io::{Read, Write};

/// Reads a GZipped NBT compound tag from any reader.
///
/// # Arguments
///
/// * `input` - Any type implementing the Read trait containing GZipped NBT data
///
/// # Returns
///
/// A Result containing either the parsed NbtCompound or an Error
pub fn read_gzip_compound_tag(input: impl Read) -> Result<NbtCompound, Error> {
    // Create a GZip decoder and directly chain it to the NBT reader
    let decoder = GzDecoder::new(input);
    let mut reader = NbtReadHelper::new(decoder);

    // Read the NBT data directly from the decoder stream
    let nbt = Nbt::read(&mut reader)?;
    Ok(nbt.root_tag)
}

/// Writes an NBT compound tag with GZip compression.
///
/// This function takes an NbtCompound and writes it as a GZipped byte vector.
///
/// # Arguments
///
/// * `compound` - The NbtCompound to serialize and compress
/// * `output` - Any type implementing the Write trait where the compressed data will be written
///
/// # Returns
///
/// A Result containing either the compressed data as a byte vector or an Error
pub fn write_gzip_compound_tag(compound: &NbtCompound, output: impl Write) -> Result<(), Error> {
    // Create a GZip encoder that writes to the output
    let mut encoder = GzEncoder::new(output, Compression::default());

    // Create an NBT wrapper and write directly to the encoder
    let nbt = Nbt::new(String::new(), compound.clone());
    nbt.write_to_writer(&mut encoder)
        .map_err(Error::Incomplete)?;

    // Finish the encoder to ensure all data is written
    encoder.finish().map_err(Error::Incomplete)?;

    Ok(())
}

/// Convenience function that returns compressed bytes
pub fn write_gzip_compound_tag_to_bytes(compound: &NbtCompound) -> Result<Vec<u8>, Error> {
    let mut buffer = Vec::new();
    write_gzip_compound_tag(compound, &mut buffer)?;
    Ok(buffer)
}

/// Reads a GZipped NBT structure into a Rust type.
///
/// # Arguments
///
/// * `input` - Any type implementing the Read trait containing GZipped NBT data
///
/// # Returns
///
/// A Result containing either the deserialized type or an Error
pub fn from_gzip_bytes<'a, T, R>(input: R) -> Result<T, Error>
where
    T: serde::Deserialize<'a>,
    R: Read,
{
    // Create a GZip decoder and directly use it for deserialization
    let decoder = GzDecoder::new(input);
    deserializer::from_bytes(decoder)
}

/// Writes a Rust type as GZipped NBT to any writer.
///
/// # Arguments
///
/// * `value` - The value to serialize and compress
/// * `output` - Any type implementing the Write trait where the compressed data will be written
///
/// # Returns
///
/// A Result indicating success or an Error
pub fn to_gzip_bytes<T, W>(value: &T, output: W) -> Result<(), Error>
where
    T: serde::Serialize,
    W: Write,
{
    // Create a GZip encoder that writes to the output
    let encoder = GzEncoder::new(output, Compression::default());

    // Serialize directly to the encoder
    serializer::to_bytes(value, encoder)
}

/// Convenience function that returns compressed bytes
pub fn to_gzip_bytes_vec<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: serde::Serialize,
{
    let mut buffer = Vec::new();
    to_gzip_bytes(value, &mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use crate::{
        NbtCompound,
        nbt_compress::{
            from_gzip_bytes, read_gzip_compound_tag, to_gzip_bytes, to_gzip_bytes_vec,
            write_gzip_compound_tag, write_gzip_compound_tag_to_bytes,
        },
        tag::NbtTag,
    };
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Cursor;

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

        // Write to GZip using streaming
        let mut buffer = Vec::new();
        write_gzip_compound_tag(&compound, &mut buffer).expect("Failed to compress compound");

        // Read from GZip using streaming
        let read_compound =
            read_gzip_compound_tag(Cursor::new(&buffer)).expect("Failed to decompress compound");

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
    fn test_gzip_convenience_methods() {
        // Create a test compound
        let mut compound = NbtCompound::new();
        compound.put_int("test_value", 12345);

        // Test convenience method for writing
        let buffer =
            write_gzip_compound_tag_to_bytes(&compound).expect("Failed to compress compound");

        // Test streaming read from the buffer
        let read_compound =
            read_gzip_compound_tag(Cursor::new(buffer)).expect("Failed to decompress compound");

        assert_eq!(read_compound.get_int("test_value"), Some(12345));
    }

    #[test]
    fn test_gzip_empty_compound() {
        let compound = NbtCompound::new();
        let mut buffer = Vec::new();
        write_gzip_compound_tag(&compound, &mut buffer).expect("Failed to compress empty compound");
        let read_compound = read_gzip_compound_tag(Cursor::new(buffer))
            .expect("Failed to decompress empty compound");

        assert_eq!(read_compound.child_tags.len(), 0);
    }

    #[test]
    fn test_gzip_large_compound() {
        let mut compound = NbtCompound::new();

        // Add 1000 integer entries
        for i in 0..1000 {
            compound.put_int(&format!("value_{}", i), i);
        }

        let mut buffer = Vec::new();
        write_gzip_compound_tag(&compound, &mut buffer).expect("Failed to compress large compound");
        let read_compound = read_gzip_compound_tag(Cursor::new(buffer))
            .expect("Failed to decompress large compound");

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

        // Test streaming serialization
        let mut buffer = Vec::new();
        to_gzip_bytes(&test_struct, &mut buffer).expect("Failed to serialize and compress struct");

        // Test streaming deserialization
        let read_struct: TestStruct = from_gzip_bytes(Cursor::new(&buffer))
            .expect("Failed to decompress and deserialize struct");

        assert_eq!(read_struct, test_struct);

        // Also test the convenience method
        let buffer2 =
            to_gzip_bytes_vec(&test_struct).expect("Failed to serialize and compress struct");
        let read_struct2: TestStruct = from_gzip_bytes(Cursor::new(&buffer2))
            .expect("Failed to decompress and deserialize struct");

        assert_eq!(read_struct2, test_struct);
    }

    #[test]
    fn test_gzip_compression_ratio() {
        let mut compound = NbtCompound::new();

        // Create a compound with repetitive data (should compress well)
        for _i in 0..1000 {
            compound.put("repeated_key", NbtTag::String("this is a test string that will be repeated many times to demonstrate compression".to_string()));
        }

        let uncompressed = compound.child_tags.len() * 100; // rough estimate
        let mut buffer = Vec::new();
        write_gzip_compound_tag(&compound, &mut buffer).expect("Failed to compress compound");

        println!("Uncompressed size (est): {} bytes", uncompressed);
        println!("Compressed size: {} bytes", buffer.len());
        println!(
            "Compression ratio: {:.2}x",
            uncompressed as f64 / buffer.len() as f64
        );

        // Just ensure we can read it back - actual compression ratio will vary
        let _ = read_gzip_compound_tag(Cursor::new(buffer)).expect("Failed to decompress compound");
    }

    #[test]
    fn test_gzip_invalid_data() {
        // Try to read from invalid data
        let invalid_data = vec![1, 2, 3, 4, 5]; // Not valid GZip data
        let result = read_gzip_compound_tag(Cursor::new(invalid_data));
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

        let mut buffer = Vec::new();
        to_gzip_bytes(&test_struct, &mut buffer).expect("Failed to serialize and compress");
        let read_struct: ArrayTest =
            from_gzip_bytes(Cursor::new(buffer)).expect("Failed to decompress and deserialize");

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

        let mut buffer = Vec::new();
        to_gzip_bytes(&test_struct, &mut buffer).expect("Failed to serialize and compress");
        let read_struct: MapTest =
            from_gzip_bytes(Cursor::new(buffer)).expect("Failed to decompress and deserialize");

        assert_eq!(read_struct, test_struct);
    }

    #[test]
    fn test_direct_file_io() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test_compound.dat");

        let mut compound = NbtCompound::new();
        compound.put_int("test_value", 42);

        let file = File::create(&file_path).expect("Failed to create temp file");
        write_gzip_compound_tag(&compound, file).expect("Failed to write compound to file");

        let file = File::open(&file_path).expect("Failed to open temp file");
        let read_compound =
            read_gzip_compound_tag(file).expect("Failed to read compound from file");

        assert_eq!(read_compound.get_int("test_value"), Some(42));
    }
}
