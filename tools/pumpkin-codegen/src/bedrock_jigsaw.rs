use std::{
    fs,
    io::{Cursor, Read},
};

use flate2::read::GzDecoder;
use proc_macro2::{Literal, TokenStream};
use pumpkin_nbt::{
    BYTE_ARRAY_ID, BYTE_ID, COMPOUND_ID, DOUBLE_ID, END_ID, FLOAT_ID, INT_ARRAY_ID, INT_ID,
    LIST_ID, LONG_ARRAY_ID, LONG_ID, Nbt, SHORT_ID, STRING_ID,
    deserializer::{NbtReadHelper, NbtReadHelperBedrock, NbtReadHelperJava},
    nbt_compress::read_gzip_compound_tag,
    serializer::{NbtWriteHelper, NbtWriteHelperBedrock},
};
use quote::quote;

fn transcode_payload<'a, R: NbtReadHelper<'a>, W: NbtWriteHelper>(
    tag_id: u8,
    reader: &mut R,
    writer: &mut W,
) {
    match tag_id {
        END_ID => {}
        BYTE_ID => {
            let v = reader.get_i8().expect("Failed to read NBT byte");
            writer.write_i8(v).expect("Failed to write NBT byte");
        }
        SHORT_ID => {
            let v = reader.get_i16().expect("Failed to read NBT short");
            writer.write_i16(v).expect("Failed to write NBT short");
        }
        INT_ID => {
            let v = reader.get_i32().expect("Failed to read NBT int");
            writer.write_i32(v).expect("Failed to write NBT int");
        }
        LONG_ID => {
            let v = reader.get_i64().expect("Failed to read NBT long");
            writer.write_i64(v).expect("Failed to write NBT long");
        }
        FLOAT_ID => {
            let v = reader.get_f32().expect("Failed to read NBT float");
            writer.write_f32(v).expect("Failed to write NBT float");
        }
        DOUBLE_ID => {
            let v = reader.get_f64().expect("Failed to read NBT double");
            writer.write_f64(v).expect("Failed to write NBT double");
        }
        BYTE_ARRAY_ID => {
            let len = reader
                .get_i32()
                .expect("Failed to read NBT byte array length");
            let len_usize = usize::try_from(len).expect("Negative NBT byte array length");
            let bytes = reader
                .get_byte_array(len_usize)
                .expect("Failed to read NBT byte array");
            writer
                .write_i32(len)
                .expect("Failed to write NBT byte array length");
            let raw: Vec<u8> = bytes.iter().map(|&b| b.cast_unsigned()).collect();
            writer
                .write_slice(&raw)
                .expect("Failed to write NBT byte array payload");
        }
        STRING_ID => {
            let s = reader.get_string().expect("Failed to read NBT string");
            writer.write_string(&s).expect("Failed to write NBT string");
        }
        LIST_ID => {
            let elem_type = reader.get_u8().expect("Failed to read NBT list tag type");
            let len = reader.get_i32().expect("Failed to read NBT list length");
            assert!(len >= 0, "Negative NBT list length");
            writer
                .write_u8(elem_type)
                .expect("Failed to write NBT list tag type");
            writer
                .write_i32(len)
                .expect("Failed to write NBT list length");
            for _ in 0..len {
                transcode_payload(elem_type, reader, writer);
            }
        }
        COMPOUND_ID => loop {
            let child_type = reader
                .get_u8()
                .expect("Failed to read NBT compound entry tag type");
            writer
                .write_u8(child_type)
                .expect("Failed to write NBT compound entry tag type");
            if child_type == END_ID {
                break;
            }
            let key = reader
                .get_string()
                .expect("Failed to read NBT compound entry key");
            writer
                .write_string(&key)
                .expect("Failed to write NBT compound entry key");
            transcode_payload(child_type, reader, writer);
        },
        INT_ARRAY_ID => {
            let len = reader
                .get_i32()
                .expect("Failed to read NBT int array length");
            let len_usize = usize::try_from(len).expect("Negative NBT int array length");
            let values = reader
                .get_i32_array(len_usize)
                .expect("Failed to read NBT int array");
            writer
                .write_i32(len)
                .expect("Failed to write NBT int array length");
            for value in values {
                writer
                    .write_i32(value)
                    .expect("Failed to write NBT int array element");
            }
        }
        LONG_ARRAY_ID => {
            let len = reader
                .get_i32()
                .expect("Failed to read NBT long array length");
            let len_usize = usize::try_from(len).expect("Negative NBT long array length");
            let values = reader
                .get_i64_array(len_usize)
                .expect("Failed to read NBT long array");
            writer
                .write_i32(len)
                .expect("Failed to write NBT long array length");
            for value in values {
                writer
                    .write_i64(value)
                    .expect("Failed to write NBT long array element");
            }
        }
        other => panic!("Unsupported NBT tag type {other} in jigsaw_structure_data.nbt"),
    }
}

pub fn build() -> TokenStream {
    let gz_bytes = fs::read("../../assets/bedrock/jigsaw_structure_data.nbt")
        .expect("Failed to read bedrock/jigsaw_structure_data.nbt");

    let root = read_gzip_compound_tag(Cursor::new(gz_bytes.as_slice()))
        .expect("Failed to parse bedrock/jigsaw_structure_data.nbt");
    let jigsaw_count = root
        .get_list("jigsaws")
        .expect("Missing jigsaws list in jigsaw_structure_data.nbt")
        .len();
    let processor_count = root
        .get_list("processors")
        .expect("Missing processors list in jigsaw_structure_data.nbt")
        .len();
    let structure_set_count = root
        .get_list("structure_sets")
        .expect("Missing structure_sets list in jigsaw_structure_data.nbt")
        .len();
    let template_pool_count = root
        .get_list("template_pools")
        .expect("Missing template_pools list in jigsaw_structure_data.nbt")
        .len();
    assert!(
        jigsaw_count > 0
            && processor_count > 0
            && structure_set_count > 0
            && template_pool_count > 0,
        "Empty jigsaw structure lists in jigsaw_structure_data.nbt"
    );

    let mut raw_java_nbt = Vec::new();
    GzDecoder::new(gz_bytes.as_slice())
        .read_to_end(&mut raw_java_nbt)
        .expect("Failed to decompress bedrock/jigsaw_structure_data.nbt");

    let mut src_cursor = Cursor::new(raw_java_nbt.as_slice());
    let mut reader = NbtReadHelperJava::new(&mut src_cursor);
    let mut payload = Vec::new();
    {
        let mut writer = NbtWriteHelperBedrock::new(&mut payload);
        let root_type = reader.get_u8().expect("Failed to read root NBT tag ID");
        assert_eq!(
            root_type, COMPOUND_ID,
            "Expected root compound tag in jigsaw_structure_data.nbt"
        );
        let root_name = reader.get_string().expect("Failed to read root NBT name");
        writer
            .write_u8(COMPOUND_ID)
            .expect("Failed to write root NBT tag ID");
        writer
            .write_string(&root_name)
            .expect("Failed to write root NBT name");
        transcode_payload(COMPOUND_ID, &mut reader, &mut writer);
    }
    assert_eq!(
        src_cursor.position() as usize,
        raw_java_nbt.len(),
        "Trailing bytes in jigsaw_structure_data.nbt"
    );

    // Verify the transcoded Bedrock Network NBT decodes cleanly with zero trailing bytes.
    let mut verify_cursor = Cursor::new(payload.as_slice());
    let mut verify_reader = NbtReadHelperBedrock::new(&mut verify_cursor);
    let _ = Nbt::read(&mut verify_reader)
        .expect("Transcoded jigsaw_structure_data Bedrock NBT failed verification");
    assert_eq!(
        verify_cursor.position() as usize,
        payload.len(),
        "Trailing bytes after verifying transcoded jigsaw_structure_data Bedrock NBT"
    );

    let payload_len = payload.len();
    let payload_lit = Literal::byte_string(&payload);

    quote! {
        pub const JIGSAW_STRUCTURE_DATA: &[u8; #payload_len] = #payload_lit;
        pub const JIGSAW_COUNT: usize = #jigsaw_count;
        pub const PROCESSOR_COUNT: usize = #processor_count;
        pub const STRUCTURE_SET_COUNT: usize = #structure_set_count;
        pub const TEMPLATE_POOL_COUNT: usize = #template_pool_count;
    }
}
