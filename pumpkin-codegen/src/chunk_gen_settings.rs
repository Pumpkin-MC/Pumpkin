use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

/// Deserialized block reference used in chunk generation settings (e.g., default block or fluid).
#[derive(Deserialize)]
pub struct BlockStateCodecStruct {
    /// Block registry name including the `minecraft:` namespace prefix.
    #[serde(rename = "Name")]
    pub name: String,
    /// Optional block state properties (e.g., `{"facing": "north"}`).
    #[serde(rename = "Properties")]
    pub properties: Option<BTreeMap<String, String>>,
}

/// Deserialized chunk generation settings for a dimension, sourced from `chunk_gen_settings.json`.
#[derive(Deserialize)]
pub struct GenerationSettingsStruct {
    /// Whether aquifer (underground water pocket) generation is enabled.
    #[serde(default)]
    pub aquifers_enabled: bool,
    /// Whether ore-vein generation is enabled.
    #[serde(default)]
    pub ore_veins_enabled: bool,
    /// Whether to use the legacy random number source for this dimension.
    #[serde(default)]
    pub legacy_random_source: bool,
    /// Y-level treated as sea level for this dimension.
    pub sea_level: i32,
    /// Default fluid block (usually water or lava) placed by the aquifer generator.
    pub default_fluid: BlockStateCodecStruct,
    /// Default solid block used to fill the terrain.
    pub default_block: BlockStateCodecStruct,
    /// Noise shape parameters controlling vertical and horizontal cell sizes.
    #[serde(rename = "noise")]
    pub shape: GenerationShapeConfigStruct,
    /// Hierarchical surface material rule determining which block is placed at each surface point.
    pub surface_rule: MaterialRuleStruct,
}

/// Deserialized noise-shape configuration controlling terrain cell dimensions.
#[derive(Deserialize)]
pub struct GenerationShapeConfigStruct {
    /// Minimum Y level for terrain generation.
    pub min_y: i8,
    /// Total vertical span of the generation region in blocks.
    pub height: u16,
    /// Log₂ of the horizontal cell block count (cell width = `1 << size_horizontal`).
    pub size_horizontal: u8,
    /// Log₂ of the vertical cell block count (cell height = `1 << size_vertical`).
    pub size_vertical: u8,
}

/// Deserialized surface material rule that determines which block to place at a given surface point.
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum MaterialRuleStruct {
    /// Place a specific block state unconditionally.
    #[serde(rename = "minecraft:block")]
    Block {
        /// The block state to place.
        result_state: BlockStateCodecStruct,
    },
    /// Evaluate each rule in order, stopping at the first match.
    #[serde(rename = "minecraft:sequence")]
    Sequence {
        /// The ordered list of child rules.
        sequence: Vec<Self>,
    },
    /// Apply `then_run` only when `if_true` evaluates to true.
    #[serde(rename = "minecraft:condition")]
    Condition {
        /// The condition that must be satisfied.
        if_true: MaterialConditionStruct,
        /// The rule to apply when the condition is met.
        then_run: Box<Self>,
    },
    /// Special Badlands terrain coloring rule.
    #[serde(rename = "minecraft:bandlands")]
    Badlands,
}

/// Deserialized surface material condition that gates a material rule.
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum MaterialConditionStruct {
    /// True when the current position is in one of the listed biomes.
    #[serde(rename = "minecraft:biome")]
    Biome {
        /// List of biome resource locations to match against.
        biome_is: Vec<String>,
    },
    /// True when a named noise value is within the given range.
    #[serde(rename = "minecraft:noise_threshold")]
    NoiseThreshold {
        /// Resource location of the noise parameter.
        noise: String,
        /// Minimum threshold (inclusive).
        min_threshold: f64,
        /// Maximum threshold (inclusive).
        max_threshold: f64,
    },
    /// True below a Y offset and false above another, with a linear gradient in between.
    #[serde(rename = "minecraft:vertical_gradient")]
    VerticalGradient {
        /// Name of the random source used for this gradient.
        random_name: String,
        /// Y offset below which the condition is always true.
        true_at_and_below: YOffsetStruct,
        /// Y offset above which the condition is always false.
        false_at_and_above: YOffsetStruct,
    },
    /// True when the position is above a given Y anchor.
    #[serde(rename = "minecraft:y_above")]
    YAbove {
        /// The Y offset anchor to compare against.
        anchor: YOffsetStruct,
        /// Multiplier applied to surface depth when computing the threshold.
        surface_depth_multiplier: i32,
        /// Whether to add stone depth to the comparison value.
        add_stone_depth: bool,
    },
    /// True when the position is above a water surface within a certain offset.
    #[serde(rename = "minecraft:water")]
    Water {
        /// Y offset relative to the water surface.
        offset: i32,
        /// Multiplier applied to surface depth.
        surface_depth_multiplier: i32,
        /// Whether to add stone depth to the comparison value.
        add_stone_depth: bool,
    },
    /// True when the biome temperature is cold (below freezing).
    #[serde(rename = "minecraft:temperature")]
    Temperature,
    /// True when the terrain is steep (high slope).
    #[serde(rename = "minecraft:steep")]
    Steep,
    /// Inverts the inner condition.
    #[serde(rename = "minecraft:not")]
    Not {
        /// The condition to invert.
        invert: Box<Self>,
    },
    /// True when there is a hole (cave opening) at the position.
    #[serde(rename = "minecraft:hole")]
    Hole,
    /// True when the position is above the preliminary surface estimate.
    #[serde(rename = "minecraft:above_preliminary_surface")]
    AbovePreliminarySurface,
    /// True based on the depth of stone/ceiling relative to the surface.
    #[serde(rename = "minecraft:stone_depth")]
    StoneDepth {
        /// Y offset added to the depth value.
        offset: i32,
        /// Whether to include surface depth in the calculation.
        add_surface_depth: bool,
        /// Additional secondary depth range.
        secondary_depth_range: i32,
        /// Surface type to measure from: `"ceiling"` or `"floor"`.
        surface_type: String,
    },
}

/// Deserialized Y offset that can be expressed relative to different reference points.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum YOffsetStruct {
    /// An absolute Y coordinate.
    Absolute {
        /// The absolute Y level.
        absolute: i16,
    },
    /// A Y level measured upward from the dimension's minimum Y.
    AboveBottom {
        /// Number of blocks above the bottom of the dimension.
        above_bottom: i8,
    },
    /// A Y level measured downward from the dimension's maximum Y.
    BelowTop {
        /// Number of blocks below the top of the dimension.
        below_top: i8,
    },
}

// --- ToTokens Implementations ---

impl ToTokens for BlockStateCodecStruct {
    /// Emits a `BlockBlueprint` literal, stripping the `minecraft:` namespace prefix from the block name.
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
    /// Emits a `GenerationSettings` struct literal with all fields populated from the deserialized data.
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
    /// Emits a `GenerationShapeConfig` struct literal with all noise-shape dimensions.
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
    /// Emits a `YOffset` enum variant literal corresponding to the deserialized offset kind.
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
    /// Emits a `MaterialCondition` enum variant literal for each surface condition kind.
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
    /// Emits a `MaterialRule` enum variant literal for each surface placement rule kind.
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

/// Reads `chunk_gen_settings.json` and emits the complete chunk generation settings `TokenStream`.
pub fn build() -> TokenStream {
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
        use crate::chunk::DoublePerlinNoiseParameters;

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
            #[inline]
            #[must_use]
            pub const fn vertical_cell_block_count(&self) -> u8 { self.size_vertical << 2 }

            #[inline]
            #[must_use]
            pub const fn horizontal_cell_block_count(&self) -> u8 { self.size_horizontal << 2 }

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
