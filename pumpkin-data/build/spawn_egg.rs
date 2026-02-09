use std::{collections::BTreeMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/spawn_egg.json");

    let eggs: BTreeMap<u16, String> =
        serde_json::from_str(&fs::read_to_string("../assets/spawn_egg.json").unwrap())
            .expect("Failed to parse spawn_egg.json");
    let mut names = TokenStream::new();
    let mut ids = TokenStream::new();

    for (egg, entity) in &eggs {
        let entity = entity.to_shouty_snake_case();
        let entity = format_ident!("{}", entity);
        ids.extend(quote! { #egg, });
        names.extend(quote! { #egg => Some(&EntityType::#entity), });
    }
    quote! {
    use crate::entity_type::EntityType;

    pub fn entity_from_egg(id: u16) -> Option<&'static EntityType> {
        match id {
            #names
            _ => None
        }
    }
    pub fn spawn_egg_ids() -> Box<[u16]> {
        [#ids].into()
    }
    }
}
