use std::{
    fs,
    io::{Error, ErrorKind, Read, Write},
};

use flate2::read::GzDecoder;
use indexmap::IndexMap;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use serde::Deserialize;

trait PacketWrite {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
}

impl PacketWrite for u8 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&[*self])
    }
}

impl PacketWrite for u16 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl PacketWrite for f32 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl PacketWrite for str {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(
            u32::try_from(self.len())
                .map_err(|_| Error::new(ErrorKind::InvalidData, "voxel shape name is too long"))?,
        )
        .write(writer)?;
        writer.write_all(self.as_bytes())
    }
}

struct VarUInt(u32);

impl PacketWrite for VarUInt {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let mut value = self.0;
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            writer.write_all(&[byte])?;
            if value == 0 {
                return Ok(());
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VoxelShapesDocumentJson {
    shapes: Vec<SerializableVoxelShapeJson>,
    names: IndexMap<String, u16>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SerializableVoxelShapeJson {
    cells: SerializableCellsJson,
    x_coordinates: Vec<f32>,
    y_coordinates: Vec<f32>,
    z_coordinates: Vec<f32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SerializableCellsJson {
    x_size: u8,
    y_size: u8,
    z_size: u8,
    storage: Vec<u8>,
}

fn write_f32_slice<W: Write>(writer: &mut W, values: &[f32]) -> Result<(), Error> {
    VarUInt(u32::try_from(values.len()).expect("coordinate count fits in u32")).write(writer)?;
    for &value in values {
        value.write(writer)?;
    }
    Ok(())
}

fn encode_voxel_shapes(doc: &VoxelShapesDocumentJson) -> Result<Vec<u8>, Error> {
    let mut payload = Vec::new();

    VarUInt(u32::try_from(doc.shapes.len()).expect("shape count fits in u32"))
        .write(&mut payload)?;
    for shape in &doc.shapes {
        assert_eq!(
            shape.x_coordinates.len(),
            usize::from(shape.cells.x_size) + 1,
            "Voxel shape xCoordinates count must equal xSize + 1"
        );
        assert_eq!(
            shape.y_coordinates.len(),
            usize::from(shape.cells.y_size) + 1,
            "Voxel shape yCoordinates count must equal ySize + 1"
        );
        assert_eq!(
            shape.z_coordinates.len(),
            usize::from(shape.cells.z_size) + 1,
            "Voxel shape zCoordinates count must equal zSize + 1"
        );

        shape.cells.x_size.write(&mut payload)?;
        shape.cells.y_size.write(&mut payload)?;
        shape.cells.z_size.write(&mut payload)?;

        VarUInt(u32::try_from(shape.cells.storage.len()).expect("storage length fits in u32"))
            .write(&mut payload)?;
        payload.write_all(&shape.cells.storage)?;

        write_f32_slice(&mut payload, &shape.x_coordinates)?;
        write_f32_slice(&mut payload, &shape.y_coordinates)?;
        write_f32_slice(&mut payload, &shape.z_coordinates)?;
    }

    VarUInt(u32::try_from(doc.names.len()).expect("name map size fits in u32"))
        .write(&mut payload)?;
    for (name, &handle) in &doc.names {
        assert!(
            usize::from(handle) < doc.shapes.len(),
            "Voxel shape handle {handle} for {name} out of bounds ({})",
            doc.shapes.len()
        );
        name.as_str().write(&mut payload)?;
        handle.write(&mut payload)?;
    }

    // Custom shape count (u16 LE, added in v944; 0 for vanilla shapes).
    0u16.write(&mut payload)?;

    Ok(payload)
}

pub fn build() -> TokenStream {
    let gz_bytes = fs::read("../../assets/bedrock/voxel_shapes.json.gz")
        .expect("Failed to read bedrock/voxel_shapes.json.gz");
    let mut json_str = String::new();
    GzDecoder::new(gz_bytes.as_slice())
        .read_to_string(&mut json_str)
        .expect("Failed to decompress bedrock/voxel_shapes.json.gz");

    let doc: VoxelShapesDocumentJson =
        serde_json::from_str(&json_str).expect("Failed to parse bedrock/voxel_shapes.json.gz");
    assert!(!doc.shapes.is_empty(), "voxel_shapes.json.gz has no shapes");
    assert!(
        !doc.names.is_empty(),
        "voxel_shapes.json.gz has empty name map"
    );

    let shape_count = doc.shapes.len();
    let name_map_count = doc.names.len();
    let payload = encode_voxel_shapes(&doc).expect("Failed to encode bedrock/voxel_shapes.json.gz");
    let payload_len = payload.len();
    let payload_lit = Literal::byte_string(&payload);

    quote! {
        pub const VOXEL_SHAPES: &[u8; #payload_len] = #payload_lit;
        pub const SHAPE_COUNT: usize = #shape_count;
        pub const NAME_MAP_COUNT: usize = #name_map_count;
    }
}
