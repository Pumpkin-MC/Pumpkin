use std::{collections::BTreeMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use pumpkin_util::text::TextComponent;
use quote::{format_ident, quote, ToTokens};
use serde::Deserialize;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextContent::Translate;

#[derive(Deserialize,Default)]
pub struct Advancement {
    pub parent : Option<ResourceLocation>,
    // pub display : Option<AdvancementDisplay>,
    // pub criteria : Vec<AdvancementCriterion>,
    // pub rewards : AdvancementRewards,
    #[serde(default)]
    pub send_telemetry : bool,
    pub display_name : Option<TextComponent>
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/advancements.json");

    let advancements: BTreeMap<String, Advancement> =
        serde_json::from_str(&fs::read_to_string("../assets/advancements.json").unwrap())
            .expect("Failed to parse advancements.json");

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();
    let mut minecraft_name_to_type = TokenStream::new();

    for (minecraft_name, advancement) in advancements {
        let raw_name = minecraft_name.strip_prefix("minecraft:").unwrap();
        let format_name = format_ident!("{}", raw_name.to_shouty_snake_case());

        let parent = match &advancement.parent {
            Some(parent) => quote! { Some(#parent) },
            None => quote! { None }
        };
        let send_telemetry = advancement.send_telemetry;
        let display_name = match &advancement.display_name {
            Some(name) => {
                let Translate { translate, with: _ } = &name.0.content else { panic!() };
                quote! { TextComponent::translate(#translate,[]) }
            }
            None => quote! { None }
        };
        variants.extend([quote! {
            pub const #format_name: Self = Self {
                id: #raw_name,
                parent : #parent,
                send_telemetry : #send_telemetry,
                display_name : #display_name,
            };
        }]);
        name_to_type.extend(quote! { #raw_name => Some(&Self::#format_name), });
        minecraft_name_to_type.extend(quote! { #minecraft_name => Some(&Self::#format_name), });
    }



    quote! {
        pub struct Advancement {
            pub id : ResourceLocation,
            pub parent : Option<ResourceLocation>,
            pub send_telemetry : bool,
            pub display_name : Option<TextComponent>
        }

        impl Advancement {
            #variants

            pub fn from_name(name: &str) -> Option<&'static Self> {
                    match name {
                        #name_to_type
                        _ => None
                    }
                }

            pub fn from_minecraft_name(name: &str) -> Option<&'static Self> {
                match name {
                    #name_to_type
                    _ => None
                }
            }
        }
    }
}
