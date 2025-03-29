use std::collections::HashMap;

use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use syn::LitInt;

#[derive(Deserialize)]
pub struct Biome {
    has_precipitation: bool,
    temperature: f32,
    downfall: f32,
    temperature_modifier: Option<TemperatureModifier>,
    //carvers: Vec<String>,
    features: Vec<Vec<String>>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TemperatureModifier {
    None,
    Frozen,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/biome.json");

    let biomes: HashMap<String, Biome> =
        serde_json::from_str(include_str!("../../assets/biome.json"))
            .expect("Failed to parse biome.json");
    let mut variants = TokenStream::new();
    let mut type_to_name = TokenStream::new();
    let mut name_to_type = TokenStream::new();
    let mut type_to_id = TokenStream::new();
    let mut id_to_type = TokenStream::new();

    for (i, (name, biome)) in biomes.iter().enumerate() {
        // let full_name = format!("minecraft:{name}");
        let format_name = format_ident!("{}", name.to_shouty_snake_case());
        let has_precipitation = biome.has_precipitation;
        let temperature = biome.temperature;
        let downfall = biome.downfall;
        //  let carvers = &biome.carvers;
        let features = &biome.features;
        let temperature_modifier = biome
            .temperature_modifier
            .clone()
            .unwrap_or(TemperatureModifier::None);
        let temperature_modifier = match temperature_modifier {
            TemperatureModifier::Frozen => quote! { TemperatureModifier::Frozen },
            TemperatureModifier::None => quote! { TemperatureModifier::None },
        };
        let index = LitInt::new(&i.to_string(), Span::call_site());

        variants.extend([quote! {
            pub const #format_name: Biome = Biome {
               index: #index,
               has_precipitation: #has_precipitation,
               temperature: #temperature,
               temperature_modifier: #temperature_modifier,
               downfall: #downfall,
              features: &[#(&[#(#features),*]),*]
            };
        }]);

        type_to_name.extend(quote! { Self::#format_name => #name, });
        name_to_type.extend(quote! { #name => Some(Self::#format_name), });
        type_to_id.extend(quote! { Self::#format_name => #index, });
        id_to_type.extend(quote! { #index => Some(Self::#format_name), });
    }

    quote! {
        use serde::{de, Deserializer};
        use std::fmt;

        #[derive(Clone, Copy, PartialEq, Debug)]
        pub struct Biome {
            pub index: u8,
            pub has_precipitation: bool,
            pub temperature: f32,
            pub temperature_modifier: TemperatureModifier,
            pub downfall: f32,
           // carvers: &'static [&str],
            pub features: &'static [&'static [&'static str]]
        }

        impl<'de> Deserialize<'de> for Biome {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct BiomeVisitor;

                impl de::Visitor<'_> for BiomeVisitor {
                    type Value = Biome;

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
                        let biome = Biome::from_name(&value.replace("minecraft:", ""));

                        biome.ok_or_else(|| E::unknown_variant(value, &["unknown biome"]))
                    }
                }

                deserializer.deserialize_str(BiomeVisitor)
            }
        }

        #[derive(Clone, Copy, PartialEq, Debug)]
        pub enum TemperatureModifier {
            None,
            Frozen
        }

        impl Biome {
            #variants

            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #name_to_type
                    _ => None
                }
            }

            pub const fn from_id(id: u16) -> Option<Self> {
                match id {
                    #id_to_type
                    _ => None
                }
            }
        }
    }
}
