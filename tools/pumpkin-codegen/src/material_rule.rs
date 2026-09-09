use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

/// Deserialized block reference used in material rules.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum BlockStateCodecStruct {
    Name(String),
    Structured {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Properties")]
        #[allow(dead_code)]
        properties: Option<BTreeMap<String, String>>,
    },
}

impl ToTokens for BlockStateCodecStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Self::Name(name) => name,
            Self::Structured { name, .. } => name,
        };
        let name_stripped = name.strip_prefix("minecraft:").unwrap_or(name);
        let block_ident =
            quote::format_ident!("{}", name_stripped.to_uppercase().replace([':', '-'], "_"));
        tokens.extend(quote! {
            crate::Block::#block_ident.default_state
        });
    }
}

/// Deserialized Y offset that can be expressed relative to different reference points.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum YOffsetStruct {
    Absolute { absolute: i16 },
    AboveBottom { above_bottom: i8 },
    BelowTop { below_top: i8 },
}

impl ToTokens for YOffsetStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Absolute { absolute } => {
                tokens.extend(quote!(YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: #absolute })));
            }
            Self::AboveBottom { above_bottom } => {
                tokens.extend(quote!(YOffset::AboveBottom(pumpkin_util::y_offset::AboveBottom { above_bottom: #above_bottom })));
            }
            Self::BelowTop { below_top } => {
                tokens.extend(quote!(YOffset::BelowTop(pumpkin_util::y_offset::BelowTop { below_top: #below_top })));
            }
        }
    }
}

/// Raw deserialized surface material condition from `material_condition` or inline in `material_rule`.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum RawMaterialCondition {
    Ref(String),
    Direct(DirectMaterialCondition),
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum DirectMaterialCondition {
    #[serde(rename = "minecraft:biome")]
    Biome {
        #[serde(deserialize_with = "deserialize_string_or_vec")]
        biome_is: Vec<String>,
    },
    #[serde(rename = "minecraft:noise_threshold")]
    NoiseThreshold {
        noise: String,
        min_threshold: f64,
        max_threshold: f64,
        #[serde(default)]
        is_3d: bool,
    },
    #[serde(rename = "minecraft:vertical_gradient")]
    VerticalGradient {
        random_name: String,
        true_at_and_below: YOffsetStruct,
        false_at_and_above: YOffsetStruct,
    },
    #[serde(rename = "minecraft:y_above")]
    YAbove {
        anchor: YOffsetStruct,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    #[serde(rename = "minecraft:water")]
    Water {
        offset: i32,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    #[serde(rename = "minecraft:temperature")]
    Temperature,
    #[serde(rename = "minecraft:steep")]
    Steep,
    #[serde(rename = "minecraft:not")]
    Not { invert: Box<RawMaterialCondition> },
    #[serde(rename = "minecraft:hole")]
    Hole,
    #[serde(rename = "minecraft:above_preliminary_surface")]
    AbovePreliminarySurface,
    #[serde(rename = "minecraft:stone_depth")]
    StoneDepth {
        offset: i32,
        add_surface_depth: bool,
        secondary_depth_range: i32,
        surface_type: String,
    },
}

fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> serde::de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: serde::de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(el) = seq.next_element()? {
                vec.push(el);
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_any(StringOrVec)
}

/// Raw deserialized material rule from `material_rule` folder or inline.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum RawMaterialRule {
    Ref(String),
    Direct(DirectMaterialRule),
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum DirectMaterialRule {
    #[serde(rename = "minecraft:block")]
    Block { result_state: BlockStateCodecStruct },
    #[serde(rename = "minecraft:sequence")]
    Sequence { sequence: Vec<RawMaterialRule> },
    #[serde(rename = "minecraft:condition")]
    Condition {
        if_true: RawMaterialCondition,
        then_run: Box<RawMaterialRule>,
    },
    #[serde(rename = "minecraft:bandlands", alias = "minecraft:badlands")]
    Badlands,
    #[serde(rename = "minecraft:ore_vein")]
    OreVein(serde_json::Value),
}

/// Deserialized surface material condition that gates a material rule.
#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MaterialConditionStruct {
    #[serde(rename = "minecraft:biome")]
    Biome {
        #[serde(deserialize_with = "deserialize_string_or_vec")]
        biome_is: Vec<String>,
    },
    #[serde(rename = "minecraft:noise_threshold")]
    NoiseThreshold {
        noise: String,
        min_threshold: f64,
        max_threshold: f64,
    },
    #[serde(rename = "minecraft:vertical_gradient")]
    VerticalGradient {
        random_name: String,
        true_at_and_below: YOffsetStruct,
        false_at_and_above: YOffsetStruct,
    },
    #[serde(rename = "minecraft:y_above")]
    YAbove {
        anchor: YOffsetStruct,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    #[serde(rename = "minecraft:water")]
    Water {
        offset: i32,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    #[serde(rename = "minecraft:temperature")]
    Temperature,
    #[serde(rename = "minecraft:steep")]
    Steep,
    #[serde(rename = "minecraft:not")]
    Not { invert: Box<Self> },
    #[serde(rename = "minecraft:hole")]
    Hole,
    #[serde(rename = "minecraft:above_preliminary_surface")]
    AbovePreliminarySurface,
    #[serde(rename = "minecraft:stone_depth")]
    StoneDepth {
        offset: i32,
        add_surface_depth: bool,
        secondary_depth_range: i32,
        surface_type: String,
    },
}

impl ToTokens for MaterialConditionStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Biome { biome_is } => {
                let biomes = biome_is
                    .iter()
                    .map(|b| b.strip_prefix("minecraft:").unwrap_or(b).to_uppercase());
                let biome_refs: Vec<TokenStream> = biomes
                    .map(|b| {
                        let ident = format_ident!("{}", b);
                        quote!(&crate::biome::Biome::#ident)
                    })
                    .collect();

                tokens.extend(quote!(
                    MaterialCondition::Biome(BiomeMaterialCondition {
                        biome_is: &[#(#biome_refs),*],
                    })
                ));
            }
            Self::NoiseThreshold {
                noise,
                min_threshold,
                max_threshold,
            } => {
                let noise_id = quote::format_ident!(
                    "{}",
                    noise
                        .strip_prefix("minecraft:")
                        .unwrap()
                        .to_shouty_snake_case()
                );

                tokens.extend(quote!(
                    MaterialCondition::NoiseThreshold(NoiseThresholdMaterialCondition {
                        noise: DoublePerlinNoiseParameters::#noise_id,
                        min_threshold: #min_threshold,
                        max_threshold: #max_threshold,
                    })
                ));
            }
            Self::VerticalGradient {
                random_name,
                true_at_and_below,
                false_at_and_above,
            } => {
                let bytes = md5::compute(random_name.as_bytes());
                let lo = u64::from_le_bytes(bytes[0..8].try_into().expect("incorrect length"));
                let hi = u64::from_le_bytes(bytes[8..16].try_into().expect("incorrect length"));
                tokens.extend(quote!(
                    MaterialCondition::VerticalGradient(VerticalGradientMaterialCondition {
                        random_lo: #lo,
                        random_hi: #hi,
                        true_at_and_below: #true_at_and_below,
                        false_at_and_above: #false_at_and_above,
                    })
                ));
            }
            Self::YAbove {
                anchor,
                surface_depth_multiplier,
                add_stone_depth,
            } => {
                tokens.extend(quote!(
                    MaterialCondition::YAbove(AboveYMaterialCondition {
                        anchor: #anchor,
                        surface_depth_multiplier: #surface_depth_multiplier,
                        add_stone_depth: #add_stone_depth,
                    })
                ));
            }
            Self::Water {
                offset,
                surface_depth_multiplier,
                add_stone_depth,
            } => {
                tokens.extend(quote!(
                    MaterialCondition::Water(WaterMaterialCondition {
                        offset: #offset,
                        surface_depth_multiplier: #surface_depth_multiplier,
                        add_stone_depth: #add_stone_depth,
                    })
                ));
            }
            Self::Temperature => {
                tokens.extend(quote!(MaterialCondition::Temperature));
            }
            Self::Steep => {
                tokens.extend(quote!(MaterialCondition::Steep));
            }
            Self::Not { invert } => {
                tokens.extend(quote!(
                    MaterialCondition::Not(NotMaterialCondition {
                        invert: &#invert,
                    })
                ));
            }
            Self::Hole => {
                tokens.extend(quote!(MaterialCondition::Hole(HoleMaterialCondition)));
            }
            Self::AbovePreliminarySurface => {
                tokens.extend(quote!(MaterialCondition::AbovePreliminarySurface(
                    SurfaceMaterialCondition
                )));
            }
            Self::StoneDepth {
                offset,
                add_surface_depth,
                secondary_depth_range,
                surface_type,
            } => {
                let surface_type_token = match surface_type.as_str() {
                    "ceiling" => quote!(
                        pumpkin_util::math::vertical_surface_type::VerticalSurfaceType::Ceiling
                    ),
                    "floor" => quote!(
                        pumpkin_util::math::vertical_surface_type::VerticalSurfaceType::Floor
                    ),
                    _ => quote!(panic!("Unknown surface type")),
                };

                tokens.extend(quote!(
                    MaterialCondition::StoneDepth(StoneDepthMaterialCondition {
                        offset: #offset,
                        add_surface_depth: #add_surface_depth,
                        secondary_depth_range: #secondary_depth_range,
                        surface_type: #surface_type_token,
                    })
                ));
            }
        }
    }
}

/// Deserialized surface material rule that determines which block to place at a given surface point.
#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MaterialRuleStruct {
    #[serde(rename = "minecraft:block")]
    Block { result_state: BlockStateCodecStruct },
    #[serde(rename = "minecraft:sequence")]
    Sequence { sequence: Vec<Self> },
    #[serde(rename = "minecraft:condition")]
    Condition {
        if_true: MaterialConditionStruct,
        then_run: Box<Self>,
    },
    #[serde(rename = "minecraft:bandlands")]
    Badlands,
}

impl ToTokens for MaterialRuleStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Block { result_state } => {
                tokens.extend(quote!(
                    MaterialRule::Block(BlockMaterialRule {
                        result_state: #result_state
                    })
                ));
            }
            Self::Sequence { sequence } => {
                tokens.extend(quote!(
                    MaterialRule::Sequence(SequenceMaterialRule {
                        sequence: &[#(#sequence),*]
                    })
                ));
            }
            Self::Condition { if_true, then_run } => {
                tokens.extend(quote!(
                    MaterialRule::Condition(ConditionMaterialRule {
                        if_true: #if_true,
                        then_run: &#then_run
                    })
                ));
            }
            Self::Badlands => {
                tokens.extend(quote!(MaterialRule::Badlands(BadLandsMaterialRule)));
            }
        }
    }
}

pub fn resolve_condition(
    raw: &RawMaterialCondition,
    cond_dir: &std::path::Path,
) -> MaterialConditionStruct {
    match raw {
        RawMaterialCondition::Ref(r) => {
            let ref_name = r.strip_prefix("minecraft:").unwrap_or(r);
            let path = cond_dir.join(format!("{}.json", ref_name));
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read condition file {:?}", path));
            let parsed: RawMaterialCondition = serde_json::from_str(&content)
                .unwrap_or_else(|e| panic!("Failed to parse condition file {:?}: {}", path, e));
            resolve_condition(&parsed, cond_dir)
        }
        RawMaterialCondition::Direct(direct) => match direct {
            DirectMaterialCondition::Biome { biome_is } => MaterialConditionStruct::Biome {
                biome_is: biome_is.clone(),
            },
            DirectMaterialCondition::NoiseThreshold {
                noise,
                min_threshold,
                max_threshold,
                is_3d: _,
            } => MaterialConditionStruct::NoiseThreshold {
                noise: noise.clone(),
                min_threshold: *min_threshold,
                max_threshold: *max_threshold,
            },
            DirectMaterialCondition::VerticalGradient {
                random_name,
                true_at_and_below,
                false_at_and_above,
            } => MaterialConditionStruct::VerticalGradient {
                random_name: random_name.clone(),
                true_at_and_below: true_at_and_below.clone(),
                false_at_and_above: false_at_and_above.clone(),
            },
            DirectMaterialCondition::YAbove {
                anchor,
                surface_depth_multiplier,
                add_stone_depth,
            } => MaterialConditionStruct::YAbove {
                anchor: anchor.clone(),
                surface_depth_multiplier: *surface_depth_multiplier,
                add_stone_depth: *add_stone_depth,
            },
            DirectMaterialCondition::Water {
                offset,
                surface_depth_multiplier,
                add_stone_depth,
            } => MaterialConditionStruct::Water {
                offset: *offset,
                surface_depth_multiplier: *surface_depth_multiplier,
                add_stone_depth: *add_stone_depth,
            },
            DirectMaterialCondition::Temperature => MaterialConditionStruct::Temperature,
            DirectMaterialCondition::Steep => MaterialConditionStruct::Steep,
            DirectMaterialCondition::Not { invert } => MaterialConditionStruct::Not {
                invert: Box::new(resolve_condition(invert, cond_dir)),
            },
            DirectMaterialCondition::Hole => MaterialConditionStruct::Hole,
            DirectMaterialCondition::AbovePreliminarySurface => {
                MaterialConditionStruct::AbovePreliminarySurface
            }
            DirectMaterialCondition::StoneDepth {
                offset,
                add_surface_depth,
                secondary_depth_range,
                surface_type,
            } => MaterialConditionStruct::StoneDepth {
                offset: *offset,
                add_surface_depth: *add_surface_depth,
                secondary_depth_range: *secondary_depth_range,
                surface_type: surface_type.clone(),
            },
        },
    }
}

pub fn resolve_rule(
    raw: &RawMaterialRule,
    rule_dir: &std::path::Path,
    cond_dir: &std::path::Path,
) -> Option<MaterialRuleStruct> {
    match raw {
        RawMaterialRule::Ref(r) => {
            let ref_name = r.strip_prefix("minecraft:").unwrap_or(r);
            let path = rule_dir.join(format!("{}.json", ref_name));
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read rule file {:?}", path));
            let parsed: RawMaterialRule = serde_json::from_str(&content)
                .unwrap_or_else(|e| panic!("Failed to parse rule file {:?}: {}", path, e));
            resolve_rule(&parsed, rule_dir, cond_dir)
        }
        RawMaterialRule::Direct(direct) => match direct {
            DirectMaterialRule::Block { result_state } => Some(MaterialRuleStruct::Block {
                result_state: result_state.clone(),
            }),
            DirectMaterialRule::Sequence { sequence } => {
                let resolved_seq: Vec<MaterialRuleStruct> = sequence
                    .iter()
                    .filter_map(|r| resolve_rule(r, rule_dir, cond_dir))
                    .collect();
                Some(MaterialRuleStruct::Sequence {
                    sequence: resolved_seq,
                })
            }
            DirectMaterialRule::Condition { if_true, then_run } => {
                let cond = resolve_condition(if_true, cond_dir);
                let resolved_then = resolve_rule(then_run, rule_dir, cond_dir)?;
                Some(MaterialRuleStruct::Condition {
                    if_true: cond,
                    then_run: Box::new(resolved_then),
                })
            }
            DirectMaterialRule::Badlands => Some(MaterialRuleStruct::Badlands),
            DirectMaterialRule::OreVein(_) => None,
        },
    }
}

/// Reads material_rule files and resolves conditions from material_condition folder.
pub fn build() -> TokenStream {
    let rule_dir =
        std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/worldgen/material_rule");
    let cond_dir = std::path::Path::new(
        "../../assets/datapacks/26_2/data/minecraft/worldgen/material_condition",
    );

    let top_level_rules = [
        "bedrock_floor",
        "bedrock_roof",
        "end",
        "nether",
        "overworld",
        "overworld_caves",
        "overworld_floating_islands",
    ];

    let mut const_defs = TokenStream::new();

    for rule_name in top_level_rules {
        let raw = RawMaterialRule::Ref(format!("minecraft:{}", rule_name));
        let resolved = resolve_rule(&raw, rule_dir, cond_dir)
            .unwrap_or_else(|| panic!("Failed to resolve material rule {}", rule_name));

        let const_ident = format_ident!("{}", rule_name.to_uppercase());
        const_defs.extend(quote!(
            pub const #const_ident: MaterialRule = #resolved;
        ));
    }

    quote!(
        use crate::chunk::DoublePerlinNoiseParameters;
        use crate::BlockState;
        use crate::biome::Biome;
        use crate::dimension::Dimension;
        use pumpkin_util::y_offset::YOffset;
        use pumpkin_util::y_offset::Absolute;

        pub struct BlockMaterialRule {
            pub result_state: &'static BlockState,
        }

        pub struct SequenceMaterialRule {
            pub sequence: &'static [MaterialRule],
        }

        pub struct ConditionMaterialRule {
            pub if_true: MaterialCondition,
            pub then_run: &'static MaterialRule,
        }

        pub struct BadLandsMaterialRule;

        pub enum MaterialRule {
            Block(BlockMaterialRule),
            Sequence(SequenceMaterialRule),
            Condition(ConditionMaterialRule),
            Badlands(BadLandsMaterialRule),
        }

        impl MaterialRule {
            #[must_use]
            pub fn from_dimension(dimension: &Dimension) -> &'static Self {
                if dimension == &Dimension::OVERWORLD {
                    &OVERWORLD
                } else if dimension == &Dimension::THE_NETHER {
                    &NETHER
                } else {
                    &END
                }
            }
        }

        pub struct BiomeMaterialCondition {
            pub biome_is: &'static [&'static Biome],
        }

        pub struct NoiseThresholdMaterialCondition {
            pub noise: DoublePerlinNoiseParameters,
            pub min_threshold: f64,
            pub max_threshold: f64,
        }

        pub struct VerticalGradientMaterialCondition {
            pub random_lo: u64,
            pub random_hi: u64,
            pub true_at_and_below: YOffset,
            pub false_at_and_above: YOffset,
        }

        pub struct AboveYMaterialCondition {
            pub anchor: YOffset,
            pub surface_depth_multiplier: i32,
            pub add_stone_depth: bool,
        }

        pub struct WaterMaterialCondition {
            pub offset: i32,
            pub surface_depth_multiplier: i32,
            pub add_stone_depth: bool,
        }

        pub struct HoleMaterialCondition;

        pub struct NotMaterialCondition {
            pub invert: &'static MaterialCondition,
        }

        pub struct SurfaceMaterialCondition;

        pub struct StoneDepthMaterialCondition {
            pub offset: i32,
            pub add_surface_depth: bool,
            pub secondary_depth_range: i32,
            pub surface_type: pumpkin_util::math::vertical_surface_type::VerticalSurfaceType,
        }

        pub enum MaterialCondition {
            Biome(BiomeMaterialCondition),
            NoiseThreshold(NoiseThresholdMaterialCondition),
            VerticalGradient(VerticalGradientMaterialCondition),
            YAbove(AboveYMaterialCondition),
            Water(WaterMaterialCondition),
            Temperature,
            Steep,
            Not(NotMaterialCondition),
            Hole(HoleMaterialCondition),
            AbovePreliminarySurface(SurfaceMaterialCondition),
            StoneDepth(StoneDepthMaterialCondition),
        }

        #const_defs
    )
}
