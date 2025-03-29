use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct Weather {
    has_precipitation: bool,
    temperature: f32,
    temperature_modifier: String,
    downfall: f32,
}

#[derive(Deserialize)]
struct BiomeData {
    name: String,
    id: u16,
    weather: Weather,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/biome.json");

    let biomes: Vec<BiomeData> = serde_json::from_str(include_str!("../../assets/biome.json"))
        .expect("Failed to parse biome.json");
    let mut variants = TokenStream::new();
    let mut weathers = TokenStream::new();
    let mut to_id = TokenStream::new();
    let mut from_id = TokenStream::new();
    let mut weather_map = TokenStream::new();

    for data in biomes.into_iter() {
        let name = data.name;
        let id = data.id;
        let full_name = format!("minecraft:{name}");

        let weather_name = format_ident!("{}_WEATHER", name.to_uppercase());
        let name = format_ident!("{}", name.to_pascal_case());

        let has_precipitation = data.weather.has_precipitation;
        let temperature = data.weather.temperature;
        let variant = data.weather.temperature_modifier.to_pascal_case();
        let modifier_variant = format_ident!("{}", variant);
        let downfall = data.weather.downfall;

        weathers.extend([quote! {
            const #weather_name : Weather = Weather::new(
                #has_precipitation,
                #temperature,
                TemperatureModifier::#modifier_variant,
                #downfall
            );
        }]);

        variants.extend([quote! {
            #[serde(rename = #full_name)]
            #name,
        }]);
        to_id.extend([quote! {
            Self::#name => #id,
        }]);
        from_id.extend([quote! {
            #id => Self::#name,
        }]);
        weather_map.extend([quote! {
            Self::#name => &#weather_name,
        }]);
    }
    quote! {
        use pumpkin_util::biome::{TemperatureModifier, Weather};

        #weathers

        #[derive(Clone, Deserialize, Copy, Hash, PartialEq, Eq, Debug)]
        pub enum Biome {
            #variants
        }

        impl Biome {
            pub fn to_id(&self) -> u16 {
                match self {
                    #to_id
                }
            }

            pub fn weather(&self) -> &Weather {
                match self {
                    #weather_map
                }
            }

            pub fn from_id(id: u16) -> Biome {
                match id {
                    #from_id
                    _ => panic!("Unknown biome id: {}", id)
                }
            }
        }
    }
}
