use std::io::{Error, Write};

use pumpkin_data::bedrock_biome::BIOME_DEFINITIONS;
use pumpkin_macros::packet;

use crate::serial::PacketWrite;

#[packet(122)]
pub struct CBiomeDefinitionList;

impl PacketWrite for CBiomeDefinitionList {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(BIOME_DEFINITIONS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{codec::var_uint::VarUInt, serial::PacketRead};

    #[test]
    fn generated_registry_contains_all_biomes() {
        let biome_count = VarUInt::read(&mut BIOME_DEFINITIONS.as_slice())
            .expect("generated biome count must decode");

        assert_eq!(
            biome_count.0 as usize,
            pumpkin_data::bedrock_biome::BIOME_COUNT
        );
        assert_eq!(biome_count.0, 88);
        assert!(BIOME_DEFINITIONS.len() > 100_000);
    }
}
