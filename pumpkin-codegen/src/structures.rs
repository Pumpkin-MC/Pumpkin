use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

/// Deserialized structure set containing placement rules and weighted structure entries.
#[derive(Deserialize)]
pub struct StructureSetStruct {
    /// Placement algorithm and parameters for distributing this structure set.
    pub placement: StructurePlacementStruct,
    /// Weighted list of structures that belong to this set.
    pub structures: Vec<WeightedEntryStruct>,
}

/// Deserialized weighted structure entry within a structure set.
#[derive(Deserialize, Clone)]
pub struct WeightedEntryStruct {
    /// Registry key of the structure (e.g., `"minecraft:village_plains"`).
    pub structure: String,
    /// Relative weight controlling how often this structure is chosen.
    pub weight: u32,
}

/// Deserialized placement configuration for a structure set.
#[derive(Deserialize)]
pub struct StructurePlacementStruct {
    /// Optional frequency-reduction method name applied before placement.
    pub frequency_reduction_method: Option<String>,
    /// Optional probability (0–1) that a candidate chunk actually spawns the structure.
    pub frequency: Option<f32>,
    /// Per-structure salt mixed into the placement RNG seed.
    pub salt: u32,
    /// Optional exclusion zone to prevent this structure from generating near others.
    pub exclusion_zone: Option<ExclusionZoneStruct>,
    /// The specific placement algorithm and its parameters.
    #[serde(flatten)]
    pub r#type: StructurePlacementTypeStruct,
}

/// Deserialized exclusion zone configuration.
#[derive(Deserialize)]
pub struct ExclusionZoneStruct {
    pub other_set: String,
    pub chunk_count: i32,
}

/// Deserialized placement algorithm for a structure set.
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum StructurePlacementTypeStruct {
    /// Places structures at pseudo-random positions within regularly spaced grid regions.
    #[serde(rename = "minecraft:random_spread")]
    RandomSpread {
        /// Region size (spacing between potential structure positions).
        spacing: i32,
        /// Minimum separation between two structures in the same region.
        separation: i32,
        /// Optional distribution type (`"linear"` or `"triangular"`).
        spread_type: Option<String>,
    },
    /// Places structures in concentric rings around the world origin.
    #[serde(rename = "minecraft:concentric_rings")]
    ConcentricRings {
        /// Angular spread between ring placements.
        spread: i32,
        /// Distance increment between successive rings.
        distance: i32,
        /// Total number of structures placed across all rings.
        count: i32,
        /// Biome tag that ring placements must be located within.
        preferred_biomes: String,
    },
}

/// Deserialized structure entry specifying its valid biomes and generation step.
#[derive(Deserialize, Clone)]
pub struct StructureStruct {
    /// Biome tag that this structure can generate in.
    pub biomes: String,
    /// Generation step during which this structure is placed.
    pub step: String,
    /// Optional jigsaw start pool.
    pub start_pool: Option<String>,
    /// Optional jigsaw size (depth).
    pub size: Option<i32>,
    /// Optional terrain adaptation (bearding).
    pub terrain_adaptation: Option<String>,
    /// Optional jigsaw start height.
    pub start_height: Option<serde_json::Value>,
    /// Optional heightmap to project the start to.
    pub project_start_to_heightmap: Option<String>,
    /// Optional max distance from center.
    pub max_distance_from_center: Option<i32>,
    /// Optional liquid settings.
    pub liquid_settings: Option<String>,
    /// Optional dimension padding.
    pub dimension_padding: Option<i32>,
    /// Critical for villages to appropriately truncate long village streets.
    pub use_expansion_hack: Option<bool>,
    /// Defines the generation behavior (e.g. "minecraft:jigsaw").
    #[serde(rename = "type")]
    pub structure_type: String,
}

impl ToTokens for StructureSetStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let placement = &self.placement;
        let structures = &self.structures;

        tokens.extend(quote!(
            StructureSet {
                placement: #placement,
                structures: &[#(#structures),*],
            }
        ));
    }
}

impl ToTokens for WeightedEntryStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let structure = structure_key_to_token(&self.structure);
        let weight = self.weight;

        tokens.extend(quote!(
            WeightedEntry {
                structure: #structure,
                weight: #weight,
            }
        ));
    }
}

impl ToTokens for StructurePlacementStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let frequency_reduction = if let Some(method) = &self.frequency_reduction_method {
            let method_token = frequency_reduction_to_token(method);
            quote!(Some(#method_token))
        } else {
            quote!(None)
        };

        let frequency = if let Some(f) = self.frequency {
            quote!(Some(#f))
        } else {
            quote!(None)
        };

        let exclusion_zone = if let Some(ez) = &self.exclusion_zone {
            let other_set = &ez.other_set;
            let chunk_count = ez.chunk_count;
            quote!(Some(ExclusionZone {
                other_set: #other_set,
                chunk_count: #chunk_count,
            }))
        } else {
            quote!(None)
        };

        let salt = self.salt;
        let placement_type = &self.r#type;

        tokens.extend(quote!(
            StructurePlacement {
                frequency_reduction_method: #frequency_reduction,
                frequency: #frequency,
                salt: #salt,
                exclusion_zone: #exclusion_zone,
                placement_type: #placement_type,
            }
        ));
    }
}

impl ToTokens for StructurePlacementTypeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::RandomSpread {
                spacing,
                separation,
                spread_type,
            } => {
                let spread_type_token = if let Some(st) = spread_type {
                    let st_token = spread_type_to_token(st);
                    quote!(Some(#st_token))
                } else {
                    quote!(None)
                };

                tokens.extend(quote!(
                    StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                        spacing: #spacing,
                        separation: #separation,
                        spread_type: #spread_type_token,
                    })
                ));
            }
            Self::ConcentricRings {
                spread,
                distance,
                count,
                preferred_biomes,
            } => {
                tokens.extend(quote!(
                    StructurePlacementType::ConcentricRings(ConcentricRingsStructurePlacement {
                        spread: #spread,
                        distance: #distance,
                        count: #count,
                        preferred_biomes: #preferred_biomes,
                    })
                ));
            }
        }
    }
}

impl ToTokens for StructureStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let biomes = &self.biomes;
        let step = generation_step_to_token(&self.step);
        let start_pool = if let Some(pool) = &self.start_pool {
            quote!(Some(#pool))
        } else {
            quote!(None)
        };
        let size = if let Some(s) = self.size {
            quote!(Some(#s))
        } else {
            quote!(None)
        };
        let terrain_adaptation = if let Some(ta) = &self.terrain_adaptation {
            let ta_token = terrain_adaptation_to_token(ta);
            quote!(#ta_token)
        } else {
            quote!(TerrainAdaptation::None)
        };
        let project_start_to_heightmap = if let Some(ps) = &self.project_start_to_heightmap {
            quote!(Some(#ps))
        } else {
            quote!(None)
        };
        let max_distance_from_center = if let Some(m) = self.max_distance_from_center {
            quote!(Some(#m))
        } else {
            quote!(None)
        };
        let liquid_settings = if let Some(ls) = &self.liquid_settings {
            quote!(Some(#ls))
        } else {
            quote!(None)
        };
        let dimension_padding = if let Some(dp) = self.dimension_padding {
            quote!(Some(#dp))
        } else {
            quote!(None)
        };
        let use_expansion_hack = if let Some(hack) = self.use_expansion_hack {
            quote!(Some(#hack))
        } else {
            quote!(None)
        };

        let structure_type = structure_type_to_token(&self.structure_type);

        let start_height = if let Some(sh) = &self.start_height {
            if let Some(abs) = sh.get("absolute") {
                let abs_val = abs.as_i64().unwrap() as i16;
                quote!(Some(#abs_val))
            } else if let Some(min_inc) = sh.get("min_inclusive") {
                if let Some(abs) = min_inc.get("absolute") {
                    let abs_val = abs.as_i64().unwrap() as i16;
                    quote!(Some(#abs_val))
                } else {
                    quote!(None)
                }
            } else {
                quote!(None)
            }
        } else {
            quote!(None)
        };

        tokens.extend(quote!(
            Structure {
                biomes: #biomes,
                step: #step,
                start_pool: #start_pool,
                size: #size,
                terrain_adaptation: #terrain_adaptation,
                start_height: #start_height,
                project_start_to_heightmap: #project_start_to_heightmap,
                max_distance_from_center: #max_distance_from_center,
                liquid_settings: #liquid_settings,
                dimension_padding: #dimension_padding,
                use_expansion_hack: #use_expansion_hack,
                structure_type: #structure_type,
            }
        ));
    }
}

// Helper functions

fn structure_type_to_token(st: &str) -> TokenStream {
    match st {
        "minecraft:jigsaw" => quote!(StructureType::Jigsaw),
        "minecraft:buried_treasure" => quote!(StructureType::BuriedTreasure),
        "minecraft:desert_pyramid" => quote!(StructureType::DesertPyramid),
        "minecraft:end_city" => quote!(StructureType::EndCity),
        "minecraft:fortress" => quote!(StructureType::Fortress),
        "minecraft:igloo" => quote!(StructureType::Igloo),
        "minecraft:jungle_temple" => quote!(StructureType::JungleTemple),
        "minecraft:woodland_mansion" => quote!(StructureType::WoodlandMansion),
        "minecraft:mineshaft" => quote!(StructureType::Mineshaft),
        "minecraft:ocean_monument" => quote!(StructureType::OceanMonument),
        "minecraft:nether_fossil" => quote!(StructureType::NetherFossil),
        "minecraft:ocean_ruin" => quote!(StructureType::OceanRuin),
        "minecraft:ruined_portal" => quote!(StructureType::RuinedPortal),
        "minecraft:shipwreck" => quote!(StructureType::Shipwreck),
        "minecraft:stronghold" => quote!(StructureType::Stronghold),
        "minecraft:swamp_hut" => quote!(StructureType::SwampHut),
        _ => quote!(StructureType::Unknown),
    }
}

fn terrain_adaptation_to_token(ta: &str) -> TokenStream {
    match ta {
        "beard_thin" => quote!(TerrainAdaptation::BeardThin),
        "beard_box" => quote!(TerrainAdaptation::BeardBox),
        "bury" => quote!(TerrainAdaptation::Bury),
        "encapsulate" => quote!(TerrainAdaptation::Encapsulate),
        _ => quote!(TerrainAdaptation::None),
    }
}

fn structure_key_to_token(key: &str) -> TokenStream {
    let stripped = key.strip_prefix("minecraft:").unwrap_or(key);

    match stripped {
        "pillager_outpost" => quote!(StructureKeys::PillagerOutpost),
        "mineshaft" => quote!(StructureKeys::Mineshaft),
        "mineshaft_mesa" => quote!(StructureKeys::MineshaftMesa),
        "mansion" => quote!(StructureKeys::Mansion),
        "jungle_pyramid" => quote!(StructureKeys::JunglePyramid),
        "desert_pyramid" => quote!(StructureKeys::DesertPyramid),
        "igloo" => quote!(StructureKeys::Igloo),
        "shipwreck" => quote!(StructureKeys::Shipwreck),
        "shipwreck_beached" => quote!(StructureKeys::ShipwreckBeached),
        "swamp_hut" => quote!(StructureKeys::SwampHut),
        "stronghold" => quote!(StructureKeys::Stronghold),
        "monument" => quote!(StructureKeys::Monument),
        "ocean_ruin_cold" => quote!(StructureKeys::OceanRuinCold),
        "ocean_ruin_warm" => quote!(StructureKeys::OceanRuinWarm),
        "fortress" => quote!(StructureKeys::Fortress),
        "nether_fossil" => quote!(StructureKeys::NetherFossil),
        "end_city" => quote!(StructureKeys::EndCity),
        "buried_treasure" => quote!(StructureKeys::BuriedTreasure),
        "bastion_remnant" => quote!(StructureKeys::BastionRemnant),
        "village_plains" => quote!(StructureKeys::VillagePlains),
        "village_desert" => quote!(StructureKeys::VillageDesert),
        "village_savanna" => quote!(StructureKeys::VillageSavanna),
        "village_snowy" => quote!(StructureKeys::VillageSnowy),
        "village_taiga" => quote!(StructureKeys::VillageTaiga),
        "ruined_portal" => quote!(StructureKeys::RuinedPortal),
        "ruined_portal_desert" => quote!(StructureKeys::RuinedPortalDesert),
        "ruined_portal_jungle" => quote!(StructureKeys::RuinedPortalJungle),
        "ruined_portal_swamp" => quote!(StructureKeys::RuinedPortalSwamp),
        "ruined_portal_mountain" => quote!(StructureKeys::RuinedPortalMountain),
        "ruined_portal_ocean" => quote!(StructureKeys::RuinedPortalOcean),
        "ruined_portal_nether" => quote!(StructureKeys::RuinedPortalNether),
        "ancient_city" => quote!(StructureKeys::AncientCity),
        "trail_ruins" => quote!(StructureKeys::TrailRuins),
        "trial_chambers" => quote!(StructureKeys::TrialChambers),
        _ => panic!("Unknown structure key: {stripped}"),
    }
}

fn frequency_reduction_to_token(method: &str) -> TokenStream {
    match method {
        "default" => quote!(FrequencyReductionMethod::Default),
        "legacy_type_1" => quote!(FrequencyReductionMethod::LegacyType1),
        "legacy_type_2" => quote!(FrequencyReductionMethod::LegacyType2),
        "legacy_type_3" => quote!(FrequencyReductionMethod::LegacyType3),
        _ => quote!(FrequencyReductionMethod::Default),
    }
}

fn spread_type_to_token(spread: &str) -> TokenStream {
    match spread {
        "linear" => quote!(SpreadType::Linear),
        "triangular" => quote!(SpreadType::Triangular),
        _ => quote!(SpreadType::Linear),
    }
}

fn generation_step_to_token(step: &str) -> TokenStream {
    match step {
        "raw_generation" => quote!(GenerationStep::RawGeneration),
        "lakes" => quote!(GenerationStep::Lakes),
        "local_modifications" => quote!(GenerationStep::LocalModifications),
        "underground_structures" => quote!(GenerationStep::UndergroundStructures),
        "surface_structures" => quote!(GenerationStep::SurfaceStructures),
        "strongholds" => quote!(GenerationStep::Strongholds),
        "underground_ores" => quote!(GenerationStep::UndergroundOres),
        "underground_decoration" => quote!(GenerationStep::UndergroundDecoration),
        "fluid_springs" => quote!(GenerationStep::FluidSprings),
        "vegetal_decoration" => quote!(GenerationStep::VegetalDecoration),
        "top_layer_modification" => quote!(GenerationStep::TopLayerModification),
        _ => panic!("Unknown generation step: {step}"),
    }
}

/// Reads `structures.json` and `structure_set.json` and emits the complete structures `TokenStream`.
pub fn build() -> TokenStream {
    let structures_json: BTreeMap<String, StructureStruct> =
        serde_json::from_str(&fs::read_to_string("../assets/structures.json").unwrap())
            .expect("Failed to parse structures.json");

    let structure_sets_json: BTreeMap<String, StructureSetStruct> =
        serde_json::from_str(&fs::read_to_string("../assets/structure_set.json").unwrap())
            .expect("Failed to parse structure_set.json");

    let mut structure_const_defs = TokenStream::new();
    let mut structure_lookup_arms = TokenStream::new();

    for (name, structure) in &structures_json {
        let stripped_name = name.strip_prefix("minecraft:").unwrap_or(name);
        let upper_name = stripped_name.to_uppercase();
        let const_name = format_ident!("{}", upper_name);
        let key_variant = structure_key_to_token(name);

        structure_const_defs.extend(quote!(
            pub const #const_name: Self = #structure;
        ));

        structure_lookup_arms.extend(quote!(
            #key_variant => &Self::#const_name,
        ));
    }

    let mut structure_set_const_defs = TokenStream::new();
    let mut structure_set_lookup_arms = TokenStream::new();
    let mut all_structure_set_idents = Vec::new();

    for (name, structure_set) in &structure_sets_json {
        let stripped_name = name.strip_prefix("minecraft:").unwrap_or(name);
        let upper_name = stripped_name.to_uppercase().replace('/', "_");
        let const_name = format_ident!("{}", upper_name);

        structure_set_const_defs.extend(quote!(
            pub const #const_name: Self = #structure_set;
        ));

        structure_set_lookup_arms.extend(quote!(
            #stripped_name => Some(&Self::#const_name),
        ));

        all_structure_set_idents.push(const_name);
    }

    quote!(
        use pumpkin_util::math::floor_div;
        use pumpkin_util::random::{
            RandomGenerator, RandomImpl, get_carver_seed, get_region_seed,
            legacy_rand::LegacyRand, xoroshiro128::Xoroshiro,
        };

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum StructureKeys {
            PillagerOutpost,
            Mineshaft,
            MineshaftMesa,
            Mansion,
            JunglePyramid,
            DesertPyramid,
            Igloo,
            Shipwreck,
            ShipwreckBeached,
            SwampHut,
            Stronghold,
            Monument,
            OceanRuinCold,
            OceanRuinWarm,
            Fortress,
            NetherFossil,
            EndCity,
            BuriedTreasure,
            BastionRemnant,
            VillagePlains,
            VillageDesert,
            VillageSavanna,
            VillageSnowy,
            VillageTaiga,
            RuinedPortal,
            RuinedPortalDesert,
            RuinedPortalJungle,
            RuinedPortalSwamp,
            RuinedPortalMountain,
            RuinedPortalOcean,
            RuinedPortalNether,
            AncientCity,
            TrailRuins,
            TrialChambers,
        }

        pub struct StructureSet {
            pub placement: StructurePlacement,
            pub structures: &'static [WeightedEntry],
        }

        #[derive(Clone)]
        pub struct WeightedEntry {
            pub structure: StructureKeys,
            pub weight: u32,
        }

        pub struct ExclusionZone {
            pub other_set: &'static str,
            pub chunk_count: i32,
        }

        pub struct StructurePlacement {
            pub frequency_reduction_method: Option<FrequencyReductionMethod>,
            pub frequency: Option<f32>,
            pub salt: u32,
            pub exclusion_zone: Option<ExclusionZone>,
            pub placement_type: StructurePlacementType,
        }

        #[derive(Clone, Copy)]
        pub enum FrequencyReductionMethod {
            Default,
            LegacyType1,
            LegacyType2,
            LegacyType3,
        }

        pub enum StructurePlacementType {
            RandomSpread(RandomSpreadStructurePlacement),
            ConcentricRings(ConcentricRingsStructurePlacement),
        }

        pub struct RandomSpreadStructurePlacement {
            pub spacing: i32,
            pub separation: i32,
            pub spread_type: Option<SpreadType>,
        }

        pub struct ConcentricRingsStructurePlacement {
            pub spread: i32,
            pub distance: i32,
            pub count: i32,
            pub preferred_biomes: &'static str,
        }

        #[derive(Clone, Copy)]
        pub enum SpreadType {
            Linear,
            Triangular,
        }

        impl SpreadType {
            pub fn get(&self, random: &mut RandomGenerator, bound: i32) -> i32 {
                match self {
                    Self::Linear => random.next_bounded_i32(bound),
                    Self::Triangular => i32::midpoint(
                        random.next_bounded_i32(bound),
                        random.next_bounded_i32(bound),
                    ),
                }
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum TerrainAdaptation {
            None,
            BeardThin,
            BeardBox,
            Bury,
            Encapsulate,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum StructureType {
            Jigsaw,
            BuriedTreasure,
            DesertPyramid,
            EndCity,
            Fortress,
            Igloo,
            JungleTemple,
            WoodlandMansion,
            Mineshaft,
            OceanMonument,
            NetherFossil,
            OceanRuin,
            RuinedPortal,
            Shipwreck,
            Stronghold,
            SwampHut,
            Unknown,
        }

        pub struct Structure {
            pub biomes: &'static str,
            pub step: GenerationStep,
            pub start_pool: Option<&'static str>,
            pub size: Option<i32>,
            pub terrain_adaptation: TerrainAdaptation,
            pub start_height: Option<i16>,
            pub project_start_to_heightmap: Option<&'static str>,
            pub max_distance_from_center: Option<i32>,
            pub liquid_settings: Option<&'static str>,
            pub dimension_padding: Option<i32>,
            pub use_expansion_hack: Option<bool>,
            pub structure_type: StructureType,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum GenerationStep {
            RawGeneration,
            Lakes,
            LocalModifications,
            UndergroundStructures,
            SurfaceStructures,
            Strongholds,
            UndergroundOres,
            UndergroundDecoration,
            FluidSprings,
            VegetalDecoration,
            TopLayerModification,
        }

        impl GenerationStep {
            #[must_use]
            pub const fn ordinal(&self) -> usize {
                *self as usize
            }
        }

        pub struct StructurePlacementCalculator {
            pub seed: i64,
        }

        impl StructurePlacementCalculator {
            #[must_use]
            pub const fn new(seed: i64) -> Self {
                Self { seed }
            }
        }

        impl Structure {
            #structure_const_defs

            #[must_use]
            pub const fn get(key: &StructureKeys) -> &'static Self {
                match *key {
                    #structure_lookup_arms
                }
            }
        }

        impl StructureSet {
            #structure_set_const_defs

            pub const ALL: &'static [StructureSet] = &[
                #(Self::#all_structure_set_idents),*
            ];

            #[must_use]
            pub fn get(name: &str) -> Option<&'static Self> {
                match name {
                    #structure_set_lookup_arms
                    _ => None,
                }
            }
        }
    )
}
