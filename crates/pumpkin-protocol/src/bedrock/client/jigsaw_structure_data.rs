use std::io::{Error, Write};

use pumpkin_data::bedrock_jigsaw::JIGSAW_STRUCTURE_DATA;
use pumpkin_macros::packet;

use crate::serial::PacketWrite;

/// Transmits server-defined jigsaw structure generation rules, processors, and template pools.
#[packet(313)]
pub struct CJigsawStructureData;

impl PacketWrite for CJigsawStructureData {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(JIGSAW_STRUCTURE_DATA)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pumpkin_nbt::{Nbt, deserializer::NbtReadHelperBedrock};

    use super::*;

    #[test]
    fn jigsaw_structure_data_decodes_as_bedrock_network_nbt() {
        let mut encoded = Vec::new();
        CJigsawStructureData.write(&mut encoded).unwrap();

        let mut cursor = Cursor::new(encoded.as_slice());
        let mut reader = NbtReadHelperBedrock::new(&mut cursor);
        let nbt = Nbt::read(&mut reader).unwrap();
        assert_eq!(cursor.position() as usize, encoded.len());

        assert_eq!(
            nbt.get_list("jigsaws").unwrap().len(),
            pumpkin_data::bedrock_jigsaw::JIGSAW_COUNT
        );
        assert_eq!(
            nbt.get_list("processors").unwrap().len(),
            pumpkin_data::bedrock_jigsaw::PROCESSOR_COUNT
        );
        assert_eq!(
            nbt.get_list("structure_sets").unwrap().len(),
            pumpkin_data::bedrock_jigsaw::STRUCTURE_SET_COUNT
        );
        assert_eq!(
            nbt.get_list("template_pools").unwrap().len(),
            pumpkin_data::bedrock_jigsaw::TEMPLATE_POOL_COUNT
        );
    }
}
