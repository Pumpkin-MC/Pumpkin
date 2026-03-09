use std::{collections::BTreeMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use syn::LitInt;

/// Raw deserialization shape for a single biome entry from `biome.json`.
#[derive(Deserialize)]
pub struct Biome {
    /// Whether this biome has precipitation (rain or snow).
    has_precipitation: bool,
    /// Base temperature of the biome affecting weather and foliage color.
    temperature: f32,
    /// Downfall value affecting how wet the biome is (influences snow/rain).
    downfall: f32,
    /// Optional modifier that changes how temperature is applied.
    temperature_modifier: Option<TemperatureModifier>,
    //carvers: Vec<String>,
    /// Nested lists of feature resource-location strings applied during world generation.
    features: Vec<Vec<String>>,
    /// Probability per chunk tick that a creature spawns, if not overridden per-biome.
    creature_spawn_probability: Option<f32>,
    /// Spawn group data for each entity category in this biome.
    spawners: SpawnGroups,
    /// Per-entity spawn cost budget entries, keyed by namespaced entity ID.
    spawn_costs: BTreeMap<String, SpawnCosts>,
    /// Numeric registry ID assigned to this biome.
    pub id: u8,
}

/// Spawn group data for all entity categories within a biome.
#[derive(Deserialize, PartialEq, Eq, Hash)]
struct SpawnGroups {
    /// Hostile mob spawners for this biome.
    monster: Vec<Spawner>,
    /// Ambient creature spawners (e.g. bats) for this biome.
    ambient: Vec<Spawner>,
    /// Axolotl spawners for this biome.
    axolotls: Vec<Spawner>,
    /// Passive creature spawners (e.g. cows, sheep) for this biome.
    creature: Vec<Spawner>,
    /// Miscellaneous entity spawners for this biome.
    misc: Vec<Spawner>,
    /// Underground water creature spawners (e.g. glow squid) for this biome.
    underground_water_creature: Vec<Spawner>,
    /// Water ambient spawners (e.g. fish) for this biome.
    water_ambient: Vec<Spawner>,
    /// Water creature spawners (e.g. dolphins) for this biome.
    water_creature: Vec<Spawner>,
}

/// A single entity spawner entry within a spawn group, as defined in `biome.json`.
#[expect(non_snake_case)]
#[derive(Deserialize, Hash, PartialEq, Eq)]
struct Spawner {
    /// Namespaced entity type ID (e.g. `"minecraft:zombie"`).
    r#type: String,
    /// Minimum number of entities in a spawn group.
    minCount: i32,
    /// Maximum number of entities in a spawn group.
    maxCount: i32,
}

impl Spawner {
    /// Converts this spawner entry into a `TokenStream` for use in generated code.
    pub fn to_tokens(&self) -> TokenStream {
        let r#type = &self.r#type;
        let min_count = &self.minCount;
        let max_count = &self.maxCount;
        quote! {
            Spawner {
                r#type: #r#type,
                min_count: #min_count,
                max_count: #max_count,
            }
        }
    }
}

/// Mob spawn cost data controlling how this entity affects biome spawn budgets.
#[derive(Deserialize, PartialEq)]
struct SpawnCosts {
    /// Maximum energy this entity type may consume from the spawn budget.
    energy_budget: f64,
    /// Energy cost charged to the budget each time this entity spawns.
    charge: f64,
}

impl SpawnCosts {
    /// Converts these spawn costs into a `TokenStream` for use in generated code.
    pub fn to_tokens(&self) -> TokenStream {
        let energy_budget = &self.energy_budget;
        let charge = &self.charge;
        quote! {
            SpawnCosts {
                energy_budget: #energy_budget,
                charge: #charge,
            }
        }
    }
}

/// Optional modifier that adjusts how biome temperature behaves (e.g. frozen biomes).
#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
enum TemperatureModifier {
    /// No modification; temperature is used as-is.
    None,
    /// Temperature is forced below freezing regardless of the base value.
    Frozen,
}

/// An inclusive range used as one dimension in the multi-noise biome source tree.
#[derive(Deserialize)]
struct ParameterRange {
    /// Lower bound of the noise range (inclusive).
    min: i64,
    /// Upper bound of the noise range (inclusive).
    max: i64,
}

impl ParameterRange {
    /// Converts this parameter range into a `TokenStream` for use in generated code.
    fn into_token_stream(self) -> TokenStream {
        let min = self.min;
        let max = self.max;

        quote! {
            ParameterRange {
                min: #min,
                max: #max
            }
        }
    }
}

/// A node in the multi-noise biome source k-d tree, either a leaf (resolved biome)
/// or a branch (spatial partition with child nodes).
#[derive(Deserialize)]
#[serde(tag = "_type", rename_all = "lowercase")]
enum BiomeTree {
    /// A terminal node resolving to a specific biome.
    Leaf {
        /// The seven noise-parameter ranges defining this leaf's position in parameter space.
        parameters: [ParameterRange; 7],
        /// Namespaced biome resource location (e.g. `"minecraft:plains"`).
        biome: String,
    },
    /// An interior node that partitions parameter space and delegates to child nodes.
    Branch {
        /// The seven noise-parameter ranges bounding this branch's region.
        parameters: [ParameterRange; 7],
        /// Child nodes of this branch (leaves or further branches).
        #[serde(rename = "subTree")]
        nodes: Box<[Self]>,
    },
}

impl BiomeTree {
    /// Converts this biome tree node into a `TokenStream` for use in generated code.
    fn into_token_stream(self) -> TokenStream {
        match self {
            Self::Leaf { parameters, biome } => {
                let biome = format_ident!(
                    "{}",
                    biome
                        .strip_prefix("minecraft:")
                        .unwrap()
                        .to_shouty_snake_case()
                );
                let parameters = parameters.map(ParameterRange::into_token_stream);
                quote! {
                    BiomeTree::Leaf {
                        parameters: [#(#parameters),*],
                        biome: &Biome::#biome
                    }
                }
            }
            Self::Branch { parameters, nodes } => {
                let nodes = nodes
                    .into_iter()
                    .map(Self::into_token_stream)
                    .collect::<Vec<_>>();
                let parameters = parameters.map(ParameterRange::into_token_stream);
                quote! {
                    BiomeTree::Branch {
                        parameters: [#(#parameters),*],
                        nodes: &[#(#nodes),*]
                    }
                }
            }
        }
    }
}

/// Root container holding the overworld and nether multi-noise biome source trees.
#[derive(Deserialize)]
struct MultiNoiseBiomeSuppliers {
    /// Multi-noise k-d tree used to resolve biomes in the overworld dimension.
    overworld: BiomeTree,
    /// Multi-noise k-d tree used to resolve biomes in the nether dimension.
    nether: BiomeTree,
}

/// Generates the `TokenStream` for the `Biome` struct, its constants, lookup methods,
/// the multi-noise biome source trees, and the `BiomeTree` search implementation.
pub fn build() -> TokenStream {
    let biomes: BTreeMap<String, Biome> =
        serde_json::from_str(&fs::read_to_string("../assets/biome.json").unwrap())
            .expect("Failed to parse biome.json");
    let biome_trees: MultiNoiseBiomeSuppliers =
        serde_json::from_str(&fs::read_to_string("../assets/multi_noise_biome_tree.json").unwrap())
            .expect("Failed to parse multi_noise_biome_tree.json");

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();
    let mut id_to_type = TokenStream::new();

    for (name, biome) in biomes {
        // let full_name = format!("minecraft:{name}");
        let format_name = format_ident!("{}", name.to_shouty_snake_case());
        let has_precipitation = biome.has_precipitation;
        let temperature = biome.temperature;
        let downfall = biome.downfall;
        //  let carvers = &biome.carvers;
        let features = &biome.features;
        let creature_spawn_probability = &biome.creature_spawn_probability.unwrap_or(0.1);

        let temperature_modifier = biome
            .temperature_modifier
            .unwrap_or(TemperatureModifier::None);

        let monster: Vec<_> = biome
            .spawners
            .monster
            .iter()
            .map(Spawner::to_tokens)
            .collect();
        let ambient: Vec<_> = biome
            .spawners
            .ambient
            .iter()
            .map(Spawner::to_tokens)
            .collect();
        let axolotls: Vec<_> = biome
            .spawners
            .axolotls
            .iter()
            .map(Spawner::to_tokens)
            .collect();
        let creature: Vec<_> = biome
            .spawners
            .creature
            .iter()
            .map(Spawner::to_tokens)
            .collect();
        let misc: Vec<_> = biome.spawners.misc.iter().map(Spawner::to_tokens).collect();
        let underground_water_creature: Vec<_> = biome
            .spawners
            .underground_water_creature
            .iter()
            .map(Spawner::to_tokens)
            .collect();
        let water_ambient: Vec<_> = biome
            .spawners
            .water_ambient
            .iter()
            .map(Spawner::to_tokens)
            .collect();
        let water_creature: Vec<_> = biome
            .spawners
            .water_creature
            .iter()
            .map(Spawner::to_tokens)
            .collect();

        let spawners = quote! {
            SpawnGroups {
                monster: &[#(#monster),*],
                ambient: &[#(#ambient),*],
                axolotls: &[#(#axolotls),*],
                creature: &[#(#creature),*],
                misc: &[#(#misc),*],
                underground_water_creature: &[#(#underground_water_creature),*],
                water_ambient: &[#(#water_ambient),*],
                water_creature: &[#(#water_creature),*],
            }
        };

        let spawn_costs: Vec<_> = biome
            .spawn_costs
            .iter()
            .map(|(name, cost)| {
                let cost_token = cost.to_tokens();
                let entity_type = name.strip_prefix("minecraft:").unwrap();
                quote! {
                    #entity_type => #cost_token
                }
            })
            .collect();

        let temperature_modifier = match temperature_modifier {
            TemperatureModifier::Frozen => quote! { TemperatureModifier::Frozen },
            TemperatureModifier::None => quote! { TemperatureModifier::None },
        };
        let index = LitInt::new(&biome.id.to_string(), Span::call_site());

        variants.extend([quote! {
            pub const #format_name: Biome = Biome {
                id: #index,
                registry_id: #name,
                weather: Weather::new(
                     #has_precipitation,
                     #temperature,
                     #temperature_modifier,
                     #downfall
                ),
                features: &[#(&[#(#features),*]),*],
                creature_spawn_probability: #creature_spawn_probability,
                spawners: #spawners,
                spawn_costs: phf::phf_map! {
                    #(#spawn_costs),*
                },
            };
        }]);

        name_to_type.extend(quote! { #name => Some(&Self::#format_name), });
        id_to_type.extend(quote! { #index => Some(&Self::#format_name), });
    }

    let overworld_tree = biome_trees.overworld.into_token_stream();
    let nether_tree = biome_trees.nether.into_token_stream();
    quote! {
        use crate::biome::de::Deserialize;
        use crate::entity_type::EntityType;
        use crate::tag::Taggable;
        use crate::tag::RegistryKey;
        use pumpkin_util::biome::{TemperatureModifier, Weather};
        use serde::{Deserializer, de};
        use std::{fmt, hash::{Hasher, Hash}};

        #[derive(Debug)]
        pub struct Biome {
            pub id: u8,
            pub registry_id: &'static str,
            pub weather: Weather,
            // carvers: &'static [&str],
            pub features: &'static [&'static [&'static str]],
            pub creature_spawn_probability: f32,
            pub spawners: SpawnGroups,
            pub spawn_costs: phf::Map<&'static str, SpawnCosts>,
        }

        impl PartialEq<u8> for Biome {
            fn eq(&self, other: &u8) -> bool {
                self.id == *other
            }
        }

        impl PartialEq<Biome> for u8 {
            fn eq(&self, other: &Biome) -> bool {
                *self == other.id
            }
        }

        #[derive(Debug)]
        pub struct SpawnGroups {
            pub monster: &'static [Spawner],
            pub ambient: &'static [Spawner],
            pub axolotls: &'static [Spawner],
            pub creature: &'static [Spawner],
            pub misc: &'static [Spawner],
            pub underground_water_creature: &'static [Spawner],
            pub water_ambient: &'static [Spawner],
            pub water_creature: &'static [Spawner],
        }

        #[derive(Debug)]
        pub struct Spawner {
            pub r#type: &'static str,
            pub min_count: i32,
            pub max_count: i32,
        }

        impl PartialEq for Biome {
            fn eq(&self, other: &Biome) -> bool {
                self.id == other.id
            }
        }

        impl Eq for Biome {}

        impl Hash for Biome {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }

        #[derive(Debug)]
        pub struct SpawnCosts {
            pub energy_budget: f64,
            pub charge: f64,
        }

        impl<'de> Deserialize<'de> for &'static Biome {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct BiomeVisitor;

                impl de::Visitor<'_> for BiomeVisitor {
                    type Value = &'static Biome;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a biome name as a string")
                    }

                    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
                        self.visit_str(&v)
                    }

                    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                        let biome = Biome::from_name(value.strip_prefix("minecraft:").unwrap_or(value));
                        biome.ok_or_else(|| E::unknown_variant(value, &["unknown biome"]))
                    }
                }

                deserializer.deserialize_str(BiomeVisitor)
            }
        }

        impl Biome {
            #variants

            pub fn from_name(name: &str) -> Option<&'static Self> {
                match name {
                    #name_to_type
                    _ => None
                }
            }

            pub const fn from_id(id: u8) -> Option<&'static Self> {
                match id {
                    #id_to_type
                    _ => None
                }
            }
        }

        impl Taggable for Biome {
            #[inline]
            fn registry_id(&self) -> u16 {
                self.id as u16
            }
            #[inline]
            fn tag_key() -> RegistryKey {
                RegistryKey::WorldgenBiome
            }
            #[inline]
            fn registry_key(&self) -> &str {
                self.registry_id
            }
        }

        #[derive(PartialEq)]
        pub struct ParameterRange {
            min: i64,
            max: i64,
        }

        impl ParameterRange {
            pub fn calc_distance(&self, noise: i64) -> i64 {
                if noise > self.max {
                    noise - self.max
                } else if noise < self.min {
                    self.min - noise
                } else {
                    0
                }
            }
        }

        #[derive(PartialEq)]
        pub enum BiomeTree {
            Leaf {
                parameters: [ParameterRange; 7],
                biome: &'static Biome,
            },
            Branch {
                parameters: [ParameterRange; 7],
                nodes: &'static [BiomeTree],
            },
        }


        impl BiomeTree {
           pub fn get(
                &'static self,
                point_list: &[i64; 7],
                previous_result_node: &mut Option<&'static BiomeTree>,
            ) -> &'static Biome {
                // Initialize best distance from the previous result for spatial coherence
                let mut best_dist = previous_result_node
                    .map(|node| node.get_squared_distance(point_list))
                    .unwrap_or(i64::MAX);

                let mut best_node = previous_result_node.unwrap_or(self);

                self.search(point_list, &mut best_dist, &mut best_node);

                match best_node {
                    BiomeTree::Leaf { biome, .. } => {
                        *previous_result_node = Some(best_node);
                        biome
                    }
                    // Should not happen with valid tree data
                    _ => unreachable!("Biome search failed to find a leaf"),
                }
            }

            fn search(
                &'static self,
                point: &[i64; 7],
                best_dist: &mut i64,
                best_node: &mut &'static BiomeTree,
            ) {
                let dist = self.get_squared_distance(point);

                // PRUNING: If this branch/leaf is further than our best, skip it and all children
                if dist >= *best_dist {
                    return;
                }

                match self {
                    Self::Leaf { .. } => {
                        *best_dist = dist;
                        *best_node = self;
                    }
                    Self::Branch { nodes, .. } => {
                        for node in *nodes {
                            node.search(point, best_dist, best_node);
                        }
                    }
                }
            }

            #[inline(always)]
            fn get_squared_distance(&self, p: &[i64; 7]) -> i64 {
                let params = match self {
                    Self::Leaf { parameters, .. } => parameters,
                    Self::Branch { parameters, .. } => parameters,
                };

                // Fully unrolled for 7 dimensions to maximize throughput
                params[0].calc_distance(p[0]) +
                params[1].calc_distance(p[1]) +
                params[2].calc_distance(p[2]) +
                params[3].calc_distance(p[3]) +
                params[4].calc_distance(p[4]) +
                params[5].calc_distance(p[5]) +
                params[6].calc_distance(p[6])
            }
        }

        pub const OVERWORLD_BIOME_SOURCE: BiomeTree = #overworld_tree;
        pub const NETHER_BIOME_SOURCE: BiomeTree = #nether_tree;
    }
}
