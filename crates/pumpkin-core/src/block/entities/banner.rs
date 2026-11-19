use super::BlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

pub struct BannerBlockEntity {
    pub position: BlockPos,
    pub custom_name: Mutex<Option<String>>,
    pub patterns: Mutex<Option<Vec<NbtTag>>>,
}

impl BlockEntity for BannerBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let custom_name = nbt.get_string("CustomName").map(ToString::to_string);
        let patterns = nbt.get_list("patterns").map(<[_]>::to_vec);
        Self {
            position,
            custom_name: Mutex::new(custom_name),
            patterns: Mutex::new(patterns),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(name) = self.custom_name.lock()
            && let Some(name) = name.as_ref()
        {
            nbt.put_string("CustomName", name.clone());
        }
        if let Ok(pats) = self.patterns.lock()
            && let Some(pats) = pats.as_ref()
        {
            nbt.put_list("patterns", pats.clone());
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(name) = self.custom_name.try_lock()
            && let Some(ref name) = *name
        {
            nbt.put_string("CustomName", name.clone());
        }
        if let Ok(patterns) = self.patterns.try_lock()
            && let Some(ref pats) = *patterns
        {
            nbt.put_list("patterns", pats.clone());
        }
        Some(nbt)
    }

    fn bedrock_block_actor_data(
        &self,
        state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        let (block, _) = pumpkin_data::BlockState::from_id_with_block(state_id);
        let base_color = banner_base_color_from_block_name(block.name)
            .unwrap_or(pumpkin_data::dye_color::DyeColor::White);
        let patterns_guard = self.patterns.lock().ok();
        let patterns = patterns_guard.as_ref().and_then(|p| p.as_deref());
        let name_guard = self.custom_name.lock().ok();
        let name = name_guard.as_ref().and_then(|n| n.as_deref());

        Some(build_banner_actor(
            self.position,
            base_color,
            patterns,
            name,
        ))
    }

    fn collect_item_components(&self, stack: &mut pumpkin_data::item_stack::ItemStack) {
        if let Ok(pats) = self.patterns.lock()
            && let Some(pats) = pats.as_ref()
            && !pats.is_empty()
            && let Some(patterns) = pumpkin_data::data_component_impl::BannerPatternsImpl::read_data(
                &NbtTag::List(pats.clone()),
            )
        {
            stack.set_data_component(patterns);
        }
        if let Ok(name) = self.custom_name.lock()
            && let Some(name) = name.as_ref()
        {
            stack.set_data_component(pumpkin_data::data_component_impl::CustomNameImpl {
                name: pumpkin_util::text::TextComponent::text(name.clone()),
            });
        }
    }

    fn apply_item_components(&self, stack: &pumpkin_data::item_stack::ItemStack) {
        if let Some(patterns) =
            stack.get_data_component::<pumpkin_data::data_component_impl::BannerPatternsImpl>()
            && !patterns.layers.is_empty()
        {
            use pumpkin_data::data_component_impl::DataComponentImpl;
            if let NbtTag::List(pats) = patterns.write_data() {
                *self
                    .patterns
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(pats);
            }
        }
        if let Some(custom_name) =
            stack.get_data_component::<pumpkin_data::data_component_impl::CustomNameImpl>()
        {
            *self
                .custom_name
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                Some(custom_name.name.clone().get_text());
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BannerBlockEntity {
    pub const ID: &'static str = "minecraft:banner";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            custom_name: Mutex::new(None),
            patterns: Mutex::new(None),
        }
    }
}

/// Converts a Java [`pumpkin_data::dye_color::DyeColor`] to Bedrock's inverted wool/banner metadata color index.
#[must_use]
pub const fn dye_color_to_bedrock_wool_color(color: pumpkin_data::dye_color::DyeColor) -> i32 {
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

/// Extracts the base [`pumpkin_data::dye_color::DyeColor`] of a banner block from its registry name.
#[must_use]
pub fn banner_base_color_from_block_name(
    block_name: &str,
) -> Option<pumpkin_data::dye_color::DyeColor> {
    let stripped = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
    let color_name = stripped
        .strip_suffix("_wall_banner")
        .or_else(|| stripped.strip_suffix("_banner"))?;
    pumpkin_data::dye_color::DyeColor::by_name(color_name)
}

/// Builds the Bedrock block-actor NBT for banners.
#[must_use]
pub fn build_banner_actor(
    position: BlockPos,
    base_color: pumpkin_data::dye_color::DyeColor,
    patterns: Option<&[NbtTag]>,
    custom_name: Option<&str>,
) -> NbtCompound {
    let mut builder = super::bedrock::BedrockBlockActorBuilder::new("Banner", position)
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
                    .and_then(pumpkin_data::dye_color::DyeColor::by_name)
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

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::dye_color::DyeColor;

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
}
