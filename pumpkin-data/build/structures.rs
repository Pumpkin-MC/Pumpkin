use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

#[derive(Deserialize)]
pub struct StructureSetStruct {
    pub placement: StructurePlacementStruct,
    pub structures: Vec<WeightedEntryStruct>,
}

#[derive(Deserialize, Clone)]
pub struct WeightedEntryStruct {
    pub structure: String,
    pub weight: u32,
}

#[derive(Deserialize)]
pub struct StructurePlacementStruct {
    frequency_reduction_method: Option<String>,
    frequency: Option<f32>,
    salt: u32,
    #[serde(flatten)]
    r#type: StructurePlacementTypeStruct,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum StructurePlacementTypeStruct {
    #[serde(rename = "minecraft:random_spread")]
    RandomSpread {
        spacing: i32,
        separation: i32,
        spread_type: Option<String>,
    },
    #[serde(rename = "minecraft:concentric_rings")]
    ConcentricRings {
        spread: i32,
        distance: i32,
        count: i32,
        preferred_biomes: String,
    },
}

#[derive(Deserialize, Clone)]
pub struct StructureStruct {
    pub biomes: String,
    pub step: String,
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

        let salt = self.salt;
        let placement_type = &self.r#type;

        tokens.extend(quote!(
            StructurePlacement {
                frequency_reduction_method: #frequency_reduction,
                frequency: #frequency,
                salt: #salt,
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

        tokens.extend(quote!(
            Structure {
                biomes: #biomes,
                step: #step,
            }
        ));
    }
}

// Helper functions

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

pub fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/structures.json");
    println!("cargo:rerun-if-changed=../assets/structure_set.json");

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
            pub const #const_name: Structure = #structure;
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
            pub const #const_name: StructureSet = #structure_set;
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

        pub struct StructurePlacement {
            pub frequency_reduction_method: Option<FrequencyReductionMethod>,
            pub frequency: Option<f32>,
            pub salt: u32,
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

        pub struct Structure {
            pub biomes: &'static str,
            pub step: GenerationStep,
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
