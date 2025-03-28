use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/biome.json");

    let biomes: Vec<String> = serde_json::from_str(include_str!("../../assets/biome.json"))
        .expect("Failed to parse biome.json");
    let mut variants = TokenStream::new();

    for status in biomes.iter() {
        let full_name = format!("minecraft:{status}");
        let name = format_ident!("{}", status.to_pascal_case());
        variants.extend([quote! {
            #[serde(rename = #full_name)]
            #name,
        }]);
    }

    let type_to_name = &biomes
        .iter()
        .map(|sound| {
            let id = &sound;
            let name = format_ident!("{}", sound.to_pascal_case());

            quote! {
                Self::#name => #id,
            }
        })
        .collect::<TokenStream>();

    let type_to_id = &biomes
        .iter()
        .enumerate()
        .map(|(i, sound)| {
            let name = format_ident!("{}", sound.to_pascal_case());

            quote! {
                Self::#name => #i as u8,
            }
        })
        .collect::<TokenStream>();

    quote! {
        #[derive(Clone, Deserialize, Copy, Hash, PartialEq, Eq, Debug)]
        pub enum Biome {
            #variants
        }

        impl Biome {
            pub const fn to_name(&self) -> &'static str {
                match self {
                    #type_to_name
                }
            }

            pub const fn to_id(&self) -> u8 {
                match self {
                    #type_to_id
                }
            }
        }
    }
}
