use heck::ToPascalCase;
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use serde_json::Value;
use std::fs;

#[derive(Deserialize)]
struct JukeboxSongData {
    length_in_seconds: f32,
    comparator_output: u8,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/jukebox_song.json");
    println!("cargo:rerun-if-changed=../assets/registry/1_21_11_synced_registries.json");

    let songs: IndexMap<String, u32> = serde_json::from_str(
        &fs::read_to_string("../assets/jukebox_song.json").expect("Missing jukebox_song.json"),
    )
    .expect("Failed to parse jukebox_song.json");

    let registries: IndexMap<String, Value> = serde_json::from_str(
        &fs::read_to_string("../assets/registry/1_21_11_synced_registries.json")
            .expect("Missing synced_registries.json"),
    )
    .expect("Failed to parse synced_registries.json");

    let song_data: IndexMap<String, JukeboxSongData> = serde_json::from_value(
        registries
            .get("jukebox_song")
            .expect("Missing jukebox_song in synced registries")
            .clone(),
    )
    .expect("Failed to parse jukebox_song data");

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

    let type_to_length = songs
        .keys()
        .map(|name| {
            let variant_name = make_variant_ident(name);
            let length = song_data
                .get(name)
                .map(|d| d.length_in_seconds as u32)
                .unwrap_or(0);
            quote! { Self::#variant_name => #length, }
        })
        .collect::<TokenStream>();

    let type_to_comparator = songs
        .keys()
        .map(|name| {
            let variant_name = make_variant_ident(name);
            let output = song_data
                .get(name)
                .map(|d| d.comparator_output)
                .unwrap_or(0);
            quote! { Self::#variant_name => #output, }
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

            #[doc = r" Returns the comparator output value (0-15) for this song."]
            pub const fn comparator_output(&self) -> u8 {
                match self {
                    #type_to_comparator
                }
            }

            #[doc = r" Returns the song length in seconds."]
            pub const fn length_in_seconds(&self) -> u32 {
                match self {
                    #type_to_length
                }
            }

            #[doc = r" Returns the song length in ticks (20 ticks per second)."]
            pub const fn length_in_ticks(&self) -> u64 {
                self.length_in_seconds() as u64 * 20
            }
        }
    }
}
