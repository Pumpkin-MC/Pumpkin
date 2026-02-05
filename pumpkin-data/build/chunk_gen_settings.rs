use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

#[derive(Deserialize)]
pub struct BlockStateCodecStruct {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Properties")]
    pub properties: Option<BTreeMap<String, String>>,
}

#[derive(Deserialize)]
pub struct GenerationSettingsStruct {
    #[serde(default)]
    pub aquifers_enabled: bool,
    #[serde(default)]
    pub ore_veins_enabled: bool,
    #[serde(default)]
    pub legacy_random_source: bool,
    pub sea_level: i32,
    pub default_fluid: BlockStateCodecStruct,
    pub default_block: BlockStateCodecStruct,
    #[serde(rename = "noise")]
    pub shape: GenerationShapeConfigStruct,
    pub surface_rule: MaterialRuleStruct,
}

#[derive(Deserialize)]
pub struct GenerationShapeConfigStruct {
    pub min_y: i8,
    pub height: u16,
    pub size_horizontal: u8,
    pub size_vertical: u8,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum MaterialRuleStruct {
    #[serde(rename = "minecraft:block")]
    Block { result_state: BlockStateCodecStruct },
    #[serde(rename = "minecraft:sequence")]
    Sequence { sequence: Vec<MaterialRuleStruct> },
    #[serde(rename = "minecraft:condition")]
    Condition {
        if_true: MaterialConditionStruct,
        then_run: Box<MaterialRuleStruct>,
    },
    #[serde(rename = "minecraft:bandlands")]
    Badlands,
    #[serde(other)]
    Unsupported,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum MaterialConditionStruct {
    #[serde(rename = "minecraft:biome")]
    Biome { biome_is: Vec<String> },
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
    Not {
        invert: Box<MaterialConditionStruct>,
    },
    #[serde(rename = "minecraft:hole")]
    Hole,
    #[serde(rename = "minecraft:above_preliminary_surface")]
    AbovePreliminarySurface,
    #[serde(rename = "minecraft:stone_depth")]
    StoneDepth {
        offset: i32,
        add_surface_depth: bool,
        secondary_depth_range: i32,
        surface_type: String, // "ceiling" or "floor"
    },
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum YOffsetStruct {
    Absolute { absolute: i16 },
    AboveBottom { above_bottom: i8 },
    BelowTop { below_top: i8 },
}

// --- ToTokens Implementations ---

impl ToTokens for BlockStateCodecStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name.strip_prefix("minecraft:").unwrap_or(&self.name);

        let props_gen = if let Some(props) = &self.properties {
            let keys = props.keys();
            let values = props.values();
            quote!(Some(&[#((#keys, #values)),*]))
        } else {
            quote!(None)
        };

        tokens.extend(quote!(
            BlockBlueprint {
                name: #name,
                properties: #props_gen,
            }
        ));
    }
}

impl ToTokens for GenerationSettingsStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let aquifers = self.aquifers_enabled;
        let ores = self.ore_veins_enabled;
        let legacy = self.legacy_random_source;
        let sea_level = self.sea_level;
        let fluid = &self.default_fluid;
        let block = &self.default_block;
        let shape = &self.shape;
        let rule = &self.surface_rule;

        tokens.extend(quote!(
            GenerationSettings {
                aquifers_enabled: #aquifers,
                ore_veins_enabled: #ores,
                legacy_random_source: #legacy,
                sea_level: #sea_level,
                default_fluid: #fluid,
                shape: #shape,
                surface_rule: #rule,
                default_block: #block,
            }
        ));
    }
}

impl ToTokens for GenerationShapeConfigStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min_y = self.min_y;
        let height = self.height;
        let hor = self.size_horizontal;
        let ver = self.size_vertical;
        tokens.extend(quote!(
            GenerationShapeConfig { min_y: #min_y, height: #height, size_horizontal: #hor, size_vertical: #ver }
        ));
    }
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
                tokens.extend(quote!(
                    MaterialCondition::NoiseThreshold(NoiseThresholdMaterialCondition {
                        noise: #noise,
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
                tokens.extend(quote!(
                    MaterialCondition::VerticalGradient(VerticalGradientMaterialCondition {
                        random_name: #random_name,
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
                    _ => quote!(
                        pumpkin_util::math::vertical_surface_type::VerticalSurfaceType::Floor
                    ),
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
            Self::Unsupported => {
                tokens.extend(quote!(MaterialRule::Unsupported));
            }
        }
    }
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/chunk_gen_settings.json");

    let json: BTreeMap<String, GenerationSettingsStruct> =
        serde_json::from_str(&fs::read_to_string("../assets/chunk_gen_settings.json").unwrap())
            .expect("Failed to parse settings.json");

    let mut const_defs = TokenStream::new();

    for (name, settings) in &json {
        let upper_name = name.to_uppercase();
        let const_name = format_ident!("{}", upper_name);

        const_defs.extend(quote!(
            pub const #const_name: GenerationSettings = #settings;
        ));
    }

    quote!(
        use crate::dimension::Dimension;
        use std::{cell::RefCell, num::NonZeroUsize};
        use pumpkin_util::random::RandomDeriver;
        use pumpkin_util::y_offset::YOffset;
        use crate::biome::Biome;
        use pumpkin_util::y_offset::Absolute;

        pub struct BlockBlueprint {
            pub name: &'static str,
            pub properties: Option<&'static [(&'static str, &'static str)]>,
        }

        pub struct GenerationSettings {
            pub aquifers_enabled: bool,
            pub ore_veins_enabled: bool,
            pub legacy_random_source: bool,
            pub sea_level: i32,
            pub default_fluid: BlockBlueprint,
            pub shape: GenerationShapeConfig,
            pub surface_rule: MaterialRule,
            pub default_block: BlockBlueprint,
        }

        pub struct GenerationShapeConfig {
            pub min_y: i8,
            pub height: u16,
            pub size_horizontal: u8,
            pub size_vertical: u8,
        }

        impl GenerationShapeConfig {
            #[inline] #[must_use]
            pub fn vertical_cell_block_count(&self) -> u8 { self.size_vertical << 2 }

            #[inline] #[must_use]
            pub fn horizontal_cell_block_count(&self) -> u8 { self.size_horizontal << 2 }

            #[must_use]
            pub const fn max_y(&self) -> u16 {
                if self.min_y >= 0 {
                    self.height + self.min_y as u16
                } else {
                    (self.height as i32 + self.min_y as i32) as u16
                }
            }

            pub fn trim_height(&self, bottom_y: i8, top_y: u16) -> Self {
                let new_min = self.min_y.max(bottom_y);
                let this_top = if self.min_y >= 0 {
                    self.height + self.min_y as u16
                } else {
                    self.height - self.min_y.unsigned_abs() as u16
                };
                let new_top = this_top.min(top_y);
                let new_height = if new_min >= 0 {
                    new_top - new_min as u16
                } else {
                    new_top + new_min.unsigned_abs() as u16
                };

                Self {
                    min_y: new_min,
                    height: new_height,
                    size_horizontal: self.size_horizontal,
                    size_vertical: self.size_vertical,
                }
            }
        }

        pub struct BlockMaterialRule {
            pub result_state: BlockBlueprint,
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
            Unsupported,
        }


        pub struct BiomeMaterialCondition {
            pub biome_is: &'static [&'static Biome],
        }

        pub struct NoiseThresholdMaterialCondition {
            pub noise: &'static str,
            pub min_threshold: f64,
            pub max_threshold: f64,
        }

        pub struct VerticalGradientMaterialCondition {
            pub random_name: &'static str,
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

        impl GenerationSettings {
            #const_defs

            pub fn from_dimension(dimension: &Dimension) -> &'static Self {
                if dimension == &Dimension::OVERWORLD {
                    &Self::OVERWORLD
                } else if dimension == &Dimension::THE_NETHER {
                    &Self::NETHER
                } else {
                    &Self::END
                }
            }
        }
    )
}
