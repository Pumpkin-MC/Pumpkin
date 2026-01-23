use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use std::fs;

pub fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/jukebox_song.json");

    let songs: BTreeMap<String, u32> = serde_json::from_str(
        &fs::read_to_string("../assets/jukebox_song.json").expect("Missing jukebox_song.json"),
    )
    .expect("Failed to parse jukebox_song.json");

    // Helper to handle numeric keys like "11" -> "Id11"
    let make_variant_ident = |name: &str| {
        let pascal = name.to_pascal_case();
        if pascal.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            format_ident!("Id{}", pascal)
        } else {
            format_ident!("{}", pascal)
        }
    };

    let variants = songs
        .keys()
        .map(|name| {
            let variant_name = make_variant_ident(name);
            quote! { #variant_name, }
        })
        .collect::<TokenStream>();

    let type_from_name = songs
        .keys()
        .map(|name| {
            let variant_name = make_variant_ident(name);
            quote! { #name => Some(Self::#variant_name), }
        })
        .collect::<TokenStream>();

    let type_to_name = songs
        .keys()
        .map(|name| {
            let variant_name = make_variant_ident(name);
            quote! { Self::#variant_name => #name, }
        })
        .collect::<TokenStream>();

    let type_to_id = songs
        .iter()
        .map(|(name, id)| {
            let variant_name = make_variant_ident(name);
            quote! { Self::#variant_name => #id, }
        })
        .collect::<TokenStream>();

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u32)]
        pub enum JukeboxSong {
            #variants
        }

        impl JukeboxSong {
            #[doc = r" Returns the JukeboxSong from the string name (e.g., 'pigstep')."]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #type_from_name
                    _ => None
                }
            }

            #[doc = r" Returns the string name of the song."]
            pub const fn to_name(&self) -> &'static str {
                match self {
                    #type_to_name
                }
            }

            #[doc = r" Returns the numeric ID associated with the song."]
            pub const fn get_id(&self) -> u32 {
                match self {
                    #type_to_id
                }
            }
        }
    }
}
