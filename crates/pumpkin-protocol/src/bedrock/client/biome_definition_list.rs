use std::{
    collections::{BTreeMap, HashMap, hash_map::Entry},
    io::{Error, Write},
    sync::LazyLock,
};

use pumpkin_macros::packet;
use serde::Deserialize;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

static BIOME_DEFINITIONS: LazyLock<Result<BTreeMap<String, BiomeDefinition>, serde_json::Error>> =
    LazyLock::new(|| {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/bedrock/biome_definitions.json"
        )))
    });

#[packet(122)]
pub struct CBiomeDefinitionList;

impl PacketWrite for CBiomeDefinitionList {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let definitions = BIOME_DEFINITIONS.as_ref().map_err(|error| {
            Error::other(format!("invalid built-in biome definitions: {error}"))
        })?;

        write_definitions(writer, definitions)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BiomeDefinition {
    id: Option<u16>,
    temperature: f32,
    downfall: f32,
    foliage_snow: f32,
    depth: f32,
    scale: f32,
    map_water_color: BiomeColor,
    rain: bool,
    tags: Vec<String>,
}

#[derive(Deserialize)]
struct BiomeColor {
    a: u8,
    r: u8,
    g: u8,
    b: u8,
}

fn write_definitions<W: Write>(
    writer: &mut W,
    definitions: &BTreeMap<String, BiomeDefinition>,
) -> Result<(), Error> {
    let mut strings = Vec::new();
    let mut string_indices = HashMap::new();

    VarUInt(definitions.len() as u32).write(writer)?;
    for (name, definition) in definitions {
        intern_string(&mut strings, &mut string_indices, name)?.write(writer)?;
        definition.id.unwrap_or(u16::MAX).write(writer)?;
        definition.temperature.write(writer)?;
        definition.downfall.write(writer)?;
        definition.foliage_snow.write(writer)?;
        definition.depth.write(writer)?;
        definition.scale.write(writer)?;

        u32::from_be_bytes([
            definition.map_water_color.a,
            definition.map_water_color.r,
            definition.map_water_color.g,
            definition.map_water_color.b,
        ])
        .write(writer)?;

        definition.rain.write(writer)?;
        true.write(writer)?;
        VarUInt(definition.tags.len() as u32).write(writer)?;
        for tag in &definition.tags {
            intern_string(&mut strings, &mut string_indices, tag)?.write(writer)?;
        }

        // The vanilla registry intentionally omits client-side chunk-generation data.
        false.write(writer)?;
    }

    VarUInt(strings.len() as u32).write(writer)?;
    for string in strings {
        string.write(writer)?;
    }

    Ok(())
}

fn intern_string<'a>(
    strings: &mut Vec<&'a str>,
    indices: &mut HashMap<&'a str, u16>,
    string: &'a str,
) -> Result<u16, Error> {
    match indices.entry(string) {
        Entry::Occupied(entry) => Ok(*entry.get()),
        Entry::Vacant(entry) => {
            let index = u16::try_from(strings.len())
                .map_err(|_| Error::other("biome string table exceeds 65536 entries"))?;
            strings.push(string);
            entry.insert(index);
            Ok(index)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_definition_and_shared_string_table() {
        let definitions = BTreeMap::from([(
            "minecraft:test".to_string(),
            BiomeDefinition {
                id: None,
                temperature: 0.5,
                downfall: 0.25,
                foliage_snow: 1.0,
                depth: -1.0,
                scale: 0.125,
                map_water_color: BiomeColor {
                    a: 0xa5,
                    r: 0x14,
                    g: 0xa2,
                    b: 0xc5,
                },
                rain: true,
                tags: vec!["ocean".to_string(), "overworld".to_string()],
            },
        )]);
        let mut actual = Vec::new();

        let result = write_definitions(&mut actual, &definitions);

        assert!(result.is_ok(), "encoding failed: {result:?}");
        assert_eq!(
            actual,
            [
                0x01, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x80, 0x3e, 0x00,
                0x00, 0x80, 0x3f, 0x00, 0x00, 0x80, 0xbf, 0x00, 0x00, 0x00, 0x3e, 0xc5, 0xa2, 0x14,
                0xa5, 0x01, 0x01, 0x02, 0x01, 0x00, 0x02, 0x00, 0x00, 0x03, 0x0e, b'm', b'i', b'n',
                b'e', b'c', b'r', b'a', b'f', b't', b':', b't', b'e', b's', b't', 0x05, b'o', b'c',
                b'e', b'a', b'n', 0x09, b'o', b'v', b'e', b'r', b'w', b'o', b'r', b'l', b'd',
            ]
        );
    }

    #[test]
    fn bundled_registry_contains_current_vanilla_biomes() {
        let Ok(definitions) = BIOME_DEFINITIONS.as_ref() else {
            panic!("bundled biome definitions must be valid JSON");
        };

        assert_eq!(definitions.len(), 88);
        assert!(definitions.contains_key("minecraft:pale_garden"));
        assert!(definitions.contains_key("minecraft:sulfur_caves"));
    }
}
