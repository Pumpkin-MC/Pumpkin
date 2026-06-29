use heck::ToPascalCase;
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct CustomStatisticEntry {
    id: i32,
}

#[derive(Deserialize)]
struct StatisticData {
    id: i32,
    registry: String,
    entries: IndexMap<String, CustomStatisticEntry>,
}

pub fn build() -> TokenStream {
    let stats_json =
        std::fs::read_to_string("../assets/stats.json").expect("Failed to read stats.json");
    let stats_data: IndexMap<String, StatisticData> =
        serde_json::from_str(&stats_json).expect("Failed to parse stats.json");

    let mut category_variants = Vec::new();
    let mut category_from_id_arms = Vec::new();
    let mut category_from_name_arms = Vec::new();
    let mut custom_stat_variants = Vec::new();
    let mut custom_stat_from_id_arms = Vec::new();

    for (category_name, data) in &stats_data {
        let category_ident = format_ident!(
            "{}",
            category_name.replace("minecraft:", "").to_pascal_case()
        );
        let category_id = data.id;
        category_variants.push(quote! { #category_ident = #category_id });
        category_from_id_arms.push(quote! { #category_id => Some(Self::#category_ident) });
        let category_name_stripped = category_name.replace("minecraft:", "");
        category_from_name_arms
            .push(quote! { #category_name_stripped => Some(Self::#category_ident) });

        if category_name == "minecraft:custom" {
            for (stat_name, entry) in &data.entries {
                let stat_ident =
                    format_ident!("{}", stat_name.replace("minecraft:", "").to_pascal_case());
                let stat_id = entry.id;
                custom_stat_variants.push(quote! { #stat_ident = #stat_id });
                custom_stat_from_id_arms.push(quote! { #stat_id => Some(Self::#stat_ident) });
            }
        }
    }

    quote! {
        use serde::{Serialize, Deserialize};

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[repr(i32)]
        pub enum StatisticCategory {
            #(#category_variants,)*
        }

        impl StatisticCategory {
            #[must_use]
            pub const fn from_i32(id: i32) -> Option<Self> {
                match id {
                    #(#category_from_id_arms,)*
                    _ => None,
                }
            }

            #[must_use]
            pub fn from_registry_key(name: &str) -> Option<Self> {
                let name = name.strip_prefix("minecraft:").unwrap_or(name);
                match name {
                    #(#category_from_name_arms,)*
                    _ => None,
                }
            }
        }

        impl TryFrom<i32> for StatisticCategory {
            type Error = ();

            fn try_from(value: i32) -> Result<Self, Self::Error> {
                Self::from_i32(value).ok_or(())
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[repr(i32)]
        pub enum CustomStatistic {
            #(#custom_stat_variants,)*
        }

        impl CustomStatistic {
            #[must_use]
            pub const fn from_i32(id: i32) -> Option<Self> {
                match id {
                    #(#custom_stat_from_id_arms,)*
                    _ => None,
                }
            }
        }

        impl TryFrom<i32> for CustomStatistic {
            type Error = ();

            fn try_from(value: i32) -> Result<Self, Self::Error> {
                Self::from_i32(value).ok_or(())
            }
        }
    }
}
