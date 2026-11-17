use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fs,
    io::{Cursor, Error, ErrorKind, Read, Write},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use flate2::read::GzDecoder;
use proc_macro2::{Literal, TokenStream};
use pumpkin_nbt::{
    Nbt,
    deserializer::{NbtDataSource, NbtReadHelper},
};
use quote::quote;
use serde::Deserialize;
use uuid::Uuid;

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

impl PacketWrite for i16 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl PacketWrite for i32 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl PacketWrite for i64 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl PacketWrite for bool {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&[u8::from(*self)])
    }
}

impl PacketWrite for str {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(
            u32::try_from(self.len())
                .map_err(|_| Error::new(ErrorKind::InvalidData, "recipe string is too long"))?,
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
        let zigzag = ((self.0 << 1) ^ (self.0 >> 31)) as u32;
        VarUInt(zigzag).write(writer)
    }
}

struct UuidLe(Uuid);

impl PacketWrite for UuidLe {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let (msb, lsb) = self.0.as_u64_pair();
        writer.write_all(&msb.to_le_bytes())?;
        writer.write_all(&lsb.to_le_bytes())
    }
}

/// Standard fixed-width Little-Endian NBT reader for Bedrock's `userDataBuf`
/// and `block_state_b64` blobs.
struct LeNbtReadHelper<D> {
    reader: D,
}

impl<D> LeNbtReadHelper<D> {
    const fn new(reader: D) -> Self {
        Self { reader }
    }
}

impl<'a, D: NbtDataSource<'a>> LeNbtReadHelper<D> {
    fn get_string_len(&mut self) -> pumpkin_nbt::deserializer::Result<u16> {
        let mut buf = [0u8; 2];
        self.reader.read_bytes(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }
}

impl<'a, D: NbtDataSource<'a>> NbtReadHelper<'a> for LeNbtReadHelper<D> {
    type Reader = D;

    fn reader(&mut self) -> &mut D {
        &mut self.reader
    }

    fn skip_string(&mut self) -> pumpkin_nbt::deserializer::Result<()> {
        let len = i64::from(self.get_string_len()?);
        self.skip_bytes(len)
    }

    fn get_u8(&mut self) -> pumpkin_nbt::deserializer::Result<u8> {
        self.reader.read_u8()
    }

    fn get_i8(&mut self) -> pumpkin_nbt::deserializer::Result<i8> {
        Ok(self.reader.read_u8()? as i8)
    }

    fn get_i16(&mut self) -> pumpkin_nbt::deserializer::Result<i16> {
        let mut buf = [0u8; 2];
        self.reader.read_bytes(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    fn get_i32(&mut self) -> pumpkin_nbt::deserializer::Result<i32> {
        let mut buf = [0u8; 4];
        self.reader.read_bytes(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    fn get_i64(&mut self) -> pumpkin_nbt::deserializer::Result<i64> {
        let mut buf = [0u8; 8];
        self.reader.read_bytes(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }

    fn get_f32(&mut self) -> pumpkin_nbt::deserializer::Result<f32> {
        let mut buf = [0u8; 4];
        self.reader.read_bytes(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    fn get_f64(&mut self) -> pumpkin_nbt::deserializer::Result<f64> {
        let mut buf = [0u8; 8];
        self.reader.read_bytes(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    fn get_string(&mut self) -> pumpkin_nbt::deserializer::Result<Cow<'a, str>> {
        let len = usize::from(self.get_string_len()?);
        self.reader.read_string(len)
    }

    fn get_byte_array(&mut self, len: usize) -> pumpkin_nbt::deserializer::Result<Cow<'a, [i8]>> {
        self.reader.read_byte_array(len)
    }
}

fn parse_le_nbt(bytes: &[u8]) -> Nbt {
    let mut cursor = Cursor::new(bytes);
    let mut helper = LeNbtReadHelper::new(&mut cursor);
    let nbt = Nbt::read(&mut helper).expect("Invalid Little-Endian NBT blob");
    assert_eq!(
        cursor.position() as usize,
        bytes.len(),
        "Trailing bytes in Little-Endian NBT blob"
    );
    nbt
}

#[derive(Deserialize)]
struct RuntimeItemStateJson {
    name: String,
    id: i16,
}

#[derive(Deserialize)]
struct CreativeItemsDocJson {
    items: Vec<CreativeItemBlockStateJson>,
}

#[derive(Deserialize)]
struct CreativeItemBlockStateJson {
    id: String,
    block_state_b64: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RecipesDocumentJson {
    version: u32,
    recipes: Vec<serde_json::Value>,
    potion_mixes: Vec<PotionMixJson>,
    container_mixes: Vec<ContainerMixJson>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum IngredientJson {
    #[serde(rename = "default")]
    Default {
        count: i32,
        id: String,
        #[serde(rename = "itemId")]
        _item_id: i16,
        #[serde(rename = "auxValue")]
        aux_value: i32,
    },
    #[serde(rename = "item_tag")]
    ItemTag {
        count: i32,
        #[serde(rename = "itemTag")]
        item_tag: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OutputItemJson {
    #[serde(rename = "legacyId")]
    legacy_id: i16,
    id: String,
    #[serde(default = "default_count")]
    count: u16,
    #[serde(default)]
    damage: u32,
    nbt_b64: Option<String>,
}

const fn default_count() -> u16 {
    1
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ShapelessRecipeJson {
    #[serde(rename = "type")]
    _recipe_type: u8,
    id: String,
    input: Vec<IngredientJson>,
    output: Vec<OutputItemJson>,
    block: String,
    priority: i32,
    #[serde(default)]
    unlock_context: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ShapedRecipeJson {
    #[serde(rename = "type")]
    _recipe_type: u8,
    id: String,
    input: BTreeMap<String, IngredientJson>,
    output: Vec<OutputItemJson>,
    shape: Vec<String>,
    block: String,
    priority: i32,
    assume_symmetry: bool,
    #[serde(default)]
    unlock_context: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MultiRecipeJson {
    #[serde(rename = "type")]
    _recipe_type: u8,
    uuid: Uuid,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SmithingInputsJson {
    template: IngredientJson,
    base: IngredientJson,
    addition: IngredientJson,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SmithingTransformRecipeJson {
    #[serde(rename = "type")]
    _recipe_type: u8,
    input: SmithingInputsJson,
    output: OutputItemJson,
    block: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SmithingTrimRecipeJson {
    #[serde(rename = "type")]
    _recipe_type: u8,
    input: SmithingInputsJson,
    block: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct PotionMixJson {
    input_id: String,
    input_meta: i32,
    reagent_id: String,
    reagent_meta: i32,
    output_id: String,
    output_meta: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ContainerMixJson {
    input_id: String,
    reagent_id: String,
    output_id: String,
}

struct StonecutterMeta {
    network_id: u32,
    output_item_id: i16,
    output_aux_value: u32,
}

fn write_empty_ingredient<W: Write>(writer: &mut W) -> Result<(), Error> {
    VarUInt(0).write(writer)?;
    VarInt(32767).write(writer)?;
    VarInt(0).write(writer)
}

fn write_ingredient<W: Write>(writer: &mut W, ingredient: &IngredientJson) -> Result<(), Error> {
    match ingredient {
        IngredientJson::Default {
            count,
            id,
            aux_value,
            ..
        } => {
            VarUInt(1).write(writer)?;
            "name".write(writer)?;
            id.as_str().write(writer)?;
            VarInt(*aux_value).write(writer)?;
            VarInt(*count).write(writer)
        }
        IngredientJson::ItemTag { count, item_tag } => {
            VarUInt(1).write(writer)?;
            "item_tag".write(writer)?;
            item_tag.as_str().write(writer)?;
            VarInt(32767).write(writer)?;
            VarInt(*count).write(writer)
        }
    }
}

fn write_item_instance<W: Write>(
    writer: &mut W,
    item: &OutputItemJson,
    runtime_items: &HashMap<String, i16>,
    block_runtime_ids: &HashMap<String, i32>,
) -> Result<(), Error> {
    let expected_id = *runtime_items
        .get(&item.id)
        .unwrap_or_else(|| panic!("Missing runtime item state for {}", item.id));
    assert_eq!(
        expected_id, item.legacy_id,
        "Recipe output legacyId mismatch for {}",
        item.id
    );

    VarInt(i32::from(item.legacy_id)).write(writer)?;
    if item.legacy_id == 0 {
        return Ok(());
    }

    item.count.write(writer)?;
    VarUInt(item.damage).write(writer)?;
    let block_runtime_id = *block_runtime_ids
        .get(&item.id)
        .unwrap_or_else(|| panic!("Missing creative item block runtime lookup for {}", item.id));
    VarInt(block_runtime_id).write(writer)?;

    let mut user_data = Vec::new();
    if let Some(nbt_b64) = &item.nbt_b64 {
        let raw_nbt = STANDARD
            .decode(nbt_b64)
            .expect("Failed to decode recipe output nbt_b64");
        let _ = parse_le_nbt(&raw_nbt);
        (-1i16).write(&mut user_data)?;
        1u8.write(&mut user_data)?;
        user_data.write_all(&raw_nbt)?;
    } else {
        0i16.write(&mut user_data)?;
    }
    0i32.write(&mut user_data)?; // canPlace count
    0i32.write(&mut user_data)?; // canBreak count
    if item.id == "minecraft:shield" {
        0i64.write(&mut user_data)?; // blockingTicks
    }

    VarUInt(u32::try_from(user_data.len()).expect("userDataBuf length fits in u32"))
        .write(writer)?;
    writer.write_all(&user_data)
}

fn write_always_unlocked<W: Write>(writer: &mut W) -> Result<(), Error> {
    // AlwaysUnlocked (context = 1) with no unlock ingredients so recipes show up
    // in the Bedrock recipe book without UnlockedRecipesPacket tracking.
    true.write(writer)?;
    VarInt(1).write(writer)?;
    false.write(writer)
}

fn recipe_uuid(net_id: u32, block: &str, id: &str) -> Uuid {
    Uuid::new_v3(
        &Uuid::NAMESPACE_OID,
        format!("bedrock_recipe:{net_id}:{block}:{id}").as_bytes(),
    )
}

fn write_shapeless_entry<W: Write>(
    writer: &mut W,
    recipe: &ShapelessRecipeJson,
    net_id: u32,
    runtime_items: &HashMap<String, i16>,
    block_runtime_ids: &HashMap<String, i32>,
) -> Result<(), Error> {
    let _ = &recipe.unlock_context;
    recipe.id.as_str().write(writer)?;
    VarUInt(u32::try_from(recipe.input.len()).expect("shapeless input count fits in u32"))
        .write(writer)?;
    for ingredient in &recipe.input {
        write_ingredient(writer, ingredient)?;
    }
    VarUInt(u32::try_from(recipe.output.len()).expect("shapeless output count fits in u32"))
        .write(writer)?;
    for output in &recipe.output {
        write_item_instance(writer, output, runtime_items, block_runtime_ids)?;
    }
    UuidLe(recipe_uuid(net_id, &recipe.block, &recipe.id)).write(writer)?;
    recipe.block.as_str().write(writer)?;
    VarInt(recipe.priority).write(writer)?;
    write_always_unlocked(writer)?;
    VarUInt(net_id).write(writer)
}

fn write_shaped_entry<W: Write>(
    writer: &mut W,
    recipe: &ShapedRecipeJson,
    net_id: u32,
    runtime_items: &HashMap<String, i16>,
    block_runtime_ids: &HashMap<String, i32>,
) -> Result<(), Error> {
    let _ = &recipe.unlock_context;
    let height = recipe.shape.len();
    let width = recipe.shape.first().map_or(0, String::len);
    assert!(
        height > 0 && width > 0,
        "Empty shape in shaped recipe {}",
        recipe.id
    );

    recipe.id.as_str().write(writer)?;
    VarInt(i32::try_from(width).expect("shaped width fits in i32")).write(writer)?;
    VarInt(i32::try_from(height).expect("shaped height fits in i32")).write(writer)?;
    VarUInt(u32::try_from(width * height).expect("shaped slot count fits in u32")).write(writer)?;
    for row in &recipe.shape {
        assert_eq!(
            row.len(),
            width,
            "Inconsistent row width in shaped recipe {}",
            recipe.id
        );
        for symbol in row.chars() {
            if symbol == ' ' {
                write_empty_ingredient(writer)?;
            } else {
                let mut key_buf = [0u8; 4];
                let key = symbol.encode_utf8(&mut key_buf);
                let ingredient = recipe.input.get(key).unwrap_or_else(|| {
                    panic!("Missing shaped key '{symbol}' in recipe {}", recipe.id)
                });
                write_ingredient(writer, ingredient)?;
            }
        }
    }

    VarUInt(u32::try_from(recipe.output.len()).expect("shaped output count fits in u32"))
        .write(writer)?;
    for output in &recipe.output {
        write_item_instance(writer, output, runtime_items, block_runtime_ids)?;
    }
    UuidLe(recipe_uuid(net_id, &recipe.block, &recipe.id)).write(writer)?;
    recipe.block.as_str().write(writer)?;
    VarInt(recipe.priority).write(writer)?;
    recipe.assume_symmetry.write(writer)?;
    write_always_unlocked(writer)?;
    VarUInt(net_id).write(writer)
}

fn load_runtime_items() -> HashMap<String, i16> {
    let items: Vec<RuntimeItemStateJson> = serde_json::from_str(
        &fs::read_to_string("../../assets/bedrock/runtime_item_states.json")
            .expect("Failed to read bedrock/runtime_item_states.json"),
    )
    .expect("Failed to parse bedrock/runtime_item_states.json");

    items.into_iter().map(|item| (item.name, item.id)).collect()
}

fn load_block_runtime_ids() -> HashMap<String, i32> {
    let doc: CreativeItemsDocJson = serde_json::from_str(
        &fs::read_to_string("../../assets/bedrock/creative_items.json")
            .expect("Failed to read bedrock/creative_items.json"),
    )
    .expect("Failed to parse bedrock/creative_items.json");

    let mut map = HashMap::new();
    for item in doc.items {
        let runtime_id = if let Some(b64) = item.block_state_b64 {
            let raw = STANDARD
                .decode(b64)
                .expect("Failed to decode creative_items block_state_b64");
            let nbt = parse_le_nbt(&raw);
            nbt.get_int("network_id")
                .unwrap_or_else(|| panic!("Missing network_id in block_state_b64 for {}", item.id))
        } else {
            0
        };
        if let Some(prev) = map.insert(item.id.clone(), runtime_id) {
            assert_eq!(
                prev, runtime_id,
                "Conflicting block runtime IDs for creative item {}",
                item.id
            );
        }
    }
    map
}

pub fn build() -> TokenStream {
    let runtime_items = load_runtime_items();
    let block_runtime_ids = load_block_runtime_ids();

    let gz_bytes = fs::read("../../assets/bedrock/recipes.json.gz")
        .expect("Failed to read bedrock/recipes.json.gz");
    let mut json_str = String::new();
    GzDecoder::new(gz_bytes.as_slice())
        .read_to_string(&mut json_str)
        .expect("Failed to decompress bedrock/recipes.json.gz");
    let doc: RecipesDocumentJson =
        serde_json::from_str(&json_str).expect("Failed to parse bedrock/recipes.json.gz");
    assert_eq!(
        doc.version, 2225,
        "Unexpected Bedrock recipes.json.gz version"
    );

    let mut shaped_entries = Vec::new();
    let mut shapeless_entries = Vec::new();
    let mut multi_entries = Vec::new();
    let mut user_data_shapeless_entries = Vec::new();
    let mut smithing_transform_entries = Vec::new();
    let mut smithing_trim_entries = Vec::new();

    for raw in doc.recipes {
        let recipe_type = raw
            .get("type")
            .and_then(serde_json::Value::as_u64)
            .expect("Recipe entry missing numeric type");
        match recipe_type {
            0 => shapeless_entries.push(
                serde_json::from_value::<ShapelessRecipeJson>(raw)
                    .expect("Invalid type 0 shapeless recipe"),
            ),
            1 => shaped_entries.push(
                serde_json::from_value::<ShapedRecipeJson>(raw)
                    .expect("Invalid type 1 shaped recipe"),
            ),
            4 => multi_entries.push(
                serde_json::from_value::<MultiRecipeJson>(raw)
                    .expect("Invalid type 4 multi recipe"),
            ),
            5 => user_data_shapeless_entries.push(
                serde_json::from_value::<ShapelessRecipeJson>(raw)
                    .expect("Invalid type 5 user-data shapeless recipe"),
            ),
            8 => smithing_transform_entries.push(
                serde_json::from_value::<SmithingTransformRecipeJson>(raw)
                    .expect("Invalid type 8 smithing transform recipe"),
            ),
            9 => smithing_trim_entries.push(
                serde_json::from_value::<SmithingTrimRecipeJson>(raw)
                    .expect("Invalid type 9 smithing trim recipe"),
            ),
            other => panic!("Unsupported Bedrock recipe type {other} in recipes.json.gz"),
        }
    }

    let mut next_net_id = 1u32;

    let shaped_count = u32::try_from(shaped_entries.len()).expect("shaped count fits in u32");
    let mut shaped_bytes = Vec::new();
    for recipe in &shaped_entries {
        let net_id = next_net_id;
        next_net_id += 1;
        write_shaped_entry(
            &mut shaped_bytes,
            recipe,
            net_id,
            &runtime_items,
            &block_runtime_ids,
        )
        .expect("Failed to encode shaped recipe");
    }

    let shapeless_count =
        u32::try_from(shapeless_entries.len()).expect("shapeless count fits in u32");
    let mut shapeless_bytes = Vec::new();
    let mut stonecutter_recipes = Vec::new();
    for recipe in &shapeless_entries {
        let net_id = next_net_id;
        next_net_id += 1;
        if recipe.block == "stonecutter" {
            let output = recipe
                .output
                .first()
                .expect("Stonecutter recipe missing output");
            stonecutter_recipes.push(StonecutterMeta {
                network_id: net_id,
                output_item_id: output.legacy_id,
                output_aux_value: output.damage,
            });
        }
        write_shapeless_entry(
            &mut shapeless_bytes,
            recipe,
            net_id,
            &runtime_items,
            &block_runtime_ids,
        )
        .expect("Failed to encode shapeless recipe");
    }

    let mut tail_bytes = Vec::new();

    // [2] multi_recipes
    let multi_count = u32::try_from(multi_entries.len()).expect("multi count fits in u32");
    VarUInt(multi_count)
        .write(&mut tail_bytes)
        .expect("Failed to write multi count");
    for recipe in &multi_entries {
        let net_id = next_net_id;
        next_net_id += 1;
        UuidLe(recipe.uuid)
            .write(&mut tail_bytes)
            .expect("Failed to write multi recipe uuid");
        VarUInt(net_id)
            .write(&mut tail_bytes)
            .expect("Failed to write multi recipe net_id");
    }

    // [3] user_data_shapeless_recipes (CraftingDataType::SHULKER_BOX = 5)
    let user_data_shapeless_count = u32::try_from(user_data_shapeless_entries.len())
        .expect("user_data_shapeless count fits in u32");
    VarUInt(user_data_shapeless_count)
        .write(&mut tail_bytes)
        .expect("Failed to write user_data_shapeless count");
    for recipe in &user_data_shapeless_entries {
        let net_id = next_net_id;
        next_net_id += 1;
        write_shapeless_entry(
            &mut tail_bytes,
            recipe,
            net_id,
            &runtime_items,
            &block_runtime_ids,
        )
        .expect("Failed to encode user_data_shapeless recipe");
    }

    // [4] shapeless_chemistry_recipes (empty)
    VarUInt(0)
        .write(&mut tail_bytes)
        .expect("Failed to write shapeless_chemistry count");
    // [5] shaped_chemistry_recipes (empty)
    VarUInt(0)
        .write(&mut tail_bytes)
        .expect("Failed to write shaped_chemistry count");

    // [6] smithing_transform_recipes
    let smithing_transform_count = u32::try_from(smithing_transform_entries.len())
        .expect("smithing_transform count fits in u32");
    VarUInt(smithing_transform_count)
        .write(&mut tail_bytes)
        .expect("Failed to write smithing_transform count");
    for recipe in &smithing_transform_entries {
        let net_id = next_net_id;
        next_net_id += 1;
        let id = format!("{}_smithing", recipe.output.id);
        id.as_str()
            .write(&mut tail_bytes)
            .expect("Failed to write smithing_transform id");
        write_ingredient(&mut tail_bytes, &recipe.input.template)
            .expect("Failed to write smithing_transform template");
        write_ingredient(&mut tail_bytes, &recipe.input.base)
            .expect("Failed to write smithing_transform base");
        write_ingredient(&mut tail_bytes, &recipe.input.addition)
            .expect("Failed to write smithing_transform addition");
        write_item_instance(
            &mut tail_bytes,
            &recipe.output,
            &runtime_items,
            &block_runtime_ids,
        )
        .expect("Failed to write smithing_transform output");
        recipe
            .block
            .as_str()
            .write(&mut tail_bytes)
            .expect("Failed to write smithing_transform block");
        VarUInt(net_id)
            .write(&mut tail_bytes)
            .expect("Failed to write smithing_transform net_id");
    }

    // [7] smithing_trim_recipes
    let smithing_trim_count =
        u32::try_from(smithing_trim_entries.len()).expect("smithing_trim count fits in u32");
    VarUInt(smithing_trim_count)
        .write(&mut tail_bytes)
        .expect("Failed to write smithing_trim count");
    for (idx, recipe) in smithing_trim_entries.iter().enumerate() {
        let net_id = next_net_id;
        next_net_id += 1;
        let id = if idx == 0 {
            "minecraft:smithing_armor_trim".to_string()
        } else {
            format!("minecraft:smithing_armor_trim_{idx}")
        };
        id.as_str()
            .write(&mut tail_bytes)
            .expect("Failed to write smithing_trim id");
        write_ingredient(&mut tail_bytes, &recipe.input.template)
            .expect("Failed to write smithing_trim template");
        write_ingredient(&mut tail_bytes, &recipe.input.base)
            .expect("Failed to write smithing_trim base");
        write_ingredient(&mut tail_bytes, &recipe.input.addition)
            .expect("Failed to write smithing_trim addition");
        recipe
            .block
            .as_str()
            .write(&mut tail_bytes)
            .expect("Failed to write smithing_trim block");
        VarUInt(net_id)
            .write(&mut tail_bytes)
            .expect("Failed to write smithing_trim net_id");
    }

    // [8] potion_mixes
    let potion_mix_count =
        u32::try_from(doc.potion_mixes.len()).expect("potion_mix count fits in u32");
    VarUInt(potion_mix_count)
        .write(&mut tail_bytes)
        .expect("Failed to write potion_mix count");
    for mix in &doc.potion_mixes {
        let input_id = *runtime_items
            .get(&mix.input_id)
            .unwrap_or_else(|| panic!("Missing runtime item state for {}", mix.input_id));
        let reagent_id = *runtime_items
            .get(&mix.reagent_id)
            .unwrap_or_else(|| panic!("Missing runtime item state for {}", mix.reagent_id));
        let output_id = *runtime_items
            .get(&mix.output_id)
            .unwrap_or_else(|| panic!("Missing runtime item state for {}", mix.output_id));
        VarInt(i32::from(input_id))
            .write(&mut tail_bytes)
            .expect("Failed to write potion input_id");
        VarInt(mix.input_meta)
            .write(&mut tail_bytes)
            .expect("Failed to write potion input_meta");
        VarInt(i32::from(reagent_id))
            .write(&mut tail_bytes)
            .expect("Failed to write potion reagent_id");
        VarInt(mix.reagent_meta)
            .write(&mut tail_bytes)
            .expect("Failed to write potion reagent_meta");
        VarInt(i32::from(output_id))
            .write(&mut tail_bytes)
            .expect("Failed to write potion output_id");
        VarInt(mix.output_meta)
            .write(&mut tail_bytes)
            .expect("Failed to write potion output_meta");
    }

    // [9] container_mixes
    let container_mix_count =
        u32::try_from(doc.container_mixes.len()).expect("container_mix count fits in u32");
    VarUInt(container_mix_count)
        .write(&mut tail_bytes)
        .expect("Failed to write container_mix count");
    for mix in &doc.container_mixes {
        let input_id = *runtime_items
            .get(&mix.input_id)
            .unwrap_or_else(|| panic!("Missing runtime item state for {}", mix.input_id));
        let reagent_id = *runtime_items
            .get(&mix.reagent_id)
            .unwrap_or_else(|| panic!("Missing runtime item state for {}", mix.reagent_id));
        let output_id = *runtime_items
            .get(&mix.output_id)
            .unwrap_or_else(|| panic!("Missing runtime item state for {}", mix.output_id));
        VarInt(i32::from(input_id))
            .write(&mut tail_bytes)
            .expect("Failed to write container input_id");
        VarInt(i32::from(reagent_id))
            .write(&mut tail_bytes)
            .expect("Failed to write container reagent_id");
        VarInt(i32::from(output_id))
            .write(&mut tail_bytes)
            .expect("Failed to write container output_id");
    }

    // [10] material_reducers (empty)
    VarUInt(0)
        .write(&mut tail_bytes)
        .expect("Failed to write material_reducers count");

    let shaped_len = shaped_bytes.len();
    let shaped_lit = Literal::byte_string(&shaped_bytes);
    let shapeless_len = shapeless_bytes.len();
    let shapeless_lit = Literal::byte_string(&shapeless_bytes);
    let tail_len = tail_bytes.len();
    let tail_lit = Literal::byte_string(&tail_bytes);

    let stonecutter_len = stonecutter_recipes.len();
    let stonecutter_tokens = stonecutter_recipes.iter().map(|entry| {
        let network_id = entry.network_id;
        let output_item_id = entry.output_item_id;
        let output_aux_value = entry.output_aux_value;
        quote! {
            StonecutterRecipe {
                network_id: #network_id,
                output_item_id: #output_item_id,
                output_aux_value: #output_aux_value,
            }
        }
    });

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct StonecutterRecipe {
            pub network_id: u32,
            pub output_item_id: i16,
            pub output_aux_value: u32,
        }

        pub const SHAPED_COUNT: u32 = #shaped_count;
        pub const SHAPELESS_COUNT: u32 = #shapeless_count;
        pub const MULTI_COUNT: u32 = #multi_count;
        pub const USER_DATA_SHAPELESS_COUNT: u32 = #user_data_shapeless_count;
        pub const SMITHING_TRANSFORM_COUNT: u32 = #smithing_transform_count;
        pub const SMITHING_TRIM_COUNT: u32 = #smithing_trim_count;
        pub const POTION_MIX_COUNT: u32 = #potion_mix_count;
        pub const CONTAINER_MIX_COUNT: u32 = #container_mix_count;
        pub const NEXT_RECIPE_NETWORK_ID: u32 = #next_net_id;

        pub const STONECUTTER_RECIPES: &[StonecutterRecipe; #stonecutter_len] = &[
            #(#stonecutter_tokens),*
        ];

        pub const SHAPED_RECIPES: &[u8; #shaped_len] = #shaped_lit;
        pub const SHAPELESS_RECIPES: &[u8; #shapeless_len] = #shapeless_lit;
        pub const TAIL_SECTIONS: &[u8; #tail_len] = #tail_lit;
    }
}
