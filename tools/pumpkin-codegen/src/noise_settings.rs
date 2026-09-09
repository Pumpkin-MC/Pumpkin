use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

/// Deserialized block reference used in noise settings (e.g., default block or fluid).
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

/// Raw deserialized noise settings file from `worldgen/noise_settings`.
#[derive(Deserialize)]
pub struct NoiseSettingsFileStruct {
    #[serde(default)]
    pub aquifers: Option<serde_json::Value>,
    #[serde(default)]
    pub aquifers_enabled: Option<bool>,
    #[serde(default)]
    pub ore_veins_enabled: Option<bool>,
    #[serde(default)]
    pub legacy_random_source: bool,
    pub sea_level: i32,
    pub default_fluid: BlockStateCodecStruct,
    pub default_block: BlockStateCodecStruct,
    #[serde(rename = "noise")]
    pub shape: GenerationShapeConfigStruct,
    #[serde(default)]
    pub material_rule: Option<serde_json::Value>,
    #[serde(default)]
    pub surface_rule: Option<serde_json::Value>,
    #[serde(default)]
    pub spawn_target: Vec<RawSpawnTarget>,
}

/// Raw spawn target deserialization supporting both noise map format and structured format.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum RawSpawnTarget {
    Structured(ParameterPointStruct),
    NoiseMap(BTreeMap<String, ParameterStruct>),
}

impl RawSpawnTarget {
    pub fn to_parameter_point(&self) -> ParameterPointStruct {
        match self {
            Self::Structured(p) => p.clone(),
            Self::NoiseMap(map) => {
                let mut temp = ParameterStruct::Span([-1.0, 1.0]);
                let mut hum = ParameterStruct::Span([-1.0, 1.0]);
                let mut cont = ParameterStruct::Span([-1.0, 1.0]);
                let mut erosion = ParameterStruct::Span([-1.0, 1.0]);
                let mut depth = ParameterStruct::Span([0.0, 0.0]);
                let mut weirdness = ParameterStruct::Span([-1.0, 1.0]);
                let mut offset = ParameterStruct::Point(0.0);

                for (key, val) in map {
                    if key.contains("temperature") {
                        temp = val.clone();
                    } else if key.contains("vegetation") || key.contains("humidity") {
                        hum = val.clone();
                    } else if key.contains("continents") || key.contains("continentalness") {
                        cont = val.clone();
                    } else if key.contains("erosion") {
                        erosion = val.clone();
                    } else if key.contains("depth") {
                        depth = val.clone();
                    } else if key.contains("ridges") || key.contains("weirdness") {
                        weirdness = val.clone();
                    } else if key.contains("offset") {
                        offset = val.clone();
                    }
                }

                ParameterPointStruct {
                    temperature: temp,
                    humidity: hum,
                    continentalness: cont,
                    erosion,
                    depth,
                    weirdness,
                    offset,
                }
            }
        }
    }
}

/// Deserialized parameter interval or point.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum ParameterStruct {
    Point(f32),
    Span([f32; 2]),
}

impl Default for ParameterStruct {
    fn default() -> Self {
        Self::Point(0.0)
    }
}

impl ToTokens for ParameterStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Point(val) => {
                let quantized = (*val * 10000.0) as i64;
                tokens.extend(quote!(
                    crate::biome::Parameter::new(#quantized, #quantized)
                ));
            }
            Self::Span([min, max]) => {
                let min_q = (*min * 10000.0) as i64;
                let max_q = (*max * 10000.0) as i64;
                tokens.extend(quote!(
                    crate::biome::Parameter::new(#min_q, #max_q)
                ));
            }
        }
    }
}

/// Deserialized parameter point for spawn target configuration.
#[derive(Deserialize, Clone)]
pub struct ParameterPointStruct {
    pub temperature: ParameterStruct,
    pub humidity: ParameterStruct,
    pub continentalness: ParameterStruct,
    pub erosion: ParameterStruct,
    pub depth: ParameterStruct,
    pub weirdness: ParameterStruct,
    #[serde(default)]
    pub offset: ParameterStruct,
}

impl ToTokens for ParameterPointStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let temp = &self.temperature;
        let hum = &self.humidity;
        let cont = &self.continentalness;
        let erosion = &self.erosion;
        let depth = &self.depth;
        let weirdness = &self.weirdness;
        let offset = match self.offset {
            ParameterStruct::Point(val) => (val * 10000.0) as i64,
            ParameterStruct::Span([min, _]) => (min * 10000.0) as i64,
        };
        tokens.extend(quote!(
            crate::biome::ParameterPoint {
                temperature: #temp,
                humidity: #hum,
                continentalness: #cont,
                erosion: #erosion,
                depth: #depth,
                weirdness: #weirdness,
                offset: #offset,
            }
        ));
    }
}

/// Deserialized noise-shape configuration controlling terrain cell dimensions.
#[derive(Deserialize, Clone)]
pub struct GenerationShapeConfigStruct {
    pub min_y: i8,
    pub height: u16,
    #[serde(default)]
    pub size_horizontal: u8,
    #[serde(default)]
    pub size_vertical: u8,
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

/// Deserialized noise settings struct.
#[derive(Clone)]
pub struct NoiseSettingsStruct {
    pub aquifers_enabled: bool,
    pub ore_veins_enabled: bool,
    pub legacy_random_source: bool,
    pub sea_level: i32,
    pub default_fluid: BlockStateCodecStruct,
    pub default_block: BlockStateCodecStruct,
    pub shape: GenerationShapeConfigStruct,
    pub spawn_target: Vec<ParameterPointStruct>,
}

impl ToTokens for NoiseSettingsStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let aquifers = self.aquifers_enabled;
        let ores = self.ore_veins_enabled;
        let legacy = self.legacy_random_source;
        let sea_level = self.sea_level;
        let fluid = &self.default_fluid;
        let block = &self.default_block;
        let shape = &self.shape;
        let spawn_target = &self.spawn_target;

        tokens.extend(quote!(
            NoiseSettings {
                aquifers_enabled: #aquifers,
                ore_veins_enabled: #ores,
                legacy_random_source: #legacy,
                sea_level: #sea_level,
                default_fluid: #fluid,
                shape: #shape,
                default_block: #block,
                spawn_target: &[#(#spawn_target),*],
            }
        ));
    }
}

/// Reads noise_settings files from datapack and generates NoiseSettings constants.
pub fn build() -> TokenStream {
    let noise_settings_dir =
        std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/worldgen/noise_settings");

    let mut json: BTreeMap<String, NoiseSettingsStruct> = BTreeMap::new();
    let mut entries: Vec<_> = fs::read_dir(noise_settings_dir)
        .expect("Missing worldgen/noise_settings directory")
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
        if stem.starts_with('_') {
            continue;
        }

        let content = fs::read_to_string(&path).expect("Failed to read noise_settings file");
        let noise_settings: NoiseSettingsFileStruct =
            serde_json::from_str(&content).expect("Failed to parse noise_settings JSON");

        let aquifers_enabled =
            noise_settings.aquifers.is_some() || noise_settings.aquifers_enabled.unwrap_or(false);
        let ore_veins_enabled = noise_settings.ore_veins_enabled.unwrap_or(aquifers_enabled);

        let spawn_target: Vec<ParameterPointStruct> = noise_settings
            .spawn_target
            .iter()
            .map(RawSpawnTarget::to_parameter_point)
            .collect();

        let (def_h, def_v) = match stem.as_str() {
            "end" | "floating_islands" => (2, 1),
            _ => (1, 2),
        };
        let shape = GenerationShapeConfigStruct {
            min_y: noise_settings.shape.min_y,
            height: noise_settings.shape.height,
            size_horizontal: if noise_settings.shape.size_horizontal != 0 {
                noise_settings.shape.size_horizontal
            } else {
                def_h
            },
            size_vertical: if noise_settings.shape.size_vertical != 0 {
                noise_settings.shape.size_vertical
            } else {
                def_v
            },
        };

        let settings = NoiseSettingsStruct {
            aquifers_enabled,
            ore_veins_enabled,
            legacy_random_source: noise_settings.legacy_random_source,
            sea_level: noise_settings.sea_level,
            default_fluid: noise_settings.default_fluid,
            default_block: noise_settings.default_block,
            shape,
            spawn_target,
        };

        json.insert(stem, settings);
    }

    let mut const_defs = TokenStream::new();

    for (name, settings) in &json {
        let upper_name = name.to_uppercase();
        let const_name = format_ident!("{}", upper_name);

        const_defs.extend(quote!(
            pub const #const_name: NoiseSettings = #settings;
        ));
    }

    quote!(
        use crate::dimension::Dimension;
        use crate::BlockState;
        use crate::biome::ParameterPoint;

        pub struct NoiseSettings {
            pub aquifers_enabled: bool,
            pub ore_veins_enabled: bool,
            pub legacy_random_source: bool,
            pub sea_level: i32,
            pub default_fluid: &'static BlockState,
            pub shape: GenerationShapeConfig,
            pub default_block: &'static BlockState,
            pub spawn_target: &'static [ParameterPoint],
        }

        pub type GenerationSettings = NoiseSettings;

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

            #[must_use]
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

        impl NoiseSettings {
            #const_defs

            #[must_use]
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
