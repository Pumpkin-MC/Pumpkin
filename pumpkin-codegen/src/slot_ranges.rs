use proc_macro2::TokenStream;
use quote::quote;
use std::{collections::BTreeMap, fs};
/// Generates the `TokenStream` for some slot range constants and a function to get one from the name of a range.
pub fn build() -> TokenStream {
    let slot_ranges: BTreeMap<String, Box<[usize]>> =
        serde_json::from_str(&fs::read_to_string("../assets/slot_ranges.json").unwrap())
            .expect("Failed to parse slot_ranges.json");

    let slot_range_match_tokens = slot_ranges
        .iter()
        .map(|(slot_range_name, range)| {
            quote! {
                #slot_range_name => Some(&[#(#range),*]),
            }
        })
        .collect::<Vec<_>>();

    let slot_range_tuples_tokens = slot_ranges
        .iter()
        .map(|(slot_range_name, range)| {
            let range = &**range;
            quote! {
                (#slot_range_name, &[#(#range),*])
            }
        })
        .collect::<Vec<_>>();

    let slot_range_all_names_tokens = slot_ranges
        .iter()
        .map(|(slot_range_name, _)| {
            quote! { #slot_range_name }
        })
        .collect::<Vec<_>>();

    let slot_range_single_slot_names_tokens = slot_ranges
        .iter()
        .filter(|(_, range)| range.len() == 1)
        .map(|(slot_range_name, _)| {
            quote! { #slot_range_name }
        })
        .collect::<Vec<_>>();

    let slot_ranges_len = slot_ranges.len();
    let single_slot_names_len = slot_range_single_slot_names_tokens.len();

    quote! {
        pub const SLOT_RANGES: [(&str, &[usize]); #slot_ranges_len] = [
            #(#slot_range_tuples_tokens),*
        ];

        pub const SLOT_RANGE_ALL_NAMES: [&str; #slot_ranges_len] = [
            #(#slot_range_all_names_tokens),*
        ];

        pub const SLOT_RANGE_SINGLE_SLOT_NAMES: [&str; #single_slot_names_len] = [
            #(#slot_range_single_slot_names_tokens),*
        ];

        #[must_use]
        pub fn get_slot_range(name: &str) -> Option<&'static [usize]> {
            match name {
                #(#slot_range_match_tokens)*
                _ => None,
            }
        }
    }
}
