use pumpkin_data::{
    BlockState, BlockStateId,
    block_properties::{ChestLikeProperties, ChestType},
    dye_color::DyeColor,
    item_stack::ItemStack,
    tag::Taggable,
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::math::position::BlockPos;

use super::sign::SignText;

/// Fluent builder for constructing Bedrock block-actor NBT compounds.
pub struct BedrockBlockActorBuilder {
    nbt: NbtCompound,
}

impl BedrockBlockActorBuilder {
    /// Creates a new builder initialized with standard coordinates and `isMovable: true`.
    #[must_use]
    pub fn new(id: &'static str, position: BlockPos) -> Self {
        let mut nbt = NbtCompound::new();
        nbt.put_string("id", id.to_string());
        nbt.put_int("x", position.0.x);
        nbt.put_int("y", position.0.y);
        nbt.put_int("z", position.0.z);
        nbt.put_bool("isMovable", true);
        Self { nbt }
    }

    /// Sets whether this block entity can be moved by pistons.
    #[must_use]
    pub fn movable(mut self, is_movable: bool) -> Self {
        self.nbt.put_bool("isMovable", is_movable);
        self
    }

    /// Optionally sets the custom name tag of this block entity.
    #[must_use]
    pub fn custom_name(mut self, name: Option<&str>) -> Self {
        if let Some(name) = name
            && !name.is_empty()
        {
            self.nbt.put_string("CustomName", name.to_string());
        }
        self
    }

    #[must_use]
    pub fn put_int(mut self, key: &'static str, val: i32) -> Self {
        self.nbt.put_int(key, val);
        self
    }

    #[must_use]
    pub fn put_short(mut self, key: &'static str, val: i16) -> Self {
        self.nbt.put_short(key, val);
        self
    }

    #[must_use]
    pub fn put_byte(mut self, key: &'static str, val: i8) -> Self {
        self.nbt.put_byte(key, val);
        self
    }

    #[must_use]
    pub fn put_long(mut self, key: &'static str, val: i64) -> Self {
        self.nbt.put_long(key, val);
        self
    }

    #[must_use]
    pub fn put_bool(mut self, key: &'static str, val: bool) -> Self {
        self.nbt.put_bool(key, val);
        self
    }

    #[must_use]
    pub fn put_string(mut self, key: &'static str, val: impl Into<String>) -> Self {
        self.nbt.put_string(key, val.into());
        self
    }

    #[must_use]
    pub fn put_compound(mut self, key: &'static str, val: NbtCompound) -> Self {
        self.nbt.put_compound(key, val);
        self
    }

    #[must_use]
    pub fn put_list(mut self, key: &'static str, val: Vec<NbtTag>) -> Self {
        self.nbt.put_list(key, val);
        self
    }

    /// Returns the finalized Bedrock NBT compound.
    #[must_use]
    pub fn finish(self) -> NbtCompound {
        self.nbt
    }

    /// Alias for [`Self::finish`].
    #[must_use]
    pub fn build(self) -> NbtCompound {
        self.finish()
    }
}

/// Converts a [`DyeColor`] into the signed 32-bit ARGB integer used by Bedrock signs.
#[must_use]
pub const fn dye_color_to_bedrock_sign_text_color(color: DyeColor) -> i32 {
    match color {
        DyeColor::White => -986896,
        DyeColor::Orange => -425955,
        DyeColor::Magenta => -3715395,
        DyeColor::LightBlue => -12930086,
        DyeColor::Yellow => -75715,
        DyeColor::Lime => -8337633,
        DyeColor::Pink => -816214,
        DyeColor::Gray => -12103854,
        DyeColor::LightGray => -6447721,
        DyeColor::Cyan => -15295332,
        DyeColor::Purple => -7785800,
        DyeColor::Blue => -12827478,
        DyeColor::Brown => -8170446,
        DyeColor::Green => -10585066,
        DyeColor::Red => -5231066,
        DyeColor::Black => -16777216,
    }
}

/// Maps a Bedrock sign text color ARGB integer back to a [`DyeColor`].
#[must_use]
pub const fn bedrock_sign_text_color_to_dye_color(color: i32) -> DyeColor {
    match color {
        -986896 => DyeColor::White,
        -425955 => DyeColor::Orange,
        -3715395 => DyeColor::Magenta,
        -12930086 => DyeColor::LightBlue,
        -75715 => DyeColor::Yellow,
        -8337633 => DyeColor::Lime,
        -816214 => DyeColor::Pink,
        -12103854 => DyeColor::Gray,
        -6447721 => DyeColor::LightGray,
        -15295332 => DyeColor::Cyan,
        -7785800 => DyeColor::Purple,
        -12827478 => DyeColor::Blue,
        -8170446 => DyeColor::Brown,
        -10585066 => DyeColor::Green,
        -5231066 => DyeColor::Red,
        _ => DyeColor::Black,
    }
}

/// Converts a Java [`DyeColor`] to Bedrock's inverted wool/banner metadata color index.
#[must_use]
pub const fn dye_color_to_bedrock_wool_color(color: DyeColor) -> i32 {
    15 - (color.id() as i32)
}

/// Translates a Java banner pattern name (bare or namespaced) into a Bedrock short code.
#[must_use]
pub fn banner_pattern_to_bedrock_code(pattern_name: &str) -> &'static str {
    let name = pattern_name
        .strip_prefix("minecraft:")
        .unwrap_or(pattern_name);
    match name {
        "border" | "bo" => "bo",
        "bricks" | "bri" => "bri",
        "circle" | "mc" => "mc",
        "creeper" | "cre" => "cre",
        "cross" | "cr" => "cr",
        "curly_border" | "cbo" => "cbo",
        "diagonal_left" | "ld" => "ld",
        "diagonal_right" | "rd" => "rd",
        "diagonal_up_left" | "lud" => "lud",
        "diagonal_up_right" | "rud" => "rud",
        "flow" | "flw" => "flw",
        "flower" | "flo" => "flo",
        "globe" | "glb" => "glb",
        "gradient" | "gra" => "gra",
        "gradient_up" | "gru" => "gru",
        "guster" | "gus" => "gus",
        "half_horizontal" | "hh" => "hh",
        "half_horizontal_bottom" | "hhb" => "hhb",
        "half_vertical" | "vh" => "vh",
        "half_vertical_right" | "vhr" => "vhr",
        "mojang" | "moj" => "moj",
        "piglin" | "pig" => "pig",
        "rhombus" | "mr" => "mr",
        "skull" | "sku" => "sku",
        "small_stripes" | "ss" => "ss",
        "square_bottom_left" | "bl" => "bl",
        "square_bottom_right" | "br" => "br",
        "square_top_left" | "tl" => "tl",
        "square_top_right" | "tr" => "tr",
        "straight_cross" | "sc" => "sc",
        "stripe_bottom" | "bs" => "bs",
        "stripe_center" | "cs" => "cs",
        "stripe_downleft" | "dls" => "dls",
        "stripe_downright" | "drs" => "drs",
        "stripe_left" | "ls" => "ls",
        "stripe_middle" | "ms" => "ms",
        "stripe_right" | "rs" => "rs",
        "stripe_top" | "ts" => "ts",
        "triangle_bottom" | "bt" => "bt",
        "triangle_top" | "tt" => "tt",
        "triangles_bottom" | "bts" => "bts",
        "triangles_top" | "tts" => "tts",
        _ => "b",
    }
}

/// Extracts the base [`DyeColor`] of a banner block from its registry name.
#[must_use]
pub fn banner_base_color_from_block_name(block_name: &str) -> Option<DyeColor> {
    let stripped = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
    let color_name = stripped
        .strip_suffix("_wall_banner")
        .or_else(|| stripped.strip_suffix("_banner"))?;
    DyeColor::by_name(color_name)
}

/// Builds the Bedrock sub-compound for sign text (`FrontText` or `BackText`).
#[must_use]
pub fn build_sign_text_compound(sign_text: &SignText) -> NbtCompound {
    let mut compound = NbtCompound::new();
    let messages = sign_text.get_messages(false);
    let text = messages.join("\n");
    compound.put_string("Text", text);
    compound.put_int(
        "SignTextColor",
        dye_color_to_bedrock_sign_text_color(sign_text.get_color()),
    );
    compound.put_bool("IgnoreLighting", sign_text.has_glowing_text());
    compound.put_bool("PersistFormatting", true);
    compound
}

/// Builds the complete Bedrock block-actor NBT for standard or hanging signs.
#[must_use]
pub fn build_sign_actor(
    position: BlockPos,
    is_hanging: bool,
    front_text: &SignText,
    back_text: &SignText,
    is_waxed: bool,
) -> NbtCompound {
    let id = if is_hanging { "HangingSign" } else { "Sign" };
    BedrockBlockActorBuilder::new(id, position)
        .put_compound("FrontText", build_sign_text_compound(front_text))
        .put_compound("BackText", build_sign_text_compound(back_text))
        .put_bool("IsWaxed", is_waxed)
        .finish()
}

/// Builds the Bedrock block-actor NBT for cooking appliances (Furnace, Smoker, `BlastFurnace`).
#[must_use]
pub fn build_furnace_actor(
    id: &'static str,
    position: BlockPos,
    cook_time: u16,
    lit_remaining: u16,
    lit_total: u16,
    stored_xp: i32,
    custom_name: Option<&str>,
) -> NbtCompound {
    BedrockBlockActorBuilder::new(id, position)
        .custom_name(custom_name)
        .put_short("BurnTime", lit_remaining as i16)
        .put_short("BurnDuration", lit_total as i16)
        .put_short("CookTime", cook_time as i16)
        .put_short("StoredXPInt", stored_xp.clamp(0, i16::MAX as i32) as i16)
        .finish()
}

/// Normalizes an item identifier to include the `minecraft:` namespace if not present.
fn bedrock_item_name(registry_key: &str) -> String {
    if registry_key.starts_with("minecraft:") {
        registry_key.to_string()
    } else {
        format!("minecraft:{registry_key}")
    }
}

/// Builds the Bedrock block-actor NBT for campfires.
#[must_use]
pub fn build_campfire_actor(position: BlockPos, items: &[ItemStack; 4]) -> NbtCompound {
    let mut builder = BedrockBlockActorBuilder::new("Campfire", position);
    for (i, item) in items.iter().enumerate() {
        if !item.is_empty() {
            let mut item_comp = NbtCompound::new();
            item_comp.put_string("Name", bedrock_item_name(item.item.registry_key));
            item_comp.put_byte("Count", item.item_count as i8);
            item_comp.put_short("Damage", 0);
            item_comp.put_byte("Slot", i as i8);
            let key = match i {
                0 => "Item1",
                1 => "Item2",
                2 => "Item3",
                _ => "Item4",
            };
            builder = builder.put_compound(key, item_comp);
        }
    }
    builder.finish()
}

/// Builds the Bedrock block-actor NBT for decorated pots.
#[must_use]
pub fn build_decorated_pot_actor(position: BlockPos, sherds: Option<&[NbtTag]>) -> NbtCompound {
    let default_brick = NbtTag::String("minecraft:brick".into());
    let sherd_tags = if let Some(tags) = sherds
        && tags.len() >= 4
    {
        tags.iter().take(4).cloned().collect()
    } else {
        vec![
            default_brick.clone(),
            default_brick.clone(),
            default_brick.clone(),
            default_brick,
        ]
    };

    BedrockBlockActorBuilder::new("DecoratedPot", position)
        .put_list("sherds", sherd_tags)
        .finish()
}

/// Builds the Bedrock block-actor NBT for brewing stands.
#[must_use]
pub fn build_brewing_stand_actor(
    position: BlockPos,
    cook_time: i32,
    fuel: i32,
    custom_name: Option<&str>,
) -> NbtCompound {
    BedrockBlockActorBuilder::new("BrewingStand", position)
        .custom_name(custom_name)
        .put_short("CookTime", cook_time as i16)
        .put_short("FuelAmount", fuel as i16)
        .put_short("FuelTotal", 20)
        .finish()
}

/// Builds the Bedrock block-actor NBT for banners.
#[must_use]
pub fn build_banner_actor(
    position: BlockPos,
    base_color: DyeColor,
    patterns: Option<&[NbtTag]>,
    custom_name: Option<&str>,
) -> NbtCompound {
    let mut builder = BedrockBlockActorBuilder::new("Banner", position)
        .custom_name(custom_name)
        .put_int("Base", dye_color_to_bedrock_wool_color(base_color))
        .put_int("Type", 0);

    if let Some(pats) = patterns {
        let mut bedrock_pats = Vec::new();
        for tag in pats {
            if let Some(compound) = tag.extract_compound() {
                let pattern_name = compound.get_string("pattern").unwrap_or("b");
                let color_idx = compound
                    .get_string("color")
                    .and_then(DyeColor::by_name)
                    .map_or(0, dye_color_to_bedrock_wool_color);

                let mut pat_comp = NbtCompound::new();
                pat_comp.put_string(
                    "Pattern",
                    banner_pattern_to_bedrock_code(pattern_name).to_string(),
                );
                pat_comp.put_int("Color", color_idx);
                bedrock_pats.push(NbtTag::Compound(pat_comp));
            }
        }
        builder = builder.put_list("Patterns", bedrock_pats);
    }

    builder.finish()
}

/// Builds the Bedrock block-actor NBT for shulker boxes.
#[must_use]
pub fn build_shulker_box_actor(
    position: BlockPos,
    facing: u8,
    custom_name: Option<&str>,
) -> NbtCompound {
    BedrockBlockActorBuilder::new("ShulkerBox", position)
        .movable(false)
        .custom_name(custom_name)
        .put_byte("facing", facing as i8)
        .finish()
}

/// Builds the Bedrock block-actor NBT for lecterns.
#[must_use]
pub fn build_lectern_actor(
    position: BlockPos,
    book: Option<&ItemStack>,
    page: i32,
    total_pages: i32,
) -> NbtCompound {
    let has_book = book.is_some_and(|b| !b.is_empty());
    let mut builder = BedrockBlockActorBuilder::new("Lectern", position)
        .put_bool("hasBook", has_book)
        .put_int("page", page)
        .put_int("totalPages", total_pages);

    if has_book && let Some(book_stack) = book {
        let mut book_nbt = NbtCompound::new();
        book_stack.write_item_stack(&mut book_nbt);
        builder = builder.put_compound("book", book_nbt);
    }

    builder.finish()
}

/// Builds the Bedrock block-actor NBT for jukeboxes.
#[must_use]
pub fn build_jukebox_actor(
    position: BlockPos,
    record: Option<&ItemStack>,
    ticks_since_song_started: u64,
) -> NbtCompound {
    let mut builder = BedrockBlockActorBuilder::new("Jukebox", position)
        .put_long("TicksSinceSongStarted", ticks_since_song_started as i64);

    if let Some(record_stack) = record
        && !record_stack.is_empty()
    {
        let mut rec_nbt = NbtCompound::new();
        rec_nbt.put_string("Name", bedrock_item_name(record_stack.item.registry_key));
        rec_nbt.put_byte("Count", 1);
        builder = builder.put_compound("RecordItem", rec_nbt);
    }

    builder.finish()
}

/// Builds the Bedrock block-actor NBT for beacons.
#[must_use]
pub fn build_beacon_actor(
    position: BlockPos,
    primary: i32,
    secondary: i32,
    custom_name: Option<&str>,
) -> NbtCompound {
    BedrockBlockActorBuilder::new("Beacon", position)
        .movable(false)
        .custom_name(custom_name)
        .put_int("primary", primary.max(0))
        .put_int("secondary", secondary.max(0))
        .finish()
}

/// Builds the Bedrock block-actor NBT for command blocks.
#[expect(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
#[must_use]
pub fn build_command_block_actor(
    position: BlockPos,
    command: &str,
    custom_name: Option<&str>,
    auto: bool,
    condition_met: bool,
    conditional: bool,
    powered: bool,
    success_count: u32,
    track_output: bool,
    last_output: &str,
) -> NbtCompound {
    BedrockBlockActorBuilder::new("CommandBlock", position)
        .movable(false)
        .custom_name(custom_name)
        .put_string("Command", command)
        .put_bool("auto", auto)
        .put_bool("conditionMet", condition_met)
        .put_bool("conditionalMode", conditional)
        .put_bool("powered", powered)
        .put_int("SuccessCount", success_count as i32)
        .put_bool("TrackOutput", track_output)
        .put_string("LastOutput", last_output)
        .finish()
}

/// Builds the Bedrock block-actor NBT for standard container blocks (Dispenser, Dropper, Hopper, Barrel).
#[must_use]
pub fn build_container_actor(
    id: &'static str,
    position: BlockPos,
    custom_name: Option<&str>,
) -> NbtCompound {
    BedrockBlockActorBuilder::new(id, position)
        .custom_name(custom_name)
        .finish()
}

/// Builds the Bedrock block-actor NBT for chests and trapped chests with double-chest pairing.
#[must_use]
pub fn build_chest_actor(
    position: BlockPos,
    state_id: BlockStateId,
    custom_name: Option<&str>,
) -> Option<NbtCompound> {
    let (block, _) = BlockState::from_id_with_block(state_id);
    if !block.has_tag(&pumpkin_data::tag::Block::C_CHESTS_WOODEN)
        && !block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_COPPER_CHESTS)
    {
        return None;
    }

    let mut builder = BedrockBlockActorBuilder::new("Chest", position).custom_name(custom_name);

    let properties = ChestLikeProperties::from_state_id(state_id);
    if properties.r#type != ChestType::Single {
        let direction = if properties.r#type == ChestType::Left {
            properties.facing.rotate_clockwise()
        } else {
            properties.facing.rotate_counter_clockwise()
        };
        let pair = position.offset(direction.to_offset());
        builder = builder
            .put_int("pairx", pair.0.x)
            .put_int("pairz", pair.0.z);
        if properties.r#type == ChestType::Right {
            builder = builder.put_bool("pairlead", true);
        }
    }

    Some(builder.finish())
}

#[cfg(test)]
mod tests {
    use pumpkin_data::{Block, dye_color::DyeColor, item::Item, item_stack::ItemStack};
    use pumpkin_nbt::tag::NbtTag;
    use pumpkin_util::math::position::BlockPos;

    use super::*;

    #[test]
    fn sign_actor_builder() {
        let pos = BlockPos::new(1, 64, -2);
        let front = SignText::from_messages([
            Box::from("Welcome"),
            Box::from("To"),
            Box::from("Pumpkin"),
            Box::from("Server"),
        ]);
        front.set_color(DyeColor::Red);
        front.set_has_glowing_text(true);

        let back = SignText::from_messages([
            Box::from("Back"),
            Box::from("Side"),
            Box::from(""),
            Box::from(""),
        ]);
        back.set_color(DyeColor::Blue);

        let nbt = build_sign_actor(pos, false, &front, &back, false);
        assert_eq!(nbt.get_string("id"), Some("Sign"));
        assert_eq!(nbt.get_int("x"), Some(1));
        assert_eq!(nbt.get_int("y"), Some(64));
        assert_eq!(nbt.get_int("z"), Some(-2));
        assert_eq!(nbt.get_bool("isMovable"), Some(true));
        assert_eq!(nbt.get_bool("IsWaxed"), Some(false));

        let front_nbt = nbt.get_compound("FrontText").expect("FrontText missing");
        assert_eq!(
            front_nbt.get_string("Text"),
            Some("Welcome\nTo\nPumpkin\nServer")
        );
        assert_eq!(
            front_nbt.get_int("SignTextColor"),
            Some(dye_color_to_bedrock_sign_text_color(DyeColor::Red))
        );
        assert_eq!(front_nbt.get_bool("IgnoreLighting"), Some(true));

        let back_nbt = nbt.get_compound("BackText").expect("BackText missing");
        assert_eq!(back_nbt.get_string("Text"), Some("Back\nSide\n\n"));
        assert_eq!(
            back_nbt.get_int("SignTextColor"),
            Some(dye_color_to_bedrock_sign_text_color(DyeColor::Blue))
        );
        assert_eq!(back_nbt.get_bool("IgnoreLighting"), Some(false));
    }

    #[test]
    fn hanging_sign_actor_id() {
        let pos = BlockPos::new(0, 70, 0);
        let front = SignText::default();
        let back = SignText::default();
        let nbt = build_sign_actor(pos, true, &front, &back, true);
        assert_eq!(nbt.get_string("id"), Some("HangingSign"));
        assert_eq!(nbt.get_bool("IsWaxed"), Some(true));
    }

    #[test]
    fn furnace_actor_builder() {
        let pos = BlockPos::new(10, 60, 20);
        let nbt = build_furnace_actor("Smoker", pos, 150, 40, 200, 5, Some("Custom Smoker"));
        assert_eq!(nbt.get_string("id"), Some("Smoker"));
        assert_eq!(nbt.get_string("CustomName"), Some("Custom Smoker"));
        assert_eq!(nbt.get_short("CookTime"), Some(150));
        assert_eq!(nbt.get_short("BurnTime"), Some(40));
        assert_eq!(nbt.get_short("BurnDuration"), Some(200));
        assert_eq!(nbt.get_short("StoredXPInt"), Some(5));
        assert_eq!(nbt.get_bool("isMovable"), Some(true));
    }

    #[test]
    fn campfire_actor_builder() {
        let pos = BlockPos::new(5, 65, 5);
        let beef = ItemStack::new(1, &Item::BEEF);
        let pork = ItemStack::new(1, &Item::PORKCHOP);
        let items = [
            beef,
            ItemStack::EMPTY.clone(),
            pork,
            ItemStack::EMPTY.clone(),
        ];
        let nbt = build_campfire_actor(pos, &items);

        assert_eq!(nbt.get_string("id"), Some("Campfire"));
        let item1 = nbt.get_compound("Item1").expect("Item1 missing");
        assert_eq!(item1.get_string("Name"), Some("minecraft:beef"));
        assert_eq!(item1.get_byte("Count"), Some(1));
        assert_eq!(item1.get_byte("Slot"), Some(0));

        assert!(nbt.get_compound("Item2").is_none());

        let item3 = nbt.get_compound("Item3").expect("Item3 missing");
        assert_eq!(item3.get_string("Name"), Some("minecraft:porkchop"));
        assert_eq!(item3.get_byte("Count"), Some(1));
        assert_eq!(item3.get_byte("Slot"), Some(2));
    }

    #[test]
    fn decorated_pot_actor_builder() {
        let pos = BlockPos::new(2, 64, 2);
        let sherds = vec![
            NbtTag::String("minecraft:archer_pottery_sherd".into()),
            NbtTag::String("minecraft:brick".into()),
            NbtTag::String("minecraft:prize_pottery_sherd".into()),
            NbtTag::String("minecraft:miner_pottery_sherd".into()),
        ];
        let nbt = build_decorated_pot_actor(pos, Some(&sherds));
        assert_eq!(nbt.get_string("id"), Some("DecoratedPot"));
        let list = nbt.get_list("sherds").expect("sherds missing");
        assert_eq!(list.len(), 4);
        assert_eq!(
            list[0].extract_string(),
            Some("minecraft:archer_pottery_sherd")
        );
        assert_eq!(list[1].extract_string(), Some("minecraft:brick"));
    }

    #[test]
    fn banner_actor_builder() {
        let pos = BlockPos::new(3, 64, 3);
        let mut pat_comp = NbtCompound::new();
        pat_comp.put_string("pattern", "stripe_top".to_string());
        pat_comp.put_string("color", "black".to_string());
        let patterns = vec![NbtTag::Compound(pat_comp)];

        let nbt = build_banner_actor(pos, DyeColor::Red, Some(&patterns), None);
        assert_eq!(nbt.get_string("id"), Some("Banner"));
        assert_eq!(
            nbt.get_int("Base"),
            Some(dye_color_to_bedrock_wool_color(DyeColor::Red))
        );

        let pat_list = nbt.get_list("Patterns").expect("Patterns missing");
        assert_eq!(pat_list.len(), 1);
        let first = pat_list[0].extract_compound().expect("compound");
        assert_eq!(first.get_string("Pattern"), Some("ts"));
    }

    #[test]
    fn shulker_box_actor_builder() {
        let pos = BlockPos::new(4, 64, 4);
        let nbt = build_shulker_box_actor(pos, 1, Some("Box"));
        assert_eq!(nbt.get_string("id"), Some("ShulkerBox"));
        assert_eq!(nbt.get_byte("facing"), Some(1));
        assert_eq!(nbt.get_bool("isMovable"), Some(false));
        assert_eq!(nbt.get_string("CustomName"), Some("Box"));
    }

    #[test]
    fn beacon_actor_builder() {
        let pos = BlockPos::new(5, 64, 5);
        let nbt = build_beacon_actor(pos, 1, 1, None);
        assert_eq!(nbt.get_string("id"), Some("Beacon"));
        assert_eq!(nbt.get_int("primary"), Some(1));
        assert_eq!(nbt.get_int("secondary"), Some(1));
        assert_eq!(nbt.get_bool("isMovable"), Some(false));
    }

    #[test]
    fn command_block_actor_builder() {
        let pos = BlockPos::new(6, 64, 6);
        let nbt = build_command_block_actor(
            pos,
            "say hi",
            Some("Cmd"),
            true,
            true,
            false,
            false,
            1,
            true,
            "hi",
        );
        assert_eq!(nbt.get_string("id"), Some("CommandBlock"));
        assert_eq!(nbt.get_string("Command"), Some("say hi"));
        assert_eq!(nbt.get_string("CustomName"), Some("Cmd"));
        assert_eq!(nbt.get_bool("auto"), Some(true));
        assert_eq!(nbt.get_bool("conditionMet"), Some(true));
        assert_eq!(nbt.get_bool("conditionalMode"), Some(false));
        assert_eq!(nbt.get_bool("powered"), Some(false));
        assert_eq!(nbt.get_int("SuccessCount"), Some(1));
        assert_eq!(nbt.get_bool("isMovable"), Some(false));
    }

    #[test]
    fn chest_actor_builder() {
        let pos = BlockPos::new(7, 64, 7);
        let single = build_chest_actor(pos, Block::CHEST.default_state.id, None).expect("chest");
        assert_eq!(single.get_string("id"), Some("Chest"));
        assert_eq!(single.get_bool("isMovable"), Some(true));
        assert!(single.get_int("pairx").is_none());
    }

    #[test]
    fn jukebox_actor_builder() {
        let pos = BlockPos::new(8, 64, 8);
        let record = ItemStack::new(1, &Item::MUSIC_DISC_CAT);
        let nbt = build_jukebox_actor(pos, Some(&record), 100);
        assert_eq!(nbt.get_string("id"), Some("Jukebox"));
        assert_eq!(nbt.get_long("TicksSinceSongStarted"), Some(100));
        let rec = nbt.get_compound("RecordItem").expect("RecordItem missing");
        assert_eq!(rec.get_string("Name"), Some("minecraft:music_disc_cat"));
    }

    #[test]
    fn brewing_stand_actor_builder() {
        let pos = BlockPos::new(9, 64, 9);
        let nbt = build_brewing_stand_actor(pos, 200, 15, Some("Brew"));
        assert_eq!(nbt.get_string("id"), Some("BrewingStand"));
        assert_eq!(nbt.get_short("CookTime"), Some(200));
        assert_eq!(nbt.get_short("FuelAmount"), Some(15));
        assert_eq!(nbt.get_short("FuelTotal"), Some(20));
        assert_eq!(nbt.get_string("CustomName"), Some("Brew"));
    }

    #[test]
    fn lectern_actor_builder() {
        let pos = BlockPos::new(10, 64, 10);
        let book = ItemStack::new(1, &Item::WRITTEN_BOOK);
        let nbt = build_lectern_actor(pos, Some(&book), 2, 5);
        assert_eq!(nbt.get_string("id"), Some("Lectern"));
        assert_eq!(nbt.get_bool("hasBook"), Some(true));
        assert_eq!(nbt.get_int("page"), Some(2));
        assert_eq!(nbt.get_int("totalPages"), Some(5));
        assert!(nbt.get_compound("book").is_some());
    }
}
