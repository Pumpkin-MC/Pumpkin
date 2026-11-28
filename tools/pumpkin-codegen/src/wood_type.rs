use std::fs;

use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct WoodTypeEntry {
    name: String,
    set_type: String,
    sound_type: String,
    hanging_sign_sound_type: String,
    fence_gate_close: String,
    fence_gate_open: String,
}

fn sound_ident(path: &str) -> proc_macro2::Ident {
    format_ident!("{}", path.to_pascal_case())
}

pub fn build() -> TokenStream {
    let entries: Vec<WoodTypeEntry> =
        serde_json::from_str(&fs::read_to_string("../../assets/wood_types.json").unwrap())
            .expect("Failed to parse wood_types.json");

    let mut constants = TokenStream::new();

    for entry in &entries {
        let name = &entry.name;
        let const_name = format_ident!("{}", name.to_shouty_snake_case());
        let set_type = format_ident!("{}", entry.set_type.to_shouty_snake_case());
        let sound_type = &entry.sound_type;
        let hanging_sign_sound_type = &entry.hanging_sign_sound_type;
        let fence_gate_close = sound_ident(&entry.fence_gate_close);
        let fence_gate_open = sound_ident(&entry.fence_gate_open);

        constants.extend(quote! {
            pub const #const_name: WoodType = WoodType {
                name: #name,
                set_type: &crate::block_set_type::BlockSetType::#set_type,
                sound_type: #sound_type,
                hanging_sign_sound_type: #hanging_sign_sound_type,
                fence_gate_close: Sound::#fence_gate_close,
                fence_gate_open: Sound::#fence_gate_open,
            };
        });
    }

    quote! {
        /* This file is generated. Do not edit manually. */

        use crate::sound::Sound;

        /// A vanilla wood type, shared by fence gates and signs.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct WoodType {
            pub name: &'static str,
            pub set_type: &'static crate::block_set_type::BlockSetType,
            /// The vanilla `SoundType` constant name.
            pub sound_type: &'static str,
            /// The vanilla `SoundType` constant name for hanging signs.
            pub hanging_sign_sound_type: &'static str,
            pub fence_gate_close: Sound,
            pub fence_gate_open: Sound,
        }

        impl WoodType {
            #constants
        }
    }
}
