use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use std::fs;

pub(crate) fn build() -> TokenStream {
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

    // Vanilla song durations in seconds from JukeboxSongs.java
    let song_lengths: BTreeMap<&str, u32> = [
        ("13", 178),
        ("cat", 185),
        ("blocks", 345),
        ("chirp", 185),
        ("far", 174),
        ("mall", 197),
        ("mellohi", 96),
        ("stal", 150),
        ("strad", 188),
        ("ward", 251),
        ("11", 71),
        ("wait", 238),
        ("pigstep", 149),
        ("otherside", 195),
        ("5", 178),
        ("relic", 218),
        ("precipice", 299),
        ("creator", 176),
        ("creator_music_box", 73),
        ("tears", 175),
        ("lava_chicken", 134),
    ]
    .into_iter()
    .collect();

    let type_to_length = songs
        .keys()
        .map(|name| {
            let variant_name = make_variant_ident(name);
            let length = song_lengths.get(name.as_str()).copied().unwrap_or(0);
            quote! { Self::#variant_name => #length, }
        })
        .collect::<TokenStream>();

    // Vanilla comparator output values from JukeboxSongs.java
    let comparator_outputs: BTreeMap<&str, u8> = [
        ("13", 1),
        ("cat", 2),
        ("blocks", 3),
        ("chirp", 4),
        ("far", 5),
        ("mall", 6),
        ("mellohi", 7),
        ("stal", 8),
        ("strad", 9),
        ("ward", 10),
        ("11", 11),
        ("wait", 12),
        ("pigstep", 13),
        ("otherside", 14),
        ("5", 15),
        ("relic", 14),
        ("precipice", 13),
        ("creator", 12),
        ("creator_music_box", 11),
        ("tears", 10),
        ("lava_chicken", 9),
    ]
    .into_iter()
    .collect();

    let type_to_comparator = songs
        .keys()
        .map(|name| {
            let variant_name = make_variant_ident(name);
            let output = comparator_outputs.get(name.as_str()).copied().unwrap_or(0);
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
