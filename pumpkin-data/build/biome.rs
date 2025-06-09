use std::{collections::HashMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use syn::LitInt;

#[derive(Deserialize)]
struct Biome {
    has_precipitation: bool,
    temperature: f32,
    downfall: f32,
    temperature_modifier: Option<TemperatureModifier>,
    //carvers: Vec<String>,
    features: Vec<Vec<String>>,
    creature_spawn_probability: Option<f32>,
    spawners: SpawnGroups,
    id: u8,
}

#[derive(Deserialize, PartialEq, Eq, Hash)]
struct SpawnGroups {
    monster: Vec<Spawner>,
    ambient: Vec<Spawner>,
    axolotls: Vec<Spawner>,
    creature: Vec<Spawner>,
    misc: Vec<Spawner>,
    underground_water_creature: Vec<Spawner>,
    water_ambient: Vec<Spawner>,
    water_creature: Vec<Spawner>,
}

#[derive(Deserialize, Hash, PartialEq, Eq)]
struct Spawner {
    r#type: String,
}

impl Spawner {
    pub fn to_tokens(&self) -> TokenStream {
        let r#type = &self.r#type;
        quote! {
            Spawner {
                r#type: #r#type,
            }
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
enum TemperatureModifier {
    None,
    Frozen,
}

#[derive(Deserialize)]
struct ParameterRange {
    min: i64,
    max: i64,
}

impl ParameterRange {
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

#[derive(Deserialize)]
#[serde(tag = "_type", rename_all = "lowercase")]
enum BiomeTree {
    Leaf {
        parameters: [ParameterRange; 7],
        biome: String,
    },
    Branch {
        parameters: [ParameterRange; 7],
        #[serde(rename = "subTree")]
        nodes: Box<[BiomeTree]>,
    },
}

impl BiomeTree {
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
                let parameters = parameters.map(|range| range.into_token_stream());
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
                    .map(|node| node.into_token_stream())
                    .collect::<Vec<_>>();
                let parameters = parameters.map(|range| range.into_token_stream());
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

#[derive(Deserialize)]
struct MultiNoiseBiomeSuppliers {
    overworld: BiomeTree,
    nether: BiomeTree,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/biome.json");
    println!("cargo:rerun-if-changed=../assets/multi_noise_biome_tree.json");

    let biomes: HashMap<String, Biome> =
        serde_json::from_str(&fs::read_to_string("../assets/biome.json").unwrap())
            .expect("Failed to parse biome.json");
    let biome_trees: MultiNoiseBiomeSuppliers =
        serde_json::from_str(&fs::read_to_string("../assets/multi_noise_biome_tree.json").unwrap())
            .expect("Failed to parse multi_noise_biome_tree.json");

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();
    let mut id_to_type = TokenStream::new();

    for (name, biome) in biomes.iter() {
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
            .clone()
            .unwrap_or(TemperatureModifier::None);

        let monster: Vec<_> = biome
            .spawners
            .monster
            .iter()
            .map(|s| s.to_tokens())
            .collect();
        let ambient: Vec<_> = biome
            .spawners
            .ambient
            .iter()
            .map(|s| s.to_tokens())
            .collect();
        let axolotls: Vec<_> = biome
            .spawners
            .axolotls
            .iter()
            .map(|s| s.to_tokens())
            .collect();
        let creature: Vec<_> = biome
            .spawners
            .creature
            .iter()
            .map(|s| s.to_tokens())
            .collect();
        let misc: Vec<_> = biome.spawners.misc.iter().map(|s| s.to_tokens()).collect();
        let underground_water_creature: Vec<_> = biome
            .spawners
            .underground_water_creature
            .iter()
            .map(|s| s.to_tokens())
            .collect();
        let water_ambient: Vec<_> = biome
            .spawners
            .water_ambient
            .iter()
            .map(|s| s.to_tokens())
            .collect();
        let water_creature: Vec<_> = biome
            .spawners
            .water_creature
            .iter()
            .map(|s| s.to_tokens())
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
            };
        }]);

        name_to_type.extend(quote! { #name => Some(&Self::#format_name), });
        id_to_type.extend(quote! { #index => Some(&Self::#format_name), });
    }

    let overworld_tree = biome_trees.overworld.into_token_stream();
    let nether_tree = biome_trees.nether.into_token_stream();
    quote! {

            use pumpkin_util::biome::{TemperatureModifier, Weather};
            use serde::{Deserializer, de};
            use crate::biome::de::Deserialize;
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
            }

            #[derive(Debug)]
            pub struct SpawnGroups {
           pub monster: &'static [Spawner],
          pub  ambient: &'static [Spawner],
           pub axolotls: &'static [Spawner],
          pub  creature: &'static [Spawner],
          pub  misc: &'static [Spawner],
          pub  underground_water_creature: &'static [Spawner],
          pub  water_ambient: &'static [Spawner],
          pub  water_creature: &'static [Spawner],
            }

            #[derive(Debug)]
    pub struct Spawner {
       pub  r#type: &'static str,
    }

            impl PartialEq for Biome {
                fn eq(&self, other: &Biome) -> bool {
                    self.id == other.id
                }
            }

            impl Eq for Biome {}

            impl Hash for Biome {
                fn hash<H>(&self, state: &mut H) where H: Hasher {
                    self.id.hash(state);
                }
            }

            impl<'de> Deserialize<'de> for &'static Biome {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct BiomeVisitor;

                    impl de::Visitor<'_> for BiomeVisitor {
                        type Value = &'static Biome;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str("a biome name as a string")
                        }

                        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            self.visit_str(&v)
                        }

                        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
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

            #[derive(PartialEq)]
            pub struct ParameterRange {
                min: i64,
                max: i64,
            }

            impl ParameterRange {
                fn calc_distance(&self, noise: i64) -> i64 {
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
                    let result_node = self.get_resulting_node(point_list, *previous_result_node);
                    match result_node {
                        BiomeTree::Leaf { biome, .. } => {
                            *previous_result_node = Some(result_node);
                            biome
                        }
                        _ => unreachable!(),
                    }
                }

                fn get_resulting_node(
                    &'static self,
                    point_list: &[i64; 7],
                    previous_result_node: Option<&'static BiomeTree>,
                ) -> &'static BiomeTree {
                    match self {
                        Self::Leaf { .. } => self,
                        Self::Branch { nodes, .. } => {
                            let mut distance = previous_result_node
                                .map(|node| node.get_squared_distance(point_list))
                                .unwrap_or(i64::MAX);
                            let mut best_node = previous_result_node;

                            for node in *nodes {
                                let node_distance = node.get_squared_distance(point_list);
                                if distance > node_distance {
                                    let node2 = node.get_resulting_node(point_list, best_node);
                                    let node2_distance = if node == node2 {
                                        node_distance
                                    } else {
                                        node2.get_squared_distance(point_list)
                                    };

                                    if distance > node2_distance {
                                        distance = node2_distance;
                                        best_node = Some(node2);
                                    }
                                }
                            }

                            best_node.expect("This should be populated after traversing the tree")
                        }
                    }
                }

                fn get_squared_distance(&self, point_list: &[i64; 7]) -> i64 {
                    let parameters = match self {
                        Self::Leaf { parameters, .. } => parameters,
                        Self::Branch { parameters, .. } => parameters,
                    };

                    parameters
                        .iter()
                        .zip(point_list)
                        .map(|(bound, value)| {
                            let distance = bound.calc_distance(*value);
                            distance * distance
                        })
                        .sum()
                }
            }

            pub const OVERWORLD_BIOME_SOURCE: BiomeTree = #overworld_tree;
            pub const NETHER_BIOME_SOURCE: BiomeTree = #nether_tree;
        }
}
