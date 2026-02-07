use std::{collections::BTreeMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use pumpkin_util::text::TextComponent;
use quote::{format_ident, quote, ToTokens};
use serde::Deserialize;
use pumpkin_util::resource_location::ResourceLocation;

#[derive(Deserialize)]
pub struct Advancement {
    pub parent : Option<ResourceLocation>,
    // pub display : Option<AdvancementDisplay>,
    // pub criteria : Vec<AdvancementCriterion>,
    // pub rewards : AdvancementRewards,
    pub send_telemetry : bool,
    pub name : Option<TextComponent>
}

impl Advancement {
    fn to_tokens(&self) {
        let parent = match &self.parent {
            Some(parent) => quote! { Some(#parent) },
            None => quote! { None }
        };
        let send_telemetry = self.send_telemetry;
        let name = match &self.name {
            Some(name) => quote! { #name },
            None => quote! { None }
        };

    }
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/advancements.json");

    let advancements: BTreeMap<String, Advancement> =
        serde_json::from_str(&fs::read_to_string("../assets/advancements.json").unwrap())
            .expect("Failed to parse advancements.json");

    for (name, advancement) in advancements {
        let raw_name = name.strip_prefix("minecraft:").unwrap();
        let format_name = format_ident!("{}", raw_name.to_shouty_snake_case());

    }

    quote! {
        pub struct Advancements {
            pub parent : Option<ResourceLocation>,
            pub send_telemetry : bool,
            pub name : Option<TextComponent>
        }


    }
}
