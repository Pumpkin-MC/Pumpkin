use std::io::{Error, Write};

use pumpkin_data::bedrock_voxel_shape::VOXEL_SHAPES;
use pumpkin_macros::packet;

use crate::serial::PacketWrite;

/// Sends collision and selection voxel shapes to the client before `CStartGame`.
#[packet(337)]
pub struct CVoxelShapes;

impl PacketWrite for CVoxelShapes {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(VOXEL_SHAPES)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use crate::{
        codec::var_uint::VarUInt,
        serial::{PacketRead, PacketWrite},
    };

    use super::*;

    fn read_str(cursor: &mut Cursor<&[u8]>) -> String {
        let len = VarUInt::read(cursor).unwrap().0 as usize;
        let mut buf = vec![0u8; len];
        cursor.read_exact(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    fn read_f32_vec(cursor: &mut Cursor<&[u8]>) -> Vec<f32> {
        let len = VarUInt::read(cursor).unwrap().0 as usize;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(f32::read(cursor).unwrap());
        }
        out
    }

    #[test]
    fn voxel_shapes_wire_payload_decodes_cleanly() {
        let mut encoded = Vec::new();
        CVoxelShapes.write(&mut encoded).unwrap();

        let mut cursor = Cursor::new(encoded.as_slice());
        let shape_count = VarUInt::read(&mut cursor).unwrap().0 as usize;
        assert_eq!(shape_count, pumpkin_data::bedrock_voxel_shape::SHAPE_COUNT);

        for _ in 0..shape_count {
            let x_size = u8::read(&mut cursor).unwrap();
            let y_size = u8::read(&mut cursor).unwrap();
            let z_size = u8::read(&mut cursor).unwrap();

            let storage_len = VarUInt::read(&mut cursor).unwrap().0 as usize;
            let mut storage = vec![0u8; storage_len];
            cursor.read_exact(&mut storage).unwrap();

            let x_coords = read_f32_vec(&mut cursor);
            let y_coords = read_f32_vec(&mut cursor);
            let z_coords = read_f32_vec(&mut cursor);
            assert_eq!(x_coords.len(), usize::from(x_size) + 1);
            assert_eq!(y_coords.len(), usize::from(y_size) + 1);
            assert_eq!(z_coords.len(), usize::from(z_size) + 1);
        }

        let name_map_count = VarUInt::read(&mut cursor).unwrap().0 as usize;
        assert_eq!(
            name_map_count,
            pumpkin_data::bedrock_voxel_shape::NAME_MAP_COUNT
        );
        let mut empty_handle = None;
        for _ in 0..name_map_count {
            let name = read_str(&mut cursor);
            let handle = u16::read(&mut cursor).unwrap();
            assert!(usize::from(handle) < shape_count);
            if name == "minecraft:empty" {
                empty_handle = Some(handle);
            }
        }
        assert_eq!(empty_handle, Some(0));

        let custom_shape_count = u16::read(&mut cursor).unwrap();
        assert_eq!(custom_shape_count, 0);
        assert_eq!(cursor.position() as usize, encoded.len());
    }
}
