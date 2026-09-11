use std::{
    fs,
    io::{Cursor, Error, ErrorKind, Write},
};

use proc_macro2::{Literal, TokenStream};
use pumpkin_nbt::{NbtCompound, nbt_compress::read_gzip_compound_tag, tag::NbtTag};
use quote::quote;

trait PacketWrite {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
}

macro_rules! packet_write_le {
    ($($ty:ty),+ $(,)?) => {$ (
        impl PacketWrite for $ty {
            fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
                writer.write_all(&self.to_le_bytes())
            }
        }
    )+ };
}

packet_write_le!(i16, i32, u8, u16, u32, f32);

impl PacketWrite for bool {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&[u8::from(*self)])
    }
}

impl PacketWrite for str {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(
            u32::try_from(self.len())
                .map_err(|_| Error::new(ErrorKind::InvalidData, "biome string is too long"))?,
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

struct VarInt(i32);

impl PacketWrite for VarInt {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(self.0 as u32).write(writer)
    }
}

fn number(compound: &NbtCompound, key: &str) -> i64 {
    match compound.get(key) {
        Some(NbtTag::Byte(value)) => i64::from(*value),
        Some(NbtTag::Short(value)) => i64::from(*value),
        Some(NbtTag::Int(value)) => i64::from(*value),
        Some(NbtTag::Long(value)) => *value,
        Some(NbtTag::Float(value)) => *value as i64,
        Some(NbtTag::Double(value)) => *value as i64,
        _ => 0,
    }
}

fn decimal(compound: &NbtCompound, key: &str) -> f32 {
    match compound.get(key) {
        Some(NbtTag::Byte(value)) => f32::from(*value),
        Some(NbtTag::Short(value)) => f32::from(*value),
        Some(NbtTag::Int(value)) => *value as f32,
        Some(NbtTag::Long(value)) => *value as f32,
        Some(NbtTag::Float(value)) => *value,
        Some(NbtTag::Double(value)) => *value as f32,
        _ => 0.0,
    }
}

fn text<'a>(compound: &'a NbtCompound, key: &str) -> &'a str {
    compound.get_string(key).unwrap_or_default()
}

fn compounds<'a>(compound: &'a NbtCompound, key: &str) -> Vec<&'a NbtCompound> {
    compound
        .get_list(key)
        .unwrap_or_default()
        .iter()
        .filter_map(NbtTag::extract_compound)
        .collect()
}

fn numbers(compound: &NbtCompound, key: &str) -> Vec<i64> {
    match compound.get(key) {
        Some(NbtTag::List(values)) => values
            .iter()
            .map(|value| match value {
                NbtTag::Byte(value) => i64::from(*value),
                NbtTag::Short(value) => i64::from(*value),
                NbtTag::Int(value) => i64::from(*value),
                NbtTag::Long(value) => *value,
                _ => 0,
            })
            .collect(),
        Some(NbtTag::IntArray(values)) => values.iter().copied().map(i64::from).collect(),
        Some(NbtTag::LongArray(values)) => values.clone(),
        _ => Vec::new(),
    }
}

fn decimals(compound: &NbtCompound, key: &str) -> Vec<f32> {
    compound
        .get_list(key)
        .unwrap_or_default()
        .iter()
        .map(|value| match value {
            NbtTag::Float(value) => *value,
            NbtTag::Double(value) => *value as f32,
            _ => 0.0,
        })
        .collect()
}

fn as_i16(value: i64) -> i16 {
    i16::try_from(value).unwrap_or_default()
}

fn as_i32(value: i64) -> i32 {
    i32::try_from(value).unwrap_or_default()
}

fn as_u8(value: i64) -> u8 {
    u8::try_from(value).unwrap_or_default()
}

fn as_u16(value: i64) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

fn as_u32(value: i64) -> u32 {
    u32::try_from(value).unwrap_or_default()
}

fn write_len<W: Write>(writer: &mut W, length: usize) -> Result<(), Error> {
    VarUInt(
        u32::try_from(length)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "biome list is too long"))?,
    )
    .write(writer)
}

fn write_expression_op<W: Write>(
    writer: &mut W,
    compound: &NbtCompound,
    key: &str,
) -> Result<(), Error> {
    VarInt(if compound.has(key) {
        as_i32(number(compound, key))
    } else {
        -1
    })
    .write(writer)
}

fn write_compound_list<W, F>(
    writer: &mut W,
    entries: Vec<&NbtCompound>,
    mut encode: F,
) -> Result<(), Error>
where
    W: Write,
    F: FnMut(&mut W, &NbtCompound) -> Result<(), Error>,
{
    write_len(writer, entries.len())?;
    for entry in entries {
        encode(writer, entry)?;
    }
    Ok(())
}

fn write_optional_compound<W, F>(
    writer: &mut W,
    compound: &NbtCompound,
    key: &str,
    encode: F,
) -> Result<(), Error>
where
    W: Write,
    F: FnOnce(&mut W, &NbtCompound) -> Result<(), Error>,
{
    if let Some(value) = compound.get_compound(key) {
        true.write(writer)?;
        encode(writer, value)
    } else {
        false.write(writer)
    }
}

fn write_coordinate<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    write_expression_op(writer, data, "minValueType")?;
    as_u16(number(data, "minValue")).write(writer)?;
    write_expression_op(writer, data, "maxValueType")?;
    as_u16(number(data, "maxValue")).write(writer)?;
    as_u32(number(data, "gridOffset")).write(writer)?;
    as_u32(number(data, "gridStepSize")).write(writer)?;
    VarInt(as_i32(number(data, "distribution"))).write(writer)
}

fn write_scatter<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    write_compound_list(writer, compounds(data, "coordinates"), write_coordinate)?;
    VarInt(as_i32(number(data, "evalOrder"))).write(writer)?;
    write_expression_op(writer, data, "chancePercentType")?;
    as_u16(number(data, "chancePercent")).write(writer)?;
    as_i32(number(data, "chanceNumerator")).write(writer)?;
    as_i32(number(data, "chanceDenominator")).write(writer)?;
    write_expression_op(writer, data, "iterationsType")?;
    as_u16(number(data, "iterations")).write(writer)
}

fn write_feature<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    if let Some(scatter) = data.get_compound("scatter") {
        write_scatter(writer, scatter)?;
    } else {
        write_scatter(writer, &NbtCompound::new())?;
    }
    as_u16(number(data, "feature")).write(writer)?;
    as_u16(number(data, "identifier")).write(writer)?;
    as_u16(number(data, "pass")).write(writer)?;
    (number(data, "canUseInternalFeature") != 0).write(writer)
}

fn write_climate<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    decimal(data, "temperature").write(writer)?;
    decimal(data, "downfall").write(writer)?;
    decimal(data, "snowAccumulationMin").write(writer)?;
    decimal(data, "snowAccumulationMax").write(writer)
}

fn write_mountain_params<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    as_i32(number(data, "steepBlock")).write(writer)?;
    for key in [
        "northSlopes",
        "southSlopes",
        "westSlopes",
        "eastSlopes",
        "topSlideEnabled",
    ] {
        (number(data, key) != 0).write(writer)?;
    }
    Ok(())
}

fn write_surface_material<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    for key in [
        "topBlock",
        "midBlock",
        "seaFloorBlock",
        "foundationBlock",
        "seaBlock",
        "seaFloorDepth",
    ] {
        as_i32(number(data, key)).write(writer)?;
    }
    Ok(())
}

fn write_element<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    decimal(data, "noiseFrequencyScale").write(writer)?;
    decimal(data, "noiseLowerBound").write(writer)?;
    decimal(data, "noiseUpperBound").write(writer)?;
    write_expression_op(writer, data, "heightMinType")?;
    as_u16(number(data, "heightMin")).write(writer)?;
    write_expression_op(writer, data, "heightMaxType")?;
    as_u16(number(data, "heightMax")).write(writer)?;
    if let Some(materials) = data.get_compound("adjustedMaterials") {
        write_surface_material(writer, materials)
    } else {
        write_surface_material(writer, &NbtCompound::new())
    }
}

fn write_weighted<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    as_u16(number(data, "biomeIdentifier")).write(writer)?;
    as_i32(number(data, "weight")).write(writer)
}

fn write_conditional<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    write_compound_list(writer, compounds(data, "transformsInto"), write_weighted)?;
    as_u16(number(data, "conditionJson")).write(writer)?;
    as_u32(number(data, "minPassingNeighbors")).write(writer)
}

fn write_weighted_temperature<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    VarInt(as_i32(number(data, "temperature"))).write(writer)?;
    as_i32(number(data, "weight")).write(writer)
}

fn write_overworld_gen_rules<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    for key in [
        "hillsTransformations",
        "mutateTransformations",
        "riverTransformations",
        "shoreTransformations",
    ] {
        write_compound_list(writer, compounds(data, key), write_weighted)?;
    }
    for key in ["preHillsEdge", "postShoreEdge"] {
        write_compound_list(writer, compounds(data, key), write_conditional)?;
    }
    write_compound_list(
        writer,
        compounds(data, "climate"),
        write_weighted_temperature,
    )
}

fn write_multinoise_gen_rules<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    for key in ["temperature", "humidity", "altitude", "weirdness", "weight"] {
        decimal(data, key).write(writer)?;
    }
    Ok(())
}

fn write_replacement<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    as_i16(number(data, "biome")).write(writer)?;
    as_i16(number(data, "dimension")).write(writer)?;
    let targets = numbers(data, "targetBiomes");
    write_len(writer, targets.len())?;
    for target in targets {
        as_i16(target).write(writer)?;
    }
    decimal(data, "amount").write(writer)?;
    decimal(data, "noiseFrequencyScale").write(writer)?;
    as_i32(number(data, "replacementIndex")).write(writer)
}

fn write_mesa_surface<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    as_i32(number(data, "clayMaterial")).write(writer)?;
    as_i32(number(data, "hardClayMaterial")).write(writer)?;
    (number(data, "brycePillars") != 0).write(writer)?;
    (number(data, "hasForest") != 0).write(writer)
}

fn write_optional_block<W: Write>(
    writer: &mut W,
    data: &NbtCompound,
    key: &str,
) -> Result<(), Error> {
    if data.has(key) {
        true.write(writer)?;
        as_i32(number(data, key)).write(writer)
    } else {
        false.write(writer)
    }
}

fn write_int_list<W: Write>(writer: &mut W, values: &[i64]) -> Result<(), Error> {
    write_len(writer, values.len())?;
    for value in values {
        as_i32(*value).write(writer)?;
    }
    Ok(())
}

fn write_capped_surface<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    write_int_list(writer, &numbers(data, "floorBlocks"))?;
    write_int_list(writer, &numbers(data, "ceilingBlocks"))?;
    for key in ["seaBlock", "foundationBlock", "beachBlock"] {
        write_optional_block(writer, data, key)?;
    }
    Ok(())
}

fn write_noise_block<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    text(data, "noise").write(writer)?;
    decimal(data, "threshold").write(writer)?;
    let range = data.get_compound("range");
    range
        .map_or(0.0, |range| decimal(range, "min"))
        .write(writer)?;
    range
        .map_or(0.0, |range| decimal(range, "max"))
        .write(writer)?;
    as_i32(number(data, "block")).write(writer)
}

fn write_noise_gradient_surface<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    write_int_list(writer, &numbers(data, "nonReplaceableBlocks"))?;
    write_compound_list(writer, compounds(data, "gradientBlocks"), write_noise_block)?;
    let empty = NbtCompound::new();
    let noise = data.get_compound("noise").unwrap_or(&empty);
    text(noise, "name").write(writer)?;
    as_i32(number(noise, "firstOctave")).write(writer)?;
    let amplitudes = decimals(noise, "amplitudes");
    write_len(writer, amplitudes.len())?;
    for amplitude in amplitudes {
        amplitude.write(writer)?;
    }
    Ok(())
}

fn write_surface_builder<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    write_optional_compound(writer, data, "surfaceMaterials", write_surface_material)?;
    for key in [
        "hasDefaultOverworldSurface",
        "hasSwampSurface",
        "hasFrozenOceanSurface",
        "hasTheEndSurface",
    ] {
        (number(data, key) != 0).write(writer)?;
    }
    write_optional_compound(writer, data, "mesaSurface", write_mesa_surface)?;
    write_optional_compound(writer, data, "cappedSurface", write_capped_surface)?;
    write_optional_compound(
        writer,
        data,
        "noiseGradientSurface",
        write_noise_gradient_surface,
    )
}

fn write_chunk_gen_data<W: Write>(writer: &mut W, data: &NbtCompound) -> Result<(), Error> {
    write_optional_compound(writer, data, "climate", write_climate)?;
    write_optional_compound(writer, data, "consolidatedFeatures", |writer, features| {
        write_compound_list(writer, compounds(features, "features"), write_feature)
    })?;
    write_optional_compound(writer, data, "mountainParams", write_mountain_params)?;
    write_optional_compound(
        writer,
        data,
        "surfaceMaterialAdjustment",
        |writer, adjustment| {
            write_compound_list(
                writer,
                compounds(adjustment, "biomeElements"),
                write_element,
            )
        },
    )?;
    write_optional_compound(writer, data, "overworldGenRules", write_overworld_gen_rules)?;
    write_optional_compound(
        writer,
        data,
        "multinoiseGenRules",
        write_multinoise_gen_rules,
    )?;
    write_optional_compound(writer, data, "legacyWorldGenRules", |writer, legacy| {
        write_compound_list(
            writer,
            compounds(legacy, "legacyPreHills"),
            write_conditional,
        )
    })?;
    write_optional_compound(writer, data, "replacementBiomes", |writer, replacements| {
        write_compound_list(
            writer,
            compounds(replacements, "replacements"),
            write_replacement,
        )
    })?;

    if data.has("villageType") {
        true.write(writer)?;
        as_u8(number(data, "villageType")).write(writer)?;
    } else {
        false.write(writer)?;
    }

    write_optional_compound(writer, data, "surfaceBuilderData", write_surface_builder)?;
    write_optional_compound(writer, data, "subSurfaceBuilderData", write_surface_builder)
}

fn write_biome<W: Write>(writer: &mut W, entry: &NbtCompound) -> Result<(), Error> {
    as_u16(number(entry, "index")).write(writer)?;
    let data = entry.get_compound("data").ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidData,
            "biome definition has no data compound",
        )
    })?;
    as_u16(number(data, "id")).write(writer)?;
    for key in ["temperature", "downfall", "foliageSnow", "depth", "scale"] {
        decimal(data, key).write(writer)?;
    }
    as_i32(number(data, "mapWaterColorARGB")).write(writer)?;
    (number(data, "rain") != 0).write(writer)?;

    write_optional_compound(writer, data, "tags", |writer, tags| {
        let values = numbers(tags, "tags");
        write_len(writer, values.len())?;
        for value in values {
            as_u16(value).write(writer)?;
        }
        Ok(())
    })?;
    write_optional_compound(writer, data, "chunkGenData", write_chunk_gen_data)
}

fn encode_biome_definitions(document: &[u8]) -> Result<(Vec<u8>, usize), Error> {
    let root = read_gzip_compound_tag(Cursor::new(document))
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
    let biomes = compounds(&root, "biomeData");
    if biomes.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "biome definitions contain no biomeData",
        ));
    }

    let mut payload = Vec::new();
    write_len(&mut payload, biomes.len())?;
    let biome_count = biomes.len();
    for biome in biomes {
        write_biome(&mut payload, biome)?;
    }

    let strings = root
        .get_list("biomeStringList")
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing biomeStringList"))?;
    write_len(&mut payload, strings.len())?;
    for string in strings {
        string
            .extract_string()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "invalid biome string"))?
            .write(&mut payload)?;
    }

    Ok((payload, biome_count))
}

pub fn build() -> TokenStream {
    let document = fs::read("../../assets/bedrock/biome_definitions.nbt")
        .expect("Failed to read bedrock/biome_definitions.nbt");
    let (payload, biome_count) = encode_biome_definitions(&document)
        .expect("Failed to encode bedrock/biome_definitions.nbt");
    let payload_len = payload.len();
    let payload = Literal::byte_string(&payload);

    quote! {
        pub const BIOME_DEFINITIONS: &[u8; #payload_len] = #payload;
        pub const BIOME_COUNT: usize = #biome_count;
    }
}
