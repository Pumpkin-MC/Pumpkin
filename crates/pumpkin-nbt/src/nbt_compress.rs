//! Helpers for reading and writing gzip-compressed NBT data.

use crate::deserializer::NbtReadHelperJava;
use crate::{Error, Nbt, NbtCompound};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::io::{Cursor, Read, Seek, Write};

/// Reads a gzip-compressed, named NBT compound from a seekable reader.
///
/// Decompressed data is limited to 64 MiB.
pub fn read_gzip_compound_tag(input: impl Read + Seek) -> Result<NbtCompound, Error> {
    // Create a GZip decoder and directly chain it to the NBT reader
    let mut decoder = GzDecoder::new(input).take(64 * 1024 * 1024); // 64 MB limit
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf).map_err(Error::Incomplete)?;
    let mut reader = NbtReadHelperJava::new(Cursor::new(buf));

    // Read the NBT data directly from the decoder stream
    let nbt = Nbt::read(&mut reader)?;
    Ok(nbt.root_tag)
}

/// Writes a named NBT compound with gzip compression.
///
/// The root name is written as an empty string.
pub fn write_gzip_compound_tag(compound: NbtCompound, output: impl Write) -> Result<(), Error> {
    // Create a GZip encoder that writes to the output
    let mut encoder = GzEncoder::new(output, Compression::default());

    // Create an NBT wrapper and write directly to the encoder
    let nbt = Nbt::new(String::new(), compound);
    nbt.write_to_writer(&mut encoder)
        .map_err(Error::Incomplete)?;

    // Finish the encoder to ensure all data is written
    encoder.finish().map_err(Error::Incomplete)?;

    Ok(())
}

/// Serializes a named NBT compound into a gzip-compressed byte vector.
///
/// The root name is written as an empty string.
pub fn write_gzip_compound_tag_to_bytes(compound: NbtCompound) -> Result<Vec<u8>, Error> {
    let mut buffer = Vec::new();
    write_gzip_compound_tag(compound, &mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use crate::{
        NbtCompound,
        nbt_compress::{
            read_gzip_compound_tag, write_gzip_compound_tag, write_gzip_compound_tag_to_bytes,
        },
        tag::NbtTag,
    };
    use std::fs::File;
    use std::io::Cursor;

    #[test]
    fn gzip_read_write_compound() {
        // Create a test compound
        let mut compound = NbtCompound::new();
        compound.put_byte("byte_value", 123);
        compound.put_short("short_value", 12345);
        compound.put_int("int_value", 1234567);
        compound.put_long("long_value", 123456789);
        compound.put_float("float_value", 123.456);
        compound.put_double("double_value", 123456.789);
        compound.put("bool_value", true);
        compound.put("string_value", NbtTag::String("test string".into()));

        // Create a nested compound
        let mut nested = NbtCompound::new();
        nested.put_int("nested_int", 42);
        compound.put_compound("nested_compound", nested);

        // Write to GZip using streaming
        let mut buffer = Vec::new();
        write_gzip_compound_tag(compound, &mut buffer).expect("Failed to compress compound");

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
            read_compound.get_string("string_value"),
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
    fn gzip_convenience_methods() {
        // Create a test compound
        let mut compound = NbtCompound::new();
        compound.put_int("test_value", 12345);

        // Test convenience method for writing
        let buffer =
            write_gzip_compound_tag_to_bytes(compound).expect("Failed to compress compound");

        // Test streaming read from the buffer
        let read_compound =
            read_gzip_compound_tag(Cursor::new(buffer)).expect("Failed to decompress compound");

        assert_eq!(read_compound.get_int("test_value"), Some(12345));
    }

    #[test]
    fn gzip_empty_compound() {
        let compound = NbtCompound::new();
        let mut buffer = Vec::new();
        write_gzip_compound_tag(compound, &mut buffer).expect("Failed to compress empty compound");
        let read_compound = read_gzip_compound_tag(Cursor::new(buffer))
            .expect("Failed to decompress empty compound");

        assert_eq!(read_compound.child_tags.len(), 0);
    }

    #[test]
    fn gzip_large_compound() {
        let mut compound = NbtCompound::new();

        // Add 1000 integer entries
        for i in 0..1000 {
            compound.put_int(&format!("value_{i}"), i);
        }

        let mut buffer = Vec::new();
        write_gzip_compound_tag(compound, &mut buffer).expect("Failed to compress large compound");
        let read_compound = read_gzip_compound_tag(Cursor::new(buffer))
            .expect("Failed to decompress large compound");

        assert_eq!(read_compound.child_tags.len(), 1000);

        // Verify a few entries
        assert_eq!(read_compound.get_int("value_0"), Some(0));
        assert_eq!(read_compound.get_int("value_500"), Some(500));
        assert_eq!(read_compound.get_int("value_999"), Some(999));
    }

    #[test]
    fn direct_file_io() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test_compound.dat");

        let mut compound = NbtCompound::new();
        compound.put_int("test_value", 42);

        let file = File::create(&file_path).expect("Failed to create temp file");
        write_gzip_compound_tag(compound, file).expect("Failed to write compound to file");

        let file = File::open(&file_path).expect("Failed to open temp file");
        let read_compound =
            read_gzip_compound_tag(file).expect("Failed to read compound from file");

        assert_eq!(read_compound.get_int("test_value"), Some(42));
    }
}
